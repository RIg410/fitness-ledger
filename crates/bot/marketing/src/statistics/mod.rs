use ai::AiModel;
use async_trait::async_trait;
use bot_core::{
    callback_data::{CallbackDateTime, Calldata as _},
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::day::fmt_dt;
use chrono::Local;
use clients::ClientsStatistics;
use eyre::Error;
use eyre::Result;
use model::{rights::Rule, statistics::range::Range};
use serde::{Deserialize, Serialize};
use teloxide::types::InlineKeyboardMarkup;

mod budget;
mod clients;
mod instructors;
mod marketing;
mod view_ai;

pub struct StatisticsView {
    range: Range,
}

impl Default for StatisticsView {
    fn default() -> Self {
        Self {
            range: Range::Day(Local::now()),
        }
    }
}

impl StatisticsView {}

#[async_trait]
impl View for StatisticsView {
    fn name(&self) -> &'static str {
        "StatisticsView"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), Error> {
        ctx.ensure(Rule::ViewStatistics)?;

        let mut keymap = InlineKeyboardMarkup::default()
            .append_row(Calldata::Budget.btn_row("💰 Бюджет"))
            .append_row(Calldata::Instructor.btn_row("👨‍🏫 Инструкторы"))
            .append_row(Calldata::Clients.btn_row("👥 Клиенты"))
            .append_row(Calldata::Marketing.btn_row("📈 Маркетинг"));

        if ctx.has_right(Rule::AIStatistic) {
            keymap = keymap.append_row(Calldata::AI.btn_row("🤖 AI"));
        }

        let (from, to) = self.range.range()?;
        ctx.edit_origin(
            &format!("📊 Статистика \nс *{}* по *{}*", fmt_dt(&from), fmt_dt(&to)),
            keymap,
        )
        .await?;

        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        ctx.ensure(Rule::ViewStatistics)?;

        match calldata!(data) {
            Calldata::Budget => Ok(Jmp::Stay),
            Calldata::Instructor => Ok(Jmp::Stay),
            Calldata::Clients => {
                Ok(ClientsStatistics.into())
            }
            Calldata::Marketing => Ok(Jmp::Stay),
            Calldata::AI => {
                ctx.ensure(Rule::AIStatistic)?;
                let view = view_ai::AiView::new(AiModel::Gpt4oMini);
                return Ok(view.into());
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Calldata {
    Budget,
    Instructor,
    Clients,
    Marketing,
    AI,
}

#[derive(Serialize, Deserialize)]
enum RangeCalldata {
    Day(CallbackDateTime),
    Week(CallbackDateTime),
    Month(CallbackDateTime),
}

impl From<Range> for RangeCalldata {
    fn from(value: Range) -> Self {
        match value {
            Range::Day(date) => RangeCalldata::Day(CallbackDateTime::from(date)),
            Range::Week(date) => RangeCalldata::Week(CallbackDateTime::from(date)),
            Range::Month(date) => RangeCalldata::Month(CallbackDateTime::from(date)),
        }
    }
}

impl From<RangeCalldata> for Range {
    fn from(value: RangeCalldata) -> Self {
        match value {
            RangeCalldata::Day(date) => Range::Day(date.into()),
            RangeCalldata::Week(date) => Range::Week(date.into()),
            RangeCalldata::Month(date) => Range::Month(date.into()),
        }
    }
}
