use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    err::bassness_error,
    widget::{Jmp, View},
};
use bot_viewer::fmt_phone;
use model::{rights::Rule, user::sanitize_phone};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

pub struct AddMember {
    id: ObjectId,
    request: Option<String>,
}

impl AddMember {
    pub fn new(id: ObjectId) -> Self {
        AddMember { id, request: None }
    }
}

#[async_trait]
impl View for AddMember {
    fn name(&self) -> &'static str {
        "AddMember"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        ctx.ensure(Rule::EditFamily)?;

        let mut keymap = InlineKeyboardMarkup::default();
        let mut msg = "Добавление члена семьи\\. Введите номер телефона члена семьи или прочерк если пользователя нет\\.".to_string();
        if let Some(request) = &self.request {
            let request = sanitize_phone(request);
            if let Some(user) = ctx
                .ledger
                .users
                .find_by_phone(&mut ctx.session, &request)
                .await?
            {
                if user.family.exists() {
                    msg.push_str("\n\nПользователь уже состоит в семье");
                } else if user.id == self.id {
                    msg.push_str("\n\nНельзя добавить самого себя");
                } else {
                    msg.push_str(&format!(
                        "\n\nПользователь найден: *{}*",
                        escape(&user.name.first_name)
                    ));
                    keymap =
                        keymap.append_row(Calldata::AddMember(user.id.bytes()).btn_row("Добавить"));
                }
            } else {
                keymap = keymap.append_row(Calldata::CreateUser.btn_row("Создать пользователя"));
            }
        } else {
            keymap = keymap.append_row(Calldata::CreateUser.btn_row("Создать пользователя"));
        }

        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        msg: &Message,
    ) -> Result<Jmp, eyre::Error> {
        ctx.delete_msg(msg.id).await?;
        let text = msg.text().map(|s| s.to_string());
        if let Some(text) = text {
            if text == "-" {
                self.request = None;
            } else {
                self.request = Some(text);
            }
        } else {
            self.request = None;
        }

        Ok(Jmp::Stay)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::EditFamily)?;

        match calldata!(data) {
            Calldata::CreateUser => Ok(Jmp::Next(CreateUser::new(self.id).into())),
            Calldata::AddMember(id) => {
                Ok(AddMemberConfirm::new(self.id, ObjectId::from_bytes(id)).into())
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Calldata {
    AddMember([u8; 12]),
    CreateUser,
}

pub struct AddMemberConfirm {
    parent_id: ObjectId,
    child_id: ObjectId,
}

impl AddMemberConfirm {
    pub fn new(parent_id: ObjectId, child_id: ObjectId) -> Self {
        AddMemberConfirm {
            parent_id,
            child_id,
        }
    }
}

#[async_trait]
impl View for AddMemberConfirm {
    fn name(&self) -> &'static str {
        "AddMemberConfirm"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        ctx.ensure(Rule::EditFamily)?;

        let child = ctx.ledger.get_user(&mut ctx.session, self.child_id).await?;
        let msg = format!(
            "Вы уверены, что хотите добавить члена семьи {} {}?",
            escape(&child.name.first_name),
            fmt_phone(child.phone.as_deref())
        );
        let keymap = InlineKeyboardMarkup::default().append_row(vec![
            ConfirmCalldata::AddMember.button("✅ Добавить"),
            ConfirmCalldata::Cancel.button("❌ Отмена"),
        ]);

        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::EditFamily)?;

        match calldata!(data) {
            ConfirmCalldata::AddMember => {
                ctx.ledger
                    .users
                    .add_family_member(&mut ctx.session, self.parent_id, self.child_id)
                    .await?;
                ctx.send_notification("Член семьи добавлен").await;
                Ok(Jmp::Back)
            }
            ConfirmCalldata::Cancel => Ok(Jmp::Back),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ConfirmCalldata {
    AddMember,
    Cancel,
}

pub struct CreateUser {
    parent_id: ObjectId,
}

impl CreateUser {
    pub fn new(parent_id: ObjectId) -> Self {
        CreateUser { parent_id }
    }
}

#[async_trait]
impl View for CreateUser {
    fn name(&self) -> &'static str {
        "CreateUser"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        ctx.ensure(Rule::EditFamily)?;

        let msg = "Введите имя и фамилию нового члена семьи".to_string();
        ctx.edit_origin(&msg, InlineKeyboardMarkup::default())
            .await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        msg: &Message,
    ) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::EditFamily)?;
        ctx.delete_msg(msg.id).await?;
        let text = msg.text();

        if let Some(text) = text {
            if text.is_empty() {
                return Ok(Jmp::Stay);
            }

            let parts = text.split(" ").collect::<Vec<&str>>();
            let name = parts.first().map(|s| s.to_string()).unwrap_or_default();
            let surname = parts.get(1).map(|s| s.to_string());

            Ok(CreateMemberConfirm::new(self.parent_id, name, surname).into())
        } else {
            Ok(Jmp::Stay)
        }
    }
}

pub struct CreateMemberConfirm {
    parent_id: ObjectId,
    name: String,
    surname: Option<String>,
}

impl CreateMemberConfirm {
    pub fn new(parent_id: ObjectId, name: String, surname: Option<String>) -> Self {
        CreateMemberConfirm {
            parent_id,
            name,
            surname,
        }
    }
}

#[async_trait]
impl View for CreateMemberConfirm {
    fn name(&self) -> &'static str {
        "AddMemberConfirm"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        ctx.ensure(Rule::EditFamily)?;

        let msg = format!(
            "Вы уверены, что хотите добавить члена семьи {} {}?",
            escape(&self.name),
            escape(self.surname.as_deref().unwrap_or_default())
        );
        let keymap = InlineKeyboardMarkup::default().append_row(vec![
            ConfirmCalldata::AddMember.button("✅ Добавить"),
            ConfirmCalldata::Cancel.button("❌ Отмена"),
        ]);

        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::EditFamily)?;

        match calldata!(data) {
            ConfirmCalldata::AddMember => {
                let result = ctx
                    .ledger
                    .users
                    .create_family_member(
                        &mut ctx.session,
                        self.parent_id,
                        &self.name,
                        &self.surname,
                    )
                    .await;
                match result {
                    Ok(_) => {
                        ctx.send_notification("Член семьи добавлен").await;
                        Ok(Jmp::Back)
                    }
                    Err(err) => {
                        if let Some(msg) = bassness_error(ctx, &err).await? {
                            ctx.send_notification(&msg).await;
                            Ok(Jmp::Back)
                        } else {
                            Err(err.into())
                        }
                    }
                }
            }
            ConfirmCalldata::Cancel => Ok(Jmp::Back),
        }
    }
}
