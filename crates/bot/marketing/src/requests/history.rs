use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::{day::fmt_dt, fmt_phone, user::fmt_come_from};
use chrono::Local;
use eyre::Result;
use model::{request::Request, rights::Rule};
use serde::{Deserialize, Serialize};
use teloxide::{types::InlineKeyboardMarkup, utils::markdown::escape};

pub const LIMIT: u64 = 7;

pub struct RequestHistory {
    offset: u64,
}

impl RequestHistory {
    pub fn new() -> Self {
        RequestHistory { offset: 0 }
    }
}

#[async_trait]
impl View for RequestHistory {
    fn name(&self) -> &'static str {
        "HistoryList"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<()> {
        ctx.ensure(Rule::RequestsHistory)?;

        let requests = ctx
            .ledger
            .requests
            .get_all_page(&mut ctx.session, LIMIT as i64, self.offset)
            .await?;
        let mut msg = "*Заявки:*".to_string();
        for req in &requests {
            msg.push_str(&format!("\n\n📌{}", fmt_row(req)));
        }
        let mut keymap = vec![];
        if self.offset > 0 {
            keymap.push(Calldata::Offset(self.offset - LIMIT).button("⬅️"));
        }
        if requests.len() as u64 >= LIMIT {
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

fn fmt_row(request: &Request) -> String {
    format!(
        "Заявка от *{}* ; *{}*\n\
        Комментарий: _{}_\n\
        Дата: _{}_\n",
        fmt_phone(&request.phone),
        fmt_come_from(request.come_from),
        escape(&request.comment),
        fmt_dt(&request.created_at.with_timezone(&Local))
    )
}
