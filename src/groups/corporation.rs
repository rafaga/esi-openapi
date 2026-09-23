use crate::prelude::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct CorporationPublicInfo {
    pub alliance_id: Option<i32>,
    pub ceo_id: Option<i32>,
    pub creator_id: Option<i32>,
    pub date_founded: Option<String>,
    pub description: Option<String>,
    pub enlisted_faction_id: Option<i32>,
    /// `legal` or `illegal`.
    pub friendly_fire: Option<String>,
    pub home_station_id: Option<i32>,
    pub member_count: i32,
    pub name: String,
    pub palette: Option<CorporationPalette>,
    pub shares: Option<u64>,
    /// `active` or `closed`.
    pub state: Option<String>,
    pub tax_rates: Option<CorporationTaxRates>,
    pub ticker: Option<String>,
    /// `player_owned` or `npc_owned`.
    #[serde(rename = "type")]
    pub corporation_type: Option<String>,
    pub url: Option<String>,
    pub war_eligible: Option<bool>,
}

/// Corporation tax rates, as percentages (0.0 - 100.0).
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CorporationTaxRates {
    /// ISK tax rate.
    pub isk: Option<f64>,
    /// Loyalty point tax rate.
    pub loyalty_point: Option<f64>,
}

/// Corporation palette colors (`#rrggbb`).
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CorporationPalette {
    /// Main color.
    pub main_color: Option<String>,
    /// Secondary color.
    pub secondary_color: Option<String>,
    /// Tertiary color.
    pub tertiary_color: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct CorporationHistoryItem {
    pub alliance_id: Option<i32>,
    pub is_deleted: Option<bool>,
    pub record_id: i32,
    pub start_date: String,
}

/// Endpoints for Corporation
pub struct CorporationGroup<'a> {
    pub(crate) esi: &'a Esi,
}

impl CorporationGroup<'_> {
    api_get!(
        /// Get a corporation's public info.
        get_public_info,
        "GetCorporationsCorporationId",
        RequestType::Public,
        CorporationPublicInfo,
        (corporation_id: i32) => "{corporation_id}"
    );

    api_get!(
        /// Get a corporation's alliance history.
        get_history,
        "GetCorporationsCorporationIdAlliancehistory",
        RequestType::Public,
        Vec<CorporationHistoryItem>,
        (corporation_id: i32) => "{corporation_id}"
    );

    api_get!(
        /// Get a corporation's member list.
        ///
        /// Requires the auth'd character to be in the corporation.
        get_members,
        "GetCorporationsCorporationIdMembers",
        RequestType::Authenticated,
        Vec<u64>,
        (corporation_id: i32) => "{corporation_id}"
    );

    api_get!(
        /// Get a list of NPC corporations.
        get_npc_corps,
        "GetCorporationsNpccorps",
        RequestType::Public,
        Vec<u64>,
    );

    // more endpoints ...
}
