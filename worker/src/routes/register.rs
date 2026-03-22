use base64::Engine;
use common::{RegisterBeginRequest, RegisterBeginResponse, RegisterFinishRequest, RegisterFinishResponse};
use pgp::composed::{Deserializable, SignedPublicKey};
use pgp::types::{KeyDetails, PublicKeyTrait};
use serde::{Deserialize, Serialize};
use worker::send::{SendFuture, SendWrapper};

use crate::{
    error::{HttpError, HttpResult},
    http::{
        context::{AppState, DbCtx, KvCtx},
        extractors::CaptchaResponse,
    },
    passkey_utils::PasskeyCtx,
};
use axum::{
    Json, Router,
    extract::{State},
    response::IntoResponse,
    routing::post,
};

const CHALLENGE_TTL_SECS: u64 = 300;

/// Find the encryption subkey whose fingerprint matches `fingerprint_hex` and
/// return its 8-byte key ID as a lowercase hex string.
fn encryption_key_id(pub_key_armor: &str, fingerprint_hex: &str) -> anyhow::Result<String> {
    let (key, _) = SignedPublicKey::from_string(pub_key_armor)?;
    for subkey in &key.public_subkeys {
        if subkey.is_encryption_key()
            && format!("{}", subkey.fingerprint()) == fingerprint_hex
        {
            return Ok(format!("{}", subkey.key_id()));
        }
    }
    // Fall back to the first encryption subkey if fingerprint doesn't match
    // (handles edge cases where the fingerprint format differs slightly).
    for subkey in &key.public_subkeys {
        if subkey.is_encryption_key() {
            return Ok(format!("{}", subkey.key_id()));
        }
    }
    anyhow::bail!("no encryption subkey found in public key")
}

#[derive(Serialize, Deserialize)]
struct RegisterSession {
    challenge_b64: String,
    pub_key: String,
    encryption_key: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/begin", post(register_begin))
        .route("/finish", post(register_finish))
}

#[axum::debug_handler(state=AppState)]
pub async fn register_begin(
    State(passkey): State<PasskeyCtx>,
    State(kv): State<KvCtx>,
    _captcha: CaptchaResponse,
    req: Json<RegisterBeginRequest>,
) -> HttpResult<impl IntoResponse> {
    let (options, challenge) = passkey.register_start(&req.email)?;

    let challenge_b64 =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(challenge);

    let session = RegisterSession {
        challenge_b64,
        pub_key: req.pub_key.clone(),
        encryption_key: req.encryption_key.clone(),
    };

    SendFuture::new(
        kv.put(
            &format!("register:{}", req.email),
            serde_json::to_string(&session).map_err(anyhow::Error::from)?,
        )
        .map_err(worker::Error::from)?
        .expiration_ttl(CHALLENGE_TTL_SECS)
        .execute(),
    )
    .await
    .map_err(worker::Error::from)?;

    Ok(Json(RegisterBeginResponse {
        passkey_challenge: options,
        pgp_channenge: String::new(),
    }))
}

#[axum::debug_handler(state=AppState)]
pub async fn register_finish(
    State(passkey): State<PasskeyCtx>,
    State(kv): State<KvCtx>,
    State(db): State<DbCtx>,
    _captcha: CaptchaResponse,
    req: Json<RegisterFinishRequest>,
) -> HttpResult<impl IntoResponse> {
    let kv_key = format!("register:{}", req.email);

    let session_json: String = SendFuture::new(kv.get(&kv_key).text())
        .await
        .map_err(worker::Error::from)?
        .ok_or(HttpError::Unauthorized)?;

    let session: RegisterSession =
        serde_json::from_str(&session_json).map_err(anyhow::Error::from)?;

    let cose_key_bytes =
        PasskeyCtx::register_finish(&req.credential, &session.challenge_b64, passkey.rp_id())
            .map_err(|e| anyhow::anyhow!("passkey verification failed: {e}"))?;
    let cose_key_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&cose_key_bytes);
    let credential_id = req.credential.id.clone();

    SendFuture::new(kv.delete(&kv_key))
        .await
        .map_err(worker::Error::from)?;

    let user_id = uuid::Uuid::new_v4().to_string();

    let insert_user = SendWrapper::new(
        db.prepare("INSERT INTO users (user_id, email) VALUES (?, ?)")
            .bind(&[user_id.clone().into(), req.email.clone().into()])?,
    );
    SendFuture::new(insert_user.run())
        .await
        .map_err(|e| anyhow::anyhow!("failed to insert user: {e:?}"))?;

    let enc_key_id = encryption_key_id(&session.pub_key, &session.encryption_key)
        .map_err(|e| anyhow::anyhow!("failed to extract encryption key id: {e}"))?;
    let insert_key = SendWrapper::new(
        db.prepare("INSERT INTO keys (user_id, public_key, encryption_key_id) VALUES (?, ?, ?)")
            .bind(&[user_id.clone().into(), session.pub_key.into(), enc_key_id.into(),
            ])?,
    );
    SendFuture::new(insert_key.run())
        .await
        .map_err(|e| anyhow::anyhow!("failed to insert key: {e:?}"))?;

    let insert_cred = SendWrapper::new(
        db.prepare(
            "INSERT INTO passkey_credentials (credential_id, user_id, cose_public_key) VALUES (?, ?, ?)",
        )
        .bind(&[credential_id.into(), user_id.into(), cose_key_b64.into()])?,
    );
    SendFuture::new(insert_cred.run())
        .await
        .map_err(|e| anyhow::anyhow!("failed to insert passkey credential: {e:?}"))?;

    Ok(Json(RegisterFinishResponse {}))
}
