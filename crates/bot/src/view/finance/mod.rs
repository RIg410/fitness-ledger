pub mod in_out;

use super::{menu::MainMenuItem, View};
use crate::{callback_data::Calldata, context::Context, state::Widget};
use async_trait::async_trait;
use eyre::Result;
use in_out::{InOut, Io};
use model::rights::Rule;
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Message};

#[derive(Default)]
pub struct FinanceView;

#[async_trait]
impl View for FinanceView {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let text = format!("💰 Финансы:");
        let mut keymap = InlineKeyboardMarkup::default();

        if ctx.has_right(Rule::MakePayment) {
            keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
                "Оплатить 💳",
                Callback::Payment.to_data(),
            )]);
        }
        if ctx.has_right(Rule::MakeDeposit) {
            keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
                "Внести средства 🤑",
                Callback::Deposit.to_data(),
            )]);
        }

        keymap = keymap.append_row(vec![MainMenuItem::Home.into()]);
        ctx.edit_origin(&text, keymap).await?;
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
            Callback::Payment => {
                ctx.ensure(Rule::MakePayment)?;
                let payment = InOut::new(Some(Box::new(FinanceView)), Io::Payment);
                Ok(Some(Box::new(payment)))
            }
            Callback::Deposit => {
                ctx.ensure(Rule::MakeDeposit)?;
                let payment = InOut::new(Some(Box::new(FinanceView)), Io::Deposit);
                Ok(Some(Box::new(payment)))
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Payment,
    Deposit,
}
