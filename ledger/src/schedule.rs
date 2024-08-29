use chrono::NaiveDate;
use eyre::Result;
use storage::schedule::model::Week;

use crate::Ledger;

impl Ledger {
    pub async fn get_week(&self, date: Option<NaiveDate>) -> Result<Week> {
        let date = date.unwrap_or_else(|| chrono::Local::now().naive_local().date());
        self.schedule.get_week(date).await
    }
}
