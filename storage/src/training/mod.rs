use std::sync::Arc;

use futures_util::TryStreamExt as _;
use model::TrainingProto;
use mongodb::{bson::{doc, oid::ObjectId}, Collection};

pub mod model;

const COLLECTION: &str = "training";

#[derive(Clone)]
pub struct TrainingStore {
    pub(crate) store: Arc<Collection<TrainingProto>>,
}

impl TrainingStore {
    pub(crate) fn new(db: &mongodb::Database) -> Self {
        let store = db.collection(COLLECTION);

        TrainingStore {
            store: Arc::new(store),
        }
    }

    pub async fn get_by_id(
        &self,
        id: ObjectId,
    ) -> Result<Option<TrainingProto>, mongodb::error::Error> {
        Ok(self.store.find_one(doc! { "_id": id }).await?)
    }

    pub async fn find(
        &self,
        query: Option<&str>,
    ) -> Result<Vec<TrainingProto>, mongodb::error::Error> {
        let filter = if let Some(query) = query {
            doc! {
                "name": { "$regex": query, "$options": "i" }
            }
        } else {
            doc! {}
        };

        let cursor = self.store.find(filter).await?;
        Ok(cursor.try_collect().await?)
    }

    pub async fn get_by_name(
        &self,
        name: &str,
    ) -> Result<Option<TrainingProto>, mongodb::error::Error> {
        Ok(self
            .store
            .find_one(doc! { "name": { "$regex": name, "$options": "i" } })
            .await?)
    }

    pub async fn insert(&self, proto: &TrainingProto) -> Result<(), mongodb::error::Error> {
        self.store.insert_one(proto).await?;
        Ok(())
    }

    pub async fn delete(&self, proto: &TrainingProto) -> Result<(), mongodb::error::Error> {
        self.store.delete_one(doc! { "id": proto.id }).await?;
        Ok(())
    }

    pub async fn update(&self, proto: &TrainingProto) -> Result<(), mongodb::error::Error> {
        self.store
            .update_one(
                doc! { "id": proto.id },
                doc! { "$set": mongodb::bson::to_document(proto)? },
            )
            .await?;
        Ok(())
    }
}
