use std::{mem, str::FromStr as _};

use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use eyre::Result;
use model::{decimal::Decimal, rights::Rule};
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

pub struct InOut {
    state: State,
    io: Io,
}
impl InOut {
    pub fn new(io: Io) -> Self {
        Self {
            state: State::Description,
            io,
        }
    }
}

#[async_trait]
impl View for InOut {
    fn name(&self) -> &'static str {
        "FinInOut"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let mut text = format!("{}\n{}", self.io.render(), self.state.render());
        let mut keymap = InlineKeyboardMarkup::default();

        match self.state {
            State::Description => {
                text.push_str("\nВведите описание платежа:");
            }
            State::Amount(_) => {
                text.push_str("\nВведите сумму платежа:");
            }
            State::DateTime(_, _) => {
                text.push_str("\nВведите дату платежа: \\d\\.m\\.Y H:M");
            }
            State::Finish(_, _, _) => {
                text.push_str("\nВсе верно?");
                keymap = keymap.append_row(vec![
                    InlineKeyboardButton::callback("✅ Сохранить", Callback::Save.to_data()),
                    InlineKeyboardButton::callback("❌ Отмена", Callback::Back.to_data()),
                ]);
            }
        }

        ctx.send_msg_with_markup(&text, keymap).await?;
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Context, message: &Message) -> Result<Jmp> {
        let text = if let Some(msg) = message.text() {
            msg
        } else {
            return Ok(Jmp::Stay);
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
                    let dt = NaiveDateTime::parse_from_str(text, "%d.%m.%Y %H:%M")
                        .ok()
                        .and_then(|dt| Local.from_local_datetime(&dt).single());
                    if let Some(dt) = dt {
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
        Ok(Jmp::Stay)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::Save => match &self.state {
                State::Finish(description, amount, date) => match self.io {
                    Io::Deposit => {
                        ctx.ledger
                            .treasury
                            .deposit(&mut ctx.session, *amount, description.to_string(), date)
                            .await?;
                        ctx.send_msg("✅ Платеж сохранен").await?;
                        Ok(Jmp::Back)
                    }
                    Io::Payment => {
                        ctx.ledger
                            .treasury
                            .payment(&mut ctx.session, *amount, description.to_string(), date)
                            .await?;
                        ctx.send_msg("✅ Платеж сохранен").await?;
                        Ok(Jmp::Back)
                    }
                },
                _ => {
                    ctx.send_msg("Заполните все поля").await?;
                    self.state = State::Description;
                    Ok(Jmp::Stay)
                }
            },
            Callback::Back => Ok(Jmp::Back),
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Save,
    Back,
}

#[derive(Default, Clone)]
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

#[derive(Clone, Copy)]
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
