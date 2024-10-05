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
                "🎟_{}_\nОсталось занятий:*{}*\\(_{}_ резерв\\)\nНе активен\\. \n",
                escape(&sub.name),
                sub.balance,
                sub.locked_balance,
            )
        }
        Status::Active { start_date } => {
            let end_date = start_date + chrono::Duration::days(i64::from(sub.days));
            format!(
                "🎟_{}_\nОсталось занятий:*{}*\\(_{}_ резерв\\)\nДействует до:_{}_\n",
                escape(&sub.name),
                sub.balance,
                sub.locked_balance,
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
        render_subscriptions(&mut msg, &user);
        render_trainings(ctx, &mut msg, &user).await?;
    }
    Ok((msg, user))
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

    let has_group = subs.iter().any(|s| !s.tp.is_personal());
    let has_personal = subs.iter().any(|s| s.tp.is_personal());

    if has_group {
        msg.push_str("Групповые:\n");
        for sub in &subs {
            if sub.tp.is_personal() {
                continue;
            }
            msg.push_str("\n");
            msg.push_str(&render_sub(sub));
        }
        msg.push_str("➖➖➖➖➖➖➖➖➖➖\n");
    }

    if has_personal {
        msg.push_str("Персональные:\n");

        for sub in &subs {
            if !sub.tp.is_personal() {
                continue;
            }
            msg.push_str("\n");
            msg.push_str(&render_sub(sub));
        }
    }
    if subs.is_empty() {
        msg.push_str("*нет абонементов*🥺\n");
    }
    msg.push_str("➖➖➖➖➖➖➖➖➖➖\n");
}

pub fn user_base_info(user: &User) -> String {
    let empty = "?".to_string();
    format!(
        "{} Пользователь : _@{}_
Имя : _{}_
Фамилия : _{}_
Телефон : _\\+{}_
Дата рождения : _{}_\n",
        fmt_user_type(user),
        escape(user.name.tg_user_name.as_ref().unwrap_or(&empty)),
        escape(&user.name.first_name),
        escape(user.name.last_name.as_ref().unwrap_or(&empty)),
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
