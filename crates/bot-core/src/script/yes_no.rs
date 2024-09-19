use async_trait::async_trait;
use eyre::Error;

use crate::context::Context;

use super::{Dispatch, Stage};

#[async_trait]
pub trait StageYesNo<S>
where
    S: Send + Sync + 'static,
{
    async fn message(&self, ctx: &mut Context, state: &mut S) -> Result<String, Error>;
    async fn yes(&self, ctx: &mut Context, state: &mut S) -> Result<Dispatch<S>, Error>;

    fn back(&self) -> Option<Stage<S>> {
        None
    }
}
