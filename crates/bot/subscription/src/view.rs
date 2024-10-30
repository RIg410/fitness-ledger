use crate::edit_type::EditSubscriptionType;

use super::{
    edit::{EditSubscription, EditType},
    sell::SellView,
    View,
};
use async_trait::async_trait;
use bot_core::{callback_data::Calldata as _, calldata, context::Context, widget::Jmp};
use bot_viewer::subscription::fmt_subscription_type;
use eyre::{Context as _, Error, Result};
use model::rights::Rule;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{types::InlineKeyboardMarkup, utils::markdown::escape};

pub struct SubscriptionOption {
    id: ObjectId,
}

impl SubscriptionOption {
    pub fn new(id: ObjectId) -> SubscriptionOption {
        SubscriptionOption { id }
    }

    async fn edit(&mut self, tp: EditType) -> Result<Jmp> {
        Ok(EditSubscription::new(self.id, tp).into())
    }

    async fn buy(&mut self, ctx: &mut Context) -> Result<Jmp> {
        let sub = ctx
            .ledger
            .subscriptions
            .get(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Subscription not found"))?;
        if !sub.user_can_buy {
            ctx.send_msg("Покупка абонемента недоступна").await?;
            return Ok(Jmp::Back);
        }

        Ok(Jmp::Back)
    }
}

#[async_trait]
impl View for SubscriptionOption {
    fn name(&self) -> &'static str {
        "SubscriptionOption"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let (txt, keymap) = render_sub(self.id, ctx).await.context("render")?;
        ctx.edit_origin(&txt, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::Delete => {
                ctx.ensure(Rule::EditSubscription)?;
                ctx.ledger
                    .subscriptions
                    .delete(&mut ctx.session, self.id)
                    .await?;
                Ok(Jmp::Back)
            }
            Callback::Buy => {
                ctx.ensure(Rule::BuySubscription)?;
                self.buy(ctx).await
            }
            Callback::Sell => {
                ctx.ensure(Rule::SellSubscription)?;
                Ok(SellView::new(self.id).into())
            }
            Callback::EditPrice => {
                ctx.ensure(Rule::EditSubscription)?;
                self.edit(EditType::Price).await
            }
            Callback::EditItems => {
                ctx.ensure(Rule::EditSubscription)?;
                self.edit(EditType::Items).await
            }
            Callback::EditName => {
                ctx.ensure(Rule::EditSubscription)?;
                self.edit(EditType::Name).await
            }
            Callback::EditFreezeDays => {
                ctx.ensure(Rule::EditSubscription)?;
                self.edit(EditType::FreezeDays).await
            }
            Callback::EditCanBuyByUser => {
                ctx.ensure(Rule::EditSubscription)?;
                self.edit(EditType::CanBuyByUser).await
            }
            Callback::EditSubscriptionType => {
                ctx.ensure(Rule::EditSubscription)?;
                Ok(EditSubscriptionType::new(self.id).into())
            }
            Callback::EditExpirationDays => {
                ctx.ensure(Rule::EditSubscription)?;
                self.edit(EditType::ExpirationDays).await
            }
        }
    }
}

async fn render_sub(
    id: ObjectId,
    ctx: &mut Context,
) -> Result<(String, InlineKeyboardMarkup), Error> {
    let sub = ctx
        .ledger
        .subscriptions
        .get(&mut ctx.session, id)
        .await?
        .ok_or_else(|| eyre::eyre!("Subscription not found"))?;
    let msg = format!(
        "📌 Тариф: _{}_\nКоличество занятий:_{}_\nЦена:_{}_\nДни заморозки:_{}_\nДействует дней:_{}_\nТип:_{}_",
        escape(&sub.name),
        sub.items,
        sub.price.to_string().replace(".", ","),
        sub.freeze_days,
        sub.expiration_days,
        fmt_subscription_type(ctx, &sub.subscription_type).await?,
    );

    let mut keymap = InlineKeyboardMarkup::default();

    if ctx.has_right(Rule::BuySubscription) {
        keymap = keymap.append_row(Callback::Buy.btn_row("🛒 Купить"));
    }
    if ctx.has_right(Rule::SellSubscription) {
        keymap = keymap.append_row(Callback::Sell.btn_row("🛒 Продать"));
    }
    if ctx.has_right(Rule::EditSubscription) {
        keymap = keymap.append_row(Callback::Delete.btn_row("❌ Удалить"));
        keymap = keymap.append_row(Callback::EditPrice.btn_row("Изменить цену 💸"));
        keymap = keymap.append_row(Callback::EditItems.btn_row("Изменить количество занятий"));
        keymap = keymap.append_row(Callback::EditName.btn_row("Изменить название"));
        keymap = keymap.append_row(Callback::EditFreezeDays.btn_row("Изменить дни заморозки"));
        keymap = keymap
            .append_row(Callback::EditCanBuyByUser.btn_row("Изменить доступность для покупки"));
        keymap = keymap.append_row(Callback::EditSubscriptionType.btn_row("Изменить тип"));
        keymap = keymap.append_row(Callback::EditExpirationDays.btn_row("Изменить время действия"));
    }

    Ok((msg, keymap))
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Delete,
    Sell,
    Buy,
    EditPrice,
    EditItems,
    EditName,
    EditFreezeDays,
    EditCanBuyByUser,
    EditSubscriptionType,
    EditExpirationDays,
}
