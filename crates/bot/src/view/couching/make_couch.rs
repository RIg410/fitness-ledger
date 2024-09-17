use super::View;
use crate::{callback_data::Calldata as _, context::Context, state::Widget};
use async_trait::async_trait;
use eyre::Result;
use model::{rights::Rule, user::User};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Message};

pub struct CouchingView {
    id: ObjectId,
    go_back: Option<Widget>,
}

impl CouchingView {
    pub fn new(id: ObjectId, go_back: Option<Widget>) -> CouchingView {
        CouchingView { id, go_back }
    }
}

#[async_trait]
impl View for CouchingView {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
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
        Ok(None)
    }

    fn take(&mut self) -> Widget {
        CouchingView {
            id: self.id,
            go_back: self.go_back.take(),
        }
        .take()
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Back,
    SelectCouch([u8; 12]),
    MakeCouch,
}
