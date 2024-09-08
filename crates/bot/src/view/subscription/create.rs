use std::{mem, num::NonZero};

use super::View;
use crate::{
    callback_data::Calldata as _, context::Context, state::Widget, view::menu::MainMenuItem,
};
use async_trait::async_trait;
use eyre::Result;
use ledger::subscriptions::CreateSubscriptionError;
use model::{decimal::Decimal, rights::Rule};
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

pub struct CreateSubscription {
    go_back: Option<Widget>,
    state: State,
}

impl CreateSubscription {
    pub fn new(go_back: Widget) -> CreateSubscription {
        CreateSubscription {
            go_back: Some(go_back),
            state: State::SetName,
        }
    }
}

#[async_trait]
impl View for CreateSubscription {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let mut text = render_state(&self.state);
        text.push_str(&escape("-------------------\n"));
        let mut keymap = InlineKeyboardMarkup::default();
        match &self.state {
            State::SetName => {
                text.push_str("*Введите название абонемента*");
            }
            State::SetItems(_) => {
                text.push_str("*Введите количество занятий в абонементе*");
            }
            State::SetPrice(_, _) => {
                text.push_str("*Введите стоимость абонемента*");
            }
            State::Finish(_, _, _) => {
                text.push_str("*Все верно?*");
                keymap = keymap.append_row(vec![
                    InlineKeyboardButton::callback(
                        "✅ Сохранить",
                        CreateCallback::Create.to_data(),
                    ),
                    InlineKeyboardButton::callback("❌ Отмена", CreateCallback::Cancel.to_data()),
                ]);
            }
        }

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
        let text = if let Some(text) = message.text() {
            text
        } else {
            return Ok(None);
        };

        self.state = match mem::take(&mut self.state) {
            State::SetName => {
                let name = text.to_string();
                let sub = ctx
                    .ledger
                    .subscriptions
                    .get_by_name(&mut ctx.session, &name)
                    .await?;
                if sub.is_some() {
                    ctx.send_msg("Абонемент с таким именем уже существует")
                        .await?;
                    return Ok(None);
                }
                State::SetItems(text.to_string())
            }
            State::SetItems(name) => {
                if let Ok(items) = text.parse() {
                    State::SetPrice(name.clone(), items)
                } else {
                    ctx.send_msg("Введите число").await?;
                    State::SetItems(name)
                }
            }
            State::SetPrice(name, items) => {
                if let Ok(price) = text.parse() {
                    State::Finish(name.clone(), items.clone(), price)
                } else {
                    ctx.send_msg("Введите число").await?;
                    State::SetPrice(name, items)
                }
            }
            State::Finish(name, items, price) => {
                ctx.delete_msg(message.id).await?;
                State::Finish(name, items, price)
            }
        };
        self.show(ctx).await?;

        Ok(None)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Option<Widget>> {
        let state = mem::take(&mut self.state).inner();
        if let Some((name, items, price)) = state {
            match CreateCallback::from_data(data)? {
                CreateCallback::Create => {
                    ctx.ensure(Rule::CreateSubscription)?;
                    let result = ctx
                        .ledger
                        .subscriptions
                        .create_subscription(&mut ctx.session, name, items.get(), price)
                        .await;
                    match result {
                        Ok(_) => {
                            ctx.send_msg("✅Абонемент создан").await?;
                            Ok(self.go_back.take())
                        }
                        Err(CreateSubscriptionError::NameAlreadyExists) => {
                            ctx.send_msg(&"Не удалось создать абонемент: Имя уже занято")
                                .await?;
                            Ok(None)
                        }
                        Err(CreateSubscriptionError::InvalidPrice) => {
                            ctx.send_msg("Не удалось создать абонемент: Неверная цена")
                                .await?;
                            Ok(None)
                        }
                        Err(CreateSubscriptionError::InvalidItems) => {
                            ctx.send_msg(
                                "Не удалось создать абонемент: Неверное количество занятий",
                            )
                            .await?;
                            Ok(None)
                        }
                        Err(CreateSubscriptionError::Common(err)) => Err(err),
                    }
                }
                CreateCallback::Cancel => Ok(self.go_back.take()),
            }
        } else {
            self.show(ctx).await?;
            Ok(None)
        }
    }
}

#[derive(Default)]
enum State {
    #[default]
    SetName,
    SetItems(String),
    SetPrice(String, NonZero<u32>),
    Finish(String, NonZero<u32>, Decimal),
}

impl State {
    fn inner(self) -> Option<(String, NonZero<u32>, Decimal)> {
        match self {
            State::Finish(name, items, price) => Some((name, items, price)),
            _ => None,
        }
    }
}

fn render_state(state: &State) -> String {
    match state {
        State::SetName => {
            format!("📌 Тариф: _❓_\nКоличество занятий:_❓_\nЦена:_❓_\n")
        }
        State::SetItems(name) => {
            format!("📌 Тариф: _{}_\nКоличество занятий:_❓_\nЦена:_❓_\n", name)
        }
        State::SetPrice(name, items) => {
            format!(
                "📌 Тариф: _{}_\nКоличество занятий:_{}_\nЦена:_❓_\n",
                name, items
            )
        }
        State::Finish(name, items, price) => {
            format!(
                "📌 Тариф: _{}_\nКоличество занятий:_{}_\nЦена:_{}_\n",
                name,
                items,
                price.to_string().replace(".", ",")
            )
        }
    }
}

#[derive(Serialize, Deserialize)]
enum CreateCallback {
    Create,
    Cancel,
}
