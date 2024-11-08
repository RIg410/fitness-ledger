use axum::{middleware, response::IntoResponse, routing::post, Extension, Router};
use bot_main::BotApp;
use contex::middleware as build_ctx;
use eyre::Result;
use jwt::JwtToken;
use ledger::Ledger;
use std::sync::Arc;

pub mod auth;
pub mod contex;
pub mod jwt;
pub mod profile;
pub mod schedule;

pub fn spawn(ledger: Arc<Ledger>, bot: BotApp) -> Result<()> {
    let ctx_builder = contex::ContextBuilder::new(ledger, bot);
    tokio::spawn(async move {
        let app = Router::new()
            .merge(profile::routes())
            .route("/auth", post(auth))
            .layer(middleware::from_fn_with_state(
                ctx_builder.clone(),
                build_ctx,
            ));
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        log::debug!("listening on {}", listener.local_addr().unwrap());
        axum::serve(listener, app).await.unwrap();
    });
    Ok(())
}

pub async fn auth(Extension(jwt): Extension<JwtToken>) -> impl IntoResponse {
    axum::Json(jwt)
}
