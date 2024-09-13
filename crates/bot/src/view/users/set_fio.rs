use super::View;
use crate::{callback_data::Calldata as _, context::Context, state::Widget};
use async_trait::async_trait;
use eyre::Result;
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardMarkup, Message};

pub struct SetFio {
    id: i64,
    go_back: Option<Widget>,
}

impl SetFio {
    pub fn new(id: i64, go_back: Option<Widget>) -> SetFio {
        SetFio { id, go_back }
    }
}

#[async_trait]
impl View for SetFio {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let mut keymap = InlineKeyboardMarkup::default();

        if self.go_back.is_some() {
            keymap = keymap.append_row(Callback::Back.btn_row("⬅️"));
        }
        ctx.edit_origin("Введите имя и фамилию", keymap).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Option<Widget>> {
        let parts = message
            .text()
            .unwrap_or_default()
            .split(" ")
            .collect::<Vec<_>>();
        if parts.len() != 2 {
            ctx.send_err("Введите имя и фамилию").await?;
            return Ok(None);
        }

        let name = parts[0];
        let last_name = parts[1];
        ctx.ledger
            .users
            .set_name(&mut ctx.session, self.id, name, last_name)
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
