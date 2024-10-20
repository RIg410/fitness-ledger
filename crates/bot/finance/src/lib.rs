pub mod history;
pub mod in_out;
pub mod marketing;
pub mod operation;
pub mod pay_rent;
pub mod reward;
pub mod stat;
pub mod sub_rent;

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
use in_out::{Op, TreasuryOp};
use model::rights::Rule;
use reward::SelectCouch;
use serde::{Deserialize, Serialize};
use stat::{Stat, StatRange};
use teloxide::types::InlineKeyboardMarkup;

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
            keymap = keymap.append_row(Callback::Payment.btn_row("Оплатить 💳"));
            keymap = keymap.append_row(Callback::PayReward.btn_row("Ваплата ЗП 🎁"));
            keymap = keymap.append_row(Callback::PayRent.btn_row("Оплата аренды 🏠"));
            keymap = keymap.append_row(Callback::PayMarketing.btn_row("Оплата маркетинга 📈"));

            keymap = keymap.append_row(Callback::SubRent.btn_row("Субаренда 🏠"));
        }

        if ctx.has_right(Rule::MakeDeposit) {
            keymap = keymap.append_row(Callback::Deposit.btn_row("Внести средства 🤑"));
        }

        keymap = keymap.append_row(Callback::StatAll.btn_row("Общая статистика 📊"));
        keymap = keymap.append_row(Callback::StatByMonth.btn_row("Статистика за месяц 📈"));

        keymap = keymap.append_row(Callback::History.btn_row("История 📜"));
        ctx.edit_origin(&text, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::Payment => {
                ctx.ensure(Rule::MakePayment)?;
                Ok(TreasuryOp::new(Op::Payment).into())
            }
            Callback::Deposit => {
                ctx.ensure(Rule::MakeDeposit)?;
                Ok(TreasuryOp::new(Op::Deposit).into())
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
            Callback::PayRent => {
                ctx.ensure(Rule::MakePayment)?;
                Ok(pay_rent::PayRent.into())
            }
            Callback::PayMarketing => {
                ctx.ensure(Rule::MakePayment)?;
                Ok(marketing::PayRent.into())
            }
            Callback::SubRent => {
                ctx.ensure(Rule::MakePayment)?;
                Ok(sub_rent::TakeSubRent.into())
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    PayReward,
    PayRent,
    PayMarketing,
    Payment,

    Deposit,
    SubRent,

    History,
    StatByMonth,
    StatAll,
}
