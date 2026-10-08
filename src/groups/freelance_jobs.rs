use crate::groups::Page;
use crate::prelude::*;
use uuid::Uuid;

/// Endpoints for Freelance Jobs
pub struct FreelanceJobsGroup<'a> {
    pub(crate) esi: &'a Esi,
}

/// The state of a freelance job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum JobState {
    /// The state is not specified.
    Unspecified,
    /// The job is open.
    Active,
    /// The job was closed.
    Closed,
    /// The job was completed.
    Completed,
    /// The job expired.
    Expired,
    /// The job was deleted.
    Deleted,
    /// A state this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// The career a freelance job belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum JobCareer {
    /// The career is not specified.
    Unspecified,
    /// Explorer.
    Explorer,
    /// Industrialist.
    Industrialist,
    /// Enforcer.
    Enforcer,
    /// Soldier of Fortune.
    #[serde(rename = "Soldier of Fortune")]
    SoldierOfFortune,
    /// A career this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// Progress towards completing a job.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct JobProgress {
    pub current: i64,
    pub desired: i64,
}

/// The ISK reward of a job.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct JobReward {
    pub initial: f64,
    pub remaining: f64,
}

/// A freelance job as listed by [`FreelanceJobsGroup::list`].
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct FreelanceJobSummary {
    pub id: Uuid,
    pub last_modified: String,
    pub name: String,
    pub progress: JobProgress,
    pub reward: Option<JobReward>,
    pub state: JobState,
}

/// A solar system a job is broadcast in.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct BroadcastLocation {
    /// Solar system ID.
    pub id: i64,
    pub name: String,
}

/// Age restrictions on who can take a job.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct JobRestrictions {
    pub maximum_age: Option<i64>,
    pub minimum_age: Option<i64>,
}

/// Who can see and take a job.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct AccessAndVisibility {
    pub acl_protected: bool,
    pub broadcast_locations: Option<Vec<BroadcastLocation>>,
    pub restrictions: Option<JobRestrictions>,
}

/// How a job is evaluated. `parameters` depends on `method`, so it is kept as raw JSON.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct JobConfiguration {
    pub method: String,
    pub parameters: serde_json::Value,
    pub version: i64,
}

/// Limits on participants' contributions.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct JobContribution {
    pub contribution_per_participant_limit: Option<i64>,
    pub max_committed_participants: i64,
    pub reward_per_contribution: Option<f64>,
    pub submission_limit: Option<i64>,
    pub submission_multiplier: Option<f64>,
}

/// The character that created a job.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct JobCreatorCharacter {
    pub id: i64,
    pub name: String,
}

/// The corporation that created a job.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct JobCreatorCorporation {
    pub id: i64,
    pub name: String,
}

/// Who created a job.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct JobCreator {
    pub character: JobCreatorCharacter,
    pub corporation: JobCreatorCorporation,
}

/// Descriptive details of a job.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[allow(missing_docs)]
pub struct JobDetails {
    pub career: JobCareer,
    pub created: String,
    pub creator: JobCreator,
    pub description: String,
    pub expires: Option<String>,
    pub finished: Option<String>,
}

/// Full information about a freelance job.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct FreelanceJob {
    pub access_and_visibility: AccessAndVisibility,
    pub configuration: JobConfiguration,
    pub contribution: Option<JobContribution>,
    pub details: JobDetails,
    pub id: Uuid,
    pub last_modified: String,
    pub name: String,
    pub progress: JobProgress,
    pub reward: Option<JobReward>,
    pub state: JobState,
}

impl FreelanceJobsGroup<'_> {
    api_get!(
        /// List freelance jobs. Use the cursor of the returned page as `after` or
        /// `before` to walk through the list; `corporation_id` filters by creator.
        list,
        "GetFreelanceJobsListing",
        RequestType::Public,
        Page<FreelanceJobSummary>,
        ;
        Optional(after: String) => "after",
        Optional(before: String) => "before",
        Optional(limit: i64) => "limit",
        Optional(corporation_id: i64) => "corporation_id"
    );

    api_get!(
        /// Get the details of a freelance job.
        get_job,
        "GetFreelanceJobsDetail",
        RequestType::Public,
        FreelanceJob,
        (job_id: Uuid) => "{job_id}"
    );
}

#[cfg(test)]
mod tests {
    use super::JobState;
    use crate::groups::{FreelanceJobSummary, Page};

    #[test]
    fn test_parse_page_with_unknown_state() {
        let json = r#"{
            "cursor": {"after": "abc"},
            "freelance_jobs": [{
                "id": "3868eaed-8278-4cb7-9709-7d7de9c20dc7",
                "last_modified": "2026-10-01T00:00:00Z",
                "name": "Haul",
                "progress": {"current": 1, "desired": 10},
                "state": "Archived"
            }]
        }"#;
        let page: Page<FreelanceJobSummary> = serde_json::from_str(json).unwrap();
        assert_eq!(page.items[0].state, JobState::Unrecognized);
        assert!(page.items[0].reward.is_none());
        assert_eq!(page.cursor.unwrap().after.as_deref(), Some("abc"));
    }
}

/// The state of a character's participation in a freelance job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum ParticipationState {
    /// No state specified.
    Unspecified,
    /// Committed to the job.
    Committed,
    /// Kicked from the job.
    Kicked,
    /// Resigned from the job.
    Resigned,
    /// A state this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// A character's participation in a freelance job.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct JobParticipation {
    pub contributed: i64,
    pub last_modified: String,
    pub state: ParticipationState,
}

/// A participant of a corporation's freelance job.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct JobParticipant {
    pub contributed: i64,
    /// Character ID of the participant.
    pub id: i64,
    pub name: String,
    pub state: ParticipationState,
}

#[derive(Debug, Deserialize)]
struct CharacterFreelanceJobs {
    freelance_jobs: Vec<FreelanceJobSummary>,
}

impl FreelanceJobsGroup<'_> {
    /// List the freelance jobs a character takes part in.
    pub async fn get_character_jobs(
        &self,
        character_id: i64,
    ) -> EsiResult<Vec<FreelanceJobSummary>> {
        let path = self
            .esi
            .get_endpoint_for_op_id("GetCharactersFreelanceJobsListing")?
            .replace("{character_id}", &character_id.to_string());
        let wrapper: CharacterFreelanceJobs = self
            .esi
            .query("GET", RequestType::Authenticated, &path, None, None)
            .await?;
        Ok(wrapper.freelance_jobs)
    }

    api_get!(
        /// Get the participation of a character in a freelance job.
        get_character_participation,
        "GetCharactersFreelanceJobsParticipation",
        RequestType::Authenticated,
        JobParticipation,
        (character_id: i64) => "{character_id}",
        (job_id: Uuid) => "{job_id}"
    );

    api_get!(
        /// List the freelance jobs of a corporation (requires the Project_Manager role).
        list_corporation_jobs,
        "GetCorporationsFreelanceJobsListing",
        RequestType::Authenticated,
        Page<FreelanceJobSummary>,
        (corporation_id: i64) => "{corporation_id}";
        Optional(after: String) => "after",
        Optional(before: String) => "before",
        Optional(limit: i64) => "limit"
    );

    api_get!(
        /// List the participants of a corporation's freelance job (requires the Project_Manager role).
        list_corporation_job_participants,
        "GetCorporationsFreelanceJobsParticipants",
        RequestType::Authenticated,
        Page<JobParticipant>,
        (corporation_id: i64) => "{corporation_id}",
        (job_id: Uuid) => "{job_id}";
        Optional(after: String) => "after",
        Optional(before: String) => "before",
        Optional(limit: i64) => "limit"
    );
}
