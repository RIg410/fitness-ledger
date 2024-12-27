use bot_core::context::Context;
use eyre::Result;
use model::errors::LedgerError;
use mongodb::bson::oid::ObjectId;
use teloxide::utils::markdown::escape;

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
    })))
}

async fn user_name(ctx: &mut Context, user_id: ObjectId) -> Result<String> {
    let user = ctx.ledger.users.get(&mut ctx.session, user_id).await?;
    Ok(user
        .map(|u| u.name.first_name)
        .unwrap_or_else(|| user_id.to_string()))
}
