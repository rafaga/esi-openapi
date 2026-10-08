//! Optional proactive throttling against ESI's rate limits.
//!
//! ESI gives each application/character pair a token bucket per route group
//! over a floating window: a request spends tokens (`X-Ratelimit-Used`), and
//! those tokens come back one window later. Every response reports the tokens
//! left (`X-Ratelimit-Remaining`). With a [`RateLimitPolicy`] other than
//! [`RateLimitPolicy::Off`], the client keeps, for each group and access token,
//! the last reported balance plus a ledger of what its own responses spent, and
//! works out when the next request fits instead of provoking a `429`.
//!
//! Only operations that declare an `x-rate-limit` group in the spec are
//! throttled; the others are covered by the error limit. Until a response of a
//! group has been seen, nothing is known about its balance and requests go out.

use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    time::Duration,
};

/// What the client does when a route group has no tokens left for a request.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitPolicy {
    /// Do not track budgets; send every request (the default). A `429` from ESI
    /// is reported as [`EsiError::RateLimited`](crate::prelude::EsiError::RateLimited).
    #[default]
    Off,
    /// Sleep until the request fits. If that takes longer than `max_wait`, the
    /// request is not sent and [`EsiError::RateLimited`](crate::prelude::EsiError::RateLimited)
    /// says how many seconds to wait.
    Wait {
        /// The longest the client sleeps for a single request.
        max_wait: Duration,
    },
    /// Do not wait: return [`EsiError::RateLimited`](crate::prelude::EsiError::RateLimited)
    /// without calling ESI when the request does not fit.
    Fail,
}

/// Window length when ESI did not say one.
const DEFAULT_WINDOW_MS: i64 = 900_000;
/// Tokens a successful request costs until a response says otherwise.
const DEFAULT_COST: i64 = 2;
/// Added to a computed wait so the tokens are surely back by then.
const WAIT_MARGIN_MS: i64 = 50;

/// The balance of one route group for one access token.
#[derive(Debug)]
struct Bucket {
    window_ms: i64,
    /// Tokens left, as of the last response.
    remaining: i64,
    /// Tokens a request is expected to spend.
    cost: i64,
    last_response_ms: i64,
    /// When each response arrived and what it spent, oldest first.
    spent: VecDeque<(i64, i64)>,
    /// Requests sent whose response has not been seen yet.
    in_flight: i64,
    /// No request fits before this time (`Retry-After` of a `429`).
    blocked_until_ms: i64,
    /// Whether any response has been seen.
    seen: bool,
}

impl Default for Bucket {
    fn default() -> Self {
        Bucket {
            window_ms: DEFAULT_WINDOW_MS,
            remaining: 0,
            cost: DEFAULT_COST,
            last_response_ms: 0,
            spent: VecDeque::new(),
            in_flight: 0,
            blocked_until_ms: 0,
            seen: false,
        }
    }
}

impl Bucket {
    /// Milliseconds to wait from `now` until one more request fits (0 if it fits).
    fn wait_ms(&self, now: i64) -> i64 {
        if !self.seen {
            return 0;
        }
        let needed = (self.in_flight + 1) * self.cost;
        let mut available = self.remaining;
        let mut ready_at = now;
        if available < needed {
            // Tokens spent before the last response return as their window ends.
            ready_at = self.last_response_ms + self.window_ms;
            for (at, cost) in &self.spent {
                let returns_at = at + self.window_ms;
                if returns_at <= self.last_response_ms {
                    continue;
                }
                available += cost;
                if available >= needed {
                    ready_at = returns_at;
                    break;
                }
            }
        }
        let wait = (ready_at - now).max(self.blocked_until_ms - now).max(0);
        if wait > 0 {
            wait + WAIT_MARGIN_MS
        } else {
            0
        }
    }
}

/// The answer to a request for budget.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Acquire {
    /// The request fits; it is counted as in flight until its permit is dropped.
    Granted,
    /// It does not fit; try again in this many milliseconds.
    Wait(i64),
}

/// Budgets by `group|access token`.
#[derive(Debug, Default)]
pub(crate) struct RateLimiter {
    buckets: Mutex<HashMap<String, Bucket>>,
}

impl RateLimiter {
    fn lock(&self) -> MutexGuard<'_, HashMap<String, Bucket>> {
        self.buckets.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The key of a bucket.
    pub(crate) fn key(group: &str, token: Option<&str>) -> String {
        format!("{group}|{}", token.unwrap_or(""))
    }

    /// The group a key was made for.
    pub(crate) fn group_of(key: &str) -> &str {
        key.split('|').next().unwrap_or(key)
    }

    /// Reserve budget for one request, if it fits at `now`.
    pub(crate) fn try_acquire(&self, key: &str, now: i64) -> Acquire {
        let mut buckets = self.lock();
        let bucket = buckets.entry(key.to_owned()).or_default();
        match bucket.wait_ms(now) {
            0 => {
                bucket.in_flight += 1;
                Acquire::Granted
            }
            wait => Acquire::Wait(wait),
        }
    }

    /// Release the reservation of a request that is no longer in flight.
    pub(crate) fn release(&self, key: &str) {
        if let Some(bucket) = self.lock().get_mut(key) {
            bucket.in_flight = (bucket.in_flight - 1).max(0);
        }
    }

    /// Take in the balance a response reported: `remaining` tokens left after
    /// it spent `used`, with the group's `window_ms` (0 if unknown).
    pub(crate) fn record(
        &self,
        key: &str,
        remaining: i64,
        used: i64,
        window_ms: i64,
        success: bool,
        now: i64,
    ) {
        let mut buckets = self.lock();
        let bucket = buckets.entry(key.to_owned()).or_default();
        if window_ms > 0 {
            bucket.window_ms = window_ms;
        }
        bucket.remaining = remaining;
        bucket.last_response_ms = now;
        bucket.seen = true;
        if success && used > 0 {
            bucket.cost = used;
        }
        if used > 0 {
            bucket.spent.push_back((now, used));
        }
        let window = bucket.window_ms;
        while bucket
            .spent
            .front()
            .is_some_and(|(at, _)| at + window <= now)
        {
            bucket.spent.pop_front();
        }
        buckets.retain(|_, b| b.in_flight > 0 || b.last_response_ms + b.window_ms > now);
    }

    /// No request fits for this key before `until_ms` (a `Retry-After`).
    pub(crate) fn block_until(&self, key: &str, until_ms: i64) {
        let mut buckets = self.lock();
        let bucket = buckets.entry(key.to_owned()).or_default();
        bucket.blocked_until_ms = bucket.blocked_until_ms.max(until_ms);
        bucket.seen = true;
    }
}

/// A reservation of budget for a request in flight; released when dropped.
pub(crate) struct Permit {
    limiter: Arc<RateLimiter>,
    key: String,
}

impl Permit {
    pub(crate) fn new(limiter: Arc<RateLimiter>, key: &str) -> Self {
        Permit {
            limiter,
            key: key.to_owned(),
        }
    }
}

impl Drop for Permit {
    fn drop(&mut self) {
        self.limiter.release(&self.key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "g|t";

    #[test]
    fn test_unseen_groups_are_not_throttled() {
        let limiter = RateLimiter::default();
        for _ in 0..10 {
            assert_eq!(limiter.try_acquire(KEY, 0), Acquire::Granted);
        }
    }

    #[test]
    fn test_requests_that_fit_are_granted_and_in_flight_ones_count() {
        let limiter = RateLimiter::default();
        // 5 tokens left, a request costs 2: two fit, the third does not.
        limiter.record(KEY, 5, 2, 10_000, true, 0);
        assert_eq!(limiter.try_acquire(KEY, 1), Acquire::Granted);
        assert_eq!(limiter.try_acquire(KEY, 1), Acquire::Granted);
        assert!(matches!(limiter.try_acquire(KEY, 1), Acquire::Wait(_)));
        // A released reservation frees its share.
        limiter.release(KEY);
        assert_eq!(limiter.try_acquire(KEY, 1), Acquire::Granted);
    }

    #[test]
    fn test_tokens_return_when_their_window_ends() {
        let limiter = RateLimiter::default();
        limiter.record(KEY, 0, 2, 1_000, true, 0);
        // The 2 tokens spent at 0 are back at 1000.
        let Acquire::Wait(wait) = limiter.try_acquire(KEY, 100) else {
            panic!("should wait");
        };
        assert_eq!(wait, 900 + WAIT_MARGIN_MS);
        assert_eq!(limiter.try_acquire(KEY, 1_000), Acquire::Granted);
    }

    #[test]
    fn test_the_wait_ends_at_the_first_return_that_is_enough() {
        let limiter = RateLimiter::default();
        // Spent 2 at 0, 2 at 400, 2 at 800; none left. One request (2) fits once
        // the first return arrives, at 1000.
        for at in [0, 400, 800] {
            limiter.record(KEY, 0, 2, 1_000, true, at);
        }
        assert_eq!(
            limiter.try_acquire(KEY, 800),
            Acquire::Wait(200 + WAIT_MARGIN_MS)
        );
        // With a request in flight, two returns are needed: 1000 and 1400.
        let limiter = RateLimiter::default();
        for at in [0, 400, 800] {
            limiter.record(KEY, 0, 2, 1_000, true, at);
        }
        limiter.record(KEY, 0, 0, 1_000, true, 800);
        assert_eq!(limiter.try_acquire(KEY, 1_000), Acquire::Granted);
        assert_eq!(
            limiter.try_acquire(KEY, 1_000),
            Acquire::Wait(400 + WAIT_MARGIN_MS)
        );
    }

    #[test]
    fn test_retry_after_blocks_the_group() {
        let limiter = RateLimiter::default();
        limiter.record(KEY, 100, 2, 10_000, true, 0);
        limiter.block_until(KEY, 5_000);
        assert_eq!(
            limiter.try_acquire(KEY, 1_000),
            Acquire::Wait(4_000 + WAIT_MARGIN_MS)
        );
        assert_eq!(limiter.try_acquire(KEY, 5_000), Acquire::Granted);
    }

    #[test]
    fn test_buckets_are_independent_per_group_and_token() {
        let limiter = RateLimiter::default();
        limiter.record("a|x", 0, 2, 10_000, true, 0);
        assert!(matches!(limiter.try_acquire("a|x", 1), Acquire::Wait(_)));
        assert_eq!(limiter.try_acquire("a|y", 1), Acquire::Granted);
        assert_eq!(limiter.try_acquire("b|x", 1), Acquire::Granted);
        assert_eq!(RateLimiter::key("a", Some("x")), "a|x");
        assert_eq!(RateLimiter::group_of("a|x"), "a");
    }

    #[test]
    fn test_idle_buckets_are_dropped() {
        let limiter = RateLimiter::default();
        limiter.record("old|x", 10, 2, 1_000, true, 0);
        limiter.record("new|x", 10, 2, 1_000, true, 5_000);
        assert_eq!(limiter.lock().len(), 1);
    }

    #[test]
    fn test_a_permit_releases_on_drop() {
        let limiter = Arc::new(RateLimiter::default());
        limiter.record(KEY, 2, 2, 10_000, true, 0);
        assert_eq!(limiter.try_acquire(KEY, 1), Acquire::Granted);
        let permit = Permit::new(Arc::clone(&limiter), KEY);
        assert!(matches!(limiter.try_acquire(KEY, 1), Acquire::Wait(_)));
        drop(permit);
        assert_eq!(limiter.try_acquire(KEY, 1), Acquire::Granted);
    }

    #[test]
    fn test_the_policy_serializes_in_snake_case() {
        assert_eq!(
            serde_json::to_string(&RateLimitPolicy::Off).unwrap(),
            "\"off\""
        );
        let wait = RateLimitPolicy::Wait {
            max_wait: Duration::from_secs(3),
        };
        let json = serde_json::to_string(&wait).unwrap();
        assert_eq!(
            serde_json::from_str::<RateLimitPolicy>(&json).unwrap(),
            wait
        );
    }
}
