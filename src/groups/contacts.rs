#![allow(unused)]

use crate::prelude::*;

/// Endpoints for Contacts
pub struct ContactsGroup<'a> {
    pub(crate) esi: &'a Esi,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
/// The kind of entity a contact is.
pub enum ContactType {
    /// A character.
    #[serde(rename = "character")]
    Character,
    /// A corporation.
    #[serde(rename = "corporation")]
    Corporation,
    /// An alliance.
    #[serde(rename = "alliance")]
    Alliance,
    /// A faction.
    #[serde(rename = "faction")]
    Faction,
    /// A value not known to this version of the crate.
    #[serde(other)]
    Unrecognized,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
/// A contact of a character, corporation or alliance.
pub struct Contact {
    /// ID of the contact.
    pub contact_id: i64,
    /// Kind of the contact.
    pub contact_type: ContactType,
    /// Whether the contact is blocked (character contacts only).
    #[serde(default)]
    pub is_blocked: Option<bool>,
    /// Whether the contact is watched (character and corporation contacts only).
    #[serde(default)]
    pub is_watched: Option<bool>,
    /// IDs of the labels attached to the contact.
    #[serde(default)]
    pub label_ids: Vec<i64>,
    /// Standing of the contact, from -10.0 to 10.0.
    pub standing: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
/// A contact label.
pub struct ContactLabel {
    /// ID of the label.
    pub label_id: i64,
    /// Name of the label.
    pub label_name: String,
}

impl ContactsGroup<'_> {
    api_get!(
        /// Return the contacts of an alliance.
        get_alliance_contacts,
        "GetAlliancesAllianceIdContacts",
        RequestType::Authenticated,
        Vec<Contact>,
        (alliance_id: i64) => "{alliance_id}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// Return the contact labels of an alliance.
        get_alliance_contact_labels,
        "GetAlliancesAllianceIdContactsLabels",
        RequestType::Authenticated,
        Vec<ContactLabel>,
        (alliance_id: i64) => "{alliance_id}"
    );

    api_delete!(
        /// Delete contacts of a character.
        delete_character_contacts,
        "DeleteCharactersCharacterIdContacts",
        RequestType::Authenticated,
        (),
        (character_id: i64) => "{character_id}";
        Many(contact_ids: &[i64]) => "contact_ids"
    );

    api_get!(
        /// Return the contacts of a character.
        get_character_contacts,
        "GetCharactersCharacterIdContacts",
        RequestType::Authenticated,
        Vec<Contact>,
        (character_id: i64) => "{character_id}";
        Optional(page: i32) => "page"
    );

    api_post!(
        /// Add contacts to a character. Returns the IDs of the created contacts.
        add_character_contacts,
        "PostCharactersCharacterIdContacts",
        RequestType::Authenticated,
        Vec<i64>,
        (character_id: i64) => "{character_id}";
        Required(standing: f64) => "standing",
        OptionalMany(label_ids: &[i64]) => "label_ids",
        Optional(watched: bool) => "watched";
        contact_ids: &[i64]
    );

    api_put!(
        /// Edit contacts of a character.
        update_character_contacts,
        "PutCharactersCharacterIdContacts",
        RequestType::Authenticated,
        (),
        (character_id: i64) => "{character_id}";
        Required(standing: f64) => "standing",
        OptionalMany(label_ids: &[i64]) => "label_ids",
        Optional(watched: bool) => "watched";
        contact_ids: &[i64]
    );

    api_get!(
        /// Return the contact labels of a character.
        get_character_contact_labels,
        "GetCharactersCharacterIdContactsLabels",
        RequestType::Authenticated,
        Vec<ContactLabel>,
        (character_id: i64) => "{character_id}"
    );

    api_get!(
        /// Return the contacts of a corporation.
        get_corporation_contacts,
        "GetCorporationsCorporationIdContacts",
        RequestType::Authenticated,
        Vec<Contact>,
        (corporation_id: i64) => "{corporation_id}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// Return the contact labels of a corporation.
        get_corporation_contact_labels,
        "GetCorporationsCorporationIdContactsLabels",
        RequestType::Authenticated,
        Vec<ContactLabel>,
        (corporation_id: i64) => "{corporation_id}"
    );
}

#[cfg(test)]
mod contacts_tests {
    use super::*;

    #[test]
    fn test_parse_contacts() {
        let json = r#"[{"contact_id": 2112625428, "contact_type": "character", "standing": 10.0,
                        "is_blocked": false, "label_ids": [1]},
                       {"contact_id": 1, "contact_type": "other", "standing": -5.5}]"#;
        let contacts: Vec<Contact> = serde_json::from_str(json).unwrap();
        assert_eq!(contacts[0].contact_type, ContactType::Character);
        assert_eq!(contacts[1].contact_type, ContactType::Unrecognized);
        assert!(contacts[1].label_ids.is_empty());
    }

    #[test]
    fn test_parse_labels() {
        let l: Vec<ContactLabel> =
            serde_json::from_str(r#"[{"label_id": 5000000000, "label_name": "Friends"}]"#).unwrap();
        assert_eq!(l[0].label_id, 5_000_000_000);
    }
}
