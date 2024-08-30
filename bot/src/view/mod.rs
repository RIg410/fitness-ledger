use crate::{context::Context, state::Widget};
use async_trait::async_trait;
use eyre::Result;
use teloxide::types::Message;

pub mod menu;
pub mod profile;
pub mod schedule;
pub mod signup;
pub mod subscription;
pub mod users;

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
        data: &str,
    ) -> Result<Option<Widget>, eyre::Error>;
}

#[allow(dead_code)]
mod template {
    use async_trait::async_trait;
    use teloxide::types::Message;

    use crate::{context::Context, state::Widget};

    use super::View;

    #[derive(Default)]
    pub struct UserProfile {}

    #[async_trait]
    impl View for UserProfile {
        async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
            Ok(())
        }

        async fn handle_message(
            &mut self,
            ctx: &mut Context,
            message: &Message,
        ) -> Result<Option<Widget>, eyre::Error> {
            Ok(None)
        }

        async fn handle_callback(
            &mut self,
            ctx: &mut Context,
            data: &str,
        ) -> Result<Option<Widget>, eyre::Error> {
            Ok(None)
        }
    }
}
