//! Reads every response type of the crate from data generated out of the OpenAPI
//! fixture in `resources/test/openapi.json`.
//!
//! `conformance.rs` checks that the field *names* of a struct are in the
//! response schema of its operation. This test checks the *values*: for each
//! operation it builds JSON documents from the response schema and
//! deserializes them into the Rust type the crate returns. It catches a field
//! that is `i32` where the schema says `int64`, a field that is not an
//! `Option` where the schema does not require it or allows `null`, and an enum
//! that lacks a variant of the schema.
//!
//! Three kinds of document are built for each operation, once for every value
//! of the enums and `oneOf` choices it contains:
//! - `Full`: every property, with a value;
//! - `Required`: only the required properties;
//! - `Nulls`: every property, with `null` where the schema allows it.

use esi_openapi::groups::*;
use serde::de::DeserializeOwned;
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;
use std::fmt::Debug;

#[derive(Clone, Copy, Debug)]
enum Mode {
    Full,
    Required,
    Nulls,
}

struct Schemas {
    spec: Value,
}

impl Schemas {
    fn load() -> Self {
        let spec = serde_json::from_str(include_str!("../resources/test/openapi.json")).unwrap();
        Self { spec }
    }

    fn resolve<'a>(&'a self, node: &'a Value) -> &'a Value {
        match node.get("$ref").and_then(Value::as_str) {
            Some(reference) => {
                let mut current = &self.spec;
                for key in reference.trim_start_matches("#/").split('/') {
                    current = &current[key];
                }
                self.resolve(current)
            }
            None => node,
        }
    }

    /// The JSON schema of the successful response of an operation, if it has one.
    fn response_schema(&self, op_id: &str) -> Option<&Value> {
        for item in self.spec["paths"].as_object().unwrap().values() {
            for operation in item.as_object().unwrap().values() {
                if operation["operationId"] != op_id {
                    continue;
                }
                let (_, response) = operation["responses"]
                    .as_object()?
                    .iter()
                    .find(|(code, _)| code.starts_with('2'))?;
                let response = self.resolve(response);
                return response["content"]["application/json"].get("schema");
            }
        }
        panic!("{op_id} is not in the spec");
    }

    /// The type of a schema, ignoring `null`, and whether `null` is allowed.
    fn kind<'a>(&self, schema: &'a Value) -> (Option<&'a str>, bool) {
        match &schema["type"] {
            Value::String(kind) => (Some(kind.as_str()), kind == "null"),
            Value::Array(kinds) => {
                let nullable = kinds.iter().any(|kind| kind == "null");
                let kind = kinds
                    .iter()
                    .filter_map(Value::as_str)
                    .find(|k| *k != "null");
                (kind, nullable)
            }
            _ => (None, false),
        }
    }

    /// How many values the biggest enum or `oneOf` under a schema has.
    fn choices(&self, schema: &Value, stack: &mut Vec<String>) -> usize {
        let reference = schema.get("$ref").and_then(Value::as_str);
        if let Some(reference) = reference {
            if stack.iter().any(|seen| seen == reference) {
                return 1;
            }
            stack.push(reference.to_owned());
        }
        let node = self.resolve(schema);
        let mut most = 1;
        if let Some(values) = node["enum"].as_array() {
            most = most.max(values.len());
        }
        if let Some(variants) = node["oneOf"].as_array() {
            most = most.max(variants.len());
            for variant in variants {
                most = most.max(self.choices(variant, stack));
            }
        }
        if let Some(properties) = node["properties"].as_object() {
            for property in properties.values() {
                most = most.max(self.choices(property, stack));
            }
        }
        if let Some(items) = node.get("items") {
            most = most.max(self.choices(items, stack));
        }
        if let Some(extra) = node.get("additionalProperties") {
            most = most.max(self.choices(extra, stack));
        }
        if reference.is_some() {
            stack.pop();
        }
        most
    }

    /// A document that satisfies the schema; `choice` picks among enum values
    /// and `oneOf` variants.
    fn sample(&self, schema: &Value, choice: usize, mode: Mode, stack: &mut Vec<String>) -> Value {
        let reference = schema.get("$ref").and_then(Value::as_str);
        if let Some(reference) = reference {
            if stack.iter().any(|seen| seen == reference) {
                return Value::Null;
            }
            stack.push(reference.to_owned());
        }
        let node = self.resolve(schema);
        let value = self.sample_node(node, choice, mode, stack);
        if reference.is_some() {
            stack.pop();
        }
        value
    }

    fn sample_node(
        &self,
        node: &Value,
        choice: usize,
        mode: Mode,
        stack: &mut Vec<String>,
    ) -> Value {
        if let Some(values) = node["enum"].as_array() {
            return values[choice % values.len()].clone();
        }
        if let Some(variants) = node["oneOf"].as_array() {
            // A variant is an object with its one property, which the schema does
            // not list as required but the variant is meant to carry.
            return self.sample(
                &variants[choice % variants.len()],
                choice,
                Mode::Full,
                stack,
            );
        }
        let (kind, _) = self.kind(node);
        match kind {
            Some("integer") => json!(5_000_000_000_i64),
            Some("number") => json!(1.5),
            Some("boolean") => json!(true),
            Some("string") => match node["format"].as_str() {
                Some("date-time") => json!("2026-01-01T00:00:00Z"),
                Some("date") => json!("2026-01-01"),
                Some("uuid") => json!("00000000-0000-4000-8000-000000000000"),
                _ => json!("text"),
            },
            Some("array") => {
                let item = self.sample(&node["items"], choice, mode, stack);
                json!([item])
            }
            _ => self.sample_object(node, choice, mode, stack),
        }
    }

    fn sample_object(
        &self,
        node: &Value,
        choice: usize,
        mode: Mode,
        stack: &mut Vec<String>,
    ) -> Value {
        let required: BTreeSet<&str> = node["required"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect();
        let mut object = Map::new();
        for (name, property) in node["properties"].as_object().into_iter().flatten() {
            let resolved = self.resolve(property);
            let (_, nullable) = self.kind(resolved);
            let value = match mode {
                Mode::Required if !required.contains(name.as_str()) => continue,
                Mode::Nulls if nullable => Value::Null,
                _ => self.sample(property, choice, mode, stack),
            };
            object.insert(name.clone(), value);
        }
        if let Some(extra) = node.get("additionalProperties").filter(|v| v.is_object()) {
            object.insert("key".to_owned(), self.sample(extra, choice, mode, stack));
        }
        Value::Object(object)
    }

    /// Deserializes the documents of one operation into `T`. Returns the
    /// failure, if any: the first one found.
    fn check<T: DeserializeOwned + Debug>(&self, op_id: &str) -> Option<String> {
        self.check_schema::<T>(op_id, self.response_schema(op_id)?)
    }

    fn check_schema<T: DeserializeOwned + Debug>(
        &self,
        op_id: &str,
        schema: &Value,
    ) -> Option<String> {
        let documents = self.choices(schema, &mut Vec::new());
        for choice in 0..documents {
            for mode in [Mode::Full, Mode::Required, Mode::Nulls] {
                let sample = self.sample(schema, choice, mode, &mut Vec::new());
                let text = sample.to_string();
                let shown: String = text.chars().take(160).collect();
                match serde_json::from_value::<T>(sample) {
                    Err(error) => {
                        return Some(format!(
                            "{op_id} as {} [{mode:?}, choice {choice}]: {error}\n      {shown}",
                            std::any::type_name::<T>()
                        ));
                    }
                    // The enums fall back to `Unrecognized` for a value they do not
                    // list, which hides a value the schema has.
                    Ok(value) if format!("{value:?}").contains("Unrecognized") => {
                        return Some(format!(
                            "{op_id} as {} [{mode:?}, choice {choice}]: a value of the schema reads as Unrecognized\n      {shown}",
                            std::any::type_name::<T>()
                        ));
                    }
                    Ok(_) => {}
                }
            }
        }
        None
    }

    /// Like `check`, for an operation whose response wraps the list the crate
    /// returns in a private struct: the schema of `field` is read into `T`.
    fn check_field<T: DeserializeOwned + Debug>(&self, op_id: &str, field: &str) -> Option<String> {
        let schema = &self.resolve(self.response_schema(op_id)?)["properties"][field];
        assert!(schema.is_object(), "{op_id} has no property {field}");
        self.check_schema::<T>(op_id, schema)
    }

    fn failures(&self) -> Vec<String> {
        let schemas = self;
        let mut failures: Vec<String> = Vec::new();
        failures.extend(schemas.check::<Vec<i64>>("GetAlliances"));
        failures.extend(schemas.check::<AllianceInfo>("GetAlliancesAllianceId"));
        failures.extend(schemas.check::<Vec<Contact>>("GetAlliancesAllianceIdContacts"));
        failures.extend(schemas.check::<Vec<ContactLabel>>("GetAlliancesAllianceIdContactsLabels"));
        failures.extend(schemas.check::<Vec<i64>>("GetAlliancesAllianceIdCorporations"));
        failures.extend(schemas.check::<AllianceIcons>("GetAlliancesAllianceIdIcons"));
        failures.extend(schemas.check::<AccessList>("GetCharactersAccessListsDetail"));
        failures.extend(schemas.check_field::<Vec<AccessListSummary>>(
            "GetCharactersAccessListsListing",
            "access_lists",
        ));
        failures
            .extend(schemas.check::<Vec<ResearchAgent>>("GetCharactersCharacterIdAgentsResearch"));
        failures.extend(schemas.check::<Vec<Asset>>("GetCharactersCharacterIdAssets"));
        failures.extend(schemas.check::<CharacterAttributes>("GetCharactersCharacterIdAttributes"));
        failures.extend(schemas.check::<Vec<Blueprint>>("GetCharactersCharacterIdBlueprints"));
        failures
            .extend(schemas.check::<Vec<CalendarEventSummary>>("GetCharactersCharacterIdCalendar"));
        failures.extend(schemas.check::<CalendarEvent>("GetCharactersCharacterIdCalendarEventId"));
        failures.extend(
            schemas.check::<Vec<EventAttendee>>("GetCharactersCharacterIdCalendarEventIdAttendees"),
        );
        failures.extend(schemas.check::<Clones>("GetCharactersCharacterIdClones"));
        failures.extend(schemas.check::<Vec<Contact>>("GetCharactersCharacterIdContacts"));
        failures
            .extend(schemas.check::<Vec<ContactLabel>>("GetCharactersCharacterIdContactsLabels"));
        failures.extend(schemas.check::<Vec<Contract>>("GetCharactersCharacterIdContracts"));
        failures.extend(
            schemas.check::<Vec<ContractBid>>("GetCharactersCharacterIdContractsContractIdBids"),
        );
        failures.extend(
            schemas.check::<Vec<ContractItem>>("GetCharactersCharacterIdContractsContractIdItems"),
        );
        failures.extend(schemas.check::<Vec<CharacterCorporationHistoryItem>>(
            "GetCharactersCharacterIdCorporationhistory",
        ));
        failures.extend(schemas.check::<Fatigue>("GetCharactersCharacterIdFatigue"));
        failures.extend(schemas.check::<Vec<Fitting>>("GetCharactersCharacterIdFittings"));
        failures.extend(schemas.check::<CharacterFleet>("GetCharactersCharacterIdFleet"));
        failures.extend(schemas.check::<CharacterFwStats>("GetCharactersCharacterIdFwStats"));
        failures.extend(schemas.check::<Vec<i64>>("GetCharactersCharacterIdImplants"));
        failures.extend(schemas.check::<Vec<IndustryJob>>("GetCharactersCharacterIdIndustryJobs"));
        failures.extend(
            schemas.check::<Vec<RecentKillMail>>("GetCharactersCharacterIdKillmailsRecent"),
        );
        failures.extend(schemas.check::<LocationInfo>("GetCharactersCharacterIdLocation"));
        failures
            .extend(schemas.check::<Vec<LoyaltyPoints>>("GetCharactersCharacterIdLoyaltyPoints"));
        failures.extend(schemas.check::<Vec<MailHeader>>("GetCharactersCharacterIdMail"));
        failures.extend(schemas.check::<MailLabels>("GetCharactersCharacterIdMailLabels"));
        failures.extend(schemas.check::<Vec<MailingList>>("GetCharactersCharacterIdMailLists"));
        failures.extend(schemas.check::<Mail>("GetCharactersCharacterIdMailMailId"));
        failures.extend(schemas.check::<Vec<Medal>>("GetCharactersCharacterIdMedals"));
        failures.extend(schemas.check::<Vec<MiningEntry>>("GetCharactersCharacterIdMining"));
        failures
            .extend(schemas.check::<Vec<Notification>>("GetCharactersCharacterIdNotifications"));
        failures
            .extend(schemas.check::<Vec<ContactNotification>>(
                "GetCharactersCharacterIdNotificationsContacts",
            ));
        failures.extend(schemas.check::<OnlineStatus>("GetCharactersCharacterIdOnline"));
        failures.extend(schemas.check::<Vec<CharacterOrder>>("GetCharactersCharacterIdOrders"));
        failures.extend(
            schemas.check::<Vec<CharacterOrderHistory>>("GetCharactersCharacterIdOrdersHistory"),
        );
        failures.extend(schemas.check::<Vec<CharacterPlanet>>("GetCharactersCharacterIdPlanets"));
        failures.extend(schemas.check::<Colony>("GetCharactersCharacterIdPlanetsPlanetId"));
        failures.extend(schemas.check::<CharacterPortraitInfo>("GetCharactersCharacterIdPortrait"));
        failures.extend(schemas.check::<CharacterRoles>("GetCharactersCharacterIdRoles"));
        failures.extend(schemas.check::<SearchResult>("GetCharactersCharacterIdSearch"));
        failures.extend(schemas.check::<CurrentShip>("GetCharactersCharacterIdShip"));
        failures.extend(schemas.check::<Vec<SkillQueueItem>>("GetCharactersCharacterIdSkillqueue"));
        failures.extend(schemas.check::<Skills>("GetCharactersCharacterIdSkills"));
        failures.extend(schemas.check::<Vec<Standing>>("GetCharactersCharacterIdStandings"));
        failures.extend(schemas.check::<Vec<CharacterTitle>>("GetCharactersCharacterIdTitles"));
        failures.extend(schemas.check::<f64>("GetCharactersCharacterIdWallet"));
        failures
            .extend(schemas.check::<Vec<JournalEntry>>("GetCharactersCharacterIdWalletJournal"));
        failures.extend(
            schemas.check::<Vec<WalletTransaction>>("GetCharactersCharacterIdWalletTransactions"),
        );
        failures.extend(
            schemas.check_field::<Vec<CharacterSkinrLicense>>(
                "GetCharactersCosmeticsSkinr",
                "licenses",
            ),
        );
        failures.extend(schemas.check_field::<Vec<SkinrComponentLicense>>(
            "GetCharactersCosmeticsSkinrComponents",
            "licenses",
        ));
        failures.extend(schemas.check::<CharacterPublicInfo>("GetCharactersDetail"));
        failures.extend(schemas.check_field::<Vec<FreelanceJobSummary>>(
            "GetCharactersFreelanceJobsListing",
            "freelance_jobs",
        ));
        failures
            .extend(schemas.check::<JobParticipation>("GetCharactersFreelanceJobsParticipation"));
        failures.extend(
            schemas.check::<TacticalOperation>("GetCharactersMercenaryTacticalOperationsDetail"),
        );
        failures.extend(schemas.check_field::<Vec<TacticalOperationSummary>>(
            "GetCharactersMercenaryTacticalOperationsListing",
            "operations",
        ));
        failures.extend(
            schemas.check::<Page<CharacterObjective>>(
                "GetCharactersMilitaryCampaignsObjectivesListing",
            ),
        );
        failures.extend(
            schemas.check::<CharacterObjective>(
                "GetCharactersMilitaryCampaignsObjectivesParticipation",
            ),
        );
        failures
            .extend(schemas.check::<Page<CharacterSkinrListing>>("GetCharactersParagonHubSkinr"));
        failures
            .extend(schemas.check::<MercenaryDen>("GetCharactersStructuresMercenaryDensDetail"));
        failures.extend(schemas.check_field::<Vec<MercenaryDenSummary>>(
            "GetCharactersStructuresMercenaryDensListing",
            "mercenary_dens",
        ));
        failures
            .extend(schemas.check::<Vec<PublicContractBid>>("GetContractsPublicBidsContractId"));
        failures
            .extend(schemas.check::<Vec<PublicContractItem>>("GetContractsPublicItemsContractId"));
        failures.extend(schemas.check::<Vec<PublicContract>>("GetContractsPublicRegionId"));
        failures.extend(
            schemas.check::<Vec<MiningExtraction>>("GetCorporationCorporationIdMiningExtractions"),
        );
        failures.extend(
            schemas.check::<Vec<MiningObserver>>("GetCorporationCorporationIdMiningObservers"),
        );
        failures.extend(
            schemas.check::<Vec<ObservedMining>>(
                "GetCorporationCorporationIdMiningObserversObserverId",
            ),
        );
        failures.extend(schemas.check::<CorporationPublicInfo>("GetCorporationsCorporationId"));
        failures.extend(
            schemas.check::<Vec<CorporationHistoryItem>>(
                "GetCorporationsCorporationIdAlliancehistory",
            ),
        );
        failures.extend(schemas.check::<Vec<Asset>>("GetCorporationsCorporationIdAssets"));
        failures.extend(schemas.check::<Vec<Blueprint>>("GetCorporationsCorporationIdBlueprints"));
        failures.extend(schemas.check::<Vec<Contact>>("GetCorporationsCorporationIdContacts"));
        failures.extend(
            schemas.check::<Vec<ContactLabel>>("GetCorporationsCorporationIdContactsLabels"),
        );
        failures.extend(
            schemas.check::<Vec<ContainerLogEntry>>("GetCorporationsCorporationIdContainersLogs"),
        );
        failures.extend(schemas.check::<Vec<Contract>>("GetCorporationsCorporationIdContracts"));
        failures.extend(
            schemas
                .check::<Vec<ContractBid>>("GetCorporationsCorporationIdContractsContractIdBids"),
        );
        failures
            .extend(schemas.check::<Vec<ContractItem>>(
                "GetCorporationsCorporationIdContractsContractIdItems",
            ));
        failures.extend(
            schemas.check::<Vec<CustomsOffice>>("GetCorporationsCorporationIdCustomsOffices"),
        );
        failures
            .extend(schemas.check::<CorporationDivisions>("GetCorporationsCorporationIdDivisions"));
        failures.extend(
            schemas.check::<Vec<CorporationFacility>>("GetCorporationsCorporationIdFacilities"),
        );
        failures.extend(schemas.check::<CorporationFwStats>("GetCorporationsCorporationIdFwStats"));
        failures.extend(schemas.check::<CorporationIcons>("GetCorporationsCorporationIdIcons"));
        failures.extend(
            schemas
                .check::<Vec<CorporationIndustryJob>>("GetCorporationsCorporationIdIndustryJobs"),
        );
        failures.extend(
            schemas.check::<Vec<RecentKillMail>>("GetCorporationsCorporationIdKillmailsRecent"),
        );
        failures
            .extend(schemas.check::<Vec<CorporationMedal>>("GetCorporationsCorporationIdMedals"));
        failures
            .extend(schemas.check::<Vec<IssuedMedal>>("GetCorporationsCorporationIdMedalsIssued"));
        failures.extend(schemas.check::<Vec<i64>>("GetCorporationsCorporationIdMembers"));
        failures.extend(schemas.check::<i64>("GetCorporationsCorporationIdMembersLimit"));
        failures.extend(
            schemas.check::<Vec<MemberTitles>>("GetCorporationsCorporationIdMembersTitles"),
        );
        failures.extend(
            schemas.check::<Vec<MemberTracking>>("GetCorporationsCorporationIdMembertracking"),
        );
        failures
            .extend(schemas.check::<Vec<CorporationOrder>>("GetCorporationsCorporationIdOrders"));
        failures
            .extend(schemas.check::<Vec<CorporationOrderHistory>>(
                "GetCorporationsCorporationIdOrdersHistory",
            ));
        failures.extend(schemas.check::<Vec<MemberRoles>>("GetCorporationsCorporationIdRoles"));
        failures.extend(
            schemas.check::<Vec<RoleHistoryEntry>>("GetCorporationsCorporationIdRolesHistory"),
        );
        failures
            .extend(schemas.check::<Vec<Shareholder>>("GetCorporationsCorporationIdShareholders"));
        failures.extend(schemas.check::<Vec<Standing>>("GetCorporationsCorporationIdStandings"));
        failures.extend(schemas.check::<Vec<Starbase>>("GetCorporationsCorporationIdStarbases"));
        failures.extend(
            schemas.check::<StarbaseDetails>("GetCorporationsCorporationIdStarbasesStarbaseId"),
        );
        failures.extend(
            schemas.check::<Vec<CorporationStructure>>("GetCorporationsCorporationIdStructures"),
        );
        failures
            .extend(schemas.check::<Vec<CorporationTitle>>("GetCorporationsCorporationIdTitles"));
        failures
            .extend(schemas.check::<Vec<DivisionBalance>>("GetCorporationsCorporationIdWallets"));
        failures.extend(
            schemas
                .check::<Vec<JournalEntry>>("GetCorporationsCorporationIdWalletsDivisionJournal"),
        );
        failures.extend(schemas.check::<Vec<CorporationWalletTransaction>>(
            "GetCorporationsCorporationIdWalletsDivisionTransactions",
        ));
        failures.extend(
            schemas.check::<Page<FreelanceJobSummary>>("GetCorporationsFreelanceJobsListing"),
        );
        failures.extend(
            schemas.check::<Page<JobParticipant>>("GetCorporationsFreelanceJobsParticipants"),
        );
        failures.extend(schemas.check::<Vec<i64>>("GetCorporationsNpccorps"));
        failures
            .extend(schemas.check::<ProjectContribution>("GetCorporationsProjectsContribution"));
        failures.extend(
            schemas.check::<Page<ProjectContributor>>("GetCorporationsProjectsContributors"),
        );
        failures.extend(schemas.check::<Project>("GetCorporationsProjectsDetail"));
        failures.extend(schemas.check::<Page<ProjectSummary>>("GetCorporationsProjectsListing"));
        failures.extend(schemas.check::<Skyhook>("GetCorporationsStructuresSkyhooksDetail"));
        failures.extend(schemas.check_field::<Vec<SkyhookSummary>>(
            "GetCorporationsStructuresSkyhooksListing",
            "skyhooks",
        ));
        failures.extend(
            schemas.check::<CorporationSovereigntyHub>(
                "GetCorporationsStructuresSovereigntyHubsDetail",
            ),
        );
        failures.extend(schemas.check_field::<Vec<SovereigntyHubSummary>>(
            "GetCorporationsStructuresSovereigntyHubsListing",
            "sovereignty_hubs",
        ));
        failures.extend(schemas.check::<Skinr>("GetCosmeticsSkinr"));
        failures.extend(schemas.check::<Vec<i64>>("GetDogmaAttributes"));
        failures.extend(schemas.check::<DogmaAttribute>("GetDogmaAttributesAttributeId"));
        failures.extend(schemas.check::<DynamicItem>("GetDogmaDynamicItemsTypeIdItemId"));
        failures.extend(schemas.check::<Vec<i64>>("GetDogmaEffects"));
        failures.extend(schemas.check::<DogmaEffect>("GetDogmaEffectsEffectId"));
        failures.extend(schemas.check::<FleetInfo>("GetFleetsFleetId"));
        failures.extend(schemas.check::<Vec<FleetMember>>("GetFleetsFleetIdMembers"));
        failures.extend(schemas.check::<Vec<FleetWing>>("GetFleetsFleetIdWings"));
        failures.extend(schemas.check::<FreelanceJob>("GetFreelanceJobsDetail"));
        failures.extend(schemas.check::<Page<FreelanceJobSummary>>("GetFreelanceJobsListing"));
        failures.extend(schemas.check::<FWLeaderboards>("GetFwLeaderboards"));
        failures.extend(schemas.check::<FWCharacterLeaderboards>("GetFwLeaderboardsCharacters"));
        failures
            .extend(schemas.check::<FWCorporationLeaderboards>("GetFwLeaderboardsCorporations"));
        failures.extend(schemas.check::<Vec<FWStatsItem>>("GetFwStats"));
        failures.extend(schemas.check::<Vec<FWSystem>>("GetFwSystems"));
        failures.extend(schemas.check::<Vec<FWWar>>("GetFwWars"));
        failures.extend(schemas.check::<Vec<Incursion>>("GetIncursions"));
        failures.extend(schemas.check::<Vec<IndustryFacility>>("GetIndustryFacilities"));
        failures.extend(schemas.check::<Vec<IndustrialSystem>>("GetIndustrySystems"));
        failures.extend(schemas.check::<Vec<InsurancePrice>>("GetInsurancePrices"));
        failures.extend(schemas.check::<Killmail>("GetKillmailsKillmailIdKillmailHash"));
        failures.extend(schemas.check::<Vec<LoyaltyOffer>>("GetLoyaltyStoresCorporationIdOffers"));
        failures.extend(schemas.check::<Vec<i64>>("GetMarketsGroups"));
        failures.extend(schemas.check::<MarketGroupInfo>("GetMarketsGroupsMarketGroupId"));
        failures.extend(schemas.check::<Vec<PriceItem>>("GetMarketsPrices"));
        failures.extend(schemas.check::<Vec<HistoryItem>>("GetMarketsRegionIdHistory"));
        failures.extend(schemas.check::<Vec<MarketOrder>>("GetMarketsRegionIdOrders"));
        failures.extend(schemas.check::<Vec<i64>>("GetMarketsRegionIdTypes"));
        failures.extend(schemas.check::<Vec<StructureOrder>>("GetMarketsStructuresStructureId"));
        failures.extend(schemas.check::<Changelog>("GetMetaChangelog"));
        failures.extend(schemas.check::<CompatibilityDates>("GetMetaCompatibilityDates"));
        failures.extend(schemas.check::<ApiName>("GetMetaName"));
        failures.extend(schemas.check::<ApiStatus>("GetMetaStatus"));
        failures.extend(schemas.check::<Campaign>("GetMilitaryCampaignsDetail"));
        failures.extend(
            schemas.check_field::<Vec<Campaign>>("GetMilitaryCampaignsListing", "campaigns"),
        );
        failures.extend(schemas.check::<Objective>("GetMilitaryCampaignsObjectivesDetail"));
        failures.extend(schemas.check::<Page<Objective>>("GetMilitaryCampaignsObjectivesListing"));
        failures.extend(schemas.check::<Page<SkinrListing>>("GetParagonHubSkinr"));
        failures.extend(schemas.check::<Page<SkinrListing>>("GetParagonHubSkinrAlliances"));
        failures.extend(schemas.check::<Page<SkinrListing>>("GetParagonHubSkinrCharacters"));
        failures.extend(schemas.check::<Page<SkinrListing>>("GetParagonHubSkinrCorporations"));
        failures
            .extend(schemas.check_field::<Vec<RaidableSkyhook>>("GetSkyhooksRaidable", "skyhooks"));
        failures.extend(schemas.check::<Vec<SovereigntyCampaign>>("GetSovereigntyCampaigns"));
        failures.extend(
            schemas.check_field::<Vec<SovereigntySystem>>("GetSovereigntySystems", "solar_systems"),
        );
        failures.extend(schemas.check::<ServerStatus>("GetStatus"));
        failures.extend(schemas.check::<Vec<Ancestry>>("GetUniverseAncestries"));
        failures.extend(schemas.check::<AsteroidBelt>("GetUniverseAsteroidBeltsAsteroidBeltId"));
        failures.extend(schemas.check::<Vec<Bloodline>>("GetUniverseBloodlines"));
        failures.extend(schemas.check::<Vec<i64>>("GetUniverseCategories"));
        failures.extend(schemas.check::<CategoriesCategory>("GetUniverseCategoriesCategoryId"));
        failures.extend(schemas.check::<Vec<i64>>("GetUniverseConstellations"));
        failures.extend(schemas.check::<Constellation>("GetUniverseConstellationsConstellationId"));
        failures.extend(schemas.check::<Vec<UniverseFaction>>("GetUniverseFactions"));
        failures.extend(schemas.check::<Vec<i64>>("GetUniverseGraphics"));
        failures.extend(schemas.check::<Graphic>("GetUniverseGraphicsGraphicId"));
        failures.extend(schemas.check::<Vec<i64>>("GetUniverseGroups"));
        failures.extend(schemas.check::<Group>("GetUniverseGroupsGroupId"));
        failures.extend(schemas.check::<Moon>("GetUniverseMoonsMoonId"));
        failures.extend(schemas.check::<PlanetInfo>("GetUniversePlanetsPlanetId"));
        failures.extend(schemas.check::<Vec<Race>>("GetUniverseRaces"));
        failures.extend(schemas.check::<Vec<i64>>("GetUniverseRegions"));
        failures.extend(schemas.check::<Region>("GetUniverseRegionsRegionId"));
        failures.extend(schemas.check::<Schematic>("GetUniverseSchematicsSchematicId"));
        failures.extend(schemas.check::<Stargate>("GetUniverseStargatesStargateId"));
        failures.extend(schemas.check::<Star>("GetUniverseStarsStarId"));
        failures.extend(schemas.check::<Station>("GetUniverseStationsStationId"));
        failures.extend(schemas.check::<Vec<i64>>("GetUniverseStructures"));
        failures.extend(schemas.check::<Structure>("GetUniverseStructuresStructureId"));
        failures.extend(schemas.check::<Vec<SystemJumps>>("GetUniverseSystemJumps"));
        failures.extend(schemas.check::<Vec<SystemKills>>("GetUniverseSystemKills"));
        failures.extend(schemas.check::<Vec<i64>>("GetUniverseSystems"));
        failures.extend(schemas.check::<System>("GetUniverseSystemsSystemId"));
        failures.extend(schemas.check::<Vec<i64>>("GetUniverseTypes"));
        failures.extend(schemas.check::<Type>("GetUniverseTypesTypeId"));
        failures.extend(schemas.check::<Vec<i64>>("GetWars"));
        failures.extend(schemas.check::<War>("GetWarsWarId"));
        failures.extend(schemas.check::<Vec<WarKillmail>>("GetWarsWarIdKillmails"));
        failures.extend(schemas.check::<Vec<CharacterAffiliation>>("PostCharactersAffiliation"));
        failures.extend(
            schemas.check::<Vec<AssetLocation>>("PostCharactersCharacterIdAssetsLocations"),
        );
        failures.extend(schemas.check::<Vec<AssetName>>("PostCharactersCharacterIdAssetsNames"));
        failures.extend(schemas.check::<Vec<i64>>("PostCharactersCharacterIdContacts"));
        failures.extend(schemas.check::<f64>("PostCharactersCharacterIdCspa"));
        failures
            .extend(schemas.check_field::<i64>("PostCharactersCharacterIdFittings", "fitting_id"));
        failures.extend(schemas.check::<i64>("PostCharactersCharacterIdMail"));
        failures.extend(schemas.check::<i64>("PostCharactersCharacterIdMailLabels"));
        failures.extend(
            schemas.check::<Vec<AssetLocation>>("PostCorporationsCorporationIdAssetsLocations"),
        );
        failures
            .extend(schemas.check::<Vec<AssetName>>("PostCorporationsCorporationIdAssetsNames"));
        failures.extend(schemas.check::<CreatedWing>("PostFleetsFleetIdWings"));
        failures.extend(schemas.check::<CreatedSquad>("PostFleetsFleetIdWingsWingIdSquads"));
        failures.extend(schemas.check::<Route>("PostRoute"));
        failures.extend(schemas.check::<Ids>("PostUniverseIds"));
        failures.extend(schemas.check::<Vec<ResolvedName>>("PostUniverseNames"));

        failures
    }
}

#[test]
fn every_response_type_reads_the_documents_the_schema_describes() {
    let failures = Schemas::load().failures();

    assert!(
        failures.is_empty(),
        "{} operations do not read what their schema allows:\n  {}",
        failures.len(),
        failures.join("\n  ")
    );
}

#[test]
fn every_operation_with_a_json_response_is_in_the_list() {
    let schemas = Schemas::load();
    let listed = include_str!("schema_values.rs");
    let mut missing = Vec::new();
    for item in schemas.spec["paths"].as_object().unwrap().values() {
        for operation in item.as_object().unwrap().values() {
            let Some(op_id) = operation["operationId"].as_str() else {
                continue;
            };
            if schemas.response_schema(op_id).is_some() && !listed.contains(&format!("\"{op_id}\""))
            {
                missing.push(op_id.to_owned());
            }
        }
    }

    assert!(
        missing.is_empty(),
        "add these operations to `failures`: {missing:?}"
    );
}

#[test]
fn the_documents_built_from_a_schema_follow_it() {
    let schemas = Schemas::load();
    let schema = json!({
        "type": "object",
        "required": ["id"],
        "properties": {
            "id": {"type": "integer", "format": "int64"},
            "kind": {"type": "string", "enum": ["a", "b", "c"]},
            "note": {"type": ["string", "null"]},
            "price": {"oneOf": [
                {"type": "object", "properties": {"isk": {"type": "number"}}},
                {"type": "object", "properties": {"plex": {"type": "integer"}}}
            ]}
        }
    });

    assert_eq!(schemas.choices(&schema, &mut Vec::new()), 3);
    let full = schemas.sample(&schema, 1, Mode::Full, &mut Vec::new());
    assert_eq!(
        full,
        json!({"id": 5_000_000_000_i64, "kind": "b", "note": "text", "price": {"plex": 5_000_000_000_i64}})
    );
    let required = schemas.sample(&schema, 0, Mode::Required, &mut Vec::new());
    assert_eq!(required, json!({"id": 5_000_000_000_i64}));
    let nulls = schemas.sample(&schema, 0, Mode::Nulls, &mut Vec::new());
    assert_eq!(nulls["note"], Value::Null);
    assert_eq!(nulls["kind"], "a");
}
