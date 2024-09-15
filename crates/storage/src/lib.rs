pub mod calendar;
pub mod logs;
pub mod session;
pub mod subscription;
pub mod training;
pub mod treasury;
pub mod user;
pub mod pre_cell;

use eyre::Result;
use logs::LogStore;
use session::Db;
use user::UserStore;

const DB_NAME: &str = "ledger_db";

pub struct Storage {
    pub db: Db,
    pub users: UserStore,
    pub calendar: calendar::CalendarStore,
    pub training: training::ProgramStore,
    pub treasury: treasury::TreasuryStore,
    pub subscriptions: subscription::SubscriptionsStore,
    pub logs: LogStore,
}

impl Storage {
    pub async fn new(uri: &str) -> Result<Self> {
        let db = Db::new(uri, DB_NAME).await?;
        let users = UserStore::new(&db).await?;
        let schedule = calendar::CalendarStore::new(&db).await?;
        let training = training::ProgramStore::new(&db);
        let treasury = treasury::TreasuryStore::new(&db).await?;
        let subscriptions = subscription::SubscriptionsStore::new(&db);
        let logs = logs::LogStore::new(&db).await?;
        Ok(Storage {
            db,
            users,
            calendar: schedule,
            training,
            treasury,
            subscriptions,
            logs,
        })
    }
}
