//! Shared HTTP response shapes.
//!
//! Every handler that can fail answers in the same envelope — `{"detail": "..."}` — because the
//! frontend's `req()` reads exactly that field to build the message it shows the user. Keeping
//! the constructors here (rather than one per route module) is what stops that contract from
//! drifting the next time a route is added.

use axum::{http::StatusCode, response::Json};
use serde_json::{json, Value};

/// The resource genuinely isn't there — an unknown card name, a deck id that no longer exists.
pub fn not_found(msg: &str) -> (StatusCode, Json<Value>) {
    (StatusCode::NOT_FOUND, Json(json!({ "detail": msg })))
}

/// The request itself is wrong: a blank name, a quantity that would break a deck rule.
pub fn bad_request(msg: &str) -> (StatusCode, Json<Value>) {
    (StatusCode::BAD_REQUEST, Json(json!({ "detail": msg })))
}

/// Something on our side failed — almost always the database being unavailable.
pub fn server_error(msg: &str) -> (StatusCode, Json<Value>) {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "detail": msg })))
}

/// The request is valid but conflicts with work already in flight — a second data update while
/// the first is still downloading.
pub fn conflict(msg: &str) -> (StatusCode, Json<Value>) {
    (StatusCode::CONFLICT, Json(json!({ "detail": msg })))
}

/// An upstream we don't control failed: Scryfall or EDHREC unreachable or answering garbage.
/// Distinct from `server_error` so the UI can say "try again later" rather than "something broke".
pub fn bad_gateway(msg: &str) -> (StatusCode, Json<Value>) {
    (StatusCode::BAD_GATEWAY, Json(json!({ "detail": msg })))
}

/// Shorthand for the overwhelmingly common failure: app.db wouldn't open.
pub fn db_unavailable() -> (StatusCode, Json<Value>) {
    server_error("Banco indisponível")
}

/// The uniform success answer for writes that have nothing to return.
pub fn ok() -> Json<Value> {
    Json(json!({ "ok": true }))
}
