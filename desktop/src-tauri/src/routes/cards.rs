//! Card index lookups — the read-only Scryfall database.
//!
//! Nothing here touches app.db: these endpoints only ever read `mtg.sqlite`, which the updater
//! rebuilds. Name matching is deliberately forgiving, in this order: exact, accent-folded,
//! official Portuguese printed name, then substring — so a name typed from memory still lands.

use axum::{
    extract::{Path, Query},
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::{json, Map, Value};

use crate::db::{fold_text, open_cards_db};
use crate::http::not_found;

pub const CARD_COLS: &str = "oracle_id, name, mana_cost, cmc, type_line, oracle_text, power, toughness, \
     loyalty, colors, color_identity, rarity, set_code, keywords, commander_legal, price_usd, \
     reserved, edhrec_rank, uri, image_uri, game_changer";

/// Turns a card row into the same JSON object shape the Python endpoints return.
pub fn card_row_to_json(r: &rusqlite::Row) -> rusqlite::Result<Value> {
    let mut m = Map::new();
    macro_rules! s {
        ($k:literal) => {
            m.insert($k.into(), r.get::<_, Option<String>>($k).unwrap_or(None).map_or(Value::Null, Value::from));
        };
    }
    macro_rules! i {
        ($k:literal) => {
            m.insert($k.into(), r.get::<_, Option<i64>>($k).unwrap_or(None).map_or(Value::Null, Value::from));
        };
    }
    s!("oracle_id"); s!("name"); s!("mana_cost"); s!("type_line"); s!("oracle_text");
    s!("power"); s!("toughness"); s!("loyalty"); s!("colors"); s!("color_identity");
    s!("rarity"); s!("set_code"); s!("keywords"); s!("commander_legal"); s!("price_usd");
    s!("uri"); s!("image_uri");
    i!("reserved"); i!("edhrec_rank"); i!("game_changer");
    m.insert(
        "cmc".into(),
        r.get::<_, Option<f64>>("cmc").unwrap_or(None).map_or(Value::Null, |v| json!(v)),
    );
    Ok(Value::Object(m))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    #[serde(default)]
    q: String,
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_limit() -> i64 { 30 }

async fn search_cards(Query(p): Query<SearchQuery>) -> Json<Value> {
    let Some(cdb) = open_cards_db() else { return Json(json!([])) };
    let like = format!("%{}%", fold_text(&p.q));
    let sql = format!(
        "SELECT {CARD_COLS} FROM cards c
         WHERE c.name_folded LIKE ?1
            OR c.oracle_id IN (SELECT oracle_id FROM names_pt WHERE printed_name_folded LIKE ?1)
         ORDER BY (CASE WHEN c.edhrec_rank IS NULL THEN 999999 ELSE c.edhrec_rank END) ASC
         LIMIT ?2"
    );
    let out = (|| -> rusqlite::Result<Vec<Value>> {
        let mut stmt = cdb.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params![like, p.limit], |r| card_row_to_json(r))?;
        rows.collect()
    })()
    .unwrap_or_default();
    Json(Value::Array(out))
}

/// Port of _lookup_card_in(): exact, then accent-folded, then Portuguese, then approximate.
pub fn lookup_card_in(cdb: &Connection, name: &str) -> Option<(Value, String)> {
    let folded = fold_text(name);
    let one = |sql: &str, p: &[&dyn rusqlite::ToSql]| -> Option<Value> {
        let mut stmt = cdb.prepare(sql).ok()?;
        stmt.query_row(p, |r| card_row_to_json(r)).ok()
    };

    if let Some(v) = one(
        &format!("SELECT {CARD_COLS} FROM cards WHERE name = ?1 COLLATE NOCASE"),
        &[&name],
    ) {
        return Some((v, "exata".into()));
    }
    if let Some(v) = one(
        &format!("SELECT {CARD_COLS} FROM cards WHERE name_folded = ?1"),
        &[&folded],
    ) {
        return Some((v, "exata".into()));
    }

    // Portuguese printed name -> oracle_id, only when unambiguous.
    let pt_ids = |sql: &str, param: &str| -> Vec<String> {
        (|| -> rusqlite::Result<Vec<String>> {
            let mut stmt = cdb.prepare(sql)?;
            let rows = stmt.query_map([param], |r| r.get::<_, String>(0))?;
            rows.collect()
        })()
        .unwrap_or_default()
    };
    let mut ids = pt_ids(
        "SELECT DISTINCT oracle_id FROM names_pt WHERE printed_name = ?1 COLLATE NOCASE",
        name,
    );
    if ids.is_empty() {
        ids = pt_ids(
            "SELECT DISTINCT oracle_id FROM names_pt WHERE printed_name_folded = ?1",
            &folded,
        );
    }
    if ids.len() == 1 {
        if let Some(v) = one(
            &format!("SELECT {CARD_COLS} FROM cards WHERE oracle_id = ?1"),
            &[&ids[0]],
        ) {
            return Some((v, "exata (nome em português)".into()));
        }
    }

    let like = format!("%{name}%");
    if let Some(v) = one(
        &format!("SELECT {CARD_COLS} FROM cards WHERE name LIKE ?1 COLLATE NOCASE LIMIT 1"),
        &[&like],
    ) {
        return Some((v, "aproximada".into()));
    }
    let like_folded = format!("%{folded}%");
    if let Some(v) = one(
        &format!("SELECT {CARD_COLS} FROM cards WHERE name_folded LIKE ?1 LIMIT 1"),
        &[&like_folded],
    ) {
        return Some((v, "aproximada".into()));
    }
    None
}

#[derive(Deserialize)]
pub struct CardQuery {
    oracle_id: Option<String>,
}

async fn get_card(Path(name): Path<String>, Query(q): Query<CardQuery>) -> impl IntoResponse {
    let Some(cdb) = open_cards_db() else {
        return not_found(&format!("Carta não encontrada: {name}")).into_response();
    };
    let found = if let Some(oid) = q.oracle_id.as_deref() {
        cdb.prepare(&format!("SELECT {CARD_COLS} FROM cards WHERE oracle_id = ?1"))
            .ok()
            .and_then(|mut s| s.query_row([oid], |r| card_row_to_json(r)).ok())
            .map(|v| (v, "exata (oracle_id)".to_string()))
    } else {
        lookup_card_in(&cdb, &name)
    };
    let Some((mut card, how)) = found else {
        return not_found(&format!("Carta não encontrada: {name}")).into_response();
    };

    let oid = card.get("oracle_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let pts: Vec<Value> = (|| -> rusqlite::Result<Vec<Value>> {
        let mut stmt = cdb.prepare(
            "SELECT DISTINCT printed_name, set_code FROM names_pt WHERE oracle_id = ?1",
        )?;
        let rows = stmt.query_map([&oid], |r| {
            Ok(json!({
                "printed_name": r.get::<_, Option<String>>(0)?,
                "set_code": r.get::<_, Option<String>>(1)?,
            }))
        })?;
        rows.collect()
    })()
    .unwrap_or_default();

    if let Some(m) = card.as_object_mut() {
        m.insert("match_type".into(), Value::from(how));
        m.insert("pt_names".into(), Value::Array(pts));
    }
    Json(card).into_response()
}

async fn card_variants(Path(name): Path<String>) -> Json<Value> {
    let Some(cdb) = open_cards_db() else { return Json(json!([])) };
    let out = (|| -> rusqlite::Result<Vec<Value>> {
        let mut stmt =
            cdb.prepare(&format!("SELECT {CARD_COLS} FROM cards WHERE name = ?1 COLLATE NOCASE"))?;
        let rows = stmt.query_map([&name], |r| card_row_to_json(r))?;
        rows.collect()
    })()
    .unwrap_or_default();
    Json(Value::Array(out))
}

pub fn router() -> Router {
    Router::new()
        .route("/api/cards/search", get(search_cards))
        .route("/api/cards/:name", get(get_card))
        .route("/api/cards/:name/variants", get(card_variants))
}
