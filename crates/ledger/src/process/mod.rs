use crate::Ledger;
use eyre::{Context, Result};
use freeze::FreezeBg;
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
        self.training
            .process(session)
            .await
            .context("training_process")?;

        self.freeze
            .process(session)
            .await
            .context("freeze_process")?;

        self.subscriptions
            .process(session)
            .await
            .context("subscriptions_process")?;
        self.logs.process(session).await.context("log_gc_proc")?;
        Ok(())
    }
}
