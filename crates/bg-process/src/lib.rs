use std::sync::Arc;

use bot_core::bot::{Origin, TgBot, ValidToken};
use bot_main::BotApp;
use eyre::{Context, Error};
use ledger::Ledger;
use log::info;
use process::{
    freeze::FreezeBg, notifier::TrainingNotifier, requests::RequestNotifier, rewards::RewardsBg,
    subscription::SubscriptionBg, training::TriningBg, user_sync::UserNameSync,
};

use teloxide::types::{ChatId, MessageId};
use tokio_cron_scheduler::{Job, JobScheduler};
mod process;

pub async fn start(ledger: Arc<Ledger>, bot: BotApp) -> Result<(), Error> {
    let bot = Arc::new(TgBot::new(
        bot.bot,
        bot.state.tokens(),
        Origin {
            chat_id: ChatId(0),
            message_id: MessageId(0),
            tkn: ValidToken::new(),
        },
        bot.env.clone(),
    ));
    let sched = JobScheduler::new().await?;

    sched.add(TriningBg::new(ledger.clone()).to_job()?).await?;
    sched.add(FreezeBg::new(ledger.clone()).to_job()?).await?;
    sched
        .add(SubscriptionBg::new(ledger.clone(), bot.clone()).to_job()?)
        .await?;
    sched.add(RewardsBg::new(ledger.clone()).to_job()?).await?;
    sched
        .add(TrainingNotifier::new(ledger.clone(), bot.clone()).to_job()?)
        .await?;
    sched
        .add(RequestNotifier::new(ledger.clone(), bot.clone()).to_job()?)
        .await?;
    sched
        .add(UserNameSync::new(ledger.clone(), bot).to_job()?)
        .await?;
    sched.start().await?;
    Ok(())
}

#[async_trait::async_trait]
pub trait Task {
    const NAME: &'static str;
    const CRON: &'static str;
    async fn process(&mut self) -> Result<(), Error>;
}

#[async_trait::async_trait]
trait CronJob {
    async fn call(&mut self);
    fn to_job(self) -> Result<Job, Error>;
}

#[async_trait::async_trait]
impl<T: Task + Send + Sync + Clone + 'static> CronJob for T {
    async fn call(&mut self) {
        info!("Starting background {} process", Self::NAME);
        if let Err(err) = self.process().await {
            log::error!("Error in background {} process: {:#}", Self::NAME, err);
        }
    }

    fn to_job(self) -> Result<Job, Error> {
        info!("Creating job for {}. CRON: {}", Self::NAME, Self::CRON);
        Job::new_async(Self::CRON, move |_, _| {
            let mut task = self.clone();
            Box::pin(async move {
                task.call().await;
            })
        })
        .context(format!(
            "Failed to create job for {}. CRON: {}",
            Self::NAME,
            Self::CRON
        ))
    }
}
