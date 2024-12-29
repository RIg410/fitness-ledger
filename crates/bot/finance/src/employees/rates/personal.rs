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

pub struct PersonalRate {
    old_rate: Option<Rate>,
    user_id: ObjectId,
}

impl PersonalRate {
    pub fn new(old_rate: Option<Rate>, user_id: ObjectId) -> PersonalRate {
        PersonalRate { old_rate, user_id }
    }
}

#[async_trait]
impl View for PersonalRate {
    fn name(&self) -> &'static str {
        "PersonalRate"
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
                        Rate::PersonalTraining {
                            percent: percent / Decimal::from(100),
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
