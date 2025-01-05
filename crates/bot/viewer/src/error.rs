use bot_core::context::Context;
use eyre::Result;
use model::{errors::LedgerError, user::rate::Rate};
use mongodb::bson::oid::ObjectId;
use teloxide::utils::markdown::escape;

pub async fn notify(
    proc: &str,
    result: Result<(), LedgerError>,
    ok: &str,
    ctx: &mut Context,
) -> Result<(), LedgerError> {
    match result {
        Ok(_) => {
            ctx.send_notification(ok).await?;
            Ok(())
        }
        Err(err) => match bassness_error(ctx, &err).await? {
            Some(msg) => {
                ctx.send_notification(&format!("{}: *{}*", proc, msg))
                    .await?;
                Ok(())
            }
            None => Err(err),
        },
    }
}

pub async fn bassness_error(ctx: &mut Context, err: &LedgerError) -> Result<Option<String>> {
    Ok(Some(escape(&match err {
        LedgerError::Eyre(_) => return Ok(None),
        LedgerError::UserNotFound(object_id) => format!("Пользователь {} не найден", object_id),
        LedgerError::MemberNotFound { user_id, member_id } => {
            let user = user_name(ctx, *user_id).await?;
            let member = user_name(ctx, *member_id).await?;
            format!(
                "Пользователь {} не найден в семье пользователя {}",
                member, user
            )
        }
        LedgerError::WrongFamilyMember { user_id, member_id } => {
            let user = user_name(ctx, *user_id).await?;
            let member = user_name(ctx, *member_id).await?;
            format!(
                "Пользователь {} не является членом семьи пользователя {}",
                member, user
            )
        }
        LedgerError::MongoError(_) => return Ok(None),
        LedgerError::UserAlreadyInFamily { user_id, member_id } => {
            let user = user_name(ctx, *user_id).await?;
            let member = user_name(ctx, *member_id).await?;
            format!(
                "Пользователь {} уже является членом семьи пользователя {}",
                member, user
            )
        }
        LedgerError::UserAlreadyEmployee { user_id } => format!(
            "Пользователь {} уже является сотрудником",
            user_name(ctx, *user_id).await?
        ),
        LedgerError::UserNotEmployee { user_id } => format!(
            "Пользователь {} не является сотрудником",
            user_name(ctx, *user_id).await?
        ),
        LedgerError::EmployeeHasReward { user_id } => format!(
            "У сотрудника {} есть не выданная награда",
            user_name(ctx, *user_id).await?
        ),
        LedgerError::CouchHasTrainings(user_id) => format!(
            "Тренер {} имеет незавершенные тренировки",
            user_name(ctx, *user_id).await?
        ),
        LedgerError::RateNotFound { user_id, rate } => {
            let user = user_name(ctx, *user_id).await?;
            format!(
                "{} тариф не найден у пользователя {}",
                rate_name(rate),
                user
            )
        }
        LedgerError::RateTypeAlreadyExists { user_id, rate } => {
            let user = user_name(ctx, *user_id).await?;
            format!(
                "{} тариф уже существует у пользователя {}",
                rate_name(rate),
                user
            )
        }
        LedgerError::NoRatesFound { user_id } => {
            let user = user_name(ctx, *user_id).await?;
            format!("У пользователя {} нет тарифов", user)
        }
        LedgerError::WrongTrainingClients { .. } => return Ok(None),
        LedgerError::RequestNotFound { id } => format!("Заявка {} не найдена", id),
    })))
}

async fn user_name(ctx: &mut Context, user_id: ObjectId) -> Result<String> {
    let user = ctx.ledger.users.get(&mut ctx.session, user_id).await?;
    Ok(user
        .map(|u| u.name.first_name)
        .unwrap_or_else(|| user_id.to_string()))
}

fn rate_name(rate: &Rate) -> &'static str {
    match rate {
        Rate::Fix { .. } => "Фиксированный",
        Rate::GroupTraining { .. } => "Групповой",
        Rate::PersonalTraining { .. } => "Персональный",
    }
}
