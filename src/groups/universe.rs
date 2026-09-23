#![allow(unused)]

use crate::prelude::*;

/// Endpoints for Universe
pub struct UniverseGroup<'a> {
    pub(crate) esi: &'a Esi,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct Constellation {
    pub constellation_id: i32,
    pub name: String,
    pub position: Position,
    pub region_id: i32,
    pub systems: Vec<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct Region {
    pub constellations: Vec<i32>,
    pub description: Option<String>,
    pub name: String,
    pub region_id: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct SystemPlanet {
    pub asteroid_belts: Option<Vec<i32>>,
    pub moons: Option<Vec<i32>>,
    pub planet_id: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct System {
    pub constellation_id: i32,
    pub name: String,
    pub planets: Option<Vec<SystemPlanet>>,
    pub position: Position,
    pub security_class: Option<String>,
    pub security_status: f64,
    pub star_id: Option<i32>,
    pub stargates: Option<Vec<i32>>,
    pub stations: Option<Vec<i32>>,
    pub system_id: i32,
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
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct TypeDogmaAttribute {
    pub attribute_id: i32,
    pub value: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct TypeDogmaEffect {
    pub effect_id: i32,
    pub is_default: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct Type {
    pub capacity: Option<f64>,
    pub description: String,
    pub dogma_attributes: Option<Vec<TypeDogmaAttribute>>,
    pub dogma_effects: Option<Vec<TypeDogmaEffect>>,
    pub graphic_id: Option<i32>,
    pub group_id: i32,
    pub icon_id: Option<i32>,
    pub market_group_id: Option<i32>,
    pub mass: Option<f64>,
    pub name: String,
    pub packaged_volume: Option<f64>,
    pub portion_size: Option<i32>,
    pub published: bool,
    pub radius: Option<f64>,
    pub type_id: i32,
    pub volume: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct Station {
    pub max_dockable_ship_volume: f64,
    pub name: String,
    pub office_rental_cost: f64,
    pub owner: Option<i32>,
    pub position: Position,
    pub race_id: Option<i32>,
    pub reprocessing_efficiency: f64,
    pub reprocessing_stations_take: f64,
    pub services: Vec<String>,
    pub station_id: i32,
    pub system_id: i32,
    pub type_id: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct Structure {
    pub name: String,
    pub owner_id: i32,
    pub position: Option<Position>,
    pub solar_system_id: i32,
    pub type_id: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct CategoriesCategory {
    pub category_id: i32,
    pub groups: Vec<i32>,
    pub name: String,
    pub published: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct Group {
    pub category_id: i32,
    pub group_id: i32,
    pub name: String,
    pub published: bool,
    pub types: Vec<i32>,
}

impl UniverseGroup<'_> {
    api_get!(
        /// Get information on a category
        get_universe_categories_category,
        "GetUniverseCategoriesCategoryId",
        RequestType::Public,
        CategoriesCategory,
        (category_id: i32) => "{category_id}"
    );

    api_get!(
        /// Get information on a group
        get_universe_groups_group,
        "GetUniverseGroupsGroupId",
        RequestType::Public,
        Group,
        (group_id: i32) => "{group_id}"
    );

    api_get!(
        /// Get information on a type
        get_universe_types_type,
        "GetUniverseTypesTypeId",
        RequestType::Public,
        Type,
        (type_id: i32) => "{type_id}"
    );

    api_get!(
        /// Get a list of constellation ids
        get_constellation_ids,
        "GetUniverseConstellations",
        RequestType::Public,
        Vec<i32>,
    );

    api_get!(
        /// Get information on a constellation
        get_constellation,
        "GetUniverseConstellationsConstellationId",
        RequestType::Public,
        Constellation,
        (constellation_id: i32) => "{constellation_id}"
    );

    api_get!(
        /// Get a list of region ids
        get_region_ids,
        "GetUniverseRegions",
        RequestType::Public,
        Vec<i32>,
    );

    api_get!(
        /// Get information on a region
        get_region,
        "GetUniverseRegionsRegionId",
        RequestType::Public,
        Region,
        (region_id: i32) => "{region_id}"
    );

    api_get!(
        /// Get a list of system ids
        get_system_ids,
        "GetUniverseSystems",
        RequestType::Public,
        Vec<i32>,
    );

    api_get!(
        /// Get information on a system
        get_system,
        "GetUniverseSystemsSystemId",
        RequestType::Public,
        System,
        (system_id: i32) => "{system_id}"
    );

    api_get!(
        /// Get a list of type ids
        get_type_ids,
        "GetUniverseTypes",
        RequestType::Public,
        Vec<i32>,
    );

    api_get!(
        /// Get information on a type
        get_type,
        "GetUniverseTypesTypeId",
        RequestType::Public,
        Type,
        (type_id: i32) => "{type_id}"
    );

    api_get!(
        /// Information about a station
        get_station,
        "GetUniverseStationsStationId",
        RequestType::Public,
        Station,
        (station_id: i32) => "{station_id}"
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
