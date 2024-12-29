use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{types::{InlineKeyboardButton, InlineKeyboardMarkup}, utils::markdown::escape};

pub struct NotificationView {
    pub id: ObjectId,
}

impl NotificationView {
    pub fn new(id: ObjectId) -> Self {
        NotificationView { id }
    }
}

#[async_trait]
impl View for NotificationView {
    fn name(&self) -> &'static str {
        "NotificationView"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), eyre::Error> {
        let user = ctx.ledger.get_user(&mut ctx.session, self.id).await?;
        let msg = escape("Настройка уведомлений.\nВы можите включить или отключить уведомления о предстоящих тренировках.  ❌-выключить\n  ✅-включить");

        let mut keymap = InlineKeyboardMarkup::default();
        let settings = user.settings.notification;

        if settings.notify_by_day {
            keymap = keymap.append_row(
                Callback::ByDayOnOff(false).btn_row("🔕 Отключить уведомления за сутки"),
            );
        } else {
            keymap = keymap
                .append_row(Callback::ByDayOnOff(true).btn_row("🔔 Включить уведомления за сутки"));
        }

        keymap = keymap.append_row(vec![InlineKeyboardButton::callback(
            "Уведомлять за несколько часов до тренировки",
            "2",
        )]);

        let hours = settings.notify_by_n_hours.unwrap_or(0);
        keymap = keymap.append_row(vec![
            InlineKeyboardButton::callback("Не уведомлять", "0"),
            Callback::ByHoursOff.button(if hours == 0 { "✅" } else { "❌" }),
        ]);
        keymap = keymap.append_row(vec![
            InlineKeyboardButton::callback("За час", "1"),
            Callback::ByHoursOn(1).button(if hours == 1 { "✅" } else { "❌" }),
        ]);

        keymap = keymap.append_row(vec![
            InlineKeyboardButton::callback("За 2 часа", "2"),
            Callback::ByHoursOn(2).button(if hours == 2 { "✅" } else { "❌" }),
        ]);

        keymap = keymap.append_row(vec![
            InlineKeyboardButton::callback("За 3 часа", "3"),
            Callback::ByHoursOn(3).button(if hours == 3 { "✅" } else { "❌" }),
        ]);

        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        match calldata!(data) {
            Callback::ByDayOnOff(on) => {
                let mut user = ctx.ledger.get_user(&mut ctx.session, self.id).await?;
                user.settings.notification.notify_by_day = on;
                ctx.ledger
                    .users
                    .update_notification_settings(
                        &mut ctx.session,
                        user.id,
                        user.settings.notification,
                    )
                    .await?;
            }
            Callback::ByHoursOff => {
                let mut user = ctx.ledger.get_user(&mut ctx.session, self.id).await?;
                user.settings.notification.notify_by_n_hours = None;
                ctx.ledger
                    .users
                    .update_notification_settings(
                        &mut ctx.session,
                        user.id,
                        user.settings.notification,
                    )
                    .await?;
            }
            Callback::ByHoursOn(hours) => {
                let mut user = ctx.ledger.get_user(&mut ctx.session, self.id).await?;
                user.settings.notification.notify_by_n_hours = Some(hours);
                ctx.ledger
                    .users
                    .update_notification_settings(
                        &mut ctx.session,
                        user.id,
                        user.settings.notification,
                    )
                    .await?;
            }
        }
        Ok(Jmp::Stay)
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    ByDayOnOff(bool),
    ByHoursOff,
    ByHoursOn(u8),
}
