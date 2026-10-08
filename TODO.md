# TODO: parity with the published ESI API

Baseline: published OpenAPI spec, compatibility date `2026-08-18` (the latest listed by `/meta/compatibility-dates`). `resources/test/openapi.json` is byte-identical to the live spec, so no fixture refresh is needed today.

Coverage: **233 of 233 operations implemented (0 missing)**. All operation IDs used in `src/groups` exist in the spec.

## Summary by group

| Tag | Module | Done | Total | Status |
| --- | --- | --- | --- | --- |
| Access List | `access_list.rs` | 2 | 2 | done |
| Activities | `activities.rs` | 3 | 3 | done |
| Alliance | `alliance.rs` | 4 | 4 | done |
| Assets | `assets.rs` | 6 | 6 | done |
| Calendar | `calendar.rs` | 4 | 4 | done |
| Character | `character.rs` | 14 | 14 | done |
| Clones | `clones.rs` | 2 | 2 | done |
| Contacts | `contacts.rs` | 9 | 9 | done |
| Contracts | `contracts.rs` | 9 | 9 | done |
| Corporation | `corporation.rs` | 22 | 22 | done |
| Corporation Projects | `corporation_projects.rs` | 4 | 4 | done |
| Cosmetics | `cosmetics.rs` | 3 | 3 | done |
| Dogma | `dogma.rs` | 5 | 5 | done |
| Faction Warfare | `faction_warfare.rs` | 8 | 8 | done |
| Fittings | `fittings.rs` | 3 | 3 | done |
| Fleets | `fleets.rs` | 14 | 14 | done |
| Freelance Jobs | `freelance_jobs.rs` | 6 | 6 | done |
| Incursions | `incursions.rs` | 1 | 1 | done |
| Industry | `industry.rs` | 8 | 8 | done |
| Insurance | `insurance.rs` | 1 | 1 | done |
| Killmails | `killmails.rs` | 3 | 3 | done |
| Location | `location.rs` | 3 | 3 | done |
| Loyalty | `loyalty.rs` | 2 | 2 | done |
| Mail | `mail.rs` | 9 | 9 | done |
| Market | `market.rs` | 11 | 11 | done |
| Meta | `meta.rs` | 4 | 4 | done |
| Military Campaigns | `military_campaigns.rs` | 6 | 6 | done |
| Paragon Hub | `paragon_hub.rs` | 5 | 5 | done |
| Planetary Interaction | `planetary_interaction.rs` | 4 | 4 | done |
| Routes | `routes.rs` | 1 | 1 | done |
| Search | `search.rs` | 1 | 1 | done |
| Skills | `skills.rs` | 3 | 3 | done |
| Sovereignty | `sovereignty.rs` | 2 | 2 | done |
| Status | `status.rs` | 1 | 1 | done |
| Structures | `structures.rs` | 6 | 6 | done |
| Universe | `universe.rs` | 30 | 30 | done |
| User Interface | `user_interface.rs` | 5 | 5 | done |
| Wallet | `wallet.rs` | 6 | 6 | done |
| Wars | `wars.rs` | 3 | 3 | done |

Modules in `src/groups` with no matching tag in the spec: `bookmarks`, `opportunities` (decide: remove, or keep as empty groups for API stability).

## Conventions used for new code

- Integers: any value the spec defines as `integer` with `format: int64` is `i64`; `page` (the only `int32` in the spec) is `i32`. Never `u32`/`u64`.
- Enums in the spec are Rust enums with an `Unrecognized` variant (`#[serde(other)]`) so a new value from ESI does not fail parsing.
- UUIDs are `uuid::Uuid` (the `uuid` crate is a regular dependency).
- Free-form values with a very large set of variants (e.g. a star's `spectral_class`, about 90 values) stay `String`; the allowed values are documented on the field.
- Date-times stay `String`, like the rest of the crate.
- Every `character_id` (path parameters and struct fields) is `i64`, as are `completed_character_id` and the `character_ids` body of `CharacterGroup::get_affiliation`. Other character-valued IDs with a different name (`sender_id`, `client_id`, `owner_id`...) were not reviewed.
- Large enums: corporation roles are a Rust enum; the wallet journal `ref_type` (162 values) stays `String`.
- Live tests in `tests/authenticated.rs` fail on a `403`, so a refresh token obtained before the new scopes were added to `ESI_SCOPES` has to be regenerated (`auth_get_refresh_token`).
- No free functions; helpers live in `impl` blocks.

## Phase 0: infrastructure (do first, everything else depends on it)

- [x] `api_put!` and `api_delete!` macros (added). Tagged query parameters (`Required`/`Optional`/`Many`/`OptionalMany`) are supported in all four macros.
- [x] Shared request parameters: `EsiBuilder::language` (`Accept-Language`) and `EsiBuilder::tenant` (`X-Tenant`) are sent on every request and are part of the cache key; `If-None-Match` / `If-Modified-Since` are sent by the response cache; `X-Compatibility-Date` was already sent.
- [x] Conditional requests: opt-in response cache (`EsiBuilder::enable_cache`, `src/cache.rs`) that stores `ETag`/`Last-Modified` per token+URL+query, sends `If-None-Match`/`If-Modified-Since`, reuses the body on `304 Not Modified` (transparent: no separate result type) and honors `x-client-cache-ttl` (falls back to `Cache-Control: max-age`).
- [x] Pagination helpers in `client.rs`: `Esi::query_with_pages` (returns `X-Pages`), `Esi::fetch_all_pages` (`page` parameter) and `Esi::fetch_all_cursor` (the 12 `x-pagination: cursor` operations).
- [x] `Spec` / `SpecPathMethod` now read `security` (scopes), `x-rate-limit`, `x-required-roles`, `x-pagination`, `x-tombstone-ttl` and `x-client-cache-ttl`. Helpers: `Esi::required_scopes`, `Esi::missing_scopes` (fail early), `Esi::required_roles`, `Esi::declared_rate_limits` (static budgets per group; live values stay in `rate_limit_status`).
- [x] `x-tombstone-ttl` operations (5): `EsiError::Gone { status, tombstone_ttl_secs }` for HTTP 404/410 on those operations. The spec does not say which status ESI uses, so both are accepted; confirm against the live API.
- [x] Scope list: `Spec::oauth_scopes` and `Spec::scope_string_for`; `tests/conformance.rs` checks `ESI_SCOPES` in `.env.example` against the spec and that it covers every scope an operation needs.
- [x] Conformance test (`tests/conformance.rs`): for every implemented operation, check method, path placeholders, query keys and the struct fields against the spec (`components/schemas`, 349 schemas). Earlier fixes (`Killmail`, `MailLabels`, `Clones`) show hand-written structs drift; this test would have caught them. Consider generating structs from the schemas instead of writing them by hand.
- [x] Coverage test (`tests/conformance.rs`): fail CI when an operation in the fixture has no implementation, using an explicit allow-list for operations deliberately skipped.
- [x] Scheduled CI job (`.github/workflows/spec-diff.yml`, weekly and manual) diffs the live spec against `resources/test/openapi.json` and opens or updates an "ESI spec drift" issue when they differ or a newer compatibility date appears.
- [x] Live tests in `tests/authenticated.rs` for the authenticated read endpoints that need no ID from an earlier call (character and corporation checks); detail endpoints and write calls are not covered. `tests/conformance.rs` and `tests/cache.rs` cover the rest without network.

- [x] Empty successful responses (e.g. `204 No Content`) are read as JSON `null`, so `()` return types work for write endpoints.

## Suggested order

1. **Phase 0** (above).
2. **Public endpoints**, no auth needed: Status, Meta, Dogma, Wars, Sovereignty, Insurance, Routes, Freelance Jobs, Military Campaigns, remaining Universe, Market (groups, types), Contracts (public), Loyalty (offers), Industry (facilities), Paragon Hub and Cosmetics (public ones).
3. **Authenticated character endpoints**: Skills, Character, Mail, Contacts, Calendar, Fittings, Fleets, Contracts, Planetary Interaction, Wallet, Loyalty, Access List, Activities.
4. **Corporation / alliance endpoints** (many need `roles`): Corporation, Structures, Corporation Projects, Industry, Killmails, Market orders, Wallet.
5. Write endpoints (PUT/POST/DELETE) last in each group, since they need the Phase 0 macros and are the riskiest to test live.

## Operations by group

Legend: `[x]` implemented. Scope in parentheses; `roles` = `x-required-roles`; `paged` = `x-pagination`; `tombstone` = `x-tombstone-ttl`. Order within a group follows the path.

### Access List (2/2) → `src/groups/access_list.rs`
- [x] `GET /characters/{character_id}/access-lists` — `GetCharactersAccessListsListing` (esi-access.read_lists.v1)
- [x] `GET /characters/{character_id}/access-lists/{access_list_id}` — `GetCharactersAccessListsDetail` (esi-access.read_lists.v1)

### Activities (3/3) → `src/groups/activities.rs`
- [x] `GET /characters/{character_id}/mercenary-tactical-operations` — `GetCharactersMercenaryTacticalOperationsListing` (esi-activities.read_character.v1)
- [x] `GET /characters/{character_id}/mercenary-tactical-operations/{operation_id}` — `GetCharactersMercenaryTacticalOperationsDetail` (esi-activities.read_character.v1)
- [x] `GET /skyhooks/raidable` — `GetSkyhooksRaidable`

### Alliance (4/4) → `src/groups/alliance.rs`
- [x] `GET /alliances` — `GetAlliances`
- [x] `GET /alliances/{alliance_id}` — `GetAlliancesAllianceId`
- [x] `GET /alliances/{alliance_id}/corporations` — `GetAlliancesAllianceIdCorporations`
- [x] `GET /alliances/{alliance_id}/icons` — `GetAlliancesAllianceIdIcons`

### Assets (6/6) → `src/groups/assets.rs`
- [x] `GET /characters/{character_id}/assets` — `GetCharactersCharacterIdAssets` (esi-assets.read_assets.v1)
- [x] `POST /characters/{character_id}/assets/locations` — `PostCharactersCharacterIdAssetsLocations` (esi-assets.read_assets.v1)
- [x] `POST /characters/{character_id}/assets/names` — `PostCharactersCharacterIdAssetsNames` (esi-assets.read_assets.v1)
- [x] `GET /corporations/{corporation_id}/assets` — `GetCorporationsCorporationIdAssets` (esi-assets.read_corporation_assets.v1; roles: Director)
- [x] `POST /corporations/{corporation_id}/assets/locations` — `PostCorporationsCorporationIdAssetsLocations` (esi-assets.read_corporation_assets.v1; roles: Director)
- [x] `POST /corporations/{corporation_id}/assets/names` — `PostCorporationsCorporationIdAssetsNames` (esi-assets.read_corporation_assets.v1; roles: Director)

### Calendar (4/4) → `src/groups/calendar.rs`
- [x] `GET /characters/{character_id}/calendar` — `GetCharactersCharacterIdCalendar` (esi-calendar.read_calendar_events.v1)
- [x] `GET /characters/{character_id}/calendar/{event_id}` — `GetCharactersCharacterIdCalendarEventId` (esi-calendar.read_calendar_events.v1)
- [x] `PUT /characters/{character_id}/calendar/{event_id}` — `PutCharactersCharacterIdCalendarEventId` (esi-calendar.respond_calendar_events.v1)
- [x] `GET /characters/{character_id}/calendar/{event_id}/attendees` — `GetCharactersCharacterIdCalendarEventIdAttendees` (esi-calendar.read_calendar_events.v1)

### Character (14/14) → `src/groups/character.rs`
- [x] `POST /characters/affiliation` — `PostCharactersAffiliation`
- [x] `GET /characters/{character_id}` — `GetCharactersDetail`
- [x] `GET /characters/{character_id}/agents_research` — `GetCharactersCharacterIdAgentsResearch` (esi-characters.read_agents_research.v1)
- [x] `GET /characters/{character_id}/blueprints` — `GetCharactersCharacterIdBlueprints` (esi-characters.read_blueprints.v1)
- [x] `GET /characters/{character_id}/corporationhistory` — `GetCharactersCharacterIdCorporationhistory`
- [x] `POST /characters/{character_id}/cspa` — `PostCharactersCharacterIdCspa` (esi-characters.read_contacts.v1)
- [x] `GET /characters/{character_id}/fatigue` — `GetCharactersCharacterIdFatigue` (esi-characters.read_fatigue.v1)
- [x] `GET /characters/{character_id}/medals` — `GetCharactersCharacterIdMedals` (esi-characters.read_medals.v1)
- [x] `GET /characters/{character_id}/notifications` — `GetCharactersCharacterIdNotifications` (esi-characters.read_notifications.v1)
- [x] `GET /characters/{character_id}/notifications/contacts` — `GetCharactersCharacterIdNotificationsContacts` (esi-characters.read_notifications.v1)
- [x] `GET /characters/{character_id}/portrait` — `GetCharactersCharacterIdPortrait`
- [x] `GET /characters/{character_id}/roles` — `GetCharactersCharacterIdRoles` (esi-characters.read_corporation_roles.v1)
- [x] `GET /characters/{character_id}/standings` — `GetCharactersCharacterIdStandings` (esi-characters.read_standings.v1)
- [x] `GET /characters/{character_id}/titles` — `GetCharactersCharacterIdTitles` (esi-characters.read_titles.v1)

### Clones (2/2) → `src/groups/clones.rs`
- [x] `GET /characters/{character_id}/clones` — `GetCharactersCharacterIdClones` (esi-clones.read_clones.v1)
- [x] `GET /characters/{character_id}/implants` — `GetCharactersCharacterIdImplants` (esi-clones.read_implants.v1)

### Contacts (9/9) → `src/groups/contacts.rs`
- [x] `GET /alliances/{alliance_id}/contacts` — `GetAlliancesAllianceIdContacts` (esi-alliances.read_contacts.v1)
- [x] `GET /alliances/{alliance_id}/contacts/labels` — `GetAlliancesAllianceIdContactsLabels` (esi-alliances.read_contacts.v1)
- [x] `DELETE /characters/{character_id}/contacts` — `DeleteCharactersCharacterIdContacts` (esi-characters.write_contacts.v1)
- [x] `GET /characters/{character_id}/contacts` — `GetCharactersCharacterIdContacts` (esi-characters.read_contacts.v1)
- [x] `POST /characters/{character_id}/contacts` — `PostCharactersCharacterIdContacts` (esi-characters.write_contacts.v1)
- [x] `PUT /characters/{character_id}/contacts` — `PutCharactersCharacterIdContacts` (esi-characters.write_contacts.v1)
- [x] `GET /characters/{character_id}/contacts/labels` — `GetCharactersCharacterIdContactsLabels` (esi-characters.read_contacts.v1)
- [x] `GET /corporations/{corporation_id}/contacts` — `GetCorporationsCorporationIdContacts` (esi-corporations.read_contacts.v1)
- [x] `GET /corporations/{corporation_id}/contacts/labels` — `GetCorporationsCorporationIdContactsLabels` (esi-corporations.read_contacts.v1)

### Contracts (9/9) → `src/groups/contracts.rs`
- [x] `GET /characters/{character_id}/contracts` — `GetCharactersCharacterIdContracts` (esi-contracts.read_character_contracts.v1)
- [x] `GET /characters/{character_id}/contracts/{contract_id}/bids` — `GetCharactersCharacterIdContractsContractIdBids` (esi-contracts.read_character_contracts.v1)
- [x] `GET /characters/{character_id}/contracts/{contract_id}/items` — `GetCharactersCharacterIdContractsContractIdItems` (esi-contracts.read_character_contracts.v1)
- [x] `GET /contracts/public/bids/{contract_id}` — `GetContractsPublicBidsContractId`
- [x] `GET /contracts/public/items/{contract_id}` — `GetContractsPublicItemsContractId`
- [x] `GET /contracts/public/{region_id}` — `GetContractsPublicRegionId`
- [x] `GET /corporations/{corporation_id}/contracts` — `GetCorporationsCorporationIdContracts` (esi-contracts.read_corporation_contracts.v1)
- [x] `GET /corporations/{corporation_id}/contracts/{contract_id}/bids` — `GetCorporationsCorporationIdContractsContractIdBids` (esi-contracts.read_corporation_contracts.v1)
- [x] `GET /corporations/{corporation_id}/contracts/{contract_id}/items` — `GetCorporationsCorporationIdContractsContractIdItems` (esi-contracts.read_corporation_contracts.v1)

### Corporation (22/22) → `src/groups/corporation.rs`
- [x] `GET /corporations/npccorps` — `GetCorporationsNpccorps`
- [x] `GET /corporations/{corporation_id}` — `GetCorporationsCorporationId`
- [x] `GET /corporations/{corporation_id}/alliancehistory` — `GetCorporationsCorporationIdAlliancehistory`
- [x] `GET /corporations/{corporation_id}/blueprints` — `GetCorporationsCorporationIdBlueprints` (esi-corporations.read_blueprints.v1; roles: Director)
- [x] `GET /corporations/{corporation_id}/containers/logs` — `GetCorporationsCorporationIdContainersLogs` (esi-corporations.read_container_logs.v1; roles: Director)
- [x] `GET /corporations/{corporation_id}/divisions` — `GetCorporationsCorporationIdDivisions` (esi-corporations.read_divisions.v1; roles: Director)
- [x] `GET /corporations/{corporation_id}/facilities` — `GetCorporationsCorporationIdFacilities` (esi-corporations.read_facilities.v1; roles: Factory_Manager)
- [x] `GET /corporations/{corporation_id}/icons` — `GetCorporationsCorporationIdIcons`
- [x] `GET /corporations/{corporation_id}/medals` — `GetCorporationsCorporationIdMedals` (esi-corporations.read_medals.v1)
- [x] `GET /corporations/{corporation_id}/medals/issued` — `GetCorporationsCorporationIdMedalsIssued` (esi-corporations.read_medals.v1; roles: Director)
- [x] `GET /corporations/{corporation_id}/members` — `GetCorporationsCorporationIdMembers` (esi-corporations.read_corporation_membership.v1)
- [x] `GET /corporations/{corporation_id}/members/limit` — `GetCorporationsCorporationIdMembersLimit` (esi-corporations.track_members.v1; roles: Director)
- [x] `GET /corporations/{corporation_id}/members/titles` — `GetCorporationsCorporationIdMembersTitles` (esi-corporations.read_titles.v1; roles: Director)
- [x] `GET /corporations/{corporation_id}/membertracking` — `GetCorporationsCorporationIdMembertracking` (esi-corporations.track_members.v1; roles: Director)
- [x] `GET /corporations/{corporation_id}/roles` — `GetCorporationsCorporationIdRoles` (esi-corporations.read_corporation_membership.v1)
- [x] `GET /corporations/{corporation_id}/roles/history` — `GetCorporationsCorporationIdRolesHistory` (esi-corporations.read_corporation_membership.v1; roles: Director)
- [x] `GET /corporations/{corporation_id}/shareholders` — `GetCorporationsCorporationIdShareholders` (esi-wallet.read_corporation_wallets.v1; roles: Director)
- [x] `GET /corporations/{corporation_id}/standings` — `GetCorporationsCorporationIdStandings` (esi-corporations.read_standings.v1)
- [x] `GET /corporations/{corporation_id}/starbases` — `GetCorporationsCorporationIdStarbases` (esi-corporations.read_starbases.v1; roles: Director)
- [x] `GET /corporations/{corporation_id}/starbases/{starbase_id}` — `GetCorporationsCorporationIdStarbasesStarbaseId` (esi-corporations.read_starbases.v1; roles: Director)
- [x] `GET /corporations/{corporation_id}/structures` — `GetCorporationsCorporationIdStructures` (esi-corporations.read_structures.v1; roles: Station_Manager)
- [x] `GET /corporations/{corporation_id}/titles` — `GetCorporationsCorporationIdTitles` (esi-corporations.read_titles.v1; roles: Director)

### Corporation Projects (4/4) → `src/groups/corporation_projects.rs`
- [x] `GET /corporations/{corporation_id}/projects` — `GetCorporationsProjectsListing` (esi-corporations.read_projects.v1; paged)
- [x] `GET /corporations/{corporation_id}/projects/{project_id}` — `GetCorporationsProjectsDetail` (esi-corporations.read_projects.v1)
- [x] `GET /corporations/{corporation_id}/projects/{project_id}/contribution/{character_id}` — `GetCorporationsProjectsContribution` (esi-corporations.read_projects.v1)
- [x] `GET /corporations/{corporation_id}/projects/{project_id}/contributors` — `GetCorporationsProjectsContributors` (esi-corporations.read_projects.v1; roles: Project_Manager; paged)

### Cosmetics (3/3) → `src/groups/cosmetics.rs`
- [x] `GET /characters/{character_id}/cosmetics/skinr` — `GetCharactersCosmeticsSkinr` (esi.cosmetic.char:read)
- [x] `GET /characters/{character_id}/cosmetics/skinr/components` — `GetCharactersCosmeticsSkinrComponents` (esi.cosmetic.char:read)
- [x] `GET /cosmetics/skinr/{skinr_id}` — `GetCosmeticsSkinr`

### Dogma (5/5) → `src/groups/dogma.rs`
- [x] `GET /dogma/attributes` — `GetDogmaAttributes`
- [x] `GET /dogma/attributes/{attribute_id}` — `GetDogmaAttributesAttributeId`
- [x] `GET /dogma/dynamic/items/{type_id}/{item_id}` — `GetDogmaDynamicItemsTypeIdItemId`
- [x] `GET /dogma/effects` — `GetDogmaEffects`
- [x] `GET /dogma/effects/{effect_id}` — `GetDogmaEffectsEffectId`

### Faction Warfare (8/8) → `src/groups/faction_warfare.rs`
- [x] `GET /characters/{character_id}/fw/stats` — `GetCharactersCharacterIdFwStats` (esi-characters.read_fw_stats.v1)
- [x] `GET /corporations/{corporation_id}/fw/stats` — `GetCorporationsCorporationIdFwStats` (esi-corporations.read_fw_stats.v1)
- [x] `GET /fw/leaderboards` — `GetFwLeaderboards`
- [x] `GET /fw/leaderboards/characters` — `GetFwLeaderboardsCharacters`
- [x] `GET /fw/leaderboards/corporations` — `GetFwLeaderboardsCorporations`
- [x] `GET /fw/stats` — `GetFwStats`
- [x] `GET /fw/systems` — `GetFwSystems`
- [x] `GET /fw/wars` — `GetFwWars`

### Fittings (3/3) → `src/groups/fittings.rs`
- [x] `GET /characters/{character_id}/fittings` — `GetCharactersCharacterIdFittings` (esi-fittings.read_fittings.v1)
- [x] `POST /characters/{character_id}/fittings` — `PostCharactersCharacterIdFittings` (esi-fittings.write_fittings.v1)
- [x] `DELETE /characters/{character_id}/fittings/{fitting_id}` — `DeleteCharactersCharacterIdFittingsFittingId` (esi-fittings.write_fittings.v1)

### Fleets (14/14) → `src/groups/fleets.rs`
- [x] `GET /characters/{character_id}/fleet` — `GetCharactersCharacterIdFleet` (esi-fleets.read_fleet.v1)
- [x] `GET /fleets/{fleet_id}` — `GetFleetsFleetId` (esi-fleets.read_fleet.v1)
- [x] `PUT /fleets/{fleet_id}` — `PutFleetsFleetId` (esi-fleets.write_fleet.v1)
- [x] `GET /fleets/{fleet_id}/members` — `GetFleetsFleetIdMembers` (esi-fleets.read_fleet.v1)
- [x] `POST /fleets/{fleet_id}/members` — `PostFleetsFleetIdMembers` (esi-fleets.write_fleet.v1)
- [x] `DELETE /fleets/{fleet_id}/members/{member_id}` — `DeleteFleetsFleetIdMembersMemberId` (esi-fleets.write_fleet.v1)
- [x] `PUT /fleets/{fleet_id}/members/{member_id}` — `PutFleetsFleetIdMembersMemberId` (esi-fleets.write_fleet.v1)
- [x] `DELETE /fleets/{fleet_id}/squads/{squad_id}` — `DeleteFleetsFleetIdSquadsSquadId` (esi-fleets.write_fleet.v1)
- [x] `PUT /fleets/{fleet_id}/squads/{squad_id}` — `PutFleetsFleetIdSquadsSquadId` (esi-fleets.write_fleet.v1)
- [x] `GET /fleets/{fleet_id}/wings` — `GetFleetsFleetIdWings` (esi-fleets.read_fleet.v1)
- [x] `POST /fleets/{fleet_id}/wings` — `PostFleetsFleetIdWings` (esi-fleets.write_fleet.v1)
- [x] `DELETE /fleets/{fleet_id}/wings/{wing_id}` — `DeleteFleetsFleetIdWingsWingId` (esi-fleets.write_fleet.v1)
- [x] `PUT /fleets/{fleet_id}/wings/{wing_id}` — `PutFleetsFleetIdWingsWingId` (esi-fleets.write_fleet.v1)
- [x] `POST /fleets/{fleet_id}/wings/{wing_id}/squads` — `PostFleetsFleetIdWingsWingIdSquads` (esi-fleets.write_fleet.v1)

### Freelance Jobs (6/6) → `src/groups/freelance_jobs.rs`
- [x] `GET /characters/{character_id}/freelance-jobs` — `GetCharactersFreelanceJobsListing` (esi-characters.read_freelance_jobs.v1)
- [x] `GET /characters/{character_id}/freelance-jobs/{job_id}/participation` — `GetCharactersFreelanceJobsParticipation` (esi-characters.read_freelance_jobs.v1)
- [x] `GET /corporations/{corporation_id}/freelance-jobs` — `GetCorporationsFreelanceJobsListing` (esi-corporations.read_freelance_jobs.v1; roles: Project_Manager; paged)
- [x] `GET /corporations/{corporation_id}/freelance-jobs/{job_id}/participants` — `GetCorporationsFreelanceJobsParticipants` (esi-corporations.read_freelance_jobs.v1; roles: Project_Manager; paged)
- [x] `GET /freelance-jobs` — `GetFreelanceJobsListing` (paged)
- [x] `GET /freelance-jobs/{job_id}` — `GetFreelanceJobsDetail`

### Incursions (1/1) → `src/groups/incursions.rs`
- [x] `GET /incursions` — `GetIncursions`

### Industry (8/8) → `src/groups/industry.rs`
- [x] `GET /characters/{character_id}/industry/jobs` — `GetCharactersCharacterIdIndustryJobs` (esi-industry.read_character_jobs.v1)
- [x] `GET /characters/{character_id}/mining` — `GetCharactersCharacterIdMining` (esi-industry.read_character_mining.v1)
- [x] `GET /corporation/{corporation_id}/mining/extractions` — `GetCorporationCorporationIdMiningExtractions` (esi-industry.read_corporation_mining.v1; roles: Station_Manager)
- [x] `GET /corporation/{corporation_id}/mining/observers` — `GetCorporationCorporationIdMiningObservers` (esi-industry.read_corporation_mining.v1; roles: Accountant)
- [x] `GET /corporation/{corporation_id}/mining/observers/{observer_id}` — `GetCorporationCorporationIdMiningObserversObserverId` (esi-industry.read_corporation_mining.v1; roles: Accountant)
- [x] `GET /corporations/{corporation_id}/industry/jobs` — `GetCorporationsCorporationIdIndustryJobs` (esi-industry.read_corporation_jobs.v1; roles: Factory_Manager)
- [x] `GET /industry/facilities` — `GetIndustryFacilities`
- [x] `GET /industry/systems` — `GetIndustrySystems`

### Insurance (1/1) → `src/groups/insurance.rs`
- [x] `GET /insurance/prices` — `GetInsurancePrices`

### Killmails (3/3) → `src/groups/killmails.rs`
- [x] `GET /characters/{character_id}/killmails/recent` — `GetCharactersCharacterIdKillmailsRecent` (esi-killmails.read_killmails.v1)
- [x] `GET /corporations/{corporation_id}/killmails/recent` — `GetCorporationsCorporationIdKillmailsRecent` (esi-killmails.read_corporation_killmails.v1; roles: Director)
- [x] `GET /killmails/{killmail_id}/{killmail_hash}` — `GetKillmailsKillmailIdKillmailHash`

### Location (3/3) → `src/groups/location.rs`
- [x] `GET /characters/{character_id}/location` — `GetCharactersCharacterIdLocation` (esi-location.read_location.v1)
- [x] `GET /characters/{character_id}/online` — `GetCharactersCharacterIdOnline` (esi-location.read_online.v1)
- [x] `GET /characters/{character_id}/ship` — `GetCharactersCharacterIdShip` (esi-location.read_ship_type.v1)

### Loyalty (2/2) → `src/groups/loyalty.rs`
- [x] `GET /characters/{character_id}/loyalty/points` — `GetCharactersCharacterIdLoyaltyPoints` (esi-characters.read_loyalty.v1)
- [x] `GET /loyalty/stores/{corporation_id}/offers` — `GetLoyaltyStoresCorporationIdOffers`

### Mail (9/9) → `src/groups/mail.rs`
- [x] `GET /characters/{character_id}/mail` — `GetCharactersCharacterIdMail` (esi-mail.read_mail.v1)
- [x] `POST /characters/{character_id}/mail` — `PostCharactersCharacterIdMail` (esi-mail.send_mail.v1)
- [x] `GET /characters/{character_id}/mail/labels` — `GetCharactersCharacterIdMailLabels` (esi-mail.read_mail.v1)
- [x] `POST /characters/{character_id}/mail/labels` — `PostCharactersCharacterIdMailLabels` (esi-mail.organize_mail.v1)
- [x] `DELETE /characters/{character_id}/mail/labels/{label_id}` — `DeleteCharactersCharacterIdMailLabelsLabelId` (esi-mail.organize_mail.v1)
- [x] `GET /characters/{character_id}/mail/lists` — `GetCharactersCharacterIdMailLists` (esi-mail.read_mail.v1)
- [x] `DELETE /characters/{character_id}/mail/{mail_id}` — `DeleteCharactersCharacterIdMailMailId` (esi-mail.organize_mail.v1)
- [x] `GET /characters/{character_id}/mail/{mail_id}` — `GetCharactersCharacterIdMailMailId` (esi-mail.read_mail.v1)
- [x] `PUT /characters/{character_id}/mail/{mail_id}` — `PutCharactersCharacterIdMailMailId` (esi-mail.organize_mail.v1)

### Market (11/11) → `src/groups/market.rs`
- [x] `GET /characters/{character_id}/orders` — `GetCharactersCharacterIdOrders` (esi-markets.read_character_orders.v1)
- [x] `GET /characters/{character_id}/orders/history` — `GetCharactersCharacterIdOrdersHistory` (esi-markets.read_character_orders.v1)
- [x] `GET /corporations/{corporation_id}/orders` — `GetCorporationsCorporationIdOrders` (esi-markets.read_corporation_orders.v1; roles: Accountant, Trader)
- [x] `GET /corporations/{corporation_id}/orders/history` — `GetCorporationsCorporationIdOrdersHistory` (esi-markets.read_corporation_orders.v1; roles: Accountant, Trader)
- [x] `GET /markets/groups` — `GetMarketsGroups`
- [x] `GET /markets/groups/{market_group_id}` — `GetMarketsGroupsMarketGroupId`
- [x] `GET /markets/prices` — `GetMarketsPrices`
- [x] `GET /markets/structures/{structure_id}` — `GetMarketsStructuresStructureId` (esi-markets.structure_markets.v1)
- [x] `GET /markets/{region_id}/history` — `GetMarketsRegionIdHistory`
- [x] `GET /markets/{region_id}/orders` — `GetMarketsRegionIdOrders`
- [x] `GET /markets/{region_id}/types` — `GetMarketsRegionIdTypes`

### Meta (4/4) → `src/groups/meta.rs`
- [x] `GET /meta/changelog` — `GetMetaChangelog`
- [x] `GET /meta/compatibility-dates` — `GetMetaCompatibilityDates`
- [x] `GET /meta/name` — `GetMetaName`
- [x] `GET /meta/status` — `GetMetaStatus`

### Military Campaigns (6/6) → `src/groups/military_campaigns.rs`
- [x] `GET /characters/{character_id}/military-campaigns/objectives` — `GetCharactersMilitaryCampaignsObjectivesListing` (esi.activity.char:read; paged)
- [x] `GET /characters/{character_id}/military-campaigns/objectives/{objective_id}` — `GetCharactersMilitaryCampaignsObjectivesParticipation` (esi.activity.char:read)
- [x] `GET /military-campaigns` — `GetMilitaryCampaignsListing`
- [x] `GET /military-campaigns/{campaign_id}` — `GetMilitaryCampaignsDetail`
- [x] `GET /military-campaigns/{campaign_id}/objectives` — `GetMilitaryCampaignsObjectivesListing` (paged)
- [x] `GET /military-campaigns/{campaign_id}/objectives/{objective_id}` — `GetMilitaryCampaignsObjectivesDetail`

### Paragon Hub (5/5) → `src/groups/paragon_hub.rs`
- [x] `GET /characters/{character_id}/paragon-hub/skinr` — `GetCharactersParagonHubSkinr` (esi.cosmetic.char:read; paged; tombstone)
- [x] `GET /paragon-hub/skinr` — `GetParagonHubSkinr` (paged; tombstone)
- [x] `GET /paragon-hub/skinr/alliances/{alliance_id}` — `GetParagonHubSkinrAlliances` (esi.cosmetic.char:read; paged; tombstone)
- [x] `GET /paragon-hub/skinr/characters/{character_id}` — `GetParagonHubSkinrCharacters` (esi.cosmetic.char:read; paged; tombstone)
- [x] `GET /paragon-hub/skinr/corporations/{corporation_id}` — `GetParagonHubSkinrCorporations` (esi.cosmetic.char:read; paged; tombstone)

### Planetary Interaction (4/4) → `src/groups/planetary_interaction.rs`
- [x] `GET /characters/{character_id}/planets` — `GetCharactersCharacterIdPlanets` (esi-planets.manage_planets.v1)
- [x] `GET /characters/{character_id}/planets/{planet_id}` — `GetCharactersCharacterIdPlanetsPlanetId` (esi-planets.manage_planets.v1)
- [x] `GET /corporations/{corporation_id}/customs_offices` — `GetCorporationsCorporationIdCustomsOffices` (esi-planets.read_customs_offices.v1; roles: Director)
- [x] `GET /universe/schematics/{schematic_id}` — `GetUniverseSchematicsSchematicId`

### Routes (1/1) → `src/groups/routes.rs`
- [x] `POST /route/{origin_system_id}/{destination_system_id}` — `PostRoute`

### Search (1/1) → `src/groups/search.rs`
- [x] `GET /characters/{character_id}/search` — `GetCharactersCharacterIdSearch` (esi-search.search_structures.v1)

### Skills (3/3) → `src/groups/skills.rs`
- [x] `GET /characters/{character_id}/attributes` — `GetCharactersCharacterIdAttributes` (esi-skills.read_skills.v1)
- [x] `GET /characters/{character_id}/skillqueue` — `GetCharactersCharacterIdSkillqueue` (esi-skills.read_skillqueue.v1)
- [x] `GET /characters/{character_id}/skills` — `GetCharactersCharacterIdSkills` (esi-skills.read_skills.v1)

### Sovereignty (2/2) → `src/groups/sovereignty.rs`
- [x] `GET /sovereignty/campaigns` — `GetSovereigntyCampaigns`
- [x] `GET /sovereignty/systems` — `GetSovereigntySystems`

### Status (1/1) → `src/groups/status.rs`
- [x] `GET /status` — `GetStatus`

### Structures (6/6) → `src/groups/structures.rs`
- [x] `GET /characters/{character_id}/structures/mercenary-dens` — `GetCharactersStructuresMercenaryDensListing` (esi-structures.read_character.v1)
- [x] `GET /characters/{character_id}/structures/mercenary-dens/{mercenary_den_id}` — `GetCharactersStructuresMercenaryDensDetail` (esi-structures.read_character.v1)
- [x] `GET /corporations/{corporation_id}/structures/skyhooks` — `GetCorporationsStructuresSkyhooksListing` (esi-structures.read_corporation.v1; roles: Station_Manager)
- [x] `GET /corporations/{corporation_id}/structures/skyhooks/{skyhook_id}` — `GetCorporationsStructuresSkyhooksDetail` (esi-structures.read_corporation.v1; roles: Station_Manager)
- [x] `GET /corporations/{corporation_id}/structures/sovereignty-hubs` — `GetCorporationsStructuresSovereigntyHubsListing` (esi-structures.read_corporation.v1)
- [x] `GET /corporations/{corporation_id}/structures/sovereignty-hubs/{sovereignty_hub_id}` — `GetCorporationsStructuresSovereigntyHubsDetail` (esi-structures.read_corporation.v1; roles: Station_Manager)

### Universe (30/30) → `src/groups/universe.rs`
- [x] `GET /universe/ancestries` — `GetUniverseAncestries`
- [x] `GET /universe/asteroid_belts/{asteroid_belt_id}` — `GetUniverseAsteroidBeltsAsteroidBeltId`
- [x] `GET /universe/bloodlines` — `GetUniverseBloodlines`
- [x] `GET /universe/categories` — `GetUniverseCategories`
- [x] `GET /universe/categories/{category_id}` — `GetUniverseCategoriesCategoryId`
- [x] `GET /universe/constellations` — `GetUniverseConstellations`
- [x] `GET /universe/constellations/{constellation_id}` — `GetUniverseConstellationsConstellationId`
- [x] `GET /universe/factions` — `GetUniverseFactions`
- [x] `GET /universe/graphics` — `GetUniverseGraphics`
- [x] `GET /universe/graphics/{graphic_id}` — `GetUniverseGraphicsGraphicId`
- [x] `GET /universe/groups` — `GetUniverseGroups`
- [x] `GET /universe/groups/{group_id}` — `GetUniverseGroupsGroupId`
- [x] `POST /universe/ids` — `PostUniverseIds`
- [x] `GET /universe/moons/{moon_id}` — `GetUniverseMoonsMoonId`
- [x] `POST /universe/names` — `PostUniverseNames`
- [x] `GET /universe/planets/{planet_id}` — `GetUniversePlanetsPlanetId`
- [x] `GET /universe/races` — `GetUniverseRaces`
- [x] `GET /universe/regions` — `GetUniverseRegions`
- [x] `GET /universe/regions/{region_id}` — `GetUniverseRegionsRegionId`
- [x] `GET /universe/stargates/{stargate_id}` — `GetUniverseStargatesStargateId`
- [x] `GET /universe/stars/{star_id}` — `GetUniverseStarsStarId`
- [x] `GET /universe/stations/{station_id}` — `GetUniverseStationsStationId`
- [x] `GET /universe/structures` — `GetUniverseStructures`
- [x] `GET /universe/structures/{structure_id}` — `GetUniverseStructuresStructureId` (esi-universe.read_structures.v1)
- [x] `GET /universe/system_jumps` — `GetUniverseSystemJumps`
- [x] `GET /universe/system_kills` — `GetUniverseSystemKills`
- [x] `GET /universe/systems` — `GetUniverseSystems`
- [x] `GET /universe/systems/{system_id}` — `GetUniverseSystemsSystemId`
- [x] `GET /universe/types` — `GetUniverseTypes`
- [x] `GET /universe/types/{type_id}` — `GetUniverseTypesTypeId`

### User Interface (5/5) → `src/groups/user_interface.rs`
- [x] `POST /ui/autopilot/waypoint` — `PostUiAutopilotWaypoint` (esi-ui.write_waypoint.v1)
- [x] `POST /ui/openwindow/contract` — `PostUiOpenwindowContract` (esi-ui.open_window.v1)
- [x] `POST /ui/openwindow/information` — `PostUiOpenwindowInformation` (esi-ui.open_window.v1)
- [x] `POST /ui/openwindow/marketdetails` — `PostUiOpenwindowMarketdetails` (esi-ui.open_window.v1)
- [x] `POST /ui/openwindow/newmail` — `PostUiOpenwindowNewmail` (esi-ui.open_window.v1)

### Wallet (6/6) → `src/groups/wallet.rs`
- [x] `GET /characters/{character_id}/wallet` — `GetCharactersCharacterIdWallet` (esi-wallet.read_character_wallet.v1)
- [x] `GET /characters/{character_id}/wallet/journal` — `GetCharactersCharacterIdWalletJournal` (esi-wallet.read_character_wallet.v1)
- [x] `GET /characters/{character_id}/wallet/transactions` — `GetCharactersCharacterIdWalletTransactions` (esi-wallet.read_character_wallet.v1)
- [x] `GET /corporations/{corporation_id}/wallets` — `GetCorporationsCorporationIdWallets` (esi-wallet.read_corporation_wallets.v1; roles: Accountant, Junior_Accountant)
- [x] `GET /corporations/{corporation_id}/wallets/{division}/journal` — `GetCorporationsCorporationIdWalletsDivisionJournal` (esi-wallet.read_corporation_wallets.v1; roles: Accountant, Junior_Accountant)
- [x] `GET /corporations/{corporation_id}/wallets/{division}/transactions` — `GetCorporationsCorporationIdWalletsDivisionTransactions` (esi-wallet.read_corporation_wallets.v1; roles: Accountant, Junior_Accountant)

### Wars (3/3) → `src/groups/wars.rs`
- [x] `GET /wars` — `GetWars`
- [x] `GET /wars/{war_id}` — `GetWarsWarId`
- [x] `GET /wars/{war_id}/killmails` — `GetWarsWarIdKillmails`

## Release housekeeping

- [x] `CHANGELOG.md` updated for the 0.2.0 release and the README has a Coverage section.
- [x] Version 0.2.0 (breaking: integer types are `i64`, new macros and errors); `Cargo.toml` and `CHANGELOG.md` updated. Publishing is still a manual step (GitHub release).
- [x] The alias table (`src/legacy.rs`) only maps rfesi IDs; a test checks every target exists in the spec and the table is smaller than the spec.
