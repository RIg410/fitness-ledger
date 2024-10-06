use crate::SubscriptionView;

use super::View;
use async_trait::async_trait;
use bot_core::{callback_data::Calldata as _, calldata, context::Context, widget::Jmp};
use eyre::{eyre, Error, Result};
use model::rights::Rule;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{types::InlineKeyboardMarkup, utils::markdown::escape};

pub struct ConfirmSell {
    user_id: i64,
    sub: ObjectId,
}

impl ConfirmSell {
    pub fn new(user_id: i64, sell: ObjectId) -> ConfirmSell {
        ConfirmSell { user_id, sub: sell }
    }
}

#[async_trait]
impl View for ConfirmSell {
    fn name(&self) -> &'static str {
        "ConfirmSell"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let (text, keymap) = render(ctx, self.user_id, self.sub).await?;
        ctx.edit_origin(&text, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::Sell => {
                ctx.ensure(Rule::SellSubscription)?;
                let result = ctx
                    .ledger
                    .sell_subscription(&mut ctx.session, self.sub, self.user_id)
                    .await;

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
    sub: ObjectId,
) -> Result<(String, InlineKeyboardMarkup), Error> {
    let sub = ctx
        .ledger
        .subscriptions
        .get(&mut ctx.session, sub)
        .await?
        .ok_or_else(|| eyre::eyre!("Subscription {} not found", sub))?;

    let user = ctx
        .ledger
        .users
        .get(&mut ctx.session, user_id)
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
        escape(&sub.name),
        sub.items,
        sub.price.to_string().replace(".", ","),
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
