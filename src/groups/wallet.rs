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
        (character_id: i32) => "{character_id}"
    );
}
