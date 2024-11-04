pub mod confirm;
pub mod create;
pub mod edit;
pub mod edit_requirement;
pub mod edit_type;
pub mod sell;
pub mod view;

use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use create::CreateSubscription;
use eyre::Result;
use model::rights::Rule;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    utils::markdown::escape,
};
use view::SubscriptionOption;

#[derive(Default)]
pub struct SubscriptionView;

#[async_trait]
impl View for SubscriptionView {
    fn name(&self) -> &'static str {
        "SubscriptionView"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let (msg, keymap) = render(ctx).await?;
        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, msg: &str) -> Result<Jmp, eyre::Error> {
        match calldata!(msg) {
            Callback::Select(id) => Ok(SubscriptionOption::new(ObjectId::from_bytes(id)).into()),
            Callback::CreateSubscription => {
                ctx.ensure(Rule::CreateSubscription)?;
                Ok(CreateSubscription::new().into())
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
    msg.push_str(&delimiter);
    msg.push_str("_Групповые абонементы:_\n");

    for subscription in &subscriptions {
        if !can_sell && !subscription.user_can_buy {
            continue;
        }
        if subscription.subscription_type.is_personal() {
            continue;
        }

        msg.push_str(&format!(
            "*{}* \\- _{}_р\n",
            escape(&subscription.name),
            subscription.price.to_string().replace(".", ",")
        ));

        if can_sell || ctx.has_right(Rule::BuySubscription) {
            keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
                subscription.name.clone(),
                Callback::Select(subscription.id.bytes()).to_data(),
            )]);
        }
    }
    msg.push_str(&delimiter);
    msg.push_str("_Индивидуальные абонементы:_\n");

    for subscription in &subscriptions {
        if !can_sell && !subscription.user_can_buy {
            continue;
        }
        if !subscription.subscription_type.is_personal() {
            continue;
        }

        msg.push_str(&format!(
            "*{}* \\- _{}_р\n",
            escape(&subscription.name),
            subscription.price.to_string().replace(".", ",")
        ));

        if can_sell || ctx.has_right(Rule::BuySubscription) {
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
    if ctx.has_right(Rule::CreateSubscription) {
        keymap = keymap.append_row(Callback::CreateSubscription.btn_row("🗒 Создать тариф"));
    }

    Ok((msg.to_string(), keymap))
}

fn can_show_subscription(
    subscription: &model::subscription::Subscription,
    ctx: &Context,
    seller: bool,
) -> bool {
    if seller {
        return true;
    }

    true
}

#[derive(Debug, Serialize, Deserialize)]
enum Callback {
    Select([u8; 12]),
    CreateSubscription,
}
