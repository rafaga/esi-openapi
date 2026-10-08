use crate::prelude::*;

/// Endpoints for FactionWarfare
pub struct FactionWarfareGroup<'a> {
    pub(crate) esi: &'a Esi,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct FactionLeaderboardItem {
    pub amount: Option<i64>,
    pub faction_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct FactionLeaderboardListing {
    pub active_total: Vec<FactionLeaderboardItem>,
    pub last_week: Vec<FactionLeaderboardItem>,
    pub yesterday: Vec<FactionLeaderboardItem>,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct FWLeaderboards {
    pub kills: FactionLeaderboardListing,
    pub victory_points: FactionLeaderboardListing,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct CharacterLeaderboardItem {
    pub amount: i64,
    pub character_id: i64,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct CharacterLeaderboardListing {
    pub active_total: Vec<CharacterLeaderboardItem>,
    pub last_week: Vec<CharacterLeaderboardItem>,
    pub yesterday: Vec<CharacterLeaderboardItem>,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct FWCharacterLeaderboards {
    pub kills: CharacterLeaderboardListing,
    pub victory_points: CharacterLeaderboardListing,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct CorporationLeaderboardItem {
    pub amount: i64,
    pub corporation_id: i64,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct CorporationLeaderboardListing {
    pub active_total: Vec<CorporationLeaderboardItem>,
    pub last_week: Vec<CorporationLeaderboardItem>,
    pub yesterday: Vec<CorporationLeaderboardItem>,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct FWCorporationLeaderboards {
    pub kills: CorporationLeaderboardListing,
    pub victory_points: CorporationLeaderboardListing,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct FWStatsItemRange {
    pub total: i64,
    pub last_week: i64,
    pub yesterday: i64,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct FWStatsItem {
    pub faction_id: i64,
    pub kills: FWStatsItemRange,
    pub pilots: i64,
    pub systems_controlled: i64,
    pub victory_points: FWStatsItemRange,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct FWSystem {
    pub contested: String,
    pub occupier_faction_id: i64,
    pub owner_faction_id: i64,
    pub solar_system_id: i64,
    pub victory_points: i64,
    pub victory_points_threshold: i64,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct FWWar {
    pub faction_id: i64,
    pub against_id: i64,
}

impl FactionWarfareGroup<'_> {
    api_get!(
        /// Get the top 4 leaderboards of factions for total, last week, and yesterday.
        leaderboards,
        "GetFwLeaderboards",
        RequestType::Public,
        FWLeaderboards,
    );

    api_get!(
        /// Get top 100 characters for total, last week, and yesterday.
        leaderboard_characters,
        "GetFwLeaderboardsCharacters",
        RequestType::Public,
        FWCharacterLeaderboards,
    );

    api_get!(
        /// Get top 10 corporations for total, last week, and yesterday.
        leaderboard_corporations,
        "GetFwLeaderboardsCorporations",
        RequestType::Public,
        FWCorporationLeaderboards,
    );

    api_get!(
        /// Get FW overview stats.
        stats,
        "GetFwStats",
        RequestType::Public,
        Vec<FWStatsItem>,
    );

    api_get!(
        /// Get FW system ownership.
        systems,
        "GetFwSystems",
        RequestType::Public,
        Vec<FWSystem>,
    );

    api_get!(
        /// Get FW faction information.
        wars,
        "GetFwWars",
        RequestType::Public,
        Vec<FWWar>,
    );

    // more endpoints ...
}

/// Faction warfare statistics of a character.
#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct CharacterFwStats {
    pub current_rank: Option<i64>,
    pub enlisted_on: Option<String>,
    pub faction_id: Option<i64>,
    pub highest_rank: Option<i64>,
    pub kills: FWStatsItemRange,
    pub victory_points: FWStatsItemRange,
}

impl FactionWarfareGroup<'_> {
    api_get!(
        /// Get the faction warfare statistics of a character.
        get_character_stats,
        "GetCharactersCharacterIdFwStats",
        RequestType::Authenticated,
        CharacterFwStats,
        (character_id: i64) => "{character_id}"
    );
}

#[cfg(test)]
mod character_basic_tests {
    use super::CharacterFwStats;

    #[test]
    fn test_parse_stats_of_unenlisted_character() {
        let json = r#"{"kills": {"last_week": 0, "total": 3, "yesterday": 0},
            "victory_points": {"last_week": 0, "total": 10, "yesterday": 0}}"#;
        let stats: CharacterFwStats = serde_json::from_str(json).unwrap();
        assert!(stats.faction_id.is_none());
        assert_eq!(stats.kills.total, 3);
    }
}

/// Faction warfare statistics of a corporation.
#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct CorporationFwStats {
    pub enlisted_on: Option<String>,
    pub faction_id: Option<i64>,
    pub kills: FWStatsItemRange,
    pub pilots: Option<i64>,
    pub victory_points: FWStatsItemRange,
}

impl FactionWarfareGroup<'_> {
    api_get!(
        /// Get the faction warfare statistics of a corporation.
        get_corporation_stats,
        "GetCorporationsCorporationIdFwStats",
        RequestType::Authenticated,
        CorporationFwStats,
        (corporation_id: i64) => "{corporation_id}"
    );
}
