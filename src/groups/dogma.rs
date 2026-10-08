use crate::prelude::*;

/// Endpoints for Dogma
pub struct DogmaGroup<'a> {
    pub(crate) esi: &'a Esi,
}

/// Information about a dogma attribute.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct DogmaAttribute {
    pub attribute_id: i64,
    pub default_value: Option<f64>,
    pub description: Option<String>,
    pub display_name: Option<String>,
    pub high_is_good: Option<bool>,
    pub icon_id: Option<i64>,
    pub name: Option<String>,
    pub published: Option<bool>,
    pub stackable: Option<bool>,
    pub unit_id: Option<i64>,
}

/// A modifier applied by a dogma effect.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct DogmaEffectModifier {
    pub domain: Option<String>,
    pub effect_id: Option<i64>,
    pub func: String,
    pub modified_attribute_id: Option<i64>,
    pub modifying_attribute_id: Option<i64>,
    pub operator: Option<i64>,
}

/// Information about a dogma effect.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct DogmaEffect {
    pub description: Option<String>,
    pub disallow_auto_repeat: Option<bool>,
    pub discharge_attribute_id: Option<i64>,
    pub display_name: Option<String>,
    pub duration_attribute_id: Option<i64>,
    pub effect_category: Option<i64>,
    pub effect_id: i64,
    pub electronic_chance: Option<bool>,
    pub falloff_attribute_id: Option<i64>,
    pub icon_id: Option<i64>,
    pub is_assistance: Option<bool>,
    pub is_offensive: Option<bool>,
    pub is_warp_safe: Option<bool>,
    pub modifiers: Option<Vec<DogmaEffectModifier>>,
    pub name: Option<String>,
    pub post_expression: Option<i64>,
    pub pre_expression: Option<i64>,
    pub published: Option<bool>,
    pub range_attribute_id: Option<i64>,
    pub range_chance: Option<bool>,
    pub tracking_speed_attribute_id: Option<i64>,
}

/// An attribute value of a dynamic (mutated) item.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct DynamicItemAttribute {
    pub attribute_id: i64,
    pub value: f64,
}

/// An effect of a dynamic (mutated) item.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct DynamicItemEffect {
    pub effect_id: i64,
    pub is_default: bool,
}

/// Dynamic (mutated) item information.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct DynamicItem {
    /// Character ID of the creator.
    pub created_by: i64,
    pub dogma_attributes: Vec<DynamicItemAttribute>,
    pub dogma_effects: Vec<DynamicItemEffect>,
    pub mutator_type_id: i64,
    pub source_type_id: i64,
}

impl DogmaGroup<'_> {
    api_get!(
        /// List the IDs of all dogma attributes.
        get_attributes,
        "GetDogmaAttributes",
        RequestType::Public,
        Vec<i64>,
    );

    api_get!(
        /// Get information about a dogma attribute.
        get_attribute,
        "GetDogmaAttributesAttributeId",
        RequestType::Public,
        DogmaAttribute,
        (attribute_id: i64) => "{attribute_id}"
    );

    api_get!(
        /// Get information about a dynamic (mutated) item.
        get_dynamic_item,
        "GetDogmaDynamicItemsTypeIdItemId",
        RequestType::Public,
        DynamicItem,
        (type_id: i64) => "{type_id}",
        (item_id: i64) => "{item_id}"
    );

    api_get!(
        /// List the IDs of all dogma effects.
        get_effects,
        "GetDogmaEffects",
        RequestType::Public,
        Vec<i64>,
    );

    api_get!(
        /// Get information about a dogma effect.
        get_effect,
        "GetDogmaEffectsEffectId",
        RequestType::Public,
        DogmaEffect,
        (effect_id: i64) => "{effect_id}"
    );
}

#[cfg(test)]
mod tests {
    use super::{DogmaEffect, DynamicItem};

    #[test]
    fn test_parse_dynamic_item() {
        let json = r#"{
            "created_by": 2112000000,
            "dogma_attributes": [{"attribute_id": 9, "value": 1.5}],
            "dogma_effects": [{"effect_id": 11, "is_default": true}],
            "mutator_type_id": 49730,
            "source_type_id": 2048
        }"#;
        let item: DynamicItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.created_by, 2_112_000_000);
        assert!(item.dogma_effects[0].is_default);
    }

    #[test]
    fn test_parse_effect_with_only_required_fields() {
        let effect: DogmaEffect = serde_json::from_str(r#"{"effect_id": 11}"#).unwrap();
        assert_eq!(effect.effect_id, 11);
        assert!(effect.modifiers.is_none());
    }
}
