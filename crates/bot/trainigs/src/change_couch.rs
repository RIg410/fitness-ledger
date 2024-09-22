use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use chrono::{DateTime, Local};
use eyre::Result;
use model::{rights::Rule, user::User};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub struct ChangeCouch {
    start_at: DateTime<Local>,
    all: bool,
}

impl ChangeCouch {
    pub fn new(start_at: DateTime<Local>, all: bool) -> ChangeCouch {
        ChangeCouch { start_at, all }
    }

    async fn change_couch(&self, ctx: &mut Context, id: ObjectId) -> Result<()> {
        ctx.ensure(Rule::EditSubscription)?;
        let training = ctx
            .ledger
            .calendar
            .get_training_by_start_at(&mut ctx.session, self.start_at)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        if training.is_processed {
            ctx.send_notification("Тренировка завершена\\. *Редактирование запрещено\\.*")
                .await?;
            return Ok(());
        }
        ctx.ledger
            .calendar
            .change_couch(
                &mut ctx.session,
                training.get_slot().start_at(),
                id,
                self.all,
            )
            .await?;
        Ok(())
    }
}

#[async_trait]
impl View for ChangeCouch {
    fn name(&self) -> &'static str {
        "ChangeCouch"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let msg = "Наши инструкторы ❤️";
        let mut keymap = InlineKeyboardMarkup::default();
        let instructs = ctx.ledger.users.instructors(&mut ctx.session).await?;

        for instruct in instructs {
            keymap = keymap.append_row(vec![render_button(&instruct)]);
        }

        ctx.edit_origin(msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::SelectCouch(id) => {
                let id: ObjectId = ObjectId::from_bytes(id);
                self.change_couch(ctx, id).await?;
                return Ok(Jmp::Back);
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    SelectCouch([u8; 12]),
}

fn render_button(user: &User) -> InlineKeyboardButton {
    Callback::SelectCouch(user.id.bytes()).button(format!(
        "💪 {} {}",
        user.name.first_name,
        user.name.last_name.clone().unwrap_or_default()
    ))
}
