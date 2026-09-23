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
}

impl Spec {
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

#[cfg(test)]
mod tests {
    use super::Spec;

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
