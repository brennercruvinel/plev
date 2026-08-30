//! Cards the user wants but doesn't own.
//!
//! Shaped like the collection on purpose - same grouped-by-name payload, same enrichment - so
//! the wishlist screen can reuse the collection's grid, filters and card modal without a parallel
//! set of components. What it deliberately does *not* have is `allocated_deck_id`: a wishlist
//! entry is not cardboard, and nothing that counts what you own may see it.

use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::db::{fold_text, log_activity, open_app_db, open_cards_db};
use crate::error::{Error, Result};
use crate::ops::cards;
use crate::types::ImageRef;

/// One stored wishlist row, kept individually inside its group so the UI can
/// list and delete the copies one at a time.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WishlistEntry {
    pub id: i64,
    pub quantity: i64,
    pub lang: String,
    pub set_code: Option<String>,
    pub artist: Option<String>,
    pub notes: Option<String>,
}

/// Every wishlist row for one card name, plus what the index knows about it.
///
/// The enriched fields are exactly the ones the grid, the chip filters and the
/// price badge read; they stay `None` when the card index is missing or cannot
/// place the name.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WishlistGroup {
    pub card_name: String,
    pub total_quantity: i64,
    pub entries: Vec<WishlistEntry>,
    pub type_line: Option<String>,
    pub mana_cost: Option<String>,
    pub image: Option<ImageRef>,
    pub image_back: Option<ImageRef>,
    pub colors: Option<String>,
    pub rarity: Option<String>,
    pub price_usd: Option<String>,
    pub cmc: Option<f64>,
    pub layout: Option<String>,
}

/// The wishlist summary line: how many copies, how many names, what it costs.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WishlistTotals {
    pub total_units: i64,
    pub distinct_cards: i64,
    pub price_usd: f64,
}

/// What is left of a card after one unit of it is removed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WishlistRemoval {
    pub card_name: String,
    pub remaining: i64,
}

/// The wishlist, grouped by card name.
///
/// Answers an empty list rather than an error when app.db is missing: this
/// screen is reachable before the first data download.
pub fn list_wishlist(q: &str) -> Vec<WishlistGroup> {
    let Ok(con) = open_app_db() else {
        return Vec::new();
    };

    // Same trick as the collection: resolve the search term through the card index first, so a
    // card saved in English is still found by typing its Portuguese name.
    let mut extra_names: Vec<String> = Vec::new();
    if !q.is_empty()
        && let Some(cdb) = open_cards_db()
    {
        let like = format!("%{}%", fold_text(q));
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

    let mut sql = String::from(
        "SELECT id, card_name, quantity, lang, set_code, artist, notes FROM wishlist WHERE 1=1",
    );
    if !q.is_empty() {
        if extra_names.is_empty() {
            sql.push_str(" AND card_name LIKE ?1 COLLATE NOCASE");
        } else {
            let ph = std::iter::repeat_n("?", extra_names.len())
                .collect::<Vec<_>>()
                .join(",");
            sql.push_str(&format!(
                " AND (card_name LIKE ?1 COLLATE NOCASE OR card_name COLLATE NOCASE IN ({ph}))"
            ));
        }
    }
    sql.push_str(" ORDER BY card_name, id");

    type Row = (
        i64,
        String,
        i64,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    );
    let read = |stmt: &mut rusqlite::Statement, params: &[&dyn rusqlite::ToSql]| -> Vec<Row> {
        stmt.query_map(params, |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
            ))
        })
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default()
    };
    let rows: Vec<Row> = match con.prepare(&sql) {
        Ok(mut stmt) => {
            if q.is_empty() {
                read(&mut stmt, &[])
            } else {
                let like = format!("%{q}%");
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
    let mut grouped: HashMap<String, (i64, Vec<WishlistEntry>)> = HashMap::new();
    for (id, card_name, quantity, lang, set_code, artist, notes) in rows {
        let e = grouped.entry(card_name.clone()).or_insert_with(|| {
            order.push(card_name.clone());
            (0, Vec::new())
        });
        e.0 += quantity;
        e.1.push(WishlistEntry {
            id,
            quantity,
            lang,
            set_code,
            artist,
            notes,
        });
    }

    let cdb = open_cards_db();
    let mut out = Vec::new();
    for card_name in &order {
        let (total, entries) = grouped.get(card_name).unwrap();
        let mut group = WishlistGroup {
            card_name: card_name.clone(),
            total_quantity: *total,
            entries: entries.clone(),
            ..WishlistGroup::default()
        };
        if let Some(ref c) = cdb
            && let Some((card, _)) = cards::lookup_card_in(c, card_name)
        {
            group.type_line = card.type_line;
            group.mana_cost = card.mana_cost;
            group.image = card.image;
            group.image_back = card.image_back;
            group.colors = card.colors;
            group.rarity = card.rarity;
            group.price_usd = card.price_usd;
            group.cmc = card.cmc;
            group.layout = card.layout;
        }
        out.push(group);
    }
    out
}

/// Copies, distinct names and what the whole list would cost.
///
/// Zeroes rather than an error without app.db, for the same reason
/// `list_wishlist` answers an empty list.
pub fn wishlist_total() -> WishlistTotals {
    let Ok(con) = open_app_db() else {
        return WishlistTotals::default();
    };
    let total: i64 = con
        .query_row("SELECT COALESCE(SUM(quantity), 0) FROM wishlist", [], |r| {
            r.get(0)
        })
        .unwrap_or(0);
    let distinct: i64 = con
        .query_row("SELECT COUNT(DISTINCT card_name) FROM wishlist", [], |r| {
            r.get(0)
        })
        .unwrap_or(0);
    // What the whole list would cost at Scryfall's current prices, which is the number anyone
    // keeping a buy-list actually wants.
    let mut usd = 0.0f64;
    if let Some(cdb) = open_cards_db()
        && let Ok(mut stmt) = con.prepare("SELECT card_name, quantity FROM wishlist")
        && let Ok(rows) = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
    {
        for (name, qty) in rows.flatten() {
            if let Some((card, _)) = cards::lookup_card_in(&cdb, &name)
                && let Some(p) = card.price_usd.as_deref()
            {
                usd += p.parse::<f64>().unwrap_or(0.0) * qty as f64;
            }
        }
    }
    WishlistTotals {
        total_units: total,
        distinct_cards: distinct,
        price_usd: (usd * 100.0).round() / 100.0,
    }
}

/// A card to put on the wishlist. The defaults are the ones the add dialog
/// relies on: one copy, in English.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WishlistIn {
    pub card_name: String,
    #[serde(default)]
    pub set_code: Option<String>,
    #[serde(default)]
    pub artist: Option<String>,
    #[serde(default = "lang_en")]
    pub lang: String,
    #[serde(default = "one")]
    pub quantity: i64,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub oracle_id: Option<String>,
}
fn lang_en() -> String {
    "en".into()
}
fn one() -> i64 {
    1
}

/// Adds one wishlist row, answering the id it was stored under.
pub fn add_wishlist(p: WishlistIn) -> Result<i64> {
    let Ok(con) = open_app_db() else {
        return Err(Error::db_unavailable());
    };
    // Same canonical spelling as everywhere else, so a wishlist entry and the collection row it
    // eventually becomes are recognisably the same card.
    let card_name = open_cards_db()
        .and_then(|cdb| cards::canonical_name(&cdb, &p.card_name))
        .unwrap_or_else(|| p.card_name.clone());

    if con
        .execute(
            "INSERT INTO wishlist (card_name, set_code, artist, lang, quantity, notes, oracle_id)
             VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![
                card_name,
                p.set_code,
                p.artist,
                p.lang,
                p.quantity,
                p.notes,
                p.oracle_id
            ],
        )
        .is_err()
    {
        return Err(Error::Internal("Falha ao adicionar à wishlist".into()));
    }
    let qty_label = if p.quantity != 1 {
        format!("{}x ", p.quantity)
    } else {
        String::new()
    };
    log_activity(
        &con,
        "wishlist_add",
        &format!("{qty_label}{card_name} entrou na wishlist"),
    );
    Ok(con.last_insert_rowid())
}

/// Removes one unit of a wishlist entry.
pub fn delete_wishlist_entry(entry_id: i64) -> Result<WishlistRemoval> {
    let Ok(con) = open_app_db() else {
        return Err(Error::db_unavailable());
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
        return Err(Error::NotFound("Item não encontrado".into()));
    };

    // One unit at a time, matching how the collection deletes copies.
    if quantity > 1 {
        let _ = con.execute(
            "UPDATE wishlist SET quantity = quantity - 1 WHERE id = ?1",
            [entry_id],
        );
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
    log_activity(
        &con,
        "wishlist_remove",
        &format!("{card_name} saiu da wishlist"),
    );
    Ok(WishlistRemoval {
        card_name,
        remaining,
    })
}

/// Moves a wishlist entry into the collection - the whole point of keeping the list.
pub fn acquire_wishlist_entry(entry_id: i64) -> Result<()> {
    let Ok(con) = open_app_db() else {
        return Err(Error::db_unavailable());
    };
    type Row = (
        String,
        Option<String>,
        Option<String>,
        String,
        i64,
        Option<String>,
        Option<String>,
    );
    let row: Option<Row> = con
        .query_row(
            "SELECT card_name, set_code, artist, lang, quantity, notes, oracle_id
             FROM wishlist WHERE id = ?1",
            [entry_id],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                ))
            },
        )
        .optional()
        .ok()
        .flatten();
    let Some((card_name, set_code, artist, lang, quantity, notes, oracle_id)) = row else {
        return Err(Error::NotFound("Item não encontrado".into()));
    };

    let _ = con.execute(
        "INSERT INTO collection (card_name, set_code, artist, lang, quantity, notes, oracle_id)
         VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![
            card_name, set_code, artist, lang, quantity, notes, oracle_id
        ],
    );
    let _ = con.execute("DELETE FROM wishlist WHERE id = ?1", [entry_id]);
    log_activity(
        &con,
        "wishlist_acquired",
        &format!("{card_name} saiu da wishlist e entrou na coleção"),
    );
    Ok(())
}
