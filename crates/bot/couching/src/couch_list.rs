use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use eyre::Result;
use model::{rights::Rule, user::User};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use super::{couch_info::couch_view, make_couch::make_make_couch_view};

pub struct CouchingList {}

impl CouchingList {
    pub fn new() -> CouchingList {
        CouchingList {}
    }
}

#[async_trait]
impl View for CouchingList {
    fn name(&self) -> &'static str {
        "CouchingList"
    }
    
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

        ctx.edit_origin(msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, _: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::SelectCouch(id) => {
                let id: ObjectId = ObjectId::from_bytes(id);
                return Ok(Jmp::Next(couch_view(id)));
            }
            Callback::MakeCouch => return Ok(Jmp::Next(make_make_couch_view())),
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
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
