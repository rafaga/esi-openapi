#![allow(unused)]

use crate::prelude::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct HistoryItem {
    pub average: f64,
    pub date: String,
    pub highest: f64,
    pub lowest: f64,
    pub order_count: i64,
    pub volume: i64,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct MarketOrder {
    pub duration: i64,
    pub is_buy_order: bool,
    pub issued: String,
    pub location_id: i64,
    pub min_volume: i64,
    pub order_id: i64,
    pub price: f64,
    pub range: String,
    pub system_id: i64,
    pub type_id: i64,
    pub volume_remain: i64,
    pub volume_total: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct PriceItem {
    pub adjusted_price: Option<f64>,
    pub average_price: Option<f64>,
    pub type_id: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct CharacterOrder {
    pub duration: i64,
    pub escrow: Option<f64>,
    pub is_buy_order: Option<bool>,
    pub is_corporation: bool,
    pub issued: String,
    pub location_id: i64,
    pub min_volume: Option<i64>,
    pub order_id: i64,
    pub price: f64,
    pub range: String,
    pub region_id: i64,
    pub type_id: i64,
    pub volume_remain: i64,
    pub volume_total: i64,
}

/// Endpoints for Market
pub struct MarketGroup<'a> {
    pub(crate) esi: &'a Esi,
}

impl MarketGroup<'_> {
    api_get!(
        /// Get a list of historical market statistics for the specified type in a region
        get_region_history,
        "GetMarketsRegionIdHistory",
        RequestType::Public,
        Vec<HistoryItem>,
        (region_id: i64) => "{region_id}";
        (type_id: i64) => "type_id"
    );

    api_get!(
        /// Get a list of orders in a region
        get_region_orders,
        "GetMarketsRegionIdOrders",
        RequestType::Public,
        Vec<MarketOrder>,
        (region_id: i64) => "{region_id}";
        Optional(order_type: String) => "order_type",
        Optional(page: i32) => "page",
        Optional(type_id: i64) => "type_id"
    );

    api_get!(
        /// Get a list of average and adjusted prices
        get_market_prices,
        "GetMarketsPrices",
        RequestType::Public,
        Vec<PriceItem>,
    );

    api_get!(
        /// List open market orders placed by a character
        get_character_orders,
        "GetCharactersCharacterIdOrders",
        RequestType::Authenticated,
        Vec<CharacterOrder>,
        (character_id: i64) => "{character_id}"
    );
}

/// Information about a market group.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct MarketGroupInfo {
    pub description: String,
    pub market_group_id: i64,
    pub name: String,
    pub parent_group_id: Option<i64>,
    pub types: Vec<i64>,
}

impl MarketGroup<'_> {
    api_get!(
        /// List the IDs of all market groups.
        get_groups,
        "GetMarketsGroups",
        RequestType::Public,
        Vec<i64>,
    );

    api_get!(
        /// Get information about a market group.
        get_group,
        "GetMarketsGroupsMarketGroupId",
        RequestType::Public,
        MarketGroupInfo,
        (market_group_id: i64) => "{market_group_id}"
    );

    api_get!(
        /// List the type IDs that have active orders in a region.
        get_region_types,
        "GetMarketsRegionIdTypes",
        RequestType::Public,
        Vec<i64>,
        (region_id: i64) => "{region_id}";
        Optional(page: i32) => "page"
    );
}

/// How far away a market order can be matched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum OrderRange {
    /// Only at the order's station.
    #[serde(rename = "station")]
    Station,
    /// Within the whole region.
    #[serde(rename = "region")]
    Region,
    /// Within the solar system.
    #[serde(rename = "solarsystem")]
    SolarSystem,
    /// Within 1 jump.
    #[serde(rename = "1")]
    Jumps1,
    /// Within 2 jumps.
    #[serde(rename = "2")]
    Jumps2,
    /// Within 3 jumps.
    #[serde(rename = "3")]
    Jumps3,
    /// Within 4 jumps.
    #[serde(rename = "4")]
    Jumps4,
    /// Within 5 jumps.
    #[serde(rename = "5")]
    Jumps5,
    /// Within 10 jumps.
    #[serde(rename = "10")]
    Jumps10,
    /// Within 20 jumps.
    #[serde(rename = "20")]
    Jumps20,
    /// Within 30 jumps.
    #[serde(rename = "30")]
    Jumps30,
    /// Within 40 jumps.
    #[serde(rename = "40")]
    Jumps40,
    /// A range this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// Final state of a market order that is no longer open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClosedOrderState {
    /// The order was cancelled.
    Cancelled,
    /// The order expired.
    Expired,
    /// A state this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// A closed market order of a character.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct CharacterOrderHistory {
    pub duration: i64,
    pub escrow: Option<f64>,
    pub is_buy_order: Option<bool>,
    pub is_corporation: bool,
    pub issued: String,
    pub location_id: i64,
    pub min_volume: Option<i64>,
    pub order_id: i64,
    pub price: f64,
    pub range: OrderRange,
    pub region_id: i64,
    pub state: ClosedOrderState,
    pub type_id: i64,
    pub volume_remain: i64,
    pub volume_total: i64,
}

/// An open market order of a corporation.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct CorporationOrder {
    pub duration: i64,
    pub escrow: Option<f64>,
    pub is_buy_order: Option<bool>,
    pub issued: String,
    pub issued_by: i64,
    pub location_id: i64,
    pub min_volume: Option<i64>,
    pub order_id: i64,
    pub price: f64,
    pub range: OrderRange,
    pub region_id: i64,
    pub type_id: i64,
    pub volume_remain: i64,
    pub volume_total: i64,
    pub wallet_division: i64,
}

/// A closed market order of a corporation.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct CorporationOrderHistory {
    pub duration: i64,
    pub escrow: Option<f64>,
    pub is_buy_order: Option<bool>,
    pub issued: String,
    pub issued_by: Option<i64>,
    pub location_id: i64,
    pub min_volume: Option<i64>,
    pub order_id: i64,
    pub price: f64,
    pub range: OrderRange,
    pub region_id: i64,
    pub state: ClosedOrderState,
    pub type_id: i64,
    pub volume_remain: i64,
    pub volume_total: i64,
    pub wallet_division: i64,
}

/// An order in a player-owned structure market.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct StructureOrder {
    pub duration: i64,
    pub is_buy_order: bool,
    pub issued: String,
    pub location_id: i64,
    pub min_volume: i64,
    pub order_id: i64,
    pub price: f64,
    pub range: OrderRange,
    pub type_id: i64,
    pub volume_remain: i64,
    pub volume_total: i64,
}

impl MarketGroup<'_> {
    api_get!(
        /// List cancelled and expired market orders placed by a character (up to 90 days).
        get_character_orders_history,
        "GetCharactersCharacterIdOrdersHistory",
        RequestType::Authenticated,
        Vec<CharacterOrderHistory>,
        (character_id: i64) => "{character_id}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// List open market orders placed on behalf of a corporation.
        get_corporation_orders,
        "GetCorporationsCorporationIdOrders",
        RequestType::Authenticated,
        Vec<CorporationOrder>,
        (corporation_id: i64) => "{corporation_id}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// List cancelled and expired market orders placed on behalf of a corporation.
        get_corporation_orders_history,
        "GetCorporationsCorporationIdOrdersHistory",
        RequestType::Authenticated,
        Vec<CorporationOrderHistory>,
        (corporation_id: i64) => "{corporation_id}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// List orders in a structure's market.
        get_structure_orders,
        "GetMarketsStructuresStructureId",
        RequestType::Authenticated,
        Vec<StructureOrder>,
        (structure_id: i64) => "{structure_id}";
        Optional(page: i32) => "page"
    );
}

#[cfg(test)]
mod order_history_tests {
    use super::*;

    #[test]
    fn test_parse_ranges_and_states() {
        let ranges: Vec<OrderRange> =
            serde_json::from_str(r#"["station","solarsystem","10","weird"]"#).unwrap();
        assert_eq!(
            ranges,
            [
                OrderRange::Station,
                OrderRange::SolarSystem,
                OrderRange::Jumps10,
                OrderRange::Unrecognized
            ]
        );
        let order: CorporationOrderHistory = serde_json::from_str(
            r#"{"duration":90,"issued":"x","location_id":1,"order_id":2,"price":1.5,"range":"region",
            "region_id":3,"state":"expired","type_id":4,"volume_remain":0,"volume_total":5,"wallet_division":1}"#,
        )
        .unwrap();
        assert_eq!(order.state, ClosedOrderState::Expired);
        assert!(order.issued_by.is_none());
    }
}
