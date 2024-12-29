use async_trait::async_trait;
use bot_core::{
    context::Context,
    widget::{Jmp, View},
};
use eyre::Result;
use model::decimal::Decimal;
use model::user::rate::Rate;
use mongodb::bson::oid::ObjectId;
use teloxide::types::{InlineKeyboardMarkup, Message};

use super::new::ConfirmCreationRate;

pub struct GroupRateMin {
    old_rate: Option<Rate>,
    user_id: ObjectId,
}

impl GroupRateMin {
    pub fn new(old_rate: Option<Rate>, user_id: ObjectId) -> GroupRateMin {
        GroupRateMin { old_rate, user_id }
    }
}

#[async_trait]
impl View for GroupRateMin {
    fn name(&self) -> &'static str {
        "GroupRate"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let msg = "Введите минимальное вознаграждение:";
        let keymap = InlineKeyboardMarkup::default();
        ctx.edit_origin(msg, keymap).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        msg: &Message,
    ) -> Result<Jmp, eyre::Error> {
        ctx.delete_msg(msg.id).await?;
        if let Some(text) = msg.text() {
            if let Ok(amount) = text.parse::<Decimal>() {
                Ok(Jmp::Next(
                    GroupRatePercent::new(self.old_rate, self.user_id, amount).into(),
                ))
            } else {
                ctx.send_notification("Неверный формат суммы").await?;
                Ok(Jmp::Stay)
            }
        } else {
            Ok(Jmp::Stay)
        }
    }
}

pub struct GroupRatePercent {
    min_amount: Decimal,
    old_rate: Option<Rate>,
    user_id: ObjectId,
}

impl GroupRatePercent {
    pub fn new(old_rate: Option<Rate>, user_id: ObjectId, min_amount: Decimal) -> GroupRatePercent {
        GroupRatePercent {
            old_rate,
            user_id,
            min_amount,
        }
    }
}

#[async_trait]
impl View for GroupRatePercent {
    fn name(&self) -> &'static str {
        "GroupRatePercent"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let msg = "Введите процент вознаграждения:";
        let keymap = InlineKeyboardMarkup::default();
        ctx.edit_origin(msg, keymap).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        msg: &Message,
    ) -> Result<Jmp, eyre::Error> {
        ctx.delete_msg(msg.id).await?;
        if let Some(text) = msg.text() {
            if let Ok(percent) = text.parse::<Decimal>() {
                if percent < Decimal::int(0) || percent > Decimal::int(100) {
                    ctx.send_notification("Процент должен быть от 0 до 100")
                        .await?;
                    return Ok(Jmp::Stay);
                }

                Ok(Jmp::Next(
                    ConfirmCreationRate::new(
                        self.old_rate,
                        Rate::GroupTraining {
                            percent: percent / Decimal::from(100),
                            min_reward: self.min_amount,
                        },
                        self.user_id,
                    )
                    .into(),
                ))
            } else {
                ctx.send_notification("Неверный формат процента от дохода")
                    .await?;
                Ok(Jmp::Stay)
            }
        } else {
            Ok(Jmp::Stay)
        }
    }
}
