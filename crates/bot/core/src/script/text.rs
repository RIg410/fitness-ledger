use crate::context::Context;
use async_trait::async_trait;
use eyre::{Error, Result};

use super::{Dispatch, Stage};

#[async_trait]
pub trait StageText<S>
where
    S: Send + Sync + 'static,
{
    async fn message(&self, ctx: &mut Context, state: &mut S) -> Result<String>;
    async fn handle_text(
        &self,
        ctx: &mut Context,
        state: &mut S,
        msg: &str,
    ) -> Result<Dispatch<S>, Error>;

    fn back(&self) -> Option<Stage<S>> {
        None
    }
}
