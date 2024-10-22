pub mod auth_key;
pub mod calendar;
pub mod history;
pub mod pre_sell;
pub mod program;
pub mod requests;
pub mod rewards;
pub mod session;
pub mod subscription;
pub mod treasury;
pub mod user;

use eyre::Result;
use history::HistoryStore;
use requests::RequestStore;
use rewards::RewardsStore;
use session::Db;
use user::UserStore;

const DB_NAME: &str = "ledger_db";

#[derive(Clone)]
pub struct Storage {
    pub db: Db,
    pub users: UserStore,
    pub calendar: calendar::CalendarStore,
    pub programs: program::ProgramStore,
    pub treasury: treasury::TreasuryStore,
    pub subscriptions: subscription::SubscriptionsStore,
    pub history: HistoryStore,
    pub presell: pre_sell::PreSellStore,
    pub rewards: RewardsStore,
    pub requests: RequestStore,
    pub auth_keys: auth_key::AuthKeys,
}

impl Storage {
    pub async fn new(uri: &str) -> Result<Self> {
        let db = Db::new(uri, DB_NAME).await?;
        let users = UserStore::new(&db).await?;
        let calendar = calendar::CalendarStore::new(&db).await?;
        let programs = program::ProgramStore::new(&db);
        let treasury = treasury::TreasuryStore::new(&db).await?;
        let subscriptions = subscription::SubscriptionsStore::new(&db);
        let presell = pre_sell::PreSellStore::new(&db).await?;
        let history = history::HistoryStore::new(&db).await?;
        let rewards = RewardsStore::new(&db).await?;
        let requests = RequestStore::new(&db).await?;
        let auth_keys = auth_key::AuthKeys::new(&db).await?;

        Ok(Storage {
            db,
            users,
            calendar,
            programs,
            treasury,
            subscriptions,
            history,
            presell,
            rewards,
            requests,
            auth_keys,
        })
    }
}
