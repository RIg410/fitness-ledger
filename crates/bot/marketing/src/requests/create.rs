use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::{fmt_phone, user::fmt_come_from};
use model::{rights::Rule, statistics::marketing::ComeFrom};
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

use crate::Marketing;

pub struct SetPhone;

#[async_trait]
impl View for SetPhone {
    fn name(&self) -> &'static str {
        "SetPhone"
    }

    async fn show(&mut self, ctx: &mut bot_core::context::Context) -> Result<(), eyre::Error> {
        let text = "Введите номер телефона";
        ctx.bot.edit_origin(text, Default::default()).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        msg: &Message,
    ) -> Result<Jmp, eyre::Error> {
        ctx.bot.delete_msg(msg.id).await?;
        let mut phone = msg.text().unwrap_or_default().to_string();
        if phone.is_empty() {
            return Ok(Jmp::Stay);
        }

        if phone.starts_with("8") {
            phone = "7".to_string() + &phone[1..];
        }

        Ok(Jmp::Goto(SetComeFrom { phone }.into()))
    }
}

pub struct SetComeFrom {
    phone: String,
}

#[async_trait]
impl View for SetComeFrom {
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

    async fn handle_callback(&mut self, _: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        let come_from: ComeFrom = calldata!(data);
        Ok(Jmp::Goto(
            SetDescription {
                phone: self.phone.clone(),
                come_from,
            }
            .into(),
        ))
    }
}

pub struct SetDescription {
    phone: String,
    come_from: ComeFrom,
}

#[async_trait]
impl View for SetDescription {
    fn name(&self) -> &'static str {
        "SetDescription"
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
        Ok(Jmp::Goto(
            SetName {
                phone: self.phone.clone(),
                come_from: self.come_from,
                comment: comment.clone(),
            }
            .into(),
        ))
    }
}

pub struct SetName {
    phone: String,
    come_from: ComeFrom,
    comment: String,
}

#[async_trait]
impl View for SetName {
    fn name(&self) -> &'static str {
        "SetName"
    }

    async fn show(&mut self, ctx: &mut bot_core::context::Context) -> Result<(), eyre::Error> {
        let text = "Введите имя и фамилию";
        ctx.bot.edit_origin(text, Default::default()).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        msg: &Message,
    ) -> Result<Jmp, eyre::Error> {
        ctx.bot.delete_msg(msg.id).await?;
        let name = msg.text().unwrap_or_default();
        let parts: Vec<_> = name.split(' ').collect();
        let first_name = parts.get(0).map(|s| s.to_string());
        let last_name = parts.get(1).map(|s| s.to_string());
        Ok(Jmp::Goto(
            Comfirm {
                phone: self.phone.clone(),
                come_from: self.come_from,
                comment: self.comment.clone(),
                first_name,
                last_name,
            }
            .into(),
        ))
    }
}

pub struct Comfirm {
    phone: String,
    come_from: ComeFrom,
    comment: String,
    first_name: Option<String>,
    last_name: Option<String>,
}

#[async_trait]
impl View for Comfirm {
    fn name(&self) -> &'static str {
        "Comfirm"
    }

    async fn show(&mut self, ctx: &mut bot_core::context::Context) -> Result<(), eyre::Error> {
        let text = format!(
            "Все верно?:\n\
            Телефон: *{}*\n\
            Откуда пришел: *{}*\n\
            Комментарий: *{}*\n",
            fmt_phone(&self.phone),
            fmt_come_from(self.come_from),
            escape(&self.comment)
        );
        let mut markup = InlineKeyboardMarkup::default();
        markup = markup.append_row(vec![
            Calldata::Yes.button("✅Да"),
            Calldata::No.button("❌Нет"),
        ]);
        ctx.bot.edit_origin(&text, markup).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        match calldata!(data) {
            Calldata::Yes => {
                ctx.ensure(Rule::CreateRequest)?;
                ctx.ledger
                    .create_request(
                        &mut ctx.session,
                        self.phone.clone(),
                        self.come_from,
                        self.comment.clone(),
                        self.first_name.clone(),
                        self.last_name.clone(),
                    )
                    .await?;
                ctx.send_msg("Заявка создана").await?;
                Ok(Jmp::Goto(Marketing {}.into()))
            }
            Calldata::No => Ok(Jmp::Goto(Marketing {}.into())),
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Calldata {
    Yes,
    No,
}
