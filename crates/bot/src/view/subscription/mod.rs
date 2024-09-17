pub mod confirm;
pub mod create;
pub mod free_sell;
pub mod sell;
pub mod view;
pub mod edit;
pub mod presell;

use super::View;
use crate::{callback_data::Calldata, context::Context, state::Widget};
use async_trait::async_trait;
use create::CreateSubscription;
use eyre::Result;
use free_sell::FeeSellView;
use model::rights::Rule;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{
    prelude::Requester as _,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};
use view::SubscriptionOption;

#[derive(Default)]
pub struct SubscriptionView;

#[async_trait]
impl View for SubscriptionView {
    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let (msg, keymap) = render(ctx).await?;
        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        msg: &Message,
    ) -> Result<Option<Widget>, eyre::Error> {
        ctx.bot.delete_message(msg.chat.id, msg.id).await?;
        Ok(None)
    }

    async fn handle_callback(
        &mut self,
        ctx: &mut Context,
        msg: &str,
    ) -> Result<Option<Widget>, eyre::Error> {
        let cb = if let Some(cb) = Callback::from_data(msg) {
            cb
        } else {
            return Ok(None);
        };
        match cb {
            Callback::Select(id) => {
                let view =
                    SubscriptionOption::new(ObjectId::from_bytes(id), Box::new(SubscriptionView));
                Ok(Some(Box::new(view)))
            }
            Callback::CreateSubscription => {
                ctx.ensure(Rule::CreateSubscription)?;
                let widget = Box::new(CreateSubscription::new(Box::new(SubscriptionView)));
                Ok(Some(widget))
            }
            Callback::FreeSell => {
                ctx.ensure(Rule::FreeSell)?;
                let widget = Box::new(FeeSellView::new(Box::new(SubscriptionView)));
                Ok(Some(widget))
            }
        }
    }
}

async fn render(ctx: &mut Context) -> Result<(String, InlineKeyboardMarkup)> {
    let mut msg = "💪 Тарифы:\n\n".to_string();

    let mut keymap = InlineKeyboardMarkup::default();
    let subscriptions = ctx.ledger.subscriptions.get_all(&mut ctx.session).await?;

    let can_sell = ctx.has_right(Rule::SellSubscription);

    let delimiter = escape("-------------------------\n");
    for subscription in subscriptions {
        msg.push_str(&format!(
            "*{}* \\- _{}_р\n",
            escape(&subscription.name),
            subscription.price.to_string().replace(".", ",")
        ));

        if can_sell {
            keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
                subscription.name.clone(),
                Callback::Select(subscription.id.bytes()).to_data(),
            )]);
        }
    }
    msg.push_str(&delimiter);
    if can_sell {
        msg.push_str("Для продажи тарифа нажмите на него");
    }
    if ctx.has_right(Rule::FreeSell) {
        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            "🤑 Свободная продажа",
            Callback::FreeSell.to_data(),
        )]);
    }

    if ctx.has_right(Rule::CreateSubscription) {
        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            "🗒 Создать тариф",
            Callback::CreateSubscription.to_data(),
        )]);
    }

    Ok((msg.to_string(), keymap))
}

#[derive(Debug, Serialize, Deserialize)]
enum Callback {
    Select([u8; 12]),
    CreateSubscription,
    FreeSell,
}
