#![allow(unused)]

use crate::prelude::*;

/// Endpoints for Universe
pub struct UniverseGroup<'a> {
    pub(crate) esi: &'a Esi,
}

/// A position in space, in meters.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct Constellation {
    pub constellation_id: i64,
    pub name: String,
    pub position: Position,
    pub region_id: i64,
    pub systems: Vec<i64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct Region {
    pub constellations: Vec<i64>,
    pub description: Option<String>,
    pub name: String,
    pub region_id: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct SystemPlanet {
    pub asteroid_belts: Option<Vec<i64>>,
    pub moons: Option<Vec<i64>>,
    pub planet_id: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct System {
    pub constellation_id: i64,
    pub name: String,
    pub planets: Option<Vec<SystemPlanet>>,
    pub position: Position,
    pub security_class: Option<String>,
    pub security_status: f64,
    pub star_id: Option<i64>,
    pub stargates: Option<Vec<i64>>,
    pub stations: Option<Vec<i64>>,
    pub system_id: i64,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct Ids {
    pub characters: Option<Vec<Category>>,
    pub alliances: Option<Vec<Category>>,
    pub constellations: Option<Vec<Category>>,
    pub agents: Option<Vec<Category>>,
    pub regions: Option<Vec<Category>>,
    pub systems: Option<Vec<Category>>,
    pub stations: Option<Vec<Category>>,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct Category {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct TypeDogmaAttribute {
    pub attribute_id: i64,
    pub value: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct TypeDogmaEffect {
    pub effect_id: i64,
    pub is_default: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct Type {
    pub capacity: Option<f64>,
    pub description: String,
    pub dogma_attributes: Option<Vec<TypeDogmaAttribute>>,
    pub dogma_effects: Option<Vec<TypeDogmaEffect>>,
    pub graphic_id: Option<i64>,
    pub group_id: i64,
    pub icon_id: Option<i64>,
    pub market_group_id: Option<i64>,
    pub mass: Option<f64>,
    pub name: String,
    pub packaged_volume: Option<f64>,
    pub portion_size: Option<i64>,
    pub published: bool,
    pub radius: Option<f64>,
    pub type_id: i64,
    pub volume: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct Station {
    pub max_dockable_ship_volume: f64,
    pub name: String,
    pub office_rental_cost: f64,
    pub owner: Option<i64>,
    pub position: Position,
    pub race_id: Option<i64>,
    pub reprocessing_efficiency: f64,
    pub reprocessing_stations_take: f64,
    pub services: Vec<String>,
    pub station_id: i64,
    pub system_id: i64,
    pub type_id: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct Structure {
    pub name: String,
    pub owner_id: i64,
    pub position: Option<Position>,
    pub solar_system_id: i64,
    pub type_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct CategoriesCategory {
    pub category_id: i64,
    pub groups: Vec<i64>,
    pub name: String,
    pub published: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct Group {
    pub category_id: i64,
    pub group_id: i64,
    pub name: String,
    pub published: bool,
    pub types: Vec<i64>,
}

impl UniverseGroup<'_> {
    api_get!(
        /// Get information on a category
        get_universe_categories_category,
        "GetUniverseCategoriesCategoryId",
        RequestType::Public,
        CategoriesCategory,
        (category_id: i64) => "{category_id}"
    );

    api_get!(
        /// Get information on a group
        get_universe_groups_group,
        "GetUniverseGroupsGroupId",
        RequestType::Public,
        Group,
        (group_id: i64) => "{group_id}"
    );

    api_get!(
        /// Get information on a type
        get_universe_types_type,
        "GetUniverseTypesTypeId",
        RequestType::Public,
        Type,
        (type_id: i64) => "{type_id}"
    );

    api_get!(
        /// Get a list of constellation ids
        get_constellation_ids,
        "GetUniverseConstellations",
        RequestType::Public,
        Vec<i64>,
    );

    api_get!(
        /// Get information on a constellation
        get_constellation,
        "GetUniverseConstellationsConstellationId",
        RequestType::Public,
        Constellation,
        (constellation_id: i64) => "{constellation_id}"
    );

    api_get!(
        /// Get a list of region ids
        get_region_ids,
        "GetUniverseRegions",
        RequestType::Public,
        Vec<i64>,
    );

    api_get!(
        /// Get information on a region
        get_region,
        "GetUniverseRegionsRegionId",
        RequestType::Public,
        Region,
        (region_id: i64) => "{region_id}"
    );

    api_get!(
        /// Get a list of system ids
        get_system_ids,
        "GetUniverseSystems",
        RequestType::Public,
        Vec<i64>,
    );

    api_get!(
        /// Get information on a system
        get_system,
        "GetUniverseSystemsSystemId",
        RequestType::Public,
        System,
        (system_id: i64) => "{system_id}"
    );

    api_get!(
        /// Get a list of type ids
        get_type_ids,
        "GetUniverseTypes",
        RequestType::Public,
        Vec<i64>,
    );

    api_get!(
        /// Get information on a type
        get_type,
        "GetUniverseTypesTypeId",
        RequestType::Public,
        Type,
        (type_id: i64) => "{type_id}"
    );

    api_get!(
        /// Information about a station
        get_station,
        "GetUniverseStationsStationId",
        RequestType::Public,
        Station,
        (station_id: i64) => "{station_id}"
    );

    api_get!(
        /// Returns information on requested structure if you are on the ACL. Otherwise, returns “Forbidden” for all inputs.
        get_structure,
        "GetUniverseStructuresStructureId",
        RequestType::Authenticated,
        Structure,
        (structure_id: i64) => "{structure_id}"
    );

    api_post!(
        /// Get IDs from a list of names
        get_ids,
        "PostUniverseIds",
        RequestType::Public,
        Ids,
        ,
        names: &[&str],
    );
}

/// A character ancestry.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct Ancestry {
    pub bloodline_id: i64,
    pub description: String,
    pub icon_id: Option<i64>,
    pub id: i64,
    pub name: String,
    pub short_description: Option<String>,
}

/// An asteroid belt.
#[derive(Debug, Clone, Deserialize)]
#[allow(missing_docs)]
pub struct AsteroidBelt {
    pub name: String,
    pub position: Position,
    pub system_id: i64,
}

/// A character bloodline.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct Bloodline {
    pub bloodline_id: i64,
    pub charisma: i64,
    pub corporation_id: i64,
    pub description: String,
    pub intelligence: i64,
    pub memory: i64,
    pub name: String,
    pub perception: i64,
    pub race_id: i64,
    pub ship_type_id: i64,
    pub willpower: i64,
}

/// An NPC faction.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct UniverseFaction {
    pub corporation_id: Option<i64>,
    pub description: String,
    pub faction_id: i64,
    pub is_unique: bool,
    pub militia_corporation_id: Option<i64>,
    pub name: String,
    pub size_factor: f64,
    pub solar_system_id: Option<i64>,
    pub station_count: i64,
    pub station_system_count: i64,
}

/// Rendering information of a graphic.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct Graphic {
    pub collision_file: Option<String>,
    pub graphic_file: Option<String>,
    pub graphic_id: i64,
    pub icon_folder: Option<String>,
    pub sof_dna: Option<String>,
    /// Named `sof_fation_name` (sic) in the ESI spec.
    pub sof_fation_name: Option<String>,
    pub sof_hull_name: Option<String>,
    pub sof_race_name: Option<String>,
}

/// A moon.
#[derive(Debug, Clone, Deserialize)]
#[allow(missing_docs)]
pub struct Moon {
    pub moon_id: i64,
    pub name: String,
    pub position: Position,
    pub system_id: i64,
}

/// A planet.
#[derive(Debug, Clone, Deserialize)]
#[allow(missing_docs)]
pub struct PlanetInfo {
    pub name: String,
    pub planet_id: i64,
    pub position: Position,
    pub system_id: i64,
    pub type_id: i64,
}

/// A playable race.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct Race {
    pub alliance_id: i64,
    pub description: String,
    pub name: String,
    pub race_id: i64,
}

/// The other end of a stargate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct StargateDestination {
    pub stargate_id: i64,
    pub system_id: i64,
}

/// A stargate.
#[derive(Debug, Clone, Deserialize)]
#[allow(missing_docs)]
pub struct Stargate {
    pub destination: StargateDestination,
    pub name: String,
    pub position: Position,
    pub stargate_id: i64,
    pub system_id: i64,
    pub type_id: i64,
}

/// A star.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct Star {
    pub age: i64,
    pub luminosity: f64,
    pub name: String,
    pub radius: i64,
    pub solar_system_id: i64,
    /// Spectral class such as `K2 V`, as published by ESI.
    pub spectral_class: String,
    pub temperature: i64,
    pub type_id: i64,
}

/// Jumps through a system in the last hour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct SystemJumps {
    pub ship_jumps: i64,
    pub system_id: i64,
}

/// Kills in a system in the last hour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct SystemKills {
    pub npc_kills: i64,
    pub pod_kills: i64,
    pub ship_kills: i64,
    pub system_id: i64,
}

/// Which public structures to list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureFilter {
    /// Structures with a market.
    Market,
    /// Structures with basic manufacturing.
    ManufacturingBasic,
}

impl std::fmt::Display for StructureFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            StructureFilter::Market => "market",
            StructureFilter::ManufacturingBasic => "manufacturing_basic",
        })
    }
}

/// The kind of entity a resolved name belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NameCategory {
    /// An alliance.
    Alliance,
    /// A character.
    Character,
    /// A constellation.
    Constellation,
    /// A corporation.
    Corporation,
    /// An inventory type.
    InventoryType,
    /// A region.
    Region,
    /// A solar system.
    SolarSystem,
    /// A station.
    Station,
    /// A faction.
    Faction,
    /// A category this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// A name resolved from an ID.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct ResolvedName {
    pub category: NameCategory,
    pub id: i64,
    pub name: String,
}

impl UniverseGroup<'_> {
    api_get!(
        /// List the character ancestries.
        get_ancestries,
        "GetUniverseAncestries",
        RequestType::Public,
        Vec<Ancestry>,
    );

    api_get!(
        /// Get information about an asteroid belt.
        get_asteroid_belt,
        "GetUniverseAsteroidBeltsAsteroidBeltId",
        RequestType::Public,
        AsteroidBelt,
        (asteroid_belt_id: i64) => "{asteroid_belt_id}"
    );

    api_get!(
        /// List the character bloodlines.
        get_bloodlines,
        "GetUniverseBloodlines",
        RequestType::Public,
        Vec<Bloodline>,
    );

    api_get!(
        /// List the IDs of all item categories.
        get_category_ids,
        "GetUniverseCategories",
        RequestType::Public,
        Vec<i64>,
    );

    api_get!(
        /// List the NPC factions.
        get_factions,
        "GetUniverseFactions",
        RequestType::Public,
        Vec<UniverseFaction>,
    );

    api_get!(
        /// List the IDs of all graphics.
        get_graphic_ids,
        "GetUniverseGraphics",
        RequestType::Public,
        Vec<i64>,
    );

    api_get!(
        /// Get information about a graphic.
        get_graphic,
        "GetUniverseGraphicsGraphicId",
        RequestType::Public,
        Graphic,
        (graphic_id: i64) => "{graphic_id}"
    );

    api_get!(
        /// List the IDs of all item groups.
        get_group_ids,
        "GetUniverseGroups",
        RequestType::Public,
        Vec<i64>,
        ;
        Optional(page: i32) => "page"
    );

    api_get!(
        /// Get information about a moon.
        get_moon,
        "GetUniverseMoonsMoonId",
        RequestType::Public,
        Moon,
        (moon_id: i64) => "{moon_id}"
    );

    api_get!(
        /// Get information about a planet.
        get_planet,
        "GetUniversePlanetsPlanetId",
        RequestType::Public,
        PlanetInfo,
        (planet_id: i64) => "{planet_id}"
    );

    api_get!(
        /// List the playable races.
        get_races,
        "GetUniverseRaces",
        RequestType::Public,
        Vec<Race>,
    );

    api_get!(
        /// Get information about a stargate.
        get_stargate,
        "GetUniverseStargatesStargateId",
        RequestType::Public,
        Stargate,
        (stargate_id: i64) => "{stargate_id}"
    );

    api_get!(
        /// Get information about a star.
        get_star,
        "GetUniverseStarsStarId",
        RequestType::Public,
        Star,
        (star_id: i64) => "{star_id}"
    );

    api_get!(
        /// List the IDs of public structures, optionally filtered.
        get_structure_ids,
        "GetUniverseStructures",
        RequestType::Public,
        Vec<i64>,
        ;
        Optional(filter: StructureFilter) => "filter"
    );

    api_get!(
        /// List the number of ship jumps per system in the last hour.
        get_system_jumps,
        "GetUniverseSystemJumps",
        RequestType::Public,
        Vec<SystemJumps>,
    );

    api_get!(
        /// List the number of kills per system in the last hour.
        get_system_kills,
        "GetUniverseSystemKills",
        RequestType::Public,
        Vec<SystemKills>,
    );

    api_post!(
        /// Resolve a list of IDs to names and categories.
        get_names,
        "PostUniverseNames",
        RequestType::Public,
        Vec<ResolvedName>,
        ; Chunked(ids: &[i64], 1000)
    );
}

#[cfg(test)]
mod new_endpoint_tests {
    use super::{NameCategory, ResolvedName, Star, StructureFilter};

    #[test]
    fn test_parse_star_with_large_values() {
        let json = r#"{"age": 4500999969, "luminosity": 0.5, "name": "Jita - Star",
            "radius": 2284000000, "solar_system_id": 30000142,
            "spectral_class": "K2 V", "temperature": 4567, "type_id": 45041}"#;
        let star: Star = serde_json::from_str(json).unwrap();
        assert_eq!(star.radius, 2_284_000_000);
        assert_eq!(star.spectral_class, "K2 V");
    }

    #[test]
    fn test_parse_resolved_names() {
        let json = r#"[{"category": "inventory_type", "id": 34, "name": "Tritanium"},
            {"category": "something_new", "id": 1, "name": "?"}]"#;
        let names: Vec<ResolvedName> = serde_json::from_str(json).unwrap();
        assert_eq!(names[0].category, NameCategory::InventoryType);
        assert_eq!(names[1].category, NameCategory::Unrecognized);
    }

    #[test]
    fn test_structure_filter_query_value() {
        assert_eq!(StructureFilter::Market.to_string(), "market");
        assert_eq!(
            StructureFilter::ManufacturingBasic.to_string(),
            "manufacturing_basic"
        );
    }
}
