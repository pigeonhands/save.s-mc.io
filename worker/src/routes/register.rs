use common::{PublicKeyRequest, PublicKeyResponse, RegisterBeginRequest, RegisterBeginResponse};
use worker::{
    Url,
    send::{SendFuture, SendWrapper},
};

use crate::{
    error::{HttpError, HttpResult},
    http::{
        context::{AppState, DbCtx},
        extractors::CaptchaResponse,
    },
    passkey_utils::{self, PasskeyCtx},
};
use axum::{
    Json, Router,
    extract::{Query, Request, State},
    response::IntoResponse,
    routing::{get, post},
};

pub fn router() -> Router<AppState> {
    Router::new().route("/begin", post(register_begin))
}

#[axum::debug_handler(state=AppState)]
pub async fn register_begin(
    State(passkey): State<PasskeyCtx>,
    _captcha: CaptchaResponse,
    req: Json<RegisterBeginRequest>,
) -> HttpResult<impl IntoResponse> {
    let (options, challenge) = passkey.register_start(&req.email)?;
    Ok(Json(RegisterBeginResponse {
        passkey_challenge: options,
        pgp_channenge: String::new(),
    }))
}
