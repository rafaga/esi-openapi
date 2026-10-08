//! Main logic

use crate::{
    cache::{CacheEntry, ResponseCache},
    client::ErrorLimitStatus::{Limited, NotLimited},
    cursor_page::CursorPage,
    groups::*,
    legacy,
    pkce::{self, PkceVerifier},
    prelude::*,
    rate_limiter::{Acquire, Permit, RateLimiter},
    spec::{Spec, SpecIndex},
};
use base64::engine::{general_purpose::STANDARD as base64, Engine};
use log::{debug, error, warn};
#[cfg(feature = "random_state")]
use rand::{distr::Alphanumeric, RngExt};
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client, Method,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::HashMap,
    str::FromStr,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::RwLock;

const BASE_URL: &str = "https://esi.evetech.net/";
const AUTHORIZE_URL: &str = "https://login.eveonline.com/v2/oauth/authorize";
const TOKEN_URL: &str = "https://login.eveonline.com/v2/oauth/token";
const SPEC_URL: &str = "https://esi.evetech.net/meta/openapi.json";
const ERROR_LIMIT_REMAIN_HEADER: &str = "x-esi-error-limit-remain";
const ERROR_LIMIT_RESET_HEADER: &str = "x-esi-error-limit-reset";
const RATE_LIMIT_GROUP_HEADER: &str = "x-ratelimit-group";
const RATE_LIMIT_LIMIT_HEADER: &str = "x-ratelimit-limit";
const RATE_LIMIT_REMAINING_HEADER: &str = "x-ratelimit-remaining";
const RATE_LIMIT_USED_HEADER: &str = "x-ratelimit-used";

static COMPATIBILITY_HEADER: &str = "X-Compatibility-Date";
static TENANT_HEADER: &str = "X-Tenant";
/// The largest `limit` the spec allows on cursor-paginated operations.
const CURSOR_PAGE_LIMIT: &str = "100";
/// The default compatibility date to use if none is specified
/// in the builder: the latest date listed by
/// `https://esi.evetech.net/meta/compatibility-dates` when this
/// version was released.
pub const COMPATIBILITY_DATE_DEFAULT: &str = "2026-08-18";

/// Response from SSO when exchanging a SSO code for tokens.
#[derive(Debug, Deserialize)]
struct AuthenticateResponse {
    access_token: String,
    expires_in: u64,
    refresh_token: Option<String>,
}

/// Response from SSO when exchanging a refresh token for access token.
#[derive(Debug, Deserialize)]
struct RefreshTokenAuthenticateResponse {
    access_token: String,
    expires_in: u64,
    refresh_token: String,
}

#[derive(Copy, Clone, Debug)]
struct ErrorLimitState {
    remaining_limit: i32,
    expires_at_millis: i64,
}

/// Whether ESI's legacy error limit (`X-Esi-Error-Limit-*` headers) is
/// currently blocking requests from this client.
#[derive(Copy, Clone, Debug)]
pub enum ErrorLimitStatus {
    /// Too many errors: requests are refused until the window resets.
    Limited {
        /// Milliseconds until the error-limit window resets.
        for_millis: i64,
    },
    /// Requests may be made.
    NotLimited,
}

/// What is known about the download of the spec held by an [`Esi`].
#[derive(Clone, Debug)]
struct SpecInfo {
    /// The compatibility date the spec was requested with.
    compatibility_date: String,
    etag: Option<String>,
    last_modified: Option<String>,
    /// Unix time in milliseconds until which the spec needs no new request.
    expires_at: i64,
}

/// Latest rate-limit state ESI reported for one route group.
///
/// ESI uses a floating-window token bucket per application/character
/// pair and route group; see the [ESI rate limiting docs]. The values
/// are those of the most recent response for the group.
///
/// [ESI rate limiting docs]: https://developers.eveonline.com/docs/services/esi/rate-limiting/
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RateLimitStatus {
    /// Route group, from `X-Ratelimit-Group`.
    pub group: String,
    /// Raw `X-Ratelimit-Limit` value, e.g. `150/15m`.
    pub limit: String,
    /// Tokens per window, parsed from `limit` (e.g. `150`).
    pub max_tokens: Option<u64>,
    /// Window length in seconds, parsed from `limit` (e.g. `900` for `15m`).
    pub window_secs: Option<u64>,
    /// Tokens left in the window, from `X-Ratelimit-Remaining`.
    pub remaining: i64,
    /// Tokens consumed by the request that returned these headers,
    /// from `X-Ratelimit-Used`.
    pub used: i64,
    /// Millisecond unix timestamp of the response these values came from.
    pub updated_at_millis: i64,
}

/// Parse an `X-Ratelimit-Limit` value such as `150/15m` into
/// `(tokens, window_secs)`.
fn parse_rate_limit(value: &str) -> (Option<u64>, Option<u64>) {
    let Some((tokens, window)) = value.trim().split_once('/') else {
        return (value.trim().parse().ok(), None);
    };
    let tokens = tokens.trim().parse().ok();
    let window = window.trim();
    let split = window
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(window.len());
    let (amount, unit) = window.split_at(split);
    let amount: Option<u64> = amount.parse().ok();
    let multiplier = match unit {
        "" | "s" => Some(1),
        "m" => Some(60),
        "h" => Some(3600),
        "d" => Some(86_400),
        _ => None,
    };
    let window_secs = amount.zip(multiplier).map(|(a, m)| a * m);
    (tokens, window_secs)
}

/// Which base URL to start with - the public URL for unauthenticated
/// calls, or the authenticated URL for making calls to endpoints that
/// require an access token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestType {
    /// Endpoints that do not require authentication
    Public,
    /// Endpoints that require acting on behalf of an authenticated character
    Authenticated,
}

/// AuthenticationInformation contains data needed to complete the requested authentication flow.
pub struct AuthenticationInformation {
    /// URL to call/pass to users to initiate an authentication and get an auth code from ESI.
    pub authorization_url: String,
    /// If the default feature "random_state" is enabled, the returned state field string will be
    /// random; otherwise it'll be "esi_openapi_unused". The ESI docs link to
    /// [this auth0 page](https://auth0.com/docs/secure/attack-protection/state-parameters)
    /// to explain. You need to check the state yourself when the response from ESI is received.
    pub state: String,
    /// Filled if you've selected PKCE authentication for application.
    /// You will need it to authenticate using the code received from ESI.
    pub pkce_verifier: Option<PkceVerifier>,
}

/// Struct to interact with ESI.
///
/// Construct an instance of this struct using [`EsiBuilder`](./struct.EsiBuilder.html).
///
/// # Example
/// ```rust,no_run
/// use esi_openapi::prelude::EsiBuilder;
/// // the struct must be mutable for some functionality
/// let mut esi = EsiBuilder::new()
///     .user_agent("some user agent")
///     .client_id("your_client_id")
///     .client_secret("your_client_secret")
///     .callback_url("your_callback_url")
///     .build()
///     .unwrap();
/// ```
#[derive(Clone, Debug)]
pub struct Esi {
    pub(crate) compatibility_date: String,
    pub(crate) client_id: Option<String>,
    pub(crate) client_secret: Option<String>,
    pub(crate) callback_url: Option<String>,
    pub(crate) base_api_url: String,
    pub(crate) authorize_url: String,
    pub(crate) token_url: String,
    pub(crate) spec_url: String,
    pub(crate) scope: String,
    pub(crate) application_auth: bool,
    /// The access token from ESI, if set.
    pub access_token: Option<String>,
    /// The millisecond unix timestamp after which the access token expires, if present.
    pub access_expiration: Option<i64>,
    /// The refresh token from ESI, if set.
    pub refresh_token: Option<String>,
    /// HTTP client
    pub(crate) client: Client,
    pub(crate) spec: Option<Spec>,
    /// How and when the spec was downloaded, if it was; see `update_spec`.
    spec_info: Option<SpecInfo>,
    /// Lookup tables built from `spec`.
    index: SpecIndex,
    error_limit_state: Arc<RwLock<Option<ErrorLimitState>>>,
    rate_limits: Arc<RwLock<HashMap<String, RateLimitStatus>>>,
    /// Budgets by route group and access token, used by `rate_limit_policy`.
    limiter: Arc<RateLimiter>,
    /// What to do when a route group has no tokens left.
    pub(crate) rate_limit_policy: RateLimitPolicy,
    /// Cached `GET` responses, when the cache is enabled.
    cache: Option<Arc<RwLock<ResponseCache>>>,
    /// Pages requested at the same time by `fetch_all_pages`.
    pub(crate) page_concurrency: usize,
    /// Language of the responses (`Accept-Language`), if set.
    pub(crate) language: Option<Language>,
    /// Tenant (`X-Tenant`), if set.
    pub(crate) tenant: Option<String>,
}

impl Esi {
    /// Consume the builder, creating an instance of this struct.
    pub(crate) fn from_builder(builder: EsiBuilder) -> EsiResult<Self> {
        let client = builder.construct_client()?;
        let compatibility_date = builder
            .compatibility_date
            .unwrap_or_else(|| COMPATIBILITY_DATE_DEFAULT.to_owned());
        let index = builder
            .spec
            .as_ref()
            .map(SpecIndex::new)
            .unwrap_or_default();
        let e = Esi {
            compatibility_date: compatibility_date.clone(),
            client_id: builder.client_id,
            client_secret: builder.client_secret,
            callback_url: builder.callback_url,
            base_api_url: builder.base_api_url.unwrap_or(BASE_URL.to_string()),
            authorize_url: builder.authorize_url.unwrap_or(AUTHORIZE_URL.to_string()),
            token_url: builder.token_url.unwrap_or(TOKEN_URL.to_string()),
            spec_url: builder.spec_url.unwrap_or(SPEC_URL.to_string()),
            scope: builder.scope.unwrap_or_else(|| "".to_owned()),
            application_auth: builder.application_auth.unwrap_or(false),
            access_token: builder.access_token,
            access_expiration: builder.access_expiration,
            refresh_token: builder.refresh_token,
            client,
            spec: builder.spec,
            spec_info: None,
            index,
            error_limit_state: Arc::new(RwLock::new(None)),
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
            limiter: Arc::new(RateLimiter::default()),
            rate_limit_policy: builder.rate_limit_policy.unwrap_or_default(),
            page_concurrency: builder.page_concurrency.unwrap_or(4).max(1),
            language: builder.language,
            tenant: builder.tenant.clone(),
            cache: builder.cache_enabled.unwrap_or(false).then(|| {
                Arc::new(RwLock::new(ResponseCache::with_limits(
                    builder
                        .cache_max_entries
                        .unwrap_or(crate::cache::DEFAULT_MAX_ENTRIES),
                    builder.cache_max_bytes,
                )))
            }),
        };
        Ok(e)
    }

    /// Get the OpenAPI spec from ESI and store it in this struct.
    ///
    /// The spec is requested with this struct's compatibility date
    /// (`X-Compatibility-Date`), so the paths match the API version
    /// that later requests will use.
    ///
    /// If you are making use of the `try_get_endpoint_for_op_id`,
    /// then this function will be called there when needed
    /// (which should only really be when the struct is
    /// constructed unless the struct is kept in memory for a very
    /// long time). When using `get_endpoint_for_op_id` however,
    /// you are responsible for calling this function beforehand.
    ///
    /// # Example
    /// ```rust,no_run
    /// # async fn run() {
    /// # use esi_openapi::prelude::*;
    /// # let mut esi = EsiBuilder::new()
    /// #     .user_agent("some user agent")
    /// #     .client_id("your_client_id")
    /// #     .client_secret("your_client_secret")
    /// #     .callback_url("your_callback_url")
    /// #     .build()
    /// #     .unwrap();
    /// esi.update_spec().await.unwrap();
    /// # }
    /// ```
    ///
    /// This always makes a request. If the spec was downloaded before with the
    /// same compatibility date and the server gave it an `ETag` or
    /// `Last-Modified`, the request is conditional (`If-None-Match` /
    /// `If-Modified-Since`) and a `304 Not Modified` keeps the spec in memory
    /// instead of downloading it again. To skip the request while the spec is
    /// still fresh, use [`Esi::ensure_spec_fresh`].
    pub async fn update_spec(&mut self) -> EsiResult<()> {
        debug!(
            "Updating spec with compatibility date {}",
            self.compatibility_date
        );
        self.assert_not_error_limited().await?;
        let mut request = self.client.get(&self.spec_url).header(
            COMPATIBILITY_HEADER,
            HeaderValue::from_str(&self.compatibility_date)?,
        );
        let known = self.spec_info.as_ref().filter(|info| {
            self.spec.is_some() && info.compatibility_date == self.compatibility_date
        });
        if let Some(info) = known {
            if let Some(etag) = &info.etag {
                request = request.header(header::IF_NONE_MATCH, HeaderValue::from_str(etag)?);
            }
            if let Some(modified) = &info.last_modified {
                request =
                    request.header(header::IF_MODIFIED_SINCE, HeaderValue::from_str(modified)?);
            }
        }
        let resp = request.send().await?;
        self.process_response_headers(resp.headers()).await?;
        if resp.status() == reqwest::StatusCode::NOT_MODIFIED && known.is_some() {
            debug!("The spec has not changed");
            let expires_at = Self::spec_expiry(resp.headers())?;
            if let Some(info) = &mut self.spec_info {
                info.expires_at = expires_at;
            }
            return Ok(());
        }
        if !resp.status().is_success() {
            error!("Got status {} when requesting spec", resp.status());
            return Err(Self::status_error(resp.status().as_u16(), resp.headers()));
        }
        let info = SpecInfo {
            compatibility_date: self.compatibility_date.clone(),
            etag: Self::header_text(resp.headers(), "etag"),
            last_modified: Self::header_text(resp.headers(), "last-modified"),
            expires_at: Self::spec_expiry(resp.headers())?,
        };
        let data: Spec = resp.json().await?;
        self.index = SpecIndex::new(&data);
        self.spec = Some(data);
        self.spec_info = Some(info);
        Ok(())
    }

    /// Make sure the spec is loaded and still fresh, requesting it only if it is
    /// not: it has not been downloaded by this struct (a spec given to the
    /// builder has no known age), it was downloaded with another compatibility
    /// date, or its `Cache-Control: max-age` has passed. Use it before a long
    /// run of calls instead of [`Esi::update_spec`] to avoid needless downloads.
    pub async fn ensure_spec_fresh(&mut self) -> EsiResult<()> {
        let now = current_time_millis()?;
        let fresh = self.spec.is_some()
            && self.spec_info.as_ref().is_some_and(|info| {
                info.compatibility_date == self.compatibility_date && info.expires_at > now
            });
        if fresh {
            debug!("The spec is still fresh");
            return Ok(());
        }
        self.update_spec().await
    }

    /// When a spec downloaded now needs a new request: `max-age` after now.
    fn spec_expiry(headers: &HeaderMap) -> EsiResult<i64> {
        let max_age = ResponseCache::max_age(headers).unwrap_or(0);
        Ok(current_time_millis()?.saturating_add(max_age.saturating_mul(1000)))
    }

    /// Ensure the user has specified all required EVE Developer App information.
    fn check_client_info(&self) -> EsiResult<()> {
        for (name, value) in &[
            ("client_id", &self.client_id),
            ("callback_url", &self.callback_url),
        ] {
            if value.is_none() {
                return Err(EsiError::EmptyClientValue(name.to_string()));
            }
        }

        if self.client_secret.is_none() {
            if !self.application_auth {
                return Err(EsiError::MissingAuthenticationFlowInformation);
            }
        } else if self.application_auth {
            return Err(EsiError::MissingAuthenticationFlowInformation);
        }

        Ok(())
    }

    /// Generate and return the URL required for the user to grant you an auth code, as wells as
    /// infos for future authentication request.
    ///
    /// You can inspect the URL returned by ESI to your web service to ensure it matches.
    /// No checking is done by `esi-openapi`.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use esi_openapi::prelude::*;
    /// # let mut esi = EsiBuilder::new()
    /// #     .user_agent("some user agent")
    /// #     .client_id("your_client_id")
    /// #     .client_secret("your_client_secret")
    /// #     .callback_url("your_callback_url")
    /// #     .build()
    /// #     .unwrap();
    /// let auth_info = esi.get_authorize_url().unwrap();
    /// // then send your user to that URL
    /// let url = auth_info.authorization_url;
    /// ```
    ///
    /// If you opted to not include client information in
    /// the EsiBuilder flow, then this function will return
    /// an error instead.
    ///
    /// [this auth0 page]: https://auth0.com/docs/secure/attack-protection/state-parameters
    pub fn get_authorize_url(&self) -> EsiResult<AuthenticationInformation> {
        self.check_client_info()?;
        #[cfg(feature = "random_state")]
        let state = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        #[cfg(not(feature = "random_state"))]
        let state = "esi_openapi_unused".to_string();
        let mut url = format!(
            "{}?response_type=code&redirect_uri={}&client_id={}&scope={}&state={state}",
            self.authorize_url,
            self.callback_url.as_ref().unwrap(),
            self.client_id.as_ref().unwrap(),
            self.scope
        );
        let mut pkce_verifier = None;
        // PKCE can be theoretically combined with client secret, but not sure if ESI supports it
        if self.client_secret.is_none() && self.application_auth {
            let pkce = pkce::generate()?;
            pkce_verifier = Some(pkce.verifier);
            url = format!(
                "{}&code_challenge={}&code_challenge_method=S256",
                url, pkce.challenge
            )
        }
        Ok(AuthenticationInformation {
            authorization_url: url,
            state,
            pkce_verifier,
        })
    }

    fn get_auth_headers(&self) -> EsiResult<HeaderMap> {
        self.check_client_info()?;
        let mut map = HeaderMap::new();
        if let Some(ref secret) = self.client_secret {
            let value = base64
                .encode(format!("{}:{secret}", self.client_id.as_ref().unwrap()))
                .replace(['\n', ' '], "");
            map.insert(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Basic {value}"))?,
            );
        }
        map.insert(
            header::HOST,
            HeaderValue::from_static("login.eveonline.com"),
        );
        Ok(map)
    }

    /// Authenticate with ESI, exchanging a code from the authorize flow
    /// for an access token that is used to make authenticated calls to ESI.
    ///
    /// Note that this is one of the functions that requires the struct be
    /// mutable, as the struct mutates to include the resulting access token.
    ///
    /// If the "validate_jwt" feature is enabled (by default), then the access
    /// token's claims will be returned. If the feature is not enabled, then
    /// the returned value will be `None`.
    ///
    /// # Example (client secret)
    /// ```rust,no_run
    /// # async fn run() {
    /// # use esi_openapi::prelude::*;
    /// # let mut esi = EsiBuilder::new()
    /// #     .user_agent("some user agent")
    /// #     .client_id("your_client_id")
    /// #     .client_secret("your_client_secret")
    /// #     .callback_url("your_callback_url")
    /// #     .build()
    /// #     .unwrap();
    /// let claims = esi.authenticate("abcdef...", None).await.unwrap();
    /// # }
    /// ```
    ///
    /// # Example (PKCE/Application authentication)
    /// ```rust,no_run
    /// # use esi_openapi::prelude::*;
    ///  async fn run() {
    /// # let mut esi = EsiBuilder::new()
    /// #     .user_agent("some user agent")
    /// #     .client_id("your_client_id")
    /// #     .callback_url("your_callback_url")
    /// #     .enable_application_authentication(true)
    /// #     .build()
    /// #     .unwrap();
    /// # let auth_infos = esi.get_authorize_url().unwrap();
    /// # let claims = esi.authenticate("abcdef...", auth_infos.pkce_verifier).await.unwrap();
    /// # }
    /// ```
    pub async fn authenticate(
        &mut self,
        code: &str,
        pkce_verifier: Option<PkceVerifier>,
    ) -> EsiResult<Option<TokenClaims>> {
        debug!("Authenticating with code {code}");
        self.assert_not_error_limited().await?;
        let mut body = HashMap::from([("grant_type", "authorization_code"), ("code", code)]);
        if self.application_auth {
            let option = self.client_id.as_ref();
            body.insert("client_id", option.unwrap());
            body.insert("code_verifier", pkce_verifier.as_ref().unwrap());
        }

        let resp = self
            .client
            .post(&self.token_url)
            .headers(self.get_auth_headers()?)
            .form(&body)
            .send()
            .await?;
        if resp.status() != 200 {
            warn!(
                "Got status {} when making call to authenticate",
                resp.status()
            );
            return Err(EsiError::InvalidStatusCode(resp.status().as_u16()));
        }
        self.process_error_limit_headers(resp.headers()).await?;
        let data: AuthenticateResponse = resp.json().await?;
        #[allow(unused_variables)]
        let claim_data: Option<TokenClaims> = None;
        #[cfg(feature = "validate_jwt")]
        let claim_data = Some(
            crate::jwt_util::validate_jwt(
                &self.client,
                &data.access_token,
                self.client_id.as_ref().unwrap(),
            )
            .await?,
        );
        self.access_token = Some(data.access_token);
        // the response's "expires_in" field is seconds but need millis
        self.access_expiration = Some((data.expires_in as i64 * 1_000) + current_time_millis()?);
        self.refresh_token = data.refresh_token;
        Ok(claim_data)
    }

    /// Authenticate via a previously-fetched refresh token.
    ///
    /// The functionality of a refresh token allows re-authenticating this struct
    /// instance without prompting the user to log into EVE SSO again. When the user
    /// is authenticated in that manner, a refresh token is returned and available
    /// via the `refresh_token` struct field. Store this securely should you wish
    /// to later make authenticate calls for that user.
    ///
    /// # Example
    /// ```rust,no_run
    /// # async fn run() {
    /// # use esi_openapi::prelude::*;
    /// # let mut esi = EsiBuilder::new()
    /// #     .user_agent("some user agent")
    /// #     .client_id("your_client_id")
    /// #     .client_secret("your_client_secret")
    /// #     .callback_url("your_callback_url")
    /// #     .build()
    /// #     .unwrap();
    /// esi.use_refresh_token("abcdef...").await.unwrap();
    /// # }
    /// ```
    pub async fn use_refresh_token(&mut self, refresh_token: &str) -> EsiResult<()> {
        self.refresh_access_token(Some(refresh_token)).await?;
        Ok(())
    }

    /// Authenticate via a refresh token given as input, or using the internal refresh_token if it's available.
    ///
    /// The functionality of a refresh token allows re-authenticating this struct
    /// instance without prompting the user to log into EVE SSO again. When the user
    /// is authenticated in that manner, a refresh token is returned and available
    /// via the `refresh_token` struct field. Store this securely should you wish
    /// to later make authenticate calls for that user.
    ///
    /// # Example with internal token
    /// ```rust,no_run
    /// # async fn run() {
    /// # use esi_openapi::prelude::*;
    /// # let mut esi = EsiBuilder::new()
    /// #     .user_agent("some user agent")
    /// #     .refresh_token(Some("MyRefreshToken"))
    /// #     .build()
    /// #     .unwrap();
    /// esi.refresh_access_token(None).await.unwrap();
    /// # }
    /// ```
    /// # Example with input token
    /// ```rust,no_run
    /// # async fn run() {
    /// # use esi_openapi::prelude::*;
    /// # let mut esi = EsiBuilder::new()
    /// #     .user_agent("some user agent")
    /// #     .build()
    /// #     .unwrap();
    /// esi.refresh_access_token(Some("MyRefreshToken")).await.unwrap();
    /// # }
    /// ```
    pub async fn refresh_access_token(&mut self, refresh_token: Option<&str>) -> EsiResult<()> {
        self.assert_not_error_limited().await?;
        let token = if let Some(token) = refresh_token {
            token.to_string()
        } else if let Some(token) = self.refresh_token.clone() {
            token
        } else {
            return Err(EsiError::NoRefreshTokenAvailable);
        };

        debug!("Authenticating with refresh token");
        let mut body = HashMap::from([("grant_type", "refresh_token"), ("refresh_token", &token)]);
        if self.application_auth {
            let option = self.client_id.as_ref();
            body.insert("client_id", option.unwrap());
        }
        let resp = self
            .client
            .post(&self.token_url)
            .headers(self.get_auth_headers()?)
            .form(&body)
            .send()
            .await?;
        self.process_error_limit_headers(resp.headers()).await?;
        if resp.status() != 200 {
            warn!(
                "Got status {} when making call to authenticate via a refresh token",
                resp.status()
            );
            return Err(EsiError::InvalidStatusCode(resp.status().as_u16()));
        }
        let data: RefreshTokenAuthenticateResponse = resp.json().await?;
        self.access_token = Some(data.access_token);
        // the response's "expires_in" field is seconds, need millis
        self.access_expiration = Some((data.expires_in as i64 * 1_000) + current_time_millis()?);
        self.refresh_token = Some(data.refresh_token);
        Ok(())
    }

    /// Make a request to ESI.
    ///
    /// This is mainly used as the underlying function for this
    /// library when making calls to ESI; the other functions that
    /// you should primarily be using contain more functionality,
    /// including matching endpoint with deserialization struct,
    /// evaluating & replacing URL parameters, etc.
    ///
    /// In the event that there is not a wrapper function for the
    /// endpoint that you want to use, you can use this function
    /// to make an API call without waiting for the library to
    /// be updated.
    ///
    /// # Example
    /// ```rust,no_run
    /// # async fn run() {
    /// # use serde::Deserialize;
    /// # use esi_openapi::prelude::*;
    /// # let mut esi = EsiBuilder::new()
    /// #     .user_agent("some user agent")
    /// #     .client_id("your_client_id")
    /// #     .client_secret("your_client_secret")
    /// #     .callback_url("your_callback_url")
    /// #     .build()
    /// #     .unwrap();
    /// #[derive(Deserialize)]
    /// struct ReturnedData {}
    /// let data: ReturnedData = esi.query("GET", RequestType::Public, "abc", None, None).await.unwrap();
    /// # }
    /// ```
    pub async fn query<T: DeserializeOwned>(
        &self,
        method: &str,
        request_type: RequestType,
        endpoint: &str,
        query: Option<&[(&str, &str)]>,
        body: Option<&str>,
    ) -> EsiResult<T> {
        let (text, _) = self
            .send_request(method, request_type, endpoint, query, body)
            .await?;
        Self::parse_body(&text)
    }

    /// Like [`Esi::query`], but also returns the total number of pages
    /// reported by the `X-Pages` response header, if present.
    ///
    /// # Example
    /// ```rust,no_run
    /// # async fn run() {
    /// # use esi_openapi::prelude::*;
    /// # let esi = EsiBuilder::new()
    /// #     .user_agent("some user agent")
    /// #     .client_id("your_client_id")
    /// #     .client_secret("your_client_secret")
    /// #     .callback_url("your_callback_url")
    /// #     .build()
    /// #     .unwrap();
    /// let (data, pages): (Vec<serde_json::Value>, Option<i64>) = esi
    ///     .query_with_pages("GET", RequestType::Public, "some/path", Some(&[("page", "2")]))
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn query_with_pages<T: DeserializeOwned>(
        &self,
        method: &str,
        request_type: RequestType,
        endpoint: &str,
        query: Option<&[(&str, &str)]>,
    ) -> EsiResult<(T, Option<i64>)> {
        let (text, headers) = self
            .send_request(method, request_type, endpoint, query, None)
            .await?;
        Ok((Self::parse_body(&text)?, Self::pages_header(&headers)))
    }

    /// Fetch every page of an endpoint paginated with the `page` query
    /// parameter and return the items of all pages, in order.
    ///
    /// The first page is requested with `page=1`; the `X-Pages` header of its
    /// response says how many pages follow, and those are requested
    /// concurrently, [`EsiBuilder::page_concurrency`] at a time. Pass the other
    /// query parameters of the endpoint in `query`, without `page`. `max_pages`
    /// limits how many pages are fetched (`None` for all of them).
    pub async fn fetch_all_pages<T: DeserializeOwned>(
        &self,
        request_type: RequestType,
        endpoint: &str,
        query: Option<&[(&str, &str)]>,
        max_pages: Option<i64>,
    ) -> EsiResult<Vec<T>> {
        let (mut items, total) = self
            .fetch_page::<T>(request_type, endpoint, query, 1)
            .await?;
        let last = total.unwrap_or(1).min(max_pages.unwrap_or(i64::MAX));
        let step = i64::try_from(self.page_concurrency).unwrap_or(1);
        let mut next: i64 = 2;
        while next <= last {
            let end = (next + step - 1).min(last);
            let batches = futures_util::future::try_join_all(
                (next..=end).map(|page| self.fetch_page::<T>(request_type, endpoint, query, page)),
            )
            .await?;
            for (mut batch, _) in batches {
                items.append(&mut batch);
            }
            next = end + 1;
        }
        Ok(items)
    }

    /// Send a `POST` whose body is a JSON array and return the answers joined
    /// in order, splitting `items` into requests of at most `chunk_size` items.
    ///
    /// Use it for endpoints that cap the length of the array (such as
    /// `characters/affiliation`, at 1000 ids). The chunks are sent
    /// concurrently, [`EsiBuilder::page_concurrency`] at a time. The answer to
    /// each chunk must be an array; an empty `items` sends no request and
    /// returns an empty list. Duplicates are not removed, and the spec asks for
    /// unique items on most of these endpoints.
    pub async fn post_chunked<B: Serialize, T: DeserializeOwned>(
        &self,
        request_type: RequestType,
        endpoint: &str,
        items: &[B],
        chunk_size: usize,
    ) -> EsiResult<Vec<T>> {
        let chunks: Vec<&[B]> = items.chunks(chunk_size.max(1)).collect();
        let mut results: Vec<T> = Vec::new();
        for group in chunks.chunks(self.page_concurrency) {
            let answers = futures_util::future::try_join_all(
                group
                    .iter()
                    .map(|chunk| self.post_chunk::<B, T>(request_type, endpoint, chunk)),
            )
            .await?;
            for mut answer in answers {
                results.append(&mut answer);
            }
        }
        Ok(results)
    }

    /// Send one chunk of a `POST` with an array body.
    async fn post_chunk<B: Serialize, T: DeserializeOwned>(
        &self,
        request_type: RequestType,
        endpoint: &str,
        chunk: &[B],
    ) -> EsiResult<Vec<T>> {
        let body = serde_json::to_string(chunk)?;
        self.query("POST", request_type, endpoint, None, Some(&body))
            .await
    }

    /// Request one page of a `page`-paginated endpoint.
    async fn fetch_page<T: DeserializeOwned>(
        &self,
        request_type: RequestType,
        endpoint: &str,
        query: Option<&[(&str, &str)]>,
        page: i64,
    ) -> EsiResult<(Vec<T>, Option<i64>)> {
        let page_text = page.to_string();
        let mut params: Vec<(&str, &str)> = query.unwrap_or(&[]).to_vec();
        params.push(("page", page_text.as_str()));
        self.query_with_pages("GET", request_type, endpoint, Some(&params))
            .await
    }

    /// Fetch every page of an endpoint paginated with cursors (`x-pagination:
    /// cursor`) and return the items of all pages, in order.
    ///
    /// `items_key` is the name of the array in the response that holds the
    /// records, such as `"projects"` or `"listings"`. The walk follows the
    /// `cursor.after` value of each response until a page has no records or no
    /// further cursor. Pass the other query parameters of the endpoint (such as
    /// `limit`) in `query`, without `after` or `before`. Unless `query` has a
    /// `limit`, the maximum the spec allows (100, against a default of 10) is
    /// requested to need fewer calls. `max_pages` limits how many pages are
    /// fetched (`None` for all of them).
    pub async fn fetch_all_cursor<T: DeserializeOwned>(
        &self,
        request_type: RequestType,
        endpoint: &str,
        query: Option<&[(&str, &str)]>,
        items_key: &str,
        max_pages: Option<i64>,
    ) -> EsiResult<Vec<T>> {
        let mut items: Vec<T> = Vec::new();
        let mut after = String::from("0");
        let mut fetched: i64 = 0;
        loop {
            let mut params: Vec<(&str, &str)> = query.unwrap_or(&[]).to_vec();
            if !params.iter().any(|(key, _)| *key == "limit") {
                params.push(("limit", CURSOR_PAGE_LIMIT));
            }
            params.push(("after", after.as_str()));
            let (text, _) = self
                .send_request("GET", request_type, endpoint, Some(&params), None)
                .await?;
            let page = CursorPage::<T>::parse(&text, items_key)?;
            fetched += 1;
            let empty = page.records.is_empty();
            let next = page.next;
            items.extend(page.records);
            match next {
                Some(next)
                    if !empty && next != after && fetched < max_pages.unwrap_or(i64::MAX) =>
                {
                    after = next;
                }
                _ => return Ok(items),
            }
        }
    }

    /// The total number of pages from the `X-Pages` header.
    fn pages_header(headers: &HeaderMap) -> Option<i64> {
        headers
            .get("x-pages")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.trim().parse().ok())
    }

    /// Read a response body as JSON. A body that is empty (such as `204 No
    /// Content`) is read as JSON `null`, so `()` and `Option<_>` return types
    /// work for it.
    fn parse_body<T: DeserializeOwned>(text: &str) -> EsiResult<T> {
        let text = if text.trim().is_empty() { "null" } else { text };
        Ok(serde_json::from_str(text)?)
    }

    /// Send a request and return the body text and the response headers.
    async fn send_request(
        &self,
        method: &str,
        request_type: RequestType,
        endpoint: &str,
        query: Option<&[(&str, &str)]>,
        body: Option<&str>,
    ) -> EsiResult<(String, HeaderMap)> {
        debug!("Making {request_type:?} {method} request to {endpoint} with query: {query:?}");
        let cache_key = self.cache_key(method, &request_type, endpoint, query);
        let mut stored: Option<CacheEntry> = None;
        if let Some(key) = &cache_key {
            if let Some(entry) = self.cache_lookup(key).await {
                if entry.expires_at > current_time_millis()? {
                    debug!("Serving {endpoint} from the cache");
                    return Ok((entry.body, entry.headers));
                }
                stored = Some(entry);
            }
        }
        self.assert_not_error_limited().await?;
        self.check_authentication(&request_type)?;
        let bucket = self.bucket_key(method, &request_type, endpoint);
        let _permit = match &bucket {
            Some(key) => Some(self.acquire_permit(key).await?),
            None => None,
        };
        let mut headers = self.request_headers(&request_type)?;
        if let Some(entry) = &stored {
            if let Some(etag) = &entry.etag {
                headers.insert(header::IF_NONE_MATCH, HeaderValue::from_str(etag)?);
            }
            if let Some(modified) = &entry.last_modified {
                headers.insert(header::IF_MODIFIED_SINCE, HeaderValue::from_str(modified)?);
            }
        }
        let url = format!("{}{endpoint}", self.base_api_url);
        let mut req_builder = self
            .client
            .request(Method::from_str(method)?, &url)
            .headers(headers)
            .query(query.unwrap_or(&[]));
        req_builder = match body {
            Some(b) => req_builder.body(b.to_owned()),
            None => req_builder,
        };
        let req = req_builder.build()?;
        let resp = self.client.execute(req).await?;
        let rate_limit = self.process_response_headers(resp.headers()).await?;
        if let Some(key) = &bucket {
            self.record_budget(key, rate_limit.as_ref(), resp.status(), resp.headers())?;
        }
        if resp.status() == reqwest::StatusCode::NOT_MODIFIED {
            if let (Some(key), Some(entry)) = (&cache_key, stored) {
                debug!("{endpoint} not modified; reusing the cached body");
                let expires_at = self.cache_expiry(endpoint, resp.headers())?;
                self.cache_refresh(key, expires_at).await;
                return Ok((entry.body, entry.headers));
            }
        }
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            if matches!(status, 404 | 410) {
                if let Some(ttl) = self.index.tombstone_ttl(endpoint) {
                    return Err(EsiError::Gone {
                        status,
                        tombstone_ttl_secs: ttl,
                    });
                }
            }
            return Err(Self::status_error(status, resp.headers()));
        }
        let headers = resp.headers().clone();
        let text = resp.text().await?;
        if let Some(key) = cache_key {
            let entry = CacheEntry {
                etag: Self::header_text(&headers, "etag"),
                last_modified: Self::header_text(&headers, "last-modified"),
                body: text.clone(),
                headers: headers.clone(),
                expires_at: self.cache_expiry(endpoint, &headers)?,
                last_used: 0,
            };
            self.cache_store(key, entry).await?;
        }
        Ok((text, headers))
    }

    /// The cache key of a request, if the cache is enabled and the request is a `GET`.
    fn cache_key(
        &self,
        method: &str,
        request_type: &RequestType,
        endpoint: &str,
        query: Option<&[(&str, &str)]>,
    ) -> Option<String> {
        self.cache.as_ref()?;
        if !method.eq_ignore_ascii_case("GET") {
            return None;
        }
        let token = match request_type {
            RequestType::Authenticated => self.access_token.as_deref(),
            RequestType::Public => None,
        };
        let url = format!("{}{endpoint}", self.base_api_url);
        let variant = format!(
            "{}|{}",
            self.language.map_or("", |l| l.as_str()),
            self.tenant.as_deref().unwrap_or("")
        );
        Some(ResponseCache::key(
            token,
            &variant,
            &url,
            query.unwrap_or(&[]),
        ))
    }

    /// The cached entry for a key, marking it as recently used.
    async fn cache_lookup(&self, key: &str) -> Option<CacheEntry> {
        self.cache.as_ref()?.write().await.lookup(key)
    }

    async fn cache_store(&self, key: String, entry: CacheEntry) -> EsiResult<()> {
        if let Some(cache) = &self.cache {
            cache
                .write()
                .await
                .insert(key, entry, current_time_millis()?);
        }
        Ok(())
    }

    async fn cache_refresh(&self, key: &str, expires_at: i64) {
        if let Some(cache) = &self.cache {
            cache.write().await.refresh(key, expires_at);
        }
    }

    /// When a response stored now stops being served without revalidation: the
    /// `x-client-cache-ttl` of the operation in the spec, else the `max-age`
    /// of the response, else immediately.
    fn cache_expiry(&self, endpoint: &str, headers: &HeaderMap) -> EsiResult<i64> {
        let ttl = self
            .index
            .client_cache_ttl(endpoint)
            .or_else(|| ResponseCache::max_age(headers))
            .unwrap_or(0);
        Ok(current_time_millis()? + ttl * 1000)
    }

    fn header_text(headers: &HeaderMap, name: &str) -> Option<String> {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
    }

    /// The number of responses held by the cache (0 when it is disabled).
    pub async fn cache_len(&self) -> usize {
        match &self.cache {
            Some(cache) => cache.read().await.len(),
            None => 0,
        }
    }

    /// The approximate size in bytes of the responses held by the cache (0 when
    /// it is disabled). See [`EsiBuilder::cache_max_bytes`].
    pub async fn cache_bytes(&self) -> usize {
        match &self.cache {
            Some(cache) => cache.read().await.bytes(),
            None => 0,
        }
    }

    /// For an authenticated request, fails unless there is a valid, unexpired
    /// access token.
    fn check_authentication(&self, request_type: &RequestType) -> EsiResult<()> {
        if *request_type != RequestType::Authenticated {
            return Ok(());
        }
        if self.access_token.is_none() {
            return Err(EsiError::MissingAuthentication);
        }
        if self.access_expiration.unwrap() < current_time_millis()? {
            return Err(EsiError::AccessTokenExpired);
        }
        Ok(())
    }

    /// The per-request headers: the authorization header, if authenticated,
    /// and the compatibility date.
    fn request_headers(&self, request_type: &RequestType) -> EsiResult<HeaderMap> {
        let mut map = HeaderMap::new();
        // The 'user-agent' and 'content-type' headers are set in the default headers
        // from the builder, so all that's required here is to set the authorization
        // header, if present, and the compatibility date.
        if *request_type == RequestType::Authenticated {
            if let Some(at) = &self.access_token {
                map.insert(
                    header::AUTHORIZATION,
                    HeaderValue::from_str(&format!("Bearer {at}"))?,
                );
            }
        }
        map.insert(
            COMPATIBILITY_HEADER,
            HeaderValue::from_str(&self.compatibility_date)?,
        );
        if let Some(language) = self.language {
            map.insert(
                header::ACCEPT_LANGUAGE,
                HeaderValue::from_static(language.as_str()),
            );
        }
        if let Some(tenant) = &self.tenant {
            map.insert(TENANT_HEADER, HeaderValue::from_str(tenant)?);
        }
        Ok(map)
    }

    /// Resolve an `operationId` to a URL path utilizing the OpenAPI spec.
    ///
    /// Operation IDs are those of the ESI OpenAPI spec, e.g.
    /// `GetMarketsRegionIdOrders`. rfesi's legacy snake_case IDs
    /// (e.g. `get_markets_region_id_orders`) are still accepted with a
    /// deprecation warning until 0.2.0.
    ///
    /// If the spec has not yet been retrieved when calling this function,
    /// an API call will be made to ESI to fetch that data (thus the
    /// async signature of this function). If you don't need that help (by
    /// explicitly making a call to `update_spec` prior) then you can use
    /// the `get_endpoint_for_op_id` function, which is synchronous.
    ///
    /// Note that when making use of this function along with `query`, you
    /// are responsible for resolving any/all URL parameters that the endpoint
    /// may contain.
    ///
    /// # Example
    /// ```rust,no_run
    /// # async fn run() {
    /// # use esi_openapi::prelude::*;
    /// # let mut esi = EsiBuilder::new()
    /// #     .user_agent("some user agent")
    /// #     .client_id("your_client_id")
    /// #     .client_secret("your_client_secret")
    /// #     .callback_url("your_callback_url")
    /// #     .build()
    /// #     .unwrap();
    /// let endpoint = esi
    ///     .try_get_endpoint_for_op_id("GetAlliancesAllianceIdContactsLabels")
    ///     .await
    ///     .unwrap();
    /// # }
    /// ```
    pub async fn try_get_endpoint_for_op_id(&mut self, op_id: &str) -> EsiResult<String> {
        if self.spec.is_none() {
            debug!("Spec is `None`; must fetch before looking up op_id");
            self.update_spec().await?;
        }
        self.get_endpoint_for_op_id(op_id)
    }

    /// Resolve an `operationId` to a URL path utilizing the OpenAPI spec.
    ///
    /// Operation IDs are those of the ESI OpenAPI spec, e.g.
    /// `GetMarketsRegionIdOrders`. rfesi's legacy snake_case IDs
    /// (e.g. `get_markets_region_id_orders`) are still accepted with a
    /// deprecation warning until 0.2.0.
    ///
    /// If the spec has not yet been retrieved when calling this function,
    /// this function will return an error.
    ///
    /// Note that when making use of this function along with `query`, you
    /// are responsible for resolving any/all URL parameters that the endpoint
    /// may contain.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use esi_openapi::prelude::*;
    /// # let mut esi = EsiBuilder::new()
    /// #     .user_agent("some user agent")
    /// #     .client_id("your_client_id")
    /// #     .client_secret("your_client_secret")
    /// #     .callback_url("your_callback_url")
    /// #     .build()
    /// #     .unwrap();
    /// let endpoint = esi.get_endpoint_for_op_id("GetAlliancesAllianceIdContactsLabels").unwrap();
    /// ```
    pub fn get_endpoint_for_op_id(&self, op_id: &str) -> EsiResult<String> {
        if self.spec.is_none() {
            return Err(EsiError::EmptySpec);
        }
        if let Some(path) = self.index.path(op_id) {
            return Ok(path.to_owned());
        }
        if let Some(new_id) = legacy::openapi_id_for(op_id) {
            warn!(
                "operationId '{op_id}' is a deprecated Swagger ID; use '{new_id}' instead (legacy IDs will be removed in 0.2.0)"
            );
            if let Some(path) = self.index.path(new_id) {
                return Ok(path.to_owned());
            }
        }
        Err(EsiError::UnknownOperationID(op_id.to_owned()))
    }

    /// The operation's metadata from the spec.
    fn spec_operation(&self, op_id: &str) -> EsiResult<&crate::spec::SpecPathMethod> {
        let spec = self.spec.as_ref().ok_or(EsiError::EmptySpec)?;
        self.index
            .operation(spec, op_id)
            .ok_or_else(|| EsiError::UnknownOperationID(op_id.to_owned()))
    }

    /// The OAuth2 scopes an operation needs, from the spec. Public operations need none.
    pub fn required_scopes(&self, op_id: &str) -> EsiResult<Vec<String>> {
        Ok(self.spec_operation(op_id)?.scopes())
    }

    /// The scopes an operation needs that are not in `granted`, a space-separated
    /// scope list such as the `scope` value of a token. Use it to fail early instead
    /// of making a request ESI will answer with `401`.
    pub fn missing_scopes(&self, op_id: &str, granted: &str) -> EsiResult<Vec<String>> {
        let granted: Vec<&str> = granted.split_whitespace().collect();
        Ok(self
            .required_scopes(op_id)?
            .into_iter()
            .filter(|scope| !granted.contains(&scope.as_str()))
            .collect())
    }

    /// The corporation roles of which the character needs at least one for an
    /// operation (empty when it needs none).
    pub fn required_roles(&self, op_id: &str) -> EsiResult<Vec<String>> {
        Ok(self.spec_operation(op_id)?.required_roles.clone())
    }

    /// The rate limit each route group declares in the spec, keyed by group.
    ///
    /// These are the budgets before any response arrives; [`Esi::rate_limit_status`]
    /// has the live values once ESI has answered for a group.
    pub fn declared_rate_limits(&self) -> EsiResult<HashMap<String, crate::spec::SpecRateLimit>> {
        let spec = self.spec.as_ref().ok_or(EsiError::EmptySpec)?;
        Ok(spec.rate_limit_groups())
    }

    /// Build the error for a non-success response status.
    fn status_error(status: u16, headers: &HeaderMap) -> EsiError {
        if status == 429 {
            let header_str = |name: &str| headers.get(name).and_then(|v| v.to_str().ok());
            let group = header_str(RATE_LIMIT_GROUP_HEADER).map(str::to_owned);
            let retry_after_secs = Self::retry_after_secs(headers);
            warn!("Rate limited by ESI (group {group:?}); retry after {retry_after_secs:?}s");
            return EsiError::RateLimited {
                group,
                retry_after_secs,
            };
        }
        EsiError::InvalidStatusCode(status)
    }

    /// Record the error-limit and rate-limit headers of a response.
    ///
    /// Returns the rate-limit status the response reported, if it has one.
    async fn process_response_headers(
        &self,
        headers: &HeaderMap,
    ) -> Result<Option<RateLimitStatus>, EsiError> {
        self.process_error_limit_headers(headers).await?;
        self.process_rate_limit_headers(headers).await
    }

    async fn process_rate_limit_headers(
        &self,
        headers: &HeaderMap,
    ) -> Result<Option<RateLimitStatus>, EsiError> {
        let Some(group) = headers.get(RATE_LIMIT_GROUP_HEADER) else {
            return Ok(None);
        };
        let group = group.to_str()?.to_owned();
        let limit = match headers.get(RATE_LIMIT_LIMIT_HEADER) {
            Some(v) => v.to_str()?.to_owned(),
            None => String::new(),
        };
        let parse_i64 = |name: &str| -> Result<i64, EsiError> {
            match headers.get(name) {
                Some(v) => v
                    .to_str()?
                    .trim()
                    .parse::<i64>()
                    .map_err(|e| EsiError::HeaderParseError(name.into(), e)),
                None => Ok(0),
            }
        };
        let remaining = parse_i64(RATE_LIMIT_REMAINING_HEADER)?;
        let used = parse_i64(RATE_LIMIT_USED_HEADER)?;
        let (max_tokens, window_secs) = parse_rate_limit(&limit);
        let status = RateLimitStatus {
            group: group.clone(),
            limit,
            max_tokens,
            window_secs,
            remaining,
            used,
            updated_at_millis: current_time_millis()?,
        };
        debug!("Rate limit status: {status:?}");
        self.rate_limits.write().await.insert(group, status.clone());
        Ok(Some(status))
    }

    /// The key of the budget a request spends from, when throttling is on and the
    /// spec says which route group the operation belongs to.
    fn bucket_key(
        &self,
        method: &str,
        request_type: &RequestType,
        endpoint: &str,
    ) -> Option<String> {
        if self.rate_limit_policy == RateLimitPolicy::Off {
            return None;
        }
        let group = self.index.rate_limit_group(method, endpoint)?;
        let token = match request_type {
            RequestType::Authenticated => self.access_token.as_deref(),
            RequestType::Public => None,
        };
        Some(RateLimiter::key(group, token))
    }

    /// Reserve budget for a request, sleeping or failing as the policy says when
    /// it does not fit.
    async fn acquire_permit(&self, key: &str) -> EsiResult<Permit> {
        let started = current_time_millis()?;
        loop {
            let now = current_time_millis()?;
            let Acquire::Wait(wait_ms) = self.limiter.try_acquire(key, now) else {
                return Ok(Permit::new(Arc::clone(&self.limiter), key));
            };
            let allowed_ms = match self.rate_limit_policy {
                RateLimitPolicy::Wait { max_wait } => {
                    i64::try_from(max_wait.as_millis()).unwrap_or(i64::MAX)
                }
                RateLimitPolicy::Off | RateLimitPolicy::Fail => 0,
            };
            if (now - started).saturating_add(wait_ms) > allowed_ms {
                let group = RateLimiter::group_of(key).to_owned();
                warn!("Not sending a request: group {group} has no tokens for {wait_ms}ms");
                return Err(EsiError::RateLimited {
                    group: Some(group),
                    retry_after_secs: Some(u64::try_from((wait_ms + 999) / 1000).unwrap_or(0)),
                });
            }
            debug!("Waiting {wait_ms}ms for rate-limit tokens of {key}");
            tokio::time::sleep(std::time::Duration::from_millis(
                u64::try_from(wait_ms).unwrap_or(0),
            ))
            .await;
        }
    }

    /// Feed a response to the budget of its route group.
    fn record_budget(
        &self,
        key: &str,
        status: Option<&RateLimitStatus>,
        http_status: reqwest::StatusCode,
        headers: &HeaderMap,
    ) -> EsiResult<()> {
        let now = current_time_millis()?;
        if let Some(status) = status {
            let window_ms = status
                .window_secs
                .and_then(|secs| i64::try_from(secs).ok())
                .map_or(0, |secs| secs.saturating_mul(1000));
            self.limiter.record(
                key,
                status.remaining,
                status.used,
                window_ms,
                http_status.is_success(),
                now,
            );
        }
        if http_status.as_u16() == 429 {
            if let Some(secs) = Self::retry_after_secs(headers) {
                let secs = i64::try_from(secs).unwrap_or(0);
                self.limiter
                    .block_until(key, now.saturating_add(secs.saturating_mul(1000)));
            }
        }
        Ok(())
    }

    /// The seconds in a `Retry-After` header, if it has them.
    fn retry_after_secs(headers: &HeaderMap) -> Option<u64> {
        headers
            .get(header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.trim().parse().ok())
    }

    /// Latest rate-limit status ESI reported for a route group
    /// (the `X-Ratelimit-Group` header value, e.g. `market`).
    ///
    /// Returns `None` if no response from that group has been seen yet.
    /// Routes not yet moved to ESI's rate limiter do not send these
    /// headers; they are covered by [`Esi::is_error_limited`] instead.
    pub async fn rate_limit_status(&self, group: &str) -> Option<RateLimitStatus> {
        self.rate_limits.read().await.get(group).cloned()
    }

    /// Latest rate-limit status for every route group seen so far.
    pub async fn rate_limit_statuses(&self) -> HashMap<String, RateLimitStatus> {
        self.rate_limits.read().await.clone()
    }

    async fn process_error_limit_headers(&self, headers: &HeaderMap) -> Result<(), EsiError> {
        match (
            headers.get(ERROR_LIMIT_REMAIN_HEADER),
            headers.get(ERROR_LIMIT_RESET_HEADER),
        ) {
            (Some(remain_header), Some(reset_header)) => {
                let remaining_limit = remain_header
                    .to_str()?
                    .parse::<i32>()
                    .map_err(|e| EsiError::HeaderParseError(ERROR_LIMIT_REMAIN_HEADER.into(), e))?;
                let resets_in = reset_header
                    .to_str()?
                    .parse::<i64>()
                    .map_err(|e| EsiError::HeaderParseError(ERROR_LIMIT_RESET_HEADER.into(), e))?;

                let expires_at_millis = current_time_millis()? + resets_in * 1000;

                self.error_limit_state
                    .write()
                    .await
                    .replace(ErrorLimitState {
                        remaining_limit,
                        expires_at_millis,
                    });
                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn assert_not_error_limited(&self) -> Result<(), EsiError> {
        match self.is_error_limited().await? {
            Limited { for_millis } => Err(EsiError::ErrorLimited(for_millis)),
            NotLimited => Ok(()),
        }
    }

    /// Returns whether we have temporarily encountered the error limit due to too many failed responses.
    ///
    /// If this returns true, then this client will refuse to process further requests.
    pub async fn is_error_limited(&self) -> Result<ErrorLimitStatus, EsiError> {
        match &self.error_limit_state.read().await.as_ref() {
            None => Ok(NotLimited),
            Some(state) => {
                if state.remaining_limit > 0 {
                    return Ok(NotLimited);
                }
                let remaining_time = state.expires_at_millis - current_time_millis()?;
                if remaining_time < 0 {
                    return Ok(NotLimited);
                }
                Ok(Limited {
                    for_millis: remaining_time,
                })
            }
        }
    }

    /// Retrieve this struct's OpenAPI specification.
    ///
    /// Use in tandem with [EsiBuilder::spec].
    pub fn get_spec(&self) -> Option<&Spec> {
        self.spec.as_ref()
    }

    /// Call endpoints under the "Access List" group in ESI.
    pub fn group_access_list(&self) -> AccessListGroup<'_> {
        AccessListGroup { esi: self }
    }

    /// Call endpoints under the "Activities" group in ESI.
    pub fn group_activities(&self) -> ActivitiesGroup<'_> {
        ActivitiesGroup { esi: self }
    }

    /// Call endpoints under the "alliance" group in ESI.
    pub fn group_alliance(&self) -> AllianceGroup<'_> {
        AllianceGroup { esi: self }
    }

    /// Call endpoints under the "Assets" group in ESI.
    pub fn group_assets(&self) -> AssetsGroup<'_> {
        AssetsGroup { esi: self }
    }

    /// Call endpoints under the "Bookmarks" group in ESI.
    pub fn group_bookmarks(&self) -> BookmarksGroup<'_> {
        BookmarksGroup { esi: self }
    }

    /// Call endpoints under the "Calendar" group in ESI.
    pub fn group_calendar(&self) -> CalendarGroup<'_> {
        CalendarGroup { esi: self }
    }

    /// Call endpoints under the "Character" group in ESI.
    pub fn group_character(&self) -> CharacterGroup<'_> {
        CharacterGroup { esi: self }
    }

    /// Call endpoints under the "Clones" group in ESI.
    pub fn group_clones(&self) -> ClonesGroup<'_> {
        ClonesGroup { esi: self }
    }

    /// Call endpoints under the "Contacts" group in ESI.
    pub fn group_contacts(&self) -> ContactsGroup<'_> {
        ContactsGroup { esi: self }
    }

    /// Call endpoints under the "Contracts" group in ESI.
    pub fn group_contracts(&self) -> ContractsGroup<'_> {
        ContractsGroup { esi: self }
    }

    /// Call endpoints under the "Corporation" group in ESI.
    pub fn group_corporation(&self) -> CorporationGroup<'_> {
        CorporationGroup { esi: self }
    }

    /// Call endpoints under the "Corporation Projects" group in ESI.
    pub fn group_corporation_projects(&self) -> CorporationProjectsGroup<'_> {
        CorporationProjectsGroup { esi: self }
    }

    /// Call endpoints under the "Dogma" group in ESI.
    pub fn group_dogma(&self) -> DogmaGroup<'_> {
        DogmaGroup { esi: self }
    }

    /// Call endpoints under the "FactionWarfare" group in ESI.
    pub fn group_faction_warfare(&self) -> FactionWarfareGroup<'_> {
        FactionWarfareGroup { esi: self }
    }

    /// Call endpoints under the "Fittings" group in ESI.
    pub fn group_fittings(&self) -> FittingsGroup<'_> {
        FittingsGroup { esi: self }
    }

    /// Call endpoints under the "Fleets" group in ESI.
    pub fn group_fleets(&self) -> FleetsGroup<'_> {
        FleetsGroup { esi: self }
    }

    /// Call endpoints under the "Incursions" group in ESI.
    pub fn group_incursions(&self) -> IncursionsGroup<'_> {
        IncursionsGroup { esi: self }
    }

    /// Call endpoints under the "Industry" group in ESI.
    pub fn group_industry(&self) -> IndustryGroup<'_> {
        IndustryGroup { esi: self }
    }

    /// Call endpoints under the "Insurance" group in ESI.
    pub fn group_insurance(&self) -> InsuranceGroup<'_> {
        InsuranceGroup { esi: self }
    }

    /// Call endpoints under the "Killmails" group in ESI.
    pub fn group_killmails(&self) -> KillmailsGroup<'_> {
        KillmailsGroup { esi: self }
    }

    /// Call endpoints under the "Location" group in ESI.
    pub fn group_location(&self) -> LocationGroup<'_> {
        LocationGroup { esi: self }
    }

    /// Call endpoints under the "Loyalty" group in ESI.
    pub fn group_loyalty(&self) -> LoyaltyGroup<'_> {
        LoyaltyGroup { esi: self }
    }

    /// Call endpoints under the "Mail" group in ESI.
    pub fn group_mail(&self) -> MailGroup<'_> {
        MailGroup { esi: self }
    }

    /// Call endpoints under the "Market" group in ESI.
    pub fn group_market(&self) -> MarketGroup<'_> {
        MarketGroup { esi: self }
    }

    /// Call endpoints under the "Opportunities" group in ESI.
    pub fn group_opportunities(&self) -> OpportunitiesGroup<'_> {
        OpportunitiesGroup { esi: self }
    }

    /// Call endpoints under the "PlanetaryInteraction" group in ESI.
    pub fn group_planetary_interaction(&self) -> PlanetaryInteractionGroup<'_> {
        PlanetaryInteractionGroup { esi: self }
    }

    /// Call endpoints under the "Routes" group in ESI.
    pub fn group_routes(&self) -> RoutesGroup<'_> {
        RoutesGroup { esi: self }
    }

    /// Call endpoints under the "Cosmetics" group in ESI.
    pub fn group_cosmetics(&self) -> CosmeticsGroup<'_> {
        CosmeticsGroup { esi: self }
    }

    /// Call endpoints under the "Paragon Hub" group in ESI.
    pub fn group_paragon_hub(&self) -> ParagonHubGroup<'_> {
        ParagonHubGroup { esi: self }
    }

    /// Call endpoints under the "Freelance Jobs" group in ESI.
    pub fn group_freelance_jobs(&self) -> FreelanceJobsGroup<'_> {
        FreelanceJobsGroup { esi: self }
    }

    /// Call endpoints under the "Military Campaigns" group in ESI.
    pub fn group_military_campaigns(&self) -> MilitaryCampaignsGroup<'_> {
        MilitaryCampaignsGroup { esi: self }
    }

    /// Call endpoints under the "Meta" group in ESI.
    pub fn group_meta(&self) -> MetaGroup<'_> {
        MetaGroup { esi: self }
    }

    /// Call endpoints under the "Search" group in ESI.
    pub fn group_search(&self) -> SearchGroup<'_> {
        SearchGroup { esi: self }
    }

    /// Call endpoints under the "Skills" group in ESI.
    pub fn group_skills(&self) -> SkillsGroup<'_> {
        SkillsGroup { esi: self }
    }

    /// Call endpoints under the "Sovereignty" group in ESI.
    pub fn group_sovereignty(&self) -> SovereigntyGroup<'_> {
        SovereigntyGroup { esi: self }
    }

    /// Call endpoints under the "Structures" group in ESI.
    pub fn group_structures(&self) -> StructuresGroup<'_> {
        StructuresGroup { esi: self }
    }

    /// Call endpoints under the "Status" group in ESI.
    pub fn group_status(&self) -> StatusGroup<'_> {
        StatusGroup { esi: self }
    }

    /// Call endpoints under the "Universe" group in ESI.
    pub fn group_universe(&self) -> UniverseGroup<'_> {
        UniverseGroup { esi: self }
    }

    /// Call endpoints under the "UserInterface" group in ESI.
    pub fn group_user_interface(&self) -> UserInterfaceGroup<'_> {
        UserInterfaceGroup { esi: self }
    }

    /// Call endpoints under the "Wallet" group in ESI.
    pub fn group_wallet(&self) -> WalletGroup<'_> {
        WalletGroup { esi: self }
    }

    /// Call endpoints under the "Wars" group in ESI.
    pub fn group_wars(&self) -> WarsGroup<'_> {
        WarsGroup { esi: self }
    }
}

/// Get the current system timestamp since the epoch.
fn current_time_millis() -> Result<i64, EsiError> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .try_into()
        .expect("i64 overflow for time"))
}

#[cfg(test)]
mod tests {
    use super::{
        parse_rate_limit, AuthenticateResponse, Esi, ERROR_LIMIT_REMAIN_HEADER,
        ERROR_LIMIT_RESET_HEADER, RATE_LIMIT_GROUP_HEADER, RATE_LIMIT_LIMIT_HEADER,
        RATE_LIMIT_REMAINING_HEADER, RATE_LIMIT_USED_HEADER,
    };
    use crate::errors::EsiError;
    use crate::prelude::EsiBuilder;
    use crate::spec::Spec;

    #[test]
    fn test_spec_metadata_helpers() {
        let spec: Spec = serde_json::from_str(FIXTURE).unwrap();
        let esi = EsiBuilder::new()
            .user_agent("t")
            .spec(Some(spec))
            .build()
            .unwrap();
        let op = "GetCorporationsCorporationIdBlueprints";
        assert_eq!(esi.required_roles(op).unwrap(), ["Director"]);
        assert_eq!(
            esi.missing_scopes(op, "esi-assets.read_assets.v1").unwrap(),
            ["esi-corporations.read_blueprints.v1"]
        );
        assert!(esi
            .missing_scopes(op, "a esi-corporations.read_blueprints.v1")
            .unwrap()
            .is_empty());
        assert!(matches!(
            esi.required_scopes("Nope"),
            Err(EsiError::UnknownOperationID(_))
        ));
        assert!(esi.declared_rate_limits().unwrap().len() > 5);
    }

    #[test]
    fn test_pages_header_and_empty_body() {
        let mut headers = reqwest::header::HeaderMap::new();
        assert_eq!(Esi::pages_header(&headers), None);
        headers.insert("x-pages", "7".parse().unwrap());
        assert_eq!(Esi::pages_header(&headers), Some(7));
        let unit: () = Esi::parse_body("").unwrap();
        assert_eq!(unit, ());
        let numbers: Vec<i64> = Esi::parse_body("[1, 2]").unwrap();
        assert_eq!(numbers, [1, 2]);
    }
    use http::{HeaderMap, HeaderValue};
    use std::time::Duration;

    const FIXTURE: &str = include_str!("../resources/test/openapi.json");

    fn esi_with_fixture() -> Esi {
        let spec: Spec = serde_json::from_str(FIXTURE).unwrap();
        EsiBuilder::new()
            .user_agent("Client test, not meant to request")
            .spec(Some(spec))
            .build()
            .unwrap()
    }

    #[test]
    fn test_resolve_openapi_op_id() {
        let esi = esi_with_fixture();
        assert_eq!(
            esi.get_endpoint_for_op_id("GetMarketsRegionIdOrders")
                .unwrap(),
            "markets/{region_id}/orders"
        );
        assert_eq!(
            esi.get_endpoint_for_op_id("PostUniverseIds").unwrap(),
            "universe/ids"
        );
    }

    #[test]
    fn test_resolve_legacy_op_id() {
        let esi = esi_with_fixture();
        assert_eq!(
            esi.get_endpoint_for_op_id("get_markets_region_id_orders")
                .unwrap(),
            "markets/{region_id}/orders"
        );
        assert_eq!(
            esi.get_endpoint_for_op_id("get_characters_character_id")
                .unwrap(),
            "characters/{character_id}"
        );
    }

    #[test]
    fn test_all_legacy_ids_resolve() {
        let esi = esi_with_fixture();
        for (legacy, openapi) in crate::legacy::LEGACY_OP_IDS {
            esi.get_endpoint_for_op_id(openapi)
                .unwrap_or_else(|_| panic!("{openapi} (from {legacy}) missing from spec"));
        }
    }

    #[test]
    fn test_resolve_unknown_op_id() {
        let esi = esi_with_fixture();
        match esi.get_endpoint_for_op_id("GetNothingHere") {
            Err(EsiError::UnknownOperationID(id)) => assert_eq!(id, "GetNothingHere"),
            other => panic!("Unexpected result: {other:?}"),
        }
    }

    #[test]
    fn test_resolve_without_spec() {
        let esi = EsiBuilder::new().user_agent("test").build().unwrap();
        assert!(matches!(
            esi.get_endpoint_for_op_id("GetMarketsPrices"),
            Err(EsiError::EmptySpec)
        ));
    }

    #[test]
    fn test_parse_rate_limit() {
        assert_eq!(parse_rate_limit("150/15m"), (Some(150), Some(900)));
        assert_eq!(parse_rate_limit("20/1h"), (Some(20), Some(3600)));
        assert_eq!(parse_rate_limit("300/30s"), (Some(300), Some(30)));
        assert_eq!(parse_rate_limit("10/5x"), (Some(10), None));
        assert_eq!(parse_rate_limit("garbage"), (None, None));
    }

    #[tokio::test]
    async fn test_rate_limit_headers() {
        let esi = EsiBuilder::new().user_agent("test").build().unwrap();
        let mut headers = HeaderMap::new();
        headers.append(RATE_LIMIT_GROUP_HEADER, HeaderValue::from_static("market"));
        headers.append(RATE_LIMIT_LIMIT_HEADER, HeaderValue::from_static("150/15m"));
        headers.append(RATE_LIMIT_REMAINING_HEADER, HeaderValue::from_static("148"));
        headers.append(RATE_LIMIT_USED_HEADER, HeaderValue::from_static("2"));
        esi.process_response_headers(&headers)
            .await
            .expect("Should parse");
        let status = esi.rate_limit_status("market").await.expect("recorded");
        assert_eq!(status.limit, "150/15m");
        assert_eq!(status.max_tokens, Some(150));
        assert_eq!(status.window_secs, Some(900));
        assert_eq!(status.remaining, 148);
        assert_eq!(status.used, 2);
        assert!(esi.rate_limit_status("other").await.is_none());
        assert_eq!(esi.rate_limit_statuses().await.len(), 1);
    }

    #[tokio::test]
    async fn test_no_rate_limit_headers() {
        let esi = EsiBuilder::new().user_agent("test").build().unwrap();
        esi.process_response_headers(&HeaderMap::new())
            .await
            .expect("Should parse");
        assert!(esi.rate_limit_statuses().await.is_empty());
    }

    #[test]
    fn test_status_error_429() {
        let mut headers = HeaderMap::new();
        headers.append(RATE_LIMIT_GROUP_HEADER, HeaderValue::from_static("market"));
        headers.append(http::header::RETRY_AFTER, HeaderValue::from_static("12"));
        match Esi::status_error(429, &headers) {
            EsiError::RateLimited {
                group,
                retry_after_secs,
            } => {
                assert_eq!(group.as_deref(), Some("market"));
                assert_eq!(retry_after_secs, Some(12));
            }
            other => panic!("Unexpected error: {other}"),
        }
        assert!(matches!(
            Esi::status_error(404, &headers),
            EsiError::InvalidStatusCode(404)
        ));
    }

    #[test]
    fn test_authenticateresponse_deserialize() {
        let source = r#"{
            "access_token": "abc",
            "expires_in": 1000,
            "refresh_token": "def"
          }"#;
        let data: AuthenticateResponse = serde_json::from_str(source).unwrap();

        assert_eq!(data.access_token, "abc");
        assert_eq!(data.expires_in, 1000);
        assert_eq!(data.refresh_token, Some("def".to_owned()));
    }

    #[test]
    fn test_authenticateresponse_deserialize_no_refresh_token() {
        let source = r#"{
            "access_token": "abc",
            "expires_in": 1000,
            "refresh_token": null
          }"#;
        let data: AuthenticateResponse = serde_json::from_str(source).unwrap();

        assert_eq!(data.access_token, "abc");
        assert_eq!(data.expires_in, 1000);
        assert_eq!(data.refresh_token, None);
    }

    #[tokio::test]
    async fn test_error_limit_header_not_limited() {
        let esi = EsiBuilder::default()
            .user_agent("Client test, not meant to request")
            .build()
            .unwrap();
        let mut headers = HeaderMap::new();
        headers.append(ERROR_LIMIT_REMAIN_HEADER, HeaderValue::from_static("100"));
        headers.append(ERROR_LIMIT_RESET_HEADER, HeaderValue::from_static("5"));
        esi.process_error_limit_headers(&headers)
            .await
            .expect("Should parse");
        esi.assert_not_error_limited()
            .await
            .expect("Should not be error limited");
    }

    #[tokio::test]
    async fn test_error_limit_header_limited() {
        let esi = EsiBuilder::default()
            .user_agent("Client test, not meant to request")
            .build()
            .unwrap();
        let mut headers = HeaderMap::new();
        headers.append(ERROR_LIMIT_REMAIN_HEADER, HeaderValue::from_static("0"));
        headers.append(ERROR_LIMIT_RESET_HEADER, HeaderValue::from_static("2"));
        esi.process_error_limit_headers(&headers)
            .await
            .expect("Should parse");
        let err = esi
            .assert_not_error_limited()
            .await
            .expect_err("Should be limited");
        match err {
            EsiError::ErrorLimited(millis) => {
                assert!(millis <= 2000)
            }
            _ => panic!("Unexpected error: {}", err),
        }
    }

    #[tokio::test]
    #[ignore] // This is a bit slow
    async fn test_error_limit_expired_limit() {
        let esi = EsiBuilder::default()
            .user_agent("Client test, not meant to request")
            .build()
            .unwrap();
        let mut headers = HeaderMap::new();
        headers.append(ERROR_LIMIT_REMAIN_HEADER, HeaderValue::from_static("0"));
        headers.append(ERROR_LIMIT_RESET_HEADER, HeaderValue::from_static("2"));
        esi.process_error_limit_headers(&headers)
            .await
            .expect("Should parse");
        println!("Waiting 2 seconds ..");
        tokio::time::sleep(Duration::from_millis(2050)).await;
        esi.assert_not_error_limited()
            .await
            .expect("Should not be error limited");
    }
}
