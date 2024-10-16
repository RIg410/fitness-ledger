pub mod calendar;
pub mod history;
pub mod pre_sell;
pub mod program;
pub mod rewards;
pub mod session;
pub mod subscription;
pub mod treasury;
pub mod user;
pub mod requests;

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
}

impl Storage {
    pub async fn new(uri: &str) -> Result<Self> {
        let db = Db::new(uri, DB_NAME).await?;
        let users = UserStore::new(&db).await?;
        let schedule = calendar::CalendarStore::new(&db).await?;
        let training = program::ProgramStore::new(&db);
        let treasury = treasury::TreasuryStore::new(&db).await?;
        let subscriptions = subscription::SubscriptionsStore::new(&db);
        let presell = pre_sell::PreSellStore::new(&db).await?;
        let logs = history::HistoryStore::new(&db).await?;
        let couch = RewardsStore::new(&db).await?;
        let requests = RequestStore::new(&db).await?;

        Ok(Storage {
            db,
            users,
            calendar: schedule,
            programs: training,
            treasury,
            subscriptions,
            history: logs,
            presell,
            rewards: couch,
            requests,
        })
    }
}
