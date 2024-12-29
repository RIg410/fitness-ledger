use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::error::notify;
use model::user::{rate::EmployeeRole, sanitize_phone};
use mongodb::bson::oid::ObjectId;
use teloxide::types::{InlineKeyboardMarkup, Message};

pub struct MakeEmployee {}

impl Default for MakeEmployee {
    fn default() -> Self {
        Self::new()
    }
}

impl MakeEmployee {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl View for MakeEmployee {
    fn name(&self) -> &'static str {
        "MakeEmployee"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let msg = "Введите номер телефона нового сотрудника:";
        let keymap = InlineKeyboardMarkup::default();
        ctx.edit_origin(msg, keymap).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Jmp, eyre::Error> {
        ctx.delete_msg(message.id).await?;
        let phone = if let Some(phone) = message.text() {
            sanitize_phone(phone)
        } else {
            ctx.send_notification("Номер телефона не найден").await?;
            return Ok(Jmp::Stay);
        };

        let user = ctx
            .ledger
            .users
            .find_by_phone(&mut ctx.session, &phone)
            .await?;

        Ok(if let Some(user) = user {
            if user.employee.is_some() {
                ctx.send_notification("Пользователь уже является сотрудником")
                    .await?;
                Jmp::Stay
            } else {
                Jmp::Next(EmployeeDescription { user_id: user.id }.into())
            }
        } else {
            ctx.send_notification("Пользователь не найден").await?;
            Jmp::Stay
        })
    }
}

pub struct EmployeeDescription {
    user_id: ObjectId,
}

#[async_trait]
impl View for EmployeeDescription {
    fn name(&self) -> &'static str {
        "EmployeeDescription"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let msg = "Введите описание нового сотрудника:";
        let keymap = InlineKeyboardMarkup::default();
        ctx.edit_origin(msg, keymap).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Jmp, eyre::Error> {
        ctx.delete_msg(message.id).await?;
        Ok(if let Some(description) = message.text() {
            Jmp::Next(
                EmployeeRoleView {
                    user_id: self.user_id,
                    description: description.to_string(),
                }
                .into(),
            )
        } else {
            ctx.send_notification("Описание не найдено").await?;
            Jmp::Stay
        })
    }
}

pub struct EmployeeRoleView {
    user_id: ObjectId,
    description: String,
}

#[async_trait]
impl View for EmployeeRoleView {
    fn name(&self) -> &'static str {
        "EmployeeRole"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let msg = "Выберите роль нового сотрудника:";
        let mut keymap = InlineKeyboardMarkup::default();
        keymap = keymap.append_row(EmployeeRole::Manager.btn_row("Менеджер"));
        keymap = keymap.append_row(EmployeeRole::Couch.btn_row("Тренер"));
        keymap = keymap.append_row(EmployeeRole::Admin.btn_row("Администратор"));
        ctx.edit_origin(msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        let role: EmployeeRole = calldata!(data);
        notify(
            "Ошибка добавления сотрудника",
            ctx.ledger
                .users
                .make_user_employee(
                    &mut ctx.session,
                    self.user_id,
                    self.description.clone(),
                    vec![],
                    role,
                )
                .await,
            "Сотрудник добавлен",
            ctx,
        )
        .await?;

        Ok(Jmp::Home)
    }
}
