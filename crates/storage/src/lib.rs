pub mod calendar;
pub mod training;
pub mod user;

use eyre::{Context as _, Result};
use mongodb::{bson::doc, Client, Database};
use user::UserStore;

const DB_NAME: &str = "ledger_db";

pub struct Storage {
    _client: Client,
    _db: Database,
    pub users: UserStore,
    pub schedule: calendar::CalendarStore,
    pub training: training::TrainingStore,
}

impl Storage {
    pub async fn new(uri: &str) -> Result<Self> {
        let client = Client::with_uri_str(uri)
            .await
            .context("Failed to connect to MongoDB")?;
        let db = client.database(DB_NAME);
        db.run_command(doc! { "ping": 1 })
            .await
            .context("Failed to ping MongoDB")?;
        let users = UserStore::new(&db);
        let schedule = calendar::CalendarStore::new(&db).await?;
        let training = training::TrainingStore::new(&db);
        Ok(Storage {
            _client: client,
            _db: db,
            users,
            schedule,
            training,
        })
    }
}
