use crate::prelude::*;

/// Endpoints for Access List
pub struct AccessListGroup<'a> {
    pub(crate) esi: &'a Esi,
}

/// A reference to an access list.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct AccessListSummary {
    /// ID of the access list.
    pub id: i64,
}

#[derive(Debug, Deserialize)]
struct AccessLists {
    access_lists: Vec<AccessListSummary>,
}

/// The access level granted to a member of an access list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum AccessLevel {
    /// No level specified.
    Unspecified,
    /// Allowed.
    Allowed,
    /// Blocked.
    Blocked,
    /// Can manage the list.
    Manager,
    /// Full administrator of the list.
    Admin,
    /// A level this version of the crate does not know about.
    #[serde(other)]
    Unrecognized,
}

/// An alliance in an access list.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct AllianceAccess {
    pub access: AccessLevel,
    pub alliance_id: i64,
}

/// A character in an access list.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct CharacterAccess {
    pub access: AccessLevel,
    pub character_id: i64,
}

/// A corporation in an access list.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct CorporationAccess {
    pub access: AccessLevel,
    pub corporation_id: i64,
}

/// Who belongs to an access list.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct AccessListMembership {
    pub alliances: Vec<AllianceAccess>,
    pub allow_everyone: bool,
    pub characters: Vec<CharacterAccess>,
    pub corporations: Vec<CorporationAccess>,
}

/// An access list with its members.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct AccessList {
    pub description: String,
    pub id: i64,
    pub membership: AccessListMembership,
    pub name: String,
}

impl AccessListGroup<'_> {
    /// List the access lists of a character.
    pub async fn get_character_access_lists(
        &self,
        character_id: i64,
    ) -> EsiResult<Vec<AccessListSummary>> {
        let path = self
            .esi
            .get_endpoint_for_op_id("GetCharactersAccessListsListing")?
            .replace("{character_id}", &character_id.to_string());
        let wrapper: AccessLists = self
            .esi
            .query("GET", RequestType::Authenticated, &path, None, None)
            .await?;
        Ok(wrapper.access_lists)
    }

    api_get!(
        /// Get an access list of a character with its members.
        get_character_access_list,
        "GetCharactersAccessListsDetail",
        RequestType::Authenticated,
        AccessList,
        (character_id: i64) => "{character_id}",
        (access_list_id: i64) => "{access_list_id}"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_access_list() {
        let list: AccessList = serde_json::from_str(
            r#"{"description":"d","id":1,"name":"n","membership":{"allow_everyone":false,
            "alliances":[],"corporations":[],
            "characters":[{"access":"Admin","character_id":2},{"access":"Odd","character_id":3}]}}"#,
        )
        .unwrap();
        assert_eq!(list.membership.characters[0].access, AccessLevel::Admin);
        assert_eq!(
            list.membership.characters[1].access,
            AccessLevel::Unrecognized
        );
    }
}
