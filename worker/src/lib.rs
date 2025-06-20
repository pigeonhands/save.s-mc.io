mod error;
mod extractors;
mod routes;
mod turnstile;

use axum::Router;
use tower_service::Service;
use worker::*;

fn router() -> Router {
    Router::new().merge(routes::router())
}

#[event(start)]
fn start() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
}

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    _env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    Ok(router().call(req).await?)
}
