use std::str;

use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use model::statistics::source::Source;
use mongodb::bson::oid::ObjectId;
use teloxide::types::InlineKeyboardMarkup;

pub struct ChangeComeFrom {
    pub id: ObjectId,
}

#[async_trait]
impl View for ChangeComeFrom {
    fn name(&self) -> &'static str {
        "SetComeFrom"
    }

    async fn show(&mut self, ctx: &mut bot_core::context::Context) -> Result<(), eyre::Error> {
        let text = "Откуда пришел клиент?";

        let mut markup = InlineKeyboardMarkup::default();
        for come_from in Source::iter() {
            markup = markup.append_row(come_from.btn_row(come_from.name()));
        }

        ctx.bot.edit_origin(text, markup).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        let come_from: Source = calldata!(data);
        let request = ctx.ledger.requests.get(&mut ctx.session, self.id).await?;
        let old_come_from = if let Some(request) = request {
            request.come_from
        } else {
            ctx.bot.send_notification("Заявка не найдена").await;
            return Ok(Jmp::Back);
        };

        let comment = format!(
            "Изменен источник с {} на {}",
            old_come_from.name(),
            come_from.name()
        );

        ctx.ledger
            .requests
            .update_come_from(&mut ctx.session, self.id, come_from, comment)
            .await?;
        ctx.bot.send_notification("Источник изменен").await;
        Ok(Jmp::Back)
    }
}
