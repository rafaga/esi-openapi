//! Single-pass reading of cursor-paginated responses.
//!
//! A cursor page holds an array of records under a key that depends on the
//! endpoint (`projects`, `listings`, ...) next to a `cursor` object. Reading it
//! through a generic [`serde_json::Value`] would build the whole tree and then
//! convert it again; this reads the records straight into `T`.

use crate::prelude::EsiResult;
use serde::de::{DeserializeOwned, DeserializeSeed, IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::{fmt, marker::PhantomData};

/// One page of a cursor-paginated endpoint.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct CursorPage<T> {
    /// The records of the page.
    pub(crate) records: Vec<T>,
    /// The `cursor.after` value, if the response has one.
    pub(crate) next: Option<String>,
}

#[derive(Deserialize)]
struct CursorAfter {
    #[serde(default)]
    after: Option<String>,
}

impl<T: DeserializeOwned> CursorPage<T> {
    fn empty() -> Self {
        CursorPage {
            records: Vec::new(),
            next: None,
        }
    }

    /// Read a response body, taking the records from `items_key`. A body that is
    /// empty or `null` is a page without records; a missing `items_key` too.
    pub(crate) fn parse(text: &str, items_key: &str) -> EsiResult<Self> {
        if text.trim().is_empty() {
            return Ok(Self::empty());
        }
        let mut deserializer = serde_json::Deserializer::from_str(text);
        let seed = CursorPageSeed {
            items_key,
            marker: PhantomData,
        };
        let page = seed.deserialize(&mut deserializer)?;
        deserializer.end()?;
        Ok(page)
    }
}

/// Carries the name of the records key into the deserializer.
struct CursorPageSeed<'a, T> {
    items_key: &'a str,
    marker: PhantomData<T>,
}

impl<'de, T: DeserializeOwned> DeserializeSeed<'de> for CursorPageSeed<'_, T> {
    type Value = CursorPage<T>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de, T: DeserializeOwned> Visitor<'de> for CursorPageSeed<'_, T> {
    type Value = CursorPage<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a cursor-paginated response object")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(CursorPage::empty())
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut page = CursorPage::empty();
        while let Some(key) = map.next_key::<String>()? {
            if key == self.items_key {
                page.records = map.next_value::<Option<Vec<T>>>()?.unwrap_or_default();
            } else if key == "cursor" {
                page.next = map
                    .next_value::<Option<CursorAfter>>()?
                    .and_then(|cursor| cursor.after);
            } else {
                map.next_value::<IgnoredAny>()?;
            }
        }
        Ok(page)
    }
}

#[cfg(test)]
mod tests {
    use super::CursorPage;

    #[derive(Debug, serde::Deserialize, PartialEq)]
    struct Record {
        id: i64,
    }

    #[test]
    fn test_reads_records_and_next_cursor() {
        let text = r#"{"cursor": {"after": "abc", "before": "xyz"}, "projects": [{"id": 1}, {"id": 2}], "other": [1, 2, 3]}"#;
        let page: CursorPage<Record> = CursorPage::parse(text, "projects").unwrap();
        assert_eq!(page.records, [Record { id: 1 }, Record { id: 2 }]);
        assert_eq!(page.next.as_deref(), Some("abc"));
    }

    #[test]
    fn test_missing_parts_are_empty() {
        let page: CursorPage<Record> =
            CursorPage::parse(r#"{"projects": []}"#, "projects").unwrap();
        assert!(page.records.is_empty() && page.next.is_none());
        for text in ["{}", "", "null", r#"{"cursor": null, "projects": null}"#] {
            let page: CursorPage<Record> = CursorPage::parse(text, "projects").unwrap();
            assert!(page.records.is_empty() && page.next.is_none(), "{text}");
        }
        let page: CursorPage<Record> = CursorPage::parse(
            r#"{"cursor": {"before": "x"}, "projects": [{"id": 7}]}"#,
            "projects",
        )
        .unwrap();
        assert_eq!(page.records.len(), 1);
        assert!(page.next.is_none());
    }

    #[test]
    fn test_invalid_json_and_record_errors() {
        assert!(CursorPage::<Record>::parse("{", "projects").is_err());
        assert!(CursorPage::<Record>::parse(r#"{"projects": [{"id": "x"}]}"#, "projects").is_err());
    }
}
