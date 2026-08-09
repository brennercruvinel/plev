//! Decks: the list, one deck's contents, the cards in it, auto-build, decklist import and
//! EDHREC synergy.
//!
//! Deck contents live in two places on purpose. `deck_cards` is the *list* — what the deck is
//! meant to contain. `collection` is the *cardboard* — one row per physical copy, pointing at
//! the deck it is sleeved in. Every write here has to keep those two honest with each other:
//! adding a card claims a copy, removing one releases it back to free.

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get, post},
    Router,
};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::HashMap;

use crate::db::{deck_name, log_activity, open_app_db, open_cards_db};
use crate::http::{bad_gateway, bad_request, db_unavailable, not_found, ok, server_error};
use crate::routes::cards::{card_row_to_json, lookup_card_in, CARD_COLS};

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
                        .map(|u| Value::from(u.replace("/normal/", "/art_crop/")))  // already a /img/ path
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
    let Ok(con) = open_app_db() else { return not_found("Deck não encontrado").into_response() };
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
    let Ok(mut deck) = deck else { return not_found("Deck não encontrado").into_response() };

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

/// Trims, drops empties, de-dupes case-insensitively — port of _normalize_tags().
fn normalize_tags(raw: Option<&str>) -> Option<String> {
    let raw = raw?;
    let mut seen: Vec<String> = Vec::new();
    let mut out: Vec<String> = Vec::new();
    for t in raw.split(',') {
        let t = t.trim();
        if t.is_empty() {
            continue;
        }
        let lower = t.to_lowercase();
        if !seen.contains(&lower) {
            seen.push(lower);
            out.push(t.to_string());
        }
    }
    if out.is_empty() { None } else { Some(out.join(", ")) }
}

#[derive(Deserialize)]
pub struct DeckIn {
    name: String,
    commander_name: String,
    #[serde(default)]
    commander_name_2: Option<String>,
    #[serde(default)]
    philosophy: Option<String>,
    #[serde(default)]
    tags: Option<String>,
}

async fn create_deck(Json(p): Json<DeckIn>) -> impl IntoResponse {
    let Ok(con) = open_app_db() else {
        return db_unavailable().into_response();
    };
    let c2 = p.commander_name_2.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let tags = normalize_tags(p.tags.as_deref());

    if con
        .execute(
            "INSERT INTO decks (name, commander_name, commander_name_2, philosophy, tags)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![p.name, p.commander_name, c2, p.philosophy, tags],
        )
        .is_err()
    {
        return server_error("Falha ao criar o deck").into_response();
    }
    let deck_id = con.last_insert_rowid();

    let _ = con.execute(
        "INSERT INTO deck_cards (deck_id, card_name, quantity, is_commander) VALUES (?1, ?2, 1, 1)",
        params![deck_id, p.commander_name],
    );
    if let Some(c2) = c2 {
        let _ = con.execute(
            "INSERT INTO deck_cards (deck_id, card_name, quantity, is_commander) VALUES (?1, ?2, 1, 1)",
            params![deck_id, c2],
        );
    }
    let label = match c2 {
        Some(c2) => format!("{} + {}", p.commander_name, c2),
        None => p.commander_name.clone(),
    };
    log_activity(&con, "deck_built", &format!("Deck {} criado (comandante: {label})", p.name));
    Json(json!({ "ok": true, "id": deck_id })).into_response()
}

async fn update_deck(Path(deck_id): Path<i64>, Json(p): Json<DeckIn>) -> impl IntoResponse {
    let Ok(con) = open_app_db() else {
        return db_unavailable().into_response();
    };
    let existing: Option<(String, Option<String>)> = con
        .query_row(
            "SELECT commander_name, commander_name_2 FROM decks WHERE id = ?1",
            [deck_id],
            |r| Ok((r.get(0)?, r.get::<_, Option<String>>(1)?)),
        )
        .optional()
        .ok()
        .flatten();
    let Some((old_c1, old_c2)) = existing else {
        return not_found("Deck não encontrado").into_response();
    };

    let new_c1 = p.commander_name.trim().to_string();
    if new_c1.is_empty() {
        return bad_request("Comandante principal é obrigatório.").into_response();
    }
    let new_c2 = p.commander_name_2.as_deref().map(str::trim).filter(|s| !s.is_empty());

    let _ = con.execute(
        "UPDATE decks SET name = ?1, philosophy = ?2, commander_name = ?3, commander_name_2 = ?4,
                          tags = ?5 WHERE id = ?6",
        params![p.name, p.philosophy, new_c1, new_c2, normalize_tags(p.tags.as_deref()), deck_id],
    );

    // Reconcile the is_commander rows to the new commander set.
    let old_set: Vec<String> = [Some(old_c1), old_c2].into_iter().flatten().collect();
    let new_set: Vec<String> =
        [Some(new_c1.clone()), new_c2.map(str::to_string)].into_iter().flatten().collect();
    let eq = |a: &str, b: &str| a.eq_ignore_ascii_case(b);

    for old in old_set.iter().filter(|o| !new_set.iter().any(|n| eq(n, o))) {
        let _ = con.execute(
            "DELETE FROM deck_cards WHERE deck_id = ?1 AND card_name = ?2 COLLATE NOCASE
             AND is_commander = 1",
            params![deck_id, old],
        );
    }
    for new in new_set.iter().filter(|n| !old_set.iter().any(|o| eq(o, n))) {
        let found: Option<i64> = con
            .query_row(
                "SELECT id FROM deck_cards WHERE deck_id = ?1 AND card_name = ?2 COLLATE NOCASE",
                params![deck_id, new],
                |r| r.get(0),
            )
            .optional()
            .ok()
            .flatten();
        match found {
            Some(id) => {
                let _ = con.execute("UPDATE deck_cards SET is_commander = 1 WHERE id = ?1", [id]);
            }
            None => {
                let _ = con.execute(
                    "INSERT INTO deck_cards (deck_id, card_name, quantity, is_commander)
                     VALUES (?1, ?2, 1, 1)",
                    params![deck_id, new],
                );
            }
        }
    }

    let label = match new_c2 {
        Some(c2) => format!("{new_c1} + {c2}"),
        None => new_c1,
    };
    log_activity(&con, "deck_built", &format!("Deck {} editado (comandante: {label})", p.name));
    ok().into_response()
}

#[derive(Deserialize)]
pub struct DeleteDeckQuery {
    /// "free" (default) keeps the deck's cards in the collection, just unallocated — the deck was
    /// taken apart but you still own the cards. "remove" deletes those collection rows too, for a
    /// deck whose cards were never physically owned (e.g. an auto-built list used as a shopping
    /// plan). Defaulting to "free" keeps the destructive option strictly opt-in.
    #[serde(default)]
    mode: Option<String>,
}

async fn delete_deck(
    Path(deck_id): Path<i64>,
    Query(q): Query<DeleteDeckQuery>,
) -> impl IntoResponse {
    let Ok(con) = open_app_db() else {
        return db_unavailable().into_response();
    };
    let Some(name) = deck_name(&con, deck_id) else {
        return not_found("Deck não encontrado").into_response();
    };
    let remove_from_collection = q.mode.as_deref() == Some("remove");

    let mut removed = 0usize;
    if remove_from_collection {
        // Only the rows this deck owns — anything already free, or allocated elsewhere, is
        // untouched. Runs before the DELETE, since the FK would otherwise null the link first.
        removed = con
            .execute("DELETE FROM collection WHERE allocated_deck_id = ?1", [deck_id])
            .unwrap_or(0);
    }
    // deck_cards cascades; any remaining collection rows fall back to ON DELETE SET NULL.
    let _ = con.execute("DELETE FROM decks WHERE id = ?1", [deck_id]);

    let msg = if remove_from_collection {
        format!("Deck {name} removido (e {removed} carta(s) tiradas da coleção)")
    } else {
        format!("Deck {name} removido (cartas voltaram para a coleção livre)")
    };
    log_activity(&con, "deck_disassembled", &msg);
    Json(json!({ "ok": true, "removed_from_collection": removed })).into_response()
}

#[derive(Deserialize)]
pub struct DeckCardIn {
    card_name: String,
    #[serde(default = "one")]
    quantity: i64,
    #[serde(default)]
    oracle_id: Option<String>,
    #[serde(default)]
    confirm: bool,
}

fn one() -> i64 { 1 }

/// Basic lands plus the handful of cards that print "A deck can have any number of cards named…".
fn allows_unlimited_copies(oracle_text: Option<&str>, card_name: &str) -> bool {
    const BASICS: &[&str] = &[
        "Plains", "Island", "Swamp", "Mountain", "Forest", "Wastes",
        "Snow-Covered Plains", "Snow-Covered Island", "Snow-Covered Swamp",
        "Snow-Covered Mountain", "Snow-Covered Forest",
    ];
    if BASICS.iter().any(|b| b.eq_ignore_ascii_case(card_name)) {
        return true;
    }
    oracle_text
        .map(|t| t.to_lowercase().contains("a deck can have any number of cards named"))
        .unwrap_or(false)
}

async fn add_deck_card(Path(deck_id): Path<i64>, Json(p): Json<DeckCardIn>) -> impl IntoResponse {
    let Ok(con) = open_app_db() else {
        return db_unavailable().into_response();
    };
    let Some(dname) = deck_name(&con, deck_id) else {
        return not_found("Deck não encontrado").into_response();
    };

    let oracle_text: Option<String> = open_cards_db().and_then(|cdb| {
        let sql = match p.oracle_id {
            Some(_) => format!("SELECT {CARD_COLS} FROM cards WHERE oracle_id = ?1"),
            None => format!("SELECT {CARD_COLS} FROM cards WHERE name = ?1 COLLATE NOCASE"),
        };
        let key: &dyn rusqlite::ToSql =
            p.oracle_id.as_ref().map(|o| o as &dyn rusqlite::ToSql).unwrap_or(&p.card_name);
        cdb.query_row(&sql, [key], |r| r.get::<_, Option<String>>("oracle_text")).ok().flatten()
    });

    // Singleton rule: a second copy of anything not explicitly unlimited needs confirmation.
    if !allows_unlimited_copies(oracle_text.as_deref(), &p.card_name) && !p.confirm {
        let existing_qty: i64 = con
            .query_row(
                "SELECT COALESCE(SUM(quantity),0) FROM deck_cards
                 WHERE deck_id = ?1 AND card_name = ?2 COLLATE NOCASE",
                params![deck_id, p.card_name],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if existing_qty > 0 {
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "needs_confirmation": true,
                    "card_name": p.card_name,
                    "existing_quantity": existing_qty,
                })),
            )
                .into_response();
        }
    }

    // Merge into the row for the same (name, oracle_id) pair — `IS` so two NULLs match — rather
    // than accumulating separate 1x rows when adding the same card repeatedly.
    let existing_row: Option<i64> = con
        .query_row(
            "SELECT id FROM deck_cards WHERE deck_id = ?1 AND card_name = ?2 COLLATE NOCASE
             AND is_commander = 0 AND oracle_id IS ?3",
            params![deck_id, p.card_name, p.oracle_id],
            |r| r.get(0),
        )
        .optional()
        .ok()
        .flatten();
    match existing_row {
        Some(id) => {
            let _ = con.execute(
                "UPDATE deck_cards SET quantity = quantity + ?1 WHERE id = ?2",
                params![p.quantity, id],
            );
        }
        None => {
            let _ = con.execute(
                "INSERT INTO deck_cards (deck_id, card_name, quantity, oracle_id)
                 VALUES (?1, ?2, ?3, ?4)",
                params![deck_id, p.card_name, p.quantity, p.oracle_id],
            );
        }
    }
    let _ = con.execute(
        "INSERT INTO collection (card_name, lang, quantity, allocated_deck_id, oracle_id, notes)
         VALUES (?1, 'en', ?2, ?3, ?4, 'Adicionado via app')",
        params![p.card_name, p.quantity, deck_id, p.oracle_id],
    );
    log_activity(&con, "card_added_deck", &format!("{} entrou no deck {dname}", p.card_name));
    ok().into_response()
}

async fn remove_deck_card(Path((deck_id, card_id)): Path<(i64, i64)>) -> impl IntoResponse {
    let Ok(con) = open_app_db() else {
        return db_unavailable().into_response();
    };
    let card_name: Option<String> = con
        .query_row(
            "SELECT card_name FROM deck_cards WHERE id = ?1 AND deck_id = ?2",
            params![card_id, deck_id],
            |r| r.get(0),
        )
        .optional()
        .ok()
        .flatten();
    let dname = deck_name(&con, deck_id);
    let _ = con.execute(
        "DELETE FROM deck_cards WHERE id = ?1 AND deck_id = ?2",
        params![card_id, deck_id],
    );
    // The physical copy goes back in the box, not out of existence. Leaving it allocated to a
    // deck that no longer lists it is what let phantom copies accumulate: the collection kept
    // counting cards as sleeved in decks they had already been pulled from.
    if let Some(ref cn) = card_name {
        let _ = con.execute(
            "UPDATE collection SET allocated_deck_id = NULL
             WHERE allocated_deck_id = ?1 AND card_name = ?2 COLLATE NOCASE",
            params![deck_id, cn],
        );
    }
    if let (Some(cn), Some(dn)) = (card_name, dname) {
        log_activity(&con, "card_removed_deck", &format!("{cn} saiu do deck {dn} (cópia voltou para as livres)"));
    }
    ok().into_response()
}

/// Synchronous, unlike the Python version's background job: the whole build is a few local
/// SQLite queries and finishes well inside a normal request, so there's no progress to poll.
/// The frontend's existing polling loop still works — it sees "done" on its first tick.
async fn auto_build(Json(p): Json<crate::wizard::AutoBuildIn>) -> impl IntoResponse {
    match crate::wizard::build(&p) {
        Ok(out) => {
            let mut st = LAST_BUILD.lock().unwrap();
            *st = Some(json!({ "deck_id": out.deck_id, "meta": out.meta }));
            ok().into_response()
        }
        Err(e) => {
            let mut st = LAST_BUILD.lock().unwrap();
            *st = None;
            *LAST_ERROR.lock().unwrap() = Some(e.clone());
            bad_request(&e).into_response()
        }
    }
}

pub static LAST_BUILD: std::sync::Mutex<Option<Value>> = std::sync::Mutex::new(None);
pub static LAST_ERROR: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// Mirrors deck_wizard.get_status(): the dialog polls this and navigates to result.deck_id.
pub async fn auto_build_status() -> Json<Value> {
    if let Some(result) = LAST_BUILD.lock().unwrap().clone() {
        return Json(json!({
            "state": "done", "task": "Concluído.", "percent": 100,
            "error": Value::Null, "result": result,
        }));
    }
    if let Some(err) = LAST_ERROR.lock().unwrap().clone() {
        return Json(json!({
            "state": "error", "task": Value::Null, "percent": 0,
            "error": err, "result": Value::Null,
        }));
    }
    Json(json!({
        "state": "idle", "task": Value::Null, "percent": 0,
        "error": Value::Null, "result": Value::Null,
    }))
}

/// Fetches the EDHREC page for this deck's commander(s) on demand — the "Buscar sinergia agora"
/// button. Needs network; everything downstream of it reads the cache offline.
async fn fetch_deck_synergy(Path(deck_id): Path<i64>) -> impl IntoResponse {
    let Ok(con) = open_app_db() else {
        return db_unavailable().into_response();
    };
    let commander: Option<String> = con
        .query_row("SELECT commander_name FROM decks WHERE id = ?1", [deck_id], |r| r.get(0))
        .optional()
        .ok()
        .flatten();
    let Some(commander) = commander else {
        return not_found("Deck não encontrado").into_response();
    };
    match crate::edhrec::fetch(&commander, true) {
        Ok(()) => ok().into_response(),
        Err(e) => bad_gateway(&e).into_response(),
    }
}

#[derive(Deserialize)]
pub struct ImportPreviewIn {
    text: String,
}

/// Parses + matches without writing anything. The user confirms the preview before any insert,
/// so a name the parser guessed wrong is surfaced rather than silently added.
async fn import_preview(
    Path(deck_id): Path<i64>,
    Json(p): Json<ImportPreviewIn>,
) -> impl IntoResponse {
    let Ok(con) = open_app_db() else {
        return db_unavailable().into_response();
    };
    if deck_name(&con, deck_id).is_none() {
        return not_found("Deck não encontrado").into_response();
    }
    let entries = crate::decklist::parse_text(&p.text);
    if entries.is_empty() {
        return bad_request("Não encontrei nenhuma carta reconhecível no texto colado.")
            .into_response();
    }

    let cdb = open_cards_db();
    let (mut matched, mut not_found): (Vec<Value>, Vec<Value>) = (Vec::new(), Vec::new());
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let total = entries.len();

    for (qty, name) in entries {
        let key = name.to_lowercase();
        // Repeated lines for the same card add up rather than becoming separate preview rows.
        if let Some(&i) = seen.get(&key) {
            if let Some(item) = matched.get_mut(i) {
                if let Some(q) = item.get_mut("quantity").and_then(|q| q.as_i64().map(|v| v + qty)) {
                    item["quantity"] = json!(q);
                }
            }
            continue;
        }
        let found = cdb.as_ref().and_then(|c| lookup_card_in(c, &name));
        match found {
            Some((card, how)) => {
                seen.insert(key, matched.len());
                matched.push(json!({
                    "name": card.get("name").cloned().unwrap_or(Value::Null),
                    "quantity": qty,
                    "requested_name": name,
                    "match_type": how,
                    "mana_cost": card.get("mana_cost").cloned().unwrap_or(Value::Null),
                    "type_line": card.get("type_line").cloned().unwrap_or(Value::Null),
                    "image_uri": card.get("image_uri").cloned().unwrap_or(Value::Null),
                }));
            }
            None => not_found.push(json!({ "requested_name": name, "quantity": qty })),
        }
    }
    Json(json!({ "matched": matched, "not_found": not_found, "total_lines": total })).into_response()
}

#[derive(Deserialize)]
pub struct ImportCard {
    card_name: String,
    #[serde(default = "one")]
    quantity: i64,
}

#[derive(Deserialize)]
pub struct ImportCommitIn {
    cards: Vec<ImportCard>,
    #[serde(default)]
    mode: Option<String>,
}

async fn import_commit(
    Path(deck_id): Path<i64>,
    Json(p): Json<ImportCommitIn>,
) -> impl IntoResponse {
    let Ok(con) = open_app_db() else {
        return db_unavailable().into_response();
    };
    let Some(dname) = deck_name(&con, deck_id) else {
        return not_found("Deck não encontrado").into_response();
    };
    let replace = p.mode.as_deref() == Some("replace");
    if replace {
        // Commanders are kept — replacing the list shouldn't decapitate the deck.
        let _ = con.execute(
            "DELETE FROM deck_cards WHERE deck_id = ?1 AND is_commander = 0",
            [deck_id],
        );
        // Release the physical copies those rows stood for, or they'd stay counted as sleeved in
        // this deck forever. The commander's copy stays put, matching the row that survived.
        let _ = con.execute(
            "UPDATE collection SET allocated_deck_id = NULL
             WHERE allocated_deck_id = ?1
               AND card_name COLLATE NOCASE NOT IN (
                   SELECT card_name FROM deck_cards WHERE deck_id = ?1 AND is_commander = 1
               )",
            [deck_id],
        );
    }

    let mut added = 0i64;
    for card in &p.cards {
        let name = card.card_name.trim();
        if name.is_empty() {
            continue;
        }
        let existing: Option<i64> = con
            .query_row(
                "SELECT id FROM deck_cards WHERE deck_id = ?1 AND card_name = ?2 COLLATE NOCASE
                 AND is_commander = 0",
                params![deck_id, name],
                |r| r.get(0),
            )
            .optional()
            .ok()
            .flatten();
        match existing {
            Some(id) => {
                let _ = con.execute(
                    "UPDATE deck_cards SET quantity = quantity + ?1 WHERE id = ?2",
                    params![card.quantity, id],
                );
            }
            None => {
                let _ = con.execute(
                    "INSERT INTO deck_cards (deck_id, card_name, quantity, is_commander)
                     VALUES (?1, ?2, ?3, 0)",
                    params![deck_id, name, card.quantity],
                );
            }
        }
        let _ = con.execute(
            "INSERT INTO collection (card_name, lang, quantity, allocated_deck_id, notes)
             VALUES (?1, 'en', ?2, ?3, 'Importado via decklist')",
            params![name, card.quantity, deck_id],
        );
        added += card.quantity;
    }

    let label = if replace { "substituindo cartas existentes" } else { "mesclado com o deck atual" };
    log_activity(
        &con,
        "card_added_deck",
        &format!("{added} carta(s) importada(s) para o deck {dname} ({label})"),
    );
    Json(json!({ "ok": true, "added": added })).into_response()
}

pub fn router() -> Router {
    Router::new()
        .route("/api/decks", get(list_decks).post(create_deck))
        .route("/api/decks/:id", get(get_deck).put(update_deck).delete(delete_deck))
        .route("/api/decks/:id/tags", get(deck_tags))
        .route("/api/decks/:id/cards", post(add_deck_card))
        .route("/api/decks/:deck_id/cards/:card_id", delete(remove_deck_card))
        .route("/api/decks/:id/synergy", get(deck_synergy))
        .route("/api/decks/:id/synergy/fetch", post(fetch_deck_synergy))
        .route("/api/decks/:id/import/preview", post(import_preview))
        .route("/api/decks/:id/import/commit", post(import_commit))
        // Static segments must be registered before ":id" would swallow them.
        .route("/api/decks/auto-build", post(auto_build))
        .route("/api/decks/auto-build/status", get(auto_build_status))
}
