use chrono::NaiveDate;
use model::session::Session;

pub mod calendar;

#[async_trait::async_trait]
pub trait LoadAggregation {
    fn load(&self, session: &mut Session, month: &NaiveDate) -> eyre::Result<String>;
}

