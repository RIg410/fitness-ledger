use super::{
    client::{ClientView, Reason},
    View,
};
use crate::{
    callback_data::Calldata as _, context::Context, state::Widget, view::users::profile::user_type,
};
use async_trait::async_trait;
use chrono::{DateTime, Local};
use eyre::{Error, Result};
use model::{rights::Rule, user::User};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

pub const LIMIT: u64 = 7;

pub struct AddClientView {
    go_back: Option<Widget>,
    training_id: DateTime<Local>,
    query: String,
    offset: u64,
}

impl AddClientView {
    pub fn new(training_id: DateTime<Local>) -> AddClientView {
        AddClientView {
            go_back: None,
            training_id,
            query: "".to_string(),
            offset: 0,
        }
    }
}

#[async_trait]
impl View for AddClientView {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let (text, keymap) = render(ctx, &self.query, self.offset).await?;
        ctx.edit_origin(&text, keymap).await?;
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Context, msg: &Message) -> Result<Option<Widget>> {
        ctx.delete_msg(msg.id).await?;

        let mut query = msg.text().to_owned().unwrap_or_default().to_string();
        if query.len() == 1 && !query.chars().next().unwrap().is_alphanumeric() {
            query = "".to_string();
        }

        self.query = remove_non_alphanumeric(&query);
        self.offset = 0;
        self.show(ctx).await?;
        Ok(None)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Option<Widget>> {
        ctx.ensure(Rule::EditTrainingClientsList)?;
        let cb = if let Some(cb) = Callback::from_data(data) {
            cb
        } else {
            return Ok(None);
        };
        match cb {
            Callback::Next => {
                self.offset += LIMIT;
                self.show(ctx).await?;
                Ok(None)
            }
            Callback::Prev => {
                self.offset = self.offset.saturating_sub(LIMIT);
                self.show(ctx).await?;
                Ok(None)
            }
            Callback::Select(user_id) => {
                let back = AddClientView {
                    go_back: self.go_back.take(),
                    query: self.query.clone(),
                    offset: self.offset,
                    training_id: self.training_id,
                }
                .boxed();
                let id = ObjectId::from_bytes(user_id);
                Ok(Some(
                    ClientView::new(id, self.training_id, Reason::AddClient).boxed(),
                ))
            }
            Callback::Back => {
                if let Some(back) = self.go_back.take() {
                    Ok(Some(back))
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn take(&mut self) -> Widget {
        AddClientView {
            go_back: self.go_back.take(),
            training_id: self.training_id,
            query: self.query.clone(),
            offset: self.offset,
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

    keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
        "🔙 Назад",
        Callback::Back.to_data(),
    )]);
    Ok((msg, keymap))
}

fn remove_non_alphanumeric(input: &str) -> String {
    input.chars().filter(|c| c.is_alphanumeric()).collect()
}

fn make_button(user: &User) -> InlineKeyboardButton {
    InlineKeyboardButton::callback(
        format!(
            "{}{} {}",
            user_type(user),
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
    Back,
    Select([u8; 12]),
}
