use super::View;
use crate::{callback_data::Calldata, context::Context, state::Widget};
use async_trait::async_trait;
use eyre::Result;
use programs_list::ProgramList;
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardMarkup, Message};

mod programs_list;

#[derive(Default)]
pub struct CouchingView;

#[async_trait]
impl View for CouchingView {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        ctx.ensure(model::rights::Rule::CouchingView)?;
        let msg = "Тренерская                 \n🏋️‍♂️";
        let mut keymap = InlineKeyboardMarkup::default();
        keymap = keymap.append_row(Callback::Training.btn_row("Программы 📋"));
        ctx.edit_origin(msg, keymap).await?;
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
            Callback::Training => Ok(Some(ProgramList::new(Some(CouchingView.boxed())).boxed())),
            Callback::Couch => Ok(None),
        }
    }

    fn take(&mut self) -> Widget {
        CouchingView.boxed()
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Training,
    Couch,
}
