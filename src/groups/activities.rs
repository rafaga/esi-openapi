use crate::prelude::*;
use uuid::Uuid;

/// Endpoints for Activities
pub struct ActivitiesGroup<'a> {
    pub(crate) esi: &'a Esi,
}

/// A mercenary tactical operation of a character, as listed.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TacticalOperationSummary {
    /// ID of the operation.
    pub id: Uuid,
    /// ID of the mercenary den that hosts the operation.
    pub mercenary_den_id: i64,
}

#[derive(Debug, Deserialize)]
struct TacticalOperations {
    operations: Vec<TacticalOperationSummary>,
}

/// State of a mercenary tactical operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum TacticalOperationState {
    /// No state specified.
    Unspecified,
    /// Available to start.
    Available,
    /// Started.
    Started,
    /// Completed.
    Completed,
    /// Expired.
    Expired,
    /// Removed.
    Removed,
    /// A state this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// A mercenary tactical operation of a character.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct TacticalOperation {
    pub dungeon_type_id: i64,
    pub expires: String,
    pub id: Uuid,
    pub mercenary_den_id: i64,
    pub state: TacticalOperationState,
}

/// The window in which a skyhook can be raided.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TheftVulnerability {
    /// When the window closes.
    pub end: String,
    /// When the window opens.
    pub start: String,
}

/// A skyhook that can currently be raided.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct RaidableSkyhook {
    pub planet_id: i64,
    pub solar_system_id: i64,
    pub theft_vulnerability: TheftVulnerability,
}

#[derive(Debug, Deserialize)]
struct RaidableSkyhooks {
    skyhooks: Vec<RaidableSkyhook>,
}

impl ActivitiesGroup<'_> {
    /// List the mercenary tactical operations of a character.
    pub async fn get_character_tactical_operations(
        &self,
        character_id: i64,
    ) -> EsiResult<Vec<TacticalOperationSummary>> {
        let path = self
            .esi
            .get_endpoint_for_op_id("GetCharactersMercenaryTacticalOperationsListing")?
            .replace("{character_id}", &character_id.to_string());
        let wrapper: TacticalOperations = self
            .esi
            .query("GET", RequestType::Authenticated, &path, None, None)
            .await?;
        Ok(wrapper.operations)
    }

    api_get!(
        /// Get a mercenary tactical operation of a character.
        get_character_tactical_operation,
        "GetCharactersMercenaryTacticalOperationsDetail",
        RequestType::Authenticated,
        TacticalOperation,
        (character_id: i64) => "{character_id}",
        (operation_id: Uuid) => "{operation_id}"
    );

    /// List the skyhooks that are currently vulnerable to theft.
    pub async fn get_raidable_skyhooks(&self) -> EsiResult<Vec<RaidableSkyhook>> {
        let path = self.esi.get_endpoint_for_op_id("GetSkyhooksRaidable")?;
        let wrapper: RaidableSkyhooks = self
            .esi
            .query("GET", RequestType::Public, &path, None, None)
            .await?;
        Ok(wrapper.skyhooks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_operation_and_skyhook() {
        let op: TacticalOperation = serde_json::from_str(
            r#"{"dungeon_type_id":1,"expires":"x","id":"67e6b7a0-6f58-4b5b-a2c4-4f3a1f0f2b11",
            "mercenary_den_id":2,"state":"Weird"}"#,
        )
        .unwrap();
        assert_eq!(op.state, TacticalOperationState::Unrecognized);
        let hooks: RaidableSkyhooks = serde_json::from_str(
            r#"{"skyhooks":[{"planet_id":1,"solar_system_id":2,
            "theft_vulnerability":{"start":"a","end":"b"}}]}"#,
        )
        .unwrap();
        assert_eq!(hooks.skyhooks[0].solar_system_id, 2);
    }
}
