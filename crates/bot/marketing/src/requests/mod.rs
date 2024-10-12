use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use history::RequestHistory;
use model::rights::Rule;
use serde::{Deserialize, Serialize};
use teloxide::types::InlineKeyboardMarkup;

mod create;
mod history;

pub struct Requests;

#[async_trait]
impl View for Requests {
    fn name(&self) -> &'static str {
        "Requests"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        ctx.ensure(Rule::ViewRequestsHistory)?;

        let text = "Заявки 🈸";
        let mut keymap: InlineKeyboardMarkup = InlineKeyboardMarkup::default();

        if ctx.has_right(Rule::CreateRequest) {
            keymap = keymap.append_row(Calldata::Create.btn_row("Создать заявку"));
        }

        if ctx.has_right(Rule::ViewRequestsHistory) {
            keymap = keymap.append_row(Calldata::History.btn_row("История 🈸"));
        }

        ctx.bot.edit_origin(text, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        match calldata!(data) {
            Calldata::Create => {
                ctx.ensure(Rule::CreateRequest)?;
                Ok(create::SetPhone.into())
            }
            Calldata::History => {
                ctx.ensure(Rule::ViewRequestsHistory)?;
                Ok(RequestHistory::new().into())
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Calldata {
    Create,
    History,
}
