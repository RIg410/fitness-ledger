use chrono::Datelike;
use chrono::NaiveDate;
use serde::Deserializer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Date {
    year: i32,
    month: u32,
    day: u32,
}

impl From<NaiveDate> for Date {
    fn from(date: NaiveDate) -> Self {
        Self {
            year: date.year(),
            month: date.month(),
            day: date.day(),
        }
    }
}

impl From<Date> for NaiveDate {
    fn from(date: Date) -> Self {
        NaiveDate::from_ymd_opt(date.year, date.month, date.day).unwrap()
    }
}

pub fn opt_naive_date_serialize<S>(
    date: &Option<NaiveDate>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    date.map(Date::from).serialize(serializer)
}

pub fn opt_naive_date_deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    let date = Option::<Date>::deserialize(deserializer)?;
    Ok(date.map(NaiveDate::from))
}

pub fn naive_date_serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    Date::from(*date).serialize(serializer)
}

pub fn naive_date_deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let date = Date::deserialize(deserializer)?;
    Ok(NaiveDate::from(date))
}

