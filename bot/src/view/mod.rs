use crate::{context::Context, state::Widget};
use async_trait::async_trait;
use eyre::Result;
use teloxide::types::Message;

pub mod menu;
pub mod signup;

#[async_trait]
pub trait View {
    fn allow_unsigned_user(&self) -> bool {
        false
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error>;

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Option<Widget>, eyre::Error>;

    async fn handle_callback(
        &mut self,
        ctx: &mut Context,
        data: Option<&str>,
    ) -> Result<Option<Widget>, eyre::Error>;
}
