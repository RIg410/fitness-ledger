use super::{
    sell::{Sell, SellView},
    View,
};
use crate::{callback_data::Calldata as _, context::Context, state::Widget};
use async_trait::async_trait;
use eyre::Result;
use model::{decimal::Decimal, rights::Rule};
use serde::{Deserialize, Serialize};
use std::{mem, num::NonZero};
use teloxide::{
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

pub struct FeeSellView {
    go_back: Option<Widget>,
    state: State,
}

impl FeeSellView {
    pub fn new(go_back: Widget) -> FeeSellView {
        FeeSellView {
            go_back: Some(go_back),
            state: State::SetItems,
        }
    }
}

#[async_trait]
impl View for FeeSellView {
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
                    InlineKeyboardButton::callback("✅ Да", Callback::Sell.to_data()),
                    InlineKeyboardButton::callback("❌ Нет", Callback::Cancel.to_data()),
                ]);
            }
        }

        let id = ctx.send_msg_with_markup(&text, keymap).await?;
        ctx.update_origin_msg_id(id);
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Option<Widget>> {
        let text = if let Some(text) = message.text() {
            text
        } else {
            return Ok(None);
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
        self.show(ctx).await?;

        Ok(None)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Option<Widget>> {
        let state = mem::take(&mut self.state).inner();
        if let Some((items, price)) = state {
            let cb = if let Some(cb) = Callback::from_data(data) {
                cb
            } else {
                return Ok(None);
            };
            match cb {
                Callback::Sell => {
                    ctx.ensure(Rule::FreeSell)?;
                    let back = Box::new(FeeSellView {
                        go_back: self.go_back.take(),
                        state: State::SetItems,
                    });
                    let widget = Box::new(SellView::new(Sell::free(price, items.get()), back));
                    return Ok(Some(widget));
                }
                Callback::Cancel => Ok(self.go_back.take()),
            }
        } else {
            self.show(ctx).await?;
            Ok(None)
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Sell,
    Cancel,
}

#[derive(Default)]
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
