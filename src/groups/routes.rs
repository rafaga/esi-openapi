use crate::prelude::*;

/// Endpoints for Routes
pub struct RoutesGroup<'a> {
    pub(crate) esi: &'a Esi,
}

/// Which route to prefer when several are possible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RoutePreference {
    /// The shortest route.
    Shorter,
    /// The route that avoids low and null security systems.
    Safer,
    /// The route that prefers low and null security systems.
    LessSecure,
}

/// An extra connection between two systems (such as a wormhole or a jump bridge).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RouteConnection {
    /// Solar system ID the connection starts at.
    pub from: i64,
    /// Solar system ID the connection ends at.
    pub to: i64,
}

/// Options for a route calculation. All fields are optional.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct RouteRequest {
    /// Solar system IDs to avoid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avoid_systems: Option<Vec<i64>>,
    /// Extra connections to consider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connections: Option<Vec<RouteConnection>>,
    /// Route preference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preference: Option<RoutePreference>,
    /// Extra penalty applied to low and null security systems.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_penalty: Option<i64>,
}

/// A calculated route.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Route {
    /// Solar system IDs along the route, origin first.
    pub route: Vec<i64>,
}

impl RoutesGroup<'_> {
    api_post!(
        /// Calculate the route between two solar systems.
        calculate_route,
        "PostRoute",
        RequestType::Public,
        Route,
        (origin_system_id: i64) => "{origin_system_id}",
        (destination_system_id: i64) => "{destination_system_id}",
        options: &RouteRequest,
    );
}

#[cfg(test)]
mod tests {
    use super::{RouteConnection, RoutePreference, RouteRequest};

    #[test]
    fn test_default_request_serializes_to_empty_object() {
        let json = serde_json::to_string(&RouteRequest::default()).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_request_serialization() {
        let request = RouteRequest {
            avoid_systems: Some(vec![30000142]),
            connections: Some(vec![RouteConnection { from: 1, to: 2 }]),
            preference: Some(RoutePreference::LessSecure),
            security_penalty: Some(50),
        };
        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(value["preference"], "LessSecure");
        assert_eq!(value["connections"][0]["from"], 1);
        assert_eq!(value["security_penalty"], 50);
    }
}
