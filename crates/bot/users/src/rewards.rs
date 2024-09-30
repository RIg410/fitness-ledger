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
use model::{couch::Reward, rights::Rule};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{types::InlineKeyboardMarkup, utils::markdown::escape};

pub const LIMIT: u64 = 7;

pub struct RewardsList {
    id: ObjectId,
    offset: u64,
}

impl RewardsList {
    pub fn new(id: ObjectId) -> Self {
        RewardsList { id, offset: 0 }
    }
}

#[async_trait]
impl View for RewardsList {
    fn name(&self) -> &'static str {
        "HistoryList"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        if !ctx.is_me(self.id) && !ctx.has_right(Rule::ViewRewards) {
            return Err(eyre::eyre!("Недостаточно прав"));
        }

        let logs = ctx
            .ledger
            .rewards
            .get(&mut ctx.session, self.id, LIMIT as i64, self.offset)
            .await?;
        let mut msg = "*История начислений:*".to_string();
        for log in &logs {
            msg.push_str(&format!("\n\n📌{}", fmt_row(log)));
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

fn fmt_row(log: &Reward) -> String {
    match &log.source {
        model::couch::RewardSource::Training {
            start_at,
            clients,
            name,
        } => {
            format!(
                "*{}*\n начислено *{}* \\- тренировка '{}' '{}' клиентов \\- {}",
                fmt_dt(&log.created_at.with_timezone(&Local)),
                escape(&log.reward.to_string()),
                escape(name),
                fmt_dt(&start_at.with_timezone(&Local)),
                clients
            )
        }
        model::couch::RewardSource::FixedMonthly {} => {
            format!(
                "*{}*\n начислено *{}* \\- _ежемесячное вознаграждение_",
                fmt_dt(&log.created_at.with_timezone(&Local)),
                escape(&log.reward.to_string())
            )
        }
    }
}