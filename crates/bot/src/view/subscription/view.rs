use super::{
    sell::{Sell, SellView},
    View,
};
use crate::{callback_data::Calldata as _, context::Context, state::Widget};
use async_trait::async_trait;
use eyre::{Error, Result};
use model::rights::Rule;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

pub struct SubscriptionOption {
    go_back: Option<Widget>,
    id: ObjectId,
}

impl SubscriptionOption {
    pub fn new(id: ObjectId, go_back: Widget) -> SubscriptionOption {
        SubscriptionOption {
            go_back: Some(go_back),
            id,
        }
    }
}

#[async_trait]
impl View for SubscriptionOption {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let (txt, keymap) = render_sub(self.id, ctx).await?;
        ctx.edit_origin(&txt, keymap).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Option<Widget>> {
        ctx.delete_msg(message.id).await?;
        Ok(None)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Option<Widget>> {
        let cb = if let Some(cb) = Callback::from_data(data) {
            cb
        } else {
            return Ok(None);
        };
        match cb {
            Callback::Delete => {
                ctx.ensure(Rule::EditSubscription)?;
                ctx.ledger
                    .subscriptions
                    .delete(&mut ctx.session, self.id)
                    .await?;
                if let Some(widget) = self.go_back.take() {
                    return Ok(Some(widget));
                }
            }
            Callback::Sell => {
                ctx.ensure(Rule::SellSubscription)?;
                let back = Box::new(SubscriptionOption {
                    go_back: self.go_back.take(),
                    id: self.id,
                });
                let widget = Box::new(SellView::new(Sell::with_id(self.id), back));
                return Ok(Some(widget));
            }
            Callback::Back => {
                if let Some(widget) = self.go_back.take() {
                    return Ok(Some(widget));
                }
            }
        }
        Ok(None)
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
        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            "❌ Удалить",
            Callback::Delete.to_data(),
        )]);
    }

    if ctx.has_right(Rule::SellSubscription) {
        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            "🛒 Продать",
            Callback::Sell.to_data(),
        )]);
    }

    keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
        "🔙 Назад",
        Callback::Back.to_data(),
    )]);

    Ok((msg, keymap))
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Delete,
    Sell,
    Back,
}
