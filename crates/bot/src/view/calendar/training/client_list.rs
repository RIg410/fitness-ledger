use super::{
    add_client::AddClientView,
    client::{ClientView, Reason},
    View,
};
use crate::{callback_data::Calldata, context::Context, state::Widget, view::menu::MainMenuItem};
use async_trait::async_trait;
use chrono::{DateTime, Local};
use eyre::{bail, Result};
use ledger::calendar::SignOutError;
use model::rights::Rule;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

#[derive(Default)]
pub struct ClientList {
    start_at: DateTime<Local>,
    go_back: Option<Widget>,
}

impl ClientList {
    pub fn new(start_at: DateTime<Local>, go_back: Option<Widget>) -> Self {
        Self { start_at, go_back }
    }

    pub fn go_back(&mut self, _: &mut Context) -> Result<Option<Widget>> {
        Ok(self.go_back.take())
    }

    pub async fn view_user_profile(&mut self, id: ObjectId) -> Result<Option<Widget>> {
        let back = ClientList::new(self.start_at, self.go_back.take()).boxed();
        let view = ClientView::new(id, self.start_at, Reason::RemoveClient, Some(back)).boxed();
        Ok(Some(view))
    }

    pub async fn add_client(&mut self, ctx: &mut Context) -> Result<Option<Widget>> {
        ctx.ensure(Rule::EditTrainingClientsList)?;
        let back = ClientList::new(self.start_at, self.go_back.take()).boxed();
        let view = AddClientView::new(self.start_at, back).boxed();
        Ok(Some(view))
    }

    pub async fn delete_client(
        &mut self,
        ctx: &mut Context,
        id: ObjectId,
    ) -> Result<Option<Widget>> {
        ctx.ensure(Rule::EditTrainingClientsList)?;

        let training = ctx
            .ledger
            .calendar
            .get_training_by_start_at(&mut ctx.session, self.start_at)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        if training.is_processed {
            ctx.send_err("Тренировка завершена\\. *Редактирование запрещено\\.*")
                .await?;
            self.show(ctx).await?;
            return Ok(None);
        }
        let result = ctx
            .ledger
            .sign_out(&mut ctx.session, &training, id, true)
            .await;
        match result {
            Ok(_) => {}
            Err(SignOutError::TrainingNotFound) => {
                bail!("Training not found");
            }
            Err(SignOutError::TrainingNotOpenToSignOut) => {
                ctx.send_err("Тренировка завершена\\. *Редактирование запрещено\\.*")
                    .await?;
            }
            Err(SignOutError::NotEnoughReservedBalance) => {
                ctx.send_err("Не удалось удалить клиента\\. Нет резерва")
                    .await?;
            }
            Err(SignOutError::UserNotFound) => {
                bail!("User not found");
            }
            Err(SignOutError::ClientNotSignedUp) => {
                ctx.send_err("Уже удален)").await?;
            }
            Err(SignOutError::Common(err)) => return Err(err),
        }

        self.show(ctx).await?;
        Ok(None)
    }
}

#[async_trait]
impl View for ClientList {
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        ctx.ensure(Rule::Train)?;
        let training = ctx
            .ledger
            .calendar
            .get_training_by_start_at(&mut ctx.session, self.start_at)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        let mut msg = format!(
            "📅 *{}*\n{}\n*Список участников:*\n",
            self.start_at.format("%d\\.%m\\.%Y %H:%M"),
            escape(&training.name)
        );
        if training.is_processed {
            msg.push_str("Тренировка завершена\\. *Редактирование запрещено\\.*");
        }

        let mut keymap = InlineKeyboardMarkup::default();
        for client in &training.clients {
            let user = ctx
                .ledger
                .users
                .get(&mut ctx.session, *client)
                .await?
                .ok_or_else(|| eyre::eyre!("User not found"))?;
            let user_name = format!(
                "{} {}",
                user.name.first_name,
                user.name.tg_user_name.unwrap_or_default()
            );
            let mut row = Vec::with_capacity(2);
            row.push(Callback::SelectClient(user.id.bytes()).button(format!("👤 {}", user_name)));
            if ctx.has_right(Rule::EditTrainingClientsList) && !training.is_processed {
                row.push(Callback::DeleteClient(user.id.bytes()).button("❌".to_string()))
            }
            keymap = keymap.append_row(row);
        }

        if ctx.has_right(Rule::EditTrainingClientsList) && !training.is_processed {
            keymap = keymap.append_row(vec![Callback::AddClient.button("Добавить 👤".to_string())]);
        }

        if self.go_back.is_some() {
            keymap = keymap.append_row(vec![Callback::Back.button("⬅️ Назад".to_string())]);
        }

        keymap = keymap.append_row(vec![MainMenuItem::Home.into()]);
        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Option<Widget>> {
        ctx.delete_msg(message.id).await?;
        Ok(None)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Option<Widget>> {
        let cb = if let Some(data) = Callback::from_data(data) {
            data
        } else {
            return Ok(None);
        };
        match cb {
            Callback::SelectClient(id) => self.view_user_profile(ObjectId::from_bytes(id)).await,
            Callback::AddClient => self.add_client(ctx).await,
            Callback::DeleteClient(id) => self.delete_client(ctx, ObjectId::from_bytes(id)).await,
            Callback::Back => self.go_back(ctx),
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    SelectClient([u8; 12]),
    AddClient,
    DeleteClient([u8; 12]),
    Back,
}
