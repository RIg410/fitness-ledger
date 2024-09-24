use std::sync::Arc;

use bson::doc;
use chrono::{DateTime, Local};
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
            .keys(doc! { "date_time": -1 })
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

    pub async fn remove(
        &self,
        session: &mut Session,
        id: bson::oid::ObjectId,
    ) -> Result<(), Error> {
        self.store
            .delete_one(doc! { "_id": id })
            .session(session)
            .await?;
        Ok(())
    }

    pub async fn list(
        &self,
        session: &mut Session,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<TreasuryEvent>, Error> {
        let mut cursor = self
            .store
            .find(doc! {})
            .sort(doc! { "date_time": -1 })
            .skip(offset)
            .limit(limit as i64)
            .session(&mut *session)
            .await?;
        let mut events = Vec::new();
        while let Some(event) = cursor.next(&mut *session).await {
            events.push(event?);
        }
        Ok(events)
    }

    pub async fn range(
        &self,
        session: &mut Session,
        from: DateTime<Local>,
        to: DateTime<Local>,
    ) -> Result<Vec<TreasuryEvent>, Error> {
        let mut cursor = self
            .store
            .find(doc! {
                "date_time": {
                    "$gte": from,
                    "$lt": to,
                }
            })
            .sort(doc! { "date_time": -1 })
            .session(&mut *session)
            .await?;
        let mut events = Vec::new();
        while let Some(event) = cursor.next(&mut *session).await {
            events.push(event?);
        }
        Ok(events)
    }

    pub async fn get(
        &self,
        session: &mut Session,
        id: bson::oid::ObjectId,
    ) -> Result<Option<TreasuryEvent>, Error> {
        Ok(self
            .store
            .find_one(doc! { "_id": id })
            .session(session)
            .await?)
    }
}
