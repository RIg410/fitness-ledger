use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
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
        if self.query.starts_with("8") {
            self.query = format!("7{}", &self.query[1..]);
        }
        self.offset = 0;
        Ok(Jmp::Stay)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        ctx.ensure(Rule::EditTrainingClientsList)?;

        match calldata!(data) {
            Callback::Next => {
                self.offset += LIMIT;
                Ok(Jmp::Stay)
            }
            Callback::Prev => {
                self.offset = self.offset.saturating_sub(LIMIT);
                Ok(Jmp::Stay)
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
    let mut users = ctx
        .ledger
        .users
        .find(&mut ctx.session, query, offset, LIMIT)
        .await?;

    let msg = format!(
        "Что бы найти пользователя, введите имя, фамилию или телефон пользователя\\.\n Запрос: _'{}'_",
        escape(query),
    );
    let mut keymap = InlineKeyboardMarkup::default();

    let mut ids = vec![];
    let mut users_count = 0;
    while let Some(user) = users.next(&mut ctx.session).await {
        let user = user?;
        users_count += 1;
        if user.couch.is_some() {
            continue;
        }
        if ids.contains(&user.id) {
            continue;
        }
        ids.push(user.id);

        keymap = keymap.append_row(vec![make_button(&user)]);
        for child_id in &user.family.children_ids {
            let child = ctx.ledger.get_user(&mut ctx.session, *child_id).await?;
            if child.phone.is_none() {
                if ids.contains(&child_id) {
                    continue;
                }
                ids.push(*child_id);
                keymap = keymap.append_row(vec![make_child_button(&user, &child)]);
            }
        }
    }

    let mut raw = vec![];

    if offset > 0 {
        raw.push(InlineKeyboardButton::callback(
            "⬅️",
            Callback::Prev.to_data(),
        ));
    }

    if users_count == LIMIT as usize {
        raw.push(InlineKeyboardButton::callback(
            "➡️",
            Callback::Next.to_data(),
        ));
    }

    if !raw.is_empty() {
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

fn make_child_button(parent: &User, child: &User) -> InlineKeyboardButton {
    InlineKeyboardButton::callback(
        format!(
            "{}{} {} ({})",
            fmt_user_type(child),
            parent.name.first_name,
            parent.name.last_name.as_ref().unwrap_or(&"".to_string()),
            child.name.first_name
        ),
        Callback::Select(child.id.bytes()).to_data(),
    )
}

#[derive(Debug, Serialize, Deserialize)]
enum Callback {
    Next,
    Prev,
    Select([u8; 12]),
}
