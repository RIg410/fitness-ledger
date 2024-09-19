use crate::{
    callback_data::Calldata as _,
    context::Context,
    state::Widget,
    view::{training::client_training::ClientTrainings, View},
};
use async_trait::async_trait;
use chrono::Local;
use eyre::{eyre, Error};
use log::warn;
use model::{
    couch::{CouchInfo, Rate},
    rights::Rule,
    subscription::{Status, UserSubscription},
    user::{User, UserIdent},
};
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

use super::{
    freeze::FreezeProfile, rights::UserRightsView, set_birthday::SetBirthday, set_fio::SetFio,
    set_phone::SetPhone,
};

pub struct UserProfile {
    tg_id: i64,
    go_back: Option<Widget>,
}

impl UserProfile {
    pub fn new(tg_id: i64) -> UserProfile {
        UserProfile {
            tg_id,
            go_back: None,
        }
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

    async fn change_reserved_balance(
        &mut self,
        ctx: &mut Context,
        amount: i32,
    ) -> Result<Option<Widget>, eyre::Error> {
        ctx.ensure(Rule::ChangeBalance)?;
        let user = ctx.ledger.get_user(&mut ctx.session, self.tg_id).await?;

        if amount < 0 {
            if user.reserved_balance < amount.abs() as u32 {
                return Err(eyre::eyre!("Not enough reserved balance"));
            }
        }

        ctx.ledger
            .users
            .change_reserved_balance(&mut ctx.session, user.tg_id, amount)
            .await?;
        ctx.reload_user().await?;
        self.show(ctx).await?;
        Ok(None)
    }

    async fn freeze_user(&mut self, ctx: &mut Context) -> Result<Option<Widget>, eyre::Error> {
        if !ctx.has_right(Rule::FreezeUsers) && ctx.me.tg_id != self.tg_id {
            return Err(eyre::eyre!("User has no rights to perform this action"));
        }
        Ok(Some(FreezeProfile::new(self.tg_id).boxed()))
    }

    async fn edit_rights(&mut self, ctx: &mut Context) -> Result<Option<Widget>, eyre::Error> {
        ctx.ensure(Rule::EditUserRights)?;
        Ok(Some(UserRightsView::new(self.tg_id).boxed()))
    }

    async fn set_birthday(&mut self, ctx: &mut Context) -> Result<Option<Widget>, eyre::Error> {
        if ctx.has_right(Rule::EditUserInfo) || ctx.me.tg_id == self.tg_id {
            Ok(Some(SetBirthday::new(self.tg_id).boxed()))
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

        Ok(Some(ClientTrainings::new(user.id).boxed()))
    }

    async fn set_fio(&mut self, ctx: &mut Context) -> Result<Option<Widget>, eyre::Error> {
        if ctx.has_right(Rule::EditUserInfo) {
            Ok(Some(SetFio::new(self.tg_id).boxed()))
        } else {
            Ok(None)
        }
    }

    async fn set_phone(&mut self, ctx: &mut Context) -> Result<Option<Widget>, eyre::Error> {
        if ctx.has_right(Rule::EditUserInfo) {
            Ok(Some(SetPhone::new(self.tg_id).boxed()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl View for UserProfile {
    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let (msg, keymap) = render_user_profile(ctx, self.tg_id, self.go_back.is_some()).await?;
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
            Callback::ChangeReservedBalance(amount) => {
                self.change_reserved_balance(ctx, amount).await
            }
            Callback::SetBirthday => self.set_birthday(ctx).await,
            Callback::EditPhone => self.set_phone(ctx).await,
            Callback::TrainingList => self.training_list(ctx).await,
        }
    }

    fn take(&mut self) -> Widget {
        UserProfile {
            tg_id: self.tg_id,
            go_back: self.go_back.take(),
        }
        .boxed()
    }

    fn set_back(&mut self, back: Widget) {
        self.go_back = Some(back);
    }

    fn back(&mut self) -> Option<Widget> {
        self.go_back.take()
    }
}

async fn render_user_profile<ID: Into<UserIdent>>(
    ctx: &mut Context,
    id: ID,
    back: bool,
) -> Result<(String, InlineKeyboardMarkup), Error> {
    let (msg, user) = render_profile_msg(ctx, id).await?;

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
        keymap = keymap.append_row(vec![
            Callback::ChangeReservedBalance(-1).button("Списать зарезервированный баланс 💸"),
            Callback::ChangeReservedBalance(1).button("Пополнить зарезервированный баланс 💰"),
        ]);
    }

    if user.is_couch() {
        keymap = keymap.append_row(Callback::TrainingList.btn_row("Тренировки 📝"));
    } else {
        keymap = keymap.append_row(Callback::TrainingList.btn_row("Записи 📝"));
    }

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
        keymap = keymap.append_row(Callback::EditPhone.btn_row("✍️ Редактировать телефон"));
    }
    if ctx.has_right(Rule::EditUserRights) {
        keymap = keymap.append_row(Callback::EditRights.btn_row("🔒 Права"));
    }
    if back {
        keymap = keymap.append_row(Callback::Back.btn_row("⬅️"));
    }
    Ok((msg, keymap))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Callback {
    Back,
    BlockUnblock,
    EditFio,
    EditPhone,
    SetBirthday,
    EditRights,
    Freeze,
    TrainingList,
    ChangeBalance(i32),
    ChangeReservedBalance(i32),
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

pub async fn render_profile_msg<ID: Into<UserIdent>>(
    ctx: &mut Context,
    id: ID,
) -> Result<(String, User), Error> {
    let user = ctx.ledger.get_user(&mut ctx.session, id).await?;

    let mut msg = user_base_info(&user);
    if let Some(couch) = user.couch.as_ref() {
        render_couch_info(&mut msg, couch);
    } else {
        render_balance_info(&mut msg, &user, ctx.has_right(Rule::ViewProfile));
        render_subscriptions(&mut msg, &user);
        render_trainings(ctx, &mut msg, &user).await?;
    }
    Ok((msg, user))
}

async fn render_trainings(ctx: &mut Context, msg: &mut String, user: &User) -> Result<(), Error> {
    let trainings = ctx
        .ledger
        .calendar
        .get_users_trainings(&mut ctx.session, user.id, 100, 0)
        .await?;
    if !trainings.is_empty() {
        msg.push_str("Записи:\n");
        for training in trainings {
            msg.push_str(&escape(&format!(
                "{} {}\n",
                training
                    .start_at
                    .with_timezone(&Local)
                    .format("%d.%m %H:%M"),
                training.name
            )))
        }
        msg.push_str("➖➖➖➖➖➖➖➖➖➖\n");
    }
    Ok(())
}

fn render_subscriptions(msg: &mut String, user: &User) {
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
}

fn render_balance_info(msg: &mut String, user: &User, sys_info: bool) {
    msg.push_str("➖➖➖➖➖➖➖➖➖➖");
    let sys_info = if sys_info {
        format!("\n*Резерв : _{}_ занятий*", user.reserved_balance)
    } else {
        "".to_owned()
    };
    msg.push_str(&format!(
        "*Баланс : _{}_ занятий*{}\n",
        user.balance, sys_info
    ));
}

pub fn user_type(user: &User) -> &str {
    if user.freeze.is_some() {
        "❄️"
    } else if !user.is_active {
        "⚫"
    } else if user.rights.is_full() {
        "🔴"
    } else if user.couch.is_some() {
        "🔵"
    } else {
        "🟢"
    }
}

pub fn user_base_info(user: &User) -> String {
    let empty = "?".to_string();
    format!(
        "{} Пользователь : _@{}_
Имя : _{}_
Фамилия : _{}_
Телефон : _\\+{}_
Дата рождения : _{}_\n",
        user_type(&user),
        escape(user.name.tg_user_name.as_ref().unwrap_or_else(|| &empty)),
        escape(&user.name.first_name),
        escape(&user.name.last_name.as_ref().unwrap_or_else(|| &empty)),
        escape(&user.phone),
        escape(
            &user
                .birthday
                .as_ref()
                .map(|d| d.format("%d.%m.%Y").to_string())
                .unwrap_or_else(|| empty.clone())
        ),
    )
}

fn render_couch_info(msg: &mut String, couch: &CouchInfo) {
    msg.push_str("➖➖➖➖➖➖➖➖➖➖");
    msg.push_str(&format!(
        "\n[Анкета]({})\nНакопленная награда : _{}_💰\n{}\n",
        escape(&couch.description),
        escape(&couch.reward.to_string()),
        render_rate(&couch.rate)
    ));
}

pub fn render_rate(rate: &Rate) -> String {
    match rate {
        Rate::FixedMonthly { rate, next_reward } => {
            format!(
                "Фиксированный месячный тариф : _{}_💰\nСледующая награда : _{}_\n",
                escape(&rate.to_string()),
                next_reward.with_timezone(&Local).format("%d\\.%m\\.%Y")
            )
        }
        Rate::PerClient { min, per_client } => {
            format!(
                "За клиента : _{}_💰\nМинимальная награда : _{}_💰\n",
                escape(&per_client.to_string()),
                escape(&min.to_string())
            )
        }
        Rate::None => "Тариф не определен".to_string(),
    }
}
