use super::View;
use crate::{context::Context, state::Widget};
use async_trait::async_trait;
use eyre::Result;
use teloxide::types::Message;

#[derive(Default)]
pub struct CouchingView;

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
        Ok(None)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Option<Widget>> {
        Ok(None)
    }
}
