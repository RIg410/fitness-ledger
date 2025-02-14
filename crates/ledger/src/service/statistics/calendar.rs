use crate::service::calendar::Calendar;
use chrono::NaiveDate;
use eyre::Error;
use model::{session::Session, statistics::month::MonthStatistics};
use std::collections::HashMap;

use super::month_id;

pub async fn load_calendar(
    calendar: &Calendar,
    session: &mut Session,
) -> Result<HashMap<NaiveDate, MonthStatistics>, Error> {
    let mut days = calendar.find_range(session, None, None).await?;
    let mut monthes = HashMap::new();

    while let Some(day) = days.next(session).await {
        let day = day?;

        let month = month_id(day.day_date());
        let month = monthes
            .entry(month)
            .or_insert_with(|| MonthStatistics::default());

        for training in &day.training {
            if !training.is_processed {
                continue;
            }
            month.training.extend(training);
        }
    }

    Ok(monthes)
}
