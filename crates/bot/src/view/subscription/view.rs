use super::{
    edit::{EditSubscription, EditType},
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
    types::{InlineKeyboardMarkup, Message},
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

    fn as_back(&mut self) -> Widget {
        SubscriptionOption {
            go_back: self.go_back.take(),
            id: self.id,
        }
        .boxed()
    }

    async fn edit(&mut self, tp: EditType) -> Result<Option<Widget>> {
        Ok(Some(
            EditSubscription::new(self.id, tp, Some(self.as_back())).boxed(),
        ))
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
                let widget = SellView::new(Sell::with_id(self.id), self.as_back()).boxed();
                return Ok(Some(widget));
            }
            Callback::Back => {
                if let Some(widget) = self.go_back.take() {
                    return Ok(Some(widget));
                }
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
        keymap = keymap.append_row(vec![Callback::Delete.button("❌ Удалить")]);
        keymap = keymap.append_row(vec![Callback::EditPrice.button("Изменить цену 💸")]);
        keymap = keymap.append_row(vec![
            Callback::EditItems.button("Изменить количество занятий")
        ]);
        keymap = keymap.append_row(vec![Callback::EditName.button("Изменить название")]);
    }

    if ctx.has_right(Rule::SellSubscription) {
        keymap = keymap.append_row(vec![Callback::Sell.button("🛒 Продать")]);
    }

    keymap = keymap.append_row(vec![Callback::Back.button("🔙 Назад")]);

    Ok((msg, keymap))
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Delete,
    Sell,
    Back,
    EditPrice,
    EditItems,
    EditName,
}
