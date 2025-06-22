mod register;
mod save;

use crate::http::context::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new().nest("/api", api())
}

pub fn api() -> Router<AppState> {
    Router::new()
        .nest("/save", save::router())
        .nest("/register", register::router())
}
