#![allow(unused)]

use crate::prelude::*;

/// Endpoints for Search
pub struct SearchGroup<'a> {
    pub(crate) esi: &'a Esi,
}

impl SearchGroup<'_> {
    api_get!(
        /// Search for entities that match a given sub-string.
        search,
        "GetCharactersCharacterIdSearch",
        RequestType::Authenticated,
        SearchResult,
        (character_id: i64) => "{character_id}";
        (categories: String) => "categories",
        (search: String) => "search";
        Optional(strict: bool) => "strict"
    );
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct SearchResult {
    pub agent: Option<Vec<i64>>,
    pub alliance: Option<Vec<i64>>,
    pub character: Option<Vec<i64>>,
    pub constellation: Option<Vec<i64>>,
    pub corporation: Option<Vec<i64>>,
    pub faction: Option<Vec<i64>>,
    pub inventory_type: Option<Vec<i64>>,
    pub region: Option<Vec<i64>>,
    pub solar_system: Option<Vec<i64>>,
    pub station: Option<Vec<i64>>,
    pub structure: Option<Vec<i64>>,
}
