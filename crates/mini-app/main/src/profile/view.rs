use axum::{response::IntoResponse, Extension};
use bot_core::context::Context;
use model::{
    couch::CouchInfo,
    rights::Rule,
    statistics::marketing::ComeFrom,
    subscription::UserSubscription,
    user::{Freeze, UserName},
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub(crate) async fn view(Extension(ctx): Extension<Arc<Context>>) -> impl IntoResponse {
    let me = &ctx.me;

    let come_from = if ctx.has_right(Rule::ViewMarketingInfo) {
        Some(me.come_from)
    } else {
        None
    };

    let payer = me.payer().unwrap();
    let profile = Profile {
        id: me.id.clone(),
        tg_id: me.tg_id,
        name: me.name.clone(),
        phone: me.phone.clone(),
        freeze: me.freeze.clone(),
        subscriptions: payer.subscriptions().to_vec(),
        freeze_days: me.freeze_days,
        couch: me.couch.clone(),
        come_from,
    };

    axum::Json(profile)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub id: ObjectId,
    pub tg_id: i64,
    pub name: UserName,
    pub phone: String,
    pub freeze: Option<Freeze>,
    pub subscriptions: Vec<UserSubscription>,
    pub freeze_days: u32,
    pub couch: Option<CouchInfo>,
    pub come_from: Option<ComeFrom>,
}
