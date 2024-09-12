use std::mem;

use crate::{
    callback_data::Calldata as _,
    context::Context,
    state::Widget,
    view::{menu::MainMenuItem, View},
};
use async_trait::async_trait;
use chrono::Local;
use log::warn;
use model::{
    rights::Rule,
    user::{User, UserSubscription},
};
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

use super::{freeze::FreezeProfile, rights::UserRightsView};

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
        let user = ctx
            .ledger
            .users
            .get_by_tg_id(&mut ctx.session, self.tg_id)
            .await?
            .ok_or_else(|| eyre::eyre!("User not found:{}", self.tg_id))?;
        let (msg, keymap) = render_user_profile(&ctx, &user, self.go_back.is_some());
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
                    warn!("Attempt to go back");
                    Ok(None)
                }
            }
            Callback::BlockUnblock => {
                ctx.ensure(Rule::BlockUser)?;
                let user = ctx
                    .ledger
                    .users
                    .get_by_tg_id(&mut ctx.session, self.tg_id)
                    .await?
                    .ok_or_else(|| eyre::eyre!("User not found"))?;
                ctx.ledger
                    .block_user(&mut ctx.session, self.tg_id, !user.is_active)
                    .await?;
                ctx.reload_user().await?;
                self.show(ctx).await?;
                Ok(None)
            }
            Callback::Edit => Ok(None),
            Callback::EditRights => {
                ctx.ensure(Rule::EditUserRights)?;
                let mut new_user_new = UserProfile::new(0, None);
                mem::swap(self, &mut new_user_new);
                Ok(Some(Box::new(UserRightsView::new(
                    new_user_new.tg_id,
                    Some(Box::new(new_user_new)),
                ))))
            }
            Callback::Freeze => {
                if !ctx.has_right(Rule::FreezeUsers) && ctx.me.tg_id != self.tg_id {
                    return Err(eyre::eyre!("User has no rights to perform this action"));
                }
                let mut new_user_new = UserProfile::new(0, None);
                mem::swap(self, &mut new_user_new);
                Ok(Some(Box::new(FreezeProfile::new(
                    new_user_new.tg_id,
                    Some(Box::new(new_user_new)),
                ))))
            }
            Callback::ChangeBalance(amount) => {
                ctx.ensure(Rule::ChangeBalance)?;
                let user = ctx
                    .ledger
                    .users
                    .get_by_tg_id(&mut ctx.session, self.tg_id)
                    .await?
                    .ok_or_else(|| eyre::eyre!("User not found"))?;

                if amount < 0 {
                    if user.balance < amount.abs() as u32 {
                        return Err(eyre::eyre!("Not enough balance"));
                    }
                }

                ctx.ledger
                    .users
                    .change_balance(&mut ctx.session, user.tg_id, amount)
                    .await?;
                ctx.reload_user().await?;
                self.show(ctx).await?;
                Ok(None)
            }
        }
    }
}

fn render_user_profile(ctx: &Context, user: &User, back: bool) -> (String, InlineKeyboardMarkup) {
    let msg = render_profile_msg(user);

    let mut keymap = InlineKeyboardMarkup::default();
    if ctx.has_right(Rule::FreezeUsers) || ctx.me.tg_id == user.tg_id {
        if user.freeze.is_none() {
            if user.freeze_days != 0 {
                keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
                    "Заморозить ❄",
                    Callback::Freeze.to_data(),
                )]);
            }
        }
    }

    if ctx.has_right(Rule::ChangeBalance) {
        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            "Списать баланс 💸",
            Callback::ChangeBalance(-1).to_data(),
        )]);
        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            "Пополнить баланс 💰",
            Callback::ChangeBalance(1).to_data(),
        )]);
    }

    if ctx.has_right(Rule::BlockUser) && ctx.me.tg_id != user.tg_id {
        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            if user.is_active {
                "❌ Заблокировать"
            } else {
                "✅ Разблокировать"
            },
            Callback::BlockUnblock.to_data(),
        )]);
    }

    if ctx.has_right(Rule::EditUserInfo) {
        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            "✍️ Редактировать",
            Callback::Edit.to_data(),
        )]);
    }

    if ctx.has_right(Rule::EditUserRights) {
        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            "🔒 Права",
            Callback::EditRights.to_data(),
        )]);
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Callback {
    Back,
    BlockUnblock,
    Edit,
    EditRights,
    Freeze,
    ChangeBalance(i32),
}

fn render_sub(sub: &UserSubscription) -> String {
    format!(
        "🎟_{}_\nОсталось занятий:_{}_\nДействует до:_{}_\n",
        escape(&sub.name),
        sub.items,
        sub.end_date.with_timezone(&Local).format("%d\\.%m\\.%Y")
    )
}

pub fn render_profile_msg(user: &User) -> String {
    let empty = "?".to_string();
    let mut msg = format!(
        "
{} Пользователь : _@{}_
Имя : _{}_
Телефон : _{}_
Дата рождения : _{}_
➖➖➖➖➖➖➖➖➖➖
*Баланс : _{}_ занятий*
*Резерв : _{}_ занятий*
*Осталось дней заморозок: _{}_*
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
        user.balance,
        user.reserved_balance,
        user.freeze_days
    );

    let mut subs = user.subscriptions.iter().collect::<Vec<_>>();
    subs.sort_by(|a, b| a.end_date.cmp(&b.end_date));
    msg.push_str("Абонементы:\n");
    if !subs.is_empty() {
        for sub in subs {
            msg.push_str(&render_sub(sub));
        }
    } else {
        msg.push_str("*нет действующих абонентов*🥺\n");
    }
    msg.push_str("➖➖➖➖➖➖➖➖➖➖");
    msg
}

pub fn user_type(user: &User) -> &str {
    if user.freeze.is_some() {
        "❄️"
    } else if !user.is_active {
        "⚫"
    } else if user.rights.is_full() {
        "🔴"
    } else if user.rights.has_rule(Rule::Train) {
        "🔵"
    } else {
        "🟢"
    }
}
