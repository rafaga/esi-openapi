//! Struct types for the ESI OpenAPI specification data.
//!
//! Only the parts of the specification needed to resolve an
//! `operationId` to a URL path are modeled; every other key in
//! the document is ignored during deserialization.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ESI OpenAPI spec type.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Spec {
    /// Map of URL path (e.g. `/markets/{region_id}/orders`) to its path item.
    pub paths: HashMap<String, SpecPathItem>,
    /// Reusable parts of the spec; only the security schemes are read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub components: Option<SpecComponents>,
}

/// The `components` section of the spec.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct SpecComponents {
    /// Security schemes by name, such as `OAuth2`.
    #[serde(
        rename = "securitySchemes",
        default,
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub security_schemes: HashMap<String, SpecSecurityScheme>,
}

/// A security scheme; only the OAuth2 authorization code flow is read.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct SpecSecurityScheme {
    /// OAuth2 flows of the scheme.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flows: Option<SpecOAuthFlows>,
}

/// The OAuth2 flows of a security scheme.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct SpecOAuthFlows {
    /// The authorization code flow.
    #[serde(
        rename = "authorizationCode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub authorization_code: Option<SpecOAuthFlow>,
}

/// One OAuth2 flow.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct SpecOAuthFlow {
    /// Scope name to description.
    #[serde(default)]
    pub scopes: HashMap<String, String>,
}

/// An OpenAPI path item: the operations available on a single URL path.
///
/// Path-level keys other than the HTTP methods (such as `parameters`
/// or `summary`) are ignored.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct SpecPathItem {
    /// `GET` operation, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub get: Option<SpecPathMethod>,
    /// `POST` operation, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post: Option<SpecPathMethod>,
    /// `PUT` operation, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub put: Option<SpecPathMethod>,
    /// `DELETE` operation, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delete: Option<SpecPathMethod>,
    /// `PATCH` operation, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patch: Option<SpecPathMethod>,
}

impl SpecPathItem {
    /// Iterate over the operations defined on this path.
    pub fn methods(&self) -> impl Iterator<Item = &SpecPathMethod> {
        [&self.get, &self.post, &self.put, &self.delete, &self.patch]
            .into_iter()
            .flatten()
    }
}

/// A single OpenAPI operation.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct SpecPathMethod {
    /// The operation ID to use this endpoint, e.g. `GetMarketsRegionIdOrders`.
    #[serde(
        rename = "operationId",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub operation_id: Option<String>,
    /// How long, in seconds, a client may reuse the response (`x-client-cache-ttl`).
    #[serde(
        rename = "x-client-cache-ttl",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub client_cache_ttl: Option<i64>,
    /// Security requirements: each entry maps a scheme (`OAuth2`) to the scopes it needs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub security: Vec<HashMap<String, Vec<String>>>,
    /// Rate limit group and budget of the operation (`x-rate-limit`).
    #[serde(
        rename = "x-rate-limit",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub rate_limit: Option<SpecRateLimit>,
    /// Corporation roles of which the character needs at least one (`x-required-roles`).
    #[serde(
        rename = "x-required-roles",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub required_roles: Vec<String>,
    /// Pagination style of the operation (`x-pagination`), such as `cursor`.
    #[serde(
        rename = "x-pagination",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub pagination: Option<String>,
    /// Seconds a deleted resource keeps answering as gone (`x-tombstone-ttl`).
    #[serde(
        rename = "x-tombstone-ttl",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub tombstone_ttl: Option<i64>,
}

/// The rate limit an operation declares with `x-rate-limit`.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct SpecRateLimit {
    /// Route group the operation spends tokens from.
    pub group: String,
    /// Tokens available per window.
    #[serde(rename = "max-tokens")]
    pub max_tokens: i64,
    /// Window length as ESI writes it, such as `15m`.
    #[serde(rename = "window-size")]
    pub window_size: String,
}

impl SpecPathMethod {
    /// The OAuth2 scopes the operation needs, without duplicates.
    pub fn scopes(&self) -> Vec<String> {
        let mut scopes: Vec<String> = Vec::new();
        for requirement in &self.security {
            for scope in requirement.values().flatten() {
                if !scopes.contains(scope) {
                    scopes.push(scope.clone());
                }
            }
        }
        scopes
    }
}

impl Spec {
    /// Every OAuth2 scope the spec defines, sorted.
    pub fn oauth_scopes(&self) -> Vec<String> {
        let mut scopes: Vec<String> = self
            .components
            .iter()
            .flat_map(|c| c.security_schemes.values())
            .filter_map(|scheme| scheme.flows.as_ref()?.authorization_code.as_ref())
            .flat_map(|flow| flow.scopes.keys().cloned())
            .collect();
        scopes.sort();
        scopes.dedup();
        scopes
    }

    /// The scopes needed to call every operation in `op_ids`, sorted and without
    /// duplicates, as a space-separated list ready for `EsiBuilder::scope`.
    pub fn scope_string_for(&self, op_ids: &[&str]) -> String {
        let mut scopes: Vec<String> = op_ids
            .iter()
            .flat_map(|id| self.required_scopes(id))
            .collect();
        scopes.sort();
        scopes.dedup();
        scopes.join(" ")
    }

    /// Find an operation by its `operationId`.
    pub fn operation(&self, op_id: &str) -> Option<&SpecPathMethod> {
        self.paths
            .values()
            .flat_map(SpecPathItem::methods)
            .find(|m| m.operation_id.as_deref() == Some(op_id))
    }

    /// The scopes an operation needs (empty for public operations).
    pub fn required_scopes(&self, op_id: &str) -> Vec<String> {
        self.operation(op_id)
            .map(SpecPathMethod::scopes)
            .unwrap_or_default()
    }

    /// The rate limit every operation declares, keyed by route group.
    pub fn rate_limit_groups(&self) -> HashMap<String, SpecRateLimit> {
        self.paths
            .values()
            .flat_map(SpecPathItem::methods)
            .filter_map(|m| m.rate_limit.clone())
            .map(|limit| (limit.group.clone(), limit))
            .collect()
    }

    /// The `x-client-cache-ttl` of the `GET` operation whose path template matches
    /// `path` (with or without the leading slash, e.g. `characters/95465499`).
    pub fn client_cache_ttl(&self, path: &str) -> Option<i64> {
        self.get_operation_for_path(path)?.client_cache_ttl
    }

    /// The `x-tombstone-ttl` of the `GET` operation whose path template matches
    /// `path`: how long ESI keeps answering for a deleted resource.
    pub fn tombstone_ttl(&self, path: &str) -> Option<i64> {
        self.get_operation_for_path(path)?.tombstone_ttl
    }

    /// The `GET` operation whose path template matches a concrete path such as
    /// `characters/95465499`. When several templates match, the one with the
    /// most literal segments wins (`characters/affiliation` over
    /// `characters/{character_id}`).
    pub fn get_operation_for_path(&self, path: &str) -> Option<&SpecPathMethod> {
        let wanted: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        self.paths
            .iter()
            .filter_map(|(template, item)| {
                let parts: Vec<&str> = template.trim_start_matches('/').split('/').collect();
                let is_param = |t: &&str| t.starts_with('{') && t.ends_with('}');
                let matches = parts.len() == wanted.len()
                    && parts
                        .iter()
                        .zip(&wanted)
                        .all(|(t, w)| is_param(t) || t == w);
                let operation = item.get.as_ref()?;
                matches.then(|| (parts.iter().filter(|t| !is_param(t)).count(), operation))
            })
            .max_by_key(|(literals, _)| *literals)
            .map(|(_, operation)| operation)
    }

    /// Build a map of `operationId` to URL path, with the path's leading
    /// slash removed so it can be appended to the base API URL.
    pub fn operation_index(&self) -> HashMap<String, String> {
        let mut index = HashMap::new();
        for (path, item) in &self.paths {
            let path = path.strip_prefix('/').unwrap_or(path);
            for op_id in item.methods().filter_map(|m| m.operation_id.as_ref()) {
                index.insert(op_id.clone(), path.to_owned());
            }
        }
        index
    }
}

/// A path template split into segments, with the metadata looked up per request.
#[derive(Debug, Clone, PartialEq, Eq)]
struct PathTemplate {
    /// One entry per segment: the literal text, or `None` for a `{parameter}`.
    segments: Vec<Option<String>>,
    /// How many segments are literal; more literals win when several templates match.
    literals: usize,
    client_cache_ttl: Option<i64>,
    tombstone_ttl: Option<i64>,
    rate_limit_group: Option<String>,
}

impl PathTemplate {
    fn matches(&self, wanted: &[&str]) -> bool {
        self.segments
            .iter()
            .zip(wanted)
            .all(|(segment, w)| segment.as_deref().is_none_or(|literal| literal == *w))
    }
}

/// Lookup tables built once from a [`Spec`], so that resolving an operation or
/// matching a concrete path does not scan every path in the spec on each call.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct SpecIndex {
    /// `operationId` -> URL path without the leading slash.
    paths: HashMap<String, String>,
    /// `operationId` -> (path template, position among that path's methods).
    locations: HashMap<String, (String, usize)>,
    /// Templates by HTTP method and segment count, the most literal first.
    templates: HashMap<&'static str, HashMap<usize, Vec<PathTemplate>>>,
}

impl SpecIndex {
    /// The HTTP methods a path item can define, as written in requests.
    const VERBS: [&'static str; 5] = ["GET", "POST", "PUT", "DELETE", "PATCH"];

    /// Index every operation and every path template of `spec`.
    pub(crate) fn new(spec: &Spec) -> Self {
        let mut index = SpecIndex::default();
        for (template, item) in &spec.paths {
            let path = template.strip_prefix('/').unwrap_or(template);
            for (position, method) in item.methods().enumerate() {
                if let Some(op_id) = &method.operation_id {
                    index.paths.insert(op_id.clone(), path.to_owned());
                    index
                        .locations
                        .insert(op_id.clone(), (template.clone(), position));
                }
            }
            let segments: Vec<Option<String>> = template
                .trim_start_matches('/')
                .split('/')
                .map(|part| {
                    (!(part.starts_with('{') && part.ends_with('}'))).then(|| part.to_owned())
                })
                .collect();
            let literals = segments.iter().flatten().count();
            let methods = [&item.get, &item.post, &item.put, &item.delete, &item.patch];
            for (verb, method) in Self::VERBS.into_iter().zip(methods) {
                let Some(method) = method else { continue };
                index
                    .templates
                    .entry(verb)
                    .or_default()
                    .entry(segments.len())
                    .or_default()
                    .push(PathTemplate {
                        segments: segments.clone(),
                        literals,
                        client_cache_ttl: method.client_cache_ttl,
                        tombstone_ttl: method.tombstone_ttl,
                        rate_limit_group: method.rate_limit.as_ref().map(|r| r.group.clone()),
                    });
            }
        }
        for by_length in index.templates.values_mut() {
            for templates in by_length.values_mut() {
                templates.sort_by(|a, b| {
                    b.literals
                        .cmp(&a.literals)
                        .then_with(|| a.segments.cmp(&b.segments))
                });
            }
        }
        index
    }

    /// The URL path (without the leading slash) of an operation.
    pub(crate) fn path(&self, op_id: &str) -> Option<&str> {
        self.paths.get(op_id).map(String::as_str)
    }

    /// The operation's metadata, read from the `spec` this index was built from.
    pub(crate) fn operation<'a>(&self, spec: &'a Spec, op_id: &str) -> Option<&'a SpecPathMethod> {
        let (template, position) = self.locations.get(op_id)?;
        spec.paths.get(template)?.methods().nth(*position)
    }

    /// The template of the given HTTP method that best matches a concrete path.
    fn template_for(&self, method: &str, path: &str) -> Option<&PathTemplate> {
        let verb = Self::VERBS
            .into_iter()
            .find(|verb| verb.eq_ignore_ascii_case(method))?;
        let wanted: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        self.templates
            .get(verb)?
            .get(&wanted.len())?
            .iter()
            .find(|template| template.matches(&wanted))
    }

    /// Same as [`Spec::client_cache_ttl`], without scanning the spec.
    pub(crate) fn client_cache_ttl(&self, path: &str) -> Option<i64> {
        self.template_for("GET", path)?.client_cache_ttl
    }

    /// Same as [`Spec::tombstone_ttl`], without scanning the spec.
    pub(crate) fn tombstone_ttl(&self, path: &str) -> Option<i64> {
        self.template_for("GET", path)?.tombstone_ttl
    }

    /// The rate-limit group the operation with this HTTP method and path spends from.
    pub(crate) fn rate_limit_group(&self, method: &str, path: &str) -> Option<&str> {
        self.template_for(method, path)?.rate_limit_group.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::{Spec, SpecIndex};

    const FIXTURE: &str = include_str!("../resources/test/openapi.json");

    #[test]
    fn test_parse_openapi_fixture() {
        let spec: Spec = serde_json::from_str(FIXTURE).expect("fixture should parse");
        let index = spec.operation_index();
        assert!(index.len() > 200, "only {} operations", index.len());
        assert_eq!(
            index.get("GetMarketsRegionIdOrders").map(String::as_str),
            Some("markets/{region_id}/orders")
        );
        assert_eq!(
            index.get("GetCharactersDetail").map(String::as_str),
            Some("characters/{character_id}")
        );
    }

    #[test]
    fn test_client_cache_ttl_matches_templates() {
        let spec: Spec = serde_json::from_str(FIXTURE).unwrap();
        assert_eq!(spec.client_cache_ttl("/alliances"), Some(3600));
        assert!(spec.client_cache_ttl("characters/95465499").is_some());
        assert_eq!(spec.client_cache_ttl("no/such/path"), None);
    }

    #[test]
    fn test_operation_metadata() {
        let spec: Spec = serde_json::from_str(FIXTURE).unwrap();
        let op = spec
            .operation("GetCorporationsCorporationIdBlueprints")
            .unwrap();
        assert_eq!(op.scopes(), ["esi-corporations.read_blueprints.v1"]);
        assert_eq!(op.required_roles, ["Director"]);
        let limit = op.rate_limit.as_ref().unwrap();
        assert_eq!(limit.group, "corp-industry");
        assert_eq!(limit.max_tokens, 600);
        assert_eq!(limit.window_size, "15m");
        assert!(spec.required_scopes("GetStatus").is_empty());
        assert_eq!(
            spec.operation("GetFreelanceJobsListing")
                .unwrap()
                .pagination
                .as_deref(),
            Some("cursor")
        );
        assert_eq!(
            spec.operation("GetParagonHubSkinr").unwrap().tombstone_ttl,
            Some(604_800)
        );
        assert!(spec.rate_limit_groups().contains_key("char-wallet"));
    }

    #[test]
    fn test_oauth_scopes_and_scope_string() {
        let spec: Spec = serde_json::from_str(FIXTURE).unwrap();
        let scopes = spec.oauth_scopes();
        assert!(scopes.len() > 30, "only {} scopes", scopes.len());
        assert!(scopes.contains(&"esi-wallet.read_character_wallet.v1".to_owned()));
        assert_eq!(
            spec.scope_string_for(&[
                "GetCharactersCharacterIdWallet",
                "GetCharactersCharacterIdWalletJournal",
                "GetStatus"
            ]),
            "esi-wallet.read_character_wallet.v1"
        );
    }

    #[test]
    fn test_tombstone_ttl_and_path_matching() {
        let spec: Spec = serde_json::from_str(FIXTURE).unwrap();
        assert_eq!(spec.tombstone_ttl("paragon-hub/skinr"), Some(604_800));
        assert_eq!(
            spec.tombstone_ttl("paragon-hub/skinr/alliances/99000001"),
            Some(604_800)
        );
        assert_eq!(spec.tombstone_ttl("characters/95465499"), None);
        assert_eq!(
            spec.get_operation_for_path("characters/95465499")
                .and_then(|m| m.operation_id.as_deref()),
            Some("GetCharactersDetail")
        );
    }

    #[test]
    fn test_index_agrees_with_spec() {
        let spec: Spec = serde_json::from_str(FIXTURE).unwrap();
        let index = SpecIndex::new(&spec);
        for (op_id, path) in spec.operation_index() {
            assert_eq!(index.path(&op_id), Some(path.as_str()));
            assert_eq!(index.operation(&spec, &op_id), spec.operation(&op_id));
        }
        assert_eq!(index.path("NoSuchOperation"), None);
        assert!(index.operation(&spec, "NoSuchOperation").is_none());
    }

    #[test]
    fn test_index_path_matching_agrees_with_spec() {
        let spec: Spec = serde_json::from_str(FIXTURE).unwrap();
        let index = SpecIndex::new(&spec);
        for path in [
            "/alliances",
            "alliances/99000001",
            "characters/95465499",
            "characters/affiliation",
            "paragon-hub/skinr",
            "paragon-hub/skinr/alliances/99000001",
            "markets/10000002/orders",
            "no/such/path",
            "",
        ] {
            assert_eq!(
                index.client_cache_ttl(path),
                spec.client_cache_ttl(path),
                "{path}"
            );
            assert_eq!(
                index.tombstone_ttl(path),
                spec.tombstone_ttl(path),
                "{path}"
            );
        }
        assert_eq!(index.tombstone_ttl("paragon-hub/skinr"), Some(604_800));
    }

    #[test]
    fn test_index_rate_limit_group_by_method() {
        let spec: Spec = serde_json::from_str(FIXTURE).unwrap();
        let index = SpecIndex::new(&spec);
        let group_of = |op_id: &str| {
            spec.operation(op_id)
                .and_then(|m| m.rate_limit.as_ref())
                .map(|r| r.group.clone())
        };
        let post = group_of("PostCharactersCharacterIdAssetsNames").expect("declares a group");
        assert_eq!(
            index.rate_limit_group("post", "/characters/95465499/assets/names"),
            Some(post.as_str())
        );
        // The same path template has other methods; each keeps its own metadata.
        let get = group_of("GetCharactersCharacterIdAssets").expect("declares a group");
        assert_eq!(
            index.rate_limit_group("GET", "characters/95465499/assets"),
            Some(get.as_str())
        );
        // An operation that declares no group, and an unknown method or path.
        assert_eq!(
            index.rate_limit_group("POST", "characters/affiliation"),
            None
        );
        assert_eq!(index.rate_limit_group("TRACE", "alliances"), None);
        assert_eq!(index.rate_limit_group("GET", "no/such/path"), None);
    }

    #[test]
    fn test_parse_ignores_unknown_keys() {
        let source = r#"{
            "openapi": "3.1.0",
            "paths": {
                "/status": {
                    "parameters": [{"name": "x", "in": "header"}],
                    "summary": "Server status",
                    "get": {"operationId": "GetStatus", "tags": ["Status"]}
                },
                "/no-op-id": {"get": {"summary": "missing id"}}
            }
        }"#;
        let spec: Spec = serde_json::from_str(source).unwrap();
        let index = spec.operation_index();
        assert_eq!(index.len(), 1);
        assert_eq!(index["GetStatus"], "status");
    }
}
