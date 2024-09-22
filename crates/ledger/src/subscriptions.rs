use eyre::Error;
use model::{decimal::Decimal, session::Session, subscription::Subscription};
use mongodb::bson::oid::ObjectId;
use storage::subscription::SubscriptionsStore;
use thiserror::Error;
use tx_macro::tx;

use crate::history::History;

#[derive(Clone)]
pub struct Subscriptions {
    pub store: SubscriptionsStore,
    pub logs: History,
}

impl Subscriptions {
    pub fn new(store: SubscriptionsStore, logs: History) -> Self {
        Subscriptions { store, logs }
    }

    pub async fn get_by_name(
        &self,
        session: &mut Session,
        name: &str,
    ) -> Result<Option<Subscription>, Error> {
        self.store.get_by_name(session, name).await
    }

    pub async fn get(
        &self,
        session: &mut Session,
        id: ObjectId,
    ) -> Result<Option<Subscription>, Error> {
        self.store.get_by_id(session, id).await
    }

    pub async fn get_all(&self, session: &mut Session) -> Result<Vec<Subscription>, Error> {
        let mut cursor = self.store.cursor(session).await?;
        let mut result = Vec::new();
        while let Some(subscription) = cursor.next(session).await {
            result.push(subscription?);
        }
        Ok(result)
    }

    #[tx]
    pub async fn delete(&self, session: &mut Session, id: ObjectId) -> Result<(), Error> {
        // let sub = self
        //     .get(session, id)
        //     .await?
        //     .ok_or_else(|| eyre::eyre!("Subscription not found"))?;
        //self.logs.delete_sub(session, sub).await;
        self.store.delete(session, id).await?;
        Ok(())
    }

    #[tx]
    pub async fn create_subscription(
        &self,
        session: &mut Session,
        name: String,
        items: u32,
        price: Decimal,
        freeze_days: u32,
        expiration_days: u32,
    ) -> Result<(), CreateSubscriptionError> {
        if self.get_by_name(session, &name).await?.is_some() {
            return Err(CreateSubscriptionError::NameAlreadyExists);
        }
        if items == 0 {
            return Err(CreateSubscriptionError::InvalidItems);
        }

        if price <= Decimal::zero() {
            return Err(CreateSubscriptionError::InvalidPrice);
        }
        let sub = Subscription::new(name, items, price, expiration_days, freeze_days);
        //self.logs.create_sub(session, sub.clone()).await;
        self.store.insert(session, sub).await?;
        Ok(())
    }

    #[tx]
    pub async fn edit_price(
        &self,
        session: &mut Session,
        id: ObjectId,
        value: Decimal,
    ) -> Result<(), Error> {
        //self.logs.edit_sub_price(session, id, value).await;
        self.store.edit_price(session, id, value).await?;
        Ok(())
    }

    #[tx]
    pub async fn edit_items(
        &self,
        session: &mut Session,
        id: ObjectId,
        value: u32,
    ) -> Result<(), Error> {
        //self.logs.edit_sub_items(session, id, value).await;
        self.store.edit_items(session, id, value).await?;
        Ok(())
    }

    #[tx]
    pub async fn edit_name(
        &self,
        session: &mut Session,
        id: ObjectId,
        value: String,
    ) -> Result<(), Error> {
        //self.logs.edit_sub_name(session, id, value.clone()).await;
        self.store.edit_name(session, id, value).await?;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum CreateSubscriptionError {
    #[error("Subscription with this name already exists")]
    NameAlreadyExists,
    #[error("Invalid items count")]
    InvalidItems,
    #[error("Invalid price")]
    InvalidPrice,
    #[error(transparent)]
    Common(#[from] Error),
}

impl From<mongodb::error::Error> for CreateSubscriptionError {
    fn from(err: mongodb::error::Error) -> Self {
        CreateSubscriptionError::Common(err.into())
    }
}
