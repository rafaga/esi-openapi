use crate::prelude::*;

/// Endpoints for Contracts
pub struct ContractsGroup<'a> {
    pub(crate) esi: &'a Esi,
}

/// The type of a contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractType {
    /// The type is not known.
    Unknown,
    /// An item exchange.
    ItemExchange,
    /// An auction.
    Auction,
    /// A courier contract.
    Courier,
    /// A loan.
    Loan,
    /// A type this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// A public contract in a region.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct PublicContract {
    pub buyout: Option<f64>,
    pub collateral: Option<f64>,
    pub contract_id: i64,
    pub date_expired: String,
    pub date_issued: String,
    pub days_to_complete: Option<i64>,
    pub end_location_id: Option<i64>,
    pub for_corporation: Option<bool>,
    pub issuer_corporation_id: i64,
    /// Character ID of the issuer.
    pub issuer_id: i64,
    pub price: Option<f64>,
    pub reward: Option<f64>,
    pub start_location_id: Option<i64>,
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub contract_type: ContractType,
    pub volume: Option<f64>,
}

/// A bid on a public auction contract.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct PublicContractBid {
    pub amount: f64,
    pub bid_id: i64,
    pub date_bid: String,
}

/// An item of a public contract.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct PublicContractItem {
    pub is_blueprint_copy: Option<bool>,
    pub is_included: bool,
    pub item_id: Option<i64>,
    pub material_efficiency: Option<i64>,
    pub quantity: i64,
    pub record_id: i64,
    pub runs: Option<i64>,
    pub time_efficiency: Option<i64>,
    pub type_id: i64,
}

/// Who can accept a contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractAvailability {
    /// Anyone.
    Public,
    /// A specific character or corporation.
    Personal,
    /// Members of a corporation.
    Corporation,
    /// Members of an alliance.
    Alliance,
    /// A value this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// The status of a contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractStatus {
    /// Waiting for an acceptor.
    Outstanding,
    /// Accepted and in progress.
    InProgress,
    /// Finished by the issuer.
    FinishedIssuer,
    /// Finished by the contractor.
    FinishedContractor,
    /// Finished.
    Finished,
    /// Cancelled.
    Cancelled,
    /// Rejected.
    Rejected,
    /// Failed.
    Failed,
    /// Deleted.
    Deleted,
    /// Reversed.
    Reversed,
    /// A value this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// A contract of a character or a corporation.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Contract {
    /// ID of the character or corporation that accepted the contract
    /// (`0` if nobody did).
    pub acceptor_id: i64,
    /// ID of the character, corporation or alliance the contract is assigned to.
    pub assignee_id: i64,
    /// Who can accept the contract.
    pub availability: ContractAvailability,
    /// Buyout price (auctions).
    pub buyout: Option<f64>,
    /// Collateral (couriers).
    pub collateral: Option<f64>,
    /// ID of the contract.
    pub contract_id: i64,
    /// Date the contract was accepted.
    pub date_accepted: Option<String>,
    /// Date the contract was completed.
    pub date_completed: Option<String>,
    /// Date the contract expires.
    pub date_expired: String,
    /// Date the contract was issued.
    pub date_issued: String,
    /// Days to complete (couriers).
    pub days_to_complete: Option<i64>,
    /// Destination location (couriers).
    pub end_location_id: Option<i64>,
    /// Whether the contract was issued on behalf of a corporation.
    pub for_corporation: bool,
    /// Corporation ID of the issuer.
    pub issuer_corporation_id: i64,
    /// Character ID of the issuer.
    pub issuer_id: i64,
    /// Price (item exchanges and auctions).
    pub price: Option<f64>,
    /// Reward (couriers).
    pub reward: Option<f64>,
    /// Start location.
    pub start_location_id: Option<i64>,
    /// Status of the contract.
    pub status: ContractStatus,
    /// Title of the contract.
    pub title: Option<String>,
    /// Type of the contract.
    #[serde(rename = "type")]
    pub contract_type: ContractType,
    /// Volume of the items, in m3.
    pub volume: Option<f64>,
}

/// A bid on an auction contract.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ContractBid {
    /// Amount of the bid.
    pub amount: f64,
    /// ID of the bid.
    pub bid_id: i64,
    /// Character ID of the bidder.
    pub bidder_id: i64,
    /// Date of the bid.
    pub date_bid: String,
}

/// An item of a contract.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ContractItem {
    /// Whether the item is offered (`true`) or requested (`false`).
    pub is_included: bool,
    /// Whether the item is a singleton (assembled).
    pub is_singleton: bool,
    /// Quantity of the item.
    pub quantity: i64,
    /// Raw quantity, used for blueprints and damaged items.
    pub raw_quantity: Option<i64>,
    /// ID of the record.
    pub record_id: i64,
    /// Type ID of the item.
    pub type_id: i64,
}

impl ContractsGroup<'_> {
    api_get!(
        /// List the bids on a public auction contract.
        get_public_contract_bids,
        "GetContractsPublicBidsContractId",
        RequestType::Public,
        Vec<PublicContractBid>,
        (contract_id: i64) => "{contract_id}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// List the items of a public contract.
        get_public_contract_items,
        "GetContractsPublicItemsContractId",
        RequestType::Public,
        Vec<PublicContractItem>,
        (contract_id: i64) => "{contract_id}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// List the public contracts in a region.
        get_public_contracts,
        "GetContractsPublicRegionId",
        RequestType::Public,
        Vec<PublicContract>,
        (region_id: i64) => "{region_id}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// List the contracts of a character.
        get_character_contracts,
        "GetCharactersCharacterIdContracts",
        RequestType::Authenticated,
        Vec<Contract>,
        (character_id: i64) => "{character_id}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// List the bids on an auction contract of a character.
        get_character_contract_bids,
        "GetCharactersCharacterIdContractsContractIdBids",
        RequestType::Authenticated,
        Vec<ContractBid>,
        (character_id: i64) => "{character_id}",
        (contract_id: i64) => "{contract_id}"
    );

    api_get!(
        /// List the items of a contract of a character.
        get_character_contract_items,
        "GetCharactersCharacterIdContractsContractIdItems",
        RequestType::Authenticated,
        Vec<ContractItem>,
        (character_id: i64) => "{character_id}",
        (contract_id: i64) => "{contract_id}"
    );

    api_get!(
        /// List the contracts of a corporation.
        get_corporation_contracts,
        "GetCorporationsCorporationIdContracts",
        RequestType::Authenticated,
        Vec<Contract>,
        (corporation_id: i64) => "{corporation_id}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// List the bids on an auction contract of a corporation.
        get_corporation_contract_bids,
        "GetCorporationsCorporationIdContractsContractIdBids",
        RequestType::Authenticated,
        Vec<ContractBid>,
        (corporation_id: i64) => "{corporation_id}",
        (contract_id: i64) => "{contract_id}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// List the items of a contract of a corporation.
        get_corporation_contract_items,
        "GetCorporationsCorporationIdContractsContractIdItems",
        RequestType::Authenticated,
        Vec<ContractItem>,
        (corporation_id: i64) => "{corporation_id}",
        (contract_id: i64) => "{contract_id}"
    );
}

#[cfg(test)]
mod tests {
    use super::{ContractType, PublicContract};

    #[test]
    fn test_parse_contract_and_unknown_type() {
        let base = |kind: &str| {
            format!(
                r#"{{"contract_id": 1, "date_expired": "2026-01-02T00:00:00Z",
                "date_issued": "2026-01-01T00:00:00Z", "issuer_corporation_id": 98000001,
                "issuer_id": 2112000000, "type": "{kind}"}}"#
            )
        };
        let known: PublicContract = serde_json::from_str(&base("item_exchange")).unwrap();
        assert_eq!(known.contract_type, ContractType::ItemExchange);
        let unknown: PublicContract = serde_json::from_str(&base("brand_new")).unwrap();
        assert_eq!(unknown.contract_type, ContractType::Unrecognized);
    }
}

#[cfg(test)]
mod private_contract_tests {
    use super::*;

    #[test]
    fn test_parse_private_contract() {
        let json = r#"[{"acceptor_id": 0, "assignee_id": 2112625428, "availability": "personal",
            "contract_id": 300000000, "date_expired": "e", "date_issued": "i",
            "for_corporation": false, "issuer_corporation_id": 98000001, "issuer_id": 2112000000,
            "status": "finished_issuer", "type": "courier", "reward": 1000.0},
            {"acceptor_id": 0, "assignee_id": 0, "availability": "x", "contract_id": 1,
            "date_expired": "e", "date_issued": "i", "for_corporation": true,
            "issuer_corporation_id": 1, "issuer_id": 1, "status": "later", "type": "loan"}]"#;
        let c: Vec<Contract> = serde_json::from_str(json).unwrap();
        assert_eq!(c[0].status, ContractStatus::FinishedIssuer);
        assert_eq!(c[0].availability, ContractAvailability::Personal);
        assert_eq!(c[1].status, ContractStatus::Unrecognized);
        assert_eq!(c[1].availability, ContractAvailability::Unrecognized);
    }

    #[test]
    fn test_parse_bids_and_items() {
        let b: Vec<ContractBid> = serde_json::from_str(
            r#"[{"amount": 5.5, "bid_id": 1, "bidder_id": 2112000000, "date_bid": "d"}]"#,
        )
        .unwrap();
        assert_eq!(b[0].bidder_id, 2_112_000_000);
        let i: Vec<ContractItem> = serde_json::from_str(
            r#"[{"is_included": true, "is_singleton": false, "quantity": 5000000000,
                 "record_id": 9, "type_id": 34}]"#,
        )
        .unwrap();
        assert_eq!(i[0].quantity, 5_000_000_000);
        assert_eq!(i[0].raw_quantity, None);
    }
}
