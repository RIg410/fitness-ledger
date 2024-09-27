use crate::SubscriptionView;

use super::{sell::Sell, View};
use async_trait::async_trait;
use bot_core::{callback_data::Calldata as _, calldata, context::Context, widget::Jmp};
use eyre::{eyre, Error, Result};
use model::rights::Rule;
use serde::{Deserialize, Serialize};
use teloxide::{types::InlineKeyboardMarkup, utils::markdown::escape};

pub struct ConfirmSell {
    user_id: i64,
    sell: Sell,
}

impl ConfirmSell {
    pub fn new(user_id: i64, sell: Sell) -> ConfirmSell {
        ConfirmSell { user_id, sell }
    }
}

#[async_trait]
impl View for ConfirmSell {
    fn name(&self) -> &'static str {
        "ConfirmSell"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let (text, keymap) = render(ctx, self.user_id, self.sell).await?;
        ctx.edit_origin(&text, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::Sell => {
                let result = match self.sell {
                    Sell::Sub(sub) => {
                        ctx.ensure(Rule::SellSubscription)?;
                        ctx.ledger
                            .sell_subscription(&mut ctx.session, sub, self.user_id)
                            .await
                    }
                    Sell::Free { price, items } => {
                        ctx.ensure(Rule::FreeSell)?;
                        ctx.ledger
                            .sell_free_subscription(&mut ctx.session, price, items, self.user_id)
                            .await
                    }
                };
                if let Err(err) = result {
                    Err(err.into())
                } else {
                    ctx.send_msg("🤑 Продано").await?;
                    ctx.reset_origin().await?;
                    Ok(Jmp::Goto(SubscriptionView.into()))
                }
            }
            Callback::Cancel => Ok(Jmp::Back),
        }
    }
}

async fn render(
    ctx: &mut Context,
    user_id: i64,
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
    let user = ctx
        .ledger
        .users
        .get_by_tg_id(&mut ctx.session, user_id)
        .await?
        .ok_or_else(|| eyre!("User not found:{}", user_id))?;

    let text = format!(
        "
 📌  Продажа
Тариф: _{}_\nКоличество занятий:_{}_\nЦена:_{}_\n
Пользователь:
    Имя:_{}_
    Фамилия:_{}_
    Номер:_{}_\n\n
    Все верно? 
    ",
        escape(&name),
        items,
        price.to_string().replace(".", ","),
        escape(&user.name.first_name),
        escape(&user.name.last_name.unwrap_or_else(|| "-".to_string())),
        escape(&user.phone)
    );

    let mut keymap = InlineKeyboardMarkup::default();
    keymap = keymap.append_row(vec![
        Callback::Sell.button("✅ Да"),
        Callback::Cancel.button("❌ Отмена"),
    ]);
    Ok((text, keymap))
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Sell,
    Cancel,
}
