use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use eyre::eyre;
use model::rights::Rule;
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardMarkup, Message};

pub struct ExtendSubscriptions;

#[async_trait]
impl View for ExtendSubscriptions {
    fn name(&self) -> &'static str {
        "ExtendSubscriptions"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        ctx.ensure(Rule::System)?;
        ctx.edit_origin("Введите количество дней", Default::default())
            .await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        msg: &Message,
    ) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::System)?;
        let days = msg.text().ok_or_else(|| eyre!("Invalid message"))?;
        let days = days.parse::<u32>()?;
        Ok(Jmp::Next(Confirm { days }.into()))
    }
}

pub struct Confirm {
    days: u32,
}

#[async_trait]
impl View for Confirm {
    fn name(&self) -> &'static str {
        "Confirm"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        ctx.ensure(Rule::System)?;

        let keymap = InlineKeyboardMarkup::default().append_row(vec![
            Calldata::Yes.button("✅ Да"),
            Calldata::No.button("❌ Нет"),
        ]);
        ctx.edit_origin(&format!("Продлить подписку на {} дней?", self.days), keymap)
            .await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::System)?;
        match calldata!(data) {
            Calldata::Yes => {
                ctx.ledger
                    .users
                    .extend_subscriptions(&mut ctx.session, self.days)
                    .await?;
                ctx.send_notification(&format!("Подписка продлена на {} дней", self.days))
                    .await;
            }
            Calldata::No => {
                ctx.send_notification("Отменено").await;
            }
        }
        Ok(Jmp::Stay)
    }
}

#[derive(Deserialize, Serialize)]
enum Calldata {
    Yes,
    No,
}
