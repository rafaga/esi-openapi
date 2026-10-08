#![allow(unused)]

use crate::prelude::*;

/// Endpoints for Calendar
pub struct CalendarGroup<'a> {
    pub(crate) esi: &'a Esi,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
/// The response of a character to a calendar event.
pub enum EventResponse {
    /// The character declined.
    #[serde(rename = "declined")]
    Declined,
    /// The character has not responded yet.
    #[serde(rename = "not_responded")]
    NotResponded,
    /// The character accepted.
    #[serde(rename = "accepted")]
    Accepted,
    /// The character is tentative.
    #[serde(rename = "tentative")]
    Tentative,
    /// A value not known to this version of the crate.
    #[serde(other)]
    Unrecognized,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
/// The response a character can send to an event. Unlike [`EventResponse`],
/// "not responded" cannot be sent.
pub enum EventReply {
    /// Accept the event.
    #[serde(rename = "accepted")]
    Accepted,
    /// Decline the event.
    #[serde(rename = "declined")]
    Declined,
    /// Mark the event as tentative.
    #[serde(rename = "tentative")]
    Tentative,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
/// The kind of entity that owns a calendar event.
pub enum EventOwnerType {
    /// The game server.
    #[serde(rename = "eve_server")]
    EveServer,
    /// A corporation.
    #[serde(rename = "corporation")]
    Corporation,
    /// A faction.
    #[serde(rename = "faction")]
    Faction,
    /// A character.
    #[serde(rename = "character")]
    Character,
    /// An alliance.
    #[serde(rename = "alliance")]
    Alliance,
    /// A value not known to this version of the crate.
    #[serde(other)]
    Unrecognized,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
/// A calendar event summary, as returned by the event list.
pub struct CalendarEventSummary {
    /// Date and time of the event.
    pub event_date: Option<String>,
    /// ID of the event.
    pub event_id: Option<i64>,
    /// Response of the character to the event.
    pub event_response: Option<EventResponse>,
    /// Importance of the event.
    pub importance: Option<i64>,
    /// Title of the event.
    pub title: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
/// The details of a calendar event.
pub struct CalendarEvent {
    /// Date and time of the event.
    pub date: String,
    /// Duration of the event in minutes.
    pub duration: i64,
    /// ID of the event.
    pub event_id: i64,
    /// Importance of the event.
    pub importance: i64,
    /// ID of the owner of the event.
    pub owner_id: i64,
    /// Name of the owner of the event.
    pub owner_name: String,
    /// Kind of the owner of the event.
    pub owner_type: EventOwnerType,
    /// Response of the character to the event.
    pub response: String,
    /// Description of the event.
    pub text: String,
    /// Title of the event.
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
/// An attendee of a calendar event.
pub struct EventAttendee {
    /// ID of the attendee.
    pub character_id: Option<i64>,
    /// Response of the attendee.
    pub event_response: Option<EventResponse>,
}

#[derive(Debug, Serialize, Clone, Copy)]
/// The body of a request to respond to an event.
pub struct EventResponseRequest {
    /// The response to send.
    pub response: EventReply,
}

impl CalendarGroup<'_> {
    api_get!(
        /// Return up to 50 events from a character's calendar, starting at
        /// `from_event` when given.
        get_character_events,
        "GetCharactersCharacterIdCalendar",
        RequestType::Authenticated,
        Vec<CalendarEventSummary>,
        (character_id: i64) => "{character_id}";
        Optional(from_event: i64) => "from_event"
    );

    api_get!(
        /// Return the details of a calendar event.
        get_character_event,
        "GetCharactersCharacterIdCalendarEventId",
        RequestType::Authenticated,
        CalendarEvent,
        (character_id: i64) => "{character_id}",
        (event_id: i64) => "{event_id}"
    );

    api_put!(
        /// Respond to a calendar event.
        respond_to_character_event,
        "PutCharactersCharacterIdCalendarEventId",
        RequestType::Authenticated,
        (),
        (character_id: i64) => "{character_id}",
        (event_id: i64) => "{event_id}",
        response: &EventResponseRequest,
    );

    api_get!(
        /// Return the attendees of a calendar event.
        get_character_event_attendees,
        "GetCharactersCharacterIdCalendarEventIdAttendees",
        RequestType::Authenticated,
        Vec<EventAttendee>,
        (character_id: i64) => "{character_id}",
        (event_id: i64) => "{event_id}"
    );
}

#[cfg(test)]
mod calendar_tests {
    use super::*;

    #[test]
    fn test_parse_events() {
        let json = r#"[{"event_date": "2026-10-01T00:00:00Z", "event_id": 3000000000,
            "event_response": "not_responded", "importance": 1, "title": "t"},
            {"event_id": 1, "event_response": "maybe"}]"#;
        let ev: Vec<CalendarEventSummary> = serde_json::from_str(json).unwrap();
        assert_eq!(ev[0].event_id, Some(3_000_000_000));
        assert_eq!(ev[0].event_response, Some(EventResponse::NotResponded));
        assert_eq!(ev[1].event_response, Some(EventResponse::Unrecognized));
    }

    #[test]
    fn test_parse_event_and_serialize_reply() {
        let json = r#"{"date": "d", "duration": 60, "event_id": 1, "importance": 0, "owner_id": 5,
            "owner_name": "n", "owner_type": "eve_server", "response": "accepted", "text": "x", "title": "t"}"#;
        let e: CalendarEvent = serde_json::from_str(json).unwrap();
        assert_eq!(e.owner_type, EventOwnerType::EveServer);
        let body = EventResponseRequest {
            response: EventReply::Tentative,
        };
        assert_eq!(
            serde_json::to_string(&body).unwrap(),
            r#"{"response":"tentative"}"#
        );
    }
}
