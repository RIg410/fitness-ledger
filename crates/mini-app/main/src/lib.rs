use std::sync::Arc;

use askama::Template;
use axum::{
    http::StatusCode,
    middleware,
    response::{Html, IntoResponse, Response},
    routing::get,
    Extension, Router,
};
use bot_core::context::Context;
use bot_main::BotApp;
use contex::middleware as build_ctx;
use eyre::Result;
use ledger::Ledger;
pub mod contex;

pub fn spawn(ledger: Ledger, bot: BotApp) -> Result<()> {
    let ctx_builder = contex::ContextBuilder::new(ledger, bot);
    tokio::spawn(async move {
        let app = Router::new()
            .route("/main", get(main))
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

async fn main(Extension(ctx): Extension<Arc<Context>>) -> impl IntoResponse {
    HtmlTemplate(MainTemplate {
        name: ctx.me.name.first_name.clone(),
    })
}

#[derive(Template)]
#[template(path = "main.html", ext = "html")]
struct MainTemplate {
    name: String,
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}

#[derive(Template)]
#[template(path = "main.html", ext = "html")]
struct MainTemplate1 {
    name: MainTemplate,
}

#[test]
fn test_render() {
    let htpl = MainTemplate1 {
        name: MainTemplate {
            name: "dddd".to_string(),
        },
    };
    let t = htpl.render().unwrap();
    assert_eq!(t.as_str(), "");
}
