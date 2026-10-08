use crate::prelude::*;

/// Endpoints for Skills
pub struct SkillsGroup<'a> {
    pub(crate) esi: &'a Esi,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct Skill {
    pub skill_id: i64,
    pub active_skill_level: i64,
    pub skillpoints_in_skill: i64,
    pub trained_skill_level: i64,
}

#[derive(Debug, Deserialize)]
#[allow(missing_docs)]
pub struct Skills {
    pub skills: Vec<Skill>,
    pub total_sp: i64,
    pub unallocated_sp: Option<i64>,
}

impl SkillsGroup<'_> {
    api_get!(
        /// Get character skills.
        get_skills,
        "GetCharactersCharacterIdSkills",
        RequestType::Authenticated,
        Skills,
        (character_id: i64) => "{character_id}"
    );
}

/// A character's attributes and remap information.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct CharacterAttributes {
    pub accrued_remap_cooldown_date: Option<String>,
    pub bonus_remaps: Option<i64>,
    pub charisma: i64,
    pub intelligence: i64,
    pub last_remap_date: Option<String>,
    pub memory: i64,
    pub perception: i64,
    pub willpower: i64,
}

/// An entry of a character's skill queue.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct SkillQueueItem {
    pub finish_date: Option<String>,
    pub finished_level: i64,
    pub level_end_sp: Option<i64>,
    pub level_start_sp: Option<i64>,
    pub queue_position: i64,
    pub skill_id: i64,
    pub start_date: Option<String>,
    pub training_start_sp: Option<i64>,
}

impl SkillsGroup<'_> {
    api_get!(
        /// Get a character's attributes.
        get_attributes,
        "GetCharactersCharacterIdAttributes",
        RequestType::Authenticated,
        CharacterAttributes,
        (character_id: i64) => "{character_id}"
    );

    api_get!(
        /// Get a character's skill queue.
        get_skillqueue,
        "GetCharactersCharacterIdSkillqueue",
        RequestType::Authenticated,
        Vec<SkillQueueItem>,
        (character_id: i64) => "{character_id}"
    );
}

#[cfg(test)]
mod character_basic_tests {
    use super::SkillQueueItem;

    #[test]
    fn test_parse_paused_queue_entry() {
        let json = r#"[{"finished_level": 5, "queue_position": 0, "skill_id": 3300}]"#;
        let queue: Vec<SkillQueueItem> = serde_json::from_str(json).unwrap();
        assert!(queue[0].finish_date.is_none());
        assert_eq!(queue[0].skill_id, 3300);
    }
}
