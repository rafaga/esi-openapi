use crate::prelude::*;

/// Endpoints for Insurance
pub struct InsuranceGroup<'a> {
    pub(crate) esi: &'a Esi,
}

/// One insurance level for a ship type.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct InsuranceLevel {
    pub cost: f64,
    pub name: String,
    pub payout: f64,
}

/// Insurance prices for a ship type.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct InsurancePrice {
    pub levels: Vec<InsuranceLevel>,
    pub type_id: i64,
}

impl InsuranceGroup<'_> {
    api_get!(
        /// List the insurance levels for all ship types.
        get_prices,
        "GetInsurancePrices",
        RequestType::Public,
        Vec<InsurancePrice>,
    );
}
