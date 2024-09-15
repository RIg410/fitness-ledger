use super::View;
use crate::{callback_data::Calldata as _, context::Context, state::Widget};
use async_trait::async_trait;
use eyre::Result;
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardMarkup, Message};

pub struct SetPhone {
    id: i64,
    go_back: Option<Widget>,
}

impl SetPhone {
    pub fn new(id: i64, go_back: Option<Widget>) -> SetPhone {
        SetPhone { id, go_back }
    }
}

#[async_trait]
impl View for SetPhone {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let mut keymap = InlineKeyboardMarkup::default();

        if self.go_back.is_some() {
            keymap = keymap.append_row(Callback::Back.btn_row("⬅️"));
        }
        ctx.edit_origin("Введите телефон", keymap).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Option<Widget>> {
        let text = message.text().unwrap_or_default();
        if text.is_empty() {
            ctx.send_err("Введите телефон").await?;
            return Ok(None);
        }

        ctx.ledger
            .users
            .set_phone(&mut ctx.session, self.id, text)
            .await?;
        ctx.delete_msg(message.id).await?;
        Ok(self.go_back.take())
    }

    async fn handle_callback(&mut self, _: &mut Context, _: &str) -> Result<Option<Widget>> {
        Ok(None)
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Back,
}
