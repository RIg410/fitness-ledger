use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use eyre::Result;
use model::rights::Rule;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{types::InlineKeyboardMarkup, utils::markdown::escape};
use crate::profile::UserProfile;

pub mod add_member;

pub struct FamilyView {
    id: ObjectId,
}

impl FamilyView {
    pub fn new(id: ObjectId) -> FamilyView {
        FamilyView { id }
    }
}

#[async_trait]
impl View for FamilyView {
    fn name(&self) -> &'static str {
        "FamilyView"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let mut keymap = InlineKeyboardMarkup::default();
        let mut msg = "👨‍👩‍👧‍👦 Cемья\n".to_string();
        let family = &ctx.me.family;
        if let Some(payer) = family.payer.as_ref() {
            msg.push_str(&format!(
                "Глава семьи: _{}_\n",
                escape(&payer.name.first_name)
            ));
            if ctx.has_right(Rule::EditFamily) {
                keymap = keymap.append_row(
                    Calldata::GoToProfile(payer.id.bytes())
                        .btn_row(format!("👤 {}", payer.name.first_name)),
                );
            }
        }

        msg.push_str("Члены семьи:\n");
        for child in family.children.iter() {
            msg.push_str(&format!(" * _{}_\n", escape(&child.name.first_name)));
            if ctx.has_right(Rule::EditFamily) {
                keymap = keymap.append_row(vec![
                    Calldata::GoToProfile(child.id.bytes())
                        .button(format!("👤 {}", child.name.first_name)),
                    Calldata::RemoveChild(child.id.bytes()).button("❌"),
                ]);
            }
        }

        if ctx.has_right(Rule::EditFamily) {
            keymap = keymap.append_row(Calldata::AddChild.btn_row("➕ Добавить"));
        }
        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::EditFamily)?;
        match calldata!(data) {
            Calldata::GoToProfile(id) => {
                Ok(Jmp::Next(UserProfile::new(ObjectId::from_bytes(id)).into()))
            }
            Calldata::RemoveChild(id) => {
                Ok(ConfirmRemoveChild {
                    parent_id: self.id,
                    child_id: ObjectId::from_bytes(id),
                }
                .into())
            }
            Calldata::AddChild => {
                Ok(add_member::AddMember::new(self.id).into())
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Calldata {
    GoToProfile([u8; 12]),
    RemoveChild([u8; 12]),
    AddChild,
}

struct ConfirmRemoveChild {
    parent_id: ObjectId,
    child_id: ObjectId,
}

#[async_trait]
impl View for ConfirmRemoveChild {
    fn name(&self) -> &'static str {
        "ConfirmRemoveChild"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        ctx.ensure(Rule::EditFamily)?;
        let child = ctx.ledger.get_user(&mut ctx.session, self.child_id).await?;
        let msg = format!(
            "Вы уверены, что хотите удалить члена семьи {}?",
            child.name.first_name
        );

        let mut keymap = InlineKeyboardMarkup::default();
        keymap = keymap.append_row(vec![
            ConfirmRemoveChildCallback::Confirm.button("✅ Подтвердить"),
            ConfirmRemoveChildCallback::Cancel.button("❌ Отмена"),
        ]);
        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        ctx.ensure(Rule::EditFamily)?;
        match calldata!(data) {
            ConfirmRemoveChildCallback::Confirm => {
                ctx.ledger.users.remove_family_member(&mut ctx.session, self.parent_id, self.child_id)
                    .await?;
                ctx.send_notification("Член семьи удален").await?;
                Ok(Jmp::Back)
            }
            ConfirmRemoveChildCallback::Cancel => Ok(Jmp::Back),
        }
    }
}

#[derive(Serialize, Deserialize)]
enum ConfirmRemoveChildCallback {
    Confirm,
    Cancel,
}
