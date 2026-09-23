//! Mapping of rfesi's legacy snake_case operation IDs (from the retired
//! Swagger spec) to the operation IDs of the ESI OpenAPI spec.
//!
//! Legacy IDs are still accepted by [`crate::prelude::Esi::get_endpoint_for_op_id`],
//! which logs a deprecation warning naming the OpenAPI ID. They will be
//! removed in 0.2.0.

/// Pairs of `(legacy_swagger_id, openapi_id)`, sorted by legacy ID.
pub(crate) const LEGACY_OP_IDS: &[(&str, &str)] = &[
    ("get_alliances", "GetAlliances"),
    ("get_alliances_alliance_id", "GetAlliancesAllianceId"),
    (
        "get_alliances_alliance_id_corporations",
        "GetAlliancesAllianceIdCorporations",
    ),
    (
        "get_alliances_alliance_id_icons",
        "GetAlliancesAllianceIdIcons",
    ),
    ("get_characters_character_id", "GetCharactersDetail"),
    (
        "get_characters_character_id_assets",
        "GetCharactersCharacterIdAssets",
    ),
    (
        "get_characters_character_id_blueprints",
        "GetCharactersCharacterIdBlueprints",
    ),
    (
        "get_characters_character_id_clones",
        "GetCharactersCharacterIdClones",
    ),
    (
        "get_characters_character_id_corporationhistory",
        "GetCharactersCharacterIdCorporationhistory",
    ),
    (
        "get_characters_character_id_implants",
        "GetCharactersCharacterIdImplants",
    ),
    (
        "get_characters_character_id_industry_jobs",
        "GetCharactersCharacterIdIndustryJobs",
    ),
    (
        "get_characters_character_id_killmails_recent",
        "GetCharactersCharacterIdKillmailsRecent",
    ),
    (
        "get_characters_character_id_location",
        "GetCharactersCharacterIdLocation",
    ),
    (
        "get_characters_character_id_mail_labels",
        "GetCharactersCharacterIdMailLabels",
    ),
    (
        "get_characters_character_id_notifications",
        "GetCharactersCharacterIdNotifications",
    ),
    (
        "get_characters_character_id_online",
        "GetCharactersCharacterIdOnline",
    ),
    (
        "get_characters_character_id_orders",
        "GetCharactersCharacterIdOrders",
    ),
    (
        "get_characters_character_id_portrait",
        "GetCharactersCharacterIdPortrait",
    ),
    (
        "get_characters_character_id_search",
        "GetCharactersCharacterIdSearch",
    ),
    (
        "get_characters_character_id_ship",
        "GetCharactersCharacterIdShip",
    ),
    (
        "get_characters_character_id_skills",
        "GetCharactersCharacterIdSkills",
    ),
    (
        "get_characters_character_id_wallet",
        "GetCharactersCharacterIdWallet",
    ),
    (
        "get_characters_character_id_wallet_transactions",
        "GetCharactersCharacterIdWalletTransactions",
    ),
    (
        "get_corporations_corporation_id",
        "GetCorporationsCorporationId",
    ),
    (
        "get_corporations_corporation_id_alliancehistory",
        "GetCorporationsCorporationIdAlliancehistory",
    ),
    (
        "get_corporations_corporation_id_assets",
        "GetCorporationsCorporationIdAssets",
    ),
    (
        "get_corporations_corporation_id_members",
        "GetCorporationsCorporationIdMembers",
    ),
    ("get_corporations_npccorps", "GetCorporationsNpccorps"),
    ("get_fw_leaderboards", "GetFwLeaderboards"),
    (
        "get_fw_leaderboards_characters",
        "GetFwLeaderboardsCharacters",
    ),
    (
        "get_fw_leaderboards_corporations",
        "GetFwLeaderboardsCorporations",
    ),
    ("get_fw_stats", "GetFwStats"),
    ("get_fw_systems", "GetFwSystems"),
    ("get_fw_wars", "GetFwWars"),
    ("get_incursions", "GetIncursions"),
    ("get_industry_systems", "GetIndustrySystems"),
    (
        "get_killmails_killmail_id_killmail_hash",
        "GetKillmailsKillmailIdKillmailHash",
    ),
    ("get_markets_prices", "GetMarketsPrices"),
    ("get_markets_region_id_history", "GetMarketsRegionIdHistory"),
    ("get_markets_region_id_orders", "GetMarketsRegionIdOrders"),
    (
        "get_universe_categories_category_id",
        "GetUniverseCategoriesCategoryId",
    ),
    ("get_universe_constellations", "GetUniverseConstellations"),
    (
        "get_universe_constellations_constellation_id",
        "GetUniverseConstellationsConstellationId",
    ),
    ("get_universe_groups_group_id", "GetUniverseGroupsGroupId"),
    ("get_universe_regions", "GetUniverseRegions"),
    (
        "get_universe_regions_region_id",
        "GetUniverseRegionsRegionId",
    ),
    (
        "get_universe_stations_station_id",
        "GetUniverseStationsStationId",
    ),
    (
        "get_universe_structures_structure_id",
        "GetUniverseStructuresStructureId",
    ),
    ("get_universe_systems", "GetUniverseSystems"),
    (
        "get_universe_systems_system_id",
        "GetUniverseSystemsSystemId",
    ),
    ("get_universe_types", "GetUniverseTypes"),
    ("get_universe_types_type_id", "GetUniverseTypesTypeId"),
    ("post_characters_affiliation", "PostCharactersAffiliation"),
    (
        "post_characters_character_id_assets_locations",
        "PostCharactersCharacterIdAssetsLocations",
    ),
    (
        "post_characters_character_id_assets_names",
        "PostCharactersCharacterIdAssetsNames",
    ),
    (
        "post_corporations_corporation_id_assets_locations",
        "PostCorporationsCorporationIdAssetsLocations",
    ),
    (
        "post_corporations_corporation_id_assets_names",
        "PostCorporationsCorporationIdAssetsNames",
    ),
    (
        "post_ui_openwindow_marketdetails",
        "PostUiOpenwindowMarketdetails",
    ),
    ("post_universe_ids", "PostUniverseIds"),
];

/// Resolve a legacy snake_case operation ID to its OpenAPI equivalent.
pub(crate) fn openapi_id_for(legacy_id: &str) -> Option<&'static str> {
    LEGACY_OP_IDS
        .binary_search_by(|(legacy, _)| legacy.cmp(&legacy_id))
        .ok()
        .map(|i| LEGACY_OP_IDS[i].1)
}

#[cfg(test)]
mod tests {
    use super::{openapi_id_for, LEGACY_OP_IDS};

    #[test]
    fn test_legacy_table_is_sorted() {
        assert!(LEGACY_OP_IDS.windows(2).all(|w| w[0].0 < w[1].0));
    }

    #[test]
    fn test_legacy_lookup() {
        assert_eq!(
            openapi_id_for("get_markets_region_id_orders"),
            Some("GetMarketsRegionIdOrders")
        );
        assert_eq!(
            openapi_id_for("get_characters_character_id"),
            Some("GetCharactersDetail")
        );
        assert_eq!(openapi_id_for("GetMarketsPrices"), None);
    }
}
