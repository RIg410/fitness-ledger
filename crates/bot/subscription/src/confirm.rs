use crate::{sell::FAMILY_DISCOUNT, SubscriptionView};

use super::View;
use async_trait::async_trait;
use bot_core::{callback_data::Calldata as _, calldata, context::Context, widget::Jmp};
use bot_viewer::fmt_phone;
use eyre::{eyre, Error, Result};
use model::{decimal::Decimal, rights::Rule};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{types::InlineKeyboardMarkup, utils::markdown::escape};

pub struct ConfirmSell {
    user_id: ObjectId,
    sub: ObjectId,
    discount: Option<Decimal>,
}

impl ConfirmSell {
    pub fn new(user_id: ObjectId, sell: ObjectId) -> ConfirmSell {
        ConfirmSell {
            user_id,
            sub: sell,
            discount: None,
        }
    }
}

#[async_trait]
impl View for ConfirmSell {
    fn name(&self) -> &'static str {
        "ConfirmSell"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let (text, keymap) = render(ctx, self.user_id, self.sub, self.discount).await?;
        ctx.edit_origin(&text, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::Sell => {
                ctx.ensure(Rule::SellSubscription)?;
                let result = ctx
                    .ledger
                    .sell_subscription(
                        &mut ctx.session,
                        self.sub,
                        self.user_id,
                        self.discount.map(|d| d / Decimal::int(100)),
                    )
                    .await;

                if let Err(err) = result {
                    Err(err.into())
                } else {
                    ctx.send_msg("🤑 Продано").await?;
                    ctx.reset_origin().await?;
                    Ok(Jmp::Goto(SubscriptionView.into()))
                }
            }
            Callback::AddFamilyDiscount => {
                self.discount = Some(FAMILY_DISCOUNT);
                Ok(Jmp::Stay)
            }
            Callback::RemoveFamilyDiscount => {
                self.discount = None;
                Ok(Jmp::Stay)
            }
            Callback::Cancel => Ok(Jmp::Back),
        }
    }
}

async fn render(
    ctx: &mut Context,
    user_id: ObjectId,
    sub: ObjectId,
    discount: Option<Decimal>,
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
    Номер:_{}_\n
    Скидка: _{}%_
    \n
    Все верно? 
    ",
        escape(&sub.name),
        sub.items,
        sub.price.to_string().replace(".", ","),
        escape(&user.name.first_name),
        escape(&user.name.last_name.unwrap_or_else(|| "-".to_string())),
        fmt_phone(user.phone.as_deref()),
        discount.unwrap_or_default().to_string().replace(".", ",")
    );

    let mut keymap = InlineKeyboardMarkup::default();
    keymap = keymap.append_row(vec![
        Callback::Sell.button("✅ Да"),
        Callback::Cancel.button("❌ Отмена"),
    ]);
    if discount.is_none() {
        keymap = keymap.append_row(vec![
            Callback::AddFamilyDiscount.button("👨‍👩‍👧‍👦 Добавить семейную скидку")
        ]);
    } else {
        keymap = keymap.append_row(vec![
            Callback::RemoveFamilyDiscount.button("👨‍👩‍👧‍👦 Убрать семейную скидку")
        ]);
    }
    Ok((text, keymap))
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Sell,
    AddFamilyDiscount,
    RemoveFamilyDiscount,
    Cancel,
}
