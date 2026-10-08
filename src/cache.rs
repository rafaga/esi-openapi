//! Optional client-side cache of `GET` responses.
//!
//! When enabled with [`EsiBuilder::enable_cache`](crate::prelude::EsiBuilder::enable_cache),
//! responses are kept in memory with their `ETag` and `Last-Modified`. A request made
//! before the entry expires is answered from the cache without calling ESI; a later
//! one is revalidated with `If-None-Match` / `If-Modified-Since`, and a `304 Not
//! Modified` reply reuses the stored body.
//!
//! The cache holds a limited number of responses and, optionally, a limited number
//! of bytes (the size of the bodies, headers and keys it stores). Only when a new
//! response does not fit in either limit, entries are dropped one at a time in this
//! order, and within each group the least recently used goes first:
//!
//! 1. expired entries that cannot be revalidated (no `ETag` and no `Last-Modified`),
//! 2. expired entries that can be revalidated,
//! 3. entries that have not expired.

use reqwest::header::HeaderMap;
use std::collections::HashMap;

/// A stored response.
#[derive(Debug, Clone)]
pub(crate) struct CacheEntry {
    pub(crate) etag: Option<String>,
    pub(crate) last_modified: Option<String>,
    pub(crate) body: String,
    /// Response headers, kept so `X-Pages` is still available on a hit.
    pub(crate) headers: HeaderMap,
    /// Unix time in milliseconds until which the entry is served without a request.
    pub(crate) expires_at: i64,
    /// When the entry was last stored, read or revalidated, as a counter: the
    /// smaller, the longer it has gone unused.
    pub(crate) last_used: i64,
}

impl CacheEntry {
    /// Whether a request can be revalidated with this entry (a `304` reuses it).
    fn can_revalidate(&self) -> bool {
        self.etag.is_some() || self.last_modified.is_some()
    }

    /// Eviction group: lower goes first. See the module documentation.
    fn eviction_group(&self, now: i64) -> u8 {
        match (self.expires_at <= now, self.can_revalidate()) {
            (true, false) => 0,
            (true, true) => 1,
            (false, _) => 2,
        }
    }
}

impl CacheEntry {
    /// Approximate memory the entry holds: body, validators and headers.
    fn size(&self) -> usize {
        self.body.len()
            + self.etag.as_deref().map_or(0, str::len)
            + self.last_modified.as_deref().map_or(0, str::len)
            + self
                .headers
                .iter()
                .map(|(name, value)| name.as_str().len() + value.len())
                .sum::<usize>()
    }
}

/// How many responses the cache keeps unless the builder says otherwise.
pub(crate) const DEFAULT_MAX_ENTRIES: usize = 1024;

/// In-memory cache keyed by request.
#[derive(Debug)]
pub(crate) struct ResponseCache {
    entries: HashMap<String, CacheEntry>,
    max_entries: usize,
    /// Limit on the size of the stored entries, if any.
    max_bytes: Option<usize>,
    /// Size of the stored entries (see [`CacheEntry::size`]) and their keys.
    bytes: usize,
    clock: i64,
}

impl Default for ResponseCache {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_ENTRIES)
    }
}

impl ResponseCache {
    /// A cache that keeps at most `max_entries` responses (at least one).
    pub(crate) fn new(max_entries: usize) -> Self {
        Self::with_limits(max_entries, None)
    }

    /// A cache that keeps at most `max_entries` responses (at least one) and,
    /// when `max_bytes` is set, at most that many bytes.
    pub(crate) fn with_limits(max_entries: usize, max_bytes: Option<usize>) -> Self {
        ResponseCache {
            entries: HashMap::new(),
            max_entries: max_entries.max(1),
            max_bytes,
            bytes: 0,
            clock: 0,
        }
    }

    /// The key of a request: access token, headers that change the response
    /// (language and tenant), URL and sorted query parameters.
    pub(crate) fn key(
        token: Option<&str>,
        variant: &str,
        url: &str,
        query: &[(&str, &str)],
    ) -> String {
        let mut pairs: Vec<String> = query.iter().map(|(k, v)| format!("{k}={v}")).collect();
        pairs.sort();
        format!(
            "{}|{variant}|{url}?{}",
            token.unwrap_or(""),
            pairs.join("&")
        )
    }

    /// The entry for a key, without marking it as used.
    #[cfg(test)]
    pub(crate) fn get(&self, key: &str) -> Option<&CacheEntry> {
        self.entries.get(key)
    }

    /// A copy of the entry for a key, marking it as the most recently used.
    pub(crate) fn lookup(&mut self, key: &str) -> Option<CacheEntry> {
        let stamp = self.tick();
        let entry = self.entries.get_mut(key)?;
        entry.last_used = stamp;
        Some(entry.clone())
    }

    /// Store a response, marking it as the most recently used. If the cache has
    /// no room for it, entries are dropped first (see the module documentation);
    /// `now` is the current Unix time in milliseconds. Replacing a key never
    /// drops other entries because of the entry count. A response larger than
    /// the byte limit by itself is not stored, and the older one under its key
    /// is dropped.
    pub(crate) fn insert(&mut self, key: String, mut entry: CacheEntry, now: i64) {
        self.remove(&key);
        let size = key.len() + entry.size();
        if self.max_bytes.is_some_and(|max| size > max) {
            return;
        }
        while self.entries.len() >= self.max_entries
            || self.max_bytes.is_some_and(|max| self.bytes + size > max)
        {
            if !self.evict_one(now) {
                break;
            }
        }
        entry.last_used = self.tick();
        self.bytes += size;
        self.entries.insert(key, entry);
    }

    /// Extend the life of an entry after a `304 Not Modified`, marking it as used.
    pub(crate) fn refresh(&mut self, key: &str, expires_at: i64) {
        let stamp = self.tick();
        if let Some(entry) = self.entries.get_mut(key) {
            entry.expires_at = expires_at;
            entry.last_used = stamp;
        }
    }

    /// Drop the next entry in eviction order. Returns whether there was one.
    fn evict_one(&mut self, now: i64) -> bool {
        let victim = self
            .entries
            .iter()
            .min_by_key(|(_, e)| (e.eviction_group(now), e.last_used))
            .map(|(k, _)| k.clone());
        victim.is_some_and(|key| self.remove(&key))
    }

    /// Drop an entry and release its bytes. Returns whether there was one.
    fn remove(&mut self, key: &str) -> bool {
        match self.entries.remove(key) {
            Some(entry) => {
                self.bytes -= key.len() + entry.size();
                true
            }
            None => false,
        }
    }

    fn tick(&mut self) -> i64 {
        self.clock += 1;
        self.clock
    }

    /// The `max-age` of a `Cache-Control` header, in seconds.
    pub(crate) fn max_age(headers: &HeaderMap) -> Option<i64> {
        let value = headers.get("cache-control")?.to_str().ok()?;
        value
            .split(',')
            .filter_map(|d| d.trim().strip_prefix("max-age="))
            .find_map(|v| v.trim().parse().ok())
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    /// The approximate size in bytes of what is stored.
    pub(crate) fn bytes(&self) -> usize {
        self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl ResponseCache {
        fn entry(expires_at: i64, etag: bool) -> CacheEntry {
            CacheEntry {
                etag: etag.then(|| "\"x\"".to_owned()),
                last_modified: None,
                body: "[]".into(),
                headers: HeaderMap::new(),
                expires_at,
                last_used: 0,
            }
        }
    }

    #[test]
    fn test_key_ignores_query_order() {
        let a = ResponseCache::key(Some("t"), "en", "u", &[("a", "1"), ("b", "2")]);
        let b = ResponseCache::key(Some("t"), "en", "u", &[("b", "2"), ("a", "1")]);
        assert_eq!(a, b);
        assert_ne!(
            a,
            ResponseCache::key(None, "en", "u", &[("a", "1"), ("b", "2")])
        );
        assert_ne!(
            a,
            ResponseCache::key(Some("t"), "de", "u", &[("a", "1"), ("b", "2")])
        );
    }

    #[test]
    fn test_max_age() {
        let mut headers = HeaderMap::new();
        assert_eq!(ResponseCache::max_age(&headers), None);
        headers.insert("cache-control", "public, max-age=300".parse().unwrap());
        assert_eq!(ResponseCache::max_age(&headers), Some(300));
    }

    #[test]
    fn test_insert_and_refresh() {
        let mut cache = ResponseCache::default();
        cache.insert("k".into(), ResponseCache::entry(10, true), 0);
        cache.refresh("k", 99);
        assert_eq!(cache.get("k").unwrap().expires_at, 99);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_nothing_is_dropped_while_there_is_room() {
        let mut cache = ResponseCache::new(3);
        cache.insert("expired".into(), ResponseCache::entry(1, false), 100);
        cache.insert("live".into(), ResponseCache::entry(1_000, true), 100);
        assert_eq!(cache.len(), 2);
        assert!(cache.get("expired").is_some());
    }

    #[test]
    fn test_full_cache_evicts_by_group_then_least_recently_used() {
        let now = 100;
        let mut cache = ResponseCache::new(4);
        cache.insert("live_old".into(), ResponseCache::entry(1_000, true), now);
        cache.insert("expired_etag".into(), ResponseCache::entry(5, true), now);
        cache.insert("expired_plain".into(), ResponseCache::entry(5, false), now);
        cache.insert("live_new".into(), ResponseCache::entry(1_000, true), now);
        // 1st: the expired entry that cannot be revalidated.
        cache.insert("a".into(), ResponseCache::entry(1_000, true), now);
        assert!(cache.get("expired_plain").is_none());
        assert!(cache.get("expired_etag").is_some());
        // 2nd: the expired entry with an ETag, before any live entry.
        cache.insert("b".into(), ResponseCache::entry(1_000, true), now);
        assert!(cache.get("expired_etag").is_none());
        assert!(cache.get("live_old").is_some());
        // 3rd: only live entries are left; the least recently used goes.
        cache.insert("c".into(), ResponseCache::entry(1_000, true), now);
        assert!(cache.get("live_old").is_none());
        assert_eq!(cache.len(), 4);
    }

    #[test]
    fn test_reads_and_refreshes_protect_an_entry() {
        let now = 100;
        let mut cache = ResponseCache::new(3);
        for key in ["a", "b", "c"] {
            cache.insert(key.into(), ResponseCache::entry(1_000, true), now);
        }
        // "a" is the oldest stored, but it is read, so "b" is the least recently used.
        assert!(cache.lookup("a").is_some());
        cache.insert("d".into(), ResponseCache::entry(1_000, true), now);
        assert!(cache.get("b").is_none());
        assert!(cache.get("a").is_some());
        // A `304` refresh counts as a use too: now "c" is the least recently used.
        cache.refresh("a", 2_000);
        cache.refresh("d", 2_000);
        cache.insert("e".into(), ResponseCache::entry(1_000, true), now);
        assert!(cache.get("c").is_none());
        assert_eq!(cache.len(), 3);
    }

    fn sized(body_len: usize, expires_at: i64, etag: bool) -> CacheEntry {
        CacheEntry {
            body: "x".repeat(body_len),
            ..ResponseCache::entry(expires_at, etag)
        }
    }

    #[test]
    fn test_byte_limit_evicts_in_the_same_order() {
        // Keys are one byte; an entry is its body, a 3-byte ETag and the key.
        let one = 1 + 3 + 10;
        let mut cache = ResponseCache::with_limits(100, Some(3 * one));
        cache.insert("a".into(), sized(10, 1_000, true), 100);
        cache.insert("b".into(), sized(10, 5, true), 100);
        cache.insert("c".into(), sized(10, 5, false), 100);
        assert_eq!(cache.bytes(), 3 * one - 3);
        // "c" is expired and cannot be revalidated, so it goes first.
        cache.insert("d".into(), sized(10, 1_000, true), 100);
        assert!(cache.get("c").is_none() && cache.get("b").is_some());
        // Then "b" (expired, revalidatable) before the live "a".
        cache.insert("e".into(), sized(10, 1_000, true), 100);
        assert!(cache.get("b").is_none() && cache.get("a").is_some());
        assert!(cache.bytes() <= 3 * one);
    }

    #[test]
    fn test_a_large_entry_evicts_as_many_as_it_needs() {
        // Each small entry is 14 bytes (key, 3-byte ETag, body); the big one is 66.
        let mut cache = ResponseCache::with_limits(100, Some(90));
        for key in ["a", "b", "c"] {
            cache.insert(key.into(), sized(10, 1_000, true), 0);
        }
        cache.insert("big".into(), sized(60, 1_000, true), 0);
        assert!(cache.get("a").is_none() && cache.get("b").is_none());
        assert!(cache.get("c").is_some() && cache.get("big").is_some());
        assert_eq!(cache.bytes(), 14 + 66);
    }

    #[test]
    fn test_an_entry_over_the_byte_limit_is_not_stored() {
        let mut cache = ResponseCache::with_limits(100, Some(50));
        cache.insert("a".into(), sized(10, 1_000, true), 0);
        cache.insert("a".into(), sized(500, 1_000, true), 0);
        assert_eq!((cache.len(), cache.bytes()), (0, 0));
        cache.insert("b".into(), sized(10, 1_000, true), 0);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_replacing_adjusts_the_byte_count() {
        let mut cache = ResponseCache::with_limits(100, Some(1_000));
        cache.insert("a".into(), sized(10, 1_000, true), 0);
        let before = cache.bytes();
        cache.insert("a".into(), sized(30, 1_000, true), 0);
        assert_eq!(cache.bytes(), before + 20);
        cache.refresh("a", 5);
        assert_eq!(cache.bytes(), before + 20);
    }

    #[test]
    fn test_replacing_a_key_never_evicts() {
        let mut cache = ResponseCache::new(2);
        cache.insert("a".into(), ResponseCache::entry(1_000, true), 0);
        cache.insert("b".into(), ResponseCache::entry(1_000, true), 0);
        cache.insert("a".into(), ResponseCache::entry(2_000, true), 0);
        assert_eq!(cache.len(), 2);
        assert!(cache.get("b").is_some());
    }
}
