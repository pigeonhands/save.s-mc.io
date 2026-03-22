use common::{PublicKeyRequest, PublicKeyResponse, SaveTextRequest, SaveTextResponse};
use pgp::composed::{Esk, Message};
use pgp::types::KeyDetails;
use worker::send::{SendFuture, SendWrapper};

use crate::{
    error::{HttpError, HttpResult},
    http::{
        context::{AppState, DbCtx},
        extractors::CaptchaResponse,
    },
};
use axum::{
    Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::{get, post},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/public-key", get(get_public_key))
        .route("/item", post(save_item))
}

#[axum::debug_handler(state=AppState)]
pub async fn get_public_key(
    State(db): State<DbCtx>,
    _captcha: CaptchaResponse,
    params: Query<PublicKeyRequest>,
) -> HttpResult<impl IntoResponse> {
    let query = SendWrapper::new(
        db.prepare(
            r#"
            SELECT (keys.public_key)
                FROM users
                JOIN keys using (user_id)
                WHERE users.email = ?
                LIMIT 1
            "#,
        )
        .bind(&[params.email.clone().into()])?,
    );

    let user_pub_key: Option<String> = SendFuture::new(query.first(Some("public_key"))).await?;

    if let Some(pub_key) = user_pub_key {
        Ok(Json(PublicKeyResponse {
            email: params.email.clone(),
            pub_key,
        }))
    } else {
        Err(HttpError::NotFound)
    }
}

/// Extract recipient key IDs from the PKESK packets in an armored PGP message.
/// Returns lowercase hex key ID strings (e.g. "4c073ae0c8445c0c").
fn recipient_key_ids(encrypted_armor: &str) -> anyhow::Result<Vec<String>> {
    let (msg, _) = Message::from_string(encrypted_armor)?;

    let esks = match msg {
        Message::Encrypted { esk, .. } => esk,
        _ => anyhow::bail!("message is not encrypted"),
    };

    let ids = esks
        .into_iter()
        .filter_map(|packet| match packet {
            Esk::PublicKeyEncryptedSessionKey(pkesk) => pkesk.id().ok().map(|id| format!("{id}")),
            _ => None,
        })
        .collect();

    Ok(ids)
}

#[derive(serde::Deserialize)]
struct UserRow {
    user_id: String,
}

#[axum::debug_handler(state=AppState)]
pub async fn save_item(
    State(db): State<DbCtx>,
    Json(payload): Json<SaveTextRequest>,
) -> HttpResult<impl IntoResponse> {
    let key_ids = recipient_key_ids(&payload.message)
        .map_err(|e| anyhow::anyhow!("could not parse message: {e}"))?;

    if key_ids.is_empty() {
        return Err(anyhow::anyhow!("message has no public-key recipients").into());
    }

    let mut user_id: Option<String> = None;
    for key_id in &key_ids {
        let stmt = SendWrapper::new(
            db.prepare("SELECT user_id FROM keys WHERE encryption_key_id = ? LIMIT 1")
                .bind(&[key_id.clone().into()])?,
        );
        let row: Option<UserRow> = SendFuture::new(stmt.first(None))
            .await
            .map_err(|e| anyhow::anyhow!("DB lookup error: {e:?}"))?;
        if let Some(r) = row {
            user_id = Some(r.user_id);
            break;
        }
    }

    let user_id =
        user_id.ok_or_else(|| anyhow::anyhow!("message not encrypted to any registered key"))?;

    let insert_saved = SendWrapper::new(
        db.prepare(
            "INSERT INTO saved (user_id, data_type, description) VALUES (?, 'text', ?) RETURNING saved_id",
        )
        .bind(&[user_id.into(), payload.description.into()])?,
    );
    let saved_id: i32 = SendFuture::new(insert_saved.first(Some("saved_id")))
        .await?
        .ok_or_else(|| anyhow::anyhow!("no saved_id returned"))?;

    let insert_text = SendWrapper::new(
        db.prepare("INSERT INTO saved_text (saved_id, message) VALUES (?, ?)")
            .bind(&[saved_id.into(), payload.message.into()])?,
    );
    SendFuture::new(insert_text.run())
        .await
        .map_err(|e| anyhow::anyhow!("failed to insert saved_text: {e:?}"))?;

    Ok(Json(SaveTextResponse {}))
}
