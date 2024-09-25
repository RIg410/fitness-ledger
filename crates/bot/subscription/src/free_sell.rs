use super::{
    sell::{Sell, SellView},
    View,
};
use async_trait::async_trait;
use bot_core::{callback_data::Calldata as _, calldata, context::Context, widget::Jmp};
use eyre::Result;
use model::{decimal::Decimal, rights::Rule};
use serde::{Deserialize, Serialize};
use std::{mem, num::NonZero};
use teloxide::{
    types::{InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

pub struct FeeSellView {
    state: State,
}

impl FeeSellView {
    pub fn new() -> FeeSellView {
        FeeSellView {
            state: State::SetItems,
        }
    }
}

#[async_trait]
impl View for FeeSellView {
    fn name(&self) -> &'static str {
        "FeeSellView"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let mut text = render_state(&self.state);
        text.push_str(&escape("-------------------\n"));
        let mut keymap = InlineKeyboardMarkup::default();
        match &self.state {
            State::SetItems => {
                text.push_str("*Введите количество занятий*");
            }
            State::SetPrice(_) => {
                text.push_str("*Введите стоимость*");
            }
            State::Finish(_, _) => {
                text.push_str("*Все верно?*");
                keymap = keymap.append_row(vec![
                    Callback::Sell.button("✅ Да"),
                    Callback::Cancel.button("❌ Нет"),
                ]);
            }
        }

        ctx.send_msg_with_markup(&text, keymap).await?;
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Context, message: &Message) -> Result<Jmp> {
        let text = if let Some(text) = message.text() {
            text
        } else {
            return Ok(Jmp::Stay);
        };

        self.state = match mem::take(&mut self.state) {
            State::SetItems => {
                if let Ok(items) = text.parse() {
                    State::SetPrice(items)
                } else {
                    ctx.send_msg("Введите число").await?;
                    State::SetItems
                }
            }
            State::SetPrice(items) => {
                if let Ok(price) = text.parse() {
                    State::Finish(items.clone(), price)
                } else {
                    ctx.send_msg("Введите число").await?;
                    State::SetPrice(items)
                }
            }
            State::Finish(items, price) => {
                ctx.delete_msg(message.id).await?;
                State::Finish(items, price)
            }
        };

        Ok(Jmp::Stay)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        let state = mem::take(&mut self.state).inner();
        if let Some((items, price)) = state {
            match calldata!(data) {
                Callback::Sell => {
                    ctx.ensure(Rule::FreeSell)?;

                    return Ok(SellView::new(Sell::free(price, items.get())).into());
                }
                Callback::Cancel => Ok(Jmp::Back),
            }
        } else {
            Ok(Jmp::Stay)
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Sell,
    Cancel,
}

#[derive(Default, Clone)]
enum State {
    #[default]
    SetItems,
    SetPrice(NonZero<u32>),
    Finish(NonZero<u32>, Decimal),
}

impl State {
    fn inner(self) -> Option<(NonZero<u32>, Decimal)> {
        match self {
            State::Finish(items, price) => Some((items, price)),
            _ => None,
        }
    }
}

fn render_state(state: &State) -> String {
    match state {
        State::SetItems => {
            format!("📌 Количество занятий:_❓_\nЦена:_❓_\n")
        }
        State::SetPrice(items) => {
            format!("📌 Количество занятий:_{}_\nЦена:_❓_\n", items)
        }
        State::Finish(items, price) => {
            format!(
                "📌 Количество занятий:_{}_\nЦена:_{}_\n",
                items,
                price.to_string().replace(".", ",")
            )
        }
    }
}
