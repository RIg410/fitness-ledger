use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use chrono::Local;
use eyre::{Error, Result};
use model::{
    log::{Action, LogEntry},
    user::UserIdent,
};
use serde::{Deserialize, Serialize};
use teloxide::{types::InlineKeyboardMarkup, utils::markdown::escape};

const PAGE_SIZE: usize = 5;

#[derive(Default)]
pub struct LogsView {
    offset: usize,
}

impl LogsView {
    async fn render_log(
        &self,
        ctx: &mut Context,
        msg: &mut String,
        entry: LogEntry,
    ) -> Result<(), Error> {
        let date = entry
            .date_time
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M:%S");

        let actor = Self::render_user_info(ctx, entry.actor)
            .await
            .ok()
            .unwrap_or_else(|| "unknown".to_owned());

        let action = match Self::render_action(ctx, &entry.action).await {
            Ok(info) => info,
            Err(err) => {
                format!("err:{}::{:?}", err, entry.action)
            }
        };

        msg.push_str(&format!("{}\n{}\n: {:?}\n\n", date, actor, action));
        Ok(())
    }

    async fn render_user_info<ID: Into<UserIdent>>(
        ctx: &mut Context,
        id: ID,
    ) -> Result<String, Error> {
        let user = ctx.ledger.get_user(&mut ctx.session, id).await?;
        Ok(format!(
            "{} {}({}) (+{})",
            user.name.first_name,
            user.name.last_name.unwrap_or_else(|| "".to_string()),
            user.name.tg_user_name.unwrap_or_else(|| "".to_string()),
            user.phone
        ))
    }

    async fn render_action(ctx: &mut Context, act: &Action) -> Result<String, Error> {
        Ok(match act {
            Action::CreateUser { tg_id, name, phone } => {
                format!(
                    "Create user: ({} {:?}) {:?} (+{}) tg_id: {}",
                    name.first_name, name.last_name, name.tg_user_name, phone, tg_id
                )
            }
            Action::SetUserBirthday { tg_id, birthday } => {
                let info = Self::render_user_info(ctx, *tg_id).await?;
                format!(
                    "Set user birthday: {} {}",
                    info,
                    birthday.format("%Y-%m-%d")
                )
            }
            Action::EditUserRule {
                tg_id,
                rule,
                is_active,
            } => {
                let info = Self::render_user_info(ctx, *tg_id).await?;
                format!(
                    "Edit user rule: {} {:?} {}",
                    info,
                    rule,
                    if *is_active { "active" } else { "inactive" }
                )
            }
            Action::Freeze { tg_id, days } => {
                let info = Self::render_user_info(ctx, *tg_id).await?;
                format!("Freeze user: {} for {} days", info, days)
            }
            Action::Unfreeze { tg_id } => {
                let info = Self::render_user_info(ctx, *tg_id).await?;
                format!("Unfreeze user: {}", info)
            }
            Action::ChangeBalance { tg_id, amount } => {
                let info = Self::render_user_info(ctx, *tg_id).await?;
                format!("Change balance: {} by {}", info, amount)
            }
            Action::SetUserName {
                tg_id,
                first_name,
                last_name,
            } => {
                let info = Self::render_user_info(ctx, *tg_id).await?;
                format!("Set user name: {} {} {}", info, first_name, last_name)
            }
            Action::Sell {
                seller,
                buyer,
                sell,
            } => {
                let seller_info = Self::render_user_info(ctx, *seller).await?;
                let buyer_info = Self::render_user_info(ctx, *buyer).await?;
                format!("Sell: {} -> {} {:?}", seller_info, buyer_info, sell)
            }
            Action::Payment {
                user,
                amount,
                description,
                date_time,
            } => {
                let info = Self::render_user_info(ctx, *user).await?;
                format!(
                    "Payment: {} {} {} {}",
                    info,
                    amount,
                    description,
                    date_time.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S")
                )
            }
            Action::Deposit {
                user,
                amount,
                description,
                date_time,
            } => {
                let info = Self::render_user_info(ctx, *user).await?;
                format!(
                    "Deposit: {} {} {} {}",
                    info,
                    amount,
                    description,
                    date_time.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S")
                )
            }
            Action::DeleteSub { sub } => {
                format!("Delete subscription: {:?}", sub)
            }
            Action::CreateSub { sub } => {
                format!("Create subscription: {:?}", sub)
            }
            Action::CreateProgram { program } => {
                format!("Create program: {:?}", program)
            }
            Action::FreeSellSub {
                seller,
                buyer,
                price,
                item,
            } => {
                let seller_info = Self::render_user_info(ctx, *seller).await?;
                let buyer_info = Self::render_user_info(ctx, *buyer).await?;
                format!(
                    "Free sell subscription: {} -> {} {} {}",
                    seller_info, buyer_info, price, item
                )
            }
            Action::SellSub {
                seller,
                buyer,
                subscription,
            } => {
                let seller_info = Self::render_user_info(ctx, *seller).await?;
                let buyer_info = Self::render_user_info(ctx, *buyer).await?;
                format!(
                    "Sell subscription: {} -> {} {:?}",
                    seller_info, buyer_info, subscription
                )
            }
            Action::SignOut {
                name,
                id: _,
                proto_id: _,
                start_at,
                user_id,
            } => {
                format!(
                    "Sign out: {} {} {}",
                    name,
                    start_at.with_timezone(&Local).format("%d/%m/%Y %H:%M"),
                    Self::render_user_info(ctx, *user_id).await?
                )
            }
            Action::SignUp {
                name,
                id: _,
                proto_id: _,
                start_at,
                user_id,
            } => {
                format!(
                    "Sign up: {} {} {}",
                    name,
                    start_at.with_timezone(&Local).format("%d/%m/%Y %H:%M"),
                    Self::render_user_info(ctx, *user_id).await?,
                )
            }
            Action::BlockUser { tg_id, is_active } => {
                let info = Self::render_user_info(ctx, *tg_id).await?;
                format!(
                    "Block user: {} {}",
                    info,
                    if *is_active { "active" } else { "inactive" }
                )
            }
            Action::CancelTraining {
                name,
                start_at,
                id: _,
                proto_id: _,
            } => {
                format!(
                    "Cancel training: {} {}",
                    name,
                    start_at.with_timezone(&Local).format("%d/%m/%Y %H:%M")
                )
            }
            Action::RestoreTraining {
                name,
                id: _,
                proto_id: _,
                start_at,
            } => {
                format!(
                    "Restore training: {} {}",
                    name,
                    start_at.with_timezone(&Local).format("%d/%m/%Y %H:%M")
                )
            }
            Action::DeleteTraining {
                name,
                id: _,
                proto_id: _,
                start_at,
                all,
            } => {
                format!(
                    "Delete training: {} {} {}",
                    name,
                    start_at.with_timezone(&Local).format("%d/%m/%Y %H:%M"),
                    all
                )
            }
            Action::Schedule {
                name,
                id: _,
                proto_id: _,
                start_at,
                instructor,
            } => {
                format!(
                    "Schedule training: {} {} {}",
                    name,
                    start_at.with_timezone(&Local).format("%d/%m/%Y %H:%M"),
                    Self::render_user_info(ctx, *instructor).await?
                )
            }
            Action::FinalizedTraining {
                name,
                id: _,
                proto_id: _,
                start_at,
                clients,
                instructor,
            } => {
                let mut clients_r = String::new();
                for client in clients {
                    clients_r.push_str(&Self::render_user_info(ctx, *client).await?);
                    clients_r.push_str(",\n");
                }

                format!(
                    "Finalized training: {} {} ({}) {}",
                    name,
                    start_at.with_timezone(&Local).format("%d/%m/%Y %H:%M"),
                    clients_r,
                    Self::render_user_info(ctx, *instructor).await?
                )
            }
            Action::FinalizedCanceledTraining {
                name,
                id,
                proto_id,
                start_at,
                clients,
                instructor,
            } => {
                let mut clients_r = String::new();
                for client in clients {
                    clients_r.push_str(&Self::render_user_info(ctx, *client).await?);
                    clients_r.push_str(",\n");
                }

                format!(
                    "Finalized canceled training: {} {} {} {} {} {}",
                    name,
                    id,
                    proto_id,
                    start_at.with_timezone(&Local).format("%d/%m/%Y %H:%M"),
                    clients_r,
                    Self::render_user_info(ctx, *instructor).await?
                )
            }
            Action::SetPhone { tg_id, phone } => {
                let info = Self::render_user_info(ctx, *tg_id).await?;
                format!("Set phone: {} {}", info, phone)
            }
            Action::PreSellSub {
                seller,
                phone,
                subscription,
            } => {
                let seller_info = Self::render_user_info(ctx, *seller).await?;
                format!(
                    "Pre sell subscription: {} {} {:?}",
                    seller_info, phone, subscription
                )
            }
            Action::PreFreeSellSub {
                seller,
                phone,
                price,
                item,
            } => {
                let seller_info = Self::render_user_info(ctx, *seller).await?;
                format!(
                    "Pre free sell subscription: {} {} {} {}",
                    seller_info, phone, price, item
                )
            }
            Action::ChangeReservedBalance { tg_id, amount } => {
                let info = Self::render_user_info(ctx, *tg_id).await?;
                format!("Change reserved balance: {} by {}", info, amount)
            }
            Action::EditProgramCapacity { id, capacity } => {
                format!("Edit program capacity: {} {}", id, capacity)
            }
            Action::EditProgramDuration { id, duration_min } => {
                format!("Edit program duration: {} {}", id, duration_min)
            }
            Action::EditProgramName { id, value } => {
                format!("Edit program name: {} {}", id, value)
            }
            Action::EditProgramDescription { id, value } => {
                format!("Edit program description: {} {}", id, value)
            }
            Action::EditSubPrice { id, value } => {
                format!("Edit sub price:{} {}", id, value)
            }
            Action::EditSubItems { id, value } => {
                format!("Edit sub items:{} {}", id, value)
            }
            Action::EditSubName { id, value } => {
                format!("Edit sub name:{} {}", id, value)
            }
            Action::MakeUserInstructor { tg_id, couch } => {
                let info = Self::render_user_info(ctx, *tg_id).await?;
                format!("Make user instructor: {} {:?}", info, couch)
            }
            Action::DeleteCouch { id } => {
                format!("Delete couch: {}", id)
            }
            Action::UpdateCouchDescription { id, description } => {
                format!("Update couch description: {} {}", id, description)
            }
            Action::UpdateCouchRate { id, rate } => {
                format!("Update couch rate: {} {:?}", id, rate)
            }
            Action::ChangeCouch { start_at, all, new } => {
                format!("Change couch: {} {} {:?}", start_at, all, new)
            }
        })
    }
}

#[async_trait]
impl View for LogsView {
    fn name(&self) -> &'static str {
        "LogView"
    }
    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let mut message = format!("Logs\n");
        let logs = ctx
            .ledger
            .logs
            .logs(&mut ctx.session, PAGE_SIZE, self.offset)
            .await?;

        for log in logs {
            self.render_log(ctx, &mut message, log).await?;
        }

        let keymap = vec![vec![
            Calldata::Back.button("⬅️ Back"),
            Calldata::Forward.button("➡️ Forward"),
        ]];
        ctx.edit_origin(&escape(&message), InlineKeyboardMarkup::new(keymap))
            .await?;
        Ok(())
    }

    async fn handle_callback(&mut self, _: &mut Context, data: &str) -> Result<Jmp> {
        let data = calldata!(data);
        match data {
            Calldata::Back => {
                self.offset = self.offset.saturating_sub(PAGE_SIZE);
            }
            Calldata::Forward => {
                self.offset += PAGE_SIZE;
            }
        }
        Ok(Jmp::None)
    }
}

#[derive(Serialize, Deserialize)]
pub enum Calldata {
    Back,
    Forward,
}
