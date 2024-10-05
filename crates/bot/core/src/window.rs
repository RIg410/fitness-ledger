use crate::context::Context;
use async_trait::async_trait;
use std::future::Future;

#[async_trait]
pub trait View2 {
    async fn show(&mut self, ctx: &mut Context) -> Result<Layout, eyre::Error>;
}

pub struct Layout {
    pub txt: String,
    pub markup: Vec<Vec<Button>>,
}

pub struct Button {
    pub text: String,
    pub action: Box<dyn Fn() -> Box<dyn Future<Output = ()>> + Send + Sync + 'static>,
}
