use common::{PublicKeyRequest, PublicKeyResponse};
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
    routing::get,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/public-key", get(get_public_key))
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
