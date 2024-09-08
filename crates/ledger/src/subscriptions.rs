use eyre::Error;
use model::{decimal::Decimal, subscription::Subscription};
use mongodb::{bson::oid::ObjectId, ClientSession};
use storage::subscription::SubscriptionsStore;
use thiserror::Error;
use tx_macro::tx;

#[derive(Clone)]
pub struct Subscriptions {
    pub store: SubscriptionsStore,
}

impl Subscriptions {
    pub fn new(store: SubscriptionsStore) -> Self {
        Subscriptions { store }
    }

    pub async fn get_by_name(
        &self,
        session: &mut ClientSession,
        name: &str,
    ) -> Result<Option<Subscription>, Error> {
        Ok(self.store.get_by_name(session, name).await?)
    }

    pub async fn get(&self, session: &mut ClientSession, id: ObjectId) -> Result<Option<Subscription>, Error> {
        Ok(self.store.get_by_id(session, id).await?)
    }

    pub async fn get_all(&self, session: &mut ClientSession) -> Result<Vec<Subscription>, Error> {
        let mut cursor = self.store.cursor(session).await?;
        let mut result = Vec::new();
        while let Some(subscription) = cursor.next(session).await {
            result.push(subscription?);
        }
        Ok(result)
    }

    pub async fn delete(&self, session: &mut ClientSession, id: ObjectId) -> Result<(), Error> {
        self.store.delete(session, id).await?;
        Ok(())
    }

    #[tx]
    pub async fn create_subscription(
        &self,
        session: &mut ClientSession,
        name: String,
        items: u32,
        price: Decimal,
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
        self.store
            .insert(session, Subscription::new(name, items, price))
            .await?;
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
