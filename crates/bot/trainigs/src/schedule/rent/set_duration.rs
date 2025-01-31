use super::{render_msg, RentPreset};
use async_trait::async_trait;
use bot_core::{
    context::Context,
    widget::{Jmp, View},
};
use chrono::Duration;
use eyre::Result;
use teloxide::types::{InlineKeyboardMarkup, Message};

#[derive(Default)]
pub struct SetDuration {
    preset: RentPreset,
}

impl SetDuration {
    pub fn new(preset: RentPreset) -> Self {
        Self { preset }
    }
}

#[async_trait]
impl View for SetDuration {
    fn name(&self) -> &'static str {
        "SetDuration"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let msg = render_msg(ctx, &self.preset, "Введите продолжительность в минутах").await?;
        ctx.edit_origin(&msg, InlineKeyboardMarkup::default())
            .await?;

        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Context, message: &Message) -> Result<Jmp> {
        ctx.delete_msg(message.id).await?;
        let msg = if let Some(msg) = message.text() {
            msg
        } else {
            return Ok(Jmp::Stay);
        };

        let duration = match msg.parse::<u32>() {
            Ok(duration) => Duration::minutes(i64::from(duration)),
            Err(_) => {
                ctx.send_notification("Неверный формат продолжительности\\.")
                    .await;
                return Ok(Jmp::Stay);
            }
        };
        self.preset.duration = Some(duration);
        Ok(self.preset.clone().into_next_view().into())
    }
}
