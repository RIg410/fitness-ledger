use eyre::Error;
use model::{decimal::Decimal, session::Session, subscription::Subscription};
use storage::subscription::SubscriptionsStore;
use thiserror::Error;
use tx_macro::tx;

use std::ops::Deref;

use super::history::History;

#[derive(Clone)]
pub struct Subscriptions {
    pub store: SubscriptionsStore,
    pub logs: History,
}

impl Subscriptions {
    pub fn new(store: SubscriptionsStore, logs: History) -> Self {
        Subscriptions { store, logs }
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
}

impl Deref for Subscriptions {
    type Target = SubscriptionsStore;

    fn deref(&self) -> &Self::Target {
        &self.store
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
