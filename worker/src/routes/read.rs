use common::{ReadItemsResponse, SavedItem};
use worker::send::{SendFuture, SendWrapper};

use crate::{
    error::HttpResult,
    http::{
        context::{AppState, DbCtx},
        extractors::SessionUser,
    },
};
use axum::{Json, Router, extract::{Path, State}, response::IntoResponse, routing::{delete, get}};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/items", get(read_items))
        .route("/item/{id}", delete(delete_item))
}

#[axum::debug_handler(state=AppState)]
pub async fn read_items(
    session: SessionUser,
    State(db): State<DbCtx>,
) -> HttpResult<impl IntoResponse> {
    let stmt = SendWrapper::new(
        db.prepare(
            "SELECT s.saved_id, s.description, s.data_type, s.created_at, st.message
             FROM saved s
             LEFT JOIN saved_text st ON s.saved_id = st.saved_id
             WHERE s.user_id = ?
             ORDER BY s.created_at DESC",
        )
        .bind(&[session.user_id.into()])?,
    );

    let result = SendFuture::new(stmt.all())
        .await
        .map_err(|e| anyhow::anyhow!("DB error: {e:?}"))?;

    let items: Vec<SavedItem> = result
        .results()
        .map_err(|e| anyhow::anyhow!("Deserialize error: {e:?}"))?;

    Ok(Json(ReadItemsResponse { items }))
}

#[axum::debug_handler(state=AppState)]
pub async fn delete_item(
    session: SessionUser,
    State(db): State<DbCtx>,
    Path(saved_id): Path<String>,
) -> HttpResult<impl IntoResponse> {
    SendFuture::new(SendWrapper::new(
        db.prepare(
            "DELETE FROM saved_text WHERE saved_id = (SELECT saved_id FROM saved WHERE saved_id = ? AND user_id = ?)",
        )
        .bind(&[saved_id.clone().into(), session.user_id.clone().into()])?,
    ).run())
    .await
    .map_err(|e| anyhow::anyhow!("DB error: {e:?}"))?;

    SendFuture::new(SendWrapper::new(
        db.prepare("DELETE FROM saved WHERE saved_id = ? AND user_id = ?")
            .bind(&[saved_id.into(), session.user_id.into()])?,
    ).run())
    .await
    .map_err(|e| anyhow::anyhow!("DB error: {e:?}"))?;

    Ok(Json(()))
}
