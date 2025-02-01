use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::{fmt_phone, request::fmt_request};
use create::SetComeFrom;
use edit::{add_comment::AddComment, change_source::ChangeComeFrom, notification::AddNotification};
use history::RequestHistory;
use model::{rights::Rule, user::sanitize_phone};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardMarkup, Message};

mod create;
mod edit;
mod history;

pub struct Requests {
    pub phone: Option<String>,
    pub found: bool,
    pub id: Option<ObjectId>,
}

impl Requests {
    pub fn new(phone: Option<String>, found: bool, id: Option<ObjectId>) -> Self {
        Self { phone, found, id }
    }
}

impl Default for Requests {
    fn default() -> Self {
        Self {
            phone: None,
            found: false,
            id: None,
        }
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
            fmt_phone(Some(&self.phone.clone().unwrap_or_default()))
        );

        let mut keymap: InlineKeyboardMarkup = InlineKeyboardMarkup::default();

        if let Some(phone) = &self.phone {
            let request = ctx
                .ledger
                .requests
                .get_by_phone(&mut ctx.session, &sanitize_phone(phone))
                .await?;
            if let Some(request) = request.as_ref() {
                self.id = Some(request.id.clone());
                self.found = true;
                text.push_str(&fmt_request(request));
                keymap = keymap.append_row(Calldata::AddComment.btn_row("Добавить комментарий 📝"));
                keymap = keymap.append_row(Calldata::ChangeSource.btn_row("Изменить источник 🔄"));
                keymap = keymap.append_row(Calldata::Notification.btn_row("Напомнить 🛎"));
            } else {
                self.found = false;
                self.id = None;
                text.push_str("Заявка не найдена");
            }
        } else {
            if let Some(id) = self.id {
                let request = ctx.ledger.requests.get(&mut ctx.session, id).await?;
                if let Some(request) = request {
                    self.phone = Some(request.phone);
                    return self.show(ctx).await;
                }
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
                self.phone = Some(phone.to_string());
            }
        } else {
            self.phone = None;
        }

        Ok(Jmp::Stay)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        match calldata!(data) {
            Calldata::Create => {
                ctx.ensure(Rule::CreateRequest)?;
                if self.phone.as_ref().map(|p| p.len() > 5).unwrap_or_default() && !self.found {
                    Ok(SetComeFrom {
                        phone: self.phone.clone().unwrap_or_default(),
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
            Calldata::AddComment => {
                ctx.ensure(Rule::CreateRequest)?;
                if let Some(id) = self.id {
                    Ok(AddComment { id }.into())
                } else {
                    ctx.bot.send_notification("Заявка не найдена").await;
                    Ok(Jmp::Stay)
                }
            }
            Calldata::ChangeSource => {
                ctx.ensure(Rule::CreateRequest)?;
                if let Some(id) = self.id {
                    Ok(ChangeComeFrom { id }.into())
                } else {
                    ctx.bot.send_notification("Заявка не найдена").await;
                    Ok(Jmp::Stay)
                }
            }
            Calldata::Notification => {
                ctx.ensure(Rule::CreateRequest)?;
                if let Some(id) = self.id {
                    Ok(AddNotification { id }.into())
                } else {
                    ctx.bot.send_notification("Заявка не найдена").await;
                    Ok(Jmp::Stay)
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Calldata {
    Create,
    AddComment,
    ChangeSource,
    Notification,
    History,
}
