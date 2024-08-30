 use async_trait::async_trait;
    use teloxide::types::Message;

    use crate::{context::Context, state::Widget};

    use super::View;

    #[derive(Default)]
    pub struct UsersView {}

    #[async_trait]
    impl View for UsersView {
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
            data: Option<&str>,
        ) -> Result<Option<Widget>, eyre::Error> {
            Ok(None)
        }
    }