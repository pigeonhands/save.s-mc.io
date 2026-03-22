use base64::Engine;
use common::{AuthBeginRequest, AuthBeginResponse, AuthFinishRequest, AuthFinishResponse};
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
    credential_id: String,
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
    State(db): State<DbCtx>,
    req: Json<AuthBeginRequest>,
) -> HttpResult<impl IntoResponse> {
    // Look up credential_id(s) for this user
    let stmt = SendWrapper::new(
        db.prepare(
            "SELECT pc.credential_id FROM passkey_credentials pc
             INNER JOIN users u ON pc.user_id = u.user_id
             WHERE u.email = ?",
        )
        .bind(&[req.email.clone().into()])?,
    );

    #[derive(serde::Deserialize)]
    struct CredIdRow {
        credential_id: String,
    }

    let result = SendFuture::new(stmt.all())
        .await
        .map_err(|e| anyhow::anyhow!("DB error looking up credentials: {e:?}"))?;

    let rows: Vec<CredIdRow> = result
        .results()
        .map_err(|e| anyhow::anyhow!("Deserialize error: {e:?}"))?;

    if rows.is_empty() {
        return Err(HttpError::Unauthorized);
    }

    let credential_ids: Vec<String> = rows.into_iter().map(|r| r.credential_id).collect();

    let (options, challenge) = passkey.auth_start(credential_ids)?;
    let challenge_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(challenge);

    SendFuture::new(
        kv.put(&format!("auth:{}", req.email), challenge_b64)
            .map_err(worker::Error::from)?
            .expiration_ttl(AUTH_CHALLENGE_TTL)
            .execute(),
    )
    .await
    .map_err(worker::Error::from)?;

    Ok(Json(AuthBeginResponse { options }))
}

#[axum::debug_handler(state=AppState)]
pub async fn auth_finish(
    State(passkey): State<PasskeyCtx>,
    State(kv): State<KvCtx>,
    State(db): State<DbCtx>,
    req: Json<AuthFinishRequest>,
) -> HttpResult<impl IntoResponse> {
    let kv_key = format!("auth:{}", req.email);

    let challenge_b64: String = SendFuture::new(kv.get(&kv_key).text())
        .await
        .map_err(worker::Error::from)?
        .ok_or(HttpError::Unauthorized)?;

    // Look up the credential by credential_id
    let stmt = SendWrapper::new(
        db.prepare(
            "SELECT credential_id, cose_public_key, user_id
             FROM passkey_credentials
             WHERE credential_id = ?",
        )
        .bind(&[req.assertion.credential_id.clone().into()])?,
    );

    let result = SendFuture::new(stmt.all())
        .await
        .map_err(|e| anyhow::anyhow!("DB error: {e:?}"))?;

    let mut rows: Vec<CredentialRow> = result
        .results()
        .map_err(|e| anyhow::anyhow!("Deserialize error: {e:?}"))?;

    let cred = rows.pop().ok_or(HttpError::Unauthorized)?;

    PasskeyCtx::auth_finish(
        &req.assertion,
        &challenge_b64,
        passkey.rp_id(),
        &cred.cose_public_key,
    )
    .map_err(|e| anyhow::anyhow!("auth verification failed: {e}"))?;

    // Delete used challenge
    SendFuture::new(kv.delete(&kv_key))
        .await
        .map_err(worker::Error::from)?;

    // Create session token
    let session_token = uuid::Uuid::new_v4().to_string();
    SendFuture::new(
        kv.put(&format!("session:{}", session_token), cred.user_id)
            .map_err(worker::Error::from)?
            .expiration_ttl(SESSION_TTL)
            .execute(),
    )
    .await
    .map_err(worker::Error::from)?;

    Ok(Json(AuthFinishResponse { session_token }))
}
