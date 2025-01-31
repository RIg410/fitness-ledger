use std::str;

use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::user::fmt_come_from;
use model::statistics::marketing::ComeFrom;
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
        for come_from in ComeFrom::iter() {
            markup = markup.append_row(come_from.btn_row(fmt_come_from(come_from)));
        }

        ctx.bot.edit_origin(text, markup).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        let come_from: ComeFrom = calldata!(data);
        let request = ctx.ledger.requests.get(&mut ctx.session, self.id).await?;
        let old_come_from = if let Some(request) = request {
            request.come_from
        } else {
            ctx.bot.send_notification("Заявка не найдена").await;
            return Ok(Jmp::Back);
        };

        let comment = format!(
            "Изменен источник с {} на {}",
            fmt_come_from(old_come_from),
            fmt_come_from(come_from)
        );

        ctx.ledger
            .requests
            .update_come_from(&mut ctx.session, self.id, come_from, comment)
            .await?;
        ctx.bot.send_notification("Источник изменен").await;
        Ok(Jmp::Back)
    }
}
