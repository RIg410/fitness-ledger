use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::day::fmt_dt;
use chrono::Local;
use eyre::Result;
use model::history::HistoryRow;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{types::InlineKeyboardMarkup, utils::markdown::escape};

pub const LIMIT: u64 = 7;

pub struct HistoryList {
    id: ObjectId,
    offset: u64,
}

impl HistoryList {
    pub fn new(id: ObjectId) -> Self {
        HistoryList { id, offset: 0 }
    }
}

#[async_trait]
impl View for HistoryList {
    fn name(&self) -> &'static str {
        "HistoryList"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        let logs = ctx
            .ledger
            .history
            .actor_logs(
                &mut ctx.session,
                self.id,
                LIMIT as usize,
                self.offset as usize,
            )
            .await?;
        let mut msg = "*История:*".to_string();
        for log in &logs {
            msg.push_str(&format!("\n\n📌{}", fmt_row(ctx, log).await?));
        }
        let mut keymap = vec![];
        if self.offset > 0 {
            keymap.push(Calldata::Offset(self.offset - LIMIT).button("⬅️"));
        }
        if logs.len() as u64 >= LIMIT {
            keymap.push(Calldata::Offset(self.offset + LIMIT).button("➡️"));
        }

        ctx.edit_origin(&msg, InlineKeyboardMarkup::new(vec![keymap]))
            .await?;
        Ok(())
    }

    async fn handle_callback(&mut self, _: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Calldata::Offset(offset) => {
                self.offset = offset;
                Ok(Jmp::Stay)
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Calldata {
    Offset(u64),
}

async fn fmt_row(ctx: &mut Context, log: &HistoryRow) -> Result<String> {
    let actor = ctx.ledger.get_user(&mut ctx.session, log.actor).await?;
    let is_actor = actor.id == ctx.me.id;
    let message = match &log.action {
        model::history::Action::BlockUser { is_active } => {
            if is_actor {
                if *is_active {
                    format!("Вы заблокировали пользователя {}", actor.name)
                } else {
                    format!("Bы заблокировали пользователя {}", actor.name)
                }
            } else if *is_active {
                format!(
                    "Вас заблокировал пользователь \\(@{}\\)",
                    escape(&actor.name.tg_user_name.unwrap_or_default())
                )
            } else {
                format!(
                    "Вас разблокировал пользователь \\(@{}\\)",
                    escape(&actor.name.tg_user_name.unwrap_or_default())
                )
            }
        }
        model::history::Action::SignUp { start_at, name } => {
            if is_actor {
                format!(
                    "Вы записались на тренировку *{}* на {}",
                    escape(name),
                    fmt_dt(start_at)
                )
            } else {
                format!(
                    "Вас записал на тренировку *{}* в _{}_ пользователь \\(@{}\\)",
                    escape(name),
                    fmt_dt(start_at),
                    escape(&actor.name.tg_user_name.unwrap_or_default())
                )
            }
        }
        model::history::Action::SignOut { start_at, name } => {
            if is_actor {
                format!(
                    "Вы отменили запись на тренировку *{}* на {}",
                    escape(name),
                    fmt_dt(start_at)
                )
            } else {
                format!(
                    "Вас удалили из списка в тренировке *{}* в _{}_ пользователь \\(@{}\\)",
                    escape(name),
                    fmt_dt(start_at),
                    escape(&actor.name.tg_user_name.unwrap_or_default())
                )
            }
        }
        model::history::Action::SellSub { subscription } => {
            if is_actor {
                let sub = if let Some(subject) = log.sub_actors.first() {
                    ctx.ledger
                        .get_user(&mut ctx.session, *subject)
                        .await?
                        .name
                        .to_string()
                } else {
                    "-".to_string()
                };
                format!(
                    "Вы продали абонемент *{}*\nКоличество занятий:_{}_\nCумма:_{}_\nПользователю {}",
                    escape(&subscription.name), subscription.items, escape(&subscription.price.to_string()), escape(&sub)
                )
            } else {
                format!(
                    "Вы купили абонемент *{}*\nКоличество занятий:_{}_\nСумма:_{}_",
                    escape(&subscription.name),
                    subscription.items,
                    escape(&subscription.price.to_string())
                )
            }
        }
        model::history::Action::PreSellSub {
            subscription,
            phone,
        } => {
            if is_actor {
                format!(
                    "Вы продали абонемент *{}*\nКоличество занятий:_{}_\nСумма:_{}_\nПользователю {}",
                    escape(&subscription.name), subscription.items, escape(&subscription.price.to_string()), escape(phone)
                )
            } else {
                format!(
                    "Вы купили абонемент *{}*\nКоличество занятий:_{}_\nСумма:_{}_",
                    escape(&subscription.name),
                    subscription.items,
                    escape(&subscription.price.to_string())
                )
            }
        }
        model::history::Action::SellFreeSub { price, item } => {
            if is_actor {
                let sub = if let Some(subject) = log.sub_actors.first() {
                    ctx.ledger
                        .get_user(&mut ctx.session, *subject)
                        .await?
                        .name
                        .to_string()
                } else {
                    "-".to_string()
                };
                format!(
                    "Вы продали абонемент\nКоличество занятий:_{}_\nСумма:_{}_\nПользователю {}",
                    item,
                    escape(&price.to_string()),
                    escape(&sub)
                )
            } else {
                format!(
                    "Вы купили абонемент\nКоличество занятий:_{}_\nСумма:_{}_",
                    item,
                    escape(&price.to_string())
                )
            }
        }
        model::history::Action::PreSellFreeSub { price, item, buyer } => {
            if is_actor {
                format!(
                    "Вы продали абонемент\nКоличество занятий:_{}_\nСумма:_{}_\nПользователю {}",
                    item,
                    escape(&price.to_string()),
                    escape(buyer)
                )
            } else {
                format!(
                    "Вы купили абонемент\nКоличество занятий:_{}_\nСумма:_{}_",
                    item,
                    escape(&price.to_string())
                )
            }
        }
        model::history::Action::FinalizedCanceledTraining { name, start_at } => {
            format!(
                "Тренировка *{}* в _{}_ отменена",
                escape(name),
                fmt_dt(&start_at.with_timezone(&Local))
            )
        }
        model::history::Action::FinalizedTraining { name, start_at } => {
            if is_actor {
                format!(
                    "Вы провели тренировку *{}* в _{}_",
                    escape(name),
                    fmt_dt(&start_at.with_timezone(&Local))
                )
            } else {
                format!(
                    "Вас посетили на тренировку *{}* в _{}_",
                    escape(name),
                    fmt_dt(&start_at.with_timezone(&Local))
                )
            }
        }
        model::history::Action::Payment {
            amount,
            description,
            date_time,
        } => {
            format!(
                "Вы произвели оплату *{}* в _{}_\n{}",
                escape(&amount.to_string()),
                fmt_dt(&date_time.with_timezone(&Local)),
                escape(description)
            )
        }
        model::history::Action::Deposit {
            amount,
            description,
            date_time,
        } => {
            format!(
                "Вы внесли депозит *{}* в _{}_\n{}",
                escape(&amount.to_string()),
                fmt_dt(&date_time.with_timezone(&Local)),
                escape(description)
            )
        }
        model::history::Action::CreateUser { name, phone } => {
            format!(
                "Регистрация *{}*\nТелефон: _{}_",
                escape(&name.to_string()),
                escape(phone)
            )
        }
        model::history::Action::Freeze { days } => {
            let sub = if let Some(subject) = log.sub_actors.first() {
                ctx.ledger
                    .get_user(&mut ctx.session, *subject)
                    .await?
                    .name
                    .to_string()
            } else {
                "-".to_string()
            };
            if is_actor {
                format!(
                    "Вы заморозили абонемент пользователя _{}_ на _{}_ дней",
                    escape(&sub),
                    days
                )
            } else {
                format!("Ваш абонемент заморозили на _{}_ дней", days)
            }
        }
        model::history::Action::Unfreeze {} => {
            let sub = if let Some(subject) = log.sub_actors.first() {
                ctx.ledger
                    .get_user(&mut ctx.session, *subject)
                    .await?
                    .name
                    .to_string()
            } else {
                "-".to_string()
            };
            if is_actor {
                format!("Вы разморозили абонемент пользователя _{}_", escape(&sub))
            } else {
                "Ваш абонемент разморозили".to_string()
            }
        }
        model::history::Action::ChangeBalance { amount } => {
            let sub = if let Some(subject) = log.sub_actors.first() {
                ctx.ledger
                    .get_user(&mut ctx.session, *subject)
                    .await?
                    .name
                    .to_string()
            } else {
                "-".to_string()
            };
            if is_actor {
                format!(
                    "Вы изменили баланс пользователя {} на _{}_ занятий",
                    escape(&sub),
                    escape(&amount.to_string())
                )
            } else {
                format!("Ваш баланс изменен на _{}_ занятий", escape(&amount.to_string()))
            }
        }
        model::history::Action::ChangeReservedBalance { amount } => {
            let sub = if let Some(subject) = log.sub_actors.first() {
                ctx.ledger
                    .get_user(&mut ctx.session, *subject)
                    .await?
                    .name
                    .to_string()
            } else {
                "-".to_string()
            };
            if is_actor {
                format!(
                    "Вы изменили резерв пользователя {} на _{}_ занятий",
                    escape(&sub),
                    escape(&amount.to_string())
                )
            } else {
                format!("Ваш резерв изменен на _{}_ занятий", escape(&amount.to_string()))
            }
        }
        model::history::Action::PayReward { amount } => {
            let sub = if let Some(subject) = log.sub_actors.first() {
                ctx.ledger
                    .get_user(&mut ctx.session, *subject)
                    .await?
                    .name
                    .to_string()
            } else {
                "-".to_string()
            };
            if is_actor {
                format!(
                    "Вы выплатили вознаграждение в размере *{}* пользователю {}",
                    escape(&amount.to_string()),
                    escape(&sub)
                )
            } else {
                format!(
                    "Вам выплатили вознаграждение в размере *{}*",
                    escape(&amount.to_string())
                )
            }
        }
        model::history::Action::ExpireSubscription { subscription } => {
            format!(
                "Абонемент *{}* пользователя _{}_ истек",
                escape(&subscription.name),
                escape(&actor.name.tg_user_name.unwrap_or_default())
            )
        }
    };

    Ok(format!(
        "{}:\n{}",
        fmt_dt(&log.date_time.with_timezone(&Local)),
        message
    ))
}
