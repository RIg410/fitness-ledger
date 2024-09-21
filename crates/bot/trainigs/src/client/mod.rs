use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::user::render_profile_msg;
use chrono::{DateTime, Local};
use eyre::{bail, Result};
use ledger::{calendar::SignOutError, SignUpError};
use model::{rights::Rule, training::Training};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::types::InlineKeyboardMarkup;

pub mod add;
pub mod list;

pub struct ClientView {
    id: ObjectId,
    training_id: DateTime<Local>,
    reason: Reason,
}

impl ClientView {
    pub fn new(id: ObjectId, training_id: DateTime<Local>, reason: Reason) -> ClientView {
        ClientView {
            id,
            reason,
            training_id,
        }
    }

    async fn training(&self, ctx: &mut Context) -> Result<Training> {
        let training = ctx
            .ledger
            .calendar
            .get_training_by_start_at(&mut ctx.session, self.training_id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        Ok(training)
    }

    async fn add_client(&self, ctx: &mut Context) -> Result<()> {
        let training = self.training(ctx).await?;

        if training.is_processed {
            ctx.send_notification("Тренировка завершена\\. *Редактирование запрещено\\.*")
                .await?;
            return Ok(());
        }
        let result = ctx
            .ledger
            .sign_up(&mut ctx.session, &training, self.id, true)
            .await;
        match result {
            Ok(_) => {}
            Err(SignUpError::ClientAlreadySignedUp) => {
                ctx.send_notification("Уже добавлен").await?;
            }
            Err(SignUpError::TrainingNotFound) => {
                bail!("Training not found");
            }
            Err(SignUpError::TrainingNotOpenToSignUp(_)) => {
                ctx.send_notification("Тренировка завершена\\. *Редактирование запрещено\\.*")
                    .await?;
            }
            Err(SignUpError::UserNotFound) => {
                bail!("User not found");
            }
            Err(SignUpError::Common(err)) => return Err(err),
            Err(SignUpError::NotEnoughBalance) => {
                ctx.send_notification("Не хватает баланса").await?;
            }
            Err(SignUpError::UserIsCouch) => {
                ctx.send_notification("Тренер не может записаться на тренировку")
                    .await?;
            }
        }
        Ok(())
    }

    async fn remove_client(&self, ctx: &mut Context) -> Result<()> {
        let training = self.training(ctx).await?;

        if training.is_processed {
            ctx.send_notification("Тренировка завершена\\. *Редактирование запрещено\\.*")
                .await?;
            return Ok(());
        }
        let result = ctx
            .ledger
            .sign_out(&mut ctx.session, &training, self.id, true)
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
        Ok(())
    }
}

#[async_trait]
impl View for ClientView {
    fn name(&self) -> &'static str {
        "ClientView"
    }
    
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let (msg, _) = render_profile_msg(ctx, self.id).await?;
        let mut keymap = InlineKeyboardMarkup::default();

        match self.reason {
            Reason::AddClient => {
                keymap = keymap.append_row(vec![Callback::AddClient.button("Добавить клиента 👤")]);
            }
            Reason::RemoveClient => {
                keymap =
                    keymap.append_row(vec![Callback::DeleteClient.button("Удалить клиента ❌")]);
            }
        }

        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        let cb = if let Some(cb) = Callback::from_data(data) {
            cb
        } else {
            return Ok(Jmp::None);
        };

        ctx.ensure(Rule::EditTrainingClientsList)?;

        match cb {
            Callback::GoBack => {}
            Callback::AddClient => {
                if let Reason::AddClient = self.reason {
                    self.add_client(ctx).await?;
                } else {
                    return Ok(Jmp::None);
                }
            }
            Callback::DeleteClient => {
                if let Reason::RemoveClient = self.reason {
                    self.remove_client(ctx).await?;
                } else {
                    return Ok(Jmp::None);
                }
            }
        };
        Ok(Jmp::Back)
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    GoBack,
    DeleteClient,
    AddClient,
}

#[derive(Clone, Copy)]
pub enum Reason {
    AddClient,
    RemoveClient,
}
