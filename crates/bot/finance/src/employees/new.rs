use async_trait::async_trait;
use bot_core::{context::Context, widget::View};

pub struct MakeEmployee {}

impl MakeEmployee {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl View for MakeEmployee {
    fn name(&self) -> &'static str {
        "MakeEmployee"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        
        Ok(())
    }
}
