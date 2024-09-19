use crate::{callback_data::Calldata as _, context::Context, state::Widget, view::View};
use async_trait::async_trait;
use eyre::Result;
use model::{rights::Rule, user::User};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Message};

use super::{couch_info::couch_view, make_couch::make_make_couch_view};

pub struct CouchingList {
    go_back: Option<Widget>,
}

impl CouchingList {
    pub fn new() -> CouchingList {
        CouchingList { go_back: None }
    }
}

#[async_trait]
impl View for CouchingList {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let msg = "Наши инструкторы ❤️";
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

    async fn handle_callback(&mut self, _: &mut Context, data: &str) -> Result<Option<Widget>> {
        let cb = if let Some(cb) = Callback::from_data(data) {
            cb
        } else {
            return Ok(None);
        };
        match cb {
            Callback::Back => {
                return Ok(self.go_back.take());
            }
            Callback::SelectCouch(id) => {
                let id = ObjectId::from_bytes(id);
                return Ok(Some(couch_view(self.take(), id)));
            }
            Callback::MakeCouch => return Ok(Some(make_make_couch_view(self.take()))),
        }
    }

    fn take(&mut self) -> Widget {
        CouchingList {
            go_back: self.go_back.take(),
        }
        .boxed()
    }

    fn set_back(&mut self, back: Widget) {
        self.go_back = Some(back);
    }

    fn back(&mut self) -> Option<Widget> {
        self.go_back.take()
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
