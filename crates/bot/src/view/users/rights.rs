use async_trait::async_trait;
use model::{rights::Rule, user::User};
use serde::{Deserialize, Serialize};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Message};

use crate::{
    callback_data::Calldata as _, context::Context, state::Widget, view::menu::MainMenuItem,
};

use super::{profile::user_type, View};

#[derive(Default)]
pub struct UserRightsView {
    tg_id: i64,
    go_back: Option<Widget>,
}

impl UserRightsView {
    pub fn new(tg_id: i64, go_back: Option<Widget>) -> UserRightsView {
        UserRightsView { tg_id, go_back }
    }
}

#[async_trait]
impl View for UserRightsView {
    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let user = ctx
            .ledger
            .users
            .get_by_tg_id(&mut ctx.session, self.tg_id)
            .await?
            .ok_or_else(|| eyre::eyre!("Failed to load user"))?;
        let (text, markup) = render_user_rights(&user, self.go_back.is_some());
        ctx.edit_origin(&text, markup).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Option<Widget>, eyre::Error> {
        ctx.delete_msg(message.id).await?;
        Ok(None)
    }

    async fn handle_callback(
        &mut self,
        ctx: &mut Context,
        data: &str,
    ) -> Result<Option<Widget>, eyre::Error> {
        let cb = if let Some(cb) = Callback::from_data(data) {
            cb
        } else {
            return Ok(None);
        };

        match cb {
            Callback::Back => {
                if let Some(back) = self.go_back.take() {
                    return Ok(Some(back));
                } else {
                    Ok(None)
                }
            }
            Callback::EditRule(rule_id, is_active) => {
                ctx.ensure(Rule::EditUserRights)?;

                let rule = Rule::try_from(rule_id)?;
                ctx.ledger
                    .users
                    .edit_user_rule(&mut ctx.session, self.tg_id, rule, is_active)
                    .await?;
                ctx.reload_user().await?;
                self.show(ctx).await?;
                Ok(None)
            }
        }
    }
}

fn render_user_rights(user: &User, back: bool) -> (String, InlineKeyboardMarkup) {
    let mut msg = format!("{} 🔒Права:", user_type(user));
    let mut keymap = InlineKeyboardMarkup::default();

    if !user.rights.is_full() {
        for (rule, is_active) in user.rights.get_all_rules().iter() {
            keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
                format!("{} {}", rule.name(), if *is_active { "✅" } else { "❌" }),
                Callback::EditRule(rule.id(), !is_active).to_data(),
            )]);
        }
    } else {
        msg.push_str("\n\nПользователь имеет права администратора");
    }

    if back {
        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            "⬅️",
            Callback::Back.to_data(),
        )]);
    }

    keymap = keymap.append_row(vec![MainMenuItem::Home.into()]);
    (msg, keymap)
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Callback {
    Back,
    EditRule(u8, bool),
}
