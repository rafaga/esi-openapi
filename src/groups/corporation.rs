use crate::groups::{Blueprint, CorporationRole, MedalStatus, Standing};
use crate::prelude::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct CorporationPublicInfo {
    pub alliance_id: Option<i64>,
    pub ceo_id: Option<i64>,
    pub creator_id: Option<i64>,
    pub date_founded: Option<String>,
    pub description: Option<String>,
    pub enlisted_faction_id: Option<i64>,
    /// `legal` or `illegal`.
    pub friendly_fire: Option<String>,
    pub home_station_id: Option<i64>,
    pub member_count: i64,
    pub name: String,
    pub palette: Option<CorporationPalette>,
    pub shares: Option<i64>,
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
    pub alliance_id: Option<i64>,
    pub is_deleted: Option<bool>,
    pub record_id: i64,
    pub start_date: String,
}

/// Icon URLs of a corporation.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CorporationIcons {
    /// 128x128 icon.
    pub px128x128: Option<String>,
    /// 256x256 icon.
    pub px256x256: Option<String>,
    /// 64x64 icon.
    pub px64x64: Option<String>,
}

/// The action recorded in a container log entry.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContainerAction {
    /// Item added.
    Add,
    /// Container assembled.
    Assemble,
    /// Container configured.
    Configure,
    /// Password entered.
    EnterPassword,
    /// Container locked.
    Lock,
    /// Item moved.
    Move,
    /// Container repackaged.
    Repackage,
    /// Container renamed.
    SetName,
    /// Password set.
    SetPassword,
    /// Container unlocked.
    Unlock,
    /// A value this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// The kind of password involved in a container log entry.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContainerPasswordType {
    /// Configuration password.
    Config,
    /// General password.
    General,
    /// A value this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// An entry of the corporation container log.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ContainerLogEntry {
    /// What happened.
    pub action: ContainerAction,
    /// Character that did it.
    pub character_id: i64,
    /// ID of the container.
    pub container_id: i64,
    /// Type ID of the container.
    pub container_type_id: i64,
    /// Where the container is (see the ESI location flags).
    pub location_flag: String,
    /// ID of the location.
    pub location_id: i64,
    /// When it happened.
    pub logged_at: String,
    /// Configuration bitmask after the change.
    pub new_config_bitmask: Option<i64>,
    /// Configuration bitmask before the change.
    pub old_config_bitmask: Option<i64>,
    /// Kind of password involved.
    pub password_type: ContainerPasswordType,
    /// Quantity moved.
    pub quantity: Option<i64>,
    /// Type ID of the item moved.
    pub type_id: Option<i64>,
}

/// A hangar or wallet division.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Division {
    /// Number of the division.
    pub division: Option<i64>,
    /// Name of the division.
    pub name: Option<String>,
}

/// The hangar and wallet divisions of a corporation.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct CorporationDivisions {
    /// Hangar divisions.
    #[serde(default)]
    pub hangar: Vec<Division>,
    /// Wallet divisions.
    #[serde(default)]
    pub wallet: Vec<Division>,
}

/// An industry facility of a corporation.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct CorporationFacility {
    /// ID of the facility.
    pub facility_id: i64,
    /// Solar system of the facility.
    pub system_id: i64,
    /// Type ID of the facility.
    pub type_id: i64,
}

/// A medal created by a corporation.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct CorporationMedal {
    /// When the medal was created.
    pub created_at: String,
    /// Character that created the medal.
    pub creator_id: i64,
    /// Description of the medal.
    pub description: String,
    /// ID of the medal.
    pub medal_id: i64,
    /// Title of the medal.
    pub title: String,
}

/// A medal issued to a character.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct IssuedMedal {
    /// Character the medal was issued to.
    pub character_id: i64,
    /// When the medal was issued.
    pub issued_at: String,
    /// Character that issued the medal.
    pub issuer_id: i64,
    /// ID of the medal.
    pub medal_id: i64,
    /// Reason for the medal.
    pub reason: String,
    /// Visibility of the medal.
    pub status: MedalStatus,
}

/// The titles of a corporation member.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct MemberTitles {
    /// Character ID of the member.
    pub character_id: i64,
    /// Title IDs held by the member.
    pub titles: Vec<i64>,
}

/// Tracking information about a corporation member.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct MemberTracking {
    /// Base (home station) of the member.
    pub base_id: Option<i64>,
    /// Character ID of the member.
    pub character_id: i64,
    /// Current location of the member.
    pub location_id: Option<i64>,
    /// Last logoff.
    pub logoff_date: Option<String>,
    /// Last logon.
    pub logon_date: Option<String>,
    /// Type ID of the ship the member flies.
    pub ship_type_id: Option<i64>,
    /// When the member joined.
    pub start_date: Option<String>,
}

/// The roles of a corporation member.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct MemberRoles {
    /// Character ID of the member.
    pub character_id: i64,
    /// Roles the member can grant.
    #[serde(default)]
    pub grantable_roles: Vec<CorporationRole>,
    /// Roles the member can grant at the base.
    #[serde(default)]
    pub grantable_roles_at_base: Vec<CorporationRole>,
    /// Roles the member can grant at the headquarters.
    #[serde(default)]
    pub grantable_roles_at_hq: Vec<CorporationRole>,
    /// Roles the member can grant at other locations.
    #[serde(default)]
    pub grantable_roles_at_other: Vec<CorporationRole>,
    /// Roles of the member.
    #[serde(default)]
    pub roles: Vec<CorporationRole>,
    /// Roles of the member at the base.
    #[serde(default)]
    pub roles_at_base: Vec<CorporationRole>,
    /// Roles of the member at the headquarters.
    #[serde(default)]
    pub roles_at_hq: Vec<CorporationRole>,
    /// Roles of the member at other locations.
    #[serde(default)]
    pub roles_at_other: Vec<CorporationRole>,
}

/// Which set of roles a role history entry changed.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoleType {
    /// Grantable roles.
    GrantableRoles,
    /// Grantable roles at the base.
    GrantableRolesAtBase,
    /// Grantable roles at the headquarters.
    GrantableRolesAtHq,
    /// Grantable roles at other locations.
    GrantableRolesAtOther,
    /// Roles.
    Roles,
    /// Roles at the base.
    RolesAtBase,
    /// Roles at the headquarters.
    RolesAtHq,
    /// Roles at other locations.
    RolesAtOther,
    /// A value this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// An entry of the corporation roles history.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct RoleHistoryEntry {
    /// When the change happened.
    pub changed_at: String,
    /// Character whose roles changed.
    pub character_id: i64,
    /// Character that made the change.
    pub issuer_id: i64,
    /// Roles after the change.
    pub new_roles: Vec<CorporationRole>,
    /// Roles before the change.
    pub old_roles: Vec<CorporationRole>,
    /// Which set of roles changed.
    pub role_type: RoleType,
}

/// The kind of entity that holds shares.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShareholderType {
    /// A character.
    Character,
    /// A corporation.
    Corporation,
    /// A value this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// A shareholder of a corporation.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Shareholder {
    /// Number of shares held.
    pub share_count: i64,
    /// ID of the shareholder.
    pub shareholder_id: i64,
    /// Kind of the shareholder.
    pub shareholder_type: ShareholderType,
}

/// The state of a starbase.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StarbaseState {
    /// Offline.
    Offline,
    /// Online.
    Online,
    /// Onlining.
    Onlining,
    /// Reinforced.
    Reinforced,
    /// Unanchoring.
    Unanchoring,
    /// A value this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// A starbase (POS) of a corporation.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Starbase {
    /// Moon the starbase is anchored at.
    pub moon_id: Option<i64>,
    /// When the starbase went online.
    pub onlined_since: Option<String>,
    /// Until when the starbase is reinforced.
    pub reinforced_until: Option<String>,
    /// ID of the starbase.
    pub starbase_id: i64,
    /// State of the starbase.
    pub state: Option<StarbaseState>,
    /// Solar system of the starbase.
    pub system_id: i64,
    /// Type ID of the starbase.
    pub type_id: i64,
    /// When the starbase unanchors.
    pub unanchor_at: Option<String>,
}

/// Who is allowed to perform an action on a starbase.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StarbasePermission {
    /// Alliance members.
    AllianceMember,
    /// Members with the Config Starbase Equipment role.
    ConfigStarbaseEquipmentRole,
    /// Corporation members.
    CorporationMember,
    /// Members with the Starbase Fuel Technician role.
    StarbaseFuelTechnicianRole,
    /// A value this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// Fuel in a starbase.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct StarbaseFuel {
    /// Quantity of fuel.
    pub quantity: i64,
    /// Type ID of the fuel.
    pub type_id: i64,
}

/// The settings of a starbase.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct StarbaseDetails {
    /// Whether alliance members may use the starbase.
    pub allow_alliance_members: bool,
    /// Whether corporation members may use the starbase.
    pub allow_corporation_members: bool,
    /// Who can anchor.
    pub anchor: StarbasePermission,
    /// Whether the starbase attacks entities at war with it.
    pub attack_if_at_war: bool,
    /// Whether the starbase attacks entities with dropping security status.
    pub attack_if_other_security_status_dropping: bool,
    /// Security status threshold for attacks.
    pub attack_security_status_threshold: Option<f64>,
    /// Standing threshold for attacks.
    pub attack_standing_threshold: Option<f64>,
    /// Who can take from the fuel bay.
    pub fuel_bay_take: StarbasePermission,
    /// Who can view the fuel bay.
    pub fuel_bay_view: StarbasePermission,
    /// Fuel in the starbase.
    #[serde(default)]
    pub fuels: Vec<StarbaseFuel>,
    /// Who can take it offline.
    pub offline: StarbasePermission,
    /// Who can bring it online.
    pub online: StarbasePermission,
    /// Who can unanchor.
    pub unanchor: StarbasePermission,
    /// Whether alliance standings are used.
    pub use_alliance_standings: bool,
}

/// The state of a structure service.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceState {
    /// Online.
    Online,
    /// Offline.
    Offline,
    /// Cleanup.
    Cleanup,
    /// A value this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// A service installed in a structure.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct StructureService {
    /// Name of the service.
    pub name: String,
    /// State of the service.
    pub state: ServiceState,
}

/// The state of a structure.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StructureState {
    /// Anchor vulnerable.
    AnchorVulnerable,
    /// Anchoring.
    Anchoring,
    /// Armor reinforce.
    ArmorReinforce,
    /// Armor vulnerable.
    ArmorVulnerable,
    /// Deploy vulnerable.
    DeployVulnerable,
    /// Fitting invulnerable.
    FittingInvulnerable,
    /// Hull reinforce.
    HullReinforce,
    /// Hull vulnerable.
    HullVulnerable,
    /// Online (deprecated).
    OnlineDeprecated,
    /// Onlining vulnerable.
    OnliningVulnerable,
    /// Shield vulnerable.
    ShieldVulnerable,
    /// Unanchored.
    Unanchored,
    /// Unknown.
    Unknown,
    /// A value this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// A structure owned by a corporation.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct CorporationStructure {
    /// Corporation that owns the structure.
    pub corporation_id: i64,
    /// When the fuel runs out.
    pub fuel_expires: Option<String>,
    /// Name of the structure.
    pub name: Option<String>,
    /// When the next reinforcement hour applies.
    pub next_reinforce_apply: Option<String>,
    /// Next reinforcement hour.
    pub next_reinforce_hour: Option<i64>,
    /// ID of the structure's vulnerability profile.
    pub profile_id: i64,
    /// Current reinforcement hour.
    pub reinforce_hour: Option<i64>,
    /// Services of the structure.
    #[serde(default)]
    pub services: Vec<StructureService>,
    /// State of the structure.
    pub state: StructureState,
    /// When the current state timer ends.
    pub state_timer_end: Option<String>,
    /// When the current state timer started.
    pub state_timer_start: Option<String>,
    /// ID of the structure.
    pub structure_id: i64,
    /// Solar system of the structure.
    pub system_id: i64,
    /// Type ID of the structure.
    pub type_id: i64,
    /// When the structure unanchors.
    pub unanchors_at: Option<String>,
}

/// A corporation title.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct CorporationTitle {
    /// Roles the title can grant.
    #[serde(default)]
    pub grantable_roles: Vec<CorporationRole>,
    /// Roles the title can grant at the base.
    #[serde(default)]
    pub grantable_roles_at_base: Vec<CorporationRole>,
    /// Roles the title can grant at the headquarters.
    #[serde(default)]
    pub grantable_roles_at_hq: Vec<CorporationRole>,
    /// Roles the title can grant at other locations.
    #[serde(default)]
    pub grantable_roles_at_other: Vec<CorporationRole>,
    /// Name of the title.
    pub name: Option<String>,
    /// Roles of the title.
    #[serde(default)]
    pub roles: Vec<CorporationRole>,
    /// Roles of the title at the base.
    #[serde(default)]
    pub roles_at_base: Vec<CorporationRole>,
    /// Roles of the title at the headquarters.
    #[serde(default)]
    pub roles_at_hq: Vec<CorporationRole>,
    /// Roles of the title at other locations.
    #[serde(default)]
    pub roles_at_other: Vec<CorporationRole>,
    /// ID of the title.
    pub title_id: Option<i64>,
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
        (corporation_id: i64) => "{corporation_id}"
    );

    api_get!(
        /// Get a corporation's alliance history.
        get_history,
        "GetCorporationsCorporationIdAlliancehistory",
        RequestType::Public,
        Vec<CorporationHistoryItem>,
        (corporation_id: i64) => "{corporation_id}"
    );

    api_get!(
        /// Get a corporation's member list.
        ///
        /// Requires the auth'd character to be in the corporation.
        get_members,
        "GetCorporationsCorporationIdMembers",
        RequestType::Authenticated,
        Vec<i64>,
        (corporation_id: i64) => "{corporation_id}"
    );

    api_get!(
        /// Get a list of NPC corporations.
        get_npc_corps,
        "GetCorporationsNpccorps",
        RequestType::Public,
        Vec<i64>,
    );

    api_get!(
        /// Get the blueprints of a corporation.
        ///
        /// Requires the Director role.
        get_blueprints,
        "GetCorporationsCorporationIdBlueprints",
        RequestType::Authenticated,
        Vec<Blueprint>,
        (corporation_id: i64) => "{corporation_id}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// Get the container logs of a corporation.
        ///
        /// Requires the Director role.
        get_container_logs,
        "GetCorporationsCorporationIdContainersLogs",
        RequestType::Authenticated,
        Vec<ContainerLogEntry>,
        (corporation_id: i64) => "{corporation_id}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// Get the hangar and wallet divisions of a corporation.
        ///
        /// Requires the Director role.
        get_divisions,
        "GetCorporationsCorporationIdDivisions",
        RequestType::Authenticated,
        CorporationDivisions,
        (corporation_id: i64) => "{corporation_id}"
    );

    api_get!(
        /// Get the industry facilities of a corporation.
        ///
        /// Requires the Factory Manager role.
        get_facilities,
        "GetCorporationsCorporationIdFacilities",
        RequestType::Authenticated,
        Vec<CorporationFacility>,
        (corporation_id: i64) => "{corporation_id}"
    );

    api_get!(
        /// Get the icons of a corporation.
        get_icons,
        "GetCorporationsCorporationIdIcons",
        RequestType::Public,
        CorporationIcons,
        (corporation_id: i64) => "{corporation_id}"
    );

    api_get!(
        /// Get the medals created by a corporation.
        get_medals,
        "GetCorporationsCorporationIdMedals",
        RequestType::Authenticated,
        Vec<CorporationMedal>,
        (corporation_id: i64) => "{corporation_id}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// Get the medals issued by a corporation.
        ///
        /// Requires the Director role.
        get_issued_medals,
        "GetCorporationsCorporationIdMedalsIssued",
        RequestType::Authenticated,
        Vec<IssuedMedal>,
        (corporation_id: i64) => "{corporation_id}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// Get the member limit of a corporation.
        ///
        /// Requires the Director role.
        get_members_limit,
        "GetCorporationsCorporationIdMembersLimit",
        RequestType::Authenticated,
        i64,
        (corporation_id: i64) => "{corporation_id}"
    );

    api_get!(
        /// Get the titles of the members of a corporation.
        ///
        /// Requires the Director role.
        get_member_titles,
        "GetCorporationsCorporationIdMembersTitles",
        RequestType::Authenticated,
        Vec<MemberTitles>,
        (corporation_id: i64) => "{corporation_id}"
    );

    api_get!(
        /// Track the members of a corporation.
        ///
        /// Requires the Director role.
        get_member_tracking,
        "GetCorporationsCorporationIdMembertracking",
        RequestType::Authenticated,
        Vec<MemberTracking>,
        (corporation_id: i64) => "{corporation_id}"
    );

    api_get!(
        /// Get the roles of the members of a corporation.
        get_roles,
        "GetCorporationsCorporationIdRoles",
        RequestType::Authenticated,
        Vec<MemberRoles>,
        (corporation_id: i64) => "{corporation_id}"
    );

    api_get!(
        /// Get the role changes of a corporation in the last 30 days.
        ///
        /// Requires the Director role.
        get_roles_history,
        "GetCorporationsCorporationIdRolesHistory",
        RequestType::Authenticated,
        Vec<RoleHistoryEntry>,
        (corporation_id: i64) => "{corporation_id}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// Get the shareholders of a corporation.
        ///
        /// Requires the Director role.
        get_shareholders,
        "GetCorporationsCorporationIdShareholders",
        RequestType::Authenticated,
        Vec<Shareholder>,
        (corporation_id: i64) => "{corporation_id}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// Get the standings of a corporation.
        get_standings,
        "GetCorporationsCorporationIdStandings",
        RequestType::Authenticated,
        Vec<Standing>,
        (corporation_id: i64) => "{corporation_id}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// Get the starbases of a corporation.
        ///
        /// Requires the Director role.
        get_starbases,
        "GetCorporationsCorporationIdStarbases",
        RequestType::Authenticated,
        Vec<Starbase>,
        (corporation_id: i64) => "{corporation_id}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// Get the settings of a starbase. `system_id` is the solar system
        /// the starbase is in.
        ///
        /// Requires the Director role.
        get_starbase,
        "GetCorporationsCorporationIdStarbasesStarbaseId",
        RequestType::Authenticated,
        StarbaseDetails,
        (corporation_id: i64) => "{corporation_id}",
        (starbase_id: i64) => "{starbase_id}";
        Required(system_id: i64) => "system_id"
    );

    api_get!(
        /// Get the structures of a corporation.
        ///
        /// Requires the Station Manager role.
        get_structures,
        "GetCorporationsCorporationIdStructures",
        RequestType::Authenticated,
        Vec<CorporationStructure>,
        (corporation_id: i64) => "{corporation_id}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// Get the titles of a corporation.
        ///
        /// Requires the Director role.
        get_titles,
        "GetCorporationsCorporationIdTitles",
        RequestType::Authenticated,
        Vec<CorporationTitle>,
        (corporation_id: i64) => "{corporation_id}"
    );
}

#[cfg(test)]
mod corporation_management_tests {
    use super::*;

    #[test]
    fn test_parse_roles_and_history() {
        let r: Vec<MemberRoles> = serde_json::from_str(
            r#"[{"character_id": 2124758468, "roles": ["Director", "Brand_New_Role"],
                 "roles_at_hq": ["Trader"]}]"#,
        )
        .unwrap();
        assert_eq!(r[0].roles[0], CorporationRole::Director);
        assert_eq!(r[0].roles[1], CorporationRole::Unrecognized);
        assert!(r[0].grantable_roles.is_empty());
        let h: Vec<RoleHistoryEntry> = serde_json::from_str(
            r#"[{"changed_at": "t", "character_id": 1, "issuer_id": 2, "new_roles": ["Trader"],
                 "old_roles": [], "role_type": "roles_at_hq"}]"#,
        )
        .unwrap();
        assert_eq!(h[0].role_type, RoleType::RolesAtHq);
    }

    #[test]
    fn test_parse_structures_and_starbases() {
        let s: Vec<CorporationStructure> = serde_json::from_str(
            r#"[{"corporation_id": 98000001, "profile_id": 1, "state": "shield_vulnerable",
                 "structure_id": 1040000000000, "system_id": 30000142, "type_id": 35832,
                 "services": [{"name": "Market", "state": "online"}]},
                {"corporation_id": 98000001, "profile_id": 1, "state": "something_new",
                 "structure_id": 1, "system_id": 1, "type_id": 1}]"#,
        )
        .unwrap();
        assert_eq!(s[0].structure_id, 1_040_000_000_000);
        assert_eq!(s[0].services[0].state, ServiceState::Online);
        assert_eq!(s[1].state, StructureState::Unrecognized);
        let d: StarbaseDetails = serde_json::from_str(
            r#"{"allow_alliance_members": true, "allow_corporation_members": true,
                "anchor": "alliance_member", "attack_if_at_war": false,
                "attack_if_other_security_status_dropping": false,
                "fuel_bay_take": "corporation_member", "fuel_bay_view": "corporation_member",
                "offline": "starbase_fuel_technician_role", "online": "x", "unanchor": "alliance_member",
                "use_alliance_standings": false, "fuels": [{"quantity": 4000000000, "type_id": 4247}]}"#,
        )
        .unwrap();
        assert_eq!(d.offline, StarbasePermission::StarbaseFuelTechnicianRole);
        assert_eq!(d.online, StarbasePermission::Unrecognized);
        assert_eq!(d.fuels[0].quantity, 4_000_000_000);
    }

    #[test]
    fn test_parse_misc() {
        let d: CorporationDivisions =
            serde_json::from_str(r#"{"hangar": [{"division": 1, "name": "Main"}]}"#).unwrap();
        assert_eq!(d.hangar[0].division, Some(1));
        assert!(d.wallet.is_empty());
        let l: Vec<ContainerLogEntry> = serde_json::from_str(
            r#"[{"action": "enter_password", "character_id": 1, "container_id": 1000000000000,
                 "container_type_id": 17366, "location_flag": "CorpSAG1", "location_id": 2,
                 "logged_at": "t", "password_type": "config"}]"#,
        )
        .unwrap();
        assert_eq!(l[0].action, ContainerAction::EnterPassword);
        let t: Vec<CorporationTitle> =
            serde_json::from_str(r#"[{"title_id": 1, "name": "CEO", "roles": ["Director"]}]"#)
                .unwrap();
        assert_eq!(t[0].roles, vec![CorporationRole::Director]);
        let sh: Vec<Shareholder> = serde_json::from_str(
            r#"[{"share_count": 5000000000, "shareholder_id": 1, "shareholder_type": "character"}]"#,
        )
        .unwrap();
        assert_eq!(sh[0].share_count, 5_000_000_000);
    }
}
