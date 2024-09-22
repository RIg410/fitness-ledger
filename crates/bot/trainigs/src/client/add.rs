use async_trait::async_trait;
use bot_core::{callback_data::Calldata as _, calldata, context::Context, widget::{Jmp, View}};
use bot_viewer::user::fmt_user_type;
use chrono::{DateTime, Local};
use eyre::{Error, Result};
use model::{rights::Rule, user::User};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

use crate::client::{ClientView, Reason};

pub const LIMIT: u64 = 7;

pub struct AddClientView {
    training_id: DateTime<Local>,
    query: String,
    offset: u64,
}

impl AddClientView {
    pub fn new(training_id: DateTime<Local>) -> AddClientView {
        AddClientView {
            training_id,
            query: "".to_string(),
            offset: 0,
        }
    }
}

#[async_trait]
impl View for AddClientView {
    fn name(&self) -> &'static str {
        "AddClientView"
    }
    
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let (text, keymap) = render(ctx, &self.query, self.offset).await?;
        ctx.edit_origin(&text, keymap).await?;
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Context, msg: &Message) -> Result<Jmp> {
        ctx.delete_msg(msg.id).await?;

        let mut query = msg.text().to_owned().unwrap_or_default().to_string();
        if query.len() == 1 && !query.chars().next().unwrap().is_alphanumeric() {
            query = "".to_string();
        }

        self.query = remove_non_alphanumeric(&query);
        self.offset = 0;
        Ok(Jmp::None)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        ctx.ensure(Rule::EditTrainingClientsList)?;

        match calldata!(data) {
            Callback::Next => {
                self.offset += LIMIT;
                Ok(Jmp::None)
            }
            Callback::Prev => {
                self.offset = self.offset.saturating_sub(LIMIT);
                Ok(Jmp::None)
            }
            Callback::Select(user_id) => {
                let id = ObjectId::from_bytes(user_id);
                Ok(ClientView::new(id, self.training_id, Reason::AddClient).into())
            }
        }
    }
}

async fn render(
    ctx: &mut Context,
    query: &str,
    offset: u64,
) -> Result<(String, InlineKeyboardMarkup), Error> {
    let users = ctx
        .ledger
        .users
        .find(&mut ctx.session, &query, offset, LIMIT)
        .await?;

    let msg = format!(
        "Что бы найти пользователя, введите имя, фамилию или телефон пользователя\\.\n Запрос: _'{}'_",
        escape(query),
    );
    let mut keymap = InlineKeyboardMarkup::default();

    for user in &users {
        if user.couch.is_some() {
            continue;
        }
        keymap = keymap.append_row(vec![make_button(user)]);
    }

    let mut raw = vec![];

    if offset > 0 {
        raw.push(InlineKeyboardButton::callback(
            "⬅️",
            Callback::Prev.to_data(),
        ));
    }

    if users.len() == LIMIT as usize {
        raw.push(InlineKeyboardButton::callback(
            "➡️",
            Callback::Next.to_data(),
        ));
    }

    if raw.len() > 0 {
        keymap = keymap.append_row(raw);
    }
    Ok((msg, keymap))
}

fn remove_non_alphanumeric(input: &str) -> String {
    input.chars().filter(|c| c.is_alphanumeric()).collect()
}

fn make_button(user: &User) -> InlineKeyboardButton {
    InlineKeyboardButton::callback(
        format!(
            "{}{} {}",
            fmt_user_type(user),
            user.name.first_name,
            user.name.last_name.as_ref().unwrap_or(&"".to_string())
        ),
        Callback::Select(user.id.bytes()).to_data(),
    )
}

#[derive(Debug, Serialize, Deserialize)]
enum Callback {
    Next,
    Prev,
    Select([u8; 12]),
}
