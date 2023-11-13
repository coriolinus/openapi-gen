//! This module contains serialization helpers for [`time::Date`].
//!
//! There are two modes of use:
//!
//! - For raw dates, use serde's `with` attribute:
//!
//!     ```rust,ignore
//!     #[derive(serde::Serialize)]
//!     struct Foo {
//!         #[serde(with = "date_as_string")]
//!         date: Date,
//!     }
//!     ```
//!
//! - For composite date types, use `serde_with::serde_as`:
//!
//!     ```rust,ignore
//!     #[serde_as]
//!     #[derive(serde::Serialize)]
//!     struct Foo {
//!         #[serde_as(as = "Option<date_as_string::Ymd>")]
//!         date: Option<Date>,
//!     }
//!     ```

use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use serde_with::{DeserializeAs, SerializeAs};
use time::{format_description::FormatItem, macros::format_description, Date};

pub const YMD_FORMAT: &[FormatItem<'_>] = format_description!(version = 2, "[year]-[month]-[day]");

fn serialize<S>(date: &Date, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = date
        .format(&YMD_FORMAT)
        .expect("all dates can format into YMD");
    s.serialize(serializer)
}

fn deserialize<'de, D>(deserializer: D) -> Result<Date, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Date::parse(&s, &YMD_FORMAT).map_err(D::Error::custom)
}

pub struct Ymd;

impl SerializeAs<Date> for Ymd {
    fn serialize_as<S>(date: &Date, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize(date, serializer)
    }
}

impl<'de> DeserializeAs<'de, Date> for Ymd {
    fn deserialize_as<D>(deserializer: D) -> Result<Date, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize(deserializer)
    }
}
