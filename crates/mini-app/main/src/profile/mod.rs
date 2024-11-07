use axum::{routing::get, Router};

mod view;

pub fn routes() -> Router {
    Router::new().route("/profile", get(view::view))
}
