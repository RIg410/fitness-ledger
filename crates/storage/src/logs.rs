use bson::doc;
use chrono::{DateTime, Local, Utc};
use eyre::Error;
use model::{log::LogEntry, session::Session};
use mongodb::{Collection, IndexModel};
use std::sync::Arc;

const COLLECTION: &str = "logs";

#[derive(Clone)]
pub struct LogStore {
    store: Arc<Collection<LogEntry>>,
}

impl LogStore {
    pub(crate) async fn new(db: &mongodb::Database) -> Result<Self, Error> {
        let store = db.collection(COLLECTION);
        store
            .create_index(IndexModel::builder().keys(doc! { "date_time": -1 }).build())
            .await?;

        Ok(LogStore {
            store: Arc::new(store),
        })
    }

    pub async fn store(&self, session: &mut Session, entry: LogEntry) -> Result<(), Error> {
        self.store.insert_one(entry).session(session).await?;
        Ok(())
    }

    pub async fn get_logs(
        &self,
        session: &mut Session,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<LogEntry>, Error> {
        let mut cursor = self
            .store
            .find(doc! {})
            .sort(doc! { "date_time": -1 })
            .skip(offset as u64)
            .session(&mut *session)
            .await?;
        let mut logs = Vec::with_capacity(limit);
        while let Some(log) = cursor.next(&mut *session).await {
            logs.push(log?);
            if logs.len() >= limit {
                break;
            }
        }
        Ok(logs)
    }

    pub async fn gc(
        &self,
        session: &mut Session,
        date_time: DateTime<Local>,
    ) -> Result<u64, Error> {
        let result = self
            .store
            .delete_many(doc! { "date_time": { "$lt": date_time.with_timezone(&Utc) } })
            .session(session)
            .await?;
        Ok(result.deleted_count)
    }
}
