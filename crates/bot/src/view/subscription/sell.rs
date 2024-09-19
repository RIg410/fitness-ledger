use super::{confirm::ConfirmSell, presell::PreSellView, View};
use crate::{
    callback_data::Calldata as _, context::Context, state::Widget, view::users::profile::user_type,
};
use async_trait::async_trait;
use eyre::{eyre, Error, Result};
use model::{decimal::Decimal, rights::Rule, user::User};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

pub const LIMIT: u64 = 7;

pub struct SellView {
    go_back: Option<Widget>,
    sell: Sell,
    query: String,
    offset: u64,
}

impl SellView {
    pub fn new(sell: Sell) -> SellView {
        SellView {
            go_back: None,
            sell,
            query: "".to_string(),
            offset: 0,
        }
    }

    pub fn select(&mut self, user_id: i64, _: &mut Context) -> Result<Option<Widget>> {
        return Ok(Some(ConfirmSell::new(user_id, self.sell).boxed()));
    }

    pub fn presell(&mut self) -> Result<Option<Widget>> {
        let back = Box::new(SellView {
            go_back: self.go_back.take(),
            sell: self.sell,
            query: self.query.clone(),
            offset: self.offset,
        });
        let view = PreSellView::new(self.sell).boxed();
        return Ok(Some(view));
    }
}

#[async_trait]
impl View for SellView {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let (text, keymap) = render(&self.sell, ctx, &self.query, self.offset).await?;
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
        ctx.ensure(Rule::SellSubscription)?;
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
            Callback::Select(user_id) => self.select(user_id, ctx),
            Callback::Back => {
                if let Some(back) = self.go_back.take() {
                    Ok(Some(back))
                } else {
                    Ok(None)
                }
            }
            Callback::PreSell => self.presell(),
        }
    }

    fn take(&mut self) -> Widget {
        SellView {
            go_back: self.go_back.take(),
            sell: self.sell.clone(),
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
    sell: &Sell,
    ctx: &mut Context,
    query: &str,
    offset: u64,
) -> Result<(String, InlineKeyboardMarkup), Error> {
    let users = ctx
        .ledger
        .users
        .find(&mut ctx.session, &query, offset, LIMIT)
        .await?;
    let (name, price, items) = match sell {
        Sell::Sub(id) => {
            let sub = ctx
                .ledger
                .subscriptions
                .get(&mut ctx.session, *id)
                .await?
                .ok_or_else(|| eyre!("Subscription {} not found", id))?;
            (sub.name, sub.price, sub.items)
        }
        Sell::Free { price, items } => ("🤑".to_owned(), *price, *items),
    };

    let msg = format!(
        "📌 Тариф: _{}_\nКоличество занятий:_{}_\nЦена:_{}_\n\n
Что бы найти пользователя, введите имя, фамилию или телефон пользователя\\.\n
Запрос: _'{}'_",
        escape(&name),
        items,
        price.to_string().replace(".", ","),
        escape(query),
    );
    let mut keymap = InlineKeyboardMarkup::default();

    for user in &users {
        keymap = keymap.append_row(vec![make_button(user)]);
    }

    keymap = keymap.append_row(Callback::PreSell.btn_row("Новый пользователь 🪪"));

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

#[derive(Clone, Copy)]
pub enum Sell {
    Sub(ObjectId),
    Free { price: Decimal, items: u32 },
}

impl Sell {
    pub fn with_id(id: ObjectId) -> Self {
        Self::Sub(id)
    }

    pub fn free(price: Decimal, items: u32) -> Self {
        Self::Free { price, items }
    }
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
        Callback::Select(user.tg_id).to_data(),
    )
}

#[derive(Debug, Serialize, Deserialize)]
enum Callback {
    Next,
    Prev,
    Back,
    Select(i64),
    PreSell,
}
