use crate::prelude::*;

/// Endpoints for Status
pub struct StatusGroup<'a> {
    pub(crate) esi: &'a Esi,
}

/// Current state of the game server.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct ServerStatus {
    pub players: i64,
    pub server_version: String,
    pub start_time: String,
    pub vip: bool,
}

impl StatusGroup<'_> {
    api_get!(
        /// Retrieve the uptime and player counts of the game server.
        get_status,
        "GetStatus",
        RequestType::Public,
        ServerStatus,
    );
}

#[cfg(test)]
mod tests {
    use super::ServerStatus;

    #[test]
    fn test_parse_server_status() {
        let json = r#"{"players":12345,"server_version":"2951234","start_time":"2026-10-07T11:00:00Z","vip":false}"#;
        let status: ServerStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status.players, 12345);
        assert!(!status.vip);
    }
}
