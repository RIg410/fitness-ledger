use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::day::fmt_dt;
use chrono::Local;
use eyre::Result;
use model::{
    training::{Training, TrainingId, TrainingStatus},
    user::User,
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    utils::markdown::escape,
};

use crate::view::{sign_out, sign_up};

pub struct FamilySignIn {
    id: TrainingId,
}

impl FamilySignIn {
    pub fn new(id: TrainingId) -> FamilySignIn {
        FamilySignIn { id }
    }
}

#[async_trait]
impl View for FamilySignIn {
    fn name(&self) -> &'static str {
        "FamilySignIn"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let training = ctx
            .ledger
            .calendar
            .get_training_by_id(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("Training not found"))?;

        let tr_status = training.status(Local::now());

        let msg = format!(
            "👨‍👩‍👧‍👦 Запись на тренировку *{}*\nв _{}_",
            escape(&training.name),
            fmt_dt(&training.get_slot().start_at())
        );

        let mut keymap = InlineKeyboardMarkup::default();

        keymap = keymap.append_row(make_row(&training, tr_status, &ctx.me, true));

        for child in ctx.me.family.children.iter() {
            keymap = keymap.append_row(make_row(&training, tr_status, child, false));
        }

        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Callback::SingIn(id) => {
                let id = ObjectId::from_bytes(id);
                sign_up(ctx, self.id, id).await?;
            }
            Callback::SignOut(id) => {
                let id = ObjectId::from_bytes(id);
                sign_out(ctx, self.id, id).await?;
            }
            Callback::None => {
                // do nothing
            }
        }
        Ok(Jmp::Stay)
    }
}

fn make_row(
    training: &Training,
    tr_status: TrainingStatus,
    user: &User,
    iam: bool,
) -> Vec<InlineKeyboardButton> {
    let can_sign_in = tr_status.can_sign_in();
    let can_sign_out = tr_status.can_sign_out();
    let name = if iam {
        "👤 Я"
    } else {
        &user.name.first_name
    };
    let signed = training.clients.contains(&user.id);

    let sign_callback = if signed {
        if can_sign_out {
            Callback::SignOut(user.id.bytes()).button("❌ Отменить запись")
        } else {
            Callback::None.button("Отмена не возможна")
        }
    } else if can_sign_in {
        Callback::SingIn(user.id.bytes()).button("✅ Записаться")
    } else {
        Callback::None.button("Запись закрыта")
    };

    let name_btn = if signed {
        Callback::None.button(format!("✅{}", name))
    } else {
        Callback::None.button(name.to_string())
    };

    vec![name_btn, sign_callback]
}

#[derive(Serialize, Deserialize)]
enum Callback {
    SingIn([u8; 12]),
    SignOut([u8; 12]),
    None,
}
