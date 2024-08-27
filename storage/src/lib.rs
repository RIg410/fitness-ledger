use eyre::{Context as _, Result};
use mongodb::Client;

pub struct Storage {
    client: Client,
}

impl Storage {
    pub async fn new(uri: &str) -> Result<Self> {
        let client = Client::with_uri_str(uri)
            .await
            .context("Failed to connect to MongoDB")?;
        Ok(Storage { client })
    }
}
