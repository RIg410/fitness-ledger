use super::{render_msg, RentPreset};
use async_trait::async_trait;
use bot_core::{
    context::Context,
    widget::{Jmp, View},
};
use eyre::Result;
use teloxide::types::{InlineKeyboardMarkup, Message};

#[derive(Default)]
pub struct SetRenter {
    preset: RentPreset,
}

impl SetRenter {
    pub fn new(preset: RentPreset) -> Self {
        Self { preset }
    }
}

#[async_trait]
impl View for SetRenter {
    fn name(&self) -> &'static str {
        "SetRenter"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let msg = render_msg(ctx, &self.preset, "Введите название арендатора").await?;
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

        self.preset.renter = Some(msg.to_string());
        Ok(self.preset.clone().into_next_view().into())
    }
}
