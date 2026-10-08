use crate::prelude::*;

/// Endpoints for Wars
pub struct WarsGroup<'a> {
    pub(crate) esi: &'a Esi,
}

/// One side of a war.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct WarParty {
    pub alliance_id: Option<i64>,
    pub corporation_id: Option<i64>,
    pub isk_destroyed: f64,
    pub ships_killed: i64,
}

/// An ally of one side of a war.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct WarAlly {
    pub alliance_id: Option<i64>,
    pub corporation_id: Option<i64>,
}

/// Information about a war.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct War {
    pub aggressor: WarParty,
    pub allies: Option<Vec<WarAlly>>,
    pub declared: String,
    pub defender: WarParty,
    pub finished: Option<String>,
    pub id: i64,
    pub mutual: bool,
    pub open_for_allies: bool,
    pub retracted: Option<String>,
    pub started: Option<String>,
}

/// A killmail reference belonging to a war.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct WarKillmail {
    pub killmail_hash: String,
    pub killmail_id: i64,
}

impl WarsGroup<'_> {
    api_get!(
        /// List war IDs, newest first. Pass `max_war_id` to start the list below that ID.
        list,
        "GetWars",
        RequestType::Public,
        Vec<i64>,
        ;
        Optional(max_war_id: i64) => "max_war_id"
    );

    api_get!(
        /// Get information about a war.
        get_war,
        "GetWarsWarId",
        RequestType::Public,
        War,
        (war_id: i64) => "{war_id}"
    );

    api_get!(
        /// List the killmails of a war, newest first.
        get_war_killmails,
        "GetWarsWarIdKillmails",
        RequestType::Public,
        Vec<WarKillmail>,
        (war_id: i64) => "{war_id}";
        Optional(page: i32) => "page"
    );
}

#[cfg(test)]
mod tests {
    use super::War;

    #[test]
    fn test_parse_war_with_optional_fields_missing() {
        let json = r#"{
            "id": 1,
            "declared": "2026-01-01T00:00:00Z",
            "mutual": false,
            "open_for_allies": true,
            "aggressor": {"corporation_id": 7, "isk_destroyed": 1.5, "ships_killed": 2},
            "defender": {"alliance_id": 9, "isk_destroyed": 0.0, "ships_killed": 0}
        }"#;
        let war: War = serde_json::from_str(json).unwrap();
        assert_eq!(war.aggressor.corporation_id, Some(7));
        assert!(war.finished.is_none());
        assert!(war.allies.is_none());
    }
}
