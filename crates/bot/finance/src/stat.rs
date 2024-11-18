use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::user::fmt_come_from;
use chrono::Local;
use eyre::Result;
use model::rights::Rule;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use teloxide::{types::InlineKeyboardMarkup, utils::markdown::escape};
use time::range::Range;

pub struct Stat {
    range: Range,
}

impl Stat {
    pub fn new(range: Range) -> Self {
        Self { range }
    }
}

#[async_trait]
impl View for Stat {
    fn name(&self) -> &'static str {
        "stat"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        ctx.ensure(Rule::ViewFinance)?;
        let (from, to) = self.range.range();
        let stat = ctx
            .ledger
            .treasury
            .aggregate(&mut ctx.session, from, to)
            .await?;
        let mut text = format!(
            "📊Статистика с _{}_ по _{}_:\n",
            from.map(|f| f.format("%d\\.%m\\.%Y").to_string())
                .unwrap_or_else(|| "\\-".to_string()),
            to.map(|f| f.format("%d\\.%m\\.%Y").to_string())
                .unwrap_or_else(|| "\\-".to_string()),
        );

        writeln!(
            &mut text,
            "*Баланс*:_{}_",
            escape(&(stat.debit - stat.credit).to_string())
        )?;
        writeln!(&mut text, "*Поступления*:")?;
        writeln!(
            &mut text,
            "Проданно абониментов:_{}_ на сумму _{}_",
            stat.income.subscriptions.count,
            escape(&stat.income.subscriptions.sum.to_string())
        )?;
        writeln!(
            &mut text,
            "Другие поступления:_{}_",
            escape(&stat.income.other.sum.to_string())
        )?;

        writeln!(&mut text, "*Расходы*:")?;
        writeln!(
            &mut text,
            "Выплачено вознаграждений: _{}_",
            escape(&stat.outcome.rewards.sum.to_string())
        )?;
        writeln!(
            &mut text,
            "Другие расходы:_{}_",
            escape(&stat.outcome.other.sum.to_string())
        )?;

        writeln!(&mut text, "*Маркетинг*:")?;
        stat.outcome
            .marketing
            .iter()
            .try_for_each(|(come_from, sum)| {
                writeln!(
                    &mut text,
                    "_{}_: _{}_",
                    fmt_come_from(*come_from),
                    escape(&sum.sum.to_string())
                )
            })?;

        let mut keymap = InlineKeyboardMarkup::default();

        if let Range::Month(date) = self.range {
            let mut row = Vec::new();
            row.push(Calldata::PrevMonth.button("🔙"));

            if date < Local::now() {
                row.push(Calldata::NextMonth.button("🔜"));
            }

            keymap = keymap.append_row(row);
        }

        ctx.edit_origin(&text, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, _: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Calldata::NextMonth => {
                self.range = self.range.next_month();
            }
            Calldata::PrevMonth => {
                self.range = self.range.prev_month();
            }
        }
        Ok(Jmp::Stay)
    }
}

#[derive(Serialize, Deserialize)]
enum Calldata {
    NextMonth,
    PrevMonth,
}
