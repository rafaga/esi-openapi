#![allow(unused)]

use crate::prelude::*;

/// Endpoints for Wallet
pub struct WalletGroup<'a> {
    pub(crate) esi: &'a Esi,
}

impl WalletGroup<'_> {
    api_get!(
        /// Returns a character’s wallet balance
        get_wallet,
        "GetCharactersCharacterIdWallet",
        RequestType::Authenticated,
        f64,
        (character_id: i64) => "{character_id}"
    );
}

/// What the context ID of a wallet journal entry refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JournalContextType {
    /// A structure ID.
    StructureId,
    /// A station ID.
    StationId,
    /// A market transaction ID.
    MarketTransactionId,
    /// A character ID.
    CharacterId,
    /// A corporation ID.
    CorporationId,
    /// An alliance ID.
    AllianceId,
    /// An EVE system.
    EveSystem,
    /// An industry job ID.
    IndustryJobId,
    /// A contract ID.
    ContractId,
    /// A planet ID.
    PlanetId,
    /// A solar system ID.
    SystemId,
    /// A type ID.
    TypeId,
    /// A kind this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// An entry of a wallet journal.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct JournalEntry {
    pub amount: Option<f64>,
    pub balance: Option<f64>,
    pub context_id: Option<i64>,
    pub context_id_type: Option<JournalContextType>,
    pub date: String,
    pub description: String,
    pub first_party_id: Option<i64>,
    pub id: i64,
    pub reason: Option<String>,
    /// The kind of transaction, such as `bounty_prizes` or `player_trading`.
    /// ESI defines more than 160 values, so it is kept as a string.
    pub ref_type: String,
    pub second_party_id: Option<i64>,
    pub tax: Option<f64>,
    pub tax_receiver_id: Option<i64>,
}

impl WalletGroup<'_> {
    api_get!(
        /// Get a character's wallet journal for the last 30 days.
        get_journal,
        "GetCharactersCharacterIdWalletJournal",
        RequestType::Authenticated,
        Vec<JournalEntry>,
        (character_id: i64) => "{character_id}";
        Optional(page: i32) => "page"
    );
}

#[cfg(test)]
mod character_basic_tests {
    use super::{JournalContextType, JournalEntry};

    #[test]
    fn test_parse_journal_entries() {
        let json = r#"[
            {"id": 9000000000, "date": "2026-10-01T00:00:00Z", "description": "Bounty",
             "ref_type": "bounty_prizes", "amount": 1500.5, "balance": 100.0,
             "context_id": 30000142, "context_id_type": "system_id", "first_party_id": 2112625428},
            {"id": 2, "date": "2026-10-01T00:00:00Z", "description": "?", "ref_type": "new_kind",
             "context_id_type": "something_new"}
        ]"#;
        let entries: Vec<JournalEntry> = serde_json::from_str(json).unwrap();
        assert_eq!(entries[0].id, 9_000_000_000);
        assert_eq!(
            entries[0].context_id_type,
            Some(JournalContextType::SystemId)
        );
        assert_eq!(
            entries[1].context_id_type,
            Some(JournalContextType::Unrecognized)
        );
        assert!(entries[1].amount.is_none());
    }
}

/// The balance of one corporation wallet division.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct DivisionBalance {
    pub balance: f64,
    pub division: i64,
}

/// A market transaction of a corporation wallet division.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct CorporationWalletTransaction {
    pub client_id: i64,
    pub date: String,
    pub is_buy: bool,
    pub journal_ref_id: i64,
    pub location_id: i64,
    pub quantity: i64,
    pub transaction_id: i64,
    pub type_id: i64,
    pub unit_price: f64,
}

impl WalletGroup<'_> {
    api_get!(
        /// Get the balance of every wallet division of a corporation.
        get_corporation_wallets,
        "GetCorporationsCorporationIdWallets",
        RequestType::Authenticated,
        Vec<DivisionBalance>,
        (corporation_id: i64) => "{corporation_id}"
    );

    api_get!(
        /// Get the journal of a corporation wallet division for the last 30 days.
        get_corporation_journal,
        "GetCorporationsCorporationIdWalletsDivisionJournal",
        RequestType::Authenticated,
        Vec<JournalEntry>,
        (corporation_id: i64) => "{corporation_id}",
        (division: i64) => "{division}";
        Optional(page: i32) => "page"
    );

    api_get!(
        /// Get the market transactions of a corporation wallet division.
        /// Pass the lowest `transaction_id` of the previous page as `from_id` to continue.
        get_corporation_transactions,
        "GetCorporationsCorporationIdWalletsDivisionTransactions",
        RequestType::Authenticated,
        Vec<CorporationWalletTransaction>,
        (corporation_id: i64) => "{corporation_id}",
        (division: i64) => "{division}";
        Optional(from_id: i64) => "from_id"
    );
}
