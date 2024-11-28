use async_trait::async_trait;
use bot_core::{
    context::Context,
    widget::{Jmp, View},
};
use model::rights::Rule;
use mongodb::bson::oid::ObjectId;

pub struct AddMember {
    id: ObjectId,
}

impl AddMember {
    pub fn new(id: ObjectId) -> Self {
        AddMember { id }
    }
}

#[async_trait]
impl View for AddMember {
    fn name(&self) -> &'static str {
        "AddMember"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        ctx.ensure(Rule::EditFamily)?;

        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::EditFamily)?;

        Ok(Jmp::Stay)
    }
}
