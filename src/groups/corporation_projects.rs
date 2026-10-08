use crate::groups::Page;
use crate::prelude::*;
use uuid::Uuid;

/// Endpoints for Corporation Projects
pub struct CorporationProjectsGroup<'a> {
    pub(crate) esi: &'a Esi,
}

/// Which projects to list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectStateFilter {
    /// All projects.
    All,
    /// Only active projects.
    Active,
}

impl std::fmt::Display for ProjectStateFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ProjectStateFilter::All => "All",
            ProjectStateFilter::Active => "Active",
        })
    }
}

/// The state of a corporation project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum ProjectState {
    /// Not specified.
    Unspecified,
    /// Open.
    Active,
    /// Closed.
    Closed,
    /// Completed.
    Completed,
    /// Expired.
    Expired,
    /// Deleted.
    Deleted,
    /// A value this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// Progress of a project.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProjectProgress {
    /// Current progress.
    pub current: i64,
    /// Progress needed to complete the project.
    pub desired: i64,
}

/// Reward of a project, in ISK.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ProjectReward {
    /// Initial reward.
    pub initial: f64,
    /// Reward left.
    pub remaining: f64,
}

/// A project in the project listing.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ProjectSummary {
    /// ID of the project.
    pub id: Uuid,
    /// When the project was last modified.
    pub last_modified: String,
    /// Name of the project.
    pub name: String,
    /// Progress of the project.
    pub progress: ProjectProgress,
    /// Reward of the project.
    pub reward: Option<ProjectReward>,
    /// State of the project.
    pub state: ProjectState,
}

/// A place a project refers to. Exactly one of the fields is set.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProjectLocation {
    /// A solar system.
    pub solar_system_id: Option<i64>,
    /// A constellation.
    pub constellation_id: Option<i64>,
    /// A region.
    pub region_id: Option<i64>,
}

/// An entity a project refers to. Exactly one of the fields is set.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProjectIdentity {
    /// A character.
    pub character_id: Option<i64>,
    /// A corporation.
    pub corporation_id: Option<i64>,
    /// An alliance.
    pub alliance_id: Option<i64>,
    /// A faction.
    pub faction_id: Option<i64>,
}

/// A type or a group of types. Exactly one of the fields is set.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TypeOrGroup {
    /// A type.
    pub type_id: Option<i64>,
    /// A group of types.
    pub group_id: Option<i64>,
}

/// A place to dock. Exactly one of the fields is set.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct DockingLocation {
    /// A structure.
    pub structure_id: Option<i64>,
    /// A station.
    pub station_id: Option<i64>,
}

/// An archetype of a faction warfare complex.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProjectArchetype {
    /// ID of the archetype.
    pub archetype_id: Option<i64>,
}

/// A faction a project refers to.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProjectFaction {
    /// ID of the faction.
    pub faction_id: Option<i64>,
}

/// A signature type a project refers to.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProjectSignature {
    /// ID of the signature type.
    pub signature_type_id: Option<i64>,
}

/// A corporation a project refers to.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProjectCorporation {
    /// ID of the corporation.
    pub corporation_id: Option<i64>,
}

/// Configuration of the faction warfare complex projects.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct FwComplexConfig {
    /// Archetypes of the complexes.
    #[serde(default)]
    pub archetypes: Vec<ProjectArchetype>,
    /// Factions involved.
    #[serde(default)]
    pub factions: Vec<ProjectFaction>,
    /// Where the complexes are.
    #[serde(default)]
    pub locations: Vec<ProjectLocation>,
}

/// Configuration of projects that involve ships and other pilots.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ShipActivityConfig {
    /// Entities involved.
    #[serde(default)]
    pub identities: Vec<ProjectIdentity>,
    /// Where it happens.
    #[serde(default)]
    pub locations: Vec<ProjectLocation>,
    /// Ships involved.
    #[serde(default)]
    pub ships: Vec<TypeOrGroup>,
}

/// Configuration of projects that only refer to locations.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LocationsConfig {
    /// Where it happens.
    #[serde(default)]
    pub locations: Vec<ProjectLocation>,
}

/// Configuration of an item delivery project.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct DeliverItemConfig {
    /// Where the items can be delivered.
    #[serde(default)]
    pub docking_locations: Vec<DockingLocation>,
    /// Items to deliver.
    #[serde(default)]
    pub items: Vec<TypeOrGroup>,
    /// Office to deliver to.
    pub office_id: Option<i64>,
}

/// Configuration of a loyalty point project.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct EarnLoyaltyPointConfig {
    /// Corporations whose loyalty points count.
    #[serde(default)]
    pub corporations: Vec<ProjectCorporation>,
}

/// Who must own the items of a manufacturing project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum ManufactureOwner {
    /// Anyone.
    Any,
    /// A corporation.
    Corporation,
    /// A character.
    Character,
    /// A value this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// Configuration of a manufacturing project.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ManufactureItemConfig {
    /// Where the items can be delivered.
    #[serde(default)]
    pub docking_locations: Vec<DockingLocation>,
    /// Items to manufacture.
    #[serde(default)]
    pub items: Vec<TypeOrGroup>,
    /// Who must own the items.
    pub owner: ManufactureOwner,
}

/// Configuration of a mining project.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct MineMaterialConfig {
    /// Where to mine.
    #[serde(default)]
    pub locations: Vec<ProjectLocation>,
    /// Materials to mine.
    #[serde(default)]
    pub materials: Vec<TypeOrGroup>,
}

/// Configuration of a signature scanning project.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ScanSignatureConfig {
    /// Where to scan.
    #[serde(default)]
    pub locations: Vec<ProjectLocation>,
    /// Signature types to scan.
    #[serde(default)]
    pub signatures: Vec<ProjectSignature>,
}

/// The kind of conflict a ship insurance project covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum ConflictType {
    /// Any conflict.
    Any,
    /// Player versus player.
    Pvp,
    /// Player versus environment.
    Pve,
    /// A value this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// Configuration of a ship insurance project.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ShipInsuranceConfig {
    /// Kind of conflict covered.
    pub conflict_type: ConflictType,
    /// Entities involved.
    #[serde(default)]
    pub identities: Vec<ProjectIdentity>,
    /// Where it applies.
    #[serde(default)]
    pub locations: Vec<ProjectLocation>,
    /// Whether implants are reimbursed.
    pub reimburse_implants: bool,
    /// Ships covered.
    #[serde(default)]
    pub ships: Vec<TypeOrGroup>,
}

/// A project type this version of the crate does not know about.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct UnknownProjectConfig {
    /// Raw data of the configuration.
    pub data: Option<serde_json::Value>,
    /// Name of the project type.
    #[serde(rename = "type")]
    pub config_type: String,
}

/// The configuration of a project. Exactly one of the fields is set, depending
/// on the project type; the fields that are not set are `None`.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
pub struct ProjectConfiguration {
    /// Capture a faction warfare complex.
    pub capture_fw_complex: Option<FwComplexConfig>,
    /// Damage ships.
    pub damage_ship: Option<ShipActivityConfig>,
    /// Defend a faction warfare complex.
    pub defend_fw_complex: Option<FwComplexConfig>,
    /// Deliver items.
    pub deliver_item: Option<DeliverItemConfig>,
    /// Destroy NPCs.
    pub destroy_npc: Option<LocationsConfig>,
    /// Destroy ships.
    pub destroy_ship: Option<ShipActivityConfig>,
    /// Earn loyalty points.
    pub earn_loyalty_point: Option<EarnLoyaltyPointConfig>,
    /// Lose ships.
    pub lost_ship: Option<ShipActivityConfig>,
    /// Manually tracked project (no data).
    pub manual: Option<serde_json::Value>,
    /// Manufacture items.
    pub manufacture_item: Option<ManufactureItemConfig>,
    /// Mine materials.
    pub mine_material: Option<MineMaterialConfig>,
    /// Remote boost shields.
    pub remote_boost_shield: Option<ShipActivityConfig>,
    /// Remote repair armor.
    pub remote_repair_armor: Option<ShipActivityConfig>,
    /// Salvage wrecks.
    pub salvage_wreck: Option<LocationsConfig>,
    /// Scan signatures.
    pub scan_signature: Option<ScanSignatureConfig>,
    /// Insure ships.
    pub ship_insurance: Option<ShipInsuranceConfig>,
    /// A project type this version of the crate does not know about.
    pub unknown: Option<UnknownProjectConfig>,
}

/// Limits and rewards for contributions to a project.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ContributionRules {
    /// Maximum number of participants.
    pub participation_limit: Option<i64>,
    /// ISK paid per contribution.
    pub reward_per_contribution: Option<f64>,
    /// Maximum number of submissions per participant.
    pub submission_limit: Option<i64>,
    /// Multiplier applied to submissions.
    pub submission_multiplier: Option<f64>,
}

/// The character that created a project.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProjectCreator {
    /// Character ID of the creator.
    pub id: i64,
    /// Name of the creator.
    pub name: String,
}

/// The career a project belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum ProjectCareer {
    /// Not specified.
    Unspecified,
    /// Explorer.
    Explorer,
    /// Industrialist.
    Industrialist,
    /// Enforcer.
    Enforcer,
    /// Soldier of Fortune.
    #[serde(rename = "Soldier of Fortune")]
    SoldierOfFortune,
    /// A value this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// General details of a project.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProjectDetails {
    /// Career of the project.
    pub career: ProjectCareer,
    /// When the project was created.
    pub created: String,
    /// Description of the project.
    pub description: String,
    /// When the project expires.
    pub expires: Option<String>,
    /// When the project finished.
    pub finished: Option<String>,
}

/// A corporation project.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Project {
    /// Configuration of the project.
    pub configuration: ProjectConfiguration,
    /// Contribution rules.
    pub contribution: Option<ContributionRules>,
    /// Creator of the project.
    pub creator: ProjectCreator,
    /// General details.
    pub details: ProjectDetails,
    /// ID of the project.
    pub id: Uuid,
    /// When the project was last modified.
    pub last_modified: String,
    /// Name of the project.
    pub name: String,
    /// Progress of the project.
    pub progress: ProjectProgress,
    /// Reward of the project.
    pub reward: Option<ProjectReward>,
    /// State of the project.
    pub state: ProjectState,
}

/// What a character contributed to a project.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProjectContribution {
    /// Amount contributed.
    pub contributed: i64,
    /// When the contribution was last modified.
    pub last_modified: Option<String>,
}

/// A contributor to a project.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProjectContributor {
    /// Amount contributed.
    pub contributed: i64,
    /// Character ID of the contributor.
    pub id: i64,
    /// Name of the contributor.
    pub name: String,
}

impl CorporationProjectsGroup<'_> {
    api_get!(
        /// List the projects of a corporation. Use the cursor of the returned
        /// page as `after` or `before` to walk through the list.
        list,
        "GetCorporationsProjectsListing",
        RequestType::Authenticated,
        Page<ProjectSummary>,
        (corporation_id: i64) => "{corporation_id}";
        Optional(after: String) => "after",
        Optional(before: String) => "before",
        Optional(limit: i64) => "limit",
        Optional(state: ProjectStateFilter) => "state"
    );

    api_get!(
        /// Get the details of a project of a corporation.
        get_project,
        "GetCorporationsProjectsDetail",
        RequestType::Authenticated,
        Project,
        (corporation_id: i64) => "{corporation_id}",
        (project_id: Uuid) => "{project_id}"
    );

    api_get!(
        /// Get what a character contributed to a project.
        get_contribution,
        "GetCorporationsProjectsContribution",
        RequestType::Authenticated,
        ProjectContribution,
        (corporation_id: i64) => "{corporation_id}",
        (project_id: Uuid) => "{project_id}",
        (character_id: i64) => "{character_id}"
    );

    api_get!(
        /// List the contributors to a project. Use the cursor of the returned
        /// page as `after` or `before` to walk through the list.
        ///
        /// Requires the Project Manager role.
        list_contributors,
        "GetCorporationsProjectsContributors",
        RequestType::Authenticated,
        Page<ProjectContributor>,
        (corporation_id: i64) => "{corporation_id}",
        (project_id: Uuid) => "{project_id}";
        Optional(after: String) => "after",
        Optional(before: String) => "before",
        Optional(limit: i64) => "limit"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_project_listing() {
        let json = r#"{"cursor": {"after": "abc"}, "projects": [
            {"id": "6f1d8a5e-0b7c-4d3e-9a2f-1c5b7e9d0a11", "last_modified": "t", "name": "n",
             "progress": {"current": 5000000000, "desired": 9000000000},
             "reward": {"initial": 1000.5, "remaining": 10.0}, "state": "Active"},
            {"id": "6f1d8a5e-0b7c-4d3e-9a2f-1c5b7e9d0a12", "last_modified": "t", "name": "m",
             "progress": {"current": 0, "desired": 1}, "state": "Archived"}]}"#;
        let page: Page<ProjectSummary> = serde_json::from_str(json).unwrap();
        assert_eq!(page.items[0].progress.current, 5_000_000_000);
        assert_eq!(page.items[0].state, ProjectState::Active);
        assert_eq!(page.items[1].state, ProjectState::Unrecognized);
        assert_eq!(page.cursor.unwrap().after.as_deref(), Some("abc"));
    }

    #[test]
    fn test_parse_project_with_each_configuration_shape() {
        let base = |config: &str| {
            format!(
                r#"{{"configuration": {config},
                "creator": {{"id": 2124758468, "name": "c"}},
                "details": {{"career": "Soldier of Fortune", "created": "t", "description": "d"}},
                "id": "6f1d8a5e-0b7c-4d3e-9a2f-1c5b7e9d0a11", "last_modified": "t", "name": "n",
                "progress": {{"current": 1, "desired": 2}}, "state": "Closed"}}"#
            )
        };
        let mining: Project = serde_json::from_str(&base(
            r#"{"mine_material": {"locations": [{"region_id": 10000002}],
                "materials": [{"group_id": 450}]}}"#,
        ))
        .unwrap();
        assert_eq!(mining.details.career, ProjectCareer::SoldierOfFortune);
        let cfg = mining.configuration.mine_material.unwrap();
        assert_eq!(cfg.locations[0].region_id, Some(10000002));
        assert_eq!(cfg.materials[0].group_id, Some(450));

        let insurance: Project = serde_json::from_str(&base(
            r#"{"ship_insurance": {"conflict_type": "Pvp", "reimburse_implants": true}}"#,
        ))
        .unwrap();
        assert_eq!(
            insurance
                .configuration
                .ship_insurance
                .unwrap()
                .conflict_type,
            ConflictType::Pvp
        );

        let manual: Project = serde_json::from_str(&base(r#"{"manual": {}}"#)).unwrap();
        assert!(manual.configuration.manual.is_some());
        assert!(manual.configuration.mine_material.is_none());

        let future: Project = serde_json::from_str(&base(r#"{"teleport_cat": {"x": 1}}"#)).unwrap();
        assert_eq!(future.configuration, ProjectConfiguration::default());
    }

    #[test]
    fn test_parse_contribution_and_contributors() {
        let c: ProjectContribution = serde_json::from_str(r#"{"contributed": 7}"#).unwrap();
        assert_eq!(c.last_modified, None);
        let page: Page<ProjectContributor> = serde_json::from_str(
            r#"{"contributors": [{"contributed": 3000000000, "id": 1, "name": "x"}]}"#,
        )
        .unwrap();
        assert_eq!(page.items[0].contributed, 3_000_000_000);
        assert!(page.cursor.is_none());
    }

    #[test]
    fn test_state_filter_display() {
        assert_eq!(ProjectStateFilter::Active.to_string(), "Active");
        assert_eq!(ProjectStateFilter::All.to_string(), "All");
    }
}
