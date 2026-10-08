use crate::prelude::*;

/// Endpoints for PlanetaryInteraction
pub struct PlanetaryInteractionGroup<'a> {
    pub(crate) esi: &'a Esi,
}

/// Information about a planetary production schematic.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct Schematic {
    /// Time in seconds to process one cycle.
    pub cycle_time: i64,
    pub schematic_name: String,
}

/// The type of a planet that supports planetary interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanetType {
    /// Temperate.
    Temperate,
    /// Barren.
    Barren,
    /// Oceanic.
    Oceanic,
    /// Ice.
    Ice,
    /// Gas.
    Gas,
    /// Lava.
    Lava,
    /// Storm.
    Storm,
    /// Plasma.
    Plasma,
    /// A value this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// A planet where a character has a colony.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct CharacterPlanet {
    /// Date of the last update of the colony.
    pub last_update: String,
    /// Number of pins in the colony.
    pub num_pins: i64,
    /// Character ID of the owner.
    pub owner_id: i64,
    /// ID of the planet.
    pub planet_id: i64,
    /// Type of the planet.
    pub planet_type: PlanetType,
    /// Solar system of the planet.
    pub solar_system_id: i64,
    /// Command center upgrade level.
    pub upgrade_level: i64,
}

/// A link between two pins of a colony.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ColonyLink {
    /// Pin the link ends at.
    pub destination_pin_id: i64,
    /// Level of the link.
    pub link_level: i64,
    /// Pin the link starts at.
    pub source_pin_id: i64,
}

/// An amount of a commodity stored in a pin.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PinContent {
    /// Amount stored.
    pub amount: i64,
    /// Type ID of the commodity.
    pub type_id: i64,
}

/// A head of an extractor control unit.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ExtractorHead {
    /// ID of the head.
    pub head_id: i64,
    /// Latitude of the head.
    pub latitude: f64,
    /// Longitude of the head.
    pub longitude: f64,
}

/// Details of an extractor control unit.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ExtractorDetails {
    /// Cycle time in seconds.
    pub cycle_time: Option<i64>,
    /// Radius of the heads.
    pub head_radius: Option<f64>,
    /// Heads of the control unit.
    #[serde(default)]
    pub heads: Vec<ExtractorHead>,
    /// Type ID of the extracted product.
    pub product_type_id: Option<i64>,
    /// Quantity extracted per cycle.
    pub qty_per_cycle: Option<i64>,
}

/// Details of a factory.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct FactoryDetails {
    /// ID of the schematic.
    pub schematic_id: i64,
}

/// A pin (structure) of a colony.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ColonyPin {
    /// Contents of the pin.
    #[serde(default)]
    pub contents: Vec<PinContent>,
    /// Expiry time of the pin.
    pub expiry_time: Option<String>,
    /// Extractor details, for extractor control units.
    pub extractor_details: Option<ExtractorDetails>,
    /// Factory details, for factories.
    pub factory_details: Option<FactoryDetails>,
    /// Installation time.
    pub install_time: Option<String>,
    /// Start of the last cycle.
    pub last_cycle_start: Option<String>,
    /// Latitude of the pin.
    pub latitude: f64,
    /// Longitude of the pin.
    pub longitude: f64,
    /// ID of the pin.
    pub pin_id: i64,
    /// ID of the schematic, for factories.
    pub schematic_id: Option<i64>,
    /// Type ID of the pin.
    pub type_id: i64,
}

/// A route of commodities between two pins.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ColonyRoute {
    /// Type ID of the commodity moved by the route.
    pub content_type_id: i64,
    /// Pin the route ends at.
    pub destination_pin_id: i64,
    /// Quantity moved.
    pub quantity: f64,
    /// ID of the route.
    pub route_id: i64,
    /// Pin the route starts at.
    pub source_pin_id: i64,
    /// Pins the route passes through.
    #[serde(default)]
    pub waypoints: Vec<i64>,
}

/// The layout of a colony.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Colony {
    /// Links between pins.
    pub links: Vec<ColonyLink>,
    /// Pins of the colony.
    pub pins: Vec<ColonyPin>,
    /// Routes of the colony.
    pub routes: Vec<ColonyRoute>,
}

/// The standing level that decides a customs office tax rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StandingLevel {
    /// Bad standing.
    Bad,
    /// Excellent standing.
    Excellent,
    /// Good standing.
    Good,
    /// Neutral standing.
    Neutral,
    /// Terrible standing.
    Terrible,
    /// A value this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// A customs office of a corporation.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CustomsOffice {
    /// Tax rate for alliance members.
    pub alliance_tax_rate: Option<f64>,
    /// Whether access is allowed based on standings.
    pub allow_access_with_standings: bool,
    /// Whether alliance members have access.
    pub allow_alliance_access: bool,
    /// Tax rate for bad standings.
    pub bad_standing_tax_rate: Option<f64>,
    /// Tax rate for corporation members.
    pub corporation_tax_rate: Option<f64>,
    /// Tax rate for excellent standings.
    pub excellent_standing_tax_rate: Option<f64>,
    /// Tax rate for good standings.
    pub good_standing_tax_rate: Option<f64>,
    /// Tax rate for neutral standings.
    pub neutral_standing_tax_rate: Option<f64>,
    /// ID of the office.
    pub office_id: i64,
    /// Hour the reinforcement exit window ends.
    pub reinforce_exit_end: i64,
    /// Hour the reinforcement exit window starts.
    pub reinforce_exit_start: i64,
    /// Minimum standing level allowed.
    pub standing_level: Option<StandingLevel>,
    /// Solar system of the office.
    pub system_id: i64,
    /// Tax rate for terrible standings.
    pub terrible_standing_tax_rate: Option<f64>,
    /// Type ID of the office.
    pub type_id: Option<i64>,
}

impl PlanetaryInteractionGroup<'_> {
    api_get!(
        /// Get information about a planetary production schematic.
        get_schematic,
        "GetUniverseSchematicsSchematicId",
        RequestType::Public,
        Schematic,
        (schematic_id: i64) => "{schematic_id}"
    );
    api_get!(
        /// List the planets where a character has colonies.
        get_character_planets,
        "GetCharactersCharacterIdPlanets",
        RequestType::Authenticated,
        Vec<CharacterPlanet>,
        (character_id: i64) => "{character_id}"
    );

    api_get!(
        /// Get the layout of a character's colony on a planet.
        get_character_planet,
        "GetCharactersCharacterIdPlanetsPlanetId",
        RequestType::Authenticated,
        Colony,
        (character_id: i64) => "{character_id}",
        (planet_id: i64) => "{planet_id}"
    );

    api_get!(
        /// List the customs offices of a corporation.
        ///
        /// Requires the Director role.
        get_corporation_customs_offices,
        "GetCorporationsCorporationIdCustomsOffices",
        RequestType::Authenticated,
        Vec<CustomsOffice>,
        (corporation_id: i64) => "{corporation_id}";
        Optional(page: i32) => "page"
    );
}

#[cfg(test)]
mod colony_tests {
    use super::*;

    #[test]
    fn test_parse_colony() {
        let json = r#"{"links": [{"destination_pin_id": 1000000000001, "link_level": 0, "source_pin_id": 2}],
            "pins": [{"latitude": 1.0, "longitude": 2.0, "pin_id": 1000000000001, "type_id": 2524,
                      "extractor_details": {"cycle_time": 3600, "heads": [{"head_id": 0, "latitude": 1.0, "longitude": 1.0}]},
                      "contents": [{"amount": 3000000000, "type_id": 2268}]}],
            "routes": [{"content_type_id": 2268, "destination_pin_id": 2, "quantity": 20.0,
                        "route_id": 1, "source_pin_id": 1}]}"#;
        let c: Colony = serde_json::from_str(json).unwrap();
        assert_eq!(c.pins[0].pin_id, 1_000_000_000_001);
        assert_eq!(c.pins[0].contents[0].amount, 3_000_000_000);
        assert!(c.routes[0].waypoints.is_empty());
        assert_eq!(c.pins[0].extractor_details.as_ref().unwrap().heads.len(), 1);
    }

    #[test]
    fn test_parse_planets_and_offices() {
        let p: Vec<CharacterPlanet> = serde_json::from_str(
            r#"[{"last_update": "u", "num_pins": 5, "owner_id": 2112000000, "planet_id": 40000001,
                 "planet_type": "lava", "solar_system_id": 30000142, "upgrade_level": 4},
                {"last_update": "u", "num_pins": 1, "owner_id": 1, "planet_id": 1,
                 "planet_type": "shattered", "solar_system_id": 1, "upgrade_level": 0}]"#,
        )
        .unwrap();
        assert_eq!(p[0].planet_type, PlanetType::Lava);
        assert_eq!(p[1].planet_type, PlanetType::Unrecognized);
        let o: Vec<CustomsOffice> = serde_json::from_str(
            r#"[{"allow_access_with_standings": true, "allow_alliance_access": false,
                 "office_id": 1040000000000, "reinforce_exit_end": 21, "reinforce_exit_start": 20,
                 "system_id": 30000142, "standing_level": "good", "good_standing_tax_rate": 0.05}]"#,
        )
        .unwrap();
        assert_eq!(o[0].standing_level, Some(StandingLevel::Good));
        assert_eq!(o[0].office_id, 1_040_000_000_000);
    }
}
