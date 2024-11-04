use bson::{doc, oid::ObjectId};
use chrono::{DateTime, Local, Utc};
use eyre::Error;
use futures_util::TryStreamExt as _;
use model::{history::HistoryRow, session::Session};
use mongodb::{Collection, IndexModel, SessionCursor};

const COLLECTION: &str = "history";

pub struct HistoryStore {
    store: Collection<HistoryRow>,
}

impl HistoryStore {
    pub(crate) async fn new(db: &mongodb::Database) -> Result<Self, Error> {
        let store = db.collection(COLLECTION);
        store
            .create_index(IndexModel::builder().keys(doc! { "date_time": -1 }).build())
            .await?;
        store
            .create_index(IndexModel::builder().keys(doc! { "actor": -1 }).build())
            .await?;
        store
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "sub_actors": -1 })
                    .build(),
            )
            .await?;
        Ok(HistoryStore { store })
    }

    pub async fn find_range(
        &self,
        session: &mut Session,
        from: Option<DateTime<Local>>,
        to: Option<DateTime<Local>>,
    ) -> Result<SessionCursor<HistoryRow>, Error> {
        let filter = match (from, to) {
            (Some(from), Some(to)) => doc! {
                "date_time": {
                    "$gte": from.with_timezone(&Utc),
                    "$lt": to.with_timezone(&Utc),
                }
            },
            (Some(from), None) => doc! {
                "date_time": {
                    "$gte": from.with_timezone(&Utc),
                }
            },
            (None, Some(to)) => doc! {
                "date_time": {
                    "$lt": to.with_timezone(&Utc),
                }
            },
            (None, None) => doc! {},
        };
        Ok(self.store.find(filter).session(&mut *session).await?)
    }

    pub async fn store(&self, session: &mut Session, entry: HistoryRow) -> Result<(), Error> {
        self.store.insert_one(entry).session(session).await?;
        Ok(())
    }

    pub async fn get_actor_logs(
        &self,
        session: &mut Session,
        actor: ObjectId,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<HistoryRow>, Error> {
        let mut cursor = self
            .store
            .find(doc! { "$or": [ { "actor": actor }, { "sub_actors": { "$elemMatch": { "$eq": actor } } } ] })
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

    pub async fn get_logs(
        &self,
        session: &mut Session,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<HistoryRow>, Error> {
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

    pub async fn dump(&self, session: &mut Session) -> Result<Vec<HistoryRow>, Error> {
        let mut cursor = self.store.find(doc! {}).session(&mut *session).await?;
        Ok(cursor.stream(&mut *session).try_collect().await?)
    }
}
