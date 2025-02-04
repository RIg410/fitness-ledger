pub mod cache;
pub mod calendar;
pub mod history;
pub mod prompt;
pub mod treasury;

use std::{sync::Arc, time::Instant};

use super::{
    calendar::Calendar, history::History, requests::Requests, treasury::Treasury, users::Users,
};
use ai::{Ai, AiContext, AiModel};
use cache::{CacheEntry, StatCache};
use calendar::load_calendar;
use chrono::{DateTime, Datelike as _, Duration, Local, Months, NaiveDate, TimeZone as _};
use eyre::Error;
use history::load_requests_and_history;
use log::info;
use model::session::Session;
use prompt::make_prompt;
use treasury::load_treasury;

pub struct Statistics {
    cache: cache::StatCache,
    calendar: Calendar,
    history: History,
    users: Users,
    requests: Requests,
    treasury: Treasury,
    ai: Ai,
}

impl Statistics {
    pub(crate) fn new(
        calendar: Calendar,
        history: History,
        users: Users,
        requests: Requests,
        ai: Ai,
        treasury: Treasury,
    ) -> Self {
        Self {
            calendar,
            history,
            users,
            requests,
            ai,
            cache: StatCache::new(),
            treasury,
        }
    }

    async fn reload_statistics(&self, session: &mut Session) -> Result<Arc<CacheEntry>, Error> {
        let start = Instant::now();
        info!("Reloading statistics...");
        let mut months = load_calendar(&self.calendar, &self.users, session).await?;

        for (month, stat) in months.iter_mut() {
            load_requests_and_history(
                session,
                *month,
                &self.requests,
                &self.history,
                &self.users,
                stat,
            )
            .await?;
            load_treasury(session, *month, &self.treasury, &self.users, stat).await?;
        }

        self.cache.set_value(months);
        info!("Statistics reloaded in {:?}", start.elapsed());
        Ok(self.cache.get_value().unwrap())
    }

    pub async fn ask_ai(
        &self,
        session: &mut Session,
        model: AiModel,
        prompt: String,
    ) -> Result<String, Error> {
        let system_prompt = if let Some(entry) = self.cache.get_value() {
            make_prompt(&entry.value)?
        } else {
            make_prompt(&self.reload_statistics(session).await?.value)?
        };
        let ctx = AiContext::with_system_prompt(system_prompt);

        let response = self.ai.ask(model, prompt.to_string(), Some(ctx)).await?;

        Ok(response.response)
    }
}

pub fn month_id(date: DateTime<Local>) -> NaiveDate {
    NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap()
}

pub fn month_range(month: NaiveDate) -> (DateTime<Local>, DateTime<Local>) {
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

    #[test]
    fn test_month_id() {
        let date = Local.ymd(2023, 10, 15).and_hms(0, 0, 0);
        let expected = NaiveDate::from_ymd(2023, 10, 1);
        assert_eq!(month_id(date), expected);
    }

    #[test]
    fn test_month_start_and() {
        let month = NaiveDate::from_ymd(2023, 10, 1);
        let (start, end) = month_range(month);

        let expected_start = Local.ymd(2023, 10, 1).and_hms(0, 0, 0);
        let expected_end = Local.ymd(2023, 10, 31).and_hms(23, 59, 59);

        assert_eq!(start, expected_start);
        assert_eq!(end, expected_end);
    }
}
