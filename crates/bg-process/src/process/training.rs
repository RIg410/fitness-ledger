use std::sync::Arc;

use crate::{Ledger, Task};
use async_trait::async_trait;
use bot_core::{bot::TgBot, CommonLocation};
use bot_viewer::{fmt_phone, user::link_to_user};
use eyre::{bail, eyre, Error, Result};
use log::{error, info};
use model::{
    program::TrainingType,
    rights::Rule,
    session::Session,
    training::{Statistics, Training, TrainingStatus},
    user::{employee::UserRewardContribution, family::FindFor, User},
};
use teloxide::types::{ChatId, InlineKeyboardMarkup};
use tx_macro::tx;

#[derive(Clone)]
pub struct TriningBg {
    ledger: Arc<Ledger>,
    bot: Arc<TgBot>,
}

#[async_trait]
impl Task for TriningBg {
    const NAME: &'static str = "training";
    const CRON: &'static str = "every 3 minutes";

    async fn process(&mut self) -> Result<(), Error> {
        let mut session = self.ledger.db.start_session().await?;

        let mut cursor = self.ledger.calendar.days_to_process(&mut session).await?;
        let now = chrono::Local::now();
        while let Some(day) = cursor.next(&mut session).await {
            let day = day?;
            for training in day.training {
                if training.is_processed {
                    continue;
                }

                let result = match training.status(now) {
                    TrainingStatus::OpenToSignup { .. }
                    | TrainingStatus::ClosedToSignup
                    | TrainingStatus::InProgress => continue,
                    TrainingStatus::Finished => match training.tp {
                        TrainingType::Group { .. } | TrainingType::Personal { .. } => {
                            let notifications = self
                                .process_finished_training(&mut session, training)
                                .await?;
                            for notification in notifications {
                                self.send_notification(&mut session, notification).await?;
                            }
                            Ok(())
                        }
                        TrainingType::SubRent { .. } => {
                            self.process_finished_sub_rent(&mut session, training).await
                        }
                    },
                    TrainingStatus::Cancelled => {
                        if training.get_slot().start_at() < now {
                            self.process_canceled(&mut session, training).await
                        } else {
                            continue;
                        }
                    }
                };
                if let Err(err) = result {
                    error!("Failed to finalize: training:{:#}. Training", err);
                }
            }
        }
        Ok(())
    }
}

impl TriningBg {
    pub fn new(ledger: Arc<Ledger>, bot: Arc<TgBot>) -> TriningBg {
        TriningBg { ledger, bot }
    }

    #[tx]
    async fn process_finished_sub_rent(
        &self,
        session: &mut Session,
        training: Training,
    ) -> Result<()> {
        info!("Finalize sub rent:{:?}", training);
        let mut statistic = Statistics::default();
        let (is_free, price) = match training.tp {
            TrainingType::SubRent { is_free, price } => (is_free, price),
            _ => bail!("Invalid training type"),
        };

        self.ledger
            .calendar
            .finalized(session, training.id(), &statistic)
            .await?;
        self.ledger
            .history
            .process_finished(session, &training)
            .await?;

        if !is_free {
            statistic.earned += price;
            self.ledger
                .treasury
                .sub_rent_txless(session, price, training.description)
                .await?;
        }

        Ok(())
    }

    #[tx]
    async fn process_finished_training(
        &self,
        session: &mut Session,
        training: Training,
    ) -> Result<Vec<Notification>> {
        info!("Finalize training:{:?}", training);

        let mut notifications = vec![];

        let mut statistic = Statistics::default();

        let mut users_info = Vec::with_capacity(training.clients.len());
        if training.tp.is_not_free() {
            for client in &training.clients {
                let mut user = self.ledger.get_user(session, *client).await?;
                let mut payer = user.payer_mut()?;
                if let Some(sub) = payer.find_subscription(FindFor::Charge, &training) {
                    if !sub.change_locked_balance(&training) {
                        return Err(eyre!("Not enough balance:{}", user.id));
                    }
                    statistic.earned += sub.item_price();
                    users_info.push(UserRewardContribution {
                        user: *client,
                        lesson_price: sub.item_price(),
                        subscription_price: sub.subscription_price(),
                        lessons_count: sub.items(),
                    });
                } else {
                    return Err(eyre!("Subscription not found for user:{}", user.id));
                }
                self.ledger.users.update(session, &mut payer).await?;
                if let Some(notification) = self.user_notification(&user, &training)? {
                    notifications.push(notification);
                }
            }
            let mut couch = self.ledger.get_user(session, training.instructor).await?;
            if let Some(couch_info) = couch.employee.as_mut() {
                if let Some(reward) = couch_info.collect_training_rewards(&training, users_info)? {
                    statistic.couch_rewards += reward.reward;
                    self.ledger.rewards.add_reward(session, reward).await?;
                    self.ledger
                        .users
                        .update_employee_reward_and_rates(
                            session,
                            training.instructor,
                            couch_info.reward,
                            None,
                        )
                        .await?;
                }
            } else {
                bail!("Failed to process training. Failed to find instructor");
            }
        }
        self.ledger
            .calendar
            .finalized(session, training.id(), &statistic)
            .await?;
        self.ledger
            .history
            .process_finished(session, &training)
            .await?;
        Ok(notifications)
    }

    #[tx]
    async fn process_canceled(&self, session: &mut Session, training: Training) -> Result<()> {
        info!("Finalize canceled training:{:?}", training);

        self.ledger
            .calendar
            .finalized(session, training.id(), &Statistics::default())
            .await?;
        self.ledger
            .history
            .process_canceled(session, &training)
            .await?;
        Ok(())
    }

    fn user_notification(&self, user: &User, training: &Training) -> Result<Option<Notification>> {
        let payer = user.payer()?;
        let balance = payer.available_balance_for_training(&training);
        if balance == 0 {
            Ok(Some(Notification {
                to_user: (
                    "Ваш абонемент закончился🥺".to_string(),
                    ChatId(payer.as_ref().tg_id),
                ),
                to_manager: (
                    format!(
                        "У {} {} заканчивается абонемент\\.",
                        link_to_user(payer.as_ref()),
                        fmt_phone(payer.as_ref().phone.as_deref())
                    ),
                    InlineKeyboardMarkup::default()
                        .append_row(vec![CommonLocation::Profile(payer.as_ref().id).button()]),
                ),
            }))
        } else {
            Ok(None)
        }
    }

    async fn send_notification(
        &self,
        session: &mut Session,
        notification: Notification,
    ) -> Result<()> {
        self.bot
            .notify(notification.to_user.1, &notification.to_user.0, false)
            .await;

        let users = self
            .ledger
            .users
            .find_users_with_right(session, Rule::ReceiveNotificationsAboutSubscriptions)
            .await?;

        for user in users {
            self.bot
                .notify_with_markup(
                    ChatId(user.tg_id),
                    &notification.to_manager.0,
                    notification.to_manager.1.clone(),
                )
                .await;
        }

        Ok(())
    }
}

struct Notification {
    to_user: (String, ChatId),
    to_manager: (String, InlineKeyboardMarkup),
}
