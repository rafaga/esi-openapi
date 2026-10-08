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
    check!(
        "attributes",
        esi.group_skills().get_attributes(character_id)
    );
    check!(
        "skill queue",
        esi.group_skills().get_skillqueue(character_id)
    );
    check!(
        "agents research",
        esi.group_character().get_agents_research(character_id)
    );
    check!("fatigue", esi.group_character().get_fatigue(character_id));
    check!("medals", esi.group_character().get_medals(character_id));
    check!(
        "contact notifications",
        esi.group_character()
            .get_contact_notifications(character_id)
    );
    check!("roles", esi.group_character().get_roles(character_id));
    check!(
        "standings",
        esi.group_character().get_standings(character_id)
    );
    check!("titles", esi.group_character().get_titles(character_id));
    check!(
        "wallet journal",
        esi.group_wallet().get_journal(character_id, Some(1))
    );
    check!(
        "loyalty points",
        esi.group_loyalty().get_character_points(character_id)
    );
    check!(
        "faction warfare stats",
        esi.group_faction_warfare()
            .get_character_stats(character_id)
    );
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
        "contracts",
        esi.group_contracts()
            .get_character_contracts(character_id, Some(1))
    );
    check!(
        "planets",
        esi.group_planetary_interaction()
            .get_character_planets(character_id)
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
        "character mining",
        esi.group_industry()
            .get_character_mining(character_id, Some(1))
    );
    check!(
        "character order history",
        esi.group_market()
            .get_character_orders_history(character_id, Some(1))
    );
    check!(
        "access lists",
        esi.group_access_list()
            .get_character_access_lists(character_id)
    );
    check!(
        "tactical operations",
        esi.group_activities()
            .get_character_tactical_operations(character_id)
    );
    check!(
        "skinr licenses",
        esi.group_cosmetics().get_character_skinr(character_id)
    );
    check!(
        "skinr components",
        esi.group_cosmetics()
            .get_character_skinr_components(character_id)
    );
    check!(
        "character freelance jobs",
        esi.group_freelance_jobs().get_character_jobs(character_id)
    );
    check!(
        "mercenary dens",
        esi.group_structures()
            .get_character_mercenary_dens(character_id)
    );
    check!(
        "character campaign objectives",
        esi.group_military_campaigns().list_character_objectives(
            character_id,
            None,
            None,
            Some(10)
        )
    );
    check!(
        "character skinr listings",
        esi.group_paragon_hub()
            .get_character_skinr_listings(character_id, None, None, Some(10))
    );
    // The raw body, to see which keys ESI really sends for this endpoint.
    let skinr_path = esi
        .get_endpoint_for_op_id("GetCharactersParagonHubSkinr")
        .unwrap()
        .replace("{character_id}", &character_id.to_string());
    let raw: esi_openapi::prelude::EsiResult<serde_json::Value> = esi
        .query(
            "GET",
            esi_openapi::prelude::RequestType::Authenticated,
            &skinr_path,
            Some(&[("limit", "10")]),
            None,
        )
        .await;
    println!("RAW   character skinr listings: {raw:?}");
    check!(
        "raidable skyhooks",
        esi.group_activities().get_raidable_skyhooks()
    );
    match esi.group_character().get_public_info(character_id).await {
        Ok(info) => {
            let corp = info.corporation_id;
            let group = esi.group_corporation();
            check!("corporation icons", group.get_icons(corp));
            check!("corporation roles", group.get_roles(corp));
            check!("corporation divisions", group.get_divisions(corp));
            check!("corporation titles", group.get_titles(corp));
            check!("corporation medals", group.get_medals(corp, Some(1)));
            check!("corporation standings", group.get_standings(corp, Some(1)));
            check!(
                "corporation member tracking",
                group.get_member_tracking(corp)
            );
            check!(
                "corporation structures",
                group.get_structures(corp, Some(1))
            );
            check!("corporation starbases", group.get_starbases(corp, Some(1)));
            check!(
                "corporation blueprints",
                group.get_blueprints(corp, Some(1))
            );
            check!(
                "corporation wallets",
                esi.group_wallet().get_corporation_wallets(corp)
            );
            check!(
                "corporation wallet journal",
                esi.group_wallet().get_corporation_journal(corp, 1, Some(1))
            );
            check!(
                "corporation wallet transactions",
                esi.group_wallet()
                    .get_corporation_transactions(corp, 1, None)
            );
            check!(
                "corporation orders",
                esi.group_market().get_corporation_orders(corp, Some(1))
            );
            check!(
                "corporation order history",
                esi.group_market()
                    .get_corporation_orders_history(corp, Some(1))
            );
            check!(
                "corporation killmails",
                esi.group_killmails().get_corporation_recent(corp, Some(1))
            );
            check!(
                "corporation industry jobs",
                esi.group_industry()
                    .get_corporation_industry_jobs(corp, Some(true), Some(1))
            );
            check!(
                "corporation mining extractions",
                esi.group_industry()
                    .get_corporation_mining_extractions(corp, Some(1))
            );
            check!(
                "corporation mining observers",
                esi.group_industry()
                    .get_corporation_mining_observers(corp, Some(1))
            );
            check!(
                "corporation skyhooks",
                esi.group_structures().get_corporation_skyhooks(corp)
            );
            check!(
                "corporation sovereignty hubs",
                esi.group_structures()
                    .get_corporation_sovereignty_hubs(corp)
            );
            check!(
                "corporation projects",
                esi.group_corporation_projects()
                    .list(corp, None, None, Some(10), None)
            );
            check!(
                "corporation freelance jobs",
                esi.group_freelance_jobs()
                    .list_corporation_jobs(corp, None, None, Some(10))
            );
            check!(
                "corporation faction warfare stats",
                esi.group_faction_warfare().get_corporation_stats(corp)
            );
        }
        Err(e) => println!("SKIP  corporation checks: could not read the character: {e}"),
    }
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
        check!(
            "asset locations",
            esi.group_assets()
                .get_character_assets_locations(character_id, &singleton_ids)
        );
        check!(
            "asset names",
            esi.group_assets()
                .get_character_assets_names(character_id, &singleton_ids)
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
