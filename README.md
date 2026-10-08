# esi-openapi

[![CI](https://github.com/rafaga/esi-openapi/actions/workflows/ci.yml/badge.svg)](https://github.com/rafaga/esi-openapi/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/esi-openapi.svg)](https://crates.io/crates/esi-openapi)
[![Docs.rs](https://docs.rs/esi-openapi/badge.svg)](https://docs.rs/esi-openapi)
[![License](https://img.shields.io/crates/l/esi-openapi)](https://github.com/rafaga/esi-openapi/blob/master/Cargo.toml)

Rust API for [EVE Online](https://www.eveonline.com/)'s [ESI](https://developers.eveonline.com/docs/services/esi/overview/), built on the ESI OpenAPI 3.1 specification.

## Origin

This crate is a fork of [rfesi](https://github.com/Celeo/rfesi) by Matt Boulanger (Celeo), starting from rfesi 0.50.2.

rfesi resolved endpoints through ESI's Swagger 2.0 spec, which CCP has retired. esi-openapi migrates it to:

- the OpenAPI spec at `https://esi.evetech.net/meta/openapi.json`,
- the `X-Compatibility-Date` versioning header,
- the new rate-limit headers (`X-Ratelimit-*`, `429` + `Retry-After`).

Versioning restarts at 0.1.0. The public API of the endpoint groups is kept, so moving from rfesi is mostly a matter of replacing `use rfesi::` with `use esi_openapi::`. See `CHANGELOG.md` for the mapping from rfesi's snake_case operation IDs to the OpenAPI ones.

Many thanks to Celeo and the rfesi contributors for the original work.

## Installing

Add the latest version to your `Cargo.toml`.

This crate has several features that are enabled by default.

- If you don't want or need random SSO state string generation, you can disable the "random_state" feature.
- If you don't want or need SSO token verification, you can disable the "validate_jwt" feature.
- Requests use [rustls](https://crates.io/crates/rustls) for TLS through the "rustls-tls" feature, which is enabled by default. The "default-tls" feature from rfesi has been removed.

## Using

[Docs link](https://docs.rs/esi-openapi).

### Coverage

All 233 operations of the ESI OpenAPI spec (compatibility date `2026-08-18`) have a function in an endpoint group, reached through `Esi::group_*()`. `tests/conformance.rs` checks this against `resources/test/openapi.json` and also that each function uses the HTTP method, path placeholders, query keys and response fields of the spec. Anything newer can be called with `Esi::get_endpoint_for_op_id` and `Esi::query`, and PRs are welcome.

Beyond the endpoints:

- `Esi::fetch_all_pages` and `Esi::fetch_all_cursor` walk paginated listings; `Esi::query_with_pages` also returns `X-Pages`.
- `EsiBuilder::enable_cache(true)` turns on a response cache that revalidates with `ETag` / `Last-Modified` and honors `x-client-cache-ttl`. `cache_max_entries` and `cache_max_bytes` bound its size.
- `Esi::post_chunked` and the bulk lookups (`get_affiliation`, `get_names`, asset names and locations) split lists longer than the spec's maximum into several requests.
- `EsiBuilder::rate_limit_policy(RateLimitPolicy::Wait { .. } | Fail)` throttles requests against the rate limits ESI reports instead of provoking `429`s (off by default).
- `Esi::ensure_spec_fresh` reloads the spec only when it is stale.
- `EsiBuilder::language` and `EsiBuilder::tenant` set `Accept-Language` and `X-Tenant`.
- `Esi::required_scopes`, `Esi::missing_scopes`, `Esi::required_roles` and `Esi::declared_rate_limits` read the scopes, roles and rate limits that the spec declares for each operation.
- `EsiError::Gone` reports resources ESI keeps as tombstones.

### Compatibility date

Requests carry an `X-Compatibility-Date` header that pins the ESI response schemas. The default is `COMPATIBILITY_DATE_DEFAULT` (`2026-08-18`); override it with `EsiBuilder::compatibility_date`. The dates ESI accepts are listed at `https://esi.evetech.net/meta/compatibility-dates`.

### Rate limits

ESI rate-limits routes per group with a token bucket. After each response, `esi.rate_limit_status("market").await` returns the latest `RateLimitStatus` for that group, and a `429` response comes back as `EsiError::RateLimited { group, retry_after_secs }`. Routes not yet moved to the new limiter still use the error limit, checked with `esi.is_error_limited().await`.

## Developing

### Requirements

* Git
* A recent version of [Rust](https://www.rust-lang.org/tools/install)

### Steps

```sh
git clone https://github.com/rafaga/esi-openapi
cd esi-openapi
cargo test
```

### Authenticated tests

The endpoints that act for a character are tested against the live ESI API in `tests/authenticated.rs`. Those tests read credentials from a `.env` file at the crate root, which is excluded in `.gitignore`. Never commit it.

1. Create an application at the [EVE developers site](https://developers.eveonline.com/applications) with the callback URL and the scopes listed in `.env.example`.
2. Copy `.env.example` to `.env` and fill in `ESI_CLIENT_ID` (and `ESI_CLIENT_SECRET`, unless you use the PKCE flow).
3. Log in once to store a refresh token in `.env`:

   ```sh
   cargo run --example auth_get_refresh_token
   ```

4. Run the tests. They run with a plain `cargo test` whenever `.env` has a client ID and a refresh token, and are skipped (with a message) otherwise. To see the result for each endpoint:

   ```sh
   cargo test --test authenticated -- --nocapture
   ```

EVE SSO may rotate the refresh token on use; the tests write the new one back to `.env`. Without a configured `.env` (for example in CI) the tests print why and pass without calling ESI.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
* MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option, the same as rfesi. The original copyright notices are kept in both license files.

## Contributing

Please feel free to contribute. Please open an issue first (or comment on an existing one) so that I know that you want to add/change something.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
