pub mod calendar;
pub mod history;
pub mod payment;
pub mod program;
pub mod requests;
pub mod rewards;
pub mod session;
pub mod subscription;
pub mod treasury;
pub mod user;
pub mod dump;

use eyre::Result;
use history::HistoryStore;
use requests::RequestStore;
use rewards::RewardsStore;
use session::Db;
use std::sync::Arc;
use user::UserStore;

const DB_NAME: &str = "ledger_db";

#[derive(Clone)]
pub struct Storage {
    pub db: Arc<Db>,
    pub users: Arc<UserStore>,
    pub calendar: Arc<calendar::CalendarStore>,
    pub programs: Arc<program::ProgramStore>,
    pub treasury: Arc<treasury::TreasuryStore>,
    pub subscriptions: Arc<subscription::SubscriptionsStore>,
    pub history: Arc<HistoryStore>,
    pub rewards: Arc<RewardsStore>,
    pub requests: Arc<RequestStore>,
}

impl Storage {
    pub async fn new(uri: &str) -> Result<Self> {
        let db = Db::new(uri, DB_NAME).await?;
        let users = UserStore::new(&db).await?;
        let calendar = calendar::CalendarStore::new(&db).await?;
        let programs = program::ProgramStore::new(&db);
        let treasury = treasury::TreasuryStore::new(&db).await?;
        let subscriptions = subscription::SubscriptionsStore::new(&db);
        let history = history::HistoryStore::new(&db).await?;
        let rewards = RewardsStore::new(&db).await?;
        let requests = RequestStore::new(&db).await?;

        Ok(Storage {
            db: Arc::new(db),
            users: Arc::new(users),
            calendar: Arc::new(calendar),
            programs: Arc::new(programs),
            treasury: Arc::new(treasury),
            subscriptions: Arc::new(subscriptions),
            history: Arc::new(history),
            rewards: Arc::new(rewards),
            requests: Arc::new(requests),
        })
    }
}
