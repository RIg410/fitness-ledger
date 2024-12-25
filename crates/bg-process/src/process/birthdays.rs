use crate::Task;
use async_trait::async_trait;
use bot_core::bot::TgBot;
use bot_viewer::request::fmt_request;
use chrono::{Datelike as _, Local};
use eyre::Error;
use ledger::Ledger;
use model::{request::Request, session::Session};
use std::sync::Arc;
use teloxide::types::ChatId;
use tx_macro::tx;

#[derive(Clone)]
pub struct BirthdaysNotifier {
    pub ledger: Arc<Ledger>,
    pub bot: Arc<TgBot>,
}

#[async_trait]
impl Task for BirthdaysNotifier {
    const NAME: &'static str = "birthdays_notifier";
    const CRON: &'static str = "every 1 day at 8:00";

    async fn process(&mut self) -> Result<(), Error> {
        let mut session = self.ledger.db.start_session().await?;
        let now = Local::now();
        let _users = self
            .ledger
            .users
            .find_by_birthday(&mut session, now.day(), now.month())
            .await?;
            
        Ok(())
    }
}

impl BirthdaysNotifier {
    pub fn new(ledger: Arc<Ledger>, bot: Arc<TgBot>) -> BirthdaysNotifier {
        BirthdaysNotifier { ledger, bot }
    }

    #[tx]
    async fn notify(
        &self,
        ledger: &Ledger,
        session: &mut Session,
        user: i64,
        request: &mut Request,
    ) -> Result<(), Error> {
        let msg = format!("Напоминание по заявке\n{}", fmt_request(&request));
        let id = self.bot.send_notification_to(ChatId(user), &msg).await?;
        self.bot.pin_message(ChatId(user), id).await?;
        request.remind_later = None;
        Ok(())
    }
}
