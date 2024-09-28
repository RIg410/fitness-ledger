use std::{collections::HashMap, hash::Hash};

use bot_core::bot::{Origin, TgBot, ValidToken};
use bot_main::BotApp;
use chrono::Local;
use eyre::Error;
use ledger::Ledger;
use model::{
    ids::DayId,
    session::Session,
    training::{Notified, Training},
    user::UserIdent,
};
use teloxide::{
    types::{ChatId, MessageId},
    utils::markdown::escape,
};

pub struct Notifier {
    pub ledger: Ledger,
    pub bot: TgBot,
}

impl Notifier {
    pub fn new(ledger: Ledger, bot: BotApp) -> Notifier {
        Notifier {
            ledger,
            bot: TgBot::new(
                bot.bot,
                bot.state.tokens(),
                Origin {
                    chat_id: ChatId(0),
                    message_id: MessageId(0),
                    tkn: ValidToken::new(),
                },
            ),
        }
    }

    async fn notify_user<ID: Into<UserIdent>>(
        &self,
        session: &mut Session,
        id: ID,
        msg: &str,
        resent: &[i32],
    ) -> Result<Option<(i64, i32)>, Error> {
        // Ok(if let Ok(user) = self.ledger.get_user(session, id).await {
        //     let id = self
        //         .bot
        //         .send_notification_to(ChatId(user.tg_id), &msg, resent)
        //         .await?;
        //     Some((user.tg_id, id.0))
        // } else {
        //     None
        // })
        Ok(None)
    }

    async fn notify_training(
        &self,
        session: &mut Session,
        training: &Training,
        msg: String,
        reasent_notifications: &HashMap<i64, Vec<i32>>,
    ) -> Result<Vec<(i64, i32)>, Error> {
        let mut ids = vec![];
        if let Ok(user) = self.ledger.get_user(session, training.instructor).await {
            let id = self
                .bot
                .send_notification_to(ChatId(user.tg_id), &msg)
                .await?;
            ids.push((user.tg_id, id.0));
        }

        for client in &training.clients {
            if let Ok(user) = self.ledger.get_user(session, *client).await {
                let id = self
                    .bot
                    .send_notification_to(ChatId(user.tg_id), &msg)
                    .await?;
                ids.push((user.tg_id, id.0));
            }
        }

        Ok(ids)
    }

    pub async fn process(&self, session: &mut Session) -> Result<(), Error> {
        // self.notify_about_tomorrow_training(session).await?;
        // self.notify_about_today_training(session).await?;

        Ok(())
    }

    async fn notify_about_tomorrow_training(&self, session: &mut Session) -> Result<(), Error> {
        let tomorrow = Local::now() + chrono::Duration::days(1);
        let day = self
            .ledger
            .calendar
            .get_day(session, DayId::default().next())
            .await?;

        let notification_map = day.notification_map();
        for training in day.training {
            if training.is_canceled || training.is_processed || training.notified.is_notified() {
                continue;
            }

            if training.get_slot().start_at() > tomorrow {
                continue;
            }

            let msg = escape(&format!(
                "Завтра в {} у вас тренировка: {}",
                training.get_slot().start_at().format("%H:%M"),
                training.name
            ));

            let ids = self
                .notify_training(session, &training, msg, &notification_map)
                .await?;

            self.ledger
                .calendar
                .notify(session, training.start_at, Notified::Tomorrow(ids))
                .await?;
        }
        Ok(())
    }
}
