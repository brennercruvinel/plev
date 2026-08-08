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

async fn data_info() -> Json<Value> {
    let Some(cdb) = open_cards_db() else {
        return Json(json!({ "exists": false, "cards": 0, "pt_names": 0, "built_at": Value::Null }));
    };
    let cards: i64 = cdb.query_row("SELECT COUNT(*) FROM cards", [], |r| r.get(0)).unwrap_or(0);
    let pt: i64 = cdb
        .query_row("SELECT COUNT(DISTINCT printed_name) FROM names_pt", [], |r| r.get(0))
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

pub fn router() -> Router {
    Router::new()
        .route("/api/data/info", get(data_info))
        .route("/api/data/update", post(data_update_start))
        .route("/api/data/update/status", get(data_update_status))
}
