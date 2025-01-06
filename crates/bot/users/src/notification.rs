use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use model::user::extension::UserExtension;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};

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
        let user = ctx
            .ledger
            .users
            .get_extension(&mut ctx.session, self.id)
            .await?;
        let msg = "Настройки уведомлений\\.\nОтметьте промежуток времени, в который вы хотите получать уведомления\\.\n ✅ - включено, ❌ - выключено";

        let mut keymap = InlineKeyboardMarkup::default();

        keymap = keymap.append_row(vec![
            Callback::btn(&user, 0, "00 \\- 01"),
            Callback::btn(&user, 1, "01 \\- 02"),
            Callback::btn(&user, 2, "02 \\- 03"),
            Callback::btn(&user, 3, "03 \\- 04"),
        ]);

        keymap = keymap.append_row(vec![
            Callback::btn(&user, 4, "04 \\- 05"),
            Callback::btn(&user, 5, "05 \\- 06"),
            Callback::btn(&user, 6, "06 \\- 07"),
            Callback::btn(&user, 7, "07 \\- 08"),
        ]);

        keymap = keymap.append_row(vec![
            Callback::btn(&user, 8, "08 \\- 09"),
            Callback::btn(&user, 9, "09 \\- 10"),
            Callback::btn(&user, 10, "10 \\- 11"),
            Callback::btn(&user, 11, "11 \\- 12"),
        ]);

        keymap = keymap.append_row(vec![
            Callback::btn(&user, 12, "12 \\- 13"),
            Callback::btn(&user, 13, "13 \\- 14"),
            Callback::btn(&user, 14, "14 \\- 15"),
            Callback::btn(&user, 15, "15 \\- 16"),
        ]);

        keymap = keymap.append_row(vec![
            Callback::btn(&user, 16, "16 \\- 17"),
            Callback::btn(&user, 17, "17 \\- 18"),
            Callback::btn(&user, 18, "18 \\- 19"),
            Callback::btn(&user, 19, "19 \\- 20"),
        ]);

        keymap = keymap.append_row(vec![
            Callback::btn(&user, 20, "20 \\- 21"),
            Callback::btn(&user, 21, "21 \\- 22"),
            Callback::btn(&user, 22, "22 \\- 23"),
            Callback::btn(&user, 23, "23 \\- 00"),
        ]);

        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp, eyre::Error> {
        match calldata!(data) {
            Callback::SetTime(hour) => {}
            Callback::ResetTime(hour) => {}
        }
        Ok(Jmp::Stay)
    }
}

#[derive(Serialize, Deserialize)]
enum Callback {
    SetTime(u8),
    ResetTime(u8),
}

impl Callback {
    fn btn(extension: &UserExtension, hour: u8, text: &str) -> InlineKeyboardButton {
        let is_enabled = extension.notification_mask.get_hour(hour as u32);
        let text = if is_enabled {
            format!("✅ {}", text)
        } else {
            format!("❌ {}", text)
        };
        if is_enabled {
            Callback::ResetTime(hour).button(text)
        } else {
            Callback::SetTime(hour).button(text)
        }
    }
}
