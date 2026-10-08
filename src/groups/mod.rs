//! Endpoint groups

mod access_list;
pub use access_list::*;
mod activities;
pub use activities::*;
mod alliance;
pub use alliance::*;
mod assets;
pub use assets::*;
mod bookmarks;
pub use bookmarks::*;
mod calendar;
pub use calendar::*;
mod character;
pub use character::*;
mod clones;
pub use clones::*;
mod contacts;
pub use contacts::*;
mod contracts;
pub use contracts::*;
mod corporation;
pub use corporation::*;
mod corporation_projects;
pub use corporation_projects::*;
mod cosmetics;
pub use cosmetics::*;
mod dogma;
pub use dogma::*;
mod faction_warfare;
pub use faction_warfare::*;
mod fittings;
pub use fittings::*;
mod freelance_jobs;
pub use freelance_jobs::*;
mod fleets;
pub use fleets::*;
mod incursions;
pub use incursions::*;
mod industry;
pub use industry::*;
mod insurance;
pub use insurance::*;
mod killmails;
pub use killmails::*;
mod location;
pub use location::*;
mod loyalty;
pub use loyalty::*;
mod mail;
pub use mail::*;
mod market;
pub use market::*;
mod military_campaigns;
pub use military_campaigns::*;
mod opportunities;
pub use opportunities::*;
mod paragon_hub;
pub use paragon_hub::*;
mod planetary_interaction;
pub use planetary_interaction::*;
mod routes;
pub use routes::*;
mod meta;
pub use meta::*;
mod search;
pub use search::*;
mod skills;
pub use skills::*;
mod sovereignty;
pub use sovereignty::*;
mod structures;
pub use structures::*;
mod status;
pub use status::*;
mod universe;
pub use universe::*;
mod user_interface;
pub use user_interface::*;
mod wallet;
pub use wallet::*;
mod wars;
pub use wars::*;

/// Cursor for walking through paginated listings.
///
/// Pass `after` or `before` from a page as the matching parameter of the next request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Cursor {
    /// Cursor to continue walking forwards in time.
    pub after: Option<String>,
    /// Cursor to continue walking backwards in time.
    pub before: Option<String>,
}

/// One page of a cursor-paginated listing, with the cursors to continue walking it.
///
/// ESI names the array of records after what the endpoint lists (`projects`,
/// `listings`, `objectives`, ...); here it is always [`Page::items`]. A response
/// without the array reads as a page without records.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Page<T> {
    /// Cursors to continue walking the list.
    pub cursor: Option<Cursor>,
    /// The records in the page.
    #[serde(
        default = "Vec::new",
        alias = "projects",
        alias = "contributors",
        alias = "freelance_jobs",
        alias = "participants",
        alias = "objectives",
        alias = "listings"
    )]
    pub items: Vec<T>,
}

#[cfg(test)]
mod page_tests {
    use super::Page;

    #[test]
    fn test_page_reads_every_key_esi_uses() {
        for key in [
            "projects",
            "contributors",
            "freelance_jobs",
            "participants",
            "objectives",
            "listings",
        ] {
            let json =
                format!(r#"{{"cursor": {{"after": "a", "before": null}}, "{key}": [1, 2]}}"#);
            let page: Page<i64> = serde_json::from_str(&json).unwrap();
            assert_eq!(page.items, [1, 2], "{key}");
            assert_eq!(page.cursor.unwrap().after.as_deref(), Some("a"));
        }
    }

    #[test]
    fn test_page_without_records_or_cursor_is_empty() {
        let page: Page<i64> = serde_json::from_str("{}").unwrap();
        assert!(page.items.is_empty() && page.cursor.is_none());
        let page: Page<i64> =
            serde_json::from_str(r#"{"cursor": {"after": null, "before": null}}"#).unwrap();
        assert!(page.items.is_empty());
    }

    #[test]
    fn test_page_round_trips() {
        let page = Page {
            cursor: None,
            items: vec![1_i64, 2],
        };
        let json = serde_json::to_string(&page).unwrap();
        assert_eq!(serde_json::from_str::<Page<i64>>(&json).unwrap(), page);
    }
}
