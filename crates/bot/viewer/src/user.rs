use bot_core::context::Context;
use chrono::{Local, Utc};
use eyre::Error;
use eyre::Result;
use model::decimal::Decimal;
use model::user::employee::Employee;
use model::user::rate::Rate;
use model::{
    rights::Rule,
    subscription::{Status, UserSubscription},
    user::{extension::UserExtension, User},
};
use mongodb::bson::oid::ObjectId;
use teloxide::utils::markdown::escape;

use crate::{day::fmt_date, fmt_phone};

pub fn render_sub(sub: &UserSubscription, is_owner: bool) -> String {
    let now = Utc::now();

    let emoji = if is_owner { "💳" } else { "🎟" };

    match sub.status {
        Status::NotActive => {
            if sub.unlimited {
                format!(
                    "{}_{}_\nБезлимитный абонемент\nНе активен\\. \n",
                    emoji,
                    escape(&sub.name),
                )
            } else {
                format!(
                    "{}_{}_\nОсталось занятий:*{}*\\(_{}_ резерв\\)\nНе активен\\. \n",
                    emoji,
                    escape(&sub.name),
                    sub.balance,
                    sub.locked_balance,
                )
            }
        }
        Status::Active {
            start_date,
            end_date,
        } => {
            let exp = if sub.is_expired(now) {
                "\n*Абонемент истек*"
            } else {
                ""
            };
            if sub.unlimited {
                format!(
                    "{}_{}_\nБезлимитный абонемент\nДействует c _{}_ по _{}_{}",
                    emoji,
                    escape(&sub.name),
                    start_date.with_timezone(&Local).format("%d\\.%m\\.%Y"),
                    end_date.with_timezone(&Local).format("%d\\.%m\\.%Y"),
                    exp
                )
            } else {
                format!(
                    "{}_{}_\nОсталось занятий:*{}*\\(_{}_ резерв\\)\nДействует c _{}_ по _{}_{}",
                    emoji,
                    escape(&sub.name),
                    sub.balance,
                    sub.locked_balance,
                    start_date.with_timezone(&Local).format("%d\\.%m\\.%Y"),
                    end_date.with_timezone(&Local).format("%d\\.%m\\.%Y"),
                    exp
                )
            }
        }
    }
}

pub async fn render_profile_msg(
    ctx: &mut Context,
    id: ObjectId,
) -> Result<(String, User, UserExtension), Error> {
    let user = ctx.ledger.get_user(&mut ctx.session, id).await?;
    let extension = ctx.ledger.users.get_extension(&mut ctx.session, id).await?;

    let mut msg = user_base_info(&user, &extension);
    if ctx.has_right(Rule::ViewMarketingInfo) {
        msg.push_str(&format!("Источник : _{}_\n", user.come_from.name()));
    }

    if let Some(employee) = user.employee.as_ref() {
        render_employee_info(ctx, id, &mut msg, employee);
    } else {
        render_subscriptions(&mut msg, &user)?;
        render_trainings(ctx, &mut msg, &user).await?;
    }
    Ok((msg, user, extension))
}

async fn render_trainings(ctx: &mut Context, msg: &mut String, user: &User) -> Result<(), Error> {
    let trainings = ctx
        .ledger
        .calendar
        .find_trainings(
            &mut ctx.session,
            model::training::Filter::Client(user.id),
            20,
            0,
        )
        .await?;
    if !trainings.is_empty() {
        msg.push_str("\nЗаписи:\n");
        for training in trainings {
            msg.push_str(&escape(&format!(
                "{} {}\n",
                training.get_slot().start_at().format("%d.%m %H:%M"),
                training.name
            )))
        }
        msg.push_str("➖➖➖➖➖➖➖➖➖➖\n");
    }
    Ok(())
}

fn render_subscriptions(msg: &mut String, user: &User) -> Result<()> {
    let payer = user.payer()?;
    let mut subs = payer.subscriptions().to_vec();
    subs.sort_by(|a, b| a.status.cmp(&b.status));

    msg.push_str("Абонементы:\n");

    let has_group = subs.iter().any(|s| !s.tp.is_personal());
    let has_personal = subs.iter().any(|s| s.tp.is_personal());

    if has_group {
        msg.push_str("Групповые:\n");
        for sub in &subs {
            if sub.tp.is_personal() {
                continue;
            }
            msg.push('\n');
            msg.push_str(&render_sub(sub, payer.is_owner()));
        }
        msg.push_str("➖➖➖➖➖➖➖➖➖➖\n");
    }

    if has_personal {
        msg.push_str("Персональные:\n");

        for sub in &subs {
            if !sub.tp.is_personal() {
                continue;
            }
            msg.push('\n');
            msg.push_str(&render_sub(sub, payer.is_owner()));
        }
    }
    if subs.is_empty() {
        msg.push_str("*нет абонементов*🥺\n");
    }
    msg.push_str("➖➖➖➖➖➖➖➖➖➖\n");
    Ok(())
}

pub fn user_base_info(user: &User, extension: &UserExtension) -> String {
    let empty = "?".to_string();

    let freeze = if let Some(freeze) = user.freeze.as_ref() {
        format!(
            "❄️ Заморожен c _{}_  по _{}_",
            fmt_date(&freeze.freeze_start.with_timezone(&Local)),
            fmt_date(&freeze.freeze_end.with_timezone(&Local))
        )
    } else {
        "".to_owned()
    };

    let link = link_to_user(user);

    format!(
        "{} Пользователь : _{}_
*{}* _{}_
Телефон : {}
Дата рождения : _{}_\n
{}\n
{}\n",
        fmt_user_type(user),
        escape(
            &user
                .name
                .tg_user_name
                .as_ref()
                .map(|n| format!("@{n}"))
                .unwrap_or_else(|| empty.to_owned())
        ),
        escape(&user.name.first_name),
        escape(user.name.last_name.as_ref().unwrap_or(&empty)),
        fmt_phone(user.phone.as_deref()),
        escape(
            &extension
                .birthday
                .as_ref()
                .map(|d| d.to_string())
                .unwrap_or_else(|| empty.clone())
        ),
        link,
        freeze
    )
}

pub fn render_rate(rate: &Rate) -> String {
    match rate {
        Rate::Fix {
            amount,
            next_payment_date,
            reward_interval: interval,
        } => {
            format!(
                "Фиксированная сумма : _{}_💰\n Следующая оплата : _{}_\n Интервал : _{}_",
                escape(&amount.to_string()),
                fmt_date(&next_payment_date.with_timezone(&Local)),
                escape(&interval.to_string())
            )
        }
        Rate::GroupTraining {
            percent,
            min_reward,
        } => {
            if percent.is_zero() {
                format!(
                    "Фиксированная сумма за тренировку : _{}_💰",
                    escape(&min_reward.to_string())
                )
            } else {
                format!(
                    "Процент от тренировки : _{}_ %\n Минимальная сумма : _{}_💰",
                    escape(&(*percent * Decimal::from(100)).to_string()),
                    escape(&min_reward.to_string()),
                )
            }
        }
        Rate::PersonalTraining { percent } => {
            format!(
                "Процент от персональной тренировки : _{}_%",
                escape(&(*percent * Decimal::from(100)).to_string())
            )
        }
    }
}

fn render_employee_info(ctx: &mut Context, id: ObjectId, msg: &mut String, employee: &Employee) {
    msg.push_str("➖➖➖➖➖➖➖➖➖➖");
    msg.push_str(&format!("\n[Анкета]({})", escape(&employee.description)));
    if ctx.has_right(Rule::ViewCouchRates) || ctx.is_me(id) {
        msg.push_str(&format!(
            "\nНакопленная награда : _{}_💰\n",
            escape(&employee.reward.to_string()),
        ));
    }

    for rate in &employee.rates {
        msg.push('\n');
        msg.push_str(&render_rate(rate));
    }
}

pub fn fmt_user_type(user: &User) -> &str {
    if user.freeze.is_some() {
        "❄️"
    } else if !user.is_active {
        "⚫"
    } else if user.rights.is_full() {
        "🔴"
    } else if user.employee.is_some() {
        "🔵"
    } else {
        "🟢"
    }
}

pub fn link_to_user(user: &User) -> String {
    if user.tg_id > 0 {
        tg_link(user.tg_id, Some(&user.name.first_name))
    } else {
        user.name
            .tg_user_name
            .as_ref()
            .map(|n| format!("@{}", n))
            .unwrap_or_else(|| "?".to_string())
    }
}

pub fn tg_link(tg: i64, name: Option<&str>) -> String {
    format!(
        " [{}](tg://user?id={}) ",
        escape(name.unwrap_or("профиль")),
        tg
    )
}
