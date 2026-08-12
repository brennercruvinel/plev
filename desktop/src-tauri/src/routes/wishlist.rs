//! Cards the user wants but doesn't own.
//!
//! Shaped like the collection on purpose — same grouped-by-name response, same enrichment — so
//! the wishlist screen can reuse the collection's grid, filters and card modal without a parallel
//! set of components. What it deliberately does *not* have is `allocated_deck_id`: a wishlist
//! entry is not cardboard, and nothing that counts what you own may see it.

use axum::{
    extract::{Path, Query},
    response::{IntoResponse, Json},
    routing::{delete, get, post},
    Router,
};
use rusqlite::{params, OptionalExtension};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::db::{fold_text, log_activity, open_app_db, open_cards_db};
use crate::http::{db_unavailable, not_found, ok, server_error};
use crate::routes::cards::CARD_COLS;

#[derive(Deserialize)]
pub struct WishQuery {
    #[serde(default)]
    q: String,
}

async fn list_wishlist(Query(p): Query<WishQuery>) -> Json<Value> {
    let Ok(con) = open_app_db() else { return Json(json!([])) };

    // Same trick as the collection: resolve the search term through the card index first, so a
    // card saved in English is still found by typing its Portuguese name.
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
    }

    let mut sql = String::from(
        "SELECT id, card_name, quantity, lang, set_code, artist, notes FROM wishlist WHERE 1=1",
    );
    if !p.q.is_empty() {
        if extra_names.is_empty() {
            sql.push_str(" AND card_name LIKE ?1 COLLATE NOCASE");
        } else {
            let ph = std::iter::repeat("?").take(extra_names.len()).collect::<Vec<_>>().join(",");
            sql.push_str(&format!(
                " AND (card_name LIKE ?1 COLLATE NOCASE OR card_name COLLATE NOCASE IN ({ph}))"
            ));
        }
    }
    sql.push_str(" ORDER BY card_name, id");

    type Row = (i64, String, i64, String, Option<String>, Option<String>, Option<String>);
    let read = |stmt: &mut rusqlite::Statement, params: &[&dyn rusqlite::ToSql]| -> Vec<Row> {
        stmt.query_map(params, |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?))
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
                let mut ps: Vec<&dyn rusqlite::ToSql> = vec![&like];
                for n in &extra_names {
                    ps.push(n);
                }
                read(&mut stmt, ps.as_slice())
            }
        }
        Err(_) => Vec::new(),
    };

    // Group by name, keeping each stored entry so the UI can list and delete them individually.
    let mut order: Vec<String> = Vec::new();
    let mut grouped: HashMap<String, (i64, Vec<Value>)> = HashMap::new();
    for (id, card_name, quantity, lang, set_code, artist, notes) in rows {
        let e = grouped.entry(card_name.clone()).or_insert_with(|| {
            order.push(card_name.clone());
            (0, Vec::new())
        });
        e.0 += quantity;
        e.1.push(json!({
            "id": id, "quantity": quantity, "lang": lang,
            "set_code": set_code, "artist": artist, "notes": notes,
        }));
    }

    let cdb = open_cards_db();
    let mut out = Vec::new();
    for card_name in &order {
        let (total, entries) = grouped.get(card_name).unwrap();
        let mut obj = json!({
            "card_name": card_name,
            "total_quantity": total,
            "entries": entries,
        });
        if let Some(ref c) = cdb {
            if let Some((card, _)) = crate::routes::cards::lookup_card_in(c, card_name) {
                if let (Some(m), Some(src)) = (obj.as_object_mut(), card.as_object()) {
                    for k in [
                        "type_line", "mana_cost", "image_uri", "image_uri_back", "colors",
                        "rarity", "price_usd", "cmc", "layout",
                    ] {
                        if let Some(v) = src.get(k) {
                            m.insert(k.into(), v.clone());
                        }
                    }
                }
            }
        }
        out.push(obj);
    }
    Json(Value::Array(out))
}

async fn wishlist_total() -> Json<Value> {
    let Ok(con) = open_app_db() else { return Json(json!({"total_units":0,"distinct_cards":0})) };
    let total: i64 = con
        .query_row("SELECT COALESCE(SUM(quantity), 0) FROM wishlist", [], |r| r.get(0))
        .unwrap_or(0);
    let distinct: i64 = con
        .query_row("SELECT COUNT(DISTINCT card_name) FROM wishlist", [], |r| r.get(0))
        .unwrap_or(0);
    // What the whole list would cost at Scryfall's current prices, which is the number anyone
    // keeping a buy-list actually wants.
    let mut usd = 0.0f64;
    if let Some(cdb) = open_cards_db() {
        if let Ok(mut stmt) =
            con.prepare("SELECT card_name, quantity FROM wishlist")
        {
            if let Ok(rows) = stmt.query_map([], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
            }) {
                for (name, qty) in rows.flatten() {
                    if let Some((card, _)) = crate::routes::cards::lookup_card_in(&cdb, &name) {
                        if let Some(p) = card.get("price_usd").and_then(|v| v.as_str()) {
                            usd += p.parse::<f64>().unwrap_or(0.0) * qty as f64;
                        }
                    }
                }
            }
        }
    }
    Json(json!({
        "total_units": total,
        "distinct_cards": distinct,
        "price_usd": (usd * 100.0).round() / 100.0,
    }))
}

#[derive(Deserialize)]
pub struct WishlistIn {
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
    oracle_id: Option<String>,
}
fn lang_en() -> String { "en".into() }
fn one() -> i64 { 1 }

async fn add_wishlist(Json(p): Json<WishlistIn>) -> impl IntoResponse {
    let Ok(con) = open_app_db() else {
        return db_unavailable().into_response();
    };
    // Same canonical spelling as everywhere else, so a wishlist entry and the collection row it
    // eventually becomes are recognisably the same card.
    let card_name = open_cards_db()
        .and_then(|cdb| crate::routes::cards::canonical_name(&cdb, &p.card_name))
        .unwrap_or_else(|| p.card_name.clone());

    if con
        .execute(
            "INSERT INTO wishlist (card_name, set_code, artist, lang, quantity, notes, oracle_id)
             VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![card_name, p.set_code, p.artist, p.lang, p.quantity, p.notes, p.oracle_id],
        )
        .is_err()
    {
        return server_error("Falha ao adicionar à wishlist").into_response();
    }
    let qty_label = if p.quantity != 1 { format!("{}x ", p.quantity) } else { String::new() };
    log_activity(&con, "wishlist_add", &format!("{qty_label}{card_name} entrou na wishlist"));
    Json(json!({ "ok": true, "id": con.last_insert_rowid() })).into_response()
}

async fn delete_wishlist_entry(Path(entry_id): Path<i64>) -> impl IntoResponse {
    let Ok(con) = open_app_db() else {
        return db_unavailable().into_response();
    };
    let row: Option<(String, i64)> = con
        .query_row(
            "SELECT card_name, quantity FROM wishlist WHERE id = ?1",
            [entry_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .ok()
        .flatten();
    let Some((card_name, quantity)) = row else {
        return not_found("Item não encontrado").into_response();
    };

    // One unit at a time, matching how the collection deletes copies.
    if quantity > 1 {
        let _ = con.execute("UPDATE wishlist SET quantity = quantity - 1 WHERE id = ?1", [entry_id]);
    } else {
        let _ = con.execute("DELETE FROM wishlist WHERE id = ?1", [entry_id]);
    }
    let remaining: i64 = con
        .query_row(
            "SELECT COALESCE(SUM(quantity), 0) FROM wishlist WHERE card_name = ?1 COLLATE NOCASE",
            [&card_name],
            |r| r.get(0),
        )
        .unwrap_or(0);
    log_activity(&con, "wishlist_remove", &format!("{card_name} saiu da wishlist"));
    Json(json!({ "ok": true, "remaining": remaining, "card_name": card_name })).into_response()
}

/// Moves a wishlist entry into the collection — the whole point of keeping the list.
async fn acquire_wishlist_entry(Path(entry_id): Path<i64>) -> impl IntoResponse {
    let Ok(con) = open_app_db() else {
        return db_unavailable().into_response();
    };
    let row: Option<(String, Option<String>, Option<String>, String, i64, Option<String>, Option<String>)> =
        con.query_row(
            "SELECT card_name, set_code, artist, lang, quantity, notes, oracle_id
             FROM wishlist WHERE id = ?1",
            [entry_id],
            |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?))
            },
        )
        .optional()
        .ok()
        .flatten();
    let Some((card_name, set_code, artist, lang, quantity, notes, oracle_id)) = row else {
        return not_found("Item não encontrado").into_response();
    };

    let _ = con.execute(
        "INSERT INTO collection (card_name, set_code, artist, lang, quantity, notes, oracle_id)
         VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![card_name, set_code, artist, lang, quantity, notes, oracle_id],
    );
    let _ = con.execute("DELETE FROM wishlist WHERE id = ?1", [entry_id]);
    log_activity(
        &con,
        "wishlist_acquired",
        &format!("{card_name} saiu da wishlist e entrou na coleção"),
    );
    ok().into_response()
}

pub fn router() -> Router {
    Router::new()
        .route("/api/wishlist", get(list_wishlist).post(add_wishlist))
        .route("/api/wishlist/total", get(wishlist_total))
        .route("/api/wishlist/:id", delete(delete_wishlist_entry))
        .route("/api/wishlist/:id/acquire", post(acquire_wishlist_entry))
}

// Kept so the enrichment above compiles against the same column list the rest of the app uses.
const _: &str = CARD_COLS;
