use async_trait::async_trait;
use storage::user::{rights::Rule, User};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Message};

use crate::{context::Context, state::Widget, view::profile::user_type};

use super::View;

#[derive(Default)]
pub struct UserRightsView {
    tg_id: i64,
    go_back: Option<Widget>,
}

impl UserRightsView {
    pub fn new(tg_id: i64, go_back: Option<Widget>) -> UserRightsView {
        UserRightsView {
            tg_id: tg_id,
            go_back: go_back,
        }
    }
}

#[async_trait]
impl View for UserRightsView {
    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let user = ctx
            .ledger
            .get_user_by_tg_id(self.tg_id)
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
        match UserRightsCallback::try_from(data)? {
            UserRightsCallback::Back => {
                if let Some(back) = self.go_back.take() {
                    return Ok(Some(back));
                } else {
                    Ok(None)
                }
            }
            UserRightsCallback::EditRule(rule_id, is_active) => {
                ctx.ensure(Rule::EditUserRights)?;

                let rule = Rule::try_from(rule_id)?;
                ctx.ledger
                    .edit_user_rule(self.tg_id, rule, is_active)
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
    let mut keyboard = InlineKeyboardMarkup::default();

    if !user.rights.is_full() {
        for (rule, is_active) in user.rights.get_all_rules().iter() {
            keyboard = keyboard.append_row(vec![InlineKeyboardButton::callback(
                format!("{} {}", rule.name(), if *is_active { "✅" } else { "❌" }),
                UserRightsCallback::EditRule(rule.id(), !is_active).to_data(),
            )]);
        }
    } else {
        msg.push_str("\n\nПользователь имеет права администратора");
    }

    if back {
        keyboard = keyboard.append_row(vec![InlineKeyboardButton::callback(
            "⬅️",
            UserRightsCallback::Back.to_data(),
        )]);
    }

    (msg, keyboard)
}

pub enum UserRightsCallback {
    Back,
    EditRule(u8, bool),
}

impl UserRightsCallback {
    pub fn to_data(&self) -> String {
        match self {
            UserRightsCallback::Back => "rc_back".to_string(),
            UserRightsCallback::EditRule(id, is_active) => format!("rc_edit:{}:{}", id, is_active),
        }
    }
}

impl UserRightsCallback {
    pub fn try_from(value: &str) -> Result<Self, eyre::Error> {
        let parts: Vec<&str> = value.split(':').collect();
        match parts.as_slice() {
            ["rc_back"] => Ok(UserRightsCallback::Back),
            ["rc_edit", id, is_active] => Ok(UserRightsCallback::EditRule(
                id.parse()?,
                is_active.parse()?,
            )),
            _ => Err(eyre::eyre!("Invalid rights callback: {}", value)),
        }
    }
}
