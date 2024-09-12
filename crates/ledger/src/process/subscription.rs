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
        Ok(())
    }
}
