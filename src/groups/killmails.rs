use crate::groups::Position;
use crate::prelude::*;

/// Endpoints for Killmails
pub struct KillmailsGroup<'a> {
    pub(crate) esi: &'a Esi,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct RecentKillMail {
    pub killmail_hash: String,
    pub killmail_id: i64,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct KillmailAttacker {
    pub alliance_id: Option<i64>,
    pub character_id: Option<i64>,
    pub corporation_id: Option<i64>,
    pub damage_done: i64,
    pub faction_id: Option<i64>,
    pub final_blow: bool,
    pub security_status: f64,
    pub ship_type_id: Option<i64>,
    pub weapon_type_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct KillmailItem {
    pub flag: i64,
    pub item_type_id: i64,
    pub quantity_destroyed: Option<i64>,
    pub quantity_dropped: Option<i64>,
    pub singleton: i64,
    /// Items inside this item (e.g. the contents of a container).
    pub items: Option<Vec<KillmailItem>>,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct KillmailVictim {
    pub alliance_id: Option<i64>,
    pub character_id: Option<i64>,
    pub corporation_id: Option<i64>,
    pub damage_taken: i64,
    pub faction_id: Option<i64>,
    pub items: Option<Vec<KillmailItem>>,
    pub position: Option<Position>,
    pub ship_type_id: i64,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct Killmail {
    pub killmail_id: i64,
    pub killmail_time: String,
    pub solar_system_id: i64,
    pub moon_id: Option<i64>,
    pub war_id: Option<i64>,
    pub attackers: Vec<KillmailAttacker>,
    pub victim: KillmailVictim,
}

impl KillmailsGroup<'_> {
    // NOTE unknown type; I haven't played in a long time
    api_get!(
        /// Get a character's recent kills & losses.
        get_character_recent,
        "GetCharactersCharacterIdKillmailsRecent",
        RequestType::Authenticated,
        Vec<RecentKillMail>,
        (character_id: i64) => "{character_id}"
    );

    api_get!(
        /// Get a killmail.
        get_killmail,
        "GetKillmailsKillmailIdKillmailHash",
        RequestType::Public,
        Killmail,
        (killmail_id: i64) => "{killmail_id}",
        (killmail_hash: &str) => "{killmail_hash}"
    );

    // more endpoints ...
}

impl KillmailsGroup<'_> {
    api_get!(
        /// Get a corporation's recent kills & losses.
        get_corporation_recent,
        "GetCorporationsCorporationIdKillmailsRecent",
        RequestType::Authenticated,
        Vec<RecentKillMail>,
        (corporation_id: i64) => "{corporation_id}";
        Optional(page: i32) => "page"
    );
}
