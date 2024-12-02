use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::day::fmt_dt;
use eyre::Result;
use model::{rights::Rule, training::TrainingId, user::User};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup},
    utils::markdown::escape,
};

pub struct ChangeCouch {
    id: TrainingId,
    all: bool,
}

impl ChangeCouch {
    pub fn new(id: TrainingId, all: bool) -> ChangeCouch {
        ChangeCouch { id, all }
    }

    async fn change_couch(&self, ctx: &mut Context, id: ObjectId) -> Result<()> {
        ctx.ensure(Rule::EditSchedule)?;
        let training = ctx
            .ledger
            .calendar
            .get_training_by_id(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;
        if training.is_processed {
            ctx.send_notification("Тренировка завершена\\. *Редактирование запрещено\\.*")
                .await?;
            return Ok(());
        }
        let old_couch = training.instructor;
        let new_couch = id;
        ctx.ledger
            .calendar
            .change_couch(&mut ctx.session, training.id(), id, self.all)
            .await?;

        ctx.send_notification("Тренер успешно изменен").await?;
        let old_couch = ctx.ledger.get_user(&mut ctx.session, old_couch).await?;
        let new_couch = ctx.ledger.get_user(&mut ctx.session, new_couch).await?;
        let msg = format!(
            "Произошла замена инструктора *{}* ➡️ *{}* на тренировке: *{}* в *{}*",
            escape(&old_couch.name.first_name),
            escape(&new_couch.name.first_name),
            escape(&training.name),
            fmt_dt(&training.get_slot().start_at())
        );
        ctx.send_notification_to(ChatId(old_couch.tg_id), &msg)
            .await?;
        ctx.send_notification_to(ChatId(new_couch.tg_id), &msg)
            .await?;
        for client in training.clients.iter() {
            let client = ctx.ledger.get_user(&mut ctx.session, *client).await?;
            ctx.send_notification_to(ChatId(client.tg_id), &msg).await?;
        }

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
