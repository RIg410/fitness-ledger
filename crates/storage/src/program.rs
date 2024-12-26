use bson::to_document;
use eyre::Error;
use futures_util::TryStreamExt as _;
use model::{program::Program, session::Session};
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::UpdateOptions,
    Collection,
};

const COLLECTION: &str = "training";

pub struct ProgramStore {
    pub(crate) store: Collection<Program>,
}

impl ProgramStore {
    pub(crate) fn new(db: &mongodb::Database) -> Self {
        let store = db.collection(COLLECTION);
        ProgramStore { store }
    }

    pub async fn get_by_id(
        &self,
        session: &mut Session,
        id: ObjectId,
    ) -> Result<Option<Program>, Error> {
        Ok(self
            .store
            .find_one(doc! { "_id": id })
            .session(&mut *session)
            .await?)
    }

    pub async fn get_all(
        &self,
        session: &mut Session,
        only_visible: bool,
    ) -> Result<Vec<Program>, Error> {
        let filter = if only_visible {
            doc! { "$or": [ { "visible": true }, { "visible": { "$exists": false } } ] }
        } else {
            doc! {}
        };

        let mut cursor = self.store.find(filter).session(&mut *session).await?;
        Ok(cursor.stream(&mut *session).try_collect().await?)
    }

    pub async fn find(
        &self,
        session: &mut Session,
        query: Option<&str>,
    ) -> Result<Vec<Program>, Error> {
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
        session: &mut Session,
        name: &str,
    ) -> Result<Option<Program>, Error> {
        Ok(self
            .store
            .find_one(doc! { "name": { "$regex": name, "$options": "i" } })
            .session(&mut *session)
            .await?)
    }

    pub async fn insert(&self, session: &mut Session, proto: &Program) -> Result<(), Error> {
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

    pub async fn delete(&self, session: &mut Session, id: &ObjectId) -> Result<(), Error> {
        self.store
            .delete_one(doc! { "_id": id })
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn set_visible(
        &self,
        session: &mut Session,
        id: &ObjectId,
        visible: bool,
    ) -> Result<(), Error> {
        self.store
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "visible": visible }, "$inc" : { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn update_name(
        &self,
        session: &mut Session,
        id: &ObjectId,
        name: &str,
    ) -> Result<(), Error> {
        self.store
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "name": name }, "$inc" : { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn update_description(
        &self,
        session: &mut Session,
        id: &ObjectId,
        description: &str,
    ) -> Result<(), Error> {
        self.store
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "description": description }, "$inc" : { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn update_duration(
        &self,
        session: &mut Session,
        id: &ObjectId,
        duration: u32,
    ) -> Result<(), Error> {
        self.store
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "duration_min": duration }, "$inc" : { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn update_capacity(
        &self,
        session: &mut Session,
        id: &ObjectId,
        capacity: u32,
    ) -> Result<(), Error> {
        self.store
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "capacity": capacity }, "$inc" : { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn edit_capacity(
        &self,
        session: &mut Session,
        id: ObjectId,
        capacity: u32,
    ) -> Result<(), Error> {
        self.store
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "capacity": capacity }, "$inc" : { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn edit_duration(
        &self,
        session: &mut Session,
        id: ObjectId,
        duration: u32,
    ) -> Result<(), Error> {
        self.store
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "duration_min": duration }, "$inc" : { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn edit_name(
        &self,
        session: &mut Session,
        id: ObjectId,
        name: String,
    ) -> Result<(), Error> {
        self.store
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "name": name }, "$inc" : { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn edit_description(
        &self,
        session: &mut Session,
        id: ObjectId,
        description: String,
    ) -> Result<(), Error> {
        self.store
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "description": description }, "$inc" : { "version": 1 } },
            )
            .session(&mut *session)
            .await?;
        Ok(())
    }
}
