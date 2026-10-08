use crate::prelude::*;

/// Endpoints for Sovereignty
pub struct SovereigntyGroup<'a> {
    pub(crate) esi: &'a Esi,
}

/// A participant in a sovereignty campaign.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct CampaignParticipant {
    pub alliance_id: i64,
    pub score: f64,
}

/// The kind of structure a sovereignty campaign is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CampaignEventType {
    /// Defense of a territorial claim unit.
    TcuDefense,
    /// Defense of an infrastructure hub.
    IhubDefense,
    /// Defense of a station.
    StationDefense,
    /// Freeport of a station.
    StationFreeport,
    /// A type this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// An active sovereignty campaign.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct SovereigntyCampaign {
    pub attackers_score: Option<f64>,
    pub campaign_id: i64,
    pub constellation_id: i64,
    pub defender_id: Option<i64>,
    pub defender_score: Option<f64>,
    pub event_type: CampaignEventType,
    pub participants: Option<Vec<CampaignParticipant>>,
    pub solar_system_id: i64,
    pub start_time: String,
    pub structure_id: i64,
}

/// The window in which a sovereignty hub can be attacked.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct VulnerabilityWindow {
    pub end: String,
    pub start: String,
}

/// The sovereignty hub of a claimed system.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct SovereigntyHub {
    pub id: i64,
    pub vulnerability_window: Option<VulnerabilityWindow>,
}

/// Development levels of a claimed system.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct SystemDevelopment {
    pub activity_defense_multiplier: f64,
    pub industrial_level: i64,
    pub military_level: i64,
    pub strategic_level: i64,
}

/// A claim held by an alliance.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct AllianceClaim {
    pub alliance_id: i64,
    pub claimed_since: String,
    pub corporation_id: i64,
    pub development: SystemDevelopment,
    pub is_capital_system: bool,
    pub sovereignty_hub: SovereigntyHub,
}

/// A claim held by a faction.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct FactionClaim {
    pub faction_id: i64,
}

/// Who holds a solar system.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
#[allow(missing_docs)]
pub enum SovereigntyClaim {
    Faction { faction: FactionClaim },
    Alliance { alliance: AllianceClaim },
    Unclaimed { unclaimed: bool },
}

/// Sovereignty state of one solar system.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct SovereigntySystem {
    pub claim: SovereigntyClaim,
    pub solar_system_id: i64,
}

#[derive(Debug, Deserialize)]
struct SovereigntySystems {
    solar_systems: Vec<SovereigntySystem>,
}

impl SovereigntyGroup<'_> {
    api_get!(
        /// List active sovereignty campaigns.
        get_campaigns,
        "GetSovereigntyCampaigns",
        RequestType::Public,
        Vec<SovereigntyCampaign>,
    );

    /// List the sovereignty state of every solar system.
    pub async fn get_systems(&self) -> EsiResult<Vec<SovereigntySystem>> {
        let path = self.esi.get_endpoint_for_op_id("GetSovereigntySystems")?;
        let wrapper: SovereigntySystems = self
            .esi
            .query("GET", RequestType::Public, &path, None, None)
            .await?;
        Ok(wrapper.solar_systems)
    }
}

#[cfg(test)]
mod tests {
    use super::{SovereigntyClaim, SovereigntySystem};

    #[test]
    fn test_parse_each_claim_kind() {
        let faction: SovereigntySystem = serde_json::from_str(
            r#"{"solar_system_id":1,"claim":{"faction":{"faction_id":500001}}}"#,
        )
        .unwrap();
        assert!(matches!(faction.claim, SovereigntyClaim::Faction { .. }));

        let none: SovereigntySystem =
            serde_json::from_str(r#"{"solar_system_id":2,"claim":{"unclaimed":true}}"#).unwrap();
        assert!(matches!(
            none.claim,
            SovereigntyClaim::Unclaimed { unclaimed: true }
        ));

        let alliance: SovereigntySystem = serde_json::from_str(
            r#"{"solar_system_id":3,"claim":{"alliance":{
                "alliance_id":1,"corporation_id":2,"claimed_since":"2026-01-01T00:00:00Z",
                "is_capital_system":false,
                "development":{"activity_defense_multiplier":1.0,"industrial_level":1,"military_level":2,"strategic_level":3},
                "sovereignty_hub":{"id":9}}}}"#,
        )
        .unwrap();
        assert!(matches!(alliance.claim, SovereigntyClaim::Alliance { .. }));
    }
}
