use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::day::fmt_dt;
use chrono::{DateTime, Local};
use eyre::{bail, Result};
use ledger::calendar::SignOutError;
use model::rights::Rule;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{types::InlineKeyboardMarkup, utils::markdown::escape};

use crate::{
    add_client::AddClientView,
    client::{ClientView, Reason},
};

#[derive(Default)]
pub struct ClientsList {
    start_at: DateTime<Local>,
}

impl ClientsList {
    pub fn new(start_at: DateTime<Local>) -> Self {
        Self { start_at }
    }

    pub async fn view_user_profile(&mut self, id: ObjectId) -> Result<Jmp> {
        Ok(ClientView::new(id, self.start_at, Reason::RemoveClient).into())
    }

    pub async fn add_client(&mut self, ctx: &mut Context) -> Result<Jmp> {
        ctx.ensure(Rule::EditTrainingClientsList)?;
        Ok(AddClientView::new(self.start_at).into())
    }

    pub async fn delete_client(&mut self, ctx: &mut Context, id: ObjectId) -> Result<Jmp> {
        ctx.ensure(Rule::EditTrainingClientsList)?;

        let training = ctx
            .ledger
            .calendar
            .get_training_by_start_at(&mut ctx.session, self.start_at)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        if training.is_processed {
            ctx.send_notification("Тренировка завершена\\. *Редактирование запрещено\\.*")
                .await?;
            return Ok(Jmp::None);
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
                ctx.send_notification("Тренировка завершена\\. *Редактирование запрещено\\.*")
                    .await?;
            }
            Err(SignOutError::NotEnoughReservedBalance) => {
                ctx.send_notification("Не удалось удалить клиента\\. Нет резерва")
                    .await?;
            }
            Err(SignOutError::UserNotFound) => {
                bail!("User not found");
            }
            Err(SignOutError::ClientNotSignedUp) => {
                ctx.send_notification("Уже удален)").await?;
            }
            Err(SignOutError::Common(err)) => return Err(err),
        }

        Ok(Jmp::None)
    }
}

#[async_trait]
impl View for ClientsList {
    fn name(&self) -> &'static str {
        "ClientsList"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        if !ctx.is_couch() {
            bail!("Only couch can see client list");
        }

        let training = ctx
            .ledger
            .calendar
            .get_training_by_start_at(&mut ctx.session, self.start_at)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        let mut msg = format!(
            "📅 *{}*\n{}\n*Список участников:*\n",
            fmt_dt(&self.start_at),
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
                row.push(Callback::DeleteClient(user.id.bytes()).button("❌"))
            }
            keymap = keymap.append_row(row);
        }

        if ctx.has_right(Rule::EditTrainingClientsList) && !training.is_processed {
            keymap = keymap.append_row(vec![Callback::AddClient.button("Добавить 👤")]);
        }
        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::SelectClient(id) => self.view_user_profile(ObjectId::from_bytes(id)).await,
            Callback::AddClient => self.add_client(ctx).await,
            Callback::DeleteClient(id) => self.delete_client(ctx, ObjectId::from_bytes(id)).await,
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    SelectClient([u8; 12]),
    AddClient,
    DeleteClient([u8; 12]),
}
