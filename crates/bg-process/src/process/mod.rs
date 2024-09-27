use crate::Ledger;
use eyre::{Context as _, Result};
use freeze::FreezeBg;
use log::error;
use model::session::Session;
use rewards::RewardsBg;
use subscription::SubscriptionBg;
use training::TriningBg;

pub mod freeze;
pub mod rewards;
pub mod subscription;
pub mod training;

pub struct BgProcessor {
    pub ledger: Ledger,
    pub training: TriningBg,
    pub freeze: FreezeBg,
    pub subscriptions: SubscriptionBg,
    pub rewards: RewardsBg,
}

impl BgProcessor {
    pub fn new(ledger: Ledger) -> BgProcessor {
        BgProcessor {
            training: TriningBg::new(ledger.clone()),
            freeze: FreezeBg::new(ledger.clone()),
            subscriptions: SubscriptionBg::new(ledger.clone()),
            rewards: RewardsBg::new(ledger.clone()),
            ledger,
        }
    }

    pub async fn process(&self, session: &mut Session) -> Result<()> {
        let result = self
            .training
            .process(session)
            .await
            .context("training_process");
        if let Err(err) = result {
            error!("Failed to training proc error:{:#?}", err);
        }

        let result = self.freeze.process(session).await.context("freeze_process");
        if let Err(err) = result {
            error!("Failed to training proc error:{:#?}", err);
        }

        let result = self
            .subscriptions
            .process(session)
            .await
            .context("subscriptions_process");
        if let Err(err) = result {
            error!("Failed to training proc error:{:#?}", err);
        }

        let result = self
            .rewards
            .process(session)
            .await
            .context("rewards_process");
        if let Err(err) = result {
            error!("Failed to training proc error:{:#?}", err);
        }

        Ok(())
    }
}
