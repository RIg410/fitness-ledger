pub mod aggregation;
pub mod calendar;
pub mod history;
pub mod prompt;
pub mod treasury;
pub mod clients;


use super::{
    calendar::Calendar, history::History, requests::Requests, treasury::Treasury, users::Users,
};
use aggregation::RequiredAggregations;
use ai::{Ai, AiContext, AiModel};
use chrono::{DateTime, Datelike as _, Local, NaiveDate};
use eyre::Error;
use model::session::Session;
use prompt::select_aggregation;

pub struct Statistics {
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
            treasury,
        }
    }

    // async fn reload_statistics(&self, session: &mut Session) -> Result<Arc<CacheEntry>, Error> {
    //     let start = Instant::now();
    //     info!("Reloading statistics...");
    //     let mut months = load_calendar(&self.calendar, session).await?;

    //     for (month, stat) in months.iter_mut() {
    //         load_requests_and_history(
    //             session,
    //             *month,
    //             &self.requests,
    //             &self.history,
    //             &self.users,
    //             stat,
    //         )
    //         .await?;
    //         load_treasury(session, *month, &self.treasury, &self.users, stat).await?;
    //     }

    //     self.cache.set_value(months);
    //     info!("Statistics reloaded in {:?}", start.elapsed());
    //     Ok(self.cache.get_value().unwrap())
    // }

    pub async fn ask_ai(
        &self,
        session: &mut Session,
        model: AiModel,
        prompt: String,
    ) -> Result<String, Error> {
        let request_aggregation = select_aggregation(&prompt);

        let mut ctx = AiContext::default();
        let response = self.ai.ask(model, request_aggregation, &mut ctx).await?;
        let aggr: RequiredAggregations = serde_json::from_str(&response.response)?;
        let aggregations = self.load_aggregations(session, aggr).await?;

        todo!()
        // Ok(response.response)
    }

    async fn load_aggregations(
        &self,
        session: &mut Session,
        aggr: RequiredAggregations,
    ) -> Result<String, Error> {
        todo!()
    }
}

pub fn month_id(date: DateTime<Local>) -> NaiveDate {
    NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap()
}
