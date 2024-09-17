use super::sell::Sell;
use crate::{
    callback_data::Calldata as _,
    context::Context,
    state::Widget,
    view::{menu::MainMenuView, View},
};
use async_trait::async_trait;
use eyre::{eyre, Error, Result};
use model::rights::Rule;
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

pub struct PreSellView {
    go_back: Option<Widget>,
    sell: Sell,
    state: State,
}

impl PreSellView {
    pub fn new(sell: Sell, go_back: Option<Widget>) -> PreSellView {
        PreSellView {
            go_back,
            sell,
            state: State::Init,
        }
    }
}

#[async_trait]
impl View for PreSellView {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        match &self.state {
            State::Init => {
                let (text, keymap) = render_init().await?;
                ctx.edit_origin(&text, keymap).await?;
            }
            State::Confirm(phone) => {
                let (text, keymap) = render_confirm(ctx, phone, self.sell).await?;
                ctx.edit_origin(&text, keymap).await?;
            }
        }
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Option<Widget>> {
        match &self.state {
            State::Init => {
                let phone = message.text().ok_or_else(|| eyre!("No text"))?;
                if phone.starts_with("+7") {
                    let exists = ctx
                        .ledger
                        .users
                        .find_by_phone(&mut ctx.session, phone)
                        .await?
                        .is_some();

                    if exists {
                        ctx.send_msg("Пользователь с таким номером уже существует")
                            .await?;
                        self.show(ctx).await?;
                        return Ok(None);
                    }

                    self.state = State::Confirm(phone.to_owned());
                    self.show(ctx).await?;
                } else {
                    ctx.send_msg("Номер должен начинаться с +7").await?;
                }
            }
            State::Confirm(_) => {
                ctx.delete_msg(message.id).await?;
            }
        }
        Ok(None)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Option<Widget>> {
        let cb = if let Some(cb) = Callback::from_data(data) {
            cb
        } else {
            return Ok(None);
        };

        let phone = if let State::Confirm(phone) = &self.state {
            phone.to_owned()
        } else {
            return Ok(None);
        };

        match cb {
            Callback::Sell => {
                let result = match self.sell {
                    Sell::Sub(sub) => {
                        ctx.ensure(Rule::SellSubscription)?;
                        ctx.ledger
                            .presell_subscription(&mut ctx.session, sub, phone, ctx.me.tg_id)
                            .await
                    }
                    Sell::Free { price, items } => {
                        ctx.ensure(Rule::FreeSell)?;
                        ctx.ledger
                            .presell_free_subscription(
                                &mut ctx.session,
                                price,
                                items,
                                phone,
                                ctx.me.tg_id,
                            )
                            .await
                    }
                };
                if let Err(err) = result {
                    Err(err.into())
                } else {
                    ctx.send_msg("🤑 Продано").await?;
                    let view = Box::new(MainMenuView);
                    view.send_self(ctx).await?;
                    Ok(Some(view))
                }
            }
            Callback::Cancel => {
                if let Some(back) = self.go_back.take() {
                    Ok(Some(back))
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn take(&mut self) -> Widget {
        PreSellView {
            go_back: self.go_back.take(),
            sell: self.sell.clone(),
            state: self.state.clone(),
        }
        .boxed()
    }
}

async fn render_init() -> Result<(String, InlineKeyboardMarkup), Error> {
    let message =
        "Введите номер телефона пользователя\\. Номер должен начинаться с *\\+7*".to_string();
    let mut keymap = InlineKeyboardMarkup::default();
    keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
        "❌ Отмена",
        Callback::Cancel.to_data(),
    )]);
    Ok((message, keymap))
}

async fn render_confirm(
    ctx: &mut Context,
    phone: &str,
    sell: Sell,
) -> Result<(String, InlineKeyboardMarkup), Error> {
    let (name, price, items) = match sell {
        Sell::Sub(id) => {
            let sub = ctx
                .ledger
                .subscriptions
                .get(&mut ctx.session, id)
                .await?
                .ok_or_else(|| eyre::eyre!("Subscription {} not found", id))?;
            (sub.name, sub.price, sub.items)
        }
        Sell::Free { price, items } => ("🤑".to_owned(), price, items),
    };

    let text = format!(
        "
 📌  Продажа
Тариф: _{}_\nКоличество занятий:_{}_\nЦена:_{}_\n
Пользователь:
    Номер:_{}_\n\n
    Все верно? 
    ",
        escape(&name),
        items,
        price.to_string().replace(".", ","),
        escape(phone)
    );

    let mut keymap = InlineKeyboardMarkup::default();
    keymap = keymap.append_row(vec![
        InlineKeyboardButton::callback("✅ Да", Callback::Sell.to_data()),
        InlineKeyboardButton::callback("❌ Отмена", Callback::Cancel.to_data()),
    ]);
    Ok((text, keymap))
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Sell,
    Cancel,
}

#[derive(Clone)]
enum State {
    Init,
    Confirm(String),
}
