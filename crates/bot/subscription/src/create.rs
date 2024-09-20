use std::mem;

use super::View;
use async_trait::async_trait;
use bot_core::{callback_data::Calldata, calldata, context::Context, widget::Dest};
use eyre::Result;
use ledger::subscriptions::CreateSubscriptionError;
use model::{rights::Rule, subscription::Subscription};
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

pub struct CreateSubscription {
    state: State,
    subscription: Subscription,
}

impl CreateSubscription {
    pub fn new() -> CreateSubscription {
        CreateSubscription {
            state: State::SetName,
            subscription: Subscription::default(),
        }
    }

    fn render_state(&self) -> String {
        let none = "❓".to_string();
        let (name, items, price, days, freeze) = match self.state {
            State::SetName => (None, None, None, None, None),
            State::SetItems => (
                Some(self.subscription.name.as_str()),
                None,
                None,
                None,
                None,
            ),
            State::SetPrice => (
                Some(self.subscription.name.as_str()),
                Some(self.subscription.items),
                None,
                None,
                None,
            ),
            State::SetExpirationDaysDays => (
                Some(self.subscription.name.as_str()),
                Some(self.subscription.items),
                Some(self.subscription.price),
                None,
                None,
            ),
            State::SetFreezeDays => (
                Some(self.subscription.name.as_str()),
                Some(self.subscription.items),
                Some(self.subscription.price),
                Some(self.subscription.expiration_days),
                None,
            ),
            State::Finish => (
                Some(self.subscription.name.as_str()),
                Some(self.subscription.items),
                Some(self.subscription.price),
                Some(self.subscription.expiration_days),
                Some(self.subscription.freeze_days),
            ),
        };

        format!("📌 Тариф: _{}_\nКоличество занятий:_{}_\nЦена:_{}_\nСрок действия:_{}_\nЗаморозка:_{}_\n",
                    escape(name.unwrap_or(&none)),
                    items.map(|i|i.to_string()).unwrap_or_else(||none.clone()),
                    price.map(|i|i.to_string().replace(".", ",")).unwrap_or_else(||none.clone()),
                    days.map(|i|i.to_string()).unwrap_or_else(||none.clone()),
                    freeze.map(|i|i.to_string()).unwrap_or_else(||none.clone()),
                )
    }
}

#[async_trait]
impl View for CreateSubscription {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let mut text = self.render_state();
        text.push_str(&escape("-------------------\n"));
        let mut keymap = InlineKeyboardMarkup::default();
        match self.state {
            State::SetName => {
                text.push_str("*Введите название абонемента*");
            }
            State::SetItems => {
                text.push_str("*Введите количество занятий в абонементе*");
            }
            State::SetPrice => {
                text.push_str("*Введите стоимость абонемента*");
            }
            State::SetExpirationDaysDays => {
                text.push_str("*Введите срок действия абонемента\\(дни\\)*");
            }
            State::SetFreezeDays => {
                text.push_str("*Введите количество дней заморозки абонемента\\(дни\\)*");
            }
            State::Finish => {
                text.push_str("*Все верно?*");
                keymap = keymap.append_row(vec![
                    Callback::Create.button("✅ Сохранить"),
                    Callback::Cancel.button("❌ Отмена"),
                ]);
            }
        }

        let id = ctx.send_msg_with_markup(&text, keymap).await?;
        ctx.update_origin_msg_id(id);
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Context, message: &Message) -> Result<Dest> {
        let text = if let Some(text) = message.text() {
            text
        } else {
            return Ok(Dest::None);
        };

        self.state = match self.state {
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
                    return Ok(Dest::None);
                }
                self.subscription.name = text.to_string();
                State::SetItems
            }
            State::SetItems => {
                if let Ok(items) = text.parse() {
                    self.subscription.items = items;
                    State::SetPrice
                } else {
                    ctx.send_msg("Введите число").await?;
                    State::SetItems
                }
            }
            State::SetPrice => {
                if let Ok(price) = text.parse() {
                    self.subscription.price = price;
                    State::SetExpirationDaysDays
                } else {
                    ctx.send_msg("Введите число").await?;
                    State::SetPrice
                }
            }
            State::SetExpirationDaysDays => {
                if let Ok(expiration_days) = text.parse() {
                    self.subscription.expiration_days = expiration_days;
                    State::SetFreezeDays
                } else {
                    ctx.send_msg("Введите число").await?;
                    State::SetExpirationDaysDays
                }
            }
            State::SetFreezeDays => {
                if let Ok(freeze_days) = text.parse() {
                    self.subscription.freeze_days = freeze_days;
                    State::Finish
                } else {
                    ctx.send_msg("Введите число").await?;
                    State::SetFreezeDays
                }
            }
            State::Finish => {
                ctx.delete_msg(message.id).await?;
                State::Finish
            }
        };
        self.show(ctx).await?;

        Ok(Dest::None)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Dest> {
        match calldata!(data) {
            Callback::Create => {
                ctx.ensure(Rule::CreateSubscription)?;
                let sub = mem::take(&mut self.subscription);
                let result = ctx
                    .ledger
                    .subscriptions
                    .create_subscription(
                        &mut ctx.session,
                        sub.name,
                        sub.items,
                        sub.price,
                        sub.expiration_days,
                        sub.freeze_days,
                    )
                    .await;
                match result {
                    Ok(_) => {
                        ctx.send_msg("✅Абонемент создан").await?;
                        Ok(Dest::Back)
                    }
                    Err(CreateSubscriptionError::NameAlreadyExists) => {
                        ctx.send_msg(&"Не удалось создать абонемент: Имя уже занято")
                            .await?;
                        Ok(Dest::None)
                    }
                    Err(CreateSubscriptionError::InvalidPrice) => {
                        ctx.send_msg("Не удалось создать абонемент: Неверная цена")
                            .await?;
                        Ok(Dest::None)
                    }
                    Err(CreateSubscriptionError::InvalidItems) => {
                        ctx.send_msg("Не удалось создать абонемент: Неверное количество занятий")
                            .await?;
                        Ok(Dest::None)
                    }
                    Err(CreateSubscriptionError::Common(err)) => Err(err),
                }
            }
            Callback::Cancel => Ok(Dest::Back),
        }
    }
}

#[derive(Default, Clone, Copy)]
enum State {
    #[default]
    SetName,
    SetItems,
    SetPrice,
    SetExpirationDaysDays,
    SetFreezeDays,
    Finish,
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Create,
    Cancel,
}
