use axum::{extract::Path, http::StatusCode, Extension, Json};
use bot_core::context::Context;
use eyre::Context as _;
use model::{rights::Rule, user::sanitize_phone};
use mongodb::bson::oid::ObjectId;
use serde::Deserialize;
use std::sync::Arc;

use crate::{contex::WebContext as _, internal_error, view::user::UserView};

#[derive(Deserialize)]
pub struct Params {
    tg_id: Option<i64>,
    phone: Option<String>,
    id: Option<ObjectId>,
}

pub(crate) async fn get(
    Extension(mut ctx): Extension<Arc<Context>>,
    Path(Params { tg_id, phone, id }): Path<Params>,
) -> Result<Json<UserView>, (StatusCode, String)> {
    let ctx = Arc::get_mut(&mut ctx).expect("Context is shared");

    let user = if let Some(id) = id {
        ctx.check_rule(Rule::ViewUsers)?;
        ctx.ledger
            .users
            .get(&mut ctx.session, id)
            .await
            .context("Failed to get user")
            .map_err(internal_error)?
    } else if let Some(tg_id) = tg_id {
        ctx.check_rule(Rule::ViewUsers)?;
        ctx.ledger
            .users
            .get_by_tg_id(&mut ctx.session, tg_id)
            .await
            .context("Failed to get user")
            .map_err(internal_error)?
    } else if let Some(phone) = phone {
        ctx.check_rule(Rule::ViewUsers)?;
        ctx.ledger
            .users
            .get_by_phone(&mut ctx.session, &sanitize_phone(&phone))
            .await
            .context("Failed to get user")
            .map_err(internal_error)?
    } else {
        Some(ctx.me.clone())
    };

    let mut user = match user {
        Some(user) => user,
        None => {
            return Err((
                axum::http::StatusCode::NOT_FOUND,
                "User not found".to_string(),
            ))
        }
    };
    ctx.ledger
        .users
        .resolve_family(&mut ctx.session, &mut user)
        .await
        .context("Failed to resolve family")
        .map_err(internal_error)?;

    let extension = ctx
        .ledger
        .users
        .get_extension(&mut ctx.session, user.id)
        .await
        .context("Failed to get user extension")
        .map_err(internal_error)?;

    let mut user_view = UserView::try_from(user).map_err(internal_error)?;
    user_view.birthday = extension.birthday;

    Ok(Json(user_view))
}
