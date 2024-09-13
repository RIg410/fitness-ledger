use std::sync::Arc;

use bson::doc;
use eyre::Error;
use model::{session::Session, treasury::TreasuryEvent};
use mongodb::{options::IndexOptions, Collection, IndexModel};

const COLLECTION: &str = "treasury";

#[derive(Clone)]
pub struct TreasuryStore {
    store: Arc<Collection<TreasuryEvent>>,
}

impl TreasuryStore {
    pub async fn new(db: &mongodb::Database) -> Result<Self, Error> {
        let store = db.collection(COLLECTION);
        let index = IndexModel::builder()
            .keys(doc! { "date_time": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();
        store.create_index(index).await?;
        Ok(TreasuryStore {
            store: Arc::new(store),
        })
    }

    pub async fn insert(&self, session: &mut Session, event: TreasuryEvent) -> Result<(), Error> {
        self.store.insert_one(event).session(session).await?;
        Ok(())
    }
}
