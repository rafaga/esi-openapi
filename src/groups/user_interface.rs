use crate::prelude::*;

/// Endpoints for UserInterface
pub struct UserInterfaceGroup<'a> {
    pub(crate) esi: &'a Esi,
}

impl UserInterfaceGroup<'_> {
    /// Open the market details window.
    pub async fn open_market_details_window(&self, type_id: i64) -> EsiResult<()> {
        // not using the macro since it doesn't like no body;
        // `type_id` is a query parameter, not part of the path
        let path = self
            .esi
            .get_endpoint_for_op_id("PostUiOpenwindowMarketdetails")?;
        let type_id = type_id.to_string();
        self.esi
            .query(
                "POST",
                RequestType::Authenticated,
                &path,
                Some(&[("type_id", type_id.as_str())]),
                None,
            )
            .await
    }
}

/// A new mail to pre-fill in the in-game mail window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NewMailWindow {
    /// Body of the mail.
    pub body: String,
    /// Character IDs of the recipients.
    pub recipients: Vec<i64>,
    /// Subject of the mail.
    pub subject: String,
    /// Corporation or alliance ID to send the mail to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_corp_or_alliance_id: Option<i64>,
    /// Mailing list ID to send the mail to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_mailing_list_id: Option<i64>,
}

impl UserInterfaceGroup<'_> {
    /// Set a solar system, station or structure as an autopilot waypoint in the client.
    pub async fn set_autopilot_waypoint(
        &self,
        destination_id: i64,
        add_to_beginning: bool,
        clear_other_waypoints: bool,
    ) -> EsiResult<()> {
        let path = self.esi.get_endpoint_for_op_id("PostUiAutopilotWaypoint")?;
        let destination_id = destination_id.to_string();
        let add_to_beginning = add_to_beginning.to_string();
        let clear_other_waypoints = clear_other_waypoints.to_string();
        self.esi
            .query(
                "POST",
                RequestType::Authenticated,
                &path,
                Some(&[
                    ("add_to_beginning", add_to_beginning.as_str()),
                    ("clear_other_waypoints", clear_other_waypoints.as_str()),
                    ("destination_id", destination_id.as_str()),
                ]),
                None,
            )
            .await
    }

    /// Open the contract window in the client.
    pub async fn open_contract_window(&self, contract_id: i64) -> EsiResult<()> {
        let path = self
            .esi
            .get_endpoint_for_op_id("PostUiOpenwindowContract")?;
        let contract_id = contract_id.to_string();
        self.esi
            .query(
                "POST",
                RequestType::Authenticated,
                &path,
                Some(&[("contract_id", contract_id.as_str())]),
                None,
            )
            .await
    }

    /// Open the information window of a character, corporation or alliance in the client.
    pub async fn open_information_window(&self, target_id: i64) -> EsiResult<()> {
        let path = self
            .esi
            .get_endpoint_for_op_id("PostUiOpenwindowInformation")?;
        let target_id = target_id.to_string();
        self.esi
            .query(
                "POST",
                RequestType::Authenticated,
                &path,
                Some(&[("target_id", target_id.as_str())]),
                None,
            )
            .await
    }

    /// Open the new mail window in the client with the given content.
    pub async fn open_new_mail_window(&self, mail: &NewMailWindow) -> EsiResult<()> {
        let path = self.esi.get_endpoint_for_op_id("PostUiOpenwindowNewmail")?;
        let body = serde_json::to_string(mail)?;
        self.esi
            .query("POST", RequestType::Authenticated, &path, None, Some(&body))
            .await
    }
}

#[cfg(test)]
mod new_mail_tests {
    use super::*;

    #[test]
    fn test_serialize_new_mail_window() {
        let mail = NewMailWindow {
            body: "b".into(),
            recipients: vec![1, 2],
            subject: "s".into(),
            to_corp_or_alliance_id: None,
            to_mailing_list_id: Some(5),
        };
        let json = serde_json::to_string(&mail).unwrap();
        assert!(json.contains("\"to_mailing_list_id\":5"));
        assert!(!json.contains("to_corp_or_alliance_id"));
    }
}
