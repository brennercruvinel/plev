//! Games played and the activity log.
//!
//! Both are append-mostly history: the app writes here as a side effect of everything else, and
//! the home screen reads it back. Nothing else depends on these tables, which is why they can
//! stay this simple.

use axum::{
    extract::Query,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use rusqlite::params;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::db::open_app_db;
use crate::http::{bad_request, db_unavailable};

async fn list_games() -> Json<Value> {
    let Ok(con) = open_app_db() else { return Json(json!([])) };
    let mut out = (|| -> rusqlite::Result<Vec<Value>> {
        let mut stmt = con.prepare(
            "SELECT games.id, games.deck_id, games.played_at, games.result, games.opponents,
                    games.turns, games.notes, decks.name
             FROM games JOIN decks ON decks.id = games.deck_id ORDER BY games.played_at DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(json!({
                "id": r.get::<_, i64>(0)?, "deck_id": r.get::<_, i64>(1)?,
                "played_at": r.get::<_, String>(2)?, "result": r.get::<_, String>(3)?,
                "opponents": r.get::<_, Option<String>>(4)?, "turns": r.get::<_, Option<i64>>(5)?,
                "notes": r.get::<_, Option<String>>(6)?, "deck_name": r.get::<_, String>(7)?,
                "highlights": json!([]),
            }))
        })?;
        rows.collect()
    })()
    .unwrap_or_default();

    // The standout cards recorded per game — the Partidas page lists them under each row.
    for game in out.iter_mut() {
        let Some(id) = game.get("id").and_then(|v| v.as_i64()) else { continue };
        let names: Vec<Value> = (|| -> rusqlite::Result<Vec<Value>> {
            let mut stmt =
                con.prepare("SELECT card_name FROM game_highlights WHERE game_id = ?1")?;
            let rows = stmt.query_map([id], |r| Ok(Value::from(r.get::<_, String>(0)?)))?;
            rows.collect()
        })()
        .unwrap_or_default();
        if let Some(m) = game.as_object_mut() {
            m.insert("highlights".into(), Value::Array(names));
        }
    }
    Json(Value::Array(out))
}

async fn games_stats() -> Json<Value> {
    let Ok(con) = open_app_db() else { return Json(json!({})) };
    let count = |result: &str| -> i64 {
        con.query_row("SELECT COUNT(*) FROM games WHERE result = ?1", [result], |r| r.get(0))
            .unwrap_or(0)
    };
    let (wins, losses, draws) = (count("vitoria"), count("derrota"), count("empate"));
    let total = wins + losses + draws;
    let win_rate = if total > 0 {
        json!(((wins as f64 / total as f64) * 1000.0).round() / 10.0)
    } else {
        Value::Null
    };
    let top: Vec<Value> = (|| -> rusqlite::Result<Vec<Value>> {
        let mut stmt = con.prepare(
            "SELECT card_name, COUNT(*) n FROM game_highlights
             GROUP BY card_name ORDER BY n DESC LIMIT 10",
        )?;
        let rows = stmt
            .query_map([], |r| Ok(json!({ "card_name": r.get::<_, String>(0)?, "n": r.get::<_, i64>(1)? })))?;
        rows.collect()
    })()
    .unwrap_or_default();
    Json(json!({
        "total_games": total, "wins": wins, "losses": losses, "draws": draws,
        "win_rate": win_rate, "top_highlight_cards": top,
    }))
}

#[derive(Deserialize)]
pub struct ActivityQuery {
    #[serde(default = "activity_limit")]
    limit: i64,
}

fn activity_limit() -> i64 { 30 }

async fn list_activity(Query(p): Query<ActivityQuery>) -> Json<Value> {
    let Ok(con) = open_app_db() else { return Json(json!([])) };
    let out = (|| -> rusqlite::Result<Vec<Value>> {
        let mut stmt = con.prepare(
            "SELECT id, ts, type, description FROM activity ORDER BY ts DESC, id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map([p.limit], |r| {
            Ok(json!({
                "id": r.get::<_, i64>(0)?, "ts": r.get::<_, String>(1)?,
                "type": r.get::<_, String>(2)?, "description": r.get::<_, String>(3)?,
            }))
        })?;
        rows.collect()
    })()
    .unwrap_or_default();
    Json(Value::Array(out))
}

#[derive(Deserialize)]
pub struct GameIn {
    deck_id: i64,
    played_at: String,
    result: String,
    #[serde(default)]
    opponents: Option<String>,
    #[serde(default)]
    turns: Option<i64>,
    #[serde(default)]
    notes: Option<String>,
    #[serde(default)]
    highlights: Vec<String>,
}

async fn add_game(Json(p): Json<GameIn>) -> impl IntoResponse {
    let Ok(con) = open_app_db() else {
        return db_unavailable().into_response();
    };
    if con
        .execute(
            "INSERT INTO games (deck_id, played_at, result, opponents, turns, notes)
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![p.deck_id, p.played_at, p.result, p.opponents, p.turns, p.notes],
        )
        .is_err()
    {
        // The CHECK constraint on result rejects anything outside vitoria/derrota/empate.
        return bad_request("Não foi possível registrar a partida").into_response();
    }
    let game_id = con.last_insert_rowid();
    for card_name in &p.highlights {
        let _ = con.execute(
            "INSERT INTO game_highlights (game_id, card_name) VALUES (?1, ?2)",
            params![game_id, card_name],
        );
    }
    Json(json!({ "ok": true, "id": game_id })).into_response()
}

pub fn router() -> Router {
    Router::new()
        .route("/api/games", get(list_games).post(add_game))
        .route("/api/games/stats", get(games_stats))
        .route("/api/activity", get(list_activity))
}
