use std::sync::Arc;

use crate::session::Db;
use bson::{doc, oid::ObjectId};
use eyre::Error;
use log::info;
use model::{decimal::Decimal, session::Session, subscription::Subscription};
use mongodb::Collection;

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
        session: &mut Session,
        subscription: Subscription,
    ) -> Result<(), Error> {
        info!("Inserting subscription: {:?}", subscription);
        self.collection
            .insert_one(subscription)
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn delete(&self, session: &mut Session, id: ObjectId) -> Result<(), Error> {
        self.collection
            .delete_one(doc! { "_id": id })
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn cursor(
        &self,
        session: &mut Session,
    ) -> Result<mongodb::SessionCursor<Subscription>, Error> {
        Ok(self.collection.find(doc! {}).session(&mut *session).await?)
    }

    pub async fn get(
        &self,
        session: &mut Session,
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
        session: &mut Session,
        name: &str,
    ) -> Result<Option<Subscription>, Error> {
        Ok(self
            .collection
            .find_one(doc! { "name": name })
            .session(&mut *session)
            .await?)
    }

    pub async fn edit_price(
        &self,
        session: &mut Session,
        id: ObjectId,
        price: Decimal,
    ) -> Result<(), Error> {
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$set": {"price": price.inner()}
                },
            )
            .session(session)
            .await?;
        Ok(())
    }

    pub async fn edit_items(
        &self,
        session: &mut Session,
        id: ObjectId,
        items: u32,
    ) -> Result<(), Error> {
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$set": {"items": items}
                },
            )
            .session(session)
            .await?;
        Ok(())
    }

    pub async fn edit_name(
        &self,
        session: &mut Session,
        id: ObjectId,
        name: String,
    ) -> Result<(), Error> {
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$set": {"name": name}
                },
            )
            .session(session)
            .await?;
        Ok(())
    }

    pub async fn dump(&self, session: &mut Session) -> Result<Vec<Subscription>, Error> {
        let mut cursor = self.collection.find(doc! {}).session(&mut *session).await?;
        let mut subscriptions = Vec::new();
        while let Some(subscription) = cursor.next(&mut *session).await {
            subscriptions.push(subscription?);
        }
        Ok(subscriptions)
    }
}
