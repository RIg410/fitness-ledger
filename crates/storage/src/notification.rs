use crate::session::Db;
use bson::{doc, oid::ObjectId};
use eyre::Result;
use futures_util::TryStreamExt as _;
use model::{
    notification::{Notification, NotificationId},
    session::Session,
};
use mongodb::{options::IndexOptions, Collection, IndexModel};

const TABLE_NAME: &str = "notifications";

pub struct NotificationStore {
    store: Collection<Notification>,
}

impl NotificationStore {
    pub async fn new(db: &Db) -> Result<Self> {
        let store = db.collection(TABLE_NAME);
        store
            .create_index(IndexModel::builder().keys(doc! { "to": 1 }).build())
            .await?;
        store
            .create_index(IndexModel::builder().keys(doc! { "sent": 1 }).build())
            .await?;
        store
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "user_notification_time": 1 })
                    .build(),
            )
            .await?;

        store
            .create_index(IndexModel::builder().keys(doc! { "deadline": 1 }).build())
            .await?;

        store
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "bassness_id": 1 })
                    .options(IndexOptions::builder().unique(true).build())
                    .build(),
            )
            .await?;

        Ok(NotificationStore { store })
    }

    pub async fn has(&self, session: &mut Session, bassness_id: NotificationId) -> Result<bool> {
        let filter = doc! {"bassness_id": bassness_id.encode() };
        let count = self.store.count_documents(filter).session(session).await?;
        Ok(count > 0)
    }

    pub async fn get(
        &self,
        session: &mut Session,
        bassness_id: NotificationId,
    ) -> Result<Option<Notification>> {
        let filter = doc! {"bassness_id": bassness_id.encode() };
        Ok(self.store.find_one(filter).session(session).await?)
    }

    pub async fn insert(&self, session: &mut Session, notification: Notification) -> Result<()> {
        self.store.insert_one(notification).session(session).await?;
        Ok(())
    }

    pub async fn get_by_id(
        &self,
        session: &mut Session,
        id: ObjectId,
    ) -> Result<Option<Notification>> {
        let filter = doc! {"_id": id };
        Ok(self.store.find_one(filter).session(session).await?)
    }

    pub async fn remove(&self, session: &mut Session, id: ObjectId) -> Result<()> {
        self.store
            .delete_one(doc! {"_id": id })
            .session(session)
            .await?;
        Ok(())
    }

    pub async fn collect_garbage(&self, session: &mut Session) -> Result<()> {
        let filter = doc! {
            "deadline": {
                "$lt": chrono::Utc::now()
            }
        };
        self.store.delete_many(filter).session(session).await?;
        Ok(())
    }

    pub async fn mark_as_sent(&self, session: &mut Session, id: ObjectId) -> Result<()> {
        self.store
            .update_one(doc! {"_id": id}, doc! {"$set": {"sent": true}})
            .session(session)
            .await?;
        Ok(())
    }

    pub async fn to_send(&self, session: &mut Session) -> Result<Vec<Notification>> {
        let filter = doc! {
            "sent": false,
            "user_notification_time": {
                "$lt": chrono::Utc::now()
            }
        };

        let mut cursor = self.store.find(filter).session(&mut *session).await?;
        Ok(cursor.stream(&mut *session).try_collect().await?)
    }
}
