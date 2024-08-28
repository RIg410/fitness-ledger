pub mod user;
mod date_time;

use eyre::{Context as _, Result};
use mongodb::{Client, Collection, Database};
use user::User;

const DB_NAME: &str = "ledger_db";

pub struct Storage {
    pub(crate) client: Client,
    pub(crate) db: Database,
    pub(crate) users: Collection<User>,
}

impl Storage {
    pub async fn new(uri: &str) -> Result<Self> {
        let client = Client::with_uri_str(uri)
            .await
            .context("Failed to connect to MongoDB")?;
        let db = client.database(DB_NAME);
        let users = db.collection(user::COLLECTION);
        Ok(Storage { client, db, users })
    }
}
