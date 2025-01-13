use chrono::{DateTime, Datelike as _, Days, Local, Months, Timelike as _};
use eyre::Error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Range {
    Day(DateTime<Local>),
    Week(DateTime<Local>),
    Month(DateTime<Local>),
}

impl Default for Range {
    fn default() -> Self {
        Range::Day(Local::now())
    }
}

impl Range {
    pub fn group_by(&self) -> GroupBy {
        match self {
            Range::Day(_) => GroupBy::Day,
            Range::Week(_) => GroupBy::Week,
            Range::Month(_) => GroupBy::Month,
        }
    }

    pub fn next(&self) -> Result<Self, Error> {
        fn inner(range: Range) -> Option<Range> {
            let base_date = range.base_date();
            let next = base_date.checked_add_months(Months::new(1))?;
            match range {
                Range::Day(_) => Some(Range::Day(next)),
                Range::Week(_) => Some(Range::Week(next)),
                Range::Month(_) => Some(Range::Month(next)),
            }
        }
        inner(*self).ok_or_else(|| eyre::eyre!("Failed to calculate next range for {:?}", self))
    }

    pub fn prev(&self) -> Result<Self, Error> {
        fn inner(range: Range) -> Option<Range> {
            let base_date = range.base_date();
            let prev = base_date.checked_sub_months(Months::new(1))?;
            match range {
                Range::Day(_) => Some(Range::Day(prev)),
                Range::Week(_) => Some(Range::Week(prev)),
                Range::Month(_) => Some(Range::Month(prev)),
            }
        }
        inner(*self).ok_or_else(|| eyre::eyre!("Failed to calculate prev range for {:?}", self))
    }

    pub fn base_date(&self) -> DateTime<Local> {
        match self {
            Range::Day(date) => *date,
            Range::Week(date) => *date,
            Range::Month(date) => *date,
        }
    }

    pub fn range(&self) -> Result<(DateTime<Local>, DateTime<Local>), Error> {
        fn inner(range: Range) -> Option<(DateTime<Local>, DateTime<Local>)> {
            let base_date = range.base_date();
            let to = base_date
                .with_day0(0)?
                .checked_add_months(Months::new(1))?
                .checked_sub_days(Days::new(1))?
                .with_hour(23)?
                .with_minute(59)?
                .with_second(59)?;

            let from = match range {
                Range::Day(_) => base_date
                    .with_day0(0)?
                    .with_hour(0)?
                    .with_minute(0)?
                    .with_second(0)?,
                Range::Week(_) => base_date
                    .with_day0(0)?
                    .checked_sub_months(Months::new(3))?
                    .with_hour(0)?
                    .with_minute(0)?
                    .with_second(0)?,
                Range::Month(_) => base_date
                    .with_day0(0)?
                    .checked_sub_months(Months::new(12))?
                    .with_hour(0)?
                    .with_minute(0)?
                    .with_second(0)?,
            };
            Some((from, to))
        }

        inner(*self).ok_or_else(|| eyre::eyre!("Failed to calculate range for {:?}", self))
    }

    pub fn is_day(&self) -> bool {
        matches!(self, Range::Day(_))
    }

    pub fn is_week(&self) -> bool {
        matches!(self, Range::Week(_))
    }

    pub fn is_month(&self) -> bool {
        matches!(self, Range::Month(_))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
pub enum GroupBy {
    Day,
    Week,
    Month,
}

#[cfg(test)]
mod test {
    #![allow(deprecated)]

    use chrono::TimeZone as _;

    #[test]
    pub fn test_range_day() {
        use super::*;
        let now = Local.ymd(2021, 1, 5).and_hms(0, 0, 0);
        let range = Range::Day(now);
        assert_eq!(range.base_date(), now);
        let (from, to) = range.range().unwrap();
        assert_eq!(from, Local.ymd(2021, 1, 1).and_hms(0, 0, 0));
        assert_eq!(to, Local.ymd(2021, 1, 31).and_hms(23, 59, 59));

        let range = range.next().unwrap();
        assert_eq!(range.base_date(), Local.ymd(2021, 2, 5).and_hms(0, 0, 0));
        let (from, to) = range.range().unwrap();
        assert_eq!(from, Local.ymd(2021, 2, 1).and_hms(0, 0, 0));
        assert_eq!(to, Local.ymd(2021, 2, 28).and_hms(23, 59, 59));

        let range = range.next().unwrap();
        assert_eq!(range.base_date(), Local.ymd(2021, 3, 5).and_hms(0, 0, 0));
        let (from, to) = range.range().unwrap();
        assert_eq!(from, Local.ymd(2021, 3, 1).and_hms(0, 0, 0));
        assert_eq!(to, Local.ymd(2021, 3, 31).and_hms(23, 59, 59));

        let range = range.prev().unwrap();
        assert_eq!(range.base_date(), Local.ymd(2021, 2, 5).and_hms(0, 0, 0));
        let (from, to) = range.range().unwrap();
        assert_eq!(from, Local.ymd(2021, 2, 1).and_hms(0, 0, 0));
        assert_eq!(to, Local.ymd(2021, 2, 28).and_hms(23, 59, 59));
    }

    #[test]
    pub fn test_range_week() {
        use super::*;
        let now = Local.ymd(2021, 1, 5).and_hms(0, 0, 0);
        let range = Range::Week(now);
        assert_eq!(range.base_date(), now);
        let (from, to) = range.range().unwrap();
        assert_eq!(from, Local.ymd(2020, 10, 1).and_hms(0, 0, 0));
        assert_eq!(to, Local.ymd(2021, 1, 31).and_hms(23, 59, 59));
    }

    #[test]
    pub fn test_range_month() {
        use super::*;
        let now = Local.ymd(2021, 1, 5).and_hms(0, 0, 0);
        let range = Range::Month(now);
        assert_eq!(range.base_date(), now);
        let (from, to) = range.range().unwrap();
        assert_eq!(from, Local.ymd(2020, 1, 1).and_hms(0, 0, 0));
        assert_eq!(to, Local.ymd(2021, 1, 31).and_hms(23, 59, 59));
    }
}
