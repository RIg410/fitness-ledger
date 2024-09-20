use bot_core::context::Context;
use chrono::Local;
use eyre::Error;
use model::{
    couch::{CouchInfo, Rate},
    rights::Rule,
    subscription::{Status, UserSubscription},
    user::{User, UserIdent},
};
use teloxide::utils::markdown::escape;

pub fn render_sub(sub: &UserSubscription) -> String {
    match sub.status {
        Status::NotActive => {
            format!(
                "🎟_{}_\nОсталось занятий:_{}_\nНе активен\\. \n",
                escape(&sub.name),
                sub.items,
            )
        }
        Status::Active { start_date } => {
            let end_date = start_date + chrono::Duration::days(i64::from(sub.days));
            format!(
                "🎟_{}_\nОсталось занятий:_{}_\nДействует до:_{}_\n",
                escape(&sub.name),
                sub.items,
                end_date.with_timezone(&Local).format("%d\\.%m\\.%Y")
            )
        }
    }
}

pub async fn render_profile_msg<ID: Into<UserIdent> + Copy>(
    ctx: &mut Context,
    id: ID,
) -> Result<(String, User), Error> {
    let user = ctx.ledger.get_user(&mut ctx.session, id).await?;

    let mut msg = user_base_info(&user);
    if let Some(couch) = user.couch.as_ref() {
        render_couch_info(ctx, id, &mut msg, couch);
    } else {
        render_balance_info(&mut msg, &user, ctx.has_right(Rule::ViewProfile));
        render_subscriptions(&mut msg, &user);
        render_trainings(ctx, &mut msg, &user).await?;
    }
    Ok((msg, user))
}

async fn render_trainings(ctx: &mut Context, msg: &mut String, user: &User) -> Result<(), Error> {
    let trainings = ctx
        .ledger
        .calendar
        .get_users_trainings(&mut ctx.session, user.id, 100, 0)
        .await?;
    if !trainings.is_empty() {
        msg.push_str("Записи:\n");
        for training in trainings {
            msg.push_str(&escape(&format!(
                "{} {}\n",
                training
                    .start_at
                    .with_timezone(&Local)
                    .format("%d.%m %H:%M"),
                training.name
            )))
        }
        msg.push_str("➖➖➖➖➖➖➖➖➖➖\n");
    }
    Ok(())
}

fn render_subscriptions(msg: &mut String, user: &User) {
    let mut subs = user.subscriptions.iter().collect::<Vec<_>>();
    subs.sort_by(|a, b| a.status.cmp(&b.status));
    msg.push_str("Абонементы:\n");
    if !subs.is_empty() {
        for sub in subs {
            msg.push_str(&render_sub(sub));
        }
    } else {
        if user.balance == 0 && user.reserved_balance == 0 {
            msg.push_str("*нет абонементов*🥺\n");
        } else {
            msg.push_str(&format!(
                "🎟_тестовый_\nОсталось занятий:_{}_\n",
                user.balance + user.reserved_balance
            ));
        }
    }
    msg.push_str("➖➖➖➖➖➖➖➖➖➖");
}

fn render_balance_info(msg: &mut String, user: &User, sys_info: bool) {
    msg.push_str("➖➖➖➖➖➖➖➖➖➖\n");
    let sys_info = if sys_info {
        format!("\n*Резерв : _{}_ занятий*", user.reserved_balance)
    } else {
        "".to_owned()
    };
    msg.push_str(&format!(
        "*Баланс : _{}_ занятий*{}\n",
        user.balance, sys_info
    ));
}

pub fn user_base_info(user: &User) -> String {
    let empty = "?".to_string();
    format!(
        "{} Пользователь : _@{}_
Имя : _{}_
Фамилия : _{}_
Телефон : _\\+{}_
Дата рождения : _{}_\n",
        fmt_user_type(&user),
        escape(user.name.tg_user_name.as_ref().unwrap_or_else(|| &empty)),
        escape(&user.name.first_name),
        escape(&user.name.last_name.as_ref().unwrap_or_else(|| &empty)),
        escape(&user.phone),
        escape(
            &user
                .birthday
                .as_ref()
                .map(|d| d.format("%d.%m.%Y").to_string())
                .unwrap_or_else(|| empty.clone())
        ),
    )
}

fn render_couch_info<ID: Into<UserIdent>>(
    ctx: &mut Context,
    id: ID,
    msg: &mut String,
    couch: &CouchInfo,
) {
    msg.push_str("➖➖➖➖➖➖➖➖➖➖");
    msg.push_str(&format!("\n[Анкета]({})", escape(&couch.description)));
    if ctx.has_right(Rule::ViewCouchRates) || ctx.is_me(id) {
        msg.push_str(&format!(
            "\nНакопленная награда : _{}_💰\n{}\n",
            escape(&couch.reward.to_string()),
            fmt_rate(&couch.rate)
        ));
    }
}

pub fn fmt_rate(rate: &Rate) -> String {
    match rate {
        Rate::FixedMonthly { rate, next_reward } => {
            format!(
                "Фиксированный месячный тариф : _{}_💰\nСледующая награда : _{}_\n",
                escape(&rate.to_string()),
                next_reward.with_timezone(&Local).format("%d\\.%m\\.%Y")
            )
        }
        Rate::PerClient { min, per_client } => {
            format!(
                "За клиента : _{}_💰\nМинимальная награда : _{}_💰\n",
                escape(&per_client.to_string()),
                escape(&min.to_string())
            )
        }
        Rate::None => "Тариф не определен".to_string(),
    }
}

pub fn fmt_user_type(user: &User) -> &str {
    if user.freeze.is_some() {
        "❄️"
    } else if !user.is_active {
        "⚫"
    } else if user.rights.is_full() {
        "🔴"
    } else if user.couch.is_some() {
        "🔵"
    } else {
        "🟢"
    }
}
