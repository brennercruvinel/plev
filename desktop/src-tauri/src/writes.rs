//! Write endpoints — port of the POST/PUT/PATCH/DELETE half of webapp/server.py.
//!
//! Deliberately faithful to the Python behaviour, including the parts that look fussy but exist
//! for a reason: the singleton duplicate-confirmation on deck adds, merging into an existing
//! deck_cards row instead of piling up 1x rows, reconciling is_commander when a deck's
//! commander(s) change, and writing the same activity-log lines (which the Dashboard renders).

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, patch, post, put},
    Json, Router,
};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::api::CARD_COLS;
use crate::db::{open_app_db, open_cards_db};

fn oops(status: StatusCode, msg: &str) -> (StatusCode, Json<Value>) {
    (status, Json(json!({ "detail": msg })))
}

fn log_activity(con: &Connection, type_: &str, description: &str) {
    let _ = con.execute(
        "INSERT INTO activity (type, description) VALUES (?1, ?2)",
        params![type_, description],
    );
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

fn deck_name(con: &Connection, deck_id: i64) -> Option<String> {
    con.query_row("SELECT name FROM decks WHERE id = ?1", [deck_id], |r| r.get(0))
        .optional()
        .ok()
        .flatten()
}

// -------------------------------------------------------------------- decks ----

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
        return oops(StatusCode::INTERNAL_SERVER_ERROR, "Banco indisponível").into_response();
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
        return oops(StatusCode::INTERNAL_SERVER_ERROR, "Falha ao criar o deck").into_response();
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
        return oops(StatusCode::INTERNAL_SERVER_ERROR, "Banco indisponível").into_response();
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
        return oops(StatusCode::NOT_FOUND, "Deck não encontrado").into_response();
    };

    let new_c1 = p.commander_name.trim().to_string();
    if new_c1.is_empty() {
        return oops(StatusCode::BAD_REQUEST, "Comandante principal é obrigatório.").into_response();
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
    Json(json!({ "ok": true })).into_response()
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
        return oops(StatusCode::INTERNAL_SERVER_ERROR, "Banco indisponível").into_response();
    };
    let Some(name) = deck_name(&con, deck_id) else {
        return oops(StatusCode::NOT_FOUND, "Deck não encontrado").into_response();
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

// --------------------------------------------------------------- deck cards ----

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
        return oops(StatusCode::INTERNAL_SERVER_ERROR, "Banco indisponível").into_response();
    };
    let Some(dname) = deck_name(&con, deck_id) else {
        return oops(StatusCode::NOT_FOUND, "Deck não encontrado").into_response();
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
    Json(json!({ "ok": true })).into_response()
}

async fn remove_deck_card(Path((deck_id, card_id)): Path<(i64, i64)>) -> impl IntoResponse {
    let Ok(con) = open_app_db() else {
        return oops(StatusCode::INTERNAL_SERVER_ERROR, "Banco indisponível").into_response();
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
    Json(json!({ "ok": true })).into_response()
}

// --------------------------------------------------------------- collection ----

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
        return oops(StatusCode::INTERNAL_SERVER_ERROR, "Banco indisponível").into_response();
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
        return oops(StatusCode::INTERNAL_SERVER_ERROR, "Falha ao adicionar").into_response();
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
        return oops(StatusCode::INTERNAL_SERVER_ERROR, "Banco indisponível").into_response();
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
    Json(json!({ "ok": true })).into_response()
}

// -------------------------------------------------------------------- games ----

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
        return oops(StatusCode::INTERNAL_SERVER_ERROR, "Banco indisponível").into_response();
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
        return oops(StatusCode::BAD_REQUEST, "Não foi possível registrar a partida").into_response();
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

/// Synchronous, unlike the Python version's background job: the whole build is a few local
/// SQLite queries and finishes well inside a normal request, so there's no progress to poll.
/// The frontend's existing polling loop still works — it sees "done" on its first tick.
async fn auto_build(Json(p): Json<crate::wizard::AutoBuildIn>) -> impl IntoResponse {
    match crate::wizard::build(&p) {
        Ok(out) => {
            let mut st = LAST_BUILD.lock().unwrap();
            *st = Some(json!({ "deck_id": out.deck_id, "meta": out.meta }));
            Json(json!({ "ok": true })).into_response()
        }
        Err(e) => {
            let mut st = LAST_BUILD.lock().unwrap();
            *st = None;
            *LAST_ERROR.lock().unwrap() = Some(e.clone());
            oops(StatusCode::BAD_REQUEST, &e).into_response()
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

// ------------------------------------------------------- data update / synergy ----

async fn data_update_start() -> impl IntoResponse {
    if crate::update::start() {
        Json(json!({ "ok": true })).into_response()
    } else {
        oops(StatusCode::CONFLICT, "Uma atualização já está em andamento.").into_response()
    }
}

/// Fetches the EDHREC page for this deck's commander(s) on demand — the "Buscar sinergia agora"
/// button. Needs network; everything downstream of it reads the cache offline.
async fn fetch_deck_synergy(Path(deck_id): Path<i64>) -> impl IntoResponse {
    let Ok(con) = open_app_db() else {
        return oops(StatusCode::INTERNAL_SERVER_ERROR, "Banco indisponível").into_response();
    };
    let commander: Option<String> = con
        .query_row("SELECT commander_name FROM decks WHERE id = ?1", [deck_id], |r| r.get(0))
        .optional()
        .ok()
        .flatten();
    let Some(commander) = commander else {
        return oops(StatusCode::NOT_FOUND, "Deck não encontrado").into_response();
    };
    match crate::edhrec::fetch(&commander, true) {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(e) => oops(StatusCode::BAD_GATEWAY, &e).into_response(),
    }
}

// ---------------------------------------------------------- decklist import ----

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
        return oops(StatusCode::INTERNAL_SERVER_ERROR, "Banco indisponível").into_response();
    };
    if deck_name(&con, deck_id).is_none() {
        return oops(StatusCode::NOT_FOUND, "Deck não encontrado").into_response();
    }
    let entries = crate::decklist::parse_text(&p.text);
    if entries.is_empty() {
        return oops(StatusCode::BAD_REQUEST, "Não encontrei nenhuma carta reconhecível no texto colado.")
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
        let found = cdb.as_ref().and_then(|c| crate::api::lookup_card_in(c, &name));
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
        return oops(StatusCode::INTERNAL_SERVER_ERROR, "Banco indisponível").into_response();
    };
    let Some(dname) = deck_name(&con, deck_id) else {
        return oops(StatusCode::NOT_FOUND, "Deck não encontrado").into_response();
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
        .route("/api/data/update", post(data_update_start))
        .route("/api/decks/:id/synergy/fetch", post(fetch_deck_synergy))
        .route("/api/decks/:id/import/preview", post(import_preview))
        .route("/api/decks/:id/import/commit", post(import_commit))
        .route("/api/decks/auto-build", post(auto_build))
        .route("/api/decks", post(create_deck))
        .route("/api/decks/:id", put(update_deck).delete(delete_deck))
        .route("/api/decks/:id/cards", post(add_deck_card))
        .route("/api/decks/:deck_id/cards/:card_id", delete(remove_deck_card))
        .route("/api/collection", post(add_collection))
        .route("/api/collection/:id/allocate", patch(allocate_collection))
        .route("/api/games", post(add_game))
}
