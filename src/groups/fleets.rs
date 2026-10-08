#![allow(unused)]

use crate::prelude::*;

/// Endpoints for Fleets
pub struct FleetsGroup<'a> {
    pub(crate) esi: &'a Esi,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
/// The role of a character in a fleet.
pub enum FleetRole {
    /// Fleet commander.
    #[serde(rename = "fleet_commander")]
    FleetCommander,
    /// Wing commander.
    #[serde(rename = "wing_commander")]
    WingCommander,
    /// Squad commander.
    #[serde(rename = "squad_commander")]
    SquadCommander,
    /// Squad member.
    #[serde(rename = "squad_member")]
    SquadMember,
    /// A value not known to this version of the crate.
    #[serde(other)]
    Unrecognized,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
/// The fleet a character is in.
pub struct CharacterFleet {
    /// Character ID of the fleet boss.
    pub fleet_boss_id: i64,
    /// ID of the fleet.
    pub fleet_id: i64,
    /// Role of the character in the fleet.
    pub role: FleetRole,
    /// ID of the squad the character is in (`-1` if none).
    pub squad_id: i64,
    /// ID of the wing the character is in (`-1` if none).
    pub wing_id: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
/// Fleet settings.
pub struct FleetInfo {
    /// Whether free-move is enabled.
    pub is_free_move: bool,
    /// Whether the fleet is registered in the fleet finder.
    pub is_registered: bool,
    /// Whether voice is enabled.
    pub is_voice_enabled: bool,
    /// Message of the day.
    pub motd: String,
}

#[derive(Debug, Serialize, Clone, Default)]
/// The body of a request to update fleet settings.
pub struct FleetUpdate {
    /// Whether free-move is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_free_move: Option<bool>,
    /// Message of the day.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motd: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
/// A member of a fleet.
pub struct FleetMember {
    /// Character ID of the member.
    pub character_id: i64,
    /// Date and time the member joined.
    pub join_time: String,
    /// Role of the member.
    pub role: FleetRole,
    /// Localized role name.
    pub role_name: String,
    /// Type ID of the ship the member flies.
    pub ship_type_id: i64,
    /// Solar system the member is in.
    pub solar_system_id: i64,
    /// Squad of the member (`-1` if none).
    pub squad_id: i64,
    /// Station the member is docked at, if any.
    #[serde(default)]
    pub station_id: Option<i64>,
    /// Whether the member takes fleet warps.
    pub takes_fleet_warp: bool,
    /// Wing of the member (`-1` if none).
    pub wing_id: i64,
}

#[derive(Debug, Serialize, Clone)]
/// The body of a request to invite a character to a fleet.
pub struct FleetInvitation {
    /// Character to invite.
    pub character_id: i64,
    /// Role to give the character.
    pub role: FleetRole,
    /// Squad to place the character in, when the role requires it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub squad_id: Option<i64>,
    /// Wing to place the character in, when the role requires it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wing_id: Option<i64>,
}

#[derive(Debug, Serialize, Clone)]
/// The body of a request to move a fleet member.
pub struct FleetMovement {
    /// New role of the member.
    pub role: FleetRole,
    /// New squad of the member, when the role requires it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub squad_id: Option<i64>,
    /// New wing of the member, when the role requires it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wing_id: Option<i64>,
}

#[derive(Debug, Serialize, Clone)]
/// The body of a request to rename a wing or squad.
pub struct FleetNaming {
    /// New name.
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
/// A squad of a fleet wing.
pub struct FleetSquad {
    /// ID of the squad.
    pub id: i64,
    /// Name of the squad.
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
/// A wing of a fleet.
pub struct FleetWing {
    /// ID of the wing.
    pub id: i64,
    /// Name of the wing.
    pub name: String,
    /// Squads of the wing.
    pub squads: Vec<FleetSquad>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
/// The result of creating a wing.
pub struct CreatedWing {
    /// ID of the new wing.
    pub wing_id: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
/// The result of creating a squad.
pub struct CreatedSquad {
    /// ID of the new squad.
    pub squad_id: i64,
}

impl FleetsGroup<'_> {
    api_get!(
        /// Return the fleet a character is in.
        get_character_fleet,
        "GetCharactersCharacterIdFleet",
        RequestType::Authenticated,
        CharacterFleet,
        (character_id: i64) => "{character_id}"
    );

    api_get!(
        /// Return the settings of a fleet.
        get_fleet,
        "GetFleetsFleetId",
        RequestType::Authenticated,
        FleetInfo,
        (fleet_id: i64) => "{fleet_id}"
    );

    api_put!(
        /// Update the settings of a fleet.
        update_fleet,
        "PutFleetsFleetId",
        RequestType::Authenticated,
        (),
        (fleet_id: i64) => "{fleet_id}",
        settings: &FleetUpdate,
    );

    api_get!(
        /// Return the members of a fleet.
        get_members,
        "GetFleetsFleetIdMembers",
        RequestType::Authenticated,
        Vec<FleetMember>,
        (fleet_id: i64) => "{fleet_id}"
    );

    api_post!(
        /// Invite a character to a fleet.
        invite_member,
        "PostFleetsFleetIdMembers",
        RequestType::Authenticated,
        (),
        (fleet_id: i64) => "{fleet_id}",
        invitation: &FleetInvitation,
    );

    api_delete!(
        /// Kick a member from a fleet.
        kick_member,
        "DeleteFleetsFleetIdMembersMemberId",
        RequestType::Authenticated,
        (),
        (fleet_id: i64) => "{fleet_id}",
        (member_id: i64) => "{member_id}"
    );

    api_put!(
        /// Move a fleet member to another role, squad or wing.
        move_member,
        "PutFleetsFleetIdMembersMemberId",
        RequestType::Authenticated,
        (),
        (fleet_id: i64) => "{fleet_id}",
        (member_id: i64) => "{member_id}",
        movement: &FleetMovement,
    );

    api_delete!(
        /// Delete a fleet squad (only empty squads can be deleted).
        delete_squad,
        "DeleteFleetsFleetIdSquadsSquadId",
        RequestType::Authenticated,
        (),
        (fleet_id: i64) => "{fleet_id}",
        (squad_id: i64) => "{squad_id}"
    );

    api_put!(
        /// Rename a fleet squad.
        rename_squad,
        "PutFleetsFleetIdSquadsSquadId",
        RequestType::Authenticated,
        (),
        (fleet_id: i64) => "{fleet_id}",
        (squad_id: i64) => "{squad_id}",
        naming: &FleetNaming,
    );

    api_get!(
        /// Return the wings and squads of a fleet.
        get_wings,
        "GetFleetsFleetIdWings",
        RequestType::Authenticated,
        Vec<FleetWing>,
        (fleet_id: i64) => "{fleet_id}"
    );

    api_post!(
        /// Create a new wing in a fleet.
        create_wing,
        "PostFleetsFleetIdWings",
        RequestType::Authenticated,
        CreatedWing,
        (fleet_id: i64) => "{fleet_id}";
        NoBody
    );

    api_delete!(
        /// Delete a fleet wing (only empty wings can be deleted).
        delete_wing,
        "DeleteFleetsFleetIdWingsWingId",
        RequestType::Authenticated,
        (),
        (fleet_id: i64) => "{fleet_id}",
        (wing_id: i64) => "{wing_id}"
    );

    api_put!(
        /// Rename a fleet wing.
        rename_wing,
        "PutFleetsFleetIdWingsWingId",
        RequestType::Authenticated,
        (),
        (fleet_id: i64) => "{fleet_id}",
        (wing_id: i64) => "{wing_id}",
        naming: &FleetNaming,
    );

    api_post!(
        /// Create a new squad in a fleet wing.
        create_squad,
        "PostFleetsFleetIdWingsWingIdSquads",
        RequestType::Authenticated,
        CreatedSquad,
        (fleet_id: i64) => "{fleet_id}",
        (wing_id: i64) => "{wing_id}";
        NoBody
    );
}

#[cfg(test)]
mod fleets_tests {
    use super::*;

    #[test]
    fn test_parse_fleet_and_members() {
        let f: CharacterFleet = serde_json::from_str(
            r#"{"fleet_boss_id": 1, "fleet_id": 1234567890123, "role": "squad_member",
                "squad_id": -1, "wing_id": 2000000000000}"#,
        )
        .unwrap();
        assert_eq!(f.fleet_id, 1_234_567_890_123);
        assert_eq!(f.role, FleetRole::SquadMember);
        let m: Vec<FleetMember> = serde_json::from_str(
            r#"[{"character_id": 1, "join_time": "t", "role": "new_role", "role_name": "r",
                 "ship_type_id": 587, "solar_system_id": 30000142, "squad_id": 1, "wing_id": 2,
                 "takes_fleet_warp": true}]"#,
        )
        .unwrap();
        assert_eq!(m[0].role, FleetRole::Unrecognized);
        assert_eq!(m[0].station_id, None);
    }

    #[test]
    fn test_serialize_bodies() {
        let inv = FleetInvitation {
            character_id: 1,
            role: FleetRole::SquadMember,
            squad_id: Some(5),
            wing_id: None,
        };
        assert_eq!(
            serde_json::to_string(&inv).unwrap(),
            r#"{"character_id":1,"role":"squad_member","squad_id":5}"#
        );
        let u = FleetUpdate {
            is_free_move: Some(true),
            motd: None,
        };
        assert_eq!(
            serde_json::to_string(&u).unwrap(),
            r#"{"is_free_move":true}"#
        );
    }

    #[test]
    fn test_parse_wings_and_created() {
        let w: Vec<FleetWing> =
            serde_json::from_str(r#"[{"id": 1, "name": "w", "squads": [{"id": 2, "name": "s"}]}]"#)
                .unwrap();
        assert_eq!(w[0].squads[0].id, 2);
        let c: CreatedWing = serde_json::from_str(r#"{"wing_id": 2000000000000}"#).unwrap();
        assert_eq!(c.wing_id, 2_000_000_000_000);
    }
}
