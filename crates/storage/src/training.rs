use std::sync::Arc;

use bson::to_document;
use eyre::Error;
use futures_util::TryStreamExt as _;
use model::proto::TrainingProto;
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::UpdateOptions,
    ClientSession, Collection,
};

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
        session: &mut ClientSession,
        id: ObjectId,
    ) -> Result<Option<TrainingProto>, Error> {
        Ok(self
            .store
            .find_one(doc! { "_id": id })
            .session(&mut *session)
            .await?)
    }

    pub async fn get_all(&self, session: &mut ClientSession) -> Result<Vec<TrainingProto>, Error> {
        let mut cursor = self.store.find(doc! {}).session(&mut *session).await?;
        Ok(cursor.stream(&mut *session).try_collect().await?)
    }

    pub async fn find(
        &self,
        session: &mut ClientSession,
        query: Option<&str>,
    ) -> Result<Vec<TrainingProto>, Error> {
        let filter = if let Some(query) = query {
            doc! {
                "name": { "$regex": query, "$options": "i" }
            }
        } else {
            doc! {}
        };

        let mut cursor = self.store.find(filter).session(&mut *session).await?;
        Ok(cursor.stream(&mut *session).try_collect().await?)
    }

    pub async fn get_by_name(
        &self,
        session: &mut ClientSession,
        name: &str,
    ) -> Result<Option<TrainingProto>, Error> {
        Ok(self
            .store
            .find_one(doc! { "name": { "$regex": name, "$options": "i" } })
            .session(&mut *session)
            .await?)
    }

    pub async fn insert(
        &self,
        session: &mut ClientSession,
        proto: &TrainingProto,
    ) -> Result<(), Error> {
        let result = self
            .store
            .update_one(
                doc! { "name": proto.name.clone() },
                doc! { "$setOnInsert": to_document(proto)? },
            )
            .session(&mut *session)
            .with_options(UpdateOptions::builder().upsert(true).build())
            .await?;

        if result.upserted_id.is_none() {
            return Err(Error::msg("Training already exists"));
        }
        Ok(())
    }

    pub async fn delete(
        &self,
        session: &mut ClientSession,
        proto: &TrainingProto,
    ) -> Result<(), Error> {
        self.store
            .delete_one(doc! { "id": proto.id })
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn update_name(
        &self,
        session: &mut ClientSession,
        proto: &TrainingProto,
        name: &str,
    ) -> Result<(), Error> {
        self.store
            .update_one(
                doc! { "id": proto.id },
                doc! { "$set": { "name": name }, "$inc" : { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn update_description(
        &self,
        session: &mut ClientSession,
        proto: &TrainingProto,
        description: &str,
    ) -> Result<(), Error> {
        self.store
            .update_one(
                doc! { "id": proto.id },
                doc! { "$set": { "description": description }, "$inc" : { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn update_duration(
        &self,
        session: &mut ClientSession,
        proto: &TrainingProto,
        duration: u32,
    ) -> Result<(), Error> {
        self.store
            .update_one(
                doc! { "id": proto.id },
                doc! { "$set": { "duration_min": duration }, "$inc" : { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn update_capacity(
        &self,
        session: &mut ClientSession,
        proto: &TrainingProto,
        capacity: u32,
    ) -> Result<(), Error> {
        self.store
            .update_one(
                doc! { "id": proto.id },
                doc! { "$set": { "capacity": capacity }, "$inc" : { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        Ok(())
    }
}
