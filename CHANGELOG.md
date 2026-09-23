# Changelog

All notable changes to this project are documented here. This project is a
fork of [rfesi](https://github.com/Celeo/rfesi) 0.50.2; versioning restarts at 0.1.0.

## [0.1.0] - 09-22-2026

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
  with a warning naming the OpenAPI ID. They will be removed in 0.2.0.

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
