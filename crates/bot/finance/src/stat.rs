use async_trait::async_trait;
use bot_core::{context::Context, widget::View};
use chrono::{DateTime, Local};
use eyre::Result;
use std::fmt::Write;
use teloxide::{types::InlineKeyboardMarkup, utils::markdown::escape};

pub struct Stat {
    pub from: DateTime<Local>,
    pub to: DateTime<Local>,
}

impl Stat {
    pub fn new(from: DateTime<Local>, to: DateTime<Local>) -> Self {
        Self { from, to }
    }
}

#[async_trait]
impl View for Stat {
    fn name(&self) -> &'static str {
        "stat"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let stat = ctx
            .ledger
            .treasury
            .aggregate(&mut ctx.session, self.from, self.to)
            .await?;
        let mut text = format!(
            "📊Статистика с _{}_ по _{}_:\n",
            self.from.format("%d\\.%m\\.%Y"),
            self.to.format("%d\\.%m\\.%Y")
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

        ctx.edit_origin(&text, InlineKeyboardMarkup::default())
            .await?;
        Ok(())
    }
}
