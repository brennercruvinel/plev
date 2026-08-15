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
     reserved, edhrec_rank, uri, image_uri, game_changer, layout, image_uri_back";

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
    s!("uri"); s!("image_uri"); s!("layout"); s!("image_uri_back");
    i!("reserved"); i!("edhrec_rank"); i!("game_changer");
    m.insert(
        "cmc".into(),
        r.get::<_, Option<f64>>("cmc").unwrap_or(None).map_or(Value::Null, |v| json!(v)),
    );
    // Art is served from the local cache (see images.rs) with a redirect fallback, so the grids
    // keep working offline. Rewritten here, at the single point every card object is built.
    let mut v = Value::Object(m);
    crate::images::rewrite_card(&mut v);
    Ok(v)
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
            OR c.oracle_id IN (SELECT oracle_id FROM names_localized WHERE printed_name_folded LIKE ?1)
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

    // Front face of a two-faced card. Exporters and older imports write "Murderous Rider" where
    // the index carries "Murderous Rider // Swift End"; this is a full match on the part before
    // the separator, not a substring guess, so it belongs with the exact matches rather than in
    // the LIKE fallback below — which would return it as "aproximada" and leave callers that
    // only trust exact matches (canonical_name) unable to place a card they can clearly name.
    let front = format!("{folded} // %");
    if let Some(v) = one(
        &format!("SELECT {CARD_COLS} FROM cards WHERE name_folded LIKE ?1 ORDER BY length(name) LIMIT 1"),
        &[&front],
    ) {
        return Some((v, "exata (face frontal)".into()));
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
        "SELECT oracle_id FROM names_localized WHERE printed_name = ?1 COLLATE NOCASE
             GROUP BY oracle_id ORDER BY MIN(lang_rank)",
        name,
    );
    if ids.is_empty() {
        ids = pt_ids(
            "SELECT oracle_id FROM names_localized WHERE printed_name_folded = ?1
                 GROUP BY oracle_id ORDER BY MIN(lang_rank)",
            &folded,
        );
    }
    if ids.len() == 1 {
        if let Some(v) = one(
            &format!("SELECT {CARD_COLS} FROM cards WHERE oracle_id = ?1"),
            &[&ids[0]],
        ) {
            return Some((v, "exata (nome traduzido)".into()));
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

/// The index's own spelling of a card, for anything that stores a name as data.
///
/// `collection.card_name` and `deck_cards.card_name` are free text, and the write paths disagree
/// about which spelling to use: an Adventure card is "Murderous Rider" from one and "Murderous
/// Rider // Swift End" from another, and a name typed without accents stays that way. Two rows
/// for one card then look like two different cards to anything grouping by the string.
///
/// Returns None for approximate matches — renaming a row on a fuzzy hit would turn a typo into
/// a confident claim about a card the user never entered.
pub fn canonical_name(cdb: &Connection, raw: &str) -> Option<String> {
    let (card, how) = lookup_card_in(cdb, raw)?;
    if !how.starts_with("exata") {
        return None;
    }
    let name = card.get("name")?.as_str()?.to_string();
    (name != raw).then_some(name)
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
            "SELECT printed_name, set_code, lang FROM names_localized WHERE oracle_id = ?1
             GROUP BY lang HAVING MIN(lang_rank) ORDER BY lang_rank",
        )?;
        let rows = stmt.query_map([&oid], |r| {
            Ok(json!({
                "printed_name": r.get::<_, Option<String>>(0)?,
                "set_code": r.get::<_, Option<String>>(1)?,
                "lang": r.get::<_, Option<String>>(2)?,
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

/// Every distinct artwork of a card, newest printing first.
///
/// Split, adventure and other single-faced "//" cards have one art per printing like anything
/// else; two-sided cards carry both faces here, so the flip control keeps working per printing.
async fn card_printings(Path(name): Path<String>) -> Json<Value> {
    let Some(cdb) = open_cards_db() else { return Json(json!([])) };
    let Some((card, _)) = lookup_card_in(&cdb, &name) else { return Json(json!([])) };
    let Some(oid) = card.get("oracle_id").and_then(|v| v.as_str()) else {
        return Json(json!([]));
    };
    let out = (|| -> rusqlite::Result<Vec<Value>> {
        let mut stmt = cdb.prepare(
            "SELECT p.set_code, p.collector_number, p.artist, p.image_uri, p.image_uri_back,
                    p.released_at, s.name
             FROM printings p LEFT JOIN sets s ON s.code = p.set_code
             WHERE p.oracle_id = ?1
             ORDER BY p.released_at DESC",
        )?;
        let rows = stmt.query_map([oid], |r| {
            let mut v = json!({
                "set_code": r.get::<_, Option<String>>(0)?,
                "collector_number": r.get::<_, Option<String>>(1)?,
                "artist": r.get::<_, Option<String>>(2)?,
                "image_uri": r.get::<_, Option<String>>(3)?,
                "image_uri_back": r.get::<_, Option<String>>(4)?,
                "released_at": r.get::<_, Option<String>>(5)?,
                "set_name": r.get::<_, Option<String>>(6)?,
            });
            crate::images::rewrite_card(&mut v);
            Ok(v)
        })?;
        rows.collect()
    })()
    .unwrap_or_default();
    Json(Value::Array(out))
}

/// The image of one specific printing, so a copy recorded as being from a given set shows that
/// set's art rather than whichever printing the index happens to treat as canonical.
pub fn printing_image(cdb: &Connection, oracle_id: &str, set_code: &str) -> Option<(String, Option<String>)> {
    use rusqlite::OptionalExtension;
    cdb.query_row(
        "SELECT image_uri, image_uri_back FROM printings
         WHERE oracle_id = ?1 AND set_code = ?2 COLLATE NOCASE
         ORDER BY released_at DESC LIMIT 1",
        rusqlite::params![oracle_id, set_code],
        |r| Ok((r.get::<_, Option<String>>(0)?, r.get::<_, Option<String>>(1)?)),
    )
    .optional()
    .ok()
    .flatten()
    .and_then(|(f, b)| f.map(|f| (crate::images::local_url(&f), b.map(|b| crate::images::local_url(&b)))))
}

#[derive(Deserialize)]
pub struct SetsQuery {
    #[serde(default)]
    q: String,
    #[serde(default = "sets_limit")]
    limit: i64,
    /// When present, only sets this card was actually printed in.
    #[serde(default)]
    card: Option<String>,
}
fn sets_limit() -> i64 { 12 }

/// Set autocomplete. Matches the name accent-folded *and* the code, because for recent sets the
/// three-letter code is what people remember, and Scryfall does not localise set names — so
/// matching only a translated name would find nothing.
///
/// Ordered newest first: a set you are cataloguing is far more likely to be recent than to be
/// from 1995, and an alphabetical list buries every current set.
async fn list_sets(Query(p): Query<SetsQuery>) -> Json<Value> {
    let Some(cdb) = open_cards_db() else { return Json(json!([])) };

    // Scoped to one card when the form already knows which card is being recorded. Offering the
    // full 1,047 sets lets someone pick a set the card was never printed in — which then resolves
    // to no artwork at all, silently. Scoping makes the field answer the question actually being
    // asked: "which printing of this card do I have?"
    if let Some(card) = p.card.as_deref().filter(|c| !c.trim().is_empty()) {
        let oid = lookup_card_in(&cdb, card)
            .and_then(|(c, _)| c.get("oracle_id").and_then(|v| v.as_str()).map(String::from));
        if let Some(oid) = oid {
            let like = format!("%{}%", fold_text(&p.q));
            let code_like = format!("{}%", p.q.to_lowercase());
            let out = (|| -> rusqlite::Result<Vec<Value>> {
                let mut stmt = cdb.prepare(
                    "SELECT DISTINCT p.set_code, s.name, s.released_at, s.set_type, s.cards
                     FROM printings p LEFT JOIN sets s ON s.code = p.set_code
                     WHERE p.oracle_id = ?1
                       AND (?2 = '' OR s.name_folded LIKE ?3 OR p.set_code LIKE ?4)
                     ORDER BY s.released_at DESC
                     LIMIT ?5",
                )?;
                let rows = stmt.query_map(
                    rusqlite::params![oid, p.q, like, code_like, p.limit.max(40)],
                    |r| {
                        Ok(json!({
                            "code": r.get::<_, Option<String>>(0)?,
                            "name": r.get::<_, Option<String>>(1)?,
                            "released_at": r.get::<_, Option<String>>(2)?,
                            "set_type": r.get::<_, Option<String>>(3)?,
                            "cards": r.get::<_, Option<i64>>(4)?,
                        }))
                    },
                )?;
                rows.collect()
            })()
            .unwrap_or_default();
            return Json(Value::Array(out));
        }
    }

    let like = format!("%{}%", fold_text(&p.q));
    let code_like = format!("{}%", p.q.to_lowercase());
    let out = (|| -> rusqlite::Result<Vec<Value>> {
        // Ranked, not just filtered. A raw "newest first" buried Dominaria United under its own
        // token, art-series and promo sets — the ones nobody catalogues a card into. Real sets
        // come first, then a name that *starts* with what was typed, then recency.
        let starts = format!("{}%", fold_text(&p.q));
        let mut stmt = cdb.prepare(
            "SELECT code, name, released_at, set_type, cards FROM sets
             WHERE name_folded LIKE ?1 OR code LIKE ?2
             ORDER BY
               (code = ?4) DESC,
               CASE set_type
                 WHEN 'expansion' THEN 0 WHEN 'core' THEN 0
                 WHEN 'masters' THEN 1 WHEN 'draft_innovation' THEN 1 WHEN 'commander' THEN 1
                 WHEN 'starter' THEN 2 WHEN 'duel_deck' THEN 2 WHEN 'box' THEN 2
                 WHEN 'token' THEN 8 WHEN 'memorabilia' THEN 9 WHEN 'minigame' THEN 9
                 WHEN 'promo' THEN 7 WHEN 'alchemy' THEN 7
                 ELSE 4
               END ASC,
               (name_folded LIKE ?5) DESC,
               released_at DESC
             LIMIT ?3",
        )?;
        let rows = stmt.query_map(
            rusqlite::params![like, code_like, p.limit, p.q.to_lowercase(), starts],
            |r| {
            Ok(json!({
                "code": r.get::<_, String>(0)?,
                "name": r.get::<_, Option<String>>(1)?,
                "released_at": r.get::<_, Option<String>>(2)?,
                "set_type": r.get::<_, Option<String>>(3)?,
                "cards": r.get::<_, Option<i64>>(4)?,
            }))
            },
        )?;
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
        .route("/api/cards/:name/printings", get(card_printings))
        .route("/api/sets", get(list_sets))
}
