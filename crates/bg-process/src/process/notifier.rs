use crate::Task;
use async_trait::async_trait;
use bot_core::bot::{Origin, TgBot, ValidToken};
use bot_main::BotApp;
use bot_viewer::day::fmt_time;
use chrono::Local;
use eyre::Error;
use ledger::Ledger;
use model::{ids::DayId, session::Session, training::Notified, user::UserIdent};
use std::sync::Arc;
use teloxide::{
    types::{ChatId, MessageId},
    utils::markdown::escape,
};

#[derive(Clone)]
pub struct TrainingNotifier {
    pub ledger: Ledger,
    pub bot: Arc<TgBot>,
}

#[async_trait]
impl Task for TrainingNotifier {
    const NAME: &'static str = "notifier";
    const CRON: &'static str = "every 30 minutes";

    async fn process(&mut self) -> Result<(), Error> {
        let mut session = self.ledger.db.start_session().await?;
        self.notify_about_tomorrow_training(&mut session).await?;
        self.notify_about_today_training(&mut session).await?;
        Ok(())
    }
}

impl TrainingNotifier {
    pub fn new(ledger: Ledger, bot: BotApp) -> TrainingNotifier {
        TrainingNotifier {
            ledger,
            bot: Arc::new(TgBot::new(
                bot.bot,
                bot.state.tokens(),
                Origin {
                    chat_id: ChatId(0),
                    message_id: MessageId(0),
                    tkn: ValidToken::new(),
                },
            )),
        }
    }

    async fn notify_user<ID: Into<UserIdent>>(
        &self,
        session: &mut Session,
        id: ID,
        msg: &str,
        by_day: bool,
    ) -> Result<bool, Error> {
        if let Ok(user) = self.ledger.get_user(session, id).await {
            if by_day {
                if user.settings.notification.notify_by_day {
                    self.bot
                        .send_notification_to(ChatId(user.tg_id), &msg)
                        .await?;
                    return Ok(true);
                }
            } else {
                let now = Local::now();
                if let Some(hours) = user.settings.notification.notify_by_n_hours {
                    if now + chrono::Duration::hours(hours as i64) > now {
                        self.bot
                            .send_notification_to(ChatId(user.tg_id), &msg)
                            .await?;
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    async fn notify_about_tomorrow_training(&self, session: &mut Session) -> Result<(), Error> {
        let tomorrow = Local::now() + chrono::Duration::days(1);
        let day = self
            .ledger
            .calendar
            .get_day(session, DayId::default().next())
            .await?;

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

            for client in &training.clients {
                self.notify_user(session, *client, &msg, true).await?;
            }

            self.ledger
                .calendar
                .notify(session, training.start_at, Notified::Tomorrow {})
                .await?;
        }
        Ok(())
    }

    async fn notify_about_today_training(&self, session: &mut Session) -> Result<(), Error> {
        let now = Local::now();
        let day = self
            .ledger
            .calendar
            .get_day(session, DayId::default())
            .await?;

        for training in day.training {
            if training.is_canceled || training.is_processed {
                continue;
            }
            let start_at = training.get_slot().start_at();
            if start_at < now {
                continue;
            }

            let mut already_notified = match training.notified {
                Notified::None {} => {
                    vec![]
                }
                Notified::Tomorrow {} => {
                    vec![]
                }
                Notified::ByHours(ids) => ids,
            };

            let msg = escape(&format!(
                "У вас запланирована тренировка: {} в {}",
                training.name,
                fmt_time(&start_at)
            ));

            let mut has_changes = false;
            for client in &training.clients {
                if !already_notified.contains(client) {
                    if self.notify_user(session, *client, &msg, false).await? {
                        already_notified.push(*client);
                        has_changes = true;
                    }
                }
            }

            if has_changes {
                self.ledger
                    .calendar
                    .notify(
                        session,
                        training.start_at,
                        Notified::ByHours(already_notified),
                    )
                    .await?;
            }
        }
        Ok(())
    }
}
