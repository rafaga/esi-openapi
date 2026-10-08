use crate::groups::Page;
use crate::prelude::*;
use uuid::Uuid;

/// Endpoints for Paragon Hub
pub struct ParagonHubGroup<'a> {
    pub(crate) esi: &'a Esi,
}

/// The state of a SKINR listing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListingState {
    /// For sale.
    Listed,
    /// Sold out.
    SoldOut,
    /// Expired.
    Expired,
    /// Removed by the seller.
    Removed,
    /// A state this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// The price of a listing, in ISK or in PLEX.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(untagged)]
#[allow(missing_docs)]
pub enum ListingPrice {
    Isk { isk: f64 },
    Plex { plex: i64 },
}

/// A SKINR listing.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct SkinrListing {
    pub created: String,
    pub expires: String,
    pub id: Uuid,
    pub last_modified: String,
    pub price: ListingPrice,
    pub quantity: i64,
    /// Character ID of the seller.
    pub seller_id: i64,
    pub skinr_id: String,
    pub state: ListingState,
}

impl ParagonHubGroup<'_> {
    api_get!(
        /// List SKINR listings. Use the cursor of the returned page as `after` or
        /// `before` to walk through the list.
        get_skinr_listings,
        "GetParagonHubSkinr",
        RequestType::Public,
        Page<SkinrListing>,
        ;
        Optional(after: String) => "after",
        Optional(before: String) => "before",
        Optional(limit: i64) => "limit"
    );
}

#[cfg(test)]
mod tests {
    use super::{ListingPrice, ListingState};
    use crate::groups::{Page, SkinrListing};

    #[test]
    fn test_parse_listings_with_both_price_kinds() {
        let json = r#"{"listings": [
            {"created": "2026-10-01T00:00:00Z", "expires": "2026-11-01T00:00:00Z",
             "id": "3868eaed-8278-4cb7-9709-7d7de9c20dc7", "last_modified": "2026-10-02T00:00:00Z",
             "price": {"isk": 1500000.5}, "quantity": 1, "seller_id": 2112000000,
             "skinr_id": "abc", "state": "sold_out"},
            {"created": "2026-10-01T00:00:00Z", "expires": "2026-11-01T00:00:00Z",
             "id": "3868eaed-8278-4cb7-9709-7d7de9c20dc8", "last_modified": "2026-10-02T00:00:00Z",
             "price": {"plex": 250}, "quantity": 3, "seller_id": 2112000001,
             "skinr_id": "def", "state": "listed"}
        ]}"#;
        let page: Page<SkinrListing> = serde_json::from_str(json).unwrap();
        assert!(page.cursor.is_none());
        assert_eq!(page.items[0].state, ListingState::SoldOut);
        assert!(matches!(page.items[0].price, ListingPrice::Isk { .. }));
        assert!(matches!(
            page.items[1].price,
            ListingPrice::Plex { plex: 250 }
        ));
    }
}

/// Who a SKINR listing is offered to: only one field is set.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ListingTarget {
    /// Offered to one character.
    pub character_id: Option<i64>,
    /// Offered to one corporation.
    pub corporation_id: Option<i64>,
    /// Offered to one alliance.
    pub alliance_id: Option<i64>,
    /// Offered to everyone.
    pub public: Option<bool>,
}

/// A SKINR listing of a character, including who it is offered to.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct CharacterSkinrListing {
    pub created: String,
    pub expires: String,
    pub id: Uuid,
    pub last_modified: String,
    pub price: ListingPrice,
    pub quantity: i64,
    pub seller_id: i64,
    pub skinr_id: String,
    pub state: ListingState,
    pub target: ListingTarget,
}

impl ParagonHubGroup<'_> {
    api_get!(
        /// List the SKINR listings of a character.
        get_character_skinr_listings,
        "GetCharactersParagonHubSkinr",
        RequestType::Authenticated,
        Page<CharacterSkinrListing>,
        (character_id: i64) => "{character_id}";
        Optional(after: String) => "after",
        Optional(before: String) => "before",
        Optional(limit: i64) => "limit"
    );

    api_get!(
        /// List the SKINR listings offered to an alliance.
        get_alliance_skinr_listings,
        "GetParagonHubSkinrAlliances",
        RequestType::Authenticated,
        Page<SkinrListing>,
        (alliance_id: i64) => "{alliance_id}";
        Optional(after: String) => "after",
        Optional(before: String) => "before",
        Optional(limit: i64) => "limit"
    );

    api_get!(
        /// List the SKINR listings offered to a character.
        get_skinr_listings_for_character,
        "GetParagonHubSkinrCharacters",
        RequestType::Authenticated,
        Page<SkinrListing>,
        (character_id: i64) => "{character_id}";
        Optional(after: String) => "after",
        Optional(before: String) => "before",
        Optional(limit: i64) => "limit"
    );

    api_get!(
        /// List the SKINR listings offered to a corporation.
        get_corporation_skinr_listings,
        "GetParagonHubSkinrCorporations",
        RequestType::Authenticated,
        Page<SkinrListing>,
        (corporation_id: i64) => "{corporation_id}";
        Optional(after: String) => "after",
        Optional(before: String) => "before",
        Optional(limit: i64) => "limit"
    );
}

#[cfg(test)]
mod target_tests {
    use super::*;

    #[test]
    fn test_parse_character_listing_target() {
        let page: Page<CharacterSkinrListing> = serde_json::from_str(
            r#"{"listings":[{"created":"a","expires":"b","id":"67e6b7a0-6f58-4b5b-a2c4-4f3a1f0f2b11",
            "last_modified":"c","price":{"plex":5},"quantity":1,"seller_id":2,"skinr_id":"s",
            "state":"listed","target":{"corporation_id":9}}]}"#,
        )
        .unwrap();
        assert_eq!(page.items[0].target.corporation_id, Some(9));
        assert!(page.cursor.is_none());
    }
}
