pub mod calendar;
pub mod session;
pub mod training;
pub mod treasury;
pub mod user;

use eyre::Result;
use session::Db;
use user::UserStore;

const DB_NAME: &str = "ledger_db";

pub struct Storage {
    pub db: Db,
    pub users: UserStore,
    pub calendar: calendar::CalendarStore,
    pub training: training::ProgramStore,
    pub treasury: treasury::TreasuryStore,
}

impl Storage {
    pub async fn new(uri: &str) -> Result<Self> {
        let db = Db::new(uri, DB_NAME).await?;
        let users = UserStore::new(&db).await?;
        let schedule = calendar::CalendarStore::new(&db).await?;
        let training = training::ProgramStore::new(&db);
        let treasury = treasury::TreasuryStore::new(&db).await?;
        Ok(Storage {
            db,
            users,
            calendar: schedule,
            training,
            treasury,
        })
    }
}
