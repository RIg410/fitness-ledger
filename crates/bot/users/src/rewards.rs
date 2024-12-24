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
use model::{reward::{Reward, RewardSource}, rights::Rule};
use mongodb::bson::oid::ObjectId;
use recalc::AddRecalcReward;
use serde::{Deserialize, Serialize};
use teloxide::{types::InlineKeyboardMarkup, utils::markdown::escape};

mod recalc;

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

        let mut keymap = InlineKeyboardMarkup::new(vec![keymap]);
        if ctx.has_right(Rule::RecalculateRewards) {
            keymap = keymap.append_row(Calldata::Recalculate.btn_row("Добавить перерасчет"));
        }

        ctx.edit_origin(&msg, keymap).await?;
        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Calldata::Offset(offset) => {
                self.offset = offset;
                Ok(Jmp::Stay)
            }
            Calldata::Recalculate => {
                ctx.ensure(Rule::RecalculateRewards)?;
                Ok(Jmp::Next(AddRecalcReward::new(self.id).into()))
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Calldata {
    Offset(u64),
    Recalculate,
}

fn fmt_row(log: &Reward) -> String {
    match &log.source {
        RewardSource::Training {
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
        RewardSource::FixedMonthly {} => {
            format!(
                "*{}*\n начислено *{}* \\- _ежемесячное вознаграждение_",
                fmt_dt(&log.created_at.with_timezone(&Local)),
                escape(&log.reward.to_string())
            )
        }
        RewardSource::Recalc { comment } => {
            format!(
                "*{}*\n начислено *{}* \\- _перерасчет_ \\- {}",
                fmt_dt(&log.created_at.with_timezone(&Local)),
                escape(&log.reward.to_string()),
                escape(comment)
            )
        }
        RewardSource::TrainingV2 { training_id, name, details } => {
            format!(
                "*{}*\n начислено *{}* \\- тренировка '{}' \\- {}",
                fmt_dt(&log.created_at.with_timezone(&Local)),
                escape(&log.reward.to_string()),
                escape(name),
                fmt_dt(&training_id.start_at.with_timezone(&Local)),
            )
        }
        RewardSource::Fixed {} => {
            format!(
                "*{}*\n начислено *{}* \\- _фиксированное вознаграждение_",
                fmt_dt(&log.created_at.with_timezone(&Local)),
                escape(&log.reward.to_string())
            )
        }
    }
}
