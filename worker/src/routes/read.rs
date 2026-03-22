use common::{ReadItemsResponse, SavedItem};
use worker::send::{SendFuture, SendWrapper};

use crate::{
    error::{HttpError, HttpResult},
    http::context::{AppState, DbCtx, KvCtx},
};
use axum::{Router, extract::State, http::Request, response::IntoResponse, routing::get};
use axum::Json;
use axum::body::Body;

pub fn router() -> Router<AppState> {
    Router::new().route("/items", get(read_items))
}

#[axum::debug_handler(state=AppState)]
pub async fn read_items(
    State(kv): State<KvCtx>,
    State(db): State<DbCtx>,
    req: Request<Body>,
) -> HttpResult<impl IntoResponse> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(HttpError::Unauthorized)?
        .to_string();

    let user_id: String = SendFuture::new(kv.get(&format!("session:{token}")).text())
        .await
        .map_err(worker::Error::from)?
        .ok_or(HttpError::Unauthorized)?;

    let stmt = SendWrapper::new(
        db.prepare(
            "SELECT s.saved_id, s.description, s.data_type, s.created_at, st.message
             FROM saved s
             LEFT JOIN saved_text st ON s.saved_id = st.saved_id
             WHERE s.user_id = ?
             ORDER BY s.created_at DESC",
        )
        .bind(&[user_id.into()])?,
    );

    let result = SendFuture::new(stmt.all())
        .await
        .map_err(|e| anyhow::anyhow!("DB error: {e:?}"))?;

    let items: Vec<SavedItem> = result
        .results()
        .map_err(|e| anyhow::anyhow!("Deserialize error: {e:?}"))?;

    Ok(Json(ReadItemsResponse { items }))
}
