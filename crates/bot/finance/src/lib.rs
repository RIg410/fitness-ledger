pub mod in_out;

use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use eyre::Result;
use in_out::{InOut, Io};
use model::rights::Rule;
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

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

        ctx.edit_origin(&text, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::Payment => {
                ctx.ensure(Rule::MakePayment)?;
                Ok(InOut::new(Io::Payment).into())
            }
            Callback::Deposit => {
                ctx.ensure(Rule::MakeDeposit)?;
                Ok(InOut::new(Io::Deposit).into())
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Payment,
    Deposit,
}
