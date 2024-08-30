use std::mem;

use async_trait::async_trait;
use log::warn;
use storage::user::{rights::Rule, User};
use teloxide::{
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

use crate::{
    context::Context,
    state::Widget,
    view::{profile::user_type, View},
};

use super::rights::UserRightsView;

pub struct UserProfile {
    tg_id: i64,
    go_back: Option<Widget>,
}

impl UserProfile {
    pub fn new(tg_id: i64, go_back: Option<Widget>) -> UserProfile {
        UserProfile { tg_id, go_back }
    }
}

#[async_trait]
impl View for UserProfile {
    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let (msg, keymap) = render_user_profile(&ctx, &ctx.me, self.go_back.is_some());
        ctx.edit_origin(&msg, keymap).await?;
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
        let cb = UserCallback::try_from(data)?;

        match cb {
            UserCallback::Back => {
                if let Some(back) = self.go_back.take() {
                    return Ok(Some(back));
                } else {
                    warn!("Attempt to go back");
                    Ok(None)
                }
            }
            UserCallback::BlockUnblock => {
                ctx.ensure(Rule::BlockUser)?;
                let user = ctx
                    .ledger
                    .get_user_by_tg_id(self.tg_id)
                    .await?
                    .ok_or_else(|| eyre::eyre!("User not found"))?;
                ctx.ledger.block_user(self.tg_id, !user.is_active).await?;
                ctx.reload_user().await?;
                self.show(ctx).await?;
                Ok(None)
            }
            UserCallback::Edit => Ok(None),
            UserCallback::EditRights => {
                ctx.ensure(Rule::EditUserRights)?;
                let mut new_user_new = UserProfile::new(0, None);
                mem::swap(self, &mut new_user_new);
                Ok(Some(Box::new(UserRightsView::new(
                    new_user_new.tg_id,
                    Some(Box::new(new_user_new)),
                ))))
            }
        }
    }
}

fn render_user_profile(ctx: &Context, user: &User, back: bool) -> (String, InlineKeyboardMarkup) {
    let empty = "?".to_string();
    let msg = format!(
        "
    {} Пользователь : _{}_
        Имя : _{}_
        Телефон : _{}_
        Дата рождения : _{}_
        ➖➖➖➖➖➖➖➖➖➖
        *Баланс : _{}_ занятий*
        ➖➖➖➖➖➖➖➖➖➖
    ",
        user_type(user),
        escape(user.name.tg_user_name.as_ref().unwrap_or_else(|| &empty)),
        escape(&user.name.first_name),
        escape(&user.phone),
        escape(
            &user
                .birthday
                .as_ref()
                .map(|d| d.format("%d.%m.%Y").to_string())
                .unwrap_or_else(|| empty.clone())
        ),
        user.balance
    );
    let mut markup = InlineKeyboardMarkup::default();
    if ctx.has_right(Rule::BlockUser) && ctx.me.tg_id != user.tg_id {
        markup = markup.append_row(vec![InlineKeyboardButton::callback(
            if user.is_active {
                "❌ Заблокировать"
            } else {
                "✅ Разблокировать"
            },
            UserCallback::BlockUnblock.to_data(),
        )]);
    }

    if ctx.has_right(Rule::EditUserInfo) {
        markup = markup.append_row(vec![InlineKeyboardButton::callback(
            "✍️ Редактировать",
            UserCallback::Edit.to_data(),
        )]);
    }

    if ctx.has_right(Rule::EditUserRights) {
        markup = markup.append_row(vec![InlineKeyboardButton::callback(
            "🔒 Права",
            UserCallback::EditRights.to_data(),
        )]);
    }

    if back {
        markup = markup.append_row(vec![InlineKeyboardButton::callback(
            "⬅️",
            UserCallback::Back.to_data(),
        )]);
    }
    (msg, markup)
}

#[derive(Clone, Debug, PartialEq)]
pub enum UserCallback {
    Back,
    BlockUnblock,
    Edit,
    EditRights,
}

impl TryFrom<&str> for UserCallback {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "uc_back" => UserCallback::Back,
            "uc_block" => UserCallback::BlockUnblock,
            "uc_edit" => UserCallback::Edit,
            "uc_rights" => UserCallback::EditRights,
            _ => return Err(eyre::eyre!("Failed to parse callback")),
        })
    }
}

impl UserCallback {
    pub fn to_data(&self) -> &str {
        match self {
            UserCallback::Back => "uc_back",
            UserCallback::BlockUnblock => "uc_block",
            UserCallback::Edit => "uc_edit",
            UserCallback::EditRights => "uc_rights",
        }
    }
}
