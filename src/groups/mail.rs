#![allow(unused)]

use crate::prelude::*;

/// Endpoints for Mail
pub struct MailGroup<'a> {
    pub(crate) esi: &'a Esi,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
/// Information about all mail labels.
pub struct MailLabels {
    /// List of individual mail labels.
    #[serde(default)]
    pub labels: Vec<MailLabel>,
    /// Total unread count across all labels.
    pub total_unread_count: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
/// Information about an individual mail label.
pub struct MailLabel {
    /// Color of the label as RGB Hex (`#rrggbb`).
    pub color: String,
    /// ID of the label.
    pub label_id: i64,
    /// Name of the label.
    pub name: String,
    /// Number of unread messages with this label.
    #[serde(default)]
    pub unread_count: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
/// The kind of entity a mail is addressed to.
pub enum RecipientType {
    /// An alliance.
    #[serde(rename = "alliance")]
    Alliance,
    /// A character.
    #[serde(rename = "character")]
    Character,
    /// A corporation.
    #[serde(rename = "corporation")]
    Corporation,
    /// A mailing list.
    #[serde(rename = "mailing_list")]
    MailingList,
    /// A value not known to this version of the crate.
    #[serde(other)]
    Unrecognized,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
/// A recipient of a mail.
pub struct MailRecipient {
    /// ID of the recipient.
    pub recipient_id: i64,
    /// Kind of the recipient.
    pub recipient_type: RecipientType,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
/// A mail header, as returned by the mail list.
pub struct MailHeader {
    /// Character ID of the sender.
    pub from: Option<i64>,
    /// Whether the mail has been read.
    pub is_read: Option<bool>,
    /// Label IDs attached to the mail.
    #[serde(default)]
    pub labels: Vec<i64>,
    /// ID of the mail.
    pub mail_id: Option<i64>,
    /// Recipients of the mail.
    #[serde(default)]
    pub recipients: Vec<MailRecipient>,
    /// Subject of the mail.
    pub subject: Option<String>,
    /// Date and time the mail was sent.
    pub timestamp: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
/// A full mail, as returned by the mail details endpoint.
pub struct Mail {
    /// Body of the mail.
    pub body: Option<String>,
    /// Character ID of the sender.
    pub from: Option<i64>,
    /// Label IDs attached to the mail.
    #[serde(default)]
    pub labels: Vec<i64>,
    /// Whether the mail has been read.
    pub read: Option<bool>,
    /// Recipients of the mail.
    #[serde(default)]
    pub recipients: Vec<MailRecipient>,
    /// Subject of the mail.
    pub subject: Option<String>,
    /// Date and time the mail was sent.
    pub timestamp: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
/// The body of a request to send a new mail.
pub struct NewMail {
    /// Body of the mail.
    pub body: String,
    /// Recipients of the mail.
    pub recipients: Vec<MailRecipient>,
    /// Subject of the mail.
    pub subject: String,
    /// Maximum CSPA charge (in ISK) the sender accepts to pay.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_cost: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
/// The colors allowed for a mail label.
pub enum MailLabelColor {
    /// `#0000fe`
    #[serde(rename = "#0000fe")]
    Blue,
    /// `#006634`
    #[serde(rename = "#006634")]
    DarkGreen,
    /// `#0099ff`
    #[serde(rename = "#0099ff")]
    LightBlue,
    /// `#00ff33`
    #[serde(rename = "#00ff33")]
    BrightGreen,
    /// `#01ffff`
    #[serde(rename = "#01ffff")]
    Cyan,
    /// `#349800`
    #[serde(rename = "#349800")]
    Green,
    /// `#660066`
    #[serde(rename = "#660066")]
    Purple,
    /// `#666666`
    #[serde(rename = "#666666")]
    DarkGrey,
    /// `#999999`
    #[serde(rename = "#999999")]
    Grey,
    /// `#99ffff`
    #[serde(rename = "#99ffff")]
    PaleCyan,
    /// `#9a0000`
    #[serde(rename = "#9a0000")]
    DarkRed,
    /// `#ccff9a`
    #[serde(rename = "#ccff9a")]
    PaleGreen,
    /// `#e6e6e6`
    #[serde(rename = "#e6e6e6")]
    LightGrey,
    /// `#fe0000`
    #[serde(rename = "#fe0000")]
    Red,
    /// `#ff6600`
    #[serde(rename = "#ff6600")]
    Orange,
    /// `#ffff01`
    #[serde(rename = "#ffff01")]
    Yellow,
    /// `#ffffcd`
    #[serde(rename = "#ffffcd")]
    PaleYellow,
    /// `#ffffff`
    #[serde(rename = "#ffffff")]
    White,
    /// A value not known to this version of the crate.
    #[serde(other)]
    Unrecognized,
}

#[derive(Debug, Serialize, Clone)]
/// The body of a request to create a mail label.
pub struct NewMailLabel {
    /// Name of the label.
    pub name: String,
    /// Color of the label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<MailLabelColor>,
}

#[derive(Debug, Serialize, Clone, Default)]
/// The body of a request to update the metadata of a mail.
pub struct MailUpdate {
    /// Label IDs to set on the mail.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<i64>>,
    /// Whether the mail is read.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
/// A mailing list the character is subscribed to.
pub struct MailingList {
    /// ID of the mailing list.
    pub mailing_list_id: i64,
    /// Name of the mailing list.
    pub name: String,
}

impl MailGroup<'_> {
    api_get!(
        /// Return the 50 most recent mail headers of a character.
        ///
        /// Use `labels` to filter by label and `last_mail_id` to page back
        /// from the oldest mail already seen.
        get_character_mail,
        "GetCharactersCharacterIdMail",
        RequestType::Authenticated,
        Vec<MailHeader>,
        (character_id: i64) => "{character_id}";
        OptionalMany(labels: &[i64]) => "labels",
        Optional(last_mail_id: i64) => "last_mail_id"
    );

    api_post!(
        /// Send a new mail. Returns the ID of the created mail.
        send_character_mail,
        "PostCharactersCharacterIdMail",
        RequestType::Authenticated,
        i64,
        (character_id: i64) => "{character_id}",
        mail: &NewMail,
    );

    api_get!(
        /// Return a list of the users mail labels, unread counts for each
        /// label and a total unread count.
        get_character_mail_labels,
        "GetCharactersCharacterIdMailLabels",
        RequestType::Authenticated,
        MailLabels,
        (character_id: i64) => "{character_id}"
    );

    api_post!(
        /// Create a mail label. Returns the ID of the created label.
        create_character_mail_label,
        "PostCharactersCharacterIdMailLabels",
        RequestType::Authenticated,
        i64,
        (character_id: i64) => "{character_id}",
        label: &NewMailLabel,
    );

    api_delete!(
        /// Delete a mail label.
        delete_character_mail_label,
        "DeleteCharactersCharacterIdMailLabelsLabelId",
        RequestType::Authenticated,
        (),
        (character_id: i64) => "{character_id}",
        (label_id: i64) => "{label_id}"
    );

    api_get!(
        /// Return all mailing lists the character is subscribed to.
        get_character_mailing_lists,
        "GetCharactersCharacterIdMailLists",
        RequestType::Authenticated,
        Vec<MailingList>,
        (character_id: i64) => "{character_id}"
    );

    api_delete!(
        /// Delete a mail.
        delete_character_mail,
        "DeleteCharactersCharacterIdMailMailId",
        RequestType::Authenticated,
        (),
        (character_id: i64) => "{character_id}",
        (mail_id: i64) => "{mail_id}"
    );

    api_get!(
        /// Return the contents of a single mail.
        get_character_mail_by_id,
        "GetCharactersCharacterIdMailMailId",
        RequestType::Authenticated,
        Mail,
        (character_id: i64) => "{character_id}",
        (mail_id: i64) => "{mail_id}"
    );

    api_put!(
        /// Update the labels and read state of a mail.
        update_character_mail,
        "PutCharactersCharacterIdMailMailId",
        RequestType::Authenticated,
        (),
        (character_id: i64) => "{character_id}",
        (mail_id: i64) => "{mail_id}",
        update: &MailUpdate,
    );
}

#[cfg(test)]
mod mail_tests {
    use super::*;

    #[test]
    fn test_parse_mail_header_and_unknown_recipient() {
        let json = r#"[{"from": 2112625428, "is_read": true, "labels": [3, 9000000000],
            "mail_id": 7000000000, "subject": "Hi", "timestamp": "2026-10-01T00:00:00Z",
            "recipients": [{"recipient_id": 1, "recipient_type": "mailing_list"},
                           {"recipient_id": 2, "recipient_type": "something_new"}]}]"#;
        let headers: Vec<MailHeader> = serde_json::from_str(json).unwrap();
        assert_eq!(headers[0].mail_id, Some(7_000_000_000));
        assert_eq!(
            headers[0].recipients[0].recipient_type,
            RecipientType::MailingList
        );
        assert_eq!(
            headers[0].recipients[1].recipient_type,
            RecipientType::Unrecognized
        );
    }

    #[test]
    fn test_serialize_new_mail_and_label() {
        let mail = NewMail {
            body: "b".into(),
            recipients: vec![MailRecipient {
                recipient_id: 1,
                recipient_type: RecipientType::Character,
            }],
            subject: "s".into(),
            approved_cost: None,
        };
        let v = serde_json::to_value(&mail).unwrap();
        assert_eq!(v["recipients"][0]["recipient_type"], "character");
        assert!(v.get("approved_cost").is_none());
        let label = NewMailLabel {
            name: "x".into(),
            color: Some(MailLabelColor::Red),
        };
        assert_eq!(serde_json::to_value(&label).unwrap()["color"], "#fe0000");
        let upd = MailUpdate {
            labels: None,
            read: Some(true),
        };
        assert_eq!(serde_json::to_string(&upd).unwrap(), r#"{"read":true}"#);
    }

    #[test]
    fn test_parse_mail_and_lists() {
        let m: Mail = serde_json::from_str(r#"{"body": "x", "from": 1, "read": false}"#).unwrap();
        assert_eq!(m.read, Some(false));
        let l: Vec<MailingList> =
            serde_json::from_str(r#"[{"mailing_list_id": 5000000000, "name": "n"}]"#).unwrap();
        assert_eq!(l[0].mailing_list_id, 5_000_000_000);
    }
}
