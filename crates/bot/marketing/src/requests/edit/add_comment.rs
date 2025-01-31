use async_trait::async_trait;
use bot_core::{
    context::Context,
    widget::{Jmp, View},
};
use mongodb::bson::oid::ObjectId;
use teloxide::types::Message;

pub struct AddComment {
    pub id: ObjectId,
}

#[async_trait]
impl View for AddComment {
    fn name(&self) -> &'static str {
        "AddComment"
    }

    async fn show(&mut self, ctx: &mut bot_core::context::Context) -> Result<(), eyre::Error> {
        let text = "Можно оставить комментарий или \\- если нечего добавить";
        ctx.bot.edit_origin(text, Default::default()).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        msg: &Message,
    ) -> Result<Jmp, eyre::Error> {
        ctx.bot.delete_msg(msg.id).await?;
        let comment = msg.text().unwrap_or_default().to_string();

        if comment == "-" {
            return Ok(Jmp::Back);
        }

        ctx.ledger
            .requests
            .add_comment(&mut ctx.session, self.id, comment)
            .await?;
        ctx.bot.send_notification("Комментарий добавлен").await;
        Ok(Jmp::Back)
    }
}
