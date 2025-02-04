use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use model::{rights::Rule, statistics::source::Source, user::sanitize_phone};
use mongodb::bson::oid::ObjectId;
use teloxide::types::InlineKeyboardMarkup;

pub struct MarketingInfoView {
    id: ObjectId,
}

impl MarketingInfoView {
    pub fn new(id: ObjectId) -> Self {
        MarketingInfoView { id }
    }
}

#[async_trait]
impl View for MarketingInfoView {
    fn name(&self) -> &'static str {
        "MarketingInfoView"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        ctx.ensure(Rule::EditMarketingInfo)?;
        let user = ctx.ledger.get_user(&mut ctx.session, self.id).await?;
        let txt = format!("Источник : _{}_\n", user.come_from.name());
        let mut markup = InlineKeyboardMarkup::default();
        for come_from in Source::iter() {
            markup = markup.append_row(come_from.btn_row(come_from.name()));
        }
        ctx.edit_origin(&txt, markup).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::EditMarketingInfo)?;
        let come_from = calldata!(data);

        let user = ctx.ledger.get_user(&mut ctx.session, self.id).await?;
        if let Some(phone) = user.phone {
            let request = ctx
                .ledger
                .requests
                .get_by_phone(&mut ctx.session, &sanitize_phone(&phone))
                .await?;
            if let Some(mut request) = request {
                request.come_from = come_from;
                ctx.ledger
                    .requests
                    .update(&mut ctx.session, &request)
                    .await?;
            }
        }

        ctx.ledger
            .users
            .update_come_from(&mut ctx.session, self.id, come_from)
            .await?;
        Ok(Jmp::Stay)
    }
}
