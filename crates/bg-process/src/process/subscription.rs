use crate::{Ledger, Task};
use async_trait::async_trait;
use eyre::{Error, Result};

#[derive(Clone)]
pub struct SubscriptionBg {
    ledger: Ledger,
}

#[async_trait]
impl Task for SubscriptionBg {
    const NAME: &'static str = "subscription";
    const CRON: &'static str = "every 1 hour";

    async fn process(&mut self) -> Result<(), Error> {
        let mut session = self.ledger.db.start_session().await?;

        let users = self
            .ledger
            .users
            .find_subscription_to_expire(&mut session)
            .await?;
        for user in users {
            self.ledger
                .users
                .expire_subscription(&mut session, user.id)
                .await?;
        }
        Ok(())
    }
}

impl SubscriptionBg {
    pub fn new(ledger: Ledger) -> SubscriptionBg {
        SubscriptionBg { ledger }
    }
}
