//! HTTP API — port of webapp/server.py's read endpoints.
//!
//! Kept route-for-route and shape-for-shape identical to the Python version so the existing
//! frontend (webapp/static/js/api.js) runs against this unchanged. Card lookups use the batched
//! IN(...) form and the accent-folded columns, matching the current Python behaviour rather than
//! the older per-card / NOCASE-only one.

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::HashMap;

use crate::db::{fold_text, open_app_db, open_cards_db};
use crate::paths;

pub const CARD_COLS: &str = "oracle_id, name, mana_cost, cmc, type_line, oracle_text, power, toughness, \
     loyalty, colors, color_identity, rarity, set_code, keywords, commander_legal, price_usd, \
     reserved, edhrec_rank, uri, image_uri, game_changer";

/// Turns a card row into the same JSON object shape the Python endpoints return.
fn card_row_to_json(r: &rusqlite::Row) -> rusqlite::Result<Value> {
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

fn err(msg: &str) -> (StatusCode, Json<Value>) {
    (StatusCode::NOT_FOUND, Json(json!({ "detail": msg })))
}

// ------------------------------------------------------------------- cards ----

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
        return err(&format!("Carta não encontrada: {name}")).into_response();
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
        return err(&format!("Carta não encontrada: {name}")).into_response();
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

// ------------------------------------------------------------------- decks ----

const TYPE_ORDER: &[&str] = &[
    "Land", "Creature", "Planeswalker", "Battle", "Artifact", "Enchantment", "Instant", "Sorcery",
];

fn classify(type_line: &str) -> String {
    for t in TYPE_ORDER {
        if type_line.contains(t) {
            return (*t).to_string();
        }
    }
    "Outro".into()
}

async fn list_decks() -> Json<Value> {
    let Ok(con) = open_app_db() else { return Json(json!([])) };
    let cdb = open_cards_db();

    type DeckRow = (i64, String, String, Option<String>, Option<String>, Option<String>, Option<String>);
    let decks: Vec<DeckRow> = (|| -> rusqlite::Result<Vec<_>> {
        let mut stmt = con.prepare(
            "SELECT id, name, commander_name, commander_name_2, philosophy, tags, created_at
             FROM decks ORDER BY id",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, Option<String>>(6)?,
            ))
        })?;
        rows.collect()
    })()
    .unwrap_or_default();

    let mut out = Vec::new();
    for (id, name, commander, commander2, philosophy, tags, created_at) in decks {
        let total: i64 = con
            .query_row(
                "SELECT COALESCE(SUM(quantity),0) FROM deck_cards WHERE deck_id = ?1",
                [id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let wins: i64 = con
            .query_row(
                "SELECT COUNT(*) FROM games WHERE deck_id = ?1 AND result = 'vitoria'",
                [id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let losses: i64 = con
            .query_row(
                "SELECT COUNT(*) FROM games WHERE deck_id = ?1 AND result = 'derrota'",
                [id],
                |r| r.get(0),
            )
            .unwrap_or(0);

        // Commander art: the tile shows just the illustration, so /normal/ -> /art_crop/.
        // Partner decks have a second commander, whose art the tile shows alongside the first.
        let art_of = |c: &Connection, nm: &str| -> (Value, Value) {
            match lookup_card_in(c, nm) {
                Some((card, _)) => (
                    card.get("image_uri")
                        .and_then(|v| v.as_str())
                        .map(|u| Value::from(u.replace("/normal/", "/art_crop/")))
                        .unwrap_or(Value::Null),
                    card.get("color_identity").cloned().unwrap_or(Value::Null),
                ),
                None => (Value::Null, Value::Null),
            }
        };
        let mut commander_image = Value::Null;
        let mut commander_image_2 = Value::Null;
        let mut color_identity = Value::Null;
        if let Some(ref c) = cdb {
            let (img, ci) = art_of(c, &commander);
            commander_image = img;
            color_identity = ci;
            if let Some(ref c2) = commander2 {
                if !c2.is_empty() {
                    commander_image_2 = art_of(c, c2).0;
                }
            }
        }

        out.push(json!({
            "id": id, "name": name, "commander_name": commander,
            "commander_name_2": commander2, "philosophy": philosophy, "tags": tags,
            "created_at": created_at,
            "total_cards": total, "wins": wins, "losses": losses,
            "commander_image": commander_image, "commander_image_2": commander_image_2,
            "color_identity": color_identity,
        }));
    }
    Json(Value::Array(out))
}

async fn get_deck(Path(deck_id): Path<i64>) -> impl IntoResponse {
    let Ok(con) = open_app_db() else { return err("Deck não encontrado").into_response() };
    let deck = con.query_row(
        "SELECT id, name, commander_name, commander_name_2, philosophy, tags, created_at
         FROM decks WHERE id = ?1",
        [deck_id],
        |r| {
            Ok(json!({
                "id": r.get::<_, i64>(0)?,
                "name": r.get::<_, String>(1)?,
                "commander_name": r.get::<_, String>(2)?,
                "commander_name_2": r.get::<_, Option<String>>(3)?,
                "philosophy": r.get::<_, Option<String>>(4)?,
                "tags": r.get::<_, Option<String>>(5)?,
                "created_at": r.get::<_, Option<String>>(6)?,
            }))
        },
    );
    let Ok(mut deck) = deck else { return err("Deck não encontrado").into_response() };

    // (id, card_name, quantity, is_commander, oracle_id)
    let dcards: Vec<(i64, String, i64, i64, Option<String>)> = (|| -> rusqlite::Result<Vec<_>> {
        let mut stmt = con.prepare(
            "SELECT id, card_name, quantity, is_commander, oracle_id FROM deck_cards
             WHERE deck_id = ?1 ORDER BY is_commander DESC, card_name",
        )?;
        let rows = stmt.query_map([deck_id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get::<_, Option<String>>(4)?))
        })?;
        rows.collect()
    })()
    .unwrap_or_default();

    // Cards allocated to another deck under the same name (cross-deck duplicates).
    let mut other_map: HashMap<String, Vec<Value>> = HashMap::new();
    if let Ok(mut stmt) = con.prepare(
        "SELECT card_name, decks.name, decks.id FROM collection
         JOIN decks ON decks.id = collection.allocated_deck_id
         WHERE collection.allocated_deck_id != ?1",
    ) {
        if let Ok(rows) = stmt.query_map([deck_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?))
        }) {
            for row in rows.flatten() {
                other_map
                    .entry(row.0)
                    .or_default()
                    .push(json!({ "deck": row.1, "deck_id": row.2 }));
            }
        }
    }

    let cdb = open_cards_db();
    let mut by_type: HashMap<String, Vec<Value>> = HashMap::new();
    let mut total = 0i64;
    let mut mana_curve: HashMap<i64, i64> = HashMap::new();

    for (row_id, card_name, quantity, is_commander, oracle_id) in dcards {
        let info = cdb.as_ref().and_then(|c| {
            oracle_id
                .as_deref()
                .and_then(|oid| {
                    c.prepare(&format!("SELECT {CARD_COLS} FROM cards WHERE oracle_id = ?1"))
                        .ok()
                        .and_then(|mut s| s.query_row([oid], |r| card_row_to_json(r)).ok())
                })
                .or_else(|| lookup_card_in(c, &card_name).map(|(v, _)| v))
        });

        let type_line = info
            .as_ref()
            .and_then(|v| v.get("type_line"))
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        let cat = if is_commander == 1 { "Comandante".to_string() } else { classify(&type_line) };
        let get = |k: &str| info.as_ref().and_then(|v| v.get(k)).cloned().unwrap_or(Value::Null);

        by_type.entry(cat).or_default().push(json!({
            "card_name": card_name, "quantity": quantity, "id": row_id,
            "oracle_id": get("oracle_id"), "mana_cost": get("mana_cost"),
            "type_line": if type_line == "?" { Value::from("?") } else { Value::from(type_line.clone()) },
            "image_uri": get("image_uri"), "cmc": get("cmc"),
            "price_usd": get("price_usd"), "edhrec_rank": get("edhrec_rank"),
            "colors": get("colors"), "color_identity": get("color_identity"),
            "rarity": get("rarity"),
            "shared_with": other_map.get(&card_name).cloned().map(Value::Array).unwrap_or(json!([])),
        }));

        total += quantity;
        if is_commander == 0 && !type_line.contains("Land") {
            let cmc = info
                .as_ref()
                .and_then(|v| v.get("cmc"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as i64;
            *mana_curve.entry(cmc).or_insert(0) += quantity;
        }
    }

    let curve: Map<String, Value> =
        mana_curve.into_iter().map(|(k, v)| (k.to_string(), Value::from(v))).collect();
    if let Some(m) = deck.as_object_mut() {
        m.insert("total_cards".into(), Value::from(total));
        m.insert("is_valid_100".into(), Value::from(total == 100));
        m.insert(
            "by_type".into(),
            Value::Object(by_type.into_iter().map(|(k, v)| (k, Value::Array(v))).collect()),
        );
        m.insert("mana_curve".into(), Value::Object(curve));
        // Per-card ownership so the deck view can flag what isn't owned (or is on loan from
        // another deck). Computed on read so it tracks the collection as it changes.
        let own = crate::wizard::deck_ownership(&con, deck_id);
        m.insert(
            "ownership".into(),
            Value::Object(own.into_iter().collect::<serde_json::Map<_, _>>()),
        );
    }
    Json(deck).into_response()
}

// -------------------------------------------------------------- collection ----

#[derive(Deserialize)]
pub struct CollectionQuery {
    #[serde(default = "all_status")]
    status: String,
    #[serde(default)]
    q: String,
}
fn all_status() -> String { "all".into() }

async fn list_collection(Query(p): Query<CollectionQuery>) -> Json<Value> {
    let Ok(con) = open_app_db() else { return Json(json!([])) };

    let mut sql = String::from(
        "SELECT collection.id, collection.card_name, collection.quantity, collection.lang,
                collection.set_code, collection.allocated_deck_id, decks.name
         FROM collection LEFT JOIN decks ON decks.id = collection.allocated_deck_id WHERE 1=1",
    );
    match p.status.as_str() {
        "free" => sql.push_str(" AND collection.allocated_deck_id IS NULL"),
        "allocated" => sql.push_str(" AND collection.allocated_deck_id IS NOT NULL"),
        _ => {}
    }
    if !p.q.is_empty() {
        sql.push_str(" AND collection.card_name LIKE ?1 COLLATE NOCASE");
    }
    sql.push_str(" ORDER BY collection.card_name");

    type Row = (i64, String, i64, String, Option<String>, Option<i64>, Option<String>);
    let read = |stmt: &mut rusqlite::Statement, params: &[&dyn rusqlite::ToSql]| -> Vec<Row> {
        stmt.query_map(params, |r| {
            Ok((
                r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?,
                r.get::<_, Option<String>>(4)?, r.get::<_, Option<i64>>(5)?,
                r.get::<_, Option<String>>(6)?,
            ))
        })
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default()
    };
    let rows: Vec<Row> = match con.prepare(&sql) {
        Ok(mut stmt) => {
            if p.q.is_empty() {
                read(&mut stmt, &[])
            } else {
                let like = format!("%{}%", p.q);
                read(&mut stmt, &[&like])
            }
        }
        Err(_) => Vec::new(),
    };

    // Group by card name, summing units — same shape the Python endpoint returns.
    let mut order: Vec<String> = Vec::new();
    let mut grouped: HashMap<String, (i64, Vec<Value>, Vec<i64>)> = HashMap::new();
    for (id, card_name, quantity, lang, set_code, deck_id, deck_name) in rows {
        let e = grouped.entry(card_name.clone()).or_insert_with(|| {
            order.push(card_name.clone());
            (0, Vec::new(), Vec::new())
        });
        e.0 += quantity;
        e.2.push(id);
        e.1.push(json!({
            "deck_id": deck_id,
            "deck_name": deck_name.unwrap_or_else(|| "Livre".into()),
            "quantity": quantity, "lang": lang, "set_code": set_code,
        }));
    }

    // Batched enrichment (one IN(...) per chunk) instead of a query per card name.
    let cdb = open_cards_db();
    let mut by_name: HashMap<String, Value> = HashMap::new();
    if let Some(ref c) = cdb {
        for chunk in order.chunks(400) {
            let ph = std::iter::repeat("?").take(chunk.len()).collect::<Vec<_>>().join(",");
            let sql = format!(
                "SELECT name, type_line, mana_cost, image_uri, colors, rarity, price_usd
                 FROM cards WHERE name COLLATE NOCASE IN ({ph})"
            );
            if let Ok(mut stmt) = c.prepare(&sql) {
                let params: Vec<&dyn rusqlite::ToSql> =
                    chunk.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
                if let Ok(rows) = stmt.query_map(params.as_slice(), |r| {
                    let name: String = r.get(0)?;
                    Ok((
                        name,
                        json!({
                            "type_line": r.get::<_, Option<String>>(1)?,
                            "mana_cost": r.get::<_, Option<String>>(2)?,
                            "image_uri": r.get::<_, Option<String>>(3)?,
                            "colors": r.get::<_, Option<String>>(4)?,
                            "rarity": r.get::<_, Option<String>>(5)?,
                            "price_usd": r.get::<_, Option<String>>(6)?,
                        }),
                    ))
                }) {
                    for (name, v) in rows.flatten() {
                        by_name.entry(name.to_lowercase()).or_insert(v);
                    }
                }
            }
        }
    }

    let mut out = Vec::new();
    for card_name in &order {
        let (total_quantity, decks, entry_ids) = grouped.get(card_name).unwrap();
        let mut obj = json!({
            "card_name": card_name,
            "total_quantity": total_quantity,
            "decks": decks,
            "entry_ids": entry_ids,
        });
        let enrich = by_name.get(&card_name.to_lowercase()).cloned().or_else(|| {
            // Prefix fallback for hand-entered names that don't match exactly.
            cdb.as_ref().and_then(|c| {
                c.prepare(
                    "SELECT type_line, mana_cost, image_uri, colors, rarity, price_usd
                     FROM cards WHERE name LIKE ?1 COLLATE NOCASE LIMIT 1",
                )
                .ok()
                .and_then(|mut s| {
                    s.query_row([format!("{card_name}%")], |r| {
                        Ok(json!({
                            "type_line": r.get::<_, Option<String>>(0)?,
                            "mana_cost": r.get::<_, Option<String>>(1)?,
                            "image_uri": r.get::<_, Option<String>>(2)?,
                            "colors": r.get::<_, Option<String>>(3)?,
                            "rarity": r.get::<_, Option<String>>(4)?,
                            "price_usd": r.get::<_, Option<String>>(5)?,
                        }))
                    })
                    .ok()
                })
            })
        });
        if let (Some(Value::Object(e)), Some(m)) = (enrich, obj.as_object_mut()) {
            for (k, v) in e {
                m.insert(k, v);
            }
        }
        out.push(obj);
    }
    Json(Value::Array(out))
}

async fn collection_total() -> Json<Value> {
    let Ok(con) = open_app_db() else { return Json(json!({"total_units":0,"distinct_cards":0})) };
    let total: i64 = con
        .query_row("SELECT COALESCE(SUM(quantity), 0) FROM collection", [], |r| r.get(0))
        .unwrap_or(0);
    let distinct: i64 = con
        .query_row("SELECT COUNT(DISTINCT card_name) FROM collection", [], |r| r.get(0))
        .unwrap_or(0);
    Json(json!({ "total_units": total, "distinct_cards": distinct }))
}

// ------------------------------------------------------- games / activity ----

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

/// Cached EDHREC synergy for this deck's commander — reads local files only, never the network.
async fn deck_synergy(Path(deck_id): Path<i64>) -> Json<Value> {
    let Ok(con) = open_app_db() else { return Json(json!({ "cached": false })) };
    let commander: Option<String> = con
        .query_row("SELECT commander_name FROM decks WHERE id = ?1", [deck_id], |r| r.get(0))
        .ok();
    let Some(commander) = commander else { return Json(json!({ "cached": false })) };

    let in_deck: Vec<String> = (|| -> rusqlite::Result<Vec<String>> {
        let mut stmt = con.prepare("SELECT card_name FROM deck_cards WHERE deck_id = ?1")?;
        let rows = stmt.query_map([deck_id], |r| r.get::<_, String>(0))?;
        rows.collect()
    })()
    .unwrap_or_default();

    match crate::edhrec::recommendations(&commander, &in_deck) {
        Some((recs, similar)) => Json(json!({
            "cached": true,
            "recommendations": recs,
            "similar_commanders": similar,
        })),
        None => Json(json!({
            "cached": false,
            "message": format!("Sem cache do EDHREC para {commander}."),
        })),
    }
}

async fn deck_tags(Path(_id): Path<i64>) -> Json<Value> {
    Json(json!([]))
}

pub fn router() -> Router {
    Router::new()
        .route("/api/cards/search", get(search_cards))
        .route("/api/cards/:name", get(get_card))
        .route("/api/cards/:name/variants", get(card_variants))
        .route("/api/decks", get(list_decks))
        .route("/api/decks/:id", get(get_deck))
        .route("/api/decks/:id/synergy", get(deck_synergy))
        .route("/api/decks/:id/tags", get(deck_tags))
        .route("/api/collection", get(list_collection))
        .route("/api/collection/total", get(collection_total))
        .route("/api/games", get(list_games))
        .route("/api/games/stats", get(games_stats))
        .route("/api/activity", get(list_activity))
        .route("/api/data/info", get(data_info))
        .route("/api/data/update/status", get(data_update_status))
        .route("/api/decks/auto-build/status", get(crate::writes::auto_build_status))
}
