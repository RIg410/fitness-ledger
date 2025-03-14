use std::str::FromStr as _;

use async_trait::async_trait;
use bot_core::{
    context::Context,
    widget::{Jmp, View},
};
use eyre::Error;
use model::{decimal::Decimal, rights::Rule};
use mongodb::bson::oid::ObjectId;
use teloxide::{
    types::{InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

pub struct SetItemPrice {
    user_id: ObjectId,
    id: ObjectId,
}

impl SetItemPrice {
    pub fn new(user_id: ObjectId, id: ObjectId) -> Self {
        Self { user_id, id }
    }
}

#[async_trait]
impl View for SetItemPrice {
    fn name(&self) -> &'static str {
        "SetItemPrice"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), Error> {
        ctx.ensure(Rule::EditSubscription)?;
        let user = ctx.ledger.get_user(&mut ctx.session, self.user_id).await?;
        let user = user.payer()?;
        let subs = user.subscriptions();
        let sub = subs
            .iter()
            .find(|s| s.id == self.id)
            .ok_or_else(|| eyre::eyre!("Subscription not found"))?;

        let msg = format!(
            "*Выберите цену занятия*\nТекущая цена занятия: {}",
            escape(&sub.item_price().to_string())
        );

        ctx.edit_origin(&msg, InlineKeyboardMarkup::default())
            .await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        msg: &Message,
    ) -> Result<Jmp, eyre::Error> {
        ctx.delete_msg(msg.id).await?;
        if let Some(price) = msg.text() {
            let price = Decimal::from_str(price)?;
            ctx.ledger
                .users
                .set_subscription_item_price(&mut ctx.session, self.user_id, self.id, price)
                .await?;
        } else {
            ctx.send_msg("Введите цену занятия").await?;
        }

        Ok(Jmp::Stay)
    }
}
