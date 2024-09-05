use std::ops::Deref;

use bson::doc;
use eyre::{Context as _, Error};
use mongodb::{Client, ClientSession, Database};

#[derive(Clone)]
pub struct Db {
    client: Client,
    db: Database,
}

impl Db {
    pub(crate) async fn new(uri: &str, db_name: &str) -> Result<Self, Error> {
        let client = Client::with_uri_str(uri)
            .await
            .context("Failed to connect to MongoDB")?;
        let db = client.database(db_name);
        db.run_command(doc! { "ping": 1 })
            .await
            .context("Failed to ping MongoDB")?;
        Ok(Db { client, db })
    }

    pub async fn start_session(&self) -> Result<ClientSession, Error> {
        Ok(self.client.start_session().await?)
    }
}

impl Deref for Db {
    type Target = Database;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}
