#![allow(unused)]

use crate::prelude::*;

/// Endpoints for Fittings
pub struct FittingsGroup<'a> {
    pub(crate) esi: &'a Esi,
}

#[allow(missing_docs)]
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
/// The slot or bay an item occupies in a fitting. Variant names match the
/// spec values exactly.
pub enum FittingFlag {
    Cargo,
    DroneBay,
    FighterBay,
    HiSlot0,
    HiSlot1,
    HiSlot2,
    HiSlot3,
    HiSlot4,
    HiSlot5,
    HiSlot6,
    HiSlot7,
    Invalid,
    LoSlot0,
    LoSlot1,
    LoSlot2,
    LoSlot3,
    LoSlot4,
    LoSlot5,
    LoSlot6,
    LoSlot7,
    MedSlot0,
    MedSlot1,
    MedSlot2,
    MedSlot3,
    MedSlot4,
    MedSlot5,
    MedSlot6,
    MedSlot7,
    RigSlot0,
    RigSlot1,
    RigSlot2,
    ServiceSlot0,
    ServiceSlot1,
    ServiceSlot2,
    ServiceSlot3,
    ServiceSlot4,
    ServiceSlot5,
    ServiceSlot6,
    ServiceSlot7,
    SubSystemSlot0,
    SubSystemSlot1,
    SubSystemSlot2,
    SubSystemSlot3,
    /// A value not known to this version of the crate.
    #[serde(other)]
    Unrecognized,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
/// An item in a fitting.
pub struct FittingItem {
    /// Slot or bay of the item.
    pub flag: FittingFlag,
    /// Quantity of the item.
    pub quantity: i64,
    /// Type ID of the item.
    pub type_id: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
/// A saved ship fitting.
pub struct Fitting {
    /// Description of the fitting.
    pub description: String,
    /// ID of the fitting.
    pub fitting_id: i64,
    /// Items of the fitting.
    pub items: Vec<FittingItem>,
    /// Name of the fitting.
    pub name: String,
    /// Type ID of the ship.
    pub ship_type_id: i64,
}

#[derive(Debug, Serialize, Clone)]
/// The body of a request to save a new fitting.
pub struct NewFitting {
    /// Description of the fitting.
    pub description: String,
    /// Items of the fitting.
    pub items: Vec<FittingItem>,
    /// Name of the fitting.
    pub name: String,
    /// Type ID of the ship.
    pub ship_type_id: i64,
}

impl FittingsGroup<'_> {
    api_get!(
        /// Return the fittings saved by a character.
        get_character_fittings,
        "GetCharactersCharacterIdFittings",
        RequestType::Authenticated,
        Vec<Fitting>,
        (character_id: i64) => "{character_id}"
    );

    api_post!(
        /// Save a new fitting. Returns the ID of the created fitting.
        create_character_fitting,
        "PostCharactersCharacterIdFittings",
        RequestType::Authenticated,
        i64,
        (character_id: i64) => "{character_id}",
        fitting: &NewFitting,
    );

    api_delete!(
        /// Delete a saved fitting.
        delete_character_fitting,
        "DeleteCharactersCharacterIdFittingsFittingId",
        RequestType::Authenticated,
        (),
        (character_id: i64) => "{character_id}",
        (fitting_id: i64) => "{fitting_id}"
    );
}

#[cfg(test)]
mod fittings_tests {
    use super::*;

    #[test]
    fn test_parse_fitting() {
        let json = r#"[{"description": "d", "fitting_id": 3000000000, "name": "n", "ship_type_id": 587,
            "items": [{"flag": "HiSlot0", "quantity": 1, "type_id": 3001},
                      {"flag": "NewSlot", "quantity": 2, "type_id": 1}]}]"#;
        let fits: Vec<Fitting> = serde_json::from_str(json).unwrap();
        assert_eq!(fits[0].fitting_id, 3_000_000_000);
        assert_eq!(fits[0].items[0].flag, FittingFlag::HiSlot0);
        assert_eq!(fits[0].items[1].flag, FittingFlag::Unrecognized);
    }

    #[test]
    fn test_serialize_new_fitting() {
        let f = NewFitting {
            description: "d".into(),
            items: vec![FittingItem {
                flag: FittingFlag::SubSystemSlot3,
                quantity: 1,
                type_id: 5,
            }],
            name: "n".into(),
            ship_type_id: 1,
        };
        let v = serde_json::to_value(&f).unwrap();
        assert_eq!(v["items"][0]["flag"], "SubSystemSlot3");
    }
}
