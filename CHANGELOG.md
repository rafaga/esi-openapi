# Changelog

All notable changes to this project are documented here. This project is a
fork of [rfesi](https://github.com/Celeo/rfesi) 0.50.2; versioning restarts at 0.1.0.

## [Unreleased]

## [0.2.0] - 2026-10-08

### Added

- New endpoint groups: `Esi::group_meta()` (`GetMetaChangelog`, `GetMetaCompatibilityDates`, `GetMetaName`, `GetMetaStatus`).
- `StatusGroup::get_status`, `WarsGroup` (`list`, `get_war`, `get_war_killmails`), `SovereigntyGroup` (`get_campaigns`, `get_systems`) and `InsuranceGroup::get_prices`.
- `DogmaGroup` (attributes, effects, dynamic items), `RoutesGroup::calculate_route` (`RouteRequest`, `RoutePreference`), `LoyaltyGroup::get_store_offers`, `ContractsGroup` public contracts (listing, items, bids), `IndustryGroup::get_facilities` and `MarketGroup` (`get_groups`, `get_group`, `get_region_types`).
- New groups: `Esi::group_freelance_jobs()` (`list`, `get_job`) and `Esi::group_military_campaigns()` (`list`, `get_campaign`, `list_objectives`, `get_objective`), with a shared `Cursor` type for cursor pagination.
- Fields that the spec defines as enums are Rust enums with an `Unrecognized` fallback variant (`HttpMethod`, `ChangeType`, `RouteHealth`, `CampaignEventType`, `ContractType`, `JobState`, `JobCareer`, `CampaignState`).
- `UniverseGroup`: ancestries, bloodlines, races, factions, graphics, categories, group IDs, asteroid belts, moons, planets, stars, stargates, public structures, system jumps and kills, and name resolution (`get_names`), with `StructureFilter` and `NameCategory`.
- New groups: `Esi::group_cosmetics()` (`get_skinr`) and `Esi::group_paragon_hub()` (`get_skinr_listings`); `PlanetaryInteractionGroup::get_schematic`.
- Authenticated character endpoints: `SkillsGroup` (`get_attributes`, `get_skillqueue`), `CharacterGroup` (`get_agents_research`, `get_fatigue`, `get_medals`, `get_contact_notifications`, `get_roles`, `get_standings`, `get_titles`, `calculate_cspa_charge`), `WalletGroup::get_journal`, `LoyaltyGroup::get_character_points` and `FactionWarfareGroup::get_character_stats`. Their `character_id` is `i64`.
- `MailGroup` (mail list and details, send, update, delete, labels create/delete, mailing lists) and `ContactsGroup` (character, corporation and alliance contacts and labels, add/edit/delete character contacts), with `RecipientType`, `MailLabelColor` and `ContactType` enums. `MailLabel::label_id` is now `i64`.
- `api_get!`, `api_post!`, `api_put!` and `api_delete!` accept tagged query parameters (`Required`, `Optional`, `Many`, `OptionalMany`); list parameters are sent as repeated keys.
- `CalendarGroup` (events, event details, respond, attendees) and `FittingsGroup` (list, create, delete), with `EventResponse`, `EventOwnerType`, `EventReply` and `FittingFlag` enums.
- `FleetsGroup` (character fleet, settings, members, wings and squads: invite, kick, move, create, rename, delete), with a `FleetRole` enum. `api_post!` accepts `; NoBody` for POST requests without a body.
- Character and corporation contracts (list, bids, items) with `ContractStatus` and `ContractAvailability` enums; `PlanetaryInteractionGroup` colonies (`get_character_planets`, `get_character_planet`) and `get_corporation_customs_offices`, with `PlanetType` and `StandingLevel` enums.
- `CorporationGroup`: blueprints, container logs, divisions, facilities, icons, medals, issued medals, member limit, member titles, member tracking, roles, roles history, shareholders, standings, starbases, structures and titles, with enums for container actions, role types, starbase and structure states (all with an `Unrecognized` fallback). `CorporationRole`, `MedalStatus` and `StandingSource` now also implement `Serialize`.
- **Breaking:** every integer the spec defines as `int64` is now `i64` in the group types and method parameters (previously `i32`, `u32`, `u64`, `u16` or `u8` in many places). `page` stays `i32`.
- `Structures` group: character mercenary dens, corporation skyhooks and sovereignty hubs (listing and detail).
- `CorporationProjects` group: project listing, detail, contribution and contributors, with cursor pagination (`Cursor`).
- Industry mining (character, corporation extractions and observers) and corporation industry jobs; corporation killmails, orders (open and history), structure market orders and character order history; corporation wallets (balances, journal, transactions). New enums `OrderRange`, `ClosedOrderState`, `IndustryJobStatus` and `ObserverType`.
- New groups `AccessListGroup` and `ActivitiesGroup`; character SKINR licenses and components, corporation faction warfare stats, character and corporation freelance jobs, character military campaign objectives, Paragon Hub SKINR listings (character, alliance, corporation) and the remaining User Interface calls (autopilot waypoint, contract, information and new mail windows). Enums `AccessLevel`, `TacticalOperationState`, `ParticipationState` and `SkinrComponentType`. All 233 operations of the spec are now covered.
- `tests/conformance.rs` checks every spec operation is implemented and that the HTTP method, path placeholders, query keys and response struct fields match `resources/test/openapi.json`.
- Pagination helpers: `Esi::query_with_pages` (also returns the `X-Pages` header), `Esi::fetch_all_pages` (walks the `page` parameter) and `Esi::fetch_all_cursor` (walks `cursor.after`). `RequestType` is now `Clone` and `Copy`.
- Optional response cache (`EsiBuilder::enable_cache`, off by default): `GET` responses are revalidated with `ETag` / `Last-Modified`, a `304 Not Modified` reuses the stored body, and entries are served without a request while younger than the operation's `x-client-cache-ttl` (or the response `max-age`). `Spec::client_cache_ttl` and `Esi::cache_len` were added.
- `Spec` now reads scopes, rate limits, required roles, pagination and tombstone TTLs per operation, and the OAuth2 scopes of the spec (`Spec::operation`, `Spec::required_scopes`, `Spec::oauth_scopes`, `Spec::scope_string_for`). `Esi::required_scopes`, `Esi::missing_scopes`, `Esi::required_roles` and `Esi::declared_rate_limits` expose them.
- Weekly `spec-diff` workflow that reports drift between the live ESI spec and the test fixture.
- `EsiBuilder::language` (`Language`, the `Accept-Language` header) and `EsiBuilder::tenant` (the `X-Tenant` header). `If-None-Match` and `If-Modified-Since` are sent by the response cache.
- `EsiError::Gone`: returned for HTTP `404` / `410` on the five operations that declare `x-tombstone-ttl` (Paragon Hub SKINR listings), with the tombstone TTL. Which status ESI uses for a tombstone is not in the spec, so both are accepted.
- Responses are requested compressed (`gzip`, `brotli`, `deflate` features of `reqwest`).
- `Esi::fetch_all_cursor` asks for the largest page size the spec allows (`limit=100`) unless the query sets `limit`; `Esi::fetch_all_pages` requests the pages after the first concurrently (`EsiBuilder::page_concurrency`, 4 by default); the response cache holds at most 1024 responses (`EsiBuilder::cache_max_entries`). Only when it is full, it drops expired entries without an `ETag` / `Last-Modified` first, then expired entries that can be revalidated, then live ones, least recently used first within each group.
- `CorporationRole` enum (with an `Unrecognized` fallback).
- The live tests cover the new endpoints; the scopes they need were added to `ESI_SCOPES` in `.env.example`.
- `uuid` is now a dependency (UUID identifiers of freelance jobs and military campaigns).
- `api_put!` and `api_delete!` macros for building PUT and DELETE endpoint functions.
- `EsiBuilder::cache_max_bytes` limits the size of the response cache in bytes, next to `cache_max_entries`; both use the same eviction order, and a response larger than the limit is not cached. `Esi::cache_bytes` reports the size.
- `Esi::post_chunked` sends a `POST` with an array body in chunks (concurrently, in order). The list endpoints with a maximum body length split longer lists automatically: `CharacterGroup::get_affiliation`, `UniverseGroup::get_names` and the character and corporation asset names and locations (1000 items per request, checked against the spec by `tests/conformance.rs`). An empty list sends no request.
- `EsiBuilder::rate_limit_policy` (`RateLimitPolicy`: `Off`, `Wait { max_wait }`, `Fail`; `Off` by default) throttles requests before ESI answers `429`. The client tracks the tokens of each route group and access token from the `X-Ratelimit-*` headers and its own spending, honors `Retry-After`, and either sleeps until the request fits or returns `EsiError::RateLimited` without calling ESI. Only operations that declare an `x-rate-limit` group in the spec are throttled.
- `Esi::ensure_spec_fresh` downloads the spec only when it is missing, was requested with another compatibility date, or its `Cache-Control: max-age` has passed. `Esi::update_spec` still always asks, but sends `If-None-Match` / `If-Modified-Since` when the server gave the spec an `ETag` or `Last-Modified` (it does not today) and keeps the spec on a `304`.

### Changed

- Resolving a path to its `x-client-cache-ttl` / `x-tombstone-ttl` and an `operationId` to its metadata uses an index built once with the spec instead of scanning every path on each call (`Spec::get_operation_for_path` still scans and is meant for one-off use).
- `Esi::fetch_all_cursor` reads each page in one pass, straight into the record type, instead of building a JSON tree first.
- **Breaking:** the cursor page types (`ProjectsPage`, `ContributorsPage`, `FreelanceJobsPage`, `JobParticipantsPage`, `ObjectivesPage`, `CharacterObjectivesPage`, `SkinrListingsPage`, `CharacterSkinrListingsPage`) are replaced by one `Page<T>` with `cursor` and `items` (the records, whatever ESI names the array). `KillmailPosition` and `AssetLocationPosition` are now `Position` (which is also `Copy` and `PartialEq`), `CorporationStanding` is `Standing` and `CorporationBlueprint` is `Blueprint`.
- **Breaking:** every `character_id` is now `i64` instead of `i32`: the path parameter of all character methods, the `character_id` fields of `CharacterAffiliation`, `CharacterLeaderboardItem` and the killmail structs, `IndustryJob::completed_character_id`, and the `character_ids` argument of `CharacterGroup::get_affiliation` (now `&[i64]`, was `&[u64]`).

### Fixed

- Successful responses with an empty body (such as `204 No Content`) no longer fail to parse when the return type is `()`.

## [0.1.0] - 2026-09-22

### Changed

- Renamed the crate from `rfesi` to `esi-openapi` (`use esi_openapi::prelude::*`).
- Endpoints are resolved through the ESI OpenAPI 3.1 spec at
  `https://esi.evetech.net/meta/openapi.json` instead of the retired
  `latest/swagger.json` (which now returns 404). The spec is requested with the
  client's `X-Compatibility-Date`.
- Default compatibility date is now `2026-08-18` (`COMPATIBILITY_DATE_DEFAULT`).
- Endpoint groups use the OpenAPI operation IDs (e.g. `GetMarketsRegionIdOrders`).
- `Spec` now models OpenAPI path items (`SpecPathItem`) and ignores unknown keys;
  `SpecPathMethod::operation_id` is an `Option<String>`. The `spec` module is public
  and `Spec` is re-exported from the prelude.
- `operationId` lookups use an index built once when the spec is loaded.
- `EsiError` is now `#[non_exhaustive]`.
- Dependencies upgraded: reqwest 0.13 (with the `form` and `query` features), rand 0.10,
  base64 0.23, jsonwebtoken 11, sha2 0.11, thiserror 2.
- TLS uses rustls by default (`rustls-tls` feature); the `default-tls` feature was removed.
- Response structs updated for the `2026-08-18` schemas:
  - `CorporationPublicInfo`: `ceo_id` and `creator_id` are optional; `faction_id` is now
    `enlisted_faction_id`; `tax_rate` is replaced by `tax_rates` (`CorporationTaxRates`);
    new `friendly_fire`, `palette`, `state` and `corporation_type` fields.
  - `CharacterPublicInfo`: `title` removed; new `achievement_score`, `character_title_id`,
    `corporation_title` and `faction_id` fields.

### Added

- Rate-limit support: `X-Ratelimit-*` headers are recorded per route group and
  exposed through `Esi::rate_limit_status` / `Esi::rate_limit_statuses`
  (`RateLimitStatus`). A `429` response returns `EsiError::RateLimited`
  with the group and the `Retry-After` seconds. The legacy error-limit handling
  (`X-Esi-Error-Limit-*`) is kept for routes that still use it.
- `ErrorLimitStatus` and `RateLimitStatus` are exported from the prelude.
- Live integration tests for the authenticated endpoints (`tests/authenticated.rs`),
  configured through a git-ignored `.env` file (template in `.env.example`), and an
  `auth_get_refresh_token` example that logs in and stores the refresh token there.

### Fixed

- `EsiError::ReqwestError` now shows the underlying error instead of always saying
  "Error constructing HTTP client".
- `UserInterfaceGroup::open_market_details_window` now sends `type_id` as a query
  parameter (it was never sent before).
- `Killmail`: replaced the non-existent `killmail_type` field (which made every call
  fail) with `killmail_time`; added `war_id`, victim `ship_type_id` and `position`,
  attacker `faction_id` and nested item `items`.
- `MailLabels`: `unread_count` is now `total_unread_count`, matching ESI.
- Fields ESI may omit are now optional: `Clones::last_clone_jump_date`,
  `Skills::unallocated_sp`, `Structure::position`. Added `Clones::last_station_change_date`.

### Deprecated

- rfesi's snake_case operation IDs are still accepted by `Esi::get_endpoint_for_op_id`,
  with a warning naming the OpenAPI ID. They will be removed before 1.0.0.

| rfesi (Swagger) ID | esi-openapi (OpenAPI) ID |
| --- | --- |
| `get_alliances` | `GetAlliances` |
| `get_alliances_alliance_id` | `GetAlliancesAllianceId` |
| `get_alliances_alliance_id_corporations` | `GetAlliancesAllianceIdCorporations` |
| `get_alliances_alliance_id_icons` | `GetAlliancesAllianceIdIcons` |
| `get_characters_character_id` | `GetCharactersDetail` |
| `get_characters_character_id_assets` | `GetCharactersCharacterIdAssets` |
| `get_characters_character_id_blueprints` | `GetCharactersCharacterIdBlueprints` |
| `get_characters_character_id_clones` | `GetCharactersCharacterIdClones` |
| `get_characters_character_id_corporationhistory` | `GetCharactersCharacterIdCorporationhistory` |
| `get_characters_character_id_implants` | `GetCharactersCharacterIdImplants` |
| `get_characters_character_id_industry_jobs` | `GetCharactersCharacterIdIndustryJobs` |
| `get_characters_character_id_killmails_recent` | `GetCharactersCharacterIdKillmailsRecent` |
| `get_characters_character_id_location` | `GetCharactersCharacterIdLocation` |
| `get_characters_character_id_mail_labels` | `GetCharactersCharacterIdMailLabels` |
| `get_characters_character_id_notifications` | `GetCharactersCharacterIdNotifications` |
| `get_characters_character_id_online` | `GetCharactersCharacterIdOnline` |
| `get_characters_character_id_orders` | `GetCharactersCharacterIdOrders` |
| `get_characters_character_id_portrait` | `GetCharactersCharacterIdPortrait` |
| `get_characters_character_id_search` | `GetCharactersCharacterIdSearch` |
| `get_characters_character_id_ship` | `GetCharactersCharacterIdShip` |
| `get_characters_character_id_skills` | `GetCharactersCharacterIdSkills` |
| `get_characters_character_id_wallet` | `GetCharactersCharacterIdWallet` |
| `get_characters_character_id_wallet_transactions` | `GetCharactersCharacterIdWalletTransactions` |
| `get_corporations_corporation_id` | `GetCorporationsCorporationId` |
| `get_corporations_corporation_id_alliancehistory` | `GetCorporationsCorporationIdAlliancehistory` |
| `get_corporations_corporation_id_assets` | `GetCorporationsCorporationIdAssets` |
| `get_corporations_corporation_id_members` | `GetCorporationsCorporationIdMembers` |
| `get_corporations_npccorps` | `GetCorporationsNpccorps` |
| `get_fw_leaderboards` | `GetFwLeaderboards` |
| `get_fw_leaderboards_characters` | `GetFwLeaderboardsCharacters` |
| `get_fw_leaderboards_corporations` | `GetFwLeaderboardsCorporations` |
| `get_fw_stats` | `GetFwStats` |
| `get_fw_systems` | `GetFwSystems` |
| `get_fw_wars` | `GetFwWars` |
| `get_incursions` | `GetIncursions` |
| `get_industry_systems` | `GetIndustrySystems` |
| `get_killmails_killmail_id_killmail_hash` | `GetKillmailsKillmailIdKillmailHash` |
| `get_markets_prices` | `GetMarketsPrices` |
| `get_markets_region_id_history` | `GetMarketsRegionIdHistory` |
| `get_markets_region_id_orders` | `GetMarketsRegionIdOrders` |
| `get_universe_categories_category_id` | `GetUniverseCategoriesCategoryId` |
| `get_universe_constellations` | `GetUniverseConstellations` |
| `get_universe_constellations_constellation_id` | `GetUniverseConstellationsConstellationId` |
| `get_universe_groups_group_id` | `GetUniverseGroupsGroupId` |
| `get_universe_regions` | `GetUniverseRegions` |
| `get_universe_regions_region_id` | `GetUniverseRegionsRegionId` |
| `get_universe_stations_station_id` | `GetUniverseStationsStationId` |
| `get_universe_structures_structure_id` | `GetUniverseStructuresStructureId` |
| `get_universe_systems` | `GetUniverseSystems` |
| `get_universe_systems_system_id` | `GetUniverseSystemsSystemId` |
| `get_universe_types` | `GetUniverseTypes` |
| `get_universe_types_type_id` | `GetUniverseTypesTypeId` |
| `post_characters_affiliation` | `PostCharactersAffiliation` |
| `post_characters_character_id_assets_locations` | `PostCharactersCharacterIdAssetsLocations` |
| `post_characters_character_id_assets_names` | `PostCharactersCharacterIdAssetsNames` |
| `post_corporations_corporation_id_assets_locations` | `PostCorporationsCorporationIdAssetsLocations` |
| `post_corporations_corporation_id_assets_names` | `PostCorporationsCorporationIdAssetsNames` |
| `post_ui_openwindow_marketdetails` | `PostUiOpenwindowMarketdetails` |
| `post_universe_ids` | `PostUniverseIds` |
