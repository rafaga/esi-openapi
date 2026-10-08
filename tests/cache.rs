//! Exercises the response cache against a tiny local HTTP server.

use esi_openapi::prelude::*;
use esi_openapi::spec::Spec;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// A local server that records the requests it receives.
struct LocalServer {
    url: String,
    requests: Arc<Mutex<Vec<String>>>,
}

impl LocalServer {
    async fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&requests);
        tokio::spawn(async move {
            loop {
                let (mut socket, _) = listener.accept().await.unwrap();
                let seen = Arc::clone(&seen);
                tokio::spawn(async move {
                    let mut buffer = vec![0u8; 8192];
                    let n = socket.read(&mut buffer).await.unwrap_or(0);
                    let request = String::from_utf8_lossy(&buffer[..n]).to_lowercase();
                    seen.lock().unwrap().push(request.clone());
                    let first_line = request.lines().next().unwrap_or_default().to_owned();
                    let response = if first_line.contains("/spec") {
                        // `spec` stays fresh for a minute; `spec0` never does.
                        let max_age = if first_line.contains("/spec0") { 0 } else { 60 };
                        if request.contains("if-none-match: \"s1\"") {
                            format!("HTTP/1.1 304 Not Modified\r\nETag: \"s1\"\r\nCache-Control: max-age={max_age}\r\nConnection: close\r\n\r\n")
                        } else {
                            let body =
                                r#"{"paths":{"/limited":{"get":{"operationId":"GetLimited"}}}}"#;
                            format!(
                                "HTTP/1.1 200 OK\r\nETag: \"s1\"\r\nCache-Control: max-age={max_age}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                                body.len()
                            )
                        }
                    } else if first_line.contains("/limited") {
                        // A group whose window of 1 second has no tokens left.
                        let body = "[1]";
                        format!(
                            "HTTP/1.1 200 OK\r\nX-Ratelimit-Group: test\r\nX-Ratelimit-Limit: 10/1s\r\nX-Ratelimit-Remaining: 0\r\nX-Ratelimit-Used: 2\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                            body.len()
                        )
                    } else if first_line.contains("/blocked") {
                        "HTTP/1.1 429 Too Many Requests\r\nX-Ratelimit-Group: blocked\r\nX-Ratelimit-Limit: 10/15m\r\nX-Ratelimit-Remaining: 0\r\nX-Ratelimit-Used: 5\r\nRetry-After: 30\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                            .to_owned()
                    } else if first_line.contains("/echo") {
                        // Answers with the number of items in the posted array.
                        let posted = request.split("\r\n\r\n").nth(1).unwrap_or_default();
                        let count = posted.matches(',').count() + 1;
                        let body = format!("[{count}]");
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                            body.len()
                        )
                    } else if first_line.contains("/cursor") {
                        let body = if first_line.contains("after=2") {
                            r#"{"items":[3,4]}"#
                        } else {
                            r#"{"cursor":{"after":"2"},"items":[1,2]}"#
                        };
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                            body.len()
                        )
                    } else if first_line.contains("/gone") {
                        "HTTP/1.1 410 Gone\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                            .to_owned()
                    } else if request.contains("if-none-match: \"v1\"") {
                        "HTTP/1.1 304 Not Modified\r\nETag: \"v1\"\r\nConnection: close\r\n\r\n"
                            .to_owned()
                    } else {
                        let max_age = if first_line.contains("/fresh") { 60 } else { 0 };
                        let body = "[1,2,3]";
                        format!(
                            "HTTP/1.1 200 OK\r\nETag: \"v1\"\r\nCache-Control: max-age={max_age}\r\nX-Pages: 4\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                            body.len()
                        )
                    };
                    let _ = socket.write_all(response.as_bytes()).await;
                });
            }
        });
        LocalServer { url, requests }
    }

    fn count(&self) -> usize {
        self.requests.lock().unwrap().len()
    }
}

struct Client;

impl Client {
    fn build(url: &str, cache: bool) -> Esi {
        EsiBuilder::new()
            .user_agent("cache test")
            .base_api_url(url)
            .enable_cache(cache)
            .build()
            .unwrap()
    }
}

#[tokio::test]
async fn revalidates_with_etag_and_reuses_the_body_on_304() {
    let server = LocalServer::start().await;
    let esi = Client::build(&server.url, true);
    let first: Vec<i64> = esi
        .query("GET", RequestType::Public, "stale", None, None)
        .await
        .unwrap();
    let second: Vec<i64> = esi
        .query("GET", RequestType::Public, "stale", None, None)
        .await
        .unwrap();
    assert_eq!(first, [1, 2, 3]);
    assert_eq!(second, [1, 2, 3]);
    assert_eq!(server.count(), 2, "max-age=0 must be revalidated");
    assert!(server.requests.lock().unwrap()[1].contains("if-none-match: \"v1\""));
    assert_eq!(esi.cache_len().await, 1);
}

#[tokio::test]
async fn serves_fresh_entries_without_calling_the_server() {
    let server = LocalServer::start().await;
    let esi = Client::build(&server.url, true);
    for _ in 0..3 {
        let (data, pages): (Vec<i64>, Option<i64>) = esi
            .query_with_pages("GET", RequestType::Public, "fresh", None)
            .await
            .unwrap();
        assert_eq!(data, [1, 2, 3]);
        assert_eq!(pages, Some(4));
    }
    assert_eq!(server.count(), 1);
}

#[tokio::test]
async fn the_cache_is_off_by_default() {
    let server = LocalServer::start().await;
    let esi = Client::build(&server.url, false);
    for _ in 0..2 {
        let _: Vec<i64> = esi
            .query("GET", RequestType::Public, "fresh", None, None)
            .await
            .unwrap();
    }
    assert_eq!(server.count(), 2);
    assert_eq!(esi.cache_len().await, 0);
}

#[tokio::test]
async fn fetch_all_pages_follows_x_pages() {
    let server = LocalServer::start().await;
    let esi = Client::build(&server.url, false);
    let items: Vec<i64> = esi
        .fetch_all_pages(RequestType::Public, "fresh", None, None)
        .await
        .unwrap();
    assert_eq!(items.len(), 12);
    assert_eq!(server.count(), 4);
    let limited: Vec<i64> = esi
        .fetch_all_pages(RequestType::Public, "fresh", None, Some(2))
        .await
        .unwrap();
    assert_eq!(limited.len(), 6);
}

#[tokio::test]
async fn sends_language_and_tenant_headers() {
    let server = LocalServer::start().await;
    let esi = EsiBuilder::new()
        .user_agent("header test")
        .base_api_url(&server.url)
        .language(Language::De)
        .tenant("singularity")
        .build()
        .unwrap();
    let _: Vec<i64> = esi
        .query("GET", RequestType::Public, "fresh", None, None)
        .await
        .unwrap();
    let request = server.requests.lock().unwrap()[0].clone();
    assert!(request.contains("accept-language: de"), "{request}");
    assert!(request.contains("x-tenant: singularity"), "{request}");
}

#[tokio::test]
async fn reports_tombstones_as_gone() {
    let server = LocalServer::start().await;
    let spec: Spec = serde_json::from_str(include_str!("../resources/test/openapi.json")).unwrap();
    let esi = EsiBuilder::new()
        .user_agent("tombstone test")
        .base_api_url(&server.url)
        .spec(Some(spec))
        .build()
        .unwrap();
    // The alliances listing declares `x-tombstone-ttl`, so a 410 there is `Gone`.
    let result: EsiResult<serde_json::Value> = esi
        .query(
            "GET",
            RequestType::Public,
            "paragon-hub/skinr/alliances/gone",
            None,
            None,
        )
        .await;
    assert!(
        matches!(
            result,
            Err(EsiError::Gone {
                status: 410,
                tombstone_ttl_secs: 604_800
            })
        ),
        "{result:?}"
    );
    // An operation without a tombstone TTL keeps the plain status error.
    let result: EsiResult<serde_json::Value> = esi
        .query("GET", RequestType::Public, "characters/gone", None, None)
        .await;
    assert!(
        matches!(result, Err(EsiError::InvalidStatusCode(410))),
        "{result:?}"
    );
}

#[tokio::test]
async fn cursor_walk_asks_for_the_largest_page_size() {
    let server = LocalServer::start().await;
    let esi = Client::build(&server.url, false);
    let items: Vec<i64> = esi
        .fetch_all_cursor(RequestType::Public, "cursor", None, "items", None)
        .await
        .unwrap();
    assert_eq!(items, [1, 2, 3, 4]);
    let requests = server.requests.lock().unwrap().clone();
    assert_eq!(requests.len(), 2);
    assert!(
        requests.iter().all(|r| r.contains("limit=100")),
        "{requests:?}"
    );
    drop(requests);
    // An explicit `limit` is kept.
    let items: Vec<i64> = esi
        .fetch_all_cursor(
            RequestType::Public,
            "cursor",
            Some(&[("limit", "10")]),
            "items",
            Some(1),
        )
        .await
        .unwrap();
    assert_eq!(items, [1, 2]);
    let last = server.requests.lock().unwrap().last().unwrap().clone();
    assert!(
        last.contains("limit=10 ") || last.contains("limit=10&"),
        "{last}"
    );
}

#[tokio::test]
async fn pages_can_be_fetched_sequentially_or_concurrently() {
    for concurrency in [1, 3] {
        let server = LocalServer::start().await;
        let esi = EsiBuilder::new()
            .user_agent("concurrency test")
            .base_api_url(&server.url)
            .page_concurrency(concurrency)
            .build()
            .unwrap();
        let items: Vec<i64> = esi
            .fetch_all_pages(RequestType::Public, "fresh", None, None)
            .await
            .unwrap();
        assert_eq!(items.len(), 12, "concurrency {concurrency}");
        assert_eq!(server.count(), 4);
    }
}

#[tokio::test]
async fn the_cache_respects_its_size_limit() {
    let server = LocalServer::start().await;
    let esi = EsiBuilder::new()
        .user_agent("limit test")
        .base_api_url(&server.url)
        .enable_cache(true)
        .cache_max_entries(2)
        .build()
        .unwrap();
    for path in ["fresh/a", "fresh/b", "fresh/c"] {
        let _: Vec<i64> = esi
            .query("GET", RequestType::Public, path, None, None)
            .await
            .unwrap();
    }
    assert_eq!(esi.cache_len().await, 2);
}

#[tokio::test]
async fn the_cache_evicts_the_least_recently_used_entry() {
    let server = LocalServer::start().await;
    let esi = EsiBuilder::new()
        .user_agent("lru test")
        .base_api_url(&server.url)
        .enable_cache(true)
        .cache_max_entries(2)
        .build()
        .unwrap();
    // "fresh/..." answers with max-age=60, so these entries stay live.
    for path in ["fresh/a", "fresh/b", "fresh/a", "fresh/c"] {
        let _: Vec<i64> = esi
            .query("GET", RequestType::Public, path, None, None)
            .await
            .unwrap();
    }
    // a (hit), b, c: three requests so far. "a" was used after "b", so "b" went.
    assert_eq!(server.count(), 3);
    let _: Vec<i64> = esi
        .query("GET", RequestType::Public, "fresh/a", None, None)
        .await
        .unwrap();
    assert_eq!(server.count(), 3, "a is still cached");
    let _: Vec<i64> = esi
        .query("GET", RequestType::Public, "fresh/b", None, None)
        .await
        .unwrap();
    assert_eq!(server.count(), 4, "b had been evicted");
}

#[tokio::test]
async fn post_chunked_splits_long_lists_and_keeps_the_order() {
    for concurrency in [1, 3] {
        let server = LocalServer::start().await;
        let esi = EsiBuilder::new()
            .user_agent("chunk test")
            .base_api_url(&server.url)
            .page_concurrency(concurrency)
            .build()
            .unwrap();
        let ids: Vec<i64> = (1..=2500).collect();
        let counts: Vec<i64> = esi
            .post_chunked(RequestType::Public, "echo", &ids, 1000)
            .await
            .unwrap();
        assert_eq!(counts, [1000, 1000, 500]);
        assert_eq!(server.count(), 3);
        assert!(server
            .requests
            .lock()
            .unwrap()
            .iter()
            .all(|r| r.starts_with("post ")));
    }
}

#[tokio::test]
async fn post_chunked_sends_nothing_for_an_empty_list() {
    let server = LocalServer::start().await;
    let esi = Client::build(&server.url, false);
    let ids: Vec<i64> = Vec::new();
    let counts: Vec<i64> = esi
        .post_chunked(RequestType::Public, "echo", &ids, 1000)
        .await
        .unwrap();
    assert!(counts.is_empty());
    assert_eq!(server.count(), 0);
}

#[tokio::test]
async fn the_cache_respects_its_byte_limit() {
    let server = LocalServer::start().await;
    let unbounded = Client::build(&server.url, true);
    let _: Vec<i64> = unbounded
        .query("GET", RequestType::Public, "fresh/a", None, None)
        .await
        .unwrap();
    let one = unbounded.cache_bytes().await;
    assert!(one > 0);

    // Room for two entries, not three.
    let esi = EsiBuilder::new()
        .user_agent("cache test")
        .base_api_url(&server.url)
        .enable_cache(true)
        .cache_max_bytes(one * 5 / 2)
        .build()
        .unwrap();
    for path in ["fresh/a", "fresh/b", "fresh/c"] {
        let _: Vec<i64> = esi
            .query("GET", RequestType::Public, path, None, None)
            .await
            .unwrap();
    }
    assert_eq!(esi.cache_len().await, 2);
    assert!(esi.cache_bytes().await <= one * 5 / 2);

    // Nothing fits in a tiny limit, but requests still work.
    let tiny = EsiBuilder::new()
        .user_agent("cache test")
        .base_api_url(&server.url)
        .enable_cache(true)
        .cache_max_bytes(10)
        .build()
        .unwrap();
    let data: Vec<i64> = tiny
        .query("GET", RequestType::Public, "fresh/a", None, None)
        .await
        .unwrap();
    assert_eq!(data, [1, 2, 3]);
    assert_eq!(tiny.cache_len().await, 0);
}

/// A spec with two rate-limited paths, `limited` and `blocked`, and one without a group.
fn throttle_spec() -> Spec {
    serde_json::from_str(
        r#"{"paths": {
            "/limited": {"get": {"operationId": "GetLimited",
                "x-rate-limit": {"group": "test", "max-tokens": 10, "window-size": "1s"}}},
            "/blocked": {"get": {"operationId": "GetBlocked",
                "x-rate-limit": {"group": "blocked", "max-tokens": 10, "window-size": "15m"}}},
            "/free": {"get": {"operationId": "GetFree"}}
        }}"#,
    )
    .unwrap()
}

fn throttled(url: &str, policy: RateLimitPolicy) -> Esi {
    EsiBuilder::new()
        .user_agent("throttle test")
        .base_api_url(url)
        .spec(Some(throttle_spec()))
        .rate_limit_policy(policy)
        .build()
        .unwrap()
}

async fn get_ok(esi: &Esi, path: &str) -> EsiResult<Vec<i64>> {
    esi.query("GET", RequestType::Public, path, None, None)
        .await
}

#[tokio::test]
async fn the_throttle_is_off_by_default() {
    let server = LocalServer::start().await;
    let esi = throttled(&server.url, RateLimitPolicy::Off);
    for _ in 0..3 {
        get_ok(&esi, "limited").await.unwrap();
    }
    assert_eq!(server.count(), 3);
}

#[tokio::test]
async fn the_fail_policy_refuses_without_calling_esi() {
    let server = LocalServer::start().await;
    let esi = throttled(&server.url, RateLimitPolicy::Fail);
    assert_eq!(get_ok(&esi, "limited").await.unwrap(), [1]);
    let error = get_ok(&esi, "limited").await.unwrap_err();
    assert!(
        matches!(&error, EsiError::RateLimited { group: Some(g), .. } if g == "test"),
        "{error:?}"
    );
    assert_eq!(server.count(), 1);
    // A path the spec gives no group is never throttled.
    get_ok(&esi, "free").await.unwrap();
    get_ok(&esi, "free").await.unwrap();
    assert_eq!(server.count(), 3);
}

#[tokio::test]
async fn the_wait_policy_sleeps_until_the_tokens_return() {
    let server = LocalServer::start().await;
    let esi = throttled(
        &server.url,
        RateLimitPolicy::Wait {
            max_wait: std::time::Duration::from_secs(5),
        },
    );
    get_ok(&esi, "limited").await.unwrap();
    let started = std::time::Instant::now();
    get_ok(&esi, "limited").await.unwrap();
    assert!(
        started.elapsed() >= std::time::Duration::from_millis(900),
        "{:?}",
        started.elapsed()
    );
    assert_eq!(server.count(), 2);
}

#[tokio::test]
async fn the_wait_policy_gives_up_after_max_wait() {
    let server = LocalServer::start().await;
    let esi = throttled(
        &server.url,
        RateLimitPolicy::Wait {
            max_wait: std::time::Duration::from_millis(100),
        },
    );
    get_ok(&esi, "limited").await.unwrap();
    let started = std::time::Instant::now();
    let error = get_ok(&esi, "limited").await.unwrap_err();
    // About a second to go, rounded up.
    assert!(
        matches!(
            error,
            EsiError::RateLimited {
                retry_after_secs: Some(1..=2),
                ..
            }
        ),
        "{error:?}"
    );
    assert!(started.elapsed() < std::time::Duration::from_millis(500));
    assert_eq!(server.count(), 1);
}

#[tokio::test]
async fn a_429_blocks_the_group_for_its_retry_after() {
    let server = LocalServer::start().await;
    let esi = throttled(&server.url, RateLimitPolicy::Fail);
    let first = get_ok(&esi, "blocked").await.unwrap_err();
    assert!(matches!(
        first,
        EsiError::RateLimited {
            retry_after_secs: Some(30),
            ..
        }
    ));
    let second = get_ok(&esi, "blocked").await.unwrap_err();
    assert!(
        matches!(second, EsiError::RateLimited { retry_after_secs: Some(s), .. } if s >= 29),
        "{second:?}"
    );
    assert_eq!(server.count(), 1, "the second call must not reach ESI");
}

fn with_spec_url(url: &str, path: &str) -> Esi {
    EsiBuilder::new()
        .user_agent("spec test")
        .base_api_url(url)
        .spec_url(&format!("{url}{path}"))
        .build()
        .unwrap()
}

#[tokio::test]
async fn ensure_spec_fresh_skips_the_request_while_the_spec_is_fresh() {
    let server = LocalServer::start().await;
    let mut esi = with_spec_url(&server.url, "spec");
    assert!(esi.get_spec().is_none());
    esi.ensure_spec_fresh().await.unwrap();
    assert!(esi.get_spec().is_some());
    assert_eq!(esi.get_endpoint_for_op_id("GetLimited").unwrap(), "limited");
    assert_eq!(server.count(), 1);
    for _ in 0..3 {
        esi.ensure_spec_fresh().await.unwrap();
    }
    assert_eq!(server.count(), 1, "the spec is fresh for a minute");
}

#[tokio::test]
async fn ensure_spec_fresh_downloads_again_once_stale() {
    let server = LocalServer::start().await;
    let mut esi = with_spec_url(&server.url, "spec0");
    esi.ensure_spec_fresh().await.unwrap();
    esi.ensure_spec_fresh().await.unwrap();
    assert_eq!(server.count(), 2, "max-age=0 is never fresh");
    // The second request was conditional and its 304 kept the spec.
    assert!(server.requests.lock().unwrap()[1].contains("if-none-match: \"s1\""));
    assert!(esi.get_endpoint_for_op_id("GetLimited").is_ok());
}

#[tokio::test]
async fn update_spec_is_conditional_once_the_spec_is_known() {
    let server = LocalServer::start().await;
    let mut esi = with_spec_url(&server.url, "spec");
    esi.update_spec().await.unwrap();
    assert!(!server.requests.lock().unwrap()[0].contains("if-none-match"));
    esi.update_spec().await.unwrap();
    assert_eq!(server.count(), 2, "update_spec always asks");
    assert!(server.requests.lock().unwrap()[1].contains("if-none-match: \"s1\""));
    assert!(esi.get_endpoint_for_op_id("GetLimited").is_ok());
}

#[tokio::test]
async fn a_spec_given_to_the_builder_has_no_known_age() {
    let server = LocalServer::start().await;
    let mut esi = EsiBuilder::new()
        .user_agent("spec test")
        .spec_url(&format!("{}spec", server.url))
        .spec(Some(throttle_spec()))
        .build()
        .unwrap();
    esi.ensure_spec_fresh().await.unwrap();
    assert_eq!(server.count(), 1);
    assert!(esi.get_endpoint_for_op_id("GetLimited").is_ok());
    assert!(esi.get_endpoint_for_op_id("GetFree").is_err());
}
