use crate::prelude::*;
use std::collections::HashMap;

/// Endpoints for Meta
pub struct MetaGroup<'a> {
    pub(crate) esi: &'a Esi,
}

/// HTTP method of a route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum HttpMethod {
    /// `GET`
    #[serde(rename = "GET")]
    Get,
    /// `POST`
    #[serde(rename = "POST")]
    Post,
    /// `PUT`
    #[serde(rename = "PUT")]
    Put,
    /// `DELETE`
    #[serde(rename = "DELETE")]
    Delete,
    /// A method this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// The kind of change recorded in the changelog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeType {
    /// A breaking change.
    Breaking,
    /// A non-breaking change.
    Changed,
    /// A new route.
    New,
    /// A removed route.
    Removed,
    /// A kind of change this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// Health of a route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum RouteHealth {
    /// The status is not known.
    Unknown,
    /// The route is working as expected.
    #[serde(rename = "OK")]
    Ok,
    /// The route is working, but with degraded performance.
    Degraded,
    /// The route is not working.
    Down,
    /// The route is recovering.
    Recovering,
    /// A status this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// The list of compatibility dates supported by ESI.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct CompatibilityDates {
    pub compatibility_dates: Vec<String>,
}

/// One change to a route, as listed in the ESI changelog.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct ChangelogEntry {
    pub compatibility_date: String,
    pub description: String,
    pub method: HttpMethod,
    pub path: String,
    #[serde(rename = "type")]
    pub change_type: ChangeType,
}

/// The ESI changelog, keyed by compatibility date.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct Changelog {
    pub changelog: HashMap<String, Vec<ChangelogEntry>>,
}

/// A previous name of the API.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct ApiNameEntry {
    pub date: String,
    pub name: String,
}

/// Current and past names of the API.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct ApiName {
    pub current: String,
    pub history: Vec<ApiNameEntry>,
}

/// Health of a single route.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct RouteStatus {
    pub method: HttpMethod,
    pub path: String,
    pub status: RouteHealth,
}

/// Health of every ESI route.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct ApiStatus {
    pub routes: Vec<RouteStatus>,
}

impl MetaGroup<'_> {
    api_get!(
        /// Get the changelog of ESI routes.
        get_changelog,
        "GetMetaChangelog",
        RequestType::Public,
        Changelog,
    );

    api_get!(
        /// Get the compatibility dates supported by ESI.
        get_compatibility_dates,
        "GetMetaCompatibilityDates",
        RequestType::Public,
        CompatibilityDates,
    );

    api_get!(
        /// Get the current and past names of the API.
        get_name,
        "GetMetaName",
        RequestType::Public,
        ApiName,
    );

    api_get!(
        /// Get the health of every ESI route.
        get_status,
        "GetMetaStatus",
        RequestType::Public,
        ApiStatus,
    );
}

#[cfg(test)]
mod tests {
    use super::{
        ApiName, ApiStatus, ChangeType, Changelog, CompatibilityDates, HttpMethod, RouteHealth,
    };

    #[test]
    fn test_parse_compatibility_dates() {
        let json = r#"{"compatibility_dates":["2026-08-18","2020-01-01"]}"#;
        let dates: CompatibilityDates = serde_json::from_str(json).unwrap();
        assert_eq!(dates.compatibility_dates.len(), 2);
    }

    #[test]
    fn test_parse_changelog() {
        let json = r#"{"changelog":{"2026-08-18":[{"method":"GET","path":"/status","compatibility_date":"2026-08-18","type":"changed","description":"Updated."}]}}"#;
        let log: Changelog = serde_json::from_str(json).unwrap();
        assert_eq!(
            log.changelog["2026-08-18"][0].change_type,
            ChangeType::Changed
        );
        assert_eq!(log.changelog["2026-08-18"][0].method, HttpMethod::Get);
    }

    #[test]
    fn test_parse_name_and_status() {
        let name: ApiName = serde_json::from_str(
            r#"{"current":"ESI","history":[{"date":"2026-01-01","name":"Old"}]}"#,
        )
        .unwrap();
        assert_eq!(name.history[0].name, "Old");
        let status: ApiStatus =
            serde_json::from_str(r#"{"routes":[{"method":"GET","path":"/status","status":"OK"}]}"#)
                .unwrap();
        assert_eq!(status.routes[0].status, RouteHealth::Ok);
    }
}
