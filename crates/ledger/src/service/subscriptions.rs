use eyre::{eyre, Error};
use model::{decimal::Decimal, session::Session, subscription::Subscription};
use mongodb::bson::oid::ObjectId;
use storage::subscription::SubscriptionsStore;
use thiserror::Error;
use tx_macro::tx;

use std::{ops::Deref, sync::Arc};

use super::{history::History, programs::Programs, users::Users};

pub struct Subscriptions {
    pub store: Arc<SubscriptionsStore>,
    pub logs: History,
    pub program: Programs,
    pub users: Users,
}

impl Subscriptions {
    pub fn new(
        store: Arc<SubscriptionsStore>,
        logs: History,
        program: Programs,
        users: Users,
    ) -> Self {
        Subscriptions {
            store,
            logs,
            program,
            users,
        }
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
        sub: Subscription,
    ) -> Result<(), CreateSubscriptionError> {
        if self.get_by_name(session, &sub.name).await?.is_some() {
            return Err(CreateSubscriptionError::NameAlreadyExists);
        }
        if sub.items == 0 {
            return Err(CreateSubscriptionError::InvalidItems);
        }

        if sub.price < Decimal::zero() {
            return Err(CreateSubscriptionError::InvalidPrice);
        }
        self.store.insert(session, sub).await?;
        Ok(())
    }

    #[tx]
    pub async fn edit_program_list(
        &self,
        session: &mut Session,
        sub: ObjectId,
        program_id: ObjectId,
        add: bool,
    ) -> Result<(), Error> {
        let mut subscription = self
            .get(session, sub)
            .await?
            .ok_or_else(|| eyre!("Subscription not found"))?;
        let _ = self
            .program
            .get_by_id(session, program_id)
            .await?
            .ok_or_else(|| eyre!("Program not found"))?;
        if let model::subscription::SubscriptionType::Group { program_filter } =
            &mut subscription.subscription_type
        {
            if add {
                if program_filter.contains(&program_id) {
                    return Ok(());
                } else {
                    program_filter.push(program_id);
                }
            } else {
                if program_filter.contains(&program_id) {
                    program_filter.retain(|&x| x != program_id);
                } else {
                    return Ok(());
                }
            }
            self.store.update(session, &subscription).await?;
        } else {
            return Err(eyre!("Only group subscriptions can have programs"));
        }

        let users_with_subscription = self.users.find_with_subscription(session, sub).await?;
        for mut user in users_with_subscription {
            let subs = user.subscriptions_mut();
            for user_sub in subs.iter_mut() {
                if user_sub.subscription_id == sub {
                    user_sub.tp = subscription.subscription_type.clone();
                }
            }
            self.users.store.update(session, &mut user).await?;
        }
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
