use axum::{routing::get, Router};

mod load_user;

pub fn routes() -> Router {
    Router::new().route("/user", get(load_user::get))
}
