use crate::groups::Position;
use crate::prelude::*;

/// Endpoints for Assets
pub struct AssetsGroup<'a> {
    pub(crate) esi: &'a Esi,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct Asset {
    pub is_blueprint_copy: Option<bool>,
    pub is_singleton: bool,
    pub item_id: i64,
    pub location_flag: String,
    pub location_id: i64,
    pub location_type: String,
    pub quantity: i64,
    pub type_id: i64,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct AssetLocation {
    pub item_id: i64,
    pub position: Position,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct AssetName {
    pub item_id: i64,
    pub name: String,
}

impl AssetsGroup<'_> {
    api_get!(
        /// Get a character's assets.
        get_character_assets,
        "GetCharactersCharacterIdAssets",
        RequestType::Authenticated,
        Vec<Asset>,
        (character_id: i64) => "{character_id}"
    );

    api_post!(
        /// Get locations of some of a character's assets.
        get_character_assets_locations,
        "PostCharactersCharacterIdAssetsLocations",
        RequestType::Authenticated,
        Vec<AssetLocation>,
        (character_id: i64) => "{character_id}"
        ; Chunked(item_ids: &[i64], 1000)
    );

    api_post!(
        /// Get names of some of a character's assets.
        get_character_assets_names,
        "PostCharactersCharacterIdAssetsNames",
        RequestType::Authenticated,
        Vec<AssetName>,
        (character_id: i64) => "{character_id}"
        ; Chunked(item_ids: &[i64], 1000)
    );

    api_get!(
        /// Get a corporation's assets.
        ///
        /// Requires the auth'd character to be a director/+ in the corp.
        get_corporation_assets,
        "GetCorporationsCorporationIdAssets",
        RequestType::Authenticated,
        Vec<Asset>,
        (corporation_id: i64) => "{corporation_id}"
    );

    api_post!(
        /// Get locations of some of a corporation's assets.
        ///
        /// Requires the auth'd character to be a director/+ in the corp.
        get_corporation_assets_locations,
        "PostCorporationsCorporationIdAssetsLocations",
        RequestType::Authenticated,
        Vec<AssetLocation>,
        (corporation_id: i64) => "{corporation_id}"
        ; Chunked(item_ids: &[i64], 1000)
    );

    api_post!(
        /// Get names of some of a corporation's assets.
        ///
        /// Requires the auth'd character to be a director/+ in the corp.
        get_corporation_assets_names,
        "PostCorporationsCorporationIdAssetsNames",
        RequestType::Authenticated,
        Vec<AssetName>,
        (corporation_id: i64) => "{corporation_id}"
        ; Chunked(item_ids: &[i64], 1000)
    );
}
