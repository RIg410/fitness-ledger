use std::mem;

use crate::{
    callback_data::Calldata as _,
    context::Context,
    state::Widget,
    view::{training::client_training::ClientTrainings, View},
};
use async_trait::async_trait;
use chrono::Local;
use eyre::eyre;
use log::warn;
use model::{
    rights::Rule,
    subscription::{Status, UserSubscription},
    training::Training,
    user::User,
};
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

use super::{
    freeze::FreezeProfile, rights::UserRightsView, set_birthday::SetBirthday, set_fio::SetFio,
};

pub struct UserProfile {
    tg_id: i64,
    go_back: Option<Widget>,
}

impl UserProfile {
    pub fn new(tg_id: i64, go_back: Option<Widget>) -> UserProfile {
        UserProfile { tg_id, go_back }
    }

    async fn block_user(&mut self, ctx: &mut Context) -> Result<Option<Widget>, eyre::Error> {
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

    async fn change_balance(
        &mut self,
        ctx: &mut Context,
        amount: i32,
    ) -> Result<Option<Widget>, eyre::Error> {
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

    async fn freeze_user(&mut self, ctx: &mut Context) -> Result<Option<Widget>, eyre::Error> {
        if !ctx.has_right(Rule::FreezeUsers) && ctx.me.tg_id != self.tg_id {
            return Err(eyre::eyre!("User has no rights to perform this action"));
        }
        let mut new_user_new = UserProfile::new(0, None);
        mem::swap(self, &mut new_user_new);
        Ok(Some(
            FreezeProfile::new(new_user_new.tg_id, Some(new_user_new.boxed())).boxed(),
        ))
    }

    async fn edit_rights(&mut self, ctx: &mut Context) -> Result<Option<Widget>, eyre::Error> {
        ctx.ensure(Rule::EditUserRights)?;
        let mut new_user_new = UserProfile::new(0, None);
        mem::swap(self, &mut new_user_new);
        Ok(Some(
            UserRightsView::new(new_user_new.tg_id, Some(new_user_new.boxed())).boxed(),
        ))
    }

    async fn set_birthday(&mut self, ctx: &mut Context) -> Result<Option<Widget>, eyre::Error> {
        if ctx.has_right(Rule::EditUserInfo) || ctx.me.tg_id == self.tg_id {
            Ok(Some(
                SetBirthday::new(
                    self.tg_id,
                    Some(UserProfile::new(self.tg_id, self.go_back.take()).boxed()),
                )
                .boxed(),
            ))
        } else {
            Ok(None)
        }
    }

    async fn training_list(&mut self, ctx: &mut Context) -> Result<Option<Widget>, eyre::Error> {
        let user = ctx
            .ledger
            .users
            .get_by_tg_id(&mut ctx.session, self.tg_id)
            .await?
            .ok_or_else(|| eyre!("User not found:{}", self.tg_id))?;

        Ok(Some(
            ClientTrainings::new(
                user.id,
                Some(UserProfile::new(self.tg_id, self.go_back.take()).boxed()),
            )
            .boxed(),
        ))
    }

    async fn set_fio(&mut self, ctx: &mut Context) -> Result<Option<Widget>, eyre::Error> {
        if ctx.has_right(Rule::EditUserInfo) {
            Ok(Some(
                SetFio::new(
                    self.tg_id,
                    Some(UserProfile::new(self.tg_id, self.go_back.take()).boxed()),
                )
                .boxed(),
            ))
        } else {
            Ok(None)
        }
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
        let trainings = ctx
            .ledger
            .calendar
            .get_users_trainings(&mut ctx.session, user.id, 100, 0)
            .await?;
        let (msg, keymap) = render_user_profile(&ctx, &user, &trainings, self.go_back.is_some());
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
            Callback::BlockUnblock => self.block_user(ctx).await,
            Callback::EditFio => self.set_fio(ctx).await,
            Callback::EditRights => self.edit_rights(ctx).await,
            Callback::Freeze => self.freeze_user(ctx).await,
            Callback::ChangeBalance(amount) => self.change_balance(ctx, amount).await,
            Callback::SetBirthday => self.set_birthday(ctx).await,
            Callback::TrainingList => self.training_list(ctx).await,
        }
    }
}

fn render_user_profile(
    ctx: &Context,
    user: &User,
    trainings: &Vec<Training>,
    back: bool,
) -> (String, InlineKeyboardMarkup) {
    let msg = render_profile_msg(user, ctx.has_right(Rule::ViewProfile), trainings);

    let mut keymap = InlineKeyboardMarkup::default();
    if ctx.has_right(Rule::FreezeUsers)
        || ctx.me.tg_id == user.tg_id
        || !user.subscriptions.is_empty()
    {
        if user.freeze.is_none() {
            if user.freeze_days != 0 {
                keymap = keymap.append_row(Callback::Freeze.btn_row("Заморозить ❄"));
            }
        }
    }

    if ctx.has_right(Rule::ChangeBalance) {
        keymap = keymap.append_row(vec![
            Callback::ChangeBalance(-1).button("Списать баланс 💸"),
            Callback::ChangeBalance(1).button("Пополнить баланс 💰"),
        ]);
    }

    keymap = keymap.append_row(Callback::TrainingList.btn_row("Записи 📝"));

    if ctx.has_right(Rule::BlockUser) && ctx.me.tg_id != user.tg_id {
        keymap = keymap.append_row(Callback::BlockUnblock.btn_row(if user.is_active {
            "❌ Заблокировать"
        } else {
            "✅ Разблокировать"
        }));
    }
    if ctx.has_right(Rule::EditUserInfo) || (ctx.me.id == user.id && user.birthday.is_none()) {
        keymap = keymap.append_row(Callback::SetBirthday.btn_row("Установить дату рождения"));
    }

    if ctx.has_right(Rule::EditUserInfo) {
        keymap = keymap.append_row(Callback::EditFio.btn_row("✍️ Редактировать ФИО"));
    }
    if ctx.has_right(Rule::EditUserRights) {
        keymap = keymap.append_row(Callback::EditRights.btn_row("🔒 Права"));
    }
    if back {
        keymap = keymap.append_row(Callback::Back.btn_row("⬅️"));
    }
    (msg, keymap)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Callback {
    Back,
    BlockUnblock,
    EditFio,
    SetBirthday,
    EditRights,
    Freeze,
    TrainingList,
    ChangeBalance(i32),
}

fn render_sub(sub: &UserSubscription) -> String {
    match sub.status {
        Status::NotActive => {
            format!(
                "🎟_{}_\nОсталось занятий:_{}_\nНе активен\\. \n",
                escape(&sub.name),
                sub.items,
            )
        }
        Status::Active { start_date } => {
            let end_date = start_date + chrono::Duration::days(i64::from(sub.days));
            format!(
                "🎟_{}_\nОсталось занятий:_{}_\nДействует до:_{}_\n",
                escape(&sub.name),
                sub.items,
                end_date.with_timezone(&Local).format("%d\\.%m\\.%Y")
            )
        }
    }
}

pub fn render_profile_msg(user: &User, sys_info: bool, trainings: &Vec<Training>) -> String {
    let empty = "?".to_string();

    let sys_info = if sys_info {
        format!("*Резерв : _{}_ занятий*", user.reserved_balance)
    } else {
        "".to_owned()
    };

    let mut msg = format!(
        "
{} Пользователь : _@{}_
Имя : _{}_
Телефон : _\\+{}_
Дата рождения : _{}_
➖➖➖➖➖➖➖➖➖➖
*Баланс : _{}_ занятий*
{}
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
        sys_info,
        user.freeze_days
    );

    if !trainings.is_empty() {
        msg.push_str("Записи:\n");
        for training in trainings {
            msg.push_str(&escape(&format!(
                "{} {}\n",
                training.start_at.format("%d.%m %H:%M"),
                training.name
            )))
        }
        msg.push_str("➖➖➖➖➖➖➖➖➖➖\n");
    }
    let mut subs = user.subscriptions.iter().collect::<Vec<_>>();
    subs.sort_by(|a, b| a.status.cmp(&b.status));
    msg.push_str("Абонементы:\n");
    if !subs.is_empty() {
        for sub in subs {
            msg.push_str(&render_sub(sub));
        }
    } else {
        if user.balance == 0 && user.reserved_balance == 0 {
            msg.push_str("*нет абонементов*🥺\n");
        } else {
            msg.push_str(&format!(
                "🎟_тестовый_\nОсталось занятий:_{}_\n",
                user.balance + user.reserved_balance
            ));
        }
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
