use crate::Task;
use async_trait::async_trait;
use bot_core::{bot::TgBot, CommonLocation};
use bot_viewer::fmt_phone;
use chrono::{Datelike as _, Local};
use eyre::Error;
use ledger::Ledger;
use log::info;
use model::{rights::Rule, user::User};
use std::sync::Arc;
use teloxide::{
    types::{ChatId, InlineKeyboardMarkup},
    utils::markdown::escape,
};

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
        let users = self
            .ledger
            .users
            .find_by_birthday(&mut session, now.day(), now.month())
            .await?;

        let notification_listener = self
            .ledger
            .users
            .find_users_with_right(&mut session, Rule::ReceiveNotificationsAboutBirthdays)
            .await?;

        for user in users {
            info!("Birthday notification for {}", user.id);
            self.notify(&user, &notification_listener).await?;
        }

        Ok(())
    }
}

impl BirthdaysNotifier {
    pub fn new(ledger: Arc<Ledger>, bot: Arc<TgBot>) -> BirthdaysNotifier {
        BirthdaysNotifier { ledger, bot }
    }

    async fn notify(&self, user: &User, notification_listener: &[User]) -> Result<(), Error> {
        let msg = format!(
            "Сегодня день рождения у {} {}",
            escape(&user.name.first_name),
            fmt_phone(user.phone.as_deref())
        );
        let keymap = InlineKeyboardMarkup::default()
            .append_row(vec![CommonLocation::Profile(user.id).button()]);

        for listener in notification_listener.iter() {
            self.bot
                .notify_with_markup(ChatId(listener.tg_id), &msg, keymap.clone())
                .await;
        }

        Ok(())
    }
}
