use async_trait::async_trait;
use teloxide::types::Message;

use crate::{context::Context, state::Widget};

use super::View;

#[derive(Default)]
pub struct SubscriptionView {}

#[async_trait]
impl View for SubscriptionView {
    async fn show(&mut self, _: &mut Context) -> Result<(), eyre::Error> {
        Ok(())
    }

    async fn handle_message(
        &mut self,
        _: &mut Context,
        _: &Message,
    ) -> Result<Option<Widget>, eyre::Error> {
        Ok(None)
    }

    async fn handle_callback(
        &mut self,
        _: &mut Context,
        _: &str,
    ) -> Result<Option<Widget>, eyre::Error> {
        Ok(None)
    }
}
