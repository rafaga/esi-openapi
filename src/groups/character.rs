use crate::prelude::*;

/// Endpoints for Character
pub struct CharacterGroup<'a> {
    pub(crate) esi: &'a Esi,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct CharacterPublicInfo {
    pub achievement_score: Option<i64>,
    pub alliance_id: Option<i64>,
    pub birthday: String,
    pub bloodline_id: i64,
    pub character_title_id: Option<String>,
    pub corporation_id: i64,
    pub corporation_title: Option<String>,
    pub description: Option<String>,
    pub faction_id: Option<i64>,
    pub gender: String,
    pub name: String,
    pub race_id: i64,
    pub security_status: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct CharacterCorporationHistoryItem {
    pub corporation_id: i64,
    pub is_deleted: Option<bool>,
    pub record_id: i64,
    pub start_date: String,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct CharacterPortraitInfo {
    pub px128x128: Option<String>,
    pub px256x256: Option<String>,
    pub px512x512: Option<String>,
    pub px64x64: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct CharacterAffiliation {
    pub alliance_id: Option<i64>,
    pub character_id: i64,
    pub corporation_id: i64,
    pub faction_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct Notification {
    #[serde(default)]
    pub is_read: bool,
    pub notification_id: i64,
    pub sender_id: i64,
    pub sender_type: String,
    pub text: Option<String>,
    pub timestamp: String,
    #[serde(rename = "type")]
    pub notification_type: String,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct WalletTransaction {
    pub client_id: i64,
    pub date: String,
    pub is_buy: bool,
    pub is_personal: bool,
    pub journal_ref_id: i64,
    pub location_id: i64,
    pub quantity: i64,
    pub transaction_id: i64,
    pub type_id: i64,
    pub unit_price: f64,
}

impl CharacterGroup<'_> {
    api_get!(
        /// Get a character's public information.
        get_public_info,
        "GetCharactersDetail",
        RequestType::Public,
        CharacterPublicInfo,
        (character_id: i64) => "{character_id}"
    );

    api_get!(
        /// Get a character's corporation history.
        get_history,
        "GetCharactersCharacterIdCorporationhistory",
        RequestType::Public,
        Vec<CharacterCorporationHistoryItem>,
        (character_id: i64) => "{character_id}"
    );

    api_get!(
        /// Get a character's portrait URLs on the image server.
        get_portrait,
        "GetCharactersCharacterIdPortrait",
        RequestType::Public,
        CharacterPortraitInfo,
        (character_id: i64) => "{character_id}"
    );

    api_post!(
        /// Get character affiliations.
        get_affiliation,
        "PostCharactersAffiliation",
        RequestType::Public,
        Vec<CharacterAffiliation>,
        ; Chunked(character_ids: &[i64], 1000)
    );

    api_get!(
        /// Get character blueprints.
        get_blueprints,
        "GetCharactersCharacterIdBlueprints",
        RequestType::Authenticated,
        Vec<Blueprint>,
        (character_id: i64) => "{character_id}"
    );

    api_get!(
        /// Get character notifications.
        get_notifications,
        "GetCharactersCharacterIdNotifications",
        RequestType::Authenticated,
        Vec<Notification>,
        (character_id: i64) => "{character_id}"
    );

    api_get!(
        /// Get character wallet transactions.
        get_wallet_transactions,
        "GetCharactersCharacterIdWalletTransactions",
        RequestType::Authenticated,
        Vec<WalletTransaction>,
        (character_id: i64) => "{character_id}"
    );
}

/// A research agent a character works with.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct ResearchAgent {
    pub agent_id: i64,
    pub points_per_day: f64,
    pub remainder_points: f64,
    pub skill_type_id: i64,
    pub started_at: String,
}

/// Jump fatigue of a character.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct Fatigue {
    pub jump_fatigue_expire_date: Option<String>,
    pub last_jump_date: Option<String>,
    pub last_update_date: Option<String>,
}

/// Whether a medal is visible to everyone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MedalStatus {
    /// Visible to everyone.
    Public,
    /// Visible only to the owner.
    Private,
    /// A status this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// A layer of a medal's artwork.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct MedalGraphic {
    pub color: Option<i64>,
    pub graphic: String,
    pub layer: i64,
    pub part: i64,
}

/// A medal awarded to a character.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct Medal {
    pub corporation_id: i64,
    pub date: String,
    pub description: String,
    pub graphics: Vec<MedalGraphic>,
    /// Character ID of the issuer.
    pub issuer_id: i64,
    pub medal_id: i64,
    pub reason: String,
    pub status: MedalStatus,
    pub title: String,
}

/// A notification about a change in contact standings.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct ContactNotification {
    pub message: String,
    pub notification_id: i64,
    pub send_date: String,
    /// Character ID of the sender.
    pub sender_character_id: i64,
    pub standing_level: f64,
}

/// A corporation role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[allow(missing_docs)]
pub enum CorporationRole {
    #[serde(rename = "Account_Take_1")]
    AccountTake1,
    #[serde(rename = "Account_Take_2")]
    AccountTake2,
    #[serde(rename = "Account_Take_3")]
    AccountTake3,
    #[serde(rename = "Account_Take_4")]
    AccountTake4,
    #[serde(rename = "Account_Take_5")]
    AccountTake5,
    #[serde(rename = "Account_Take_6")]
    AccountTake6,
    #[serde(rename = "Account_Take_7")]
    AccountTake7,
    #[serde(rename = "Accountant")]
    Accountant,
    #[serde(rename = "Auditor")]
    Auditor,
    #[serde(rename = "Brand_Manager")]
    BrandManager,
    #[serde(rename = "Communications_Officer")]
    CommunicationsOfficer,
    #[serde(rename = "Config_Equipment")]
    ConfigEquipment,
    #[serde(rename = "Config_Starbase_Equipment")]
    ConfigStarbaseEquipment,
    #[serde(rename = "Container_Take_1")]
    ContainerTake1,
    #[serde(rename = "Container_Take_2")]
    ContainerTake2,
    #[serde(rename = "Container_Take_3")]
    ContainerTake3,
    #[serde(rename = "Container_Take_4")]
    ContainerTake4,
    #[serde(rename = "Container_Take_5")]
    ContainerTake5,
    #[serde(rename = "Container_Take_6")]
    ContainerTake6,
    #[serde(rename = "Container_Take_7")]
    ContainerTake7,
    #[serde(rename = "Contract_Manager")]
    ContractManager,
    #[serde(rename = "Deliveries_Container_Take")]
    DeliveriesContainerTake,
    #[serde(rename = "Deliveries_Query")]
    DeliveriesQuery,
    #[serde(rename = "Deliveries_Take")]
    DeliveriesTake,
    #[serde(rename = "Diplomat")]
    Diplomat,
    #[serde(rename = "Director")]
    Director,
    #[serde(rename = "Factory_Manager")]
    FactoryManager,
    #[serde(rename = "Fitting_Manager")]
    FittingManager,
    #[serde(rename = "Hangar_Query_1")]
    HangarQuery1,
    #[serde(rename = "Hangar_Query_2")]
    HangarQuery2,
    #[serde(rename = "Hangar_Query_3")]
    HangarQuery3,
    #[serde(rename = "Hangar_Query_4")]
    HangarQuery4,
    #[serde(rename = "Hangar_Query_5")]
    HangarQuery5,
    #[serde(rename = "Hangar_Query_6")]
    HangarQuery6,
    #[serde(rename = "Hangar_Query_7")]
    HangarQuery7,
    #[serde(rename = "Hangar_Take_1")]
    HangarTake1,
    #[serde(rename = "Hangar_Take_2")]
    HangarTake2,
    #[serde(rename = "Hangar_Take_3")]
    HangarTake3,
    #[serde(rename = "Hangar_Take_4")]
    HangarTake4,
    #[serde(rename = "Hangar_Take_5")]
    HangarTake5,
    #[serde(rename = "Hangar_Take_6")]
    HangarTake6,
    #[serde(rename = "Hangar_Take_7")]
    HangarTake7,
    #[serde(rename = "Junior_Accountant")]
    JuniorAccountant,
    #[serde(rename = "Personnel_Manager")]
    PersonnelManager,
    #[serde(rename = "Project_Manager")]
    ProjectManager,
    #[serde(rename = "Rent_Factory_Facility")]
    RentFactoryFacility,
    #[serde(rename = "Rent_Office")]
    RentOffice,
    #[serde(rename = "Rent_Research_Facility")]
    RentResearchFacility,
    #[serde(rename = "Security_Officer")]
    SecurityOfficer,
    #[serde(rename = "Skill_Plan_Manager")]
    SkillPlanManager,
    #[serde(rename = "Starbase_Defense_Operator")]
    StarbaseDefenseOperator,
    #[serde(rename = "Starbase_Fuel_Technician")]
    StarbaseFuelTechnician,
    #[serde(rename = "Station_Manager")]
    StationManager,
    #[serde(rename = "Trader")]
    Trader,
    /// A role this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// The corporation roles of a character.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct CharacterRoles {
    pub roles: Option<Vec<CorporationRole>>,
    pub roles_at_base: Option<Vec<CorporationRole>>,
    pub roles_at_hq: Option<Vec<CorporationRole>>,
    pub roles_at_other: Option<Vec<CorporationRole>>,
}

/// Where a standing comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StandingSource {
    /// An agent.
    Agent,
    /// An NPC corporation.
    NpcCorp,
    /// A faction.
    Faction,
    /// A source this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// A corporation title held by a character.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct CharacterTitle {
    pub name: Option<String>,
    pub title_id: Option<i64>,
}

impl CharacterGroup<'_> {
    api_get!(
        /// List the research agents of a character.
        get_agents_research,
        "GetCharactersCharacterIdAgentsResearch",
        RequestType::Authenticated,
        Vec<ResearchAgent>,
        (character_id: i64) => "{character_id}"
    );

    api_get!(
        /// Get a character's jump fatigue.
        get_fatigue,
        "GetCharactersCharacterIdFatigue",
        RequestType::Authenticated,
        Fatigue,
        (character_id: i64) => "{character_id}"
    );

    api_get!(
        /// List the medals of a character.
        get_medals,
        "GetCharactersCharacterIdMedals",
        RequestType::Authenticated,
        Vec<Medal>,
        (character_id: i64) => "{character_id}"
    );

    api_get!(
        /// List the contact standing change notifications of a character.
        get_contact_notifications,
        "GetCharactersCharacterIdNotificationsContacts",
        RequestType::Authenticated,
        Vec<ContactNotification>,
        (character_id: i64) => "{character_id}"
    );

    api_get!(
        /// Get the corporation roles of a character.
        get_roles,
        "GetCharactersCharacterIdRoles",
        RequestType::Authenticated,
        CharacterRoles,
        (character_id: i64) => "{character_id}"
    );

    api_get!(
        /// List the standings of a character with agents, NPC corporations and factions.
        get_standings,
        "GetCharactersCharacterIdStandings",
        RequestType::Authenticated,
        Vec<Standing>,
        (character_id: i64) => "{character_id}"
    );

    api_get!(
        /// List the corporation titles of a character.
        get_titles,
        "GetCharactersCharacterIdTitles",
        RequestType::Authenticated,
        Vec<CharacterTitle>,
        (character_id: i64) => "{character_id}"
    );

    api_post!(
        /// Calculate the CONCORD spam (CSPA) charge for contacting the given characters.
        /// Returns the charge in ISK.
        calculate_cspa_charge,
        "PostCharactersCharacterIdCspa",
        RequestType::Authenticated,
        f64,
        (character_id: i64) => "{character_id}",
        characters: &[i64],
    );
}

/// A blueprint owned by a character or a corporation.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Blueprint {
    /// ID of the item.
    pub item_id: i64,
    /// Where the blueprint is (see the ESI location flags).
    pub location_flag: String,
    /// ID of the location.
    pub location_id: i64,
    /// Material efficiency level.
    pub material_efficiency: i64,
    /// Quantity; `-1` for originals, `-2` for copies.
    pub quantity: i64,
    /// Remaining runs; `-1` for originals.
    pub runs: i64,
    /// Time efficiency level.
    pub time_efficiency: i64,
    /// Type ID of the blueprint.
    pub type_id: i64,
}

/// A standing a character or a corporation has with an agent, NPC corporation or faction.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Standing {
    /// ID of the entity the standing comes from.
    pub from_id: i64,
    /// Kind of that entity.
    pub from_type: StandingSource,
    /// Standing, from -10.0 to 10.0.
    pub standing: f64,
}

#[cfg(test)]
mod character_basic_tests {
    use super::{CharacterRoles, CorporationRole, Medal, MedalStatus, Standing, StandingSource};

    #[test]
    fn test_parse_roles_with_unknown_role() {
        let json =
            r#"{"roles": ["Director", "Account_Take_1", "Brand_New_Role"], "roles_at_hq": []}"#;
        let roles: CharacterRoles = serde_json::from_str(json).unwrap();
        let list = roles.roles.unwrap();
        assert_eq!(list[0], CorporationRole::Director);
        assert_eq!(list[1], CorporationRole::AccountTake1);
        assert_eq!(list[2], CorporationRole::Unrecognized);
        assert!(roles.roles_at_base.is_none());
    }

    #[test]
    fn test_parse_standing() {
        let standing: Standing = serde_json::from_str(
            r#"{"from_id": 500001, "from_type": "npc_corp", "standing": 1.5}"#,
        )
        .unwrap();
        assert_eq!(standing.from_type, StandingSource::NpcCorp);
    }

    #[test]
    fn test_parse_medal() {
        let json = r#"{"corporation_id": 1, "date": "2026-01-01T00:00:00Z", "description": "d",
            "graphics": [{"graphic": "g", "layer": 0, "part": 1}], "issuer_id": 2112625428,
            "medal_id": 7, "reason": "r", "status": "private", "title": "t"}"#;
        let medal: Medal = serde_json::from_str(json).unwrap();
        assert_eq!(medal.status, MedalStatus::Private);
        assert!(medal.graphics[0].color.is_none());
    }
}
