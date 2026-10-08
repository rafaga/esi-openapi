use crate::prelude::*;

/// Endpoints for Loyalty
pub struct LoyaltyGroup<'a> {
    pub(crate) esi: &'a Esi,
}

/// An item required, besides LP and ISK, to take an offer.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct RequiredItem {
    pub quantity: i64,
    pub type_id: i64,
}

/// An offer in a loyalty points store.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct LoyaltyOffer {
    pub ak_cost: Option<i64>,
    pub isk_cost: i64,
    pub lp_cost: i64,
    pub offer_id: i64,
    pub quantity: i64,
    pub required_items: Vec<RequiredItem>,
    pub type_id: i64,
}

/// A character's loyalty points with a corporation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct LoyaltyPoints {
    pub corporation_id: i64,
    pub loyalty_points: i64,
}

impl LoyaltyGroup<'_> {
    api_get!(
        /// List a character's loyalty points per corporation.
        get_character_points,
        "GetCharactersCharacterIdLoyaltyPoints",
        RequestType::Authenticated,
        Vec<LoyaltyPoints>,
        (character_id: i64) => "{character_id}"
    );

    api_get!(
        /// List the offers of a corporation's loyalty points store.
        get_store_offers,
        "GetLoyaltyStoresCorporationIdOffers",
        RequestType::Public,
        Vec<LoyaltyOffer>,
        (corporation_id: i64) => "{corporation_id}"
    );
}

#[cfg(test)]
mod tests {
    use super::LoyaltyOffer;

    #[test]
    fn test_parse_offer() {
        let json = r#"[{
            "isk_cost": 5000000000, "lp_cost": 1000, "offer_id": 3751,
            "quantity": 1, "required_items": [{"quantity": 2, "type_id": 34}], "type_id": 23
        }]"#;
        let offers: Vec<LoyaltyOffer> = serde_json::from_str(json).unwrap();
        assert_eq!(offers[0].isk_cost, 5_000_000_000);
        assert!(offers[0].ak_cost.is_none());
    }
}
