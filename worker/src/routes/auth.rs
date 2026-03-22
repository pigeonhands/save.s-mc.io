use base64::Engine;
use common::{AuthBeginResponse, AuthFinishRequest, AuthFinishResponse};
use worker::send::{SendFuture, SendWrapper};

use crate::{
    error::{HttpError, HttpResult},
    http::context::{AppState, DbCtx, KvCtx},
    passkey_utils::PasskeyCtx,
};
use axum::{Json, Router, extract::State, response::IntoResponse, routing::post};

const AUTH_CHALLENGE_TTL: u64 = 300;
const SESSION_TTL: u64 = 3600;

#[derive(serde::Deserialize)]
struct CredentialRow {
    cose_public_key: String,
    user_id: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/begin", post(auth_begin))
        .route("/finish", post(auth_finish))
}

#[axum::debug_handler(state=AppState)]
pub async fn auth_begin(
    State(passkey): State<PasskeyCtx>,
    State(kv): State<KvCtx>,
) -> HttpResult<impl IntoResponse> {
    let (options, challenge) = passkey.auth_start(vec![])?;
    let challenge_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(challenge);
    let challenge_id = uuid::Uuid::new_v4().to_string();

    SendFuture::new(
        kv.put(&format!("auth:{challenge_id}"), challenge_b64)
            .map_err(worker::Error::from)?
            .expiration_ttl(AUTH_CHALLENGE_TTL)
            .execute(),
    )
    .await
    .map_err(worker::Error::from)?;

    Ok(Json(AuthBeginResponse { options, challenge_id }))
}

#[axum::debug_handler(state=AppState)]
pub async fn auth_finish(
    State(passkey): State<PasskeyCtx>,
    State(kv): State<KvCtx>,
    State(db): State<DbCtx>,
    req: Json<AuthFinishRequest>,
) -> HttpResult<impl IntoResponse> {
    let kv_key = format!("auth:{}", req.challenge_id);

    let challenge_b64: String = SendFuture::new(kv.get(&kv_key).text())
        .await
        .map_err(worker::Error::from)?
        .ok_or(HttpError::Unauthorized)?;

    let cred: CredentialRow = SendFuture::new(SendWrapper::new(
        db.prepare("SELECT cose_public_key, user_id FROM passkey_credentials WHERE credential_id = ?")
            .bind(&[req.assertion.credential_id.clone().into()])?,
    ).first(None))
    .await?
    .ok_or(HttpError::Unauthorized)?;

    PasskeyCtx::auth_finish(&req.assertion, &challenge_b64, passkey.rp_id(), &cred.cose_public_key)
        .map_err(|e| anyhow::anyhow!("auth verification failed: {e}"))?;

    SendFuture::new(kv.delete(&kv_key))
        .await
        .map_err(worker::Error::from)?;

    let session_token = uuid::Uuid::new_v4().to_string();
    SendFuture::new(
        kv.put(&format!("session:{session_token}"), cred.user_id)
            .map_err(worker::Error::from)?
            .expiration_ttl(SESSION_TTL)
            .execute(),
    )
    .await
    .map_err(worker::Error::from)?;

    Ok(Json(AuthFinishResponse { session_token }))
}
