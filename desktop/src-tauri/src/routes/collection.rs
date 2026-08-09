//! The collection: every physical copy the user owns.
//!
//! One row is one card (or a stack of identical ones via `quantity`). A card sleeved in two
//! decks is two rows, because it is two cards — `allocated_deck_id` says which deck holds each,
//! and NULL means it is free in the box. Counts throughout the app are copies, never distinct
//! names; collapsing the two is what made the home screen undercount.

use axum::{
    extract::{Path, Query},
    response::{IntoResponse, Json},
    routing::{get, patch},
    Router,
};
use rusqlite::{params, OptionalExtension};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::db::{deck_name, fold_text, log_activity, open_app_db, open_cards_db};
use crate::http::{db_unavailable, ok, server_error};

/// serde default for `quantity`: adding a card means one copy unless told otherwise.
fn one() -> i64 { 1 }

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
    // The collection stores whichever name the card was entered under — usually English. Matching
    // only that string is why searching "Anel Solar" found nothing while "Sol Ring" did. The
    // card index knows every printed name, so the search resolves the term through it first and
    // matches the resulting English names too. Accent-folded, so "cemiterio" finds "Cemitério".
    let mut extra_names: Vec<String> = Vec::new();
    if !p.q.is_empty() {
        if let Some(cdb) = open_cards_db() {
            let like = format!("%{}%", fold_text(&p.q));
            extra_names = (|| -> rusqlite::Result<Vec<String>> {
                let mut stmt = cdb.prepare(
                    "SELECT DISTINCT c.name FROM cards c
                     JOIN names_localized n ON n.oracle_id = c.oracle_id
                     WHERE n.printed_name_folded LIKE ?1 LIMIT 400",
                )?;
                let rows = stmt.query_map([&like], |r| r.get::<_, String>(0))?;
                rows.collect()
            })()
            .unwrap_or_default();
        }
        if extra_names.is_empty() {
            sql.push_str(" AND collection.card_name LIKE ?1 COLLATE NOCASE");
        } else {
            let ph = std::iter::repeat("?").take(extra_names.len())
                .collect::<Vec<_>>().join(",");
            sql.push_str(&format!(
                " AND (collection.card_name LIKE ?1 COLLATE NOCASE
                       OR collection.card_name COLLATE NOCASE IN ({}))",
                ph.replace('?', "?")
            ));
        }
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
                let mut params: Vec<&dyn rusqlite::ToSql> = vec![&like];
                for n in &extra_names {
                    params.push(n);
                }
                read(&mut stmt, params.as_slice())
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
                            "image_uri": r.get::<_, Option<String>>(3)?.map(|u| crate::images::local_url(&u)),
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
                            "image_uri": r.get::<_, Option<String>>(2)?.map(|u| crate::images::local_url(&u)),
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
    // Free/allocated are counted in copies, not names. The home screen used to show how many
    // distinct cards had a free copy, which reads as a card count and undercounts every playset.
    let free: i64 = con
        .query_row(
            "SELECT COALESCE(SUM(quantity), 0) FROM collection WHERE allocated_deck_id IS NULL",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    Json(json!({
        "total_units": total,
        "distinct_cards": distinct,
        "free_units": free,
        "allocated_units": total - free,
    }))
}

#[derive(Deserialize)]
pub struct CardCopiesQuery {
    name: String,
}

/// Every physical copy of one exact card name: how many are free and which decks hold the rest.
/// Exact match (folded), unlike /api/collection's substring search — the card modal is asking
/// about one card, and "Swamp" must not pick up "Swamp Mosquito".
async fn card_copies(Query(p): Query<CardCopiesQuery>) -> Json<Value> {
    let Ok(con) = open_app_db() else { return Json(json!({ "total": 0, "free": 0, "decks": [] })) };
    // fold_text() is a Rust function, not a registered SQLite one, so the accent-insensitive
    // comparison happens here rather than in the WHERE clause. The collection is a few hundred
    // rows — small enough that scanning it costs less than keeping a folded column in sync.
    let wanted = crate::db::fold_text(&p.name);
    let rows: Vec<(String, Option<i64>, Option<String>, i64)> = (|| -> rusqlite::Result<_> {
        let mut stmt = con.prepare(
            "SELECT collection.card_name, collection.allocated_deck_id, decks.name,
                    collection.quantity
             FROM collection LEFT JOIN decks ON decks.id = collection.allocated_deck_id",
        )?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?;
        rows.collect()
    })()
    .unwrap_or_default();

    let mut free = 0i64;
    let mut by_deck: HashMap<i64, (String, i64)> = HashMap::new();
    for (card_name, deck_id, deck_name, quantity) in rows {
        if crate::db::fold_text(&card_name) != wanted {
            continue;
        }
        match deck_id {
            None => free += quantity,
            Some(id) => {
                let e = by_deck.entry(id).or_insert_with(|| (deck_name.unwrap_or_default(), 0));
                e.1 += quantity;
            }
        }
    }
    let mut decks: Vec<Value> = by_deck
        .into_iter()
        .map(|(id, (name, qty))| json!({ "deck_id": id, "deck_name": name, "copies": qty }))
        .collect();
    decks.sort_by_key(|v| v["deck_name"].as_str().unwrap_or("").to_lowercase());
    let in_decks: i64 = decks.iter().filter_map(|v| v["copies"].as_i64()).sum();

    Json(json!({ "total": free + in_decks, "free": free, "decks": decks }))
}

#[derive(Deserialize)]
pub struct CollectionIn {
    card_name: String,
    #[serde(default)]
    set_code: Option<String>,
    #[serde(default)]
    artist: Option<String>,
    #[serde(default = "lang_en")]
    lang: String,
    #[serde(default = "one")]
    quantity: i64,
    #[serde(default)]
    notes: Option<String>,
    #[serde(default)]
    deck_id: Option<i64>,
    #[serde(default)]
    oracle_id: Option<String>,
}

fn lang_en() -> String { "en".into() }

async fn add_collection(Json(p): Json<CollectionIn>) -> impl IntoResponse {
    let Ok(con) = open_app_db() else {
        return db_unavailable().into_response();
    };
    if con
        .execute(
            "INSERT INTO collection (card_name, set_code, artist, lang, quantity, notes,
                                     allocated_deck_id, oracle_id)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![p.card_name, p.set_code, p.artist, p.lang, p.quantity, p.notes, p.deck_id, p.oracle_id],
        )
        .is_err()
    {
        return server_error("Falha ao adicionar").into_response();
    }
    let entry_id = con.last_insert_rowid();
    let qty_label = if p.quantity != 1 { format!("{}x ", p.quantity) } else { String::new() };

    match p.deck_id {
        Some(did) => {
            let dn = deck_name(&con, did).unwrap_or_else(|| "?".into());
            let _ = con.execute(
                "INSERT INTO deck_cards (deck_id, card_name, quantity, oracle_id)
                 VALUES (?1, ?2, ?3, ?4)",
                params![did, p.card_name, p.quantity, p.oracle_id],
            );
            log_activity(
                &con,
                "card_new",
                &format!("{qty_label}{} adicionada à coleção e ao deck {dn}", p.card_name),
            );
        }
        None => log_activity(
            &con,
            "card_new",
            &format!("{qty_label}{} adicionada à coleção", p.card_name),
        ),
    }
    Json(json!({ "ok": true, "id": entry_id })).into_response()
}

#[derive(Deserialize)]
pub struct AllocateIn {
    #[serde(default)]
    deck_id: Option<i64>,
}

async fn allocate_collection(
    Path(entry_id): Path<i64>,
    Json(p): Json<AllocateIn>,
) -> impl IntoResponse {
    let Ok(con) = open_app_db() else {
        return db_unavailable().into_response();
    };
    let card_name: Option<String> = con
        .query_row("SELECT card_name FROM collection WHERE id = ?1", [entry_id], |r| r.get(0))
        .optional()
        .ok()
        .flatten();
    let _ = con.execute(
        "UPDATE collection SET allocated_deck_id = ?1 WHERE id = ?2",
        params![p.deck_id, entry_id],
    );
    if let Some(cn) = card_name {
        match p.deck_id {
            Some(did) => {
                let dn = deck_name(&con, did).unwrap_or_else(|| "?".into());
                log_activity(&con, "card_added_deck", &format!("{cn} alocada ao deck {dn}"));
            }
            None => log_activity(
                &con,
                "card_removed_deck",
                &format!("{cn} liberada (ficou fora de deck)"),
            ),
        }
    }
    ok().into_response()
}

pub fn router() -> Router {
    Router::new()
        .route("/api/collection", get(list_collection).post(add_collection))
        .route("/api/collection/total", get(collection_total))
        .route("/api/collection/copies", get(card_copies))
        .route("/api/collection/:id/allocate", patch(allocate_collection))
}
