//! Integration tests for the endpoints that need an authenticated character.
//!
//! They call the live ESI API with the credentials in `.env` (see
//! `.env.example`) and run as part of `cargo test`. When `.env` is missing
//! or incomplete (no `ESI_CLIENT_ID` or `ESI_REFRESH_TOKEN`), the test prints
//! why and passes without calling ESI, so `cargo test` works anywhere (CI
//! included). To see the per-endpoint results:
//!
//! ```sh
//! cargo test --test authenticated -- --nocapture
//! ```

mod common;

use std::fmt::Debug;

/// Outcome of one endpoint check.
struct Check {
    name: &'static str,
    result: Result<String, String>,
}

fn summarize<T: Debug>(
    result: Result<T, esi_openapi::prelude::EsiError>,
) -> Result<String, String> {
    match result {
        Ok(value) => {
            let text = format!("{value:?}");
            let short: String = text.chars().take(80).collect();
            Ok(if text.len() > 80 {
                format!("{short}...")
            } else {
                short
            })
        }
        Err(e) => Err(format!("{e} ({e:?})")),
    }
}

#[tokio::test]
async fn authenticated_endpoints() {
    let cfg = match common::load() {
        Ok(cfg) => cfg,
        Err(reason) => {
            eprintln!("skipping authenticated tests: {reason}");
            return;
        }
    };
    let (esi, character_id) = match common::authenticated_esi(&cfg).await {
        Ok(v) => v,
        Err(reason) if reason.starts_with("missing") => {
            eprintln!("skipping authenticated tests: {reason}");
            return;
        }
        Err(reason) => panic!("{reason}"),
    };
    println!("Testing as character {character_id}");

    let mut checks = Vec::new();
    macro_rules! check {
        ($name:literal, $call:expr) => {
            checks.push(Check {
                name: $name,
                result: summarize($call.await),
            });
        };
    }

    check!("skills", esi.group_skills().get_skills(character_id));
    check!("wallet", esi.group_wallet().get_wallet(character_id));
    check!(
        "wallet transactions",
        esi.group_character().get_wallet_transactions(character_id)
    );
    check!("location", esi.group_location().get_location(character_id));
    check!("online", esi.group_location().get_online(character_id));
    check!("ship", esi.group_location().get_ship(character_id));
    check!("clones", esi.group_clones().get_clones(character_id));
    check!(
        "implants",
        esi.group_clones().get_clone_implants(character_id)
    );
    check!(
        "blueprints",
        esi.group_character().get_blueprints(character_id)
    );
    check!(
        "notifications",
        esi.group_character().get_notifications(character_id)
    );
    check!(
        "mail labels",
        esi.group_mail().get_character_mail_labels(character_id)
    );
    check!(
        "market orders",
        esi.group_market().get_character_orders(character_id)
    );
    check!(
        "industry jobs",
        esi.group_industry()
            .get_character_industry_jobs(character_id, Some(true))
    );
    check!(
        "recent killmails",
        esi.group_killmails().get_character_recent(character_id)
    );
    check!(
        "search",
        esi.group_search().search(
            character_id,
            "solar_system".to_owned(),
            "Jita".to_owned(),
            Some(true)
        )
    );

    let assets = esi.group_assets().get_character_assets(character_id).await;
    let singleton_ids: Vec<i64> = match &assets {
        Ok(items) => items
            .iter()
            .filter(|a| a.is_singleton)
            .take(5)
            .map(|a| a.item_id)
            .collect(),
        Err(_) => Vec::new(),
    };
    checks.push(Check {
        name: "assets",
        result: summarize(assets.map(|a| a.len())),
    });
    if singleton_ids.is_empty() {
        println!("SKIP  asset locations/names: no singleton assets");
    } else {
        let unsigned: Vec<u64> = singleton_ids.iter().map(|&id| id as u64).collect();
        check!(
            "asset locations",
            esi.group_assets()
                .get_character_assets_locations(character_id, &singleton_ids)
        );
        check!(
            "asset names",
            esi.group_assets()
                .get_character_assets_names(character_id, &unsigned)
        );
    }

    match cfg.structure_id {
        Some(structure_id) => {
            check!(
                "structure",
                esi.group_universe().get_structure(structure_id)
            );
        }
        None => println!("SKIP  structure: ESI_TEST_STRUCTURE_ID not set"),
    }

    let mut failures = 0;
    for check in &checks {
        match &check.result {
            Ok(summary) => println!("OK    {}: {summary}", check.name),
            Err(error) => {
                failures += 1;
                println!("FAIL  {}: {error}", check.name);
            }
        }
    }
    println!("Rate limits: {:?}", esi.rate_limit_statuses().await);
    assert_eq!(failures, 0, "{failures} of {} checks failed", checks.len());
}
