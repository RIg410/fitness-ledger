use async_trait::async_trait;
use bot_core::{context::Context, widget::View};
use mongodb::bson::oid::ObjectId;

pub struct ViewEmployee {
    id: ObjectId,
}

impl ViewEmployee {
    pub fn new(id: ObjectId) -> Self {
        Self { id }
    }
}

#[async_trait]
impl View for ViewEmployee {
    fn name(&self) -> &'static str {
        "ViewEmployee"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
       
        Ok(())
    }
}