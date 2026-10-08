use crate::prelude::*;

/// Endpoints for Cosmetics
pub struct CosmeticsGroup<'a> {
    pub(crate) esi: &'a Esi,
}

/// How a pattern is blended with the layers below it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternBlendMode {
    /// Normal blending.
    Normal,
    /// Subtract.
    Subtract,
    /// Exclusion.
    Exclusion,
    /// Nested.
    Nested,
    /// Nested, inverted.
    NestedInverted,
    /// A mode this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// A 3-component vector.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// A 4-component vector (a quaternion for rotations).
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct Vector4 {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Which slots a pattern is projected onto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct PatternProjection {
    pub slot1: bool,
    pub slot2: bool,
    pub slot3: bool,
    pub slot4: bool,
}

/// Transformation applied to a pattern.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct PatternTransform {
    pub position: Vector3,
    pub rotation: Vector4,
    pub scaling: Vector3,
}

/// Configuration of a pattern component.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct PatternConfiguration {
    pub mirrored: bool,
    pub projection: PatternProjection,
    pub transform: PatternTransform,
}

/// A nanocoating component (see the SDE for details).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct NanocoatingComponent {
    pub id: i64,
}

/// A pattern component (see the SDE for details).
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct PatternComponent {
    pub configuration: PatternConfiguration,
    pub id: i64,
}

/// What a slot of a SKINR design holds.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(untagged)]
#[allow(missing_docs)]
pub enum SlotConfiguration {
    Nanocoating { nanocoating: NanocoatingComponent },
    Pattern { pattern: PatternComponent },
}

/// A slot of a SKINR design.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct SkinrSlot {
    pub configuration: SlotConfiguration,
    pub id: i64,
}

/// Layout of a SKINR design.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct SkinrLayout {
    pub pattern_blend_mode: PatternBlendMode,
    pub slots: Vec<SkinrSlot>,
}

/// Tier of a SKINR design.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct SkinrTier {
    pub level: i64,
}

/// A SKINR design.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct Skinr {
    /// Character ID of the creator.
    pub creator_id: i64,
    pub id: String,
    pub layout: SkinrLayout,
    pub line: Option<String>,
    pub name: String,
    pub ship_type_id: i64,
    pub tier: SkinrTier,
}

impl CosmeticsGroup<'_> {
    api_get!(
        /// Get a SKINR design.
        get_skinr,
        "GetCosmeticsSkinr",
        RequestType::Public,
        Skinr,
        (skinr_id: &str) => "{skinr_id}"
    );
}

#[cfg(test)]
mod tests {
    use super::{Skinr, SlotConfiguration};
    use serde_json::json;

    #[test]
    fn test_parse_skinr_with_both_slot_kinds() {
        let vec3 = json!({"x": 0.0, "y": 0.0, "z": 0.0});
        let value = json!({
            "creator_id": 2_112_000_000_i64,
            "id": "abc",
            "name": "Design",
            "ship_type_id": 587,
            "tier": {"level": 2},
            "layout": {
                "pattern_blend_mode": "nested_inverted",
                "slots": [
                    {"id": 1, "configuration": {"nanocoating": {"id": 10}}},
                    {"id": 2, "configuration": {"pattern": {
                        "id": 20,
                        "configuration": {
                            "mirrored": false,
                            "projection": {
                                "slot1": true, "slot2": false, "slot3": false, "slot4": false
                            },
                            "transform": {
                                "position": vec3,
                                "scaling": vec3,
                                "rotation": {"w": 1.0, "x": 0.0, "y": 0.0, "z": 0.0}
                            }
                        }
                    }}}
                ]
            }
        });
        let skinr: Skinr = serde_json::from_value(value).unwrap();
        assert!(skinr.line.is_none());
        assert!(matches!(
            skinr.layout.slots[0].configuration,
            SlotConfiguration::Nanocoating { .. }
        ));
        assert!(matches!(
            skinr.layout.slots[1].configuration,
            SlotConfiguration::Pattern { .. }
        ));
    }
}

/// A SKINR license of a character.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct CharacterSkinrLicense {
    /// Whether the license is activated.
    pub activated: bool,
    /// ID of the SKINR design.
    pub skinr_id: String,
    /// Number of unactivated licenses.
    pub unactivated: i64,
}

#[derive(Debug, Deserialize)]
struct SkinrLicenses {
    licenses: Vec<CharacterSkinrLicense>,
}

/// Kind of a SKINR component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SkinrComponentType {
    /// A nanocoating.
    Nanocoating,
    /// A pattern.
    Pattern,
    /// A kind this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// How many times a component can be used: either `remaining` or `unlimited` is set.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ComponentRuns {
    /// Remaining uses.
    pub remaining: Option<i64>,
    /// Whether the component has unlimited uses.
    pub unlimited: Option<bool>,
}

/// A SKINR component license of a character.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct SkinrComponentLicense {
    pub component_id: i64,
    pub runs: ComponentRuns,
    #[serde(rename = "type")]
    pub component_type: SkinrComponentType,
}

#[derive(Debug, Deserialize)]
struct SkinrComponentLicenses {
    licenses: Vec<SkinrComponentLicense>,
}

impl CosmeticsGroup<'_> {
    /// List the SKINR licenses of a character.
    pub async fn get_character_skinr(
        &self,
        character_id: i64,
    ) -> EsiResult<Vec<CharacterSkinrLicense>> {
        let path = self
            .esi
            .get_endpoint_for_op_id("GetCharactersCosmeticsSkinr")?
            .replace("{character_id}", &character_id.to_string());
        let wrapper: SkinrLicenses = self
            .esi
            .query("GET", RequestType::Authenticated, &path, None, None)
            .await?;
        Ok(wrapper.licenses)
    }

    /// List the SKINR component licenses of a character.
    pub async fn get_character_skinr_components(
        &self,
        character_id: i64,
    ) -> EsiResult<Vec<SkinrComponentLicense>> {
        let path = self
            .esi
            .get_endpoint_for_op_id("GetCharactersCosmeticsSkinrComponents")?
            .replace("{character_id}", &character_id.to_string());
        let wrapper: SkinrComponentLicenses = self
            .esi
            .query("GET", RequestType::Authenticated, &path, None, None)
            .await?;
        Ok(wrapper.licenses)
    }
}

#[cfg(test)]
mod license_tests {
    use super::*;

    #[test]
    fn test_parse_component_licenses() {
        let w: SkinrComponentLicenses = serde_json::from_str(
            r#"{"licenses":[{"component_id":1,"type":"pattern","runs":{"unlimited":true}},
            {"component_id":2,"type":"x","runs":{"remaining":3}}]}"#,
        )
        .unwrap();
        assert_eq!(w.licenses[0].runs.unlimited, Some(true));
        assert_eq!(w.licenses[1].runs.remaining, Some(3));
        assert_eq!(
            w.licenses[1].component_type,
            SkinrComponentType::Unrecognized
        );
    }
}
