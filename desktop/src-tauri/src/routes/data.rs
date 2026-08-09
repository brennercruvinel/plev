//! The card database itself: how fresh it is, and kicking off a refresh.
//!
//! This is the app's only scheduled network use. Everything else reads what the update left
//! behind, so the app keeps working with the plug pulled.

use axum::{
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};

use crate::db::open_cards_db;
use crate::http::{conflict, ok};
use crate::paths;
use axum::extract::Path as AxumPath;
use axum::http::{header, StatusCode};
use axum::response::Redirect;

use crate::images;


async fn data_info() -> Json<Value> {
    let Some(cdb) = open_cards_db() else {
        return Json(json!({ "exists": false, "cards": 0, "pt_names": 0, "built_at": Value::Null }));
    };
    let cards: i64 = cdb.query_row("SELECT COUNT(*) FROM cards", [], |r| r.get(0)).unwrap_or(0);
    let pt: i64 = cdb
        .query_row("SELECT COUNT(DISTINCT printed_name) FROM names_localized", [], |r| r.get(0))
        .unwrap_or(0);
    let built_at = std::fs::metadata(paths::cards_db())
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| json!(d.as_secs_f64()))
        .unwrap_or(Value::Null);
    Json(json!({ "exists": true, "cards": cards, "pt_names": pt, "built_at": built_at }))
}

/// Progress of the background data update, polled by the sidebar panel.
async fn data_update_status() -> Json<Value> {
    let s = crate::update::STATUS.lock().unwrap();
    Json(json!({
        "state": s.state,
        "task": s.task.clone().map_or(Value::Null, Value::from),
        "percent": s.percent,
        "error": s.error.clone().map_or(Value::Null, Value::from),
        "result": s.result.clone().unwrap_or(Value::Null),
    }))
}

async fn data_update_start() -> impl IntoResponse {
    if crate::update::start() {
        ok().into_response()
    } else {
        conflict("Uma atualização já está em andamento.").into_response()
    }
}

// Serves cached card art. Falls back to a redirect rather than a 404 so the app still shows
// images while the cache is still filling, and so a card added by a newer Scryfall drop than the
// last image sync isn't a permanent hole.
async fn card_image(AxumPath(rel): AxumPath<String>) -> axum::response::Response {
    use axum::response::IntoResponse;
    if let Some(clean) = images::relative_path(&format!("{}{}", images::HOST, rel)) {
        let file = images::cached_file(&clean);
        if let Ok(bytes) = std::fs::read(&file) {
            // Immutable: a printing's art never changes, so the webview should never re-ask.
            return (
                [
                    (header::CONTENT_TYPE, "image/jpeg"),
                    (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
                ],
                bytes,
            )
                .into_response();
        }
        return Redirect::temporary(&format!("{}{}", images::HOST, clean)).into_response();
    }
    StatusCode::NOT_FOUND.into_response()
}

/// Stops an update in flight. The already-downloaded art stays: the cache is resumable, so
/// "cancel" means "pause", and re-running picks up exactly where this left off.
async fn data_update_cancel() -> impl IntoResponse {
    crate::update::cancel();
    ok()
}

/// How much art is cached, so the sidebar can say whether the app is genuinely offline-ready.
async fn images_info() -> Json<Value> {
    let bytes = images::cache_size_bytes();
    let cached: i64 = crate::db::open_cards_db()
        .and_then(|c| {
            c.query_row("SELECT COUNT(*) FROM cards WHERE image_uri IS NOT NULL", [], |r| r.get(0))
                .ok()
        })
        .unwrap_or(0);
    Json(json!({
        "bytes": bytes,
        "gb": (bytes as f64 / 1_073_741_824.0 * 100.0).round() / 100.0,
        "cards_with_art": cached,
    }))
}

pub fn router() -> Router {
    Router::new()
        .route("/api/data/info", get(data_info))
        .route("/api/data/update", post(data_update_start))
        .route("/api/data/update/status", get(data_update_status))
        .route("/api/data/images", get(images_info))
        .route("/api/data/update/cancel", post(data_update_cancel))
        .route("/img/*rel", get(card_image))
}
