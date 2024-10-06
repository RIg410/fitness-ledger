use super::{confirm::ConfirmSell, View};
use async_trait::async_trait;
use bot_core::{callback_data::Calldata as _, calldata, context::Context, widget::Jmp};
use bot_viewer::{fmt_phone, user::fmt_user_type};
use eyre::{eyre, Error, Result};
use model::{
    rights::Rule,
    user::{sanitize_phone, User},
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardMarkup, Message};

pub const LIMIT: u64 = 7;

pub struct SellView {
    sell: ObjectId,
    state: State,
}

impl SellView {
    pub fn new(sell: ObjectId) -> SellView {
        SellView {
            sell,
            state: State::SelectUser,
        }
    }
}

#[async_trait]
impl View for SellView {
    fn name(&self) -> &'static str {
        "SellView"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let mut text = String::new();
        let mut keymap = InlineKeyboardMarkup::default();

        match &self.state {
            State::SelectUser => {
                text = "Введите номер телефона пользователя".to_string();
            }
            State::FindByPhone(phone) => {
                if ctx
                    .ledger
                    .users
                    .get_by_phone(&mut ctx.session, &phone)
                    .await?
                    .is_some()
                {
                    return Ok(());
                } else {
                    text = format!(
                        "Пользователь с номером *{}* не найден\\. Создать нового пользователя?",
                        fmt_phone(phone)
                    );
                    keymap = keymap.append_row(Callback::CreateNewUser.btn_row("Создать"));
                }
            }
        }

        ctx.edit_origin(&text, keymap).await?;
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Context, msg: &Message) -> Result<Jmp> {
        ctx.delete_msg(msg.id).await?;
        let query = msg.text().unwrap_or_default();

        if query.starts_with("8") {
            let query = "7".to_string() + &query[1..];
            self.state = State::FindByPhone(sanitize_phone(&query));
        } else if query.starts_with("+7") {
            self.state = State::FindByPhone(sanitize_phone(&query));
        } else {
            ctx.send_msg("Номер телефона должен начинаться с 8 или \\+7")
                .await?;
            return Ok(Jmp::Stay);
        }

        Ok(Jmp::Stay)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        ctx.ensure(Rule::SellSubscription)?;
        match calldata!(data) {
            Callback::CreateNewUser => Ok(Jmp::Stay),
        }
    }
}

// async fn render(
//     sell: &Sell,
//     ctx: &mut Context,
//     query: &str,
// ) -> Result<(String, InlineKeyboardMarkup), Error> {

//     //     let user = ctx
//     //         .ledger.users
//     //         .get_by_phone(&mut ctx.session, query)
//     //         .await?;

//     //     let (name, price, items) = match sell {
//     //         Sell::Sub(id) => {
//     //             let sub = ctx
//     //                 .ledger
//     //                 .subscriptions
//     //                 .get(&mut ctx.session, *id)
//     //                 .await?
//     //                 .ok_or_else(|| eyre!("Subscription {} not found", id))?;
//     //             (sub.name, sub.price, sub.items)
//     //         }
//     //     };

//     //     let msg = format!(
//     //         "📌 Тариф: _{}_\nКоличество занятий:_{}_\nЦена:_{}_\n\n
//     // Что бы найти пользователя, введите телефон пользователя\\.\n
//     // Телефон: _'{}'_",
//     //         escape(&name),
//     //         items,
//     //         price.to_string().replace(".", ","),
//     //         escape(&fmt_phone(&query)),
//     //     );
//     //     let mut keymap = InlineKeyboardMarkup::default();

//     //     for user in &users {
//     //         keymap = keymap.append_row(vec![make_button(user)]);
//     //     }

//     //     keymap = keymap.append_row(Callback::PreSell.btn_row("Новый пользователь 🪪"));

//     //     let mut raw = vec![];

//     //     if !raw.is_empty() {
//     //         keymap = keymap.append_row(raw);
//     //     }

//     //     Ok((msg, keymap))
// }

// #[derive(Clone, Copy)]
// pub enum Sell {
//     Sub(ObjectId),
// }

// impl Sell {
//     pub fn with_id(id: ObjectId) -> Self {
//         Self::Sub(id)
//     }
// }

// fn remove_non_alphanumeric(input: &str) -> String {
//     input.chars().filter(|c| c.is_alphanumeric()).collect()
// }

// fn make_button(user: &User) -> InlineKeyboardButton {
//     InlineKeyboardButton::callback(
//         format!(
//             "{}{} {}",
//             fmt_user_type(user),
//             user.name.first_name,
//             user.name.last_name.as_ref().unwrap_or(&"".to_string())
//         ),
//         Callback::Select(user.tg_id).to_data(),
//     )
// }

#[derive(Debug, Serialize, Deserialize)]
enum Callback {
    CreateNewUser,
}

enum State {
    SelectUser,
    FindByPhone(String),
}
