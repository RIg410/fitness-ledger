use std::collections::HashMap;

use async_trait::async_trait;
use bot_core::{
    callback_data::Calldata as _,
    calldata,
    context::Context,
    widget::{Jmp, View},
};
use bot_viewer::{
    day::{fmt_date, fmt_weekday},
    fmt_phone,
    user::fmt_come_from,
};
use chrono::{Local, Weekday};
use eyre::Error;
use eyre::Result;
use itertools::Itertools;
use model::{
    rights::Rule,
    statistics::{
        calendar::{EntryInfo, TimeSlot, UserStat},
        history::SubscriptionStatistics,
    },
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use teloxide::{types::InlineKeyboardMarkup, utils::markdown::escape};
use time::range::Range;

pub struct StatisticsView {
    range: Range,
}

impl Default for StatisticsView {
    fn default() -> Self {
        Self {
            range: Range::Month(Local::now()),
        }
    }
}

impl StatisticsView {
    async fn print_statistics(&self, ctx: &mut Context) -> Result<()> {
        let stat = ctx
            .ledger
            .statistics
            .calendar(&mut ctx.session, None, None)
            .await?;
        ctx.send_notification("📊Статистика посещений:").await?;
        by_program(ctx, stat.by_program).await?;
        by_weekday(ctx, stat.by_weekday).await?;
        by_instructor(ctx, stat.by_instructor).await?;
        by_time_slot(ctx, stat.by_time_slot).await?;
        user_stat(ctx, stat.users).await?;

        let stat = ctx
            .ledger
            .statistics
            .subscriptions(&mut ctx.session, None, None)
            .await?;
        subscriptions(ctx, &stat).await?;
        Ok(())
    }
}

#[async_trait]
impl View for StatisticsView {
    fn name(&self) -> &'static str {
        "StatisticsView"
    }

    async fn show(&mut self, ctx: &mut Context) -> Result<(), Error> {
        ctx.ensure(Rule::ViewStatistics)?;
        let keymap = InlineKeyboardMarkup::default()
            .append_row(vec![
                Calldata::PrevMonth.button("⬅️"),
                Calldata::NextMonth.button("➡️"),
            ])
            .append_row(Calldata::Full.btn_row("За все время"))
            .append_row(Calldata::Request.btn_row("Получить статистику📊"));

        let (from, to) = self.range.range();
        ctx.edit_origin(
            &format!(
                "📊Статистика с _{}_ по _{}_",
                from.map(|f| f.format("%d\\.%m\\.%Y").to_string())
                    .unwrap_or_else(|| "-".to_string()),
                to.map(|f| f.format("%d\\.%m\\.%Y").to_string())
                    .unwrap_or_else(|| "-".to_string())
            ),
            keymap,
        )
        .await?;

        Ok(())
    }

    async fn handle_callback(&mut self, ctx: &mut Context, data: &str) -> Result<Jmp> {
        match calldata!(data) {
            Calldata::NextMonth => {
                if !self.range.is_month() {
                    self.range = Range::Month(Local::now());
                } else {
                    self.range.next_month();
                }
            }
            Calldata::PrevMonth => {
                if !self.range.is_month() {
                    self.range = Range::Month(Local::now());
                } else {
                    self.range.prev_month();
                }
            }
            Calldata::Request => {
                self.print_statistics(ctx).await?;
            }
            Calldata::Full => {
                self.range = Range::Full;
            }
        }
        Ok(Jmp::Stay)
    }
}

#[derive(Serialize, Deserialize)]
enum Calldata {
    NextMonth,
    PrevMonth,
    Full,
    Request,
}

async fn subscriptions(ctx: &mut Context, stat: &SubscriptionStatistics) -> Result<(), Error> {
    let mut msg = "📊Статистика по абонементам".to_string();
    msg.push_str(&format!(
        "\nc _{}_ по _{}_:",
        fmt_date(&stat.from),
        fmt_date(&stat.to)
    ));

    msg.push_str(&format!(
        "\n\nВсего продано абонементов: *{}* на сумму *{}*",
        stat.subs_count,
        escape(&stat.total_subs_sum.to_string())
    ));

    for sub in &stat.subs {
        msg.push_str(&format!(
            "\n\n\n📚{}:\nВсего продано: *{}* на сумму *{}*",
            escape(&sub.name),
            sub.total,
            escape(&sub.sum.to_string())
        ));
    }

    msg.push_str(&format!(
        "\n\n\nПродано пробных занятий: *{}* из них купили абонимент: *{}*",
        stat.test_subs_count, stat.users_buy_test_sub_and_stay,
    ));

    if stat.unresolved_presells > 0 {
        msg.push_str(&format!(
            "\n\nКупили пробное занятия но не зашли в бот:*{}*",
            stat.unresolved_presells
        ));
    }

    if stat.people_buys_only_test_sub.len() > 0 {
        msg.push_str("\n\nКупили только пробное занятие:");
        for id in &stat.people_buys_only_test_sub {
            let user = ctx.ledger.users.get(&mut ctx.session, *id).await?;
            if let Some(user) = user {
                msg.push_str(&format!("\n👤{}", fmt_phone(&user.phone)));
            } else {
                msg.push_str(&format!("\n👤{}", id));
            }
        }
    }

    if stat.come_from.len() > 0 {
        msg.push_str("\n\nОткуда пришли:");
        for (come_from, stat) in &stat.come_from {
            msg.push_str(&format!(
                "\n\n📚{}:\nВсего пользователей: *{}* на сумму *{}*\nкупили тестовое:*{}*\nкупили абонемент:*{}*",
                fmt_come_from(*come_from),
                stat.total_users,
                escape(&stat.sum.to_string()),
                stat.buy_test_subs,
                stat.buy_subs,
            ));
        }
    }

    if stat.people_without_subs.len() > 0 {
        msg.push_str(&format!(
            "\n\nКлиенты без абонементов {}:",
            stat.people_without_subs.len()
        ));
        for id in &stat.people_without_subs {
            let user = ctx.ledger.users.get(&mut ctx.session, *id).await?;
            if let Some(user) = user {
                msg.push_str(&format!("\n👤{}", fmt_phone(&user.phone)));
            } else {
                msg.push_str(&format!("\n👤{}", id));
            }
        }
    }

    msg.push_str(&format!("\n👥Всего клиентов: *{}*", stat.total_users));
    msg.push_str(&format!(
        "\nКлиентов с абонементами: *{}*",
        stat.total_users as usize - stat.people_without_subs.len()
    ));
    ctx.send_notification(&msg).await?;
    Ok(())
}

async fn user_stat(ctx: &mut Context, users: HashMap<ObjectId, UserStat>) -> Result<(), Error> {
    if users.is_empty() {
        return Ok(());
    }
    let mut msg = "📊Статистика по клиентам \\(ТОП 10\\):".to_string();
    let mut users: Vec<(ObjectId, UserStat)> = users.into_iter().collect();
    users.sort_by(|a, b| b.1.total.cmp(&a.1.total));

    for idx in 0..users.len().min(10) {
        let (id, stat) = &users[idx];
        let user_name = user_name(ctx, *id).await?;
        msg.push_str(&format!(
            "\n\n\n👤{}:\n📅Посещено тренировок: _{}_\nПо дням: {}\nПо инструкторам:{}\nПо времени:{}\nПо программе:{}",
            escape(&user_name),
            stat.total,
            user_by_day(&stat.weekdays, stat.total),
            user_by_instructor(ctx, &stat.instructors, stat.total).await?,
            user_by_time_slot(&stat.time_slots, stat.total),
            user_by_program(ctx, &stat.programs, stat.total).await?
        ));
    }

    ctx.send_notification(&msg).await?;
    Ok(())
}

async fn user_by_program(
    ctx: &mut Context,
    program: &HashMap<ObjectId, u32>,
    total: u32,
) -> Result<String, Error> {
    let mut program: Vec<(ObjectId, u32)> = program.iter().map(|f| (*f.0, *f.1)).collect();
    program.sort_by(|a, b| b.1.cmp(&a.1));
    let mut msg = "".to_string();
    for (id, count) in program {
        let name = program_name(ctx, id).await?;
        msg.push_str(&format!(
            "\n 📚{}:_{}%_",
            escape(&name),
            (count as f64 / total as f64 * 100.0).round()
        ));
    }
    Ok(msg)
}

fn user_by_time_slot(time_slot: &HashMap<TimeSlot, u32>, total: u32) -> String {
    let mut time_slot: Vec<(TimeSlot, u32)> = time_slot.iter().map(|f| (*f.0, *f.1)).collect();
    time_slot.sort_by(|a, b| b.1.cmp(&a.1));
    time_slot
        .into_iter()
        .map(|(day, count)| {
            format!(
                "\n *{}*:_{}%_",
                day,
                (count as f64 / total as f64 * 100.0).round()
            )
        })
        .join(", ")
}

fn user_by_day(weekdays: &HashMap<Weekday, u32>, total: u32) -> String {
    let mut days: Vec<(Weekday, u32)> = weekdays.iter().map(|f| (*f.0, *f.1)).collect();
    days.sort_by(|a, b| b.1.cmp(&a.1));
    days.into_iter()
        .map(|(day, count)| {
            format!(
                "\n *{}*:_{}%_",
                fmt_weekday(day),
                (count as f64 / total as f64 * 100.0).round()
            )
        })
        .join(", ")
}

async fn user_by_instructor(
    ctx: &mut Context,
    instructors: &HashMap<ObjectId, u32>,
    total: u32,
) -> Result<String, Error> {
    let mut instructors: Vec<(ObjectId, u32)> = instructors.iter().map(|f| (*f.0, *f.1)).collect();
    instructors.sort_by(|a, b| b.1.cmp(&a.1));
    let mut msg = "".to_string();
    for (id, count) in instructors {
        let name = instructor_name(ctx, id).await?;
        msg.push_str(&format!(
            "\n 👤{}:_{}%_",
            escape(&name),
            (count as f64 / total as f64 * 100.0).round()
        ));
    }
    Ok(msg)
}

async fn by_time_slot(ctx: &mut Context, stat: HashMap<TimeSlot, EntryInfo>) -> Result<(), Error> {
    if stat.is_empty() {
        return Ok(());
    }
    let mut msg = "📊Статистика по времени:".to_string();
    let mut stat: Vec<(TimeSlot, EntryInfo, f64)> = stat
        .into_iter()
        .map(|(id, info)| {
            let avg = info.avg_visits();
            (id, info, avg)
        })
        .collect();

    stat.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    for (slot, info, avg) in stat {
        msg.push_str(&format!(
            "\n\n\n🕒{}:\n{}\nСредняя посещаемость:{}",
            slot,
            print_entry_info(&info),
            avg.round()
        ));
    }
    ctx.send_notification(&msg).await?;
    Ok(())
}

async fn by_instructor(ctx: &mut Context, stat: HashMap<ObjectId, EntryInfo>) -> Result<(), Error> {
    if stat.is_empty() {
        return Ok(());
    }
    let mut msg = "📊Статистика по инструкторам:".to_string();
    let mut stat: Vec<(ObjectId, EntryInfo, f64)> = stat
        .into_iter()
        .map(|(id, info)| {
            let avg = info.avg_visits();
            (id, info, avg)
        })
        .collect();

    stat.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    for (id, info, avg) in stat {
        let name = instructor_name(ctx, id).await?;
        msg.push_str(&format!(
            "\n\n\n👤{}:\n{}\nСредняя посещаемость:{}",
            escape(&name),
            print_entry_info(&info),
            avg.round()
        ));
    }
    ctx.send_notification(&msg).await?;
    Ok(())
}

async fn by_program(ctx: &mut Context, stat: HashMap<ObjectId, EntryInfo>) -> Result<(), Error> {
    if stat.is_empty() {
        return Ok(());
    }
    let mut msg = "📊Статистика по программам:".to_string();
    let mut stat: Vec<(ObjectId, EntryInfo, f64)> = stat
        .into_iter()
        .map(|(id, info)| {
            let avg = info.avg_visits();
            (id, info, avg)
        })
        .collect();

    stat.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    for (id, info, avg) in stat {
        let program = program_name(ctx, id).await?;
        msg.push_str(&format!(
            "\n\n\n📚{}:\n{}\nСредняя посещаемость:{}",
            escape(&program),
            print_entry_info(&info),
            avg.round()
        ));
    }
    ctx.send_notification(&msg).await?;
    Ok(())
}

async fn by_weekday(
    ctx: &mut Context,
    by_weekday: HashMap<Weekday, EntryInfo>,
) -> Result<(), Error> {
    if by_weekday.is_empty() {
        return Ok(());
    }

    let mut msg = "📊Статистика по дням недели:".to_string();
    let mut by_weekday: Vec<(Weekday, EntryInfo, f64)> = by_weekday
        .into_iter()
        .map(|(id, info)| {
            let avg = info.avg_visits();
            (id, info, avg)
        })
        .collect();

    by_weekday.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    for (weekday, info, avg) in by_weekday {
        msg.push_str(&format!(
            "\n\n\n📅{}:\n{}\nСредняя посещаемость:{}",
            fmt_weekday(weekday),
            print_entry_info(&info),
            avg.round()
        ));
    }
    ctx.send_notification(&msg).await?;
    Ok(())
}

fn print_entry_info(entry_info: &EntryInfo) -> String {
    format!(
        "📅Всего тренировок: _{}_\n💰Заработано: _{}_\n🎁Награда инструкторов: _{}_\n👥Посещений: _{}_\n🚫Тренеровок без клиентов: _{}_",
        entry_info.total_training,
        escape(&entry_info.earn.to_string()),
        escape(&entry_info.reward.to_string()),
        entry_info.visit,
        entry_info.without_clients
    )
}

async fn program_name(ctx: &mut Context, id: ObjectId) -> Result<String, Error> {
    let program = ctx.ledger.programs.get_by_id(&mut ctx.session, id).await?;
    Ok(program.map(|p| p.name).unwrap_or_else(|| id.to_string()))
}

async fn instructor_name(ctx: &mut Context, id: ObjectId) -> Result<String, Error> {
    let user = ctx.ledger.users.get(&mut ctx.session, id).await?;
    Ok(user
        .map(|u| u.name.first_name)
        .unwrap_or_else(|| id.to_string()))
}

async fn user_name(ctx: &mut Context, id: ObjectId) -> Result<String, Error> {
    let user = ctx.ledger.users.get(&mut ctx.session, id).await?;
    if let Some(user) = user {
        Ok(user.name.to_string())
    } else {
        Ok(id.to_string())
    }
}
