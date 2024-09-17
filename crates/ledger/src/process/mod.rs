use crate::Ledger;
use eyre::{Context as _, Result};
use freeze::FreezeBg;
use log::{debug, error};
use logs::LogsBg;
use model::session::Session;
use subscription::SubscriptionBg;
use training::TriningBg;

pub mod freeze;
pub mod logs;
pub mod subscription;
pub mod training;

pub struct BgProcessor {
    pub ledger: Ledger,
    pub training: TriningBg,
    pub freeze: FreezeBg,
    pub subscriptions: SubscriptionBg,
    pub logs: LogsBg,
}

impl BgProcessor {
    pub fn new(ledger: Ledger) -> BgProcessor {
        BgProcessor {
            training: TriningBg::new(ledger.clone()),
            freeze: FreezeBg::new(ledger.clone()),
            subscriptions: SubscriptionBg::new(ledger.clone()),
            logs: LogsBg::new(ledger.clone()),
            ledger,
        }
    }

    pub async fn process(&self, session: &mut Session) -> Result<()> {
        debug!("training process");
        let result = self
            .training
            .process(session)
            .await
            .context("training_process");
        if let Err(err) = result {
            error!("Failed to training proc error:{:#?}", err);
        }

        debug!("freeze process");
        let result = self.freeze.process(session).await.context("freeze_process");
        if let Err(err) = result {
            error!("Failed to training proc error:{:#?}", err);
        }

        debug!("subscriptions process");
        let result = self
            .subscriptions
            .process(session)
            .await
            .context("subscriptions_process");
        if let Err(err) = result {
            error!("Failed to training proc error:{:#?}", err);
        }

        debug!("logs process");
        let result = self.logs.process(session).await.context("log_gc_proc");
        if let Err(err) = result {
            error!("Failed to training proc error:{:#?}", err);
        }
        Ok(())
    }
}
