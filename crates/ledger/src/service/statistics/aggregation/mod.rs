mod loader;

use chrono::DateTime;
use chrono::Datelike;
use chrono::Duration;
use chrono::Local;
use chrono::Months;
use chrono::NaiveDate;
use chrono::TimeZone;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RequiredAggregations {
    pub aggregations: Vec<Aggregation>,
    pub months: Vec<NaiveDate>,
}

impl RequiredAggregations {
    pub fn date_range(&self) -> Vec<(DateTime<Local>, DateTime<Local>)> {
        let mut dates = self.months.iter().map(month_range).collect::<Vec<_>>();
        dates.sort_by_key(|(date, _)| *date);
        dates
    }
}

#[derive(Deserialize, Debug)]
pub enum Aggregation {
    #[serde(rename = "trainings_by_program")]
    TrainingsByProgram,
    #[serde(rename = "trainings_by_instructor")]
    TrainingsByInstructor,
    #[serde(rename = "trainings_by_room")]
    TrainingsByRoom,
    #[serde(rename = "trainings_by_type")]
    TrainingsByType,
    #[serde(rename = "trainings_by_weekday")]
    TrainingsByWeekday,
    #[serde(rename = "trainings_by_time")]
    TrainingsByTime,
    #[serde(rename = "request_aggregation")]
    RequestAggregation,
    #[serde(rename = "subscription_aggregation")]
    SubscriptionAggregation,
    #[serde(rename = "financial_statistics")]
    FinancialStatistics,
    #[serde(rename = "salary_statistics")]
    SalaryStatistics,
    #[serde(rename = "marketing_financial_statistics")]
    MarketingFinancialStatistics,
    #[serde(rename = "marketing_statistics")]
    MarketingStatistics,
}

pub fn month_range(month: &NaiveDate) -> (DateTime<Local>, DateTime<Local>) {
    let start = Local
        .with_ymd_and_hms(month.year(), month.month(), 1, 0, 0, 0)
        .unwrap();
    let end = start.checked_add_months(Months::new(1)).unwrap() - Duration::seconds(1);
    (start, end)
}

#[cfg(test)]
mod tests {
    #![allow(deprecated)]
    use super::*;
    use crate::statistics::month_id;

    #[test]
    fn test_month_id() {
        let date = Local.ymd(2023, 10, 15).and_hms(0, 0, 0);
        let expected = NaiveDate::from_ymd(2023, 10, 1);
        assert_eq!(month_id(date), expected);
    }

    #[test]
    fn test_month_start_and() {
        let month = NaiveDate::from_ymd(2023, 10, 1);
        let (start, end) = month_range(&month);

        let expected_start = Local.ymd(2023, 10, 1).and_hms(0, 0, 0);
        let expected_end = Local.ymd(2023, 10, 31).and_hms(23, 59, 59);

        assert_eq!(start, expected_start);
        assert_eq!(end, expected_end);
    }
}
