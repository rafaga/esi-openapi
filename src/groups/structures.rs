use crate::prelude::*;

/// Endpoints for Structures
pub struct StructuresGroup<'a> {
    pub(crate) esi: &'a Esi,
}

/// A mercenary den owned by a character.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct MercenaryDenSummary {
    /// ID of the mercenary den.
    pub id: i64,
    /// Planet the mercenary den is on.
    pub planet_id: i64,
}

#[derive(Debug, Deserialize)]
struct MercenaryDens {
    mercenary_dens: Vec<MercenaryDenSummary>,
}

/// The level of a mercenary den evolution track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum EvolutionLevel {
    /// Not specified.
    Unspecified,
    /// Level 0.
    Level0,
    /// Level 1.
    Level1,
    /// Level 2.
    Level2,
    /// Level 3.
    Level3,
    /// Level 4.
    Level4,
    /// A value this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// Progress on one evolution track of a mercenary den.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct EvolutionTrack {
    /// Amount accumulated.
    pub amount: i64,
    /// Current level.
    pub level: EvolutionLevel,
}

/// The evolution tracks of a mercenary den.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct MercenaryDenEvolution {
    /// Anarchy track.
    pub anarchy: EvolutionTrack,
    /// Development track.
    pub development: EvolutionTrack,
}

/// Infomorphs stored in a mercenary den.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Infomorphs {
    /// Amount stored.
    pub amount: i64,
}

/// A timer that has an end date.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct EndTimer {
    /// When the timer ends.
    pub end: String,
}

/// The skyhook a mercenary den is linked to.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct DenSkyhook {
    /// Corporation that owns the skyhook.
    pub corporation_id: i64,
    /// ID of the skyhook.
    pub id: i64,
    /// Planet the skyhook is on.
    pub planet_id: i64,
}

/// The state of a mercenary den.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum MercenaryDenState {
    /// Not specified.
    Unspecified,
    /// Running.
    Running,
    /// Paused.
    Paused,
    /// Disabled.
    Disabled,
    /// A value this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// The details of a mercenary den.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct MercenaryDen {
    /// Evolution tracks.
    pub evolution: MercenaryDenEvolution,
    /// ID of the mercenary den.
    pub id: i64,
    /// Infomorphs stored.
    pub infomorphs: Infomorphs,
    /// Reinforcement timer, if reinforced.
    pub reinforcement_timer: Option<EndTimer>,
    /// Linked skyhook.
    pub skyhook: DenSkyhook,
    /// State of the mercenary den.
    pub state: MercenaryDenState,
    /// Type ID of the mercenary den.
    pub type_id: i64,
}

/// A skyhook owned by a corporation.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SkyhookSummary {
    /// ID of the skyhook.
    pub id: i64,
    /// Planet the skyhook is on.
    pub planet_id: i64,
}

#[derive(Debug, Deserialize)]
struct Skyhooks {
    skyhooks: Vec<SkyhookSummary>,
}

/// The state of a skyhook.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum SkyhookState {
    /// Not specified.
    Unspecified,
    /// Shield vulnerable.
    ShieldVulnerable,
    /// Armor reinforced.
    ArmorReinforced,
    /// Armor vulnerable.
    ArmorVulnerable,
    /// Hull reinforced.
    HullReinforced,
    /// Hull vulnerable.
    HullVulnerable,
    /// A value this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// Reagent stock in a skyhook.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SkyhookReagent {
    /// Last time the reagent was cycled.
    pub last_cycle: String,
    /// Secured stock.
    pub secured_stock: i64,
    /// Type ID of the reagent.
    pub type_id: i64,
    /// Unsecured stock.
    pub unsecured_stock: i64,
}

/// A time window.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TimeWindow {
    /// Start of the window.
    pub start: String,
    /// End of the window.
    pub end: String,
}

/// The details of a skyhook.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Skyhook {
    /// Effective workforce.
    pub effective_workforce: Option<i64>,
    /// ID of the skyhook.
    pub id: i64,
    /// Whether the skyhook is active.
    pub is_active: bool,
    /// Planet the skyhook is on.
    pub planet_id: i64,
    /// Reagent stock.
    #[serde(default)]
    pub reagents: Vec<SkyhookReagent>,
    /// Reinforcement timer, if reinforced.
    pub reinforcement_timer: Option<EndTimer>,
    /// State of the skyhook.
    pub state: SkyhookState,
    /// Window in which the contents can be stolen.
    pub theft_vulnerability: Option<TimeWindow>,
}

/// A sovereignty hub owned by a corporation.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SovereigntyHubSummary {
    /// ID of the hub.
    pub id: i64,
    /// Solar system of the hub.
    pub solar_system_id: i64,
}

#[derive(Debug, Deserialize)]
struct SovereigntyHubs {
    sovereignty_hubs: Vec<SovereigntyHubSummary>,
}

/// Reagent burned by a sovereignty hub.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct HubReagent {
    /// Amount in the bay.
    pub amount: i64,
    /// Amount burned per hour.
    pub burning_per_hour: i64,
    /// Type ID of the reagent.
    pub type_id: i64,
}

/// The reagent bay of a sovereignty hub.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ReagentBay {
    /// When the bay was last updated.
    pub last_updated: String,
    /// Reagents in the bay.
    pub reagents: Vec<HubReagent>,
}

/// Allocated and available amount of a resource.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ResourceUsage {
    /// Amount allocated.
    pub allocated: i64,
    /// Amount available.
    pub available: i64,
}

/// Power and workforce of a sovereignty hub.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct HubResources {
    /// Power usage.
    pub power: ResourceUsage,
    /// Workforce usage.
    pub workforce: ResourceUsage,
}

/// The power state of a hub upgrade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum UpgradePowerState {
    /// Not specified.
    Unspecified,
    /// Online.
    Online,
    /// Offline.
    Offline,
    /// Low power.
    Low,
    /// Pending.
    Pending,
    /// A value this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// An upgrade installed in a sovereignty hub.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct HubUpgrade {
    /// Power state of the upgrade.
    pub power_state: UpgradePowerState,
    /// Type ID of the upgrade.
    pub type_id: i64,
}

/// A solar system and an amount of workforce.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct WorkforceFlow {
    /// Amount of workforce.
    pub amount: Option<i64>,
    /// Solar system the workforce goes to or comes from.
    pub solar_system_id: Option<i64>,
}

/// The workforce transport of a sovereignty hub, as configured or as running.
///
/// Exactly one of the fields is set.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct WorkforceTransportMode {
    /// Workforce is exported.
    pub export: Option<WorkforceFlow>,
    /// Workforce is imported from the listed systems.
    pub import: Option<WorkforceImport>,
    /// Workforce is in transit.
    pub transit: Option<bool>,
}

/// Workforce import sources.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct WorkforceImport {
    /// Systems workforce is imported from.
    pub sources: Vec<WorkforceFlow>,
}

/// The workforce transport of a sovereignty hub.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct WorkforceTransport {
    /// Configured mode.
    pub configuration: WorkforceTransportMode,
    /// Mode currently running.
    pub state: WorkforceTransportMode,
}

/// The details of a sovereignty hub.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct CorporationSovereigntyHub {
    /// Access list that controls fuel access.
    pub fuel_access_list_id: Option<i64>,
    /// ID of the hub.
    pub id: i64,
    /// Reagent bay.
    pub reagent_bay: ReagentBay,
    /// Power and workforce.
    pub resources: HubResources,
    /// Solar system of the hub.
    pub solar_system_id: i64,
    /// Installed upgrades.
    pub upgrades: Vec<HubUpgrade>,
    /// Vulnerability window.
    pub vulnerability_window: Option<TimeWindow>,
    /// Workforce transport.
    pub workforce_transport: WorkforceTransport,
}

impl StructuresGroup<'_> {
    /// List the mercenary dens of a character.
    pub async fn get_character_mercenary_dens(
        &self,
        character_id: i64,
    ) -> EsiResult<Vec<MercenaryDenSummary>> {
        let path = self
            .esi
            .get_endpoint_for_op_id("GetCharactersStructuresMercenaryDensListing")?
            .replace("{character_id}", &character_id.to_string());
        let wrapper: MercenaryDens = self
            .esi
            .query("GET", RequestType::Authenticated, &path, None, None)
            .await?;
        Ok(wrapper.mercenary_dens)
    }

    api_get!(
        /// Get the details of a mercenary den of a character.
        get_character_mercenary_den,
        "GetCharactersStructuresMercenaryDensDetail",
        RequestType::Authenticated,
        MercenaryDen,
        (character_id: i64) => "{character_id}",
        (mercenary_den_id: i64) => "{mercenary_den_id}"
    );

    /// List the skyhooks of a corporation.
    ///
    /// Requires the Station Manager role.
    pub async fn get_corporation_skyhooks(
        &self,
        corporation_id: i64,
    ) -> EsiResult<Vec<SkyhookSummary>> {
        let path = self
            .esi
            .get_endpoint_for_op_id("GetCorporationsStructuresSkyhooksListing")?
            .replace("{corporation_id}", &corporation_id.to_string());
        let wrapper: Skyhooks = self
            .esi
            .query("GET", RequestType::Authenticated, &path, None, None)
            .await?;
        Ok(wrapper.skyhooks)
    }

    api_get!(
        /// Get the details of a skyhook of a corporation.
        ///
        /// Requires the Station Manager role.
        get_corporation_skyhook,
        "GetCorporationsStructuresSkyhooksDetail",
        RequestType::Authenticated,
        Skyhook,
        (corporation_id: i64) => "{corporation_id}",
        (skyhook_id: i64) => "{skyhook_id}"
    );

    /// List the sovereignty hubs of a corporation.
    pub async fn get_corporation_sovereignty_hubs(
        &self,
        corporation_id: i64,
    ) -> EsiResult<Vec<SovereigntyHubSummary>> {
        let path = self
            .esi
            .get_endpoint_for_op_id("GetCorporationsStructuresSovereigntyHubsListing")?
            .replace("{corporation_id}", &corporation_id.to_string());
        let wrapper: SovereigntyHubs = self
            .esi
            .query("GET", RequestType::Authenticated, &path, None, None)
            .await?;
        Ok(wrapper.sovereignty_hubs)
    }

    api_get!(
        /// Get the details of a sovereignty hub of a corporation.
        ///
        /// Requires the Station Manager role.
        get_corporation_sovereignty_hub,
        "GetCorporationsStructuresSovereigntyHubsDetail",
        RequestType::Authenticated,
        CorporationSovereigntyHub,
        (corporation_id: i64) => "{corporation_id}",
        (sovereignty_hub_id: i64) => "{sovereignty_hub_id}"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mercenary_den() {
        let json = r#"{"evolution": {"anarchy": {"amount": 5, "level": "Level2"},
            "development": {"amount": 1, "level": "Level9"}},
            "id": 1040000000000, "infomorphs": {"amount": 10},
            "skyhook": {"corporation_id": 98000001, "id": 2, "planet_id": 40000001},
            "state": "Running", "type_id": 85230}"#;
        let den: MercenaryDen = serde_json::from_str(json).unwrap();
        assert_eq!(den.id, 1_040_000_000_000);
        assert_eq!(den.evolution.anarchy.level, EvolutionLevel::Level2);
        assert_eq!(
            den.evolution.development.level,
            EvolutionLevel::Unrecognized
        );
        assert!(den.reinforcement_timer.is_none());
    }

    #[test]
    fn test_parse_skyhook() {
        let json = r#"{"id": 1, "is_active": true, "planet_id": 2, "state": "ArmorReinforced",
            "reagents": [{"last_cycle": "t", "secured_stock": 1, "type_id": 81143, "unsecured_stock": 2}],
            "theft_vulnerability": {"start": "a", "end": "b"}}"#;
        let hook: Skyhook = serde_json::from_str(json).unwrap();
        assert_eq!(hook.state, SkyhookState::ArmorReinforced);
        assert_eq!(hook.reagents.len(), 1);
        assert!(hook.effective_workforce.is_none());
    }

    #[test]
    fn test_parse_sovereignty_hub() {
        let json = r#"{"id": 1040000000001, "solar_system_id": 30000142,
            "reagent_bay": {"last_updated": "t", "reagents": [{"amount": 1, "burning_per_hour": 2, "type_id": 81144}]},
            "resources": {"power": {"allocated": 1, "available": 2}, "workforce": {"allocated": 3, "available": 4}},
            "upgrades": [{"power_state": "Low", "type_id": 81615}],
            "workforce_transport": {
                "configuration": {"import": {"sources": [{"solar_system_id": 30000143}]}},
                "state": {"transit": true}}}"#;
        let hub: CorporationSovereigntyHub = serde_json::from_str(json).unwrap();
        assert_eq!(hub.upgrades[0].power_state, UpgradePowerState::Low);
        let cfg = &hub.workforce_transport.configuration;
        assert_eq!(
            cfg.import.as_ref().unwrap().sources[0].solar_system_id,
            Some(30000143)
        );
        assert_eq!(hub.workforce_transport.state.transit, Some(true));
    }
}
