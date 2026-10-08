#![allow(unused)]

use crate::prelude::*;

/// Endpoints for Industry
pub struct IndustryGroup<'a> {
    pub(crate) esi: &'a Esi,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct CostIndex {
    pub activity: String,
    pub cost_index: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct IndustrialSystem {
    pub cost_indices: Vec<CostIndex>,
    pub solar_system_id: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct IndustryJob {
    pub activity_id: i64,
    pub blueprint_id: i64,
    pub blueprint_location_id: i64,
    pub blueprint_type_id: i64,
    pub completed_character_id: Option<i64>,
    pub completed_date: Option<String>,
    pub cost: Option<f64>,
    pub duration: i64,
    pub end_date: String,
    pub facility_id: i64,
    pub installer_id: i64,
    pub job_id: i64,
    pub licensed_runs: Option<i64>,
    pub output_location_id: i64,
    pub pause_date: Option<String>,
    pub probability: Option<f64>,
    pub product_type_id: Option<i64>,
    pub runs: i64,
    pub start_date: String,
    pub station_id: i64,
    pub status: String,
    pub successful_runs: Option<i64>,
}

impl IndustryGroup<'_> {
    api_get!(
        /// Returns a list of solar systems with the cost index for every
        /// activity
        get_industry_systems,
        "GetIndustrySystems",
        RequestType::Public,
        Vec<IndustrialSystem>,
    );

    api_get!(
        /// List industry jobs placed by a character
        get_character_industry_jobs,
        "GetCharactersCharacterIdIndustryJobs",
        RequestType::Authenticated,
        Vec<IndustryJob>,
        (character_id: i64) => "{character_id}";
        Optional(include_completed: bool) => "include_completed"
    );
}

/// A public industry facility.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct IndustryFacility {
    pub facility_id: i64,
    pub owner_id: i64,
    pub region_id: i64,
    pub solar_system_id: i64,
    pub tax: Option<f64>,
    pub type_id: i64,
}

impl IndustryGroup<'_> {
    api_get!(
        /// List the public industry facilities.
        get_facilities,
        "GetIndustryFacilities",
        RequestType::Public,
        Vec<IndustryFacility>,
    );
}

/// A day of mining by a character.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct MiningEntry {
    pub date: String,
    pub quantity: i64,
    pub solar_system_id: i64,
    pub type_id: i64,
}

/// A moon mining extraction of a corporation.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct MiningExtraction {
    pub chunk_arrival_time: String,
    pub extraction_start_time: String,
    pub moon_id: i64,
    pub natural_decay_time: String,
    pub structure_id: i64,
}

/// The kind of structure that observes mining.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObserverType {
    /// A structure.
    Structure,
    /// A kind this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// A structure that observed mining for a corporation.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct MiningObserver {
    pub last_updated: String,
    pub observer_id: i64,
    pub observer_type: ObserverType,
}

/// Mining recorded by an observer for one character.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct ObservedMining {
    pub character_id: i64,
    pub last_updated: String,
    pub quantity: i64,
    pub recorded_corporation_id: i64,
    pub type_id: i64,
}

/// Status of a corporation industry job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IndustryJobStatus {
    /// The job is running.
    Active,
    /// The job was cancelled.
    Cancelled,
    /// The job was delivered.
    Delivered,
    /// The job is paused.
    Paused,
    /// The job is ready to be delivered.
    Ready,
    /// The job was reverted.
    Reverted,
    /// A status this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// An industry job placed by a corporation.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct CorporationIndustryJob {
    pub activity_id: i64,
    pub blueprint_id: i64,
    pub blueprint_location_id: i64,
    pub blueprint_type_id: i64,
    pub completed_character_id: Option<i64>,
    pub completed_date: Option<String>,
    pub cost: Option<f64>,
    pub duration: i64,
    pub end_date: String,
    pub facility_id: i64,
    pub installer_id: i64,
    pub job_id: i64,
    pub licensed_runs: Option<i64>,
    pub location_id: i64,
    pub output_location_id: i64,
    pub pause_date: Option<String>,
    pub probability: Option<f64>,
    pub product_type_id: Option<i64>,
    pub runs: i64,
    pub start_date: String,
    pub status: IndustryJobStatus,
    pub successful_runs: Option<i64>,
}

impl IndustryGroup<'_> {
    api_get!(
        /// Paginated record of the mining done by a character in the last 30 days.
        get_character_mining,
        "GetCharactersCharacterIdMining",
        RequestType::Authenticated,
        Vec<MiningEntry>,
        (character_id: i64) => "{character_id}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// Moon mining extractions of a corporation.
        get_corporation_mining_extractions,
        "GetCorporationCorporationIdMiningExtractions",
        RequestType::Authenticated,
        Vec<MiningExtraction>,
        (corporation_id: i64) => "{corporation_id}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// Structures that observed mining for a corporation.
        get_corporation_mining_observers,
        "GetCorporationCorporationIdMiningObservers",
        RequestType::Authenticated,
        Vec<MiningObserver>,
        (corporation_id: i64) => "{corporation_id}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// Mining recorded by one observer structure of a corporation.
        get_corporation_mining_observer,
        "GetCorporationCorporationIdMiningObserversObserverId",
        RequestType::Authenticated,
        Vec<ObservedMining>,
        (corporation_id: i64) => "{corporation_id}",
        (observer_id: i64) => "{observer_id}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// List industry jobs placed by a corporation.
        get_corporation_industry_jobs,
        "GetCorporationsCorporationIdIndustryJobs",
        RequestType::Authenticated,
        Vec<CorporationIndustryJob>,
        (corporation_id: i64) => "{corporation_id}";
        Optional(include_completed: bool) => "include_completed",
        Optional(page: i32) => "page"
    );
}

#[cfg(test)]
mod mining_tests {
    use super::*;

    #[test]
    fn test_parse_mining_and_jobs() {
        let observer: MiningObserver = serde_json::from_str(
            r#"{"last_updated":"2026-10-01","observer_id":1,"observer_type":"other"}"#,
        )
        .unwrap();
        assert_eq!(observer.observer_type, ObserverType::Unrecognized);
        let job: CorporationIndustryJob = serde_json::from_str(
            r#"{"activity_id":1,"blueprint_id":2,"blueprint_location_id":3,"blueprint_type_id":4,
            "duration":5,"end_date":"x","facility_id":6,"installer_id":7,"job_id":8,"location_id":9,
            "output_location_id":10,"runs":1,"start_date":"y","status":"ready"}"#,
        )
        .unwrap();
        assert_eq!(job.status, IndustryJobStatus::Ready);
        assert!(job.cost.is_none());
    }
}
