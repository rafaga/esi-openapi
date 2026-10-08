use crate::groups::Page;
use crate::prelude::*;
use uuid::Uuid;

/// Endpoints for Military Campaigns
pub struct MilitaryCampaignsGroup<'a> {
    pub(crate) esi: &'a Esi,
}

/// The state of a campaign or objective.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum CampaignState {
    /// The state is not specified.
    Unspecified,
    /// In progress.
    Active,
    /// Completed.
    Completed,
    /// Expired.
    Expired,
    /// A state this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// A military campaign.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct Campaign {
    pub finished: Option<String>,
    pub id: Uuid,
    pub progress: i64,
    pub started: Option<String>,
    pub state: CampaignState,
}

/// Participation counts of an objective.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct ObjectiveParticipants {
    pub committed: i64,
    pub contributors: i64,
    pub total: i64,
}

/// An objective of a campaign.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct Objective {
    pub finished: Option<String>,
    pub id: Uuid,
    pub last_modified: String,
    pub participants: ObjectiveParticipants,
    pub progress: i64,
    pub started: Option<String>,
    pub state: CampaignState,
}

#[derive(Debug, Deserialize)]
struct CampaignList {
    campaigns: Vec<Campaign>,
}

impl MilitaryCampaignsGroup<'_> {
    /// List the military campaigns.
    pub async fn list(&self) -> EsiResult<Vec<Campaign>> {
        let path = self
            .esi
            .get_endpoint_for_op_id("GetMilitaryCampaignsListing")?;
        let wrapper: CampaignList = self
            .esi
            .query("GET", RequestType::Public, &path, None, None)
            .await?;
        Ok(wrapper.campaigns)
    }

    api_get!(
        /// Get a military campaign.
        get_campaign,
        "GetMilitaryCampaignsDetail",
        RequestType::Public,
        Campaign,
        (campaign_id: Uuid) => "{campaign_id}"
    );

    api_get!(
        /// List the objectives of a campaign. Use the cursor of the returned page
        /// as `after` or `before` to walk through the list.
        list_objectives,
        "GetMilitaryCampaignsObjectivesListing",
        RequestType::Public,
        Page<Objective>,
        (campaign_id: Uuid) => "{campaign_id}";
        Optional(after: String) => "after",
        Optional(before: String) => "before",
        Optional(limit: i64) => "limit"
    );

    api_get!(
        /// Get an objective of a campaign.
        get_objective,
        "GetMilitaryCampaignsObjectivesDetail",
        RequestType::Public,
        Objective,
        (campaign_id: Uuid) => "{campaign_id}",
        (objective_id: Uuid) => "{objective_id}"
    );
}

#[cfg(test)]
mod tests {
    use super::{Campaign, CampaignState};
    use crate::groups::{Objective, Page};

    #[test]
    fn test_parse_campaign() {
        let json =
            r#"{"id": "3868eaed-8278-4cb7-9709-7d7de9c20dc7", "progress": 40, "state": "Active"}"#;
        let campaign: Campaign = serde_json::from_str(json).unwrap();
        assert_eq!(campaign.state, CampaignState::Active);
        assert!(campaign.started.is_none());
    }

    #[test]
    fn test_parse_objectives_page_without_cursor() {
        let json = r#"{"objectives": [{
            "id": "3868eaed-8278-4cb7-9709-7d7de9c20dc7",
            "last_modified": "2026-10-01T00:00:00Z",
            "participants": {"committed": 1, "contributors": 2, "total": 3},
            "progress": 5, "state": "Completed"
        }]}"#;
        let page: Page<Objective> = serde_json::from_str(json).unwrap();
        assert!(page.cursor.is_none());
        assert_eq!(page.items[0].participants.total, 3);
    }
}

/// The participation of a character in a campaign objective.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct CharacterObjective {
    pub campaign_id: Uuid,
    pub contributed: i64,
    pub id: Uuid,
    pub is_committed: bool,
    pub last_modified: String,
}

impl MilitaryCampaignsGroup<'_> {
    api_get!(
        /// List the campaign objectives a character takes part in.
        list_character_objectives,
        "GetCharactersMilitaryCampaignsObjectivesListing",
        RequestType::Authenticated,
        Page<CharacterObjective>,
        (character_id: i64) => "{character_id}";
        Optional(after: String) => "after",
        Optional(before: String) => "before",
        Optional(limit: i64) => "limit"
    );

    api_get!(
        /// Get the participation of a character in one objective.
        get_character_objective,
        "GetCharactersMilitaryCampaignsObjectivesParticipation",
        RequestType::Authenticated,
        CharacterObjective,
        (character_id: i64) => "{character_id}",
        (objective_id: Uuid) => "{objective_id}"
    );
}
