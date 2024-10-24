use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::{fmt_phone, request::fmt_request};
use create::SetComeFrom;
use history::RequestHistory;
use model::{rights::Rule, user::sanitize_phone};
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardMarkup, Message};

mod create;
mod history;

pub struct Requests(pub Option<String>, bool);

impl Requests {
    pub fn new() -> Self {
        Self(None, false)
    }
}

#[async_trait]
impl View for Requests {
    fn name(&self) -> &'static str {
        "Requests"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        ctx.ensure(Rule::ViewMarketingInfo)?;

        let mut text = format!(
            "Заявки 🈸\nВведите номер телефона чтобы найти заявку: '{}'\n",
            fmt_phone(&self.0.clone().unwrap_or_default())
        );

        let mut keymap: InlineKeyboardMarkup = InlineKeyboardMarkup::default();

        if let Some(phone) = &self.0 {
            let request = ctx
                .ledger
                .requests
                .get_by_phone(&mut ctx.session, &sanitize_phone(phone))
                .await?;
            if let Some(request) = request.as_ref() {
                self.1 = true;
                text.push_str(&fmt_request(&request));
                keymap = keymap.append_row(Calldata::Edit.btn_row("Изменить заявку"));
            } else {
                self.1 = false;
                text.push_str("Заявка не найдена");
            }
        }

        if ctx.has_right(Rule::CreateRequest) {
            keymap = keymap.append_row(Calldata::Create.btn_row("Создать заявку"));
        }

        if ctx.has_right(Rule::RequestsHistory) {
            keymap = keymap.append_row(Calldata::History.btn_row("История 🈸"));
        }

        ctx.bot.edit_origin(&text, keymap).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        msg: &Message,
    ) -> Result<Jmp, eyre::Error> {
        ctx.delete_msg(msg.id).await?;
        if let Some(phone) = &msg.text() {
            if phone.len() > 5 {
                self.0 = Some(phone.to_string());
            }
        } else {
            self.0 = None;
        }

        Ok(Jmp::Stay)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        match calldata!(data) {
            Calldata::Create => {
                ctx.ensure(Rule::CreateRequest)?;
                if self.0.as_ref().map(|p| p.len() > 5).unwrap_or_default() && !self.1 {
                    Ok(SetComeFrom {
                        phone: self.0.clone().unwrap_or_default(),
                    }
                    .into())
                } else {
                    Ok(create::SetPhone.into())
                }
            }
            Calldata::History => {
                ctx.ensure(Rule::RequestsHistory)?;
                Ok(RequestHistory::new().into())
            }
            Calldata::Edit => {
                ctx.ensure(Rule::CreateRequest)?;
                Ok(SetComeFrom {
                    phone: self.0.clone().unwrap_or_default(),
                }
                .into())
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Calldata {
    Create,
    Edit,
    History,
}
