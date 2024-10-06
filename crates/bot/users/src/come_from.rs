use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::user::fmt_come_from;
use model::{rights::Rule, statistics::marketing::ComeFrom};
use serde::{Deserialize, Serialize};
use teloxide::types::InlineKeyboardMarkup;

pub struct MarketingInfoView {
    id: i64,
}

impl MarketingInfoView {
    pub fn new(id: i64) -> Self {
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
        let txt = format!(
            "Источник : _{}_\n",
            fmt_come_from(ctx, &user.come_from).await?
        );
        let mut markup = InlineKeyboardMarkup::default();
        markup = markup.append_row(Callback::Website {}.btn_row("Сайт"));
        markup = markup.append_row(Callback::Instagram {}.btn_row("Instagram"));
        markup = markup.append_row(Callback::VK {}.btn_row("VK"));
        markup = markup.append_row(Callback::YandexMap {}.btn_row("Яндекс.Карты"));
        markup = markup.append_row(Callback::DirectAdds {}.btn_row("Прямые рекламные каналы"));
        markup = markup.append_row(Callback::VkAdds {}.btn_row("Реклама ВК"));
        markup = markup.append_row(Callback::DoubleGIS {}.btn_row("2ГИС"));
        markup = markup.append_row(Callback::Unknown {}.btn_row("Неизвестно"));
        ctx.edit_origin(&txt, markup).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::EditMarketingInfo)?;
        let come_from = match calldata!(data) {
            Callback::Unknown => ComeFrom::Unknown {},
            Callback::Website => ComeFrom::Website {},
            Callback::Instagram => ComeFrom::Instagram {},
            Callback::VK => ComeFrom::VK {},
            Callback::YandexMap => ComeFrom::YandexMap {},
            Callback::DirectAdds => ComeFrom::DirectAdds {},
            Callback::VkAdds => ComeFrom::VkAdds {},
            Callback::DoubleGIS => ComeFrom::DoubleGIS {},
        };

        ctx.ledger
            .users
            .update_come_from(&mut ctx.session, self.id, come_from)
            .await?;
        Ok(Jmp::Stay)
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    Unknown,
    Website,
    Instagram,
    VK,
    YandexMap,
    DirectAdds,
    VkAdds,
    DoubleGIS,
}
