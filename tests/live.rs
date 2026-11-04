//! Smoke test against the live ESI API, for the public endpoints.
//!
//! It calls one endpoint of every public group with real data and reads the
//! answer into the type the crate returns, so a change in what ESI sends that
//! the OpenAPI spec does not show (a field that becomes `null`, a new type of
//! value) is found here. Ids are taken from earlier answers where they can be
//! (the first alliance, the CEO of a corporation, ...) and fixed only for the
//! stable ones (The Forge, Jita, Tritanium).
//!
//! It is ignored by default because it needs the network. To run it:
//!
//! ```sh
//! cargo test --test live -- --ignored --nocapture
//! ```
//!
//! The weekly `live-esi` workflow runs it too. A step that fails does not stop
//! the others; the test reports every failure at the end.

use esi_openapi::groups::RouteRequest;
use esi_openapi::prelude::*;
use std::fmt::Debug;

/// What each step found.
#[derive(Default)]
struct Report {
    rows: Vec<(String, Result<String, String>)>,
}

impl Report {
    /// Records the outcome of a step and gives back its value.
    fn note<T: Debug>(&mut self, name: &str, result: EsiResult<T>) -> Option<T> {
        match result {
            Ok(value) => {
                let text = format!("{value:?}");
                let short: String = text.chars().take(70).collect();
                self.rows.push((name.to_owned(), Ok(short)));
                Some(value)
            }
            Err(error) => {
                self.rows
                    .push((name.to_owned(), Err(format!("{error} ({error:?})"))));
                None
            }
        }
    }

    fn failures(&self) -> Vec<String> {
        self.rows
            .iter()
            .filter_map(|(name, result)| result.as_ref().err().map(|e| format!("{name}: {e}")))
            .collect()
    }

    fn print(&self) {
        for (name, result) in &self.rows {
            match result {
                Ok(short) => println!("ok    {name}: {short}"),
                Err(error) => println!("FAIL  {name}: {error}"),
            }
        }
    }
}

struct Smoke {
    esi: Esi,
    report: Report,
}

impl Smoke {
    /// A client with the live OpenAPI spec downloaded.
    async fn new() -> Self {
        let mut esi = EsiBuilder::new()
            .user_agent("esi-openapi live smoke test (https://github.com/rafaga/esi-openapi)")
            .http_timeout(Some(30_000))
            .build()
            .unwrap();
        let mut report = Report::default();
        let spec = report.note("spec download", esi.update_spec().await);
        assert!(spec.is_some(), "{:?}", report.failures());
        Self { esi, report }
    }

    async fn meta_and_status(&mut self) {
        let r = &mut self.report;
        r.note("status", self.esi.group_status().get_status().await);
        r.note("meta status", self.esi.group_meta().get_status().await);
        r.note("meta name", self.esi.group_meta().get_name().await);
        r.note(
            "meta compatibility dates",
            self.esi.group_meta().get_compatibility_dates().await,
        );
        r.note(
            "meta changelog",
            self.esi.group_meta().get_changelog().await,
        );
    }

    /// Alliance, corporation and character, each found from the one before.
    async fn people(&mut self) {
        let r = &mut self.report;
        let alliances = r.note("alliance ids", self.esi.group_alliance().list_ids().await);
        let Some(alliance_id) = alliances.and_then(|ids| ids.first().copied()) else {
            return;
        };
        r.note(
            "alliance info",
            self.esi.group_alliance().get_info(alliance_id).await,
        );
        r.note(
            "alliance icons",
            self.esi
                .group_alliance()
                .get_alliance_icons(alliance_id)
                .await,
        );
        let members = r.note(
            "alliance corporations",
            self.esi
                .group_alliance()
                .get_alliance_corporations(alliance_id)
                .await,
        );
        if let Some(corporation_id) = members.and_then(|ids| ids.first().copied()) {
            let corporation = r.note(
                "corporation info",
                self.esi
                    .group_corporation()
                    .get_public_info(corporation_id)
                    .await,
            );
            r.note(
                "corporation history",
                self.esi
                    .group_corporation()
                    .get_history(corporation_id)
                    .await,
            );
            r.note(
                "corporation icons",
                self.esi.group_corporation().get_icons(corporation_id).await,
            );
            r.note(
                "loyalty store offers",
                self.esi
                    .group_loyalty()
                    .get_store_offers(corporation_id)
                    .await,
            );
            if let Some(ceo_id) = corporation.and_then(|c| c.ceo_id) {
                r.note(
                    "character info",
                    self.esi.group_character().get_public_info(ceo_id).await,
                );
                r.note(
                    "character history",
                    self.esi.group_character().get_history(ceo_id).await,
                );
                r.note(
                    "character portrait",
                    self.esi.group_character().get_portrait(ceo_id).await,
                );
                r.note(
                    "character affiliation",
                    self.esi.group_character().get_affiliation(&[ceo_id]).await,
                );
            }
        }
        let npc = r.note(
            "npc corporations",
            self.esi.group_corporation().get_npc_corps().await,
        );
        if let Some(id) = npc.and_then(|ids| ids.first().copied()) {
            r.note(
                "npc corporation info",
                self.esi.group_corporation().get_public_info(id).await,
            );
        }
    }

    async fn market(&mut self) {
        let r = &mut self.report;
        let forge = 10000002;
        let tritanium = 34;
        r.note(
            "market history",
            self.esi
                .group_market()
                .get_region_history(forge, tritanium)
                .await,
        );
        r.note(
            "market orders",
            self.esi
                .group_market()
                .get_region_orders(forge, Some("all".to_owned()), None, Some(tritanium))
                .await,
        );
        r.note(
            "market prices",
            self.esi.group_market().get_market_prices().await,
        );
        r.note(
            "market region types",
            self.esi.group_market().get_region_types(forge, None).await,
        );
        let groups = r.note("market groups", self.esi.group_market().get_groups().await);
        if let Some(id) = groups.and_then(|ids| ids.first().copied()) {
            r.note("market group", self.esi.group_market().get_group(id).await);
        }
        r.note(
            "public contracts",
            self.esi
                .group_contracts()
                .get_public_contracts(forge, None)
                .await,
        );
        r.note(
            "insurance prices",
            self.esi.group_insurance().get_prices().await,
        );
    }

    async fn universe(&mut self) {
        let r = &mut self.report;
        let universe = self.esi.group_universe();
        let jita = r.note("jita system", universe.get_system(30000142).await);
        if let Some(jita) = jita {
            if let Some(star) = jita.star_id {
                r.note("star", universe.get_star(star).await);
            }
            if let Some(gate) = jita.stargates.as_ref().and_then(|g| g.first()) {
                r.note("stargate", universe.get_stargate(*gate).await);
            }
        }
        r.note("station", universe.get_station(60003760).await);
        r.note("region", universe.get_region(10000002).await);
        r.note("constellation", universe.get_constellation(20000020).await);
        r.note("type (Tritanium)", universe.get_type(34).await);
        r.note(
            "group (Mineral)",
            universe.get_universe_groups_group(18).await,
        );
        r.note(
            "category (Material)",
            universe.get_universe_categories_category(4).await,
        );
        r.note("system ids", universe.get_system_ids().await);
        r.note("region ids", universe.get_region_ids().await);
        r.note("constellation ids", universe.get_constellation_ids().await);
        r.note("category ids", universe.get_category_ids().await);
        let graphics = r.note("graphic ids", universe.get_graphic_ids().await);
        if let Some(id) = graphics.and_then(|ids| ids.first().copied()) {
            r.note("graphic", universe.get_graphic(id).await);
        }
        r.note("system jumps", universe.get_system_jumps().await);
        r.note("system kills", universe.get_system_kills().await);
        r.note(
            "system kills snapshot",
            universe.get_system_kills_snapshot().await,
        );
        r.note("ancestries", universe.get_ancestries().await);
        r.note("bloodlines", universe.get_bloodlines().await);
        r.note("races", universe.get_races().await);
        r.note("factions", universe.get_factions().await);
        r.note("names", universe.get_names(&[30000142, 10000002, 34]).await);
        r.note("ids", universe.get_ids(&["Jita", "The Forge"]).await);
        let route = RouteRequest::default();
        r.note(
            "route",
            self.esi
                .group_routes()
                .calculate_route(30000142, 30002187, &route)
                .await,
        );
    }

    async fn dogma(&mut self) {
        let r = &mut self.report;
        let attributes = r.note(
            "dogma attributes",
            self.esi.group_dogma().get_attributes().await,
        );
        if let Some(id) = attributes.and_then(|ids| ids.first().copied()) {
            r.note(
                "dogma attribute",
                self.esi.group_dogma().get_attribute(id).await,
            );
        }
        let effects = r.note("dogma effects", self.esi.group_dogma().get_effects().await);
        if let Some(id) = effects.and_then(|ids| ids.first().copied()) {
            r.note("dogma effect", self.esi.group_dogma().get_effect(id).await);
        }
        r.note(
            "planetary schematic",
            self.esi
                .group_planetary_interaction()
                .get_schematic(65)
                .await,
        );
    }

    async fn conflict(&mut self) {
        let r = &mut self.report;
        let fw = self.esi.group_faction_warfare();
        r.note("fw leaderboards", fw.leaderboards().await);
        r.note(
            "fw character leaderboards",
            fw.leaderboard_characters().await,
        );
        r.note(
            "fw corporation leaderboards",
            fw.leaderboard_corporations().await,
        );
        r.note("fw stats", fw.stats().await);
        r.note("fw systems", fw.systems().await);
        r.note("fw wars", fw.wars().await);
        r.note("incursions", self.esi.group_incursions().list().await);
        r.note(
            "sovereignty campaigns",
            self.esi.group_sovereignty().get_campaigns().await,
        );
        r.note(
            "sovereignty systems",
            self.esi.group_sovereignty().get_systems().await,
        );
        r.note(
            "industry systems",
            self.esi.group_industry().get_industry_systems().await,
        );
        r.note(
            "industry facilities",
            self.esi.group_industry().get_facilities().await,
        );
        let wars = r.note("wars", self.esi.group_wars().list(None).await);
        if let Some(id) = wars.and_then(|ids| ids.first().copied()) {
            r.note("war", self.esi.group_wars().get_war(id).await);
            let killmails = r.note(
                "war killmails",
                self.esi.group_wars().get_war_killmails(id, None).await,
            );
            if let Some(killmail) = killmails.and_then(|k| k.into_iter().next()) {
                r.note(
                    "killmail",
                    self.esi
                        .group_killmails()
                        .get_killmail(killmail.killmail_id, &killmail.killmail_hash)
                        .await,
                );
            }
        }
    }

    /// The groups whose answers are pages with a cursor.
    async fn paged(&mut self) {
        let r = &mut self.report;
        let campaigns = r.note(
            "military campaigns",
            self.esi.group_military_campaigns().list().await,
        );
        if let Some(campaign) = campaigns.and_then(|c| c.into_iter().next()) {
            r.note(
                "military campaign",
                self.esi
                    .group_military_campaigns()
                    .get_campaign(campaign.id)
                    .await,
            );
            let objectives = r.note(
                "campaign objectives",
                self.esi
                    .group_military_campaigns()
                    .list_objectives(campaign.id, None, None, Some(10))
                    .await,
            );
            if let Some(objective) = objectives.and_then(|p| p.items.into_iter().next()) {
                r.note(
                    "campaign objective",
                    self.esi
                        .group_military_campaigns()
                        .get_objective(campaign.id, objective.id)
                        .await,
                );
            }
        }
        let jobs = r.note(
            "freelance jobs",
            self.esi
                .group_freelance_jobs()
                .list(None, None, Some(10), None)
                .await,
        );
        if let Some(job) = jobs.and_then(|p| p.items.into_iter().next()) {
            r.note(
                "freelance job",
                self.esi.group_freelance_jobs().get_job(job.id).await,
            );
        }
        r.note(
            "paragon hub listings",
            self.esi
                .group_paragon_hub()
                .get_skinr_listings(None, None, Some(10))
                .await,
        );
    }
}

#[tokio::test]
#[ignore = "needs the live ESI; run it explicitly"]
async fn the_public_endpoints_read_what_esi_sends() {
    let mut smoke = Smoke::new().await;
    smoke.meta_and_status().await;
    smoke.people().await;
    smoke.market().await;
    smoke.universe().await;
    smoke.dogma().await;
    smoke.conflict().await;
    smoke.paged().await;

    smoke.report.print();
    let failures = smoke.report.failures();
    assert!(
        failures.is_empty(),
        "{} of {} steps failed:\n  {}",
        failures.len(),
        smoke.report.rows.len(),
        failures.join("\n  ")
    );
}
