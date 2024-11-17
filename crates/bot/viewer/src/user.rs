use bot_core::context::Context;
use chrono::Local;
use eyre::Error;
use model::{
    couch::{CouchInfo, GroupRate, PersonalRate},
    rights::Rule,
    statistics::marketing::ComeFrom,
    subscription::{Status, UserSubscription},
    user::{extension::UserExtension, User},
};
use mongodb::bson::oid::ObjectId;
use teloxide::utils::markdown::escape;

use crate::{day::fmt_date, fmt_phone};

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
        Status::Active {
            start_date,
            end_date,
        } => {
            format!(
                "🎟_{}_\nОсталось занятий:*{}*\\(_{}_ резерв\\)\nДействует c _{}_ по _{}_\n",
                escape(&sub.name),
                sub.balance,
                sub.locked_balance,
                start_date.with_timezone(&Local).format("%d\\.%m\\.%Y"),
                end_date.with_timezone(&Local).format("%d\\.%m\\.%Y")
            )
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
        msg.push_str(&format!("Источник : _{}_\n", fmt_come_from(user.come_from)));
    }

    if let Some(couch) = user.couch.as_ref() {
        render_couch_info(ctx, id, &mut msg, couch);
    } else {
        render_subscriptions(&mut msg, &user);
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
        fmt_phone(&user.phone),
        escape(
            &extension
                .birthday
                .as_ref()
                .map(|d| d.dt.format("%d.%m.%Y").to_string())
                .unwrap_or_else(|| empty.clone())
        ),
        link,
        freeze
    )
}

fn render_couch_info(ctx: &mut Context, id: ObjectId, msg: &mut String, couch: &CouchInfo) {
    msg.push_str("➖➖➖➖➖➖➖➖➖➖");
    msg.push_str(&format!("\n[Анкета]({})", escape(&couch.description)));
    if ctx.has_right(Rule::ViewCouchRates) || ctx.is_me(id) {
        msg.push_str(&format!(
            "\nНакопленная награда : _{}_💰\n{}\n{}\n",
            escape(&couch.reward.to_string()),
            fmt_group_rate(&couch.group_rate),
            fmt_personal_rate(&couch.personal_rate),
        ));
    }
}

pub fn fmt_group_rate(rate: &GroupRate) -> String {
    match rate {
        GroupRate::FixedMonthly { rate, next_reward } => {
            format!(
                "Фиксированный месячный тариф : _{}_💰\nСледующая награда : _{}_\n",
                escape(&rate.to_string()),
                next_reward.with_timezone(&Local).format("%d\\.%m\\.%Y")
            )
        }
        GroupRate::PerClient { min, per_client } => {
            format!(
                "За клиента : _{}_💰\nМинимальная награда : _{}_💰\n",
                escape(&per_client.to_string()),
                escape(&min.to_string())
            )
        }
        GroupRate::None => "Тариф не определен".to_string(),
    }
}

pub fn fmt_personal_rate(rate: &PersonalRate) -> String {
    format!(
        "Вознаграждение за персональные тренировки : _{}%_💰",
        escape(&rate.couch_interest.to_string())
    )
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

pub fn link_to_user(user: &User) -> String {
    if user.tg_id > 0 {
        tg_link(user.tg_id)
    } else {
        user.name
            .tg_user_name
            .as_ref()
            .map(|n| format!("@{}", n))
            .unwrap_or_else(|| "?".to_string())
    }
}

pub fn tg_link(tg: i64) -> String {
    format!(" [🔗Профиль](tg://user?id={}) ", tg)
}

pub fn fmt_come_from(from: ComeFrom) -> &'static str {
    match from {
        ComeFrom::Unknown {} => "Неизвестно",
        ComeFrom::DoubleGIS {} => "2ГИС",
        ComeFrom::Website {} => "Сайт",
        ComeFrom::Instagram {} => "Инстаграм",
        ComeFrom::VK {} => "ВКонтакте",
        ComeFrom::YandexMap {} => "Яндекс Карты",
        ComeFrom::DirectAdds {} => "Прямые рекламные объявления",
        ComeFrom::VkAdds {} => "Таргет ВКонтакте",
        ComeFrom::YandexDirect {} => "Яндекс Директ",
    }
}
