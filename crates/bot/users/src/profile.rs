use crate::{
    come_from::MarketingInfoView, comments::Comments, family::FamilyView, history::HistoryList,
    notification::NotificationView, rewards::RewardsList, subscriptions::SubscriptionsList,
};

use super::{
    freeze::FreezeProfile, rights::UserRightsView, set_birthday::SetBirthday, set_fio::SetFio,
    set_phone::SetPhone,
};
use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_trainigs::list::TrainingList;
use bot_viewer::user::render_profile_msg;
use eyre::Error;
use model::{
    rights::Rule,
    statistics::user::{SubscriptionStat, TrainingsStat},
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardMarkup, Message},
    utils::markdown::escape,
};

pub struct UserProfile {
    id: ObjectId,
    skip_show: bool,
}

impl UserProfile {
    pub fn new(id: ObjectId) -> UserProfile {
        UserProfile {
            id,
            skip_show: false,
        }
    }

    async fn block_user(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::BlockUser)?;
        let user = ctx
            .ledger
            .users
            .get(&mut ctx.session, self.id)
            .await?
            .ok_or_else(|| eyre::eyre!("User not found"))?;
        ctx.ledger
            .block_user(&mut ctx.session, self.id, !user.is_active)
            .await?;
        ctx.reload_user().await?;
        Ok(Jmp::Stay)
    }

    async fn freeze_user(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        if !ctx.has_right(Rule::FreezeUsers) && ctx.me.id != self.id {
            return Err(eyre::eyre!("User has no rights to perform this action"));
        }
        Ok(FreezeProfile::new(self.id).into())
    }

    async fn edit_rights(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::EditUserRights)?;
        Ok(UserRightsView::new(self.id).into())
    }

    async fn set_birthday(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        if ctx.has_right(Rule::EditUserInfo) || ctx.me.id == self.id {
            Ok(SetBirthday::new(self.id).into())
        } else {
            Ok(Jmp::Stay)
        }
    }

    async fn training_list(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        let user = ctx.ledger.get_user(&mut ctx.session, self.id).await?;
        if user.employee.is_some() {
            Ok(TrainingList::couches(user.id).into())
        } else {
            Ok(TrainingList::users(user.id).into())
        }
    }

    async fn history_list(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        let user = ctx.ledger.get_user(&mut ctx.session, self.id).await?;
        Ok(HistoryList::new(user.id).into())
    }

    async fn rewards_list(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        let user = ctx.ledger.get_user(&mut ctx.session, self.id).await?;
        if user.employee.is_some() && (ctx.is_me(user.id) || ctx.has_right(Rule::ViewRewards)) {
            Ok(RewardsList::new(user.id).into())
        } else {
            Ok(Jmp::Stay)
        }
    }

    async fn set_fio(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        if ctx.has_right(Rule::EditUserInfo) {
            Ok(SetFio::new(self.id).into())
        } else {
            Ok(Jmp::Stay)
        }
    }

    async fn set_phone(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        if ctx.has_right(Rule::EditUserInfo) {
            Ok(SetPhone::new(self.id).into())
        } else {
            Ok(Jmp::Stay)
        }
    }

    async fn family_view(&mut self, ctx: &mut Context, id: ObjectId) -> Result<Jmp, eyre::Error> {
        if ctx.has_right(Rule::ViewFamily) || (ctx.me.id == id && ctx.me.has_family()) {
            Ok(FamilyView::new(self.id).into())
        } else {
            Ok(Jmp::Stay)
        }
    }

    async fn unfreeze_user(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::FreezeUsers)?;
        ctx.ledger.users.unfreeze(&mut ctx.session, self.id).await?;
        ctx.reload_user().await?;
        Ok(Jmp::Stay)
    }

    async fn show_statistics(&mut self, ctx: &mut Context) -> Result<Jmp, eyre::Error> {
        ctx.ensure(Rule::ViewStatistics)?;

        let user_stat = ctx
            .ledger
            .users
            .collect_statistics(&mut ctx.session, &self.id)
            .await?;
        let mut message = "Статистика пользователя: \n".to_string();

        message.push_str("Cтатистика по абонентам 📊: \n");
        let mut total_sub = SubscriptionStat::new("Общая".to_string());
        for (_, sub) in user_stat.subscriptions {
            message.push_str(&print_sub_stat(&sub));
            total_sub.join(&sub);
        }

        let mut total = TrainingsStat::default();
        message.push_str("Cтатистика по тренировкам 📊: \n");
        for (name, training) in user_stat.training {
            message.push_str(&print_training_stat(&training, name));
            total.join(&training);
        }

        message.push_str(&print_training_stat(&total, "Общая статистика".to_string()));
        message.push_str(&print_sub_stat(&total_sub));

        if user_stat.changed_subscription_days > 0 {
            message.push_str(&format!(
                "Добавлено *{}* дней к абонементу\n",
                user_stat.changed_subscription_days
            ));
        } else {
            message.push_str(&format!(
                "Списано *{}* дней с абонемента\n",
                user_stat.changed_subscription_days.abs()
            ));
        }

        if user_stat.changed_subscription_balance > 0 {
            message.push_str(&format!(
                "Добавлено *{}* тренировок к абонементу\n",
                user_stat.changed_subscription_balance
            ));
        } else {
            message.push_str(&format!(
                "Списано *{}* тренировок с абонемента\n",
                user_stat.changed_subscription_balance.abs()
            ));
        }

        ctx.send_notification(&message).await;
        Ok(Jmp::Stay)
    }
}

fn print_training_stat(training: &TrainingsStat, name: String) -> String {
    format!(
        "{}\nВсего посещяно *{}* тренировок\nОтменено *{}* тренировок\n\n",
        escape(&name),
        training.count,
        training.cancellations_count
    )
}

fn print_sub_stat(sub: &SubscriptionStat) -> String {
    format!(
        "📦 {}\nКуплено: *{}* шт\nПотрачено: *{}* руб\nОбщая скидка: *{}* руб\nВозвраты: *{}* руб\nСгорело *{}* тренировок на сумму *{}* руб\n\n",
        escape(&sub.name),
        sub.soult_count,
        sub.spent.to_string().replace(".", ","),
        sub.discount.to_string().replace(".", ","),
        sub.refunds_sum.to_string().replace(".", ","),
        sub.expired_trainings,
        sub.expired_sum.to_string().replace(".", ","),
    )
}

#[async_trait]
impl View for UserProfile {
    fn name(&self) -> &'static str {
        "UserProfile"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        if self.skip_show {
            self.skip_show = false;
            return Ok(());
        }
        let (msg, keymap) = render_user_profile(ctx, self.id).await?;
        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        message: &Message,
    ) -> Result<Jmp, eyre::Error> {
        ctx.delete_msg(message.id).await?;
        if ctx.has_right(Rule::AIUserInfo) {
            if let Some(text) = message.text() {
                let ai_response = ctx
                    .ledger
                    .users
                    .ask_ai(
                        &mut ctx.session,
                        self.id,
                        ai::AiModel::Gpt4o,
                        text.to_string(),
                    )
                    .await;
                match ai_response {
                    Ok(resp) => {
                        let msg =
                            format!("\n\nВопрос: {}\nОтвет: {}", escape(&text), escape(&resp));
                        self.skip_show = true;
                        ctx.send_notification(&msg).await;
                    }
                    Err(err) => {
                        ctx.send_notification(&format!("Ошибка: {}", err)).await;
                    }
                }
            }
        }
        Ok(Jmp::Stay)
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        match calldata!(data) {
            Callback::BlockUnblock => self.block_user(ctx).await,
            Callback::EditFio => self.set_fio(ctx).await,
            Callback::EditRights => self.edit_rights(ctx).await,
            Callback::Freeze => self.freeze_user(ctx).await,
            Callback::SetBirthday => self.set_birthday(ctx).await,
            Callback::EditPhone => self.set_phone(ctx).await,
            Callback::TrainingList => self.training_list(ctx).await,
            Callback::HistoryList => self.history_list(ctx).await,
            Callback::RewardsList => self.rewards_list(ctx).await,
            Callback::Notification => Ok(NotificationView::new(self.id).into()),
            Callback::SubscriptionsList => {
                ctx.ensure(Rule::EditUserSubscription)?;
                Ok(SubscriptionsList::new(self.id).into())
            }
            Callback::EditMarketingInfo => {
                ctx.ensure(Rule::EditMarketingInfo)?;
                Ok(MarketingInfoView::new(self.id).into())
            }
            Callback::FamilyView => self.family_view(ctx, self.id).await,
            Callback::UnFreeze => self.unfreeze_user(ctx).await,
            Callback::Comments => Ok(Comments::new(self.id).into()),
            Callback::Statistics => self.show_statistics(ctx).await,
        }
    }
}

async fn render_user_profile(
    ctx: &mut Context,
    id: ObjectId,
) -> Result<(String, InlineKeyboardMarkup), Error> {
    let (msg, user, extension) = render_profile_msg(ctx, id).await?;

    let mut keymap = InlineKeyboardMarkup::default();

    if ctx.has_right(Rule::ViewFamily) || (ctx.me.tg_id == user.tg_id && ctx.me.has_family()) {
        keymap = keymap.append_row(Callback::FamilyView.btn_row("Семья 👨‍👩‍👧‍👦"));
    }

    if ctx.has_right(Rule::EditMarketingInfo) {
        keymap = keymap.append_row(Callback::EditMarketingInfo.btn_row("Изменить источник 📝"));
    }

    if extension.birthday.is_none() || ctx.has_right(Rule::EditUserInfo) {
        keymap = keymap.append_row(Callback::SetBirthday.btn_row("Установить дату рождения 🎂"));
    }

    if (ctx.has_right(Rule::FreezeUsers)
        || (ctx.me.tg_id == user.tg_id
            && user.payer()?.has_subscription()
            && user.freeze_days != 0))
        && user.freeze.is_none()
    {
        keymap = keymap.append_row(Callback::Freeze.btn_row("Заморозить ❄"));
    } else if ctx.has_right(Rule::FreezeUsers) && user.freeze.is_some() {
        keymap = keymap.append_row(Callback::UnFreeze.btn_row("Разморозить ❄"));
    }

    if user.employee.is_some() {
        keymap = keymap.append_row(Callback::TrainingList.btn_row("Тренировки 📝"));
    } else {
        keymap = keymap.append_row(Callback::TrainingList.btn_row("Записи 📝"));
    }

    if ctx.has_right(Rule::BlockUser) && ctx.me.tg_id != user.tg_id {
        keymap = keymap.append_row(Callback::BlockUnblock.btn_row(if user.is_active {
            "Заблокировать ❌"
        } else {
            "Разблокировать ✅"
        }));
    }
    if ctx.has_right(Rule::EditUserInfo) || (ctx.me.id == user.id && extension.birthday.is_none()) {
        keymap = keymap.append_row(Callback::SetBirthday.btn_row("Установить дату рождения"));
    }

    if ctx.has_right(Rule::EditUserInfo) {
        keymap = keymap.append_row(Callback::EditFio.btn_row("✍️ Редактировать ФИО"));
        keymap = keymap.append_row(Callback::EditPhone.btn_row("✍️ Редактировать телефон"));
    }

    if ctx.has_right(Rule::EditMarketingInfo) {
        keymap = keymap.append_row(Callback::EditMarketingInfo.btn_row("Изменить источник 📝"));
    }

    if ctx.has_right(Rule::EditUserSubscription) {
        keymap = keymap.append_row(Callback::SubscriptionsList.btn_row("Абонементы 📝"));
    }

    if ctx.has_right(Rule::EditUserRights) {
        keymap = keymap.append_row(Callback::EditRights.btn_row("Права 🔒"));
    }
    keymap = keymap.append_row(Callback::Notification.btn_row("Уведомления 🔔"));

    keymap = keymap.append_row(Callback::HistoryList.btn_row("История 📝"));
    if user.employee.is_some() && (ctx.is_me(id) || ctx.has_right(Rule::ViewRewards)) {
        keymap = keymap.append_row(Callback::RewardsList.btn_row("Вознаграждения 📝"));
    }

    if ctx.has_right(Rule::ViewUserComments) {
        keymap = keymap.append_row(Callback::Comments.btn_row("Комментарии 📝"));
    }

    if ctx.has_right(Rule::ViewStatistics) {
        keymap = keymap.append_row(Callback::Statistics.btn_row("Статистика 📊"));
    }

    Ok((msg, keymap))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Callback {
    BlockUnblock,
    EditFio,
    EditPhone,
    SetBirthday,
    EditRights,
    Freeze,
    TrainingList,
    HistoryList,
    RewardsList,
    Notification,
    SubscriptionsList,
    EditMarketingInfo,
    FamilyView,
    UnFreeze,
    Comments,
    Statistics,
}
