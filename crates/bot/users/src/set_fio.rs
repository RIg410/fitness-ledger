use super::View;
use async_trait::async_trait;
use bot_core::{context::Context, widget::Goto};
use eyre::Result;
use teloxide::types::{InlineKeyboardMarkup, Message};

pub struct SetFio {
    id: i64,
}

impl SetFio {
    pub fn new(id: i64) -> SetFio {
        SetFio { id }
    }
}

#[async_trait]
impl View for SetFio {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        ctx.edit_origin("Введите имя и фамилию", InlineKeyboardMarkup::default())
            .await?;
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Context, message: &Message) -> Result<Goto> {
        let parts = message
            .text()
            .unwrap_or_default()
            .split(" ")
            .collect::<Vec<_>>();
        if parts.len() != 2 {
            ctx.send_notification("Введите имя и фамилию").await?;
            return Ok(Goto::None);
        }

        let name = parts[0];
        let last_name = parts[1];
        ctx.ledger
            .users
            .set_name(&mut ctx.session, self.id, name, last_name)
            .await?;
        ctx.delete_msg(message.id).await?;
        Ok(Goto::Back)
    }
}
