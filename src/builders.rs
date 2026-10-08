//! Builders

use crate::{prelude::*, spec::Spec};
use reqwest::{header, Client};
use std::time::Duration;

/// Builder for the `Esi` struct.
///
/// # Example
///
/// ```rust
/// # use esi_openapi::prelude::EsiBuilder;
/// let mut esi = EsiBuilder::new()
///     .user_agent("some user agent")
///     .client_id("your_client_id")
///     .client_secret("your_client_secret")
///     .callback_url("your_callback_url")
///     .build()
///     .unwrap();
/// ```
///
/// # Overriding the API specification
///
/// If you will be creating multiple struct instances,
/// you will probably run into the issue of needing to
/// retrieve the API spec on each. Given that this
/// operation is fairly expensive (certainly as far as
/// the rest of the API endpoints that EVE makes available),
/// this builder supports setting that struct:
///
/// ```rust
/// # use esi_openapi::prelude::EsiBuilder;
/// # let your_spec = serde_json::from_str(r#"{"paths": {}}"#).unwrap();
/// let mut esi = EsiBuilder::new()
///     .user_agent("some user agent")
///     .spec(Some(your_spec))
///     .build()
///     .unwrap();
/// ```
///
/// Note that this "spec" function is just another builder
/// function; you can make use it alongside all of the others.
/// Note also that this is entirely optional: if you don't
/// mind retrieving the spec from ESI on each of your
/// struct instances, you can safely ignore this.
///
/// # Not including client info
///
/// If you are only making calls to non-authenticated
/// endpoints, then you don't need to make use of
/// the authentication flow, which means you don't need
/// a client ID, client secret, and callback URL.
/// In this case, you can construct your client without
/// those parameters:
///
/// ```rust
/// # use esi_openapi::prelude::EsiBuilder;
/// let mut esi = EsiBuilder::new()
///     .user_agent("some user agent")
///     .build()
///     .unwrap();
/// ```
///
/// Note that you still need to set the user agent, as this is good
/// API usage behavior.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct EsiBuilder {
    pub(crate) compatibility_date: Option<String>,
    pub(crate) client_id: Option<String>,
    pub(crate) client_secret: Option<String>,
    pub(crate) application_auth: Option<bool>,
    pub(crate) callback_url: Option<String>,
    pub(crate) base_api_url: Option<String>,
    pub(crate) authorize_url: Option<String>,
    pub(crate) token_url: Option<String>,
    pub(crate) spec_url: Option<String>,
    pub(crate) scope: Option<String>,
    pub(crate) access_token: Option<String>,
    pub(crate) access_expiration: Option<i64>,
    pub(crate) refresh_token: Option<String>,
    pub(crate) user_agent: Option<String>,
    pub(crate) http_timeout: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) cache_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) cache_max_entries: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) cache_max_bytes: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) rate_limit_policy: Option<RateLimitPolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) page_concurrency: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) language: Option<Language>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) tenant: Option<String>,
    pub(crate) spec: Option<Spec>,
}

/// Languages ESI can answer in (the `Accept-Language` header).
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// English (the default).
    #[default]
    En,
    /// German.
    De,
    /// French.
    Fr,
    /// Japanese.
    Ja,
    /// Russian.
    Ru,
    /// Chinese.
    Zh,
    /// Korean.
    Ko,
    /// Spanish.
    Es,
}

impl Language {
    /// The value of the `Accept-Language` header.
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::En => "en",
            Language::De => "de",
            Language::Fr => "fr",
            Language::Ja => "ja",
            Language::Ru => "ru",
            Language::Zh => "zh",
            Language::Ko => "ko",
            Language::Es => "es",
        }
    }
}

impl EsiBuilder {
    /// Start a new builder.
    pub fn new() -> Self {
        Default::default()
    }

    /// Set the compatibility header to use.
    ///
    /// Will default to a hardcoded value if not set.
    pub fn compatibility_date(mut self, val: &str) -> Self {
        self.compatibility_date = Some(val.to_owned());
        self
    }

    /// Set the client_id.
    pub fn client_id(mut self, val: &str) -> Self {
        self.client_id = Some(val.to_owned());
        self
    }

    /// Set the client_secret (https://docs.esi.evetech.net/docs/sso/web_based_sso_flow.html).
    pub fn client_secret(mut self, val: &str) -> Self {
        self.client_secret = Some(val.to_owned());
        self
    }

    /// Enable PKCE Authentication flow for Applications (https://docs.esi.evetech.net/docs/sso/native_sso_flow.html)
    pub fn enable_application_authentication(mut self, val: bool) -> Self {
        self.application_auth = Some(val);
        self
    }

    /// Set the callback_url.
    pub fn callback_url(mut self, val: &str) -> Self {
        self.callback_url = Some(val.to_owned());
        self
    }

    /// Set the base_api_url.
    pub fn base_api_url(mut self, val: &str) -> Self {
        self.base_api_url = Some(val.to_owned());
        self
    }

    /// Set the authorize_url.
    pub fn authorize_url(mut self, val: &str) -> Self {
        self.authorize_url = Some(val.to_owned());
        self
    }

    /// Set the token_url.
    pub fn token_url(mut self, val: &str) -> Self {
        self.token_url = Some(val.to_owned());
        self
    }

    /// Set the spec_url.
    pub fn spec_url(mut self, val: &str) -> Self {
        self.spec_url = Some(val.to_owned());
        self
    }

    /// Set the scope.
    pub fn scope(mut self, val: &str) -> Self {
        self.scope = Some(val.to_owned().replace(' ', "%20"));
        self
    }

    /// Set the access_token.
    pub fn access_token(mut self, val: Option<&str>) -> Self {
        self.access_token = val.map(|v| v.to_owned());
        self
    }

    /// Set the access_expiration.
    pub fn access_expiration(mut self, val: Option<i64>) -> Self {
        self.access_expiration = val;
        self
    }

    /// Set the refresh_token.
    pub fn refresh_token(mut self, val: Option<&str>) -> Self {
        self.refresh_token = val.map(|v| v.to_owned());
        self
    }

    /// Set the user_agent.
    pub fn user_agent(mut self, val: &str) -> Self {
        self.user_agent = Some(val.to_owned());
        self
    }

    /// Set the timeout to use in millis when sending HTTP requests.
    ///
    /// Will default to 60,000 (1 minute) if not set.
    pub fn http_timeout(mut self, val: Option<u64>) -> Self {
        self.http_timeout = val;
        self
    }

    /// Explicitly set the OpenAPI specification.
    ///
    /// Allows copying the spec from another `Esi` struct
    /// or other source, avoiding fetching the spec from
    /// ESI again. May be useful given the runtime cost
    /// of retrieving the spec.
    ///
    /// Be aware of the potential for out-of-date data.
    pub fn spec(mut self, spec: Option<Spec>) -> Self {
        self.spec = spec;
        self
    }

    pub(crate) fn construct_client(&self) -> EsiResult<Client> {
        let http_timeout = self
            .http_timeout
            .map(Duration::from_millis)
            .unwrap_or_else(|| Duration::from_secs(60));
        let headers = {
            let mut map = header::HeaderMap::new();
            let user_agent = &self
                .user_agent
                .as_ref()
                .ok_or_else(|| EsiError::EmptyClientValue("user_agent".to_owned()))?
                .to_owned();
            map.insert(
                header::USER_AGENT,
                header::HeaderValue::from_str(user_agent)?,
            );
            map.insert(
                header::ACCEPT,
                header::HeaderValue::from_static("application/json"),
            );
            map
        };
        let builder = Client::builder()
            .timeout(http_timeout)
            .default_headers(headers);
        #[cfg(feature = "rustls-tls")]
        let builder = builder.tls_backend_rustls();
        Ok(builder.build()?)
    }

    /// Limit how many responses the cache keeps (1024 by default). When it is
    /// full, expired entries are dropped first and then the oldest ones.
    /// Has no effect unless the cache is enabled with [`EsiBuilder::enable_cache`].
    pub fn cache_max_entries(mut self, val: usize) -> Self {
        self.cache_max_entries = Some(val);
        self
    }

    /// Limit how many bytes the cache keeps (no limit by default), counting the
    /// bodies, validators, headers and keys it stores. It applies together with
    /// [`EsiBuilder::cache_max_entries`]: when a response does not fit in either
    /// limit, entries are dropped in the same order. A response larger than the
    /// limit by itself is not cached. Has no effect unless the cache is enabled
    /// with [`EsiBuilder::enable_cache`].
    pub fn cache_max_bytes(mut self, val: usize) -> Self {
        self.cache_max_bytes = Some(val);
        self
    }

    /// Choose what the client does when a route group has no rate-limit tokens
    /// left for a request ([`RateLimitPolicy::Off`] by default): wait until the
    /// request fits, or fail without calling ESI.
    ///
    /// The balance is tracked per route group and access token, from the
    /// `X-Ratelimit-*` headers of the responses, and only for operations that
    /// declare a group in the spec, so the spec must be loaded.
    pub fn rate_limit_policy(mut self, val: RateLimitPolicy) -> Self {
        self.rate_limit_policy = Some(val);
        self
    }

    /// How many pages `Esi::fetch_all_pages` requests at the same time once it
    /// knows the page count (4 by default; 1 makes it sequential).
    pub fn page_concurrency(mut self, val: usize) -> Self {
        self.page_concurrency = Some(val);
        self
    }

    /// Set the language of the responses (the `Accept-Language` header).
    ///
    /// ESI answers in English when it is not set.
    pub fn language(mut self, val: Language) -> Self {
        self.language = Some(val);
        self
    }

    /// Set the tenant (the `X-Tenant` header). ESI uses `tranquility` when it is
    /// not set.
    pub fn tenant(mut self, val: &str) -> Self {
        self.tenant = Some(val.to_owned());
        self
    }

    /// Keep `GET` responses in memory and revalidate them with `ETag` /
    /// `Last-Modified` (disabled by default).
    ///
    /// While an entry is younger than the operation's `x-client-cache-ttl` in the
    /// spec (or the `max-age` of the response when the spec has none) it is served
    /// without calling ESI; afterwards it is revalidated and a `304 Not Modified`
    /// reuses the stored body. Entries are kept per access token.
    pub fn enable_cache(mut self, val: bool) -> Self {
        self.cache_enabled = Some(val);
        self
    }

    /// Construct the `Esi` instance.
    ///
    /// There are a few things that could go wrong, like
    /// not setting one of the mandatory fields or providing a user
    /// agent that is not a valid HTTP header value.
    pub fn build(self) -> EsiResult<Esi> {
        Esi::from_builder(self)
    }
}

#[cfg(test)]
mod tests {
    use super::EsiBuilder;
    use crate::spec::Spec;

    #[test]
    fn test_language_and_tenant() {
        let esi = EsiBuilder::new()
            .user_agent("d")
            .language(super::Language::De)
            .tenant("singularity")
            .build()
            .unwrap();
        assert_eq!(esi.language, Some(super::Language::De));
        assert_eq!(esi.tenant.as_deref(), Some("singularity"));
        assert_eq!(super::Language::Ko.as_str(), "ko");
        let json = serde_json::to_string(&EsiBuilder::new().language(super::Language::Fr)).unwrap();
        assert!(json.contains("\"language\":\"fr\""));
    }

    #[test]
    fn test_builder_valid() {
        let b = EsiBuilder::new()
            .client_id("a")
            .client_secret("b")
            .callback_url("c")
            .user_agent("d")
            .build()
            .unwrap();

        assert_eq!(b.client_id, Some(String::from("a")));
        assert_eq!(b.client_secret, Some(String::from("b")));
        assert_eq!(b.callback_url, Some(String::from("c")));
        assert_eq!(b.compatibility_date, "2026-08-18");
        assert_eq!(b.access_token, None);
        assert_eq!(b.spec, None);
    }

    #[test]
    fn test_builder_no_client() {
        let b = EsiBuilder::new().user_agent("d").build().unwrap();

        assert_eq!(b.client_id, None);
        assert_eq!(b.client_secret, None);
        assert_eq!(b.callback_url, None);
        assert_eq!(b.base_api_url, "https://esi.evetech.net/");
        assert_eq!(
            b.authorize_url,
            "https://login.eveonline.com/v2/oauth/authorize"
        );
        assert_eq!(b.token_url, "https://login.eveonline.com/v2/oauth/token");
        assert_eq!(b.spec_url, "https://esi.evetech.net/meta/openapi.json");
        assert_eq!(b.compatibility_date, "2026-08-18");
        assert_eq!(b.access_token, None);
        assert_eq!(b.spec, None);
    }

    #[test]
    fn test_builder_change_urls() {
        let b = EsiBuilder::new()
            .user_agent("d")
            .base_api_url("http://eve-api/")
            .authorize_url("http://authorize-url/")
            .token_url("http://token-url")
            .spec_url("http://spec-url/")
            .build()
            .unwrap();

        assert_eq!(b.base_api_url, "http://eve-api/");
        assert_eq!(b.authorize_url, "http://authorize-url/");
        assert_eq!(b.token_url, "http://token-url");
        assert_eq!(b.spec_url, "http://spec-url/");
    }

    #[test]
    fn test_builder_missing_value() {
        let res = EsiBuilder::new().build();
        assert!(res.is_err());
        let s = format!("{}", res.unwrap_err());
        assert_eq!(s, "Missing required builder struct value 'user_agent'");
    }

    #[test]
    fn test_builder_with_spec() {
        let spec: Spec = serde_json::from_str(r#"{"paths": {}}"#).unwrap();
        let b = EsiBuilder::new()
            .user_agent("d")
            .spec(Some(spec.clone()))
            .build()
            .unwrap();

        assert_eq!(spec, b.spec.unwrap());
    }

    #[test]
    fn test_builder_to_json_empty() {
        let json = r#"{"compatibility_date":null,"client_id":null,"client_secret":null,"application_auth":null,"callback_url":null,"base_api_url":null,"authorize_url":null,"token_url":null,"spec_url":null,"scope":null,"access_token":null,"access_expiration":null,"refresh_token":null,"user_agent":null,"http_timeout":null,"spec":null}"#;
        assert_eq!(json, serde_json::to_string(&EsiBuilder::new()).unwrap());
    }

    #[test]
    fn test_builder_from_json_filled() {
        let json = r#"{
            "compatibility_date": "2026-08-18",
            "client_id": "a",
            "client_secret": "b",
            "callback_url": "c",
            "scope": "d",
            "access_token": "e",
            "access_expiration": 1,
            "refresh_token": "f",
            "user_agent": "g",
            "http_timeout": 60000,
            "spec": null
          }"#;
        let actual: EsiBuilder = serde_json::from_str(json).unwrap();
        let expected = EsiBuilder::new()
            .compatibility_date("2026-08-18")
            .client_id("a")
            .client_secret("b")
            .callback_url("c")
            .scope("d")
            .access_token(Some("e"))
            .access_expiration(Some(1))
            .refresh_token(Some("f"))
            .user_agent("g")
            .http_timeout(Some(60_000))
            .spec(None);

        assert_eq!(actual, expected);
    }
}
