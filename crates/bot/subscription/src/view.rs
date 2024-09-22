use super::{
    edit::{EditSubscription, EditType},
    sell::{Sell, SellView},
    View,
};
use async_trait::async_trait;
use bot_core::{callback_data::Calldata as _, calldata, context::Context, widget::Jmp};
use eyre::{Error, Result};
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
}

#[async_trait]
impl View for SubscriptionOption {
    fn name(&self) -> &'static str {
        "SubscriptionOption"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let (txt, keymap) = render_sub(self.id, ctx).await?;
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
                return Ok(Jmp::Back);
            }
            Callback::Sell => {
                ctx.ensure(Rule::SellSubscription)?;
                return Ok(SellView::new(Sell::with_id(self.id)).into());
            }
            Callback::EditPrice => {
                return self.edit(EditType::Price).await;
            }
            Callback::EditItems => {
                return self.edit(EditType::Items).await;
            }
            Callback::EditName => {
                return self.edit(EditType::Name).await;
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
        "📌 Тариф: _{}_\nКоличество занятий:_{}_\nЦена:_{}_\n",
        escape(&sub.name),
        sub.items,
        sub.price.to_string().replace(".", ",")
    );
    let mut keymap = InlineKeyboardMarkup::default();

    if ctx.has_right(Rule::EditSubscription) {
        keymap = keymap.append_row(Callback::Delete.btn_row("❌ Удалить"));
        keymap = keymap.append_row(Callback::EditPrice.btn_row("Изменить цену 💸"));
        keymap = keymap.append_row(Callback::EditItems.btn_row("Изменить количество занятий"));
        keymap = keymap.append_row(Callback::EditName.btn_row("Изменить название"));
    }

    if ctx.has_right(Rule::SellSubscription) {
        keymap = keymap.append_row(Callback::Sell.btn_row("🛒 Продать"));
    }

    Ok((msg, keymap))
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Delete,
    Sell,
    EditPrice,
    EditItems,
    EditName,
}
