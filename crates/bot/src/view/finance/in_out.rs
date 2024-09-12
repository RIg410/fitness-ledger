use std::{mem, str::FromStr as _};

use super::View;
use crate::{
    callback_data::Calldata as _, context::Context, state::Widget, view::menu::MainMenuItem,
};
use async_trait::async_trait;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use eyre::Result;
use model::{decimal::Decimal, rights::Rule};
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

pub struct InOut {
    go_back: Option<Widget>,
    state: State,
    io: Io,
}
impl InOut {
    pub fn new(go_back: Option<Widget>, io: Io) -> Self {
        Self {
            go_back,
            state: State::Description,
            io,
        }
    }
}

#[async_trait]
impl View for InOut {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let mut text = format!("{}{}", self.io.render(), self.state.render());
        let mut keymap = InlineKeyboardMarkup::default();

        match self.state {
            State::Description => {
                text.push_str("\nВведите описание платежа:");
            }
            State::Amount(_) => {
                text.push_str("\nВведите сумму платежа:");
            }
            State::DateTime(_, _) => {
                text.push_str("\nВведите дату платежа: Y\\-m\\-d H:M");
            }
            State::Finish(_, _, _) => {
                text.push_str("\nВсе верно?");
                keymap = keymap.append_row(vec![
                    InlineKeyboardButton::callback("✅ Сохранить", Callback::Save.to_data()),
                    InlineKeyboardButton::callback("❌ Отмена", Callback::Back.to_data()),
                ]);
            }
        }

        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            "⬅️ Назад",
            Callback::Back.to_data(),
        )]);
        keymap = keymap.append_row(vec![MainMenuItem::Home.into()]);
        let id = ctx.send_msg_with_markup(&text, keymap).await?;
        ctx.update_origin_msg_id(id);
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Option<Widget>> {
        let text = if let Some(msg) = message.text() {
            msg
        } else {
            return Ok(None);
        };

        let state = mem::take(&mut self.state);
        self.state = match state {
            State::Description => State::Amount(text.to_string()),
            State::Amount(des) => {
                if let Ok(amount) = Decimal::from_str(text) {
                    if ctx.has_right(Rule::FinanceHistoricalDate) {
                        State::DateTime(des, amount)
                    } else {
                        State::Finish(des, amount, Local::now())
                    }
                } else {
                    ctx.send_msg("Введите корректное число").await?;
                    State::Amount(des)
                }
            }
            State::DateTime(des, amount) => {
                if text == "-" {
                    State::Finish(des, amount, Local::now())
                } else {
                    let dt = NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M")?;
                    if let Some(dt) = Local.from_local_datetime(&dt).single() {
                        State::Finish(des, amount, dt)
                    } else {
                        ctx.send_msg("Введите корректную дату").await?;
                        State::DateTime(des, amount)
                    }
                }
            }
            State::Finish(d, a, dt) => {
                ctx.delete_msg(message.id).await?;
                State::Finish(d, a, dt)
            }
        };
        self.show(ctx).await?;
        Ok(None)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Option<Widget>> {
        let cb = if let Some(cb) = Callback::from_data(data) {
            cb
        } else {
            return Ok(None);
        };
        match cb {
            Callback::Back => {
                if let Some(widget) = self.go_back.take() {
                    Ok(Some(widget))
                } else {
                    Ok(None)
                }
            }
            Callback::Save => match &self.state {
                State::Finish(description, amount, date) => match self.io {
                    Io::Deposit => {
                        ctx.ledger
                            .treasury
                            .deposit(
                                &mut ctx.session,
                                ctx.me.clone(),
                                *amount,
                                description.to_string(),
                                date,
                            )
                            .await?;
                        ctx.send_msg("✅ Платеж сохранен").await?;
                        if let Some(widget) = self.go_back.take() {
                            Ok(Some(widget))
                        } else {
                            Ok(None)
                        }
                    }
                    Io::Payment => {
                        ctx.ledger
                            .treasury
                            .payment(
                                &mut ctx.session,
                                ctx.me.clone(),
                                *amount,
                                description.to_string(),
                                date,
                            )
                            .await?;
                        ctx.send_msg("✅ Платеж сохранен").await?;
                        if let Some(widget) = self.go_back.take() {
                            Ok(Some(widget))
                        } else {
                            Ok(None)
                        }
                    }
                },
                _ => {
                    ctx.send_msg("Заполните все поля").await?;
                    self.state = State::Description;
                    self.show(ctx).await?;
                    Ok(None)
                }
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Back,
    Save,
}

#[derive(Default)]
enum State {
    #[default]
    Description,
    Amount(String),
    DateTime(String, Decimal),
    Finish(String, Decimal, DateTime<Local>),
}

impl State {
    pub fn render(&self) -> String {
        match self {
            State::Description => format!(
                "📝Описание:_❓_\n💲Сумма:❓\nДата:_{}_",
                Local::now().format("%d/%m/%Y %H:%M")
            ),
            State::Amount(description) => format!(
                "📝Описание:_{}_\n💲Сумма:❓\nДата:_{}_",
                escape(description),
                Local::now().format("%d/%m/%Y %H:%M")
            ),
            State::DateTime(description, amount) => format!(
                "📝Описание:_{}_\n💲Сумма:_{}_\nДата:_{}_",
                escape(description),
                amount.to_string().replace(".", ","),
                Local::now().format("%d/%m/%Y %H:%M")
            ),
            State::Finish(description, amount, date) => {
                format!(
                    "📝Описание:_{}_\n💲Сумма:_{}_\nДата:_{}_",
                    escape(description),
                    amount.to_string().replace(".", ","),
                    date.format("%d/%m/%Y %H:%M")
                )
            }
        }
    }
}

pub enum Io {
    Deposit,
    Payment,
}

impl Io {
    pub fn render(&self) -> &str {
        match self {
            Io::Deposit => "🤑Внести средства",
            Io::Payment => "💳Оплатить",
        }
    }
}
