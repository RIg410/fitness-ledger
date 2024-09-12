use crate::Ledger;
use eyre::Result;
use mongodb::ClientSession;

pub struct SubscriptionBg {
    ledger: Ledger,
}

impl SubscriptionBg {
    pub fn new(ledger: Ledger) -> SubscriptionBg {
        SubscriptionBg { ledger }
    }

    pub async fn process(&self, session: &mut ClientSession) -> Result<()> {
        let users = self
            .ledger
            .users
            .find_subscription_to_expire(session)
            .await?;
        for user in users {
            self.ledger
                .users
                .expire_subscription(session, user.tg_id)
                .await?;
        }
        Ok(())
    }
}