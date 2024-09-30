pub mod history;
pub mod in_out;
pub mod operation;
pub mod reward;
pub mod stat;

use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use chrono::{Datelike as _, Local};
use eyre::Result;
use history::history_view;
use in_out::{InOut, Io};
use model::rights::Rule;
use reward::SelectCouch;
use serde::{Deserialize, Serialize};
use stat::{Stat, StatRange};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

#[derive(Default)]
pub struct FinanceView;

#[async_trait]
impl View for FinanceView {
    fn name(&self) -> &'static str {
        "FinView"
    }
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let text = "💰 Финансы:".to_string();
        let mut keymap = InlineKeyboardMarkup::default();

        if ctx.has_right(Rule::MakePayment) {
            keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
                "Оплатить 💳",
                Callback::Payment.to_data(),
            )]);
            keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
                "Ваплата ЗП 🎁",
                Callback::PayReward.to_data(),
            )]);
        }
        if ctx.has_right(Rule::MakeDeposit) {
            keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
                "Внести средства 🤑",
                Callback::Deposit.to_data(),
            )]);
        }

        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            "Общая статистика 📊",
            Callback::StatAll.to_data(),
        )]);
        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            "Статистика за месяц 📈",
            Callback::StatByMonth.to_data(),
        )]);

        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            "История 📜",
            Callback::History.to_data(),
        )]);
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
            Callback::PayReward => {
                ctx.ensure(Rule::MakePayment)?;
                Ok(SelectCouch.into())
            }
            Callback::History => {
                ctx.ensure(Rule::ViewFinance)?;
                Ok(Jmp::Next(history_view()))
            }
            Callback::StatAll => {
                ctx.ensure(Rule::ViewFinance)?;
                Ok(Stat::new(StatRange::Full).into())
            }
            Callback::StatByMonth => {
                ctx.ensure(Rule::ViewFinance)?;
                Ok(Stat::new(StatRange::Month(
                    Local::now().with_day(1).unwrap_or_default(),
                ))
                .into())
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    PayReward,
    Payment,
    Deposit,
    History,
    StatByMonth,
    StatAll,
}
