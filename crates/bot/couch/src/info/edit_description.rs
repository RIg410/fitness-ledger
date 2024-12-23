use async_trait::async_trait;
use bot_core::{
    context::Context,
    script::{text::StageText, Dispatch, Stage},
};
use eyre::Error;

use super::{CouchInfo, State};

pub struct CouchDescription;

#[async_trait]
impl StageText<State> for CouchDescription {
    async fn message(&self, _: &mut Context, _: &mut State) -> Result<String, eyre::Error> {
        Ok("Введите описание 📝".to_string())
    }

    async fn handle_text(
        &self,
        ctx: &mut Context,
        state: &mut State,
        query: &str,
    ) -> Result<Dispatch<State>, Error> {
        ctx.ledger
            .users
            .update_employee_description(&mut ctx.session, state.id, query.to_string())
            .await?;
        Ok(Dispatch::Stage(Stage::list(CouchInfo)))
    }

    fn back(&self) -> Option<Stage<State>> {
        Some(Stage::list(CouchInfo))
    }
}
