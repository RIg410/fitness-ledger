use super::View;
use crate::{callback_data::Calldata as _, context::Context, state::Widget};
use async_trait::async_trait;
use eyre::Result;
use model::{rights::Rule, user::User};
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Message};

pub struct CouchingList {
    go_back: Option<Widget>,
}

impl CouchingList {
    pub fn new(go_back: Option<Widget>) -> CouchingList {
        CouchingList { go_back }
    }
}

#[async_trait]
impl View for CouchingList {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        ctx.ensure(Rule::CouchingView)?;
        let msg = "Инструкторы 🧘";
        let mut keymap = InlineKeyboardMarkup::default();
        let instructs = ctx.ledger.users.instructors(&mut ctx.session).await?;

        for instruct in instructs {
            keymap = keymap.append_row(vec![render_button(&instruct)]);
        }

        if ctx.has_right(Rule::CreateCouch) {
            keymap = keymap.append_row(Callback::MakeCouch.btn_row("Новый инструктор 🔥"));
        }

        if self.go_back.is_some() {
            keymap = keymap.append_row(Callback::Back.btn_row("Назад"));
        }
        ctx.edit_origin(msg, keymap).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Option<Widget>> {
        ctx.delete_msg(message.id).await?;
        Ok(None)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Option<Widget>> {
        Ok(None)
    }

    fn take(&mut self) -> Widget {
        CouchingList {
            go_back: self.go_back.take(),
        }
        .take()
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Back,
    SelectCouch([u8; 12]),
    MakeCouch,
}

fn render_button(user: &User) -> InlineKeyboardButton {
    Callback::SelectCouch(user.id.bytes()).button(format!(
        "💪 {} {}",
        user.name.first_name,
        user.name.last_name.clone().unwrap_or_default()
    ))
}
