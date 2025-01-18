use crate::{context::Context, widget::Jmp};
use eyre::{Error, Result};
use model::{errors::LedgerError, user::rate::Rate};
use mongodb::bson::oid::ObjectId;
use teloxide::utils::markdown::escape;

pub async fn handle_result(ctx: &mut Context, result: Result<Jmp, Error>) -> Result<Jmp, Error> {
    match result {
        Ok(jmp) => Ok(jmp),
        Err(err) => {
            let ledger_err = err.downcast::<LedgerError>()?;
            if let Some(notification) = bassness_error(ctx, &ledger_err).await? {
                ctx.send_notification(&notification).await?;
                Ok(Jmp::Stay)
            } else {
                Err(Error::new(ledger_err))
            }
        }
    }
}

pub async fn bassness_error(ctx: &mut Context, err: &LedgerError) -> Result<Option<String>> {
    Ok(Some(match err {
        LedgerError::Eyre(_) => return Ok(None),
        LedgerError::UserNotFound(object_id) => format!(
            "Ошибка: *Пользователь {} не найден*",
            escape(&object_id.to_string())
        ),
        LedgerError::MemberNotFound { user_id, member_id } => {
            let user = user_name(ctx, *user_id).await?;
            let member = user_name(ctx, *member_id).await?;
            format!(
                "Ошибка: *Пользователь {} не найден в семье пользователя {}*",
                member, user
            )
        }
        LedgerError::WrongFamilyMember { user_id, member_id } => {
            let user = user_name(ctx, *user_id).await?;
            let member = user_name(ctx, *member_id).await?;
            format!(
                "Ошибка:*Пользователь {} не является членом семьи пользователя {}*",
                member, user
            )
        }
        LedgerError::MongoError(_) => return Ok(None),
        LedgerError::UserAlreadyInFamily { user_id, member_id } => {
            let user = user_name(ctx, *user_id).await?;
            let member = user_name(ctx, *member_id).await?;
            format!(
                "Ошибка:*Пользователь {} уже является членом семьи пользователя {}*",
                member, user
            )
        }
        LedgerError::UserAlreadyEmployee { user_id } => format!(
            "Ошибка:*Пользователь {} уже является сотрудником*",
            user_name(ctx, *user_id).await?
        ),
        LedgerError::UserNotEmployee { user_id } => format!(
            "Ошибка:*Пользователь {} не является сотрудником*",
            user_name(ctx, *user_id).await?
        ),
        LedgerError::EmployeeHasReward { user_id } => format!(
            "Ошибка:*У сотрудника {} есть не выданная награда*",
            user_name(ctx, *user_id).await?
        ),
        LedgerError::CouchHasTrainings(user_id) => format!(
            "Ошибка:*Тренер {} имеет незавершенные тренировки*",
            user_name(ctx, *user_id).await?
        ),
        LedgerError::RateNotFound { user_id, rate } => {
            let user = user_name(ctx, *user_id).await?;
            format!(
                "Ошибка:*{} тариф не найден у пользователя {}*",
                rate_name(rate),
                user
            )
        }
        LedgerError::RateTypeAlreadyExists { user_id, rate } => {
            let user = user_name(ctx, *user_id).await?;
            format!(
                "Ошибка:*{} тариф уже существует у пользователя {}*",
                rate_name(rate),
                user
            )
        }
        LedgerError::NoRatesFound { user_id } => {
            let user = user_name(ctx, *user_id).await?;
            format!("Ошибка:*У пользователя {} нет тарифов*", user)
        }
        LedgerError::WrongTrainingClients { .. } => return Ok(None),
        LedgerError::RequestNotFound { id } => format!("Ошибка:*Заявка {} не найдена*", id),
    }))
}

async fn user_name(ctx: &mut Context, user_id: ObjectId) -> Result<String> {
    let user = ctx.ledger.users.get(&mut ctx.session, user_id).await?;
    Ok(user
        .map(|u| escape(&u.name.first_name))
        .unwrap_or_else(|| escape(&user_id.to_string())))
}

fn rate_name(rate: &Rate) -> &'static str {
    match rate {
        Rate::Fix { .. } => "Фиксированный",
        Rate::GroupTraining { .. } => "Групповой",
        Rate::PersonalTraining { .. } => "Персональный",
    }
}
