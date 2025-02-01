use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use serde::{Deserialize, Serialize};
use teloxide::types::InlineKeyboardMarkup;

pub mod requests;
mod statistics;

#[derive(Default)]
pub struct Marketing {}

#[async_trait]
impl View for Marketing {
    fn name(&self) -> &'static str {
        "Marketing"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        ctx.ensure(model::rights::Rule::ViewMarketingInfo)?;
        let text = "Маркетинг🚀";
        let mut keymap = InlineKeyboardMarkup::default();

        if ctx.has_right(model::rights::Rule::ViewMarketingInfo) {
            keymap = keymap.append_row(Calldata::Request.btn_row("Заявки 🈸"));
        }
        if ctx.has_right(model::rights::Rule::ViewStatistics) {
            keymap = keymap.append_row(Calldata::Statistics.btn_row("Статистика 📊"));
        }

        ctx.bot.edit_origin(text, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        match calldata!(data) {
            Calldata::Request => {
                ctx.ensure(model::rights::Rule::ViewMarketingInfo)?;
                Ok(requests::Requests::default().into())
            }
            Calldata::Statistics => {
                ctx.ensure(model::rights::Rule::ViewStatistics)?;
                Ok(statistics::StatisticsView::default().into())
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Calldata {
    Request,
    Statistics,
}
