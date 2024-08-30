use crate::{Ledger, MAX_WEEKS};
use chrono::NaiveDate;
use eyre::Result;
use storage::schedule::model::{Day, Week};

impl Ledger {
    pub async fn get_week(&self, date: Option<NaiveDate>) -> Result<Week> {
        let date = date.unwrap_or_else(|| chrono::Local::now().naive_local().date());
        if !self.has_week(date) {
            return Err(eyre::eyre!("Week is too far in the future"));
        }

        self.schedule.get_week(date).await
    }

    pub fn has_week(&self, id: NaiveDate) -> bool {
        chrono::Local::now().naive_local().date() + chrono::Duration::days(7 * MAX_WEEKS as i64)
            >= id
    }

    pub fn has_next_week(&self, week: &Week) -> bool {
        self.has_week(week.id + chrono::Duration::days(7))
    }

    pub fn has_prev_week(&self, week: &Week) -> bool {
        week.id - chrono::Duration::days(1) >= chrono::Local::now().naive_local().date()
    }

    pub async fn get_day(&self, day: chrono::NaiveDate) -> Result<Day> {
        self.schedule
            .get_week(day)
            .await?
            .days
            .into_iter()
            .find(|d| d.date == day)
            .ok_or_else(|| eyre::eyre!("Day not found"))
    }
}
