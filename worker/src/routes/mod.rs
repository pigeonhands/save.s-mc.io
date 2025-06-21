mod save;

use crate::http::context::AppState;
use axum::{
    Router,
    body::Body as AxumBody,
    extract::{FromRef, Path, RawQuery, State},
    response::{IntoResponse, Response},
    routing::{get, post},
};

pub fn router() -> Router<AppState> {
    Router::new().nest("/api", api())
}

pub fn api() -> Router<AppState> {
    Router::new().nest("/save", save::router())
}
