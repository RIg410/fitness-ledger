use std::sync::Arc;

use crate::session::Db;
use bson::{doc, oid::ObjectId};
use eyre::Error;
use log::info;
use model::subscription::Subscription;
use mongodb::{ClientSession, Collection};

const TABLE_NAME: &str = "subscriptions";

#[derive(Clone)]
pub struct SubscriptionsStore {
    collection: Arc<Collection<Subscription>>,
}

impl SubscriptionsStore {
    pub fn new(db: &Db) -> Self {
        SubscriptionsStore {
            collection: Arc::new(db.collection(TABLE_NAME)),
        }
    }

    pub async fn insert(
        &self,
        session: &mut ClientSession,
        subscription: Subscription,
    ) -> Result<(), Error> {
        info!("Inserting subscription: {:?}", subscription);
        self.collection
            .insert_one(subscription)
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn delete(&self, session: &mut ClientSession, id: ObjectId) -> Result<(), Error> {
        self.collection
            .delete_one(doc! { "_id": id })
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn cursor(
        &self,
        session: &mut ClientSession,
    ) -> Result<mongodb::SessionCursor<Subscription>, Error> {
        Ok(self.collection.find(doc! {}).session(&mut *session).await?)
    }

    pub async fn get_by_id(
        &self,
        session: &mut ClientSession,
        id: ObjectId,
    ) -> Result<Option<Subscription>, Error> {
        Ok(self
            .collection
            .find_one(doc! { "_id": id })
            .session(&mut *session)
            .await?)
    }

    pub async fn get_by_name(
        &self,
        session: &mut ClientSession,
        name: &str,
    ) -> Result<Option<Subscription>, Error> {
        Ok(self
            .collection
            .find_one(doc! { "name": name })
            .session(&mut *session)
            .await?)
    }
}