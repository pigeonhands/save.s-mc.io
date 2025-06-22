mod error;
mod http;
mod logger;
mod passkey_utils;
mod routes;
mod turnstile;

use axum::Router;
use http::context::AppState;
use tower_service::Service;
use worker::*;

fn router() -> Router<AppState> {
    Router::new().merge(routes::router())
}

#[event(start)]
fn start() {
    console_error_panic_hook::set_once();
    logger::init_with_level(&log::Level::Debug);
}

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    Ok(router()
        .with_state(AppState::from_env(env, &req))
        .call(req)
        .await?)
}
