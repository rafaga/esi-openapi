use crate::prelude::*;

/// Endpoints for Incursions
pub struct IncursionsGroup<'a> {
    pub(crate) esi: &'a Esi,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct Incursion {
    pub constellation_id: i64,
    pub faction_id: i64,
    pub has_boss: bool,
    pub infested_solar_systems: Vec<i64>,
    pub influence: f64,
    pub staging_solar_system_id: i64,
    pub state: String,
    #[serde(rename = "type")]
    pub incursion_type: String,
}

impl IncursionsGroup<'_> {
    api_get!(
        /// Get the current incursions.
        list,
        "GetIncursions",
        RequestType::Public,
        Vec<Incursion>,
    );
}
