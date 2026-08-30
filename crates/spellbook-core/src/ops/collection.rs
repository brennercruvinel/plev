//! The collection: every physical copy the user owns.
//!
//! One row is one card (or a stack of identical ones via `quantity`). A card sleeved in two
//! decks is two rows, because it is two cards - `allocated_deck_id` says which deck holds each,
//! and NULL means it is free in the box. Counts throughout the app are copies, never distinct
//! names; collapsing the two is what made the home screen undercount.

use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::db::{deck_name, fold_text, log_activity, open_app_db, open_cards_db};
use crate::error::{Error, Result};
use crate::ops::cards::{canonical_name, lookup_card_in, printing_image};
use crate::types::ImageRef;

/// serde default for `quantity`: adding a card means one copy unless told otherwise.
fn one() -> i64 {
    1
}

fn lang_en() -> String {
    "en".into()
}

/// One card in the collection list: every copy of that name, grouped, plus what the index
/// knows about the card so the grid can draw a tile without a second query.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CollectionEntry {
    pub card_name: String,
    /// Copies, summed across the rows below - not a number of rows.
    pub total_quantity: i64,
    /// Where each row of copies sits. `deck_name` is "Livre" for a free copy.
    pub decks: Vec<CollectionCopy>,
    pub entry_ids: Vec<i64>,
    pub first_added: Option<String>,
    pub last_added: Option<String>,
    pub type_line: Option<String>,
    pub mana_cost: Option<String>,
    pub colors: Option<String>,
    pub rarity: Option<String>,
    pub price_usd: Option<String>,
    pub cmc: Option<f64>,
    pub image: Option<ImageRef>,
    pub image_back: Option<ImageRef>,
}

/// One stored row inside a grouped [`CollectionEntry`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionCopy {
    pub deck_id: Option<i64>,
    /// The deck holding these copies, or "Livre" when they are free in the box.
    pub deck_name: String,
    pub quantity: i64,
    pub lang: String,
    pub set_code: Option<String>,
    pub created_at: Option<String>,
}

/// The home screen's collection counters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionTotals {
    pub total_units: i64,
    pub distinct_cards: i64,
    pub free_units: i64,
    pub allocated_units: i64,
}

/// Every physical copy of one card: the totals, a line per deck, and the rows themselves.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CardCopies {
    pub total: i64,
    pub free: i64,
    pub decks: Vec<DeckCopies>,
    pub entries: Vec<CopyEntry>,
}

/// How many copies of one card a single deck holds.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeckCopies {
    pub deck_id: i64,
    pub deck_name: String,
    pub copies: i64,
}

/// One stored copy, individually addressable so the UI can edit or delete just it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CopyEntry {
    pub id: i64,
    pub quantity: i64,
    pub deck_id: Option<i64>,
    pub deck_name: Option<String>,
    pub set_code: Option<String>,
    pub lang: String,
    pub artist: Option<String>,
    pub notes: Option<String>,
    pub image: Option<ImageRef>,
    pub image_back: Option<ImageRef>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionIn {
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
    pub deck_id: Option<i64>,
    #[serde(default)]
    pub oracle_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionEditIn {
    #[serde(default)]
    pub set_code: Option<String>,
    #[serde(default)]
    pub artist: Option<String>,
    #[serde(default)]
    pub lang: Option<String>,
    #[serde(default)]
    pub quantity: Option<i64>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// What removing one copy left behind.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CopyRemoved {
    /// Copies of this card still in the collection, anywhere. Zero means the card itself is
    /// gone, which the UI reports differently from thinning a playset.
    pub remaining: i64,
    pub card_name: String,
}

/// One row of the collection query, before grouping.
struct ListRow {
    id: i64,
    card_name: String,
    quantity: i64,
    lang: String,
    set_code: Option<String>,
    deck_id: Option<i64>,
    deck_name: Option<String>,
    created_at: Option<String>,
}

/// One row of the copies query.
struct CopyRow {
    id: i64,
    card_name: String,
    deck_id: Option<i64>,
    deck_name: Option<String>,
    quantity: i64,
    set_code: Option<String>,
    notes: Option<String>,
    lang: String,
    artist: Option<String>,
}

/// The card's copies gathered under one name, before the index fills in the rest.
#[derive(Default)]
struct Group {
    total_quantity: i64,
    decks: Vec<CollectionCopy>,
    entry_ids: Vec<i64>,
    first_added: Option<String>,
    last_added: Option<String>,
}

/// What the card index adds to a name the collection stores.
#[derive(Clone)]
struct Enrichment {
    type_line: Option<String>,
    mana_cost: Option<String>,
    image: Option<ImageRef>,
    colors: Option<String>,
    rarity: Option<String>,
    price_usd: Option<String>,
    cmc: Option<f64>,
    image_back: Option<ImageRef>,
}

impl Enrichment {
    /// Reads the eight index columns starting at `base`, in the order both queries select them.
    fn from_row(r: &rusqlite::Row, base: usize) -> rusqlite::Result<Self> {
        Ok(Self {
            type_line: r.get(base)?,
            mana_cost: r.get(base + 1)?,
            image: ImageRef::from_url(r.get::<_, Option<String>>(base + 2)?.as_deref()),
            colors: r.get(base + 3)?,
            rarity: r.get(base + 4)?,
            price_usd: r.get(base + 5)?,
            cmc: r.get(base + 6)?,
            image_back: ImageRef::from_url(r.get::<_, Option<String>>(base + 7)?.as_deref()),
        })
    }
}

impl CollectionEntry {
    fn apply(&mut self, e: Enrichment) {
        self.type_line = e.type_line;
        self.mana_cost = e.mana_cost;
        self.image = e.image;
        self.colors = e.colors;
        self.rarity = e.rarity;
        self.price_usd = e.price_usd;
        self.cmc = e.cmc;
        self.image_back = e.image_back;
    }
}

/// The collection, one item per card name. `status` is "free", "allocated" or anything else
/// for all of them; `q` searches the stored names.
///
/// Answers an empty list rather than an error when app.db is missing: the screen is reachable
/// before the first data download.
pub fn list_collection(status: &str, q: &str) -> Vec<CollectionEntry> {
    let Ok(con) = open_app_db() else {
        return Vec::new();
    };

    let mut sql = String::from(
        "SELECT collection.id, collection.card_name, collection.quantity, collection.lang,
                collection.set_code, collection.allocated_deck_id, decks.name,
                collection.created_at
         FROM collection LEFT JOIN decks ON decks.id = collection.allocated_deck_id WHERE 1=1",
    );
    match status {
        "free" => sql.push_str(" AND collection.allocated_deck_id IS NULL"),
        "allocated" => sql.push_str(" AND collection.allocated_deck_id IS NOT NULL"),
        _ => {}
    }
    // The collection stores whichever name the card was entered under - usually English. Matching
    // only that string is why searching "Anel Solar" found nothing while "Sol Ring" did. The
    // card index knows every printed name, so the search resolves the term through it first and
    // matches the resulting English names too. Accent-folded, so "cemiterio" finds "Cemitério".
    let mut extra_names: Vec<String> = Vec::new();
    if !q.is_empty() {
        if let Some(cdb) = open_cards_db() {
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
        if extra_names.is_empty() {
            sql.push_str(" AND collection.card_name LIKE ?1 COLLATE NOCASE");
        } else {
            let ph = std::iter::repeat_n("?", extra_names.len())
                .collect::<Vec<_>>()
                .join(",");
            sql.push_str(&format!(
                " AND (collection.card_name LIKE ?1 COLLATE NOCASE
                       OR collection.card_name COLLATE NOCASE IN ({ph}))"
            ));
        }
    }
    sql.push_str(" ORDER BY collection.card_name");

    let read = |stmt: &mut rusqlite::Statement, params: &[&dyn rusqlite::ToSql]| -> Vec<ListRow> {
        stmt.query_map(params, |r| {
            Ok(ListRow {
                id: r.get(0)?,
                card_name: r.get(1)?,
                quantity: r.get(2)?,
                lang: r.get(3)?,
                set_code: r.get(4)?,
                deck_id: r.get(5)?,
                deck_name: r.get(6)?,
                created_at: r.get(7)?,
            })
        })
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default()
    };
    let rows: Vec<ListRow> = match con.prepare(&sql) {
        Ok(mut stmt) => {
            if q.is_empty() {
                read(&mut stmt, &[])
            } else {
                let like = format!("%{q}%");
                let mut params: Vec<&dyn rusqlite::ToSql> = vec![&like];
                for n in &extra_names {
                    params.push(n);
                }
                read(&mut stmt, params.as_slice())
            }
        }
        Err(_) => Vec::new(),
    };

    // Group by card name, summing units - same shape the Python endpoint returned.
    let mut order: Vec<String> = Vec::new();
    // Also the oldest and newest acquisition dates across a card's copies, so the collection can
    // be ordered by when things arrived. Grouping by name means a card has several dates; the
    // extremes are what "added first" and "added last" actually mean for the group.
    let mut grouped: HashMap<String, Group> = HashMap::new();
    for row in rows {
        let e = grouped.entry(row.card_name.clone()).or_insert_with(|| {
            order.push(row.card_name.clone());
            Group::default()
        });
        e.total_quantity += row.quantity;
        e.entry_ids.push(row.id);
        e.decks.push(CollectionCopy {
            deck_id: row.deck_id,
            deck_name: row.deck_name.unwrap_or_else(|| "Livre".into()),
            quantity: row.quantity,
            lang: row.lang,
            set_code: row.set_code,
            created_at: row.created_at.clone(),
        });
        if let Some(ts) = row.created_at {
            if e.first_added.as_ref().is_none_or(|cur| ts < *cur) {
                e.first_added = Some(ts.clone());
            }
            if e.last_added.as_ref().is_none_or(|cur| ts > *cur) {
                e.last_added = Some(ts);
            }
        }
    }

    // Batched enrichment (one IN(...) per chunk) instead of a query per card name.
    let cdb = open_cards_db();
    let mut by_name: HashMap<String, Enrichment> = HashMap::new();
    if let Some(ref c) = cdb {
        for chunk in order.chunks(400) {
            let ph = std::iter::repeat_n("?", chunk.len())
                .collect::<Vec<_>>()
                .join(",");
            let sql = format!(
                "SELECT name, type_line, mana_cost, image_uri, colors, rarity, price_usd, cmc, image_uri_back
                 FROM cards WHERE name COLLATE NOCASE IN ({ph})"
            );
            if let Ok(mut stmt) = c.prepare(&sql) {
                let params: Vec<&dyn rusqlite::ToSql> =
                    chunk.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
                if let Ok(rows) = stmt.query_map(params.as_slice(), |r| {
                    let name: String = r.get(0)?;
                    Ok((name, Enrichment::from_row(r, 1)?))
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
        let g = grouped.get(card_name).unwrap();
        let mut entry = CollectionEntry {
            card_name: card_name.clone(),
            total_quantity: g.total_quantity,
            decks: g.decks.clone(),
            entry_ids: g.entry_ids.clone(),
            first_added: g.first_added.clone(),
            last_added: g.last_added.clone(),
            ..CollectionEntry::default()
        };
        let enrich = by_name.get(&card_name.to_lowercase()).cloned().or_else(|| {
            // Prefix fallback for hand-entered names that don't match exactly.
            cdb.as_ref().and_then(|c| {
                c.prepare(
                    "SELECT type_line, mana_cost, image_uri, colors, rarity, price_usd, cmc, image_uri_back
                     FROM cards WHERE name LIKE ?1 COLLATE NOCASE LIMIT 1",
                )
                .ok()
                .and_then(|mut s| {
                    s.query_row([format!("{card_name}%")], |r| Enrichment::from_row(r, 0))
                        .ok()
                })
            })
        });
        if let Some(e) = enrich {
            entry.apply(e);
        }

        // The tile shows the art of the printing you actually own, not the index's canonical one.
        // Without this a Secret Lair copy displayed the regular-set illustration - the card was
        // right and the picture wasn't, which is the one thing a collection view must get right.
        // With copies from several sets the first is used; the card modal lists them all.
        if let Some(ref c) = cdb {
            let owned_set = entry
                .decks
                .iter()
                .filter_map(|d| d.set_code.as_deref())
                .find(|s| !s.is_empty())
                .map(String::from);
            // The columns selected above carry no oracle_id, so the id always comes from a
            // lookup by name, exactly as it did when these were merged JSON objects.
            let oracle_id = lookup_card_in(c, card_name).and_then(|(card, _)| card.oracle_id);
            if let (Some(set_code), Some(oid)) = (owned_set, oracle_id)
                && let Some((front, back)) = printing_image(c, &oid, &set_code)
            {
                entry.image = Some(front);
                if let Some(b) = back {
                    entry.image_back = Some(b);
                }
            }
        }
        out.push(entry);
    }
    out
}

/// The home screen's counters. Zeroes rather than an error when app.db is missing, for the
/// same reason [`list_collection`] answers an empty list.
pub fn collection_total() -> CollectionTotals {
    let Ok(con) = open_app_db() else {
        return CollectionTotals::default();
    };
    let total: i64 = con
        .query_row(
            "SELECT COALESCE(SUM(quantity), 0) FROM collection",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let distinct: i64 = con
        .query_row(
            "SELECT COUNT(DISTINCT card_name) FROM collection",
            [],
            |r| r.get(0),
        )
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
    CollectionTotals {
        total_units: total,
        distinct_cards: distinct,
        free_units: free,
        allocated_units: total - free,
    }
}

/// Every physical copy of one exact card name: how many are free and which decks hold the rest.
/// Exact match (folded), unlike [`list_collection`]'s substring search - the card modal is asking
/// about one card, and "Swamp" must not pick up "Swamp Mosquito".
pub fn card_copies(name: &str) -> CardCopies {
    let Ok(con) = open_app_db() else {
        return CardCopies::default();
    };
    // fold_text() is a Rust function, not a registered SQLite one, so the accent-insensitive
    // comparison happens here rather than in the WHERE clause. The collection is a few hundred
    // rows - small enough that scanning it costs less than keeping a folded column in sync.
    let wanted = fold_text(name);
    let rows: Vec<CopyRow> = (|| -> rusqlite::Result<Vec<CopyRow>> {
        let mut stmt = con.prepare(
            "SELECT collection.id, collection.card_name, collection.allocated_deck_id,
                    decks.name, collection.quantity, collection.set_code, collection.notes,
                    collection.lang, collection.artist
             FROM collection LEFT JOIN decks ON decks.id = collection.allocated_deck_id
             ORDER BY collection.allocated_deck_id IS NOT NULL, collection.id",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(CopyRow {
                id: r.get(0)?,
                card_name: r.get(1)?,
                deck_id: r.get(2)?,
                deck_name: r.get(3)?,
                quantity: r.get(4)?,
                set_code: r.get(5)?,
                notes: r.get(6)?,
                lang: r.get(7)?,
                artist: r.get(8)?,
            })
        })?;
        rows.collect()
    })()
    .unwrap_or_default();

    let mut free = 0i64;
    let mut by_deck: HashMap<i64, (String, i64)> = HashMap::new();
    // Individual rows too, so the UI can offer a delete per copy rather than only a total.
    let mut entries: Vec<CopyEntry> = Vec::new();
    for row in rows {
        if fold_text(&row.card_name) != wanted {
            continue;
        }
        // Each copy shows the art of the set it was recorded from - the whole point of noting
        // the set on a copy you own.
        let art = row.set_code.as_deref().and_then(|sc| {
            let cdb = open_cards_db()?;
            let oid = lookup_card_in(&cdb, &row.card_name)?.0.oracle_id?;
            printing_image(&cdb, &oid, sc)
        });
        entries.push(CopyEntry {
            id: row.id,
            quantity: row.quantity,
            deck_id: row.deck_id,
            deck_name: row.deck_name.clone(),
            set_code: row.set_code,
            lang: row.lang,
            artist: row.artist,
            notes: row.notes,
            image: art.as_ref().map(|(f, _)| f.clone()),
            image_back: art.as_ref().and_then(|(_, b)| b.clone()),
        });
        match row.deck_id {
            None => free += row.quantity,
            Some(id) => {
                let e = by_deck
                    .entry(id)
                    .or_insert_with(|| (row.deck_name.unwrap_or_default(), 0));
                e.1 += row.quantity;
            }
        }
    }
    let mut decks: Vec<DeckCopies> = by_deck
        .into_iter()
        .map(|(id, (name, qty))| DeckCopies {
            deck_id: id,
            deck_name: name,
            copies: qty,
        })
        .collect();
    decks.sort_by_key(|d| d.deck_name.to_lowercase());
    let in_decks: i64 = decks.iter().map(|d| d.copies).sum();

    CardCopies {
        total: free + in_decks,
        free,
        decks,
        entries,
    }
}

/// Records one physical copy (or a stack of them), optionally straight into a deck.
/// Answers the id of the row it created.
pub fn add_collection(p: CollectionIn) -> Result<i64> {
    let Ok(con) = open_app_db() else {
        return Err(Error::db_unavailable());
    };
    // Store the index's spelling, not whatever reached this function. Without it the same card
    // accumulates rows under "Murderous Rider" and "Murderous Rider // Swift End", and every
    // count that groups by name sees two cards.
    let card_name = open_cards_db()
        .and_then(|cdb| canonical_name(&cdb, &p.card_name))
        .unwrap_or_else(|| p.card_name.clone());
    if con
        .execute(
            "INSERT INTO collection (card_name, set_code, artist, lang, quantity, notes,
                                     allocated_deck_id, oracle_id)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                card_name,
                p.set_code,
                p.artist,
                p.lang,
                p.quantity,
                p.notes,
                p.deck_id,
                p.oracle_id
            ],
        )
        .is_err()
    {
        return Err(Error::Internal("Falha ao adicionar".into()));
    }
    let entry_id = con.last_insert_rowid();
    let qty_label = if p.quantity != 1 {
        format!("{}x ", p.quantity)
    } else {
        String::new()
    };

    match p.deck_id {
        Some(did) => {
            let dn = deck_name(&con, did).unwrap_or_else(|| "?".into());
            let _ = con.execute(
                "INSERT INTO deck_cards (deck_id, card_name, quantity, oracle_id)
                 VALUES (?1, ?2, ?3, ?4)",
                params![did, card_name, p.quantity, p.oracle_id],
            );
            log_activity(
                &con,
                "card_new",
                &format!("{qty_label}{card_name} adicionada à coleção e ao deck {dn}"),
            );
        }
        None => log_activity(
            &con,
            "card_new",
            &format!("{qty_label}{card_name} adicionada à coleção"),
        ),
    }
    Ok(entry_id)
}

/// Moves one stored copy into a deck, or back out of every deck when `deck_id` is None.
pub fn allocate_collection(entry_id: i64, deck_id: Option<i64>) -> Result<()> {
    let Ok(con) = open_app_db() else {
        return Err(Error::db_unavailable());
    };
    let card_name: Option<String> = con
        .query_row(
            "SELECT card_name FROM collection WHERE id = ?1",
            [entry_id],
            |r| r.get(0),
        )
        .optional()
        .ok()
        .flatten();
    let _ = con.execute(
        "UPDATE collection SET allocated_deck_id = ?1 WHERE id = ?2",
        params![deck_id, entry_id],
    );
    if let Some(cn) = card_name {
        match deck_id {
            Some(did) => {
                let dn = deck_name(&con, did).unwrap_or_else(|| "?".into());
                log_activity(
                    &con,
                    "card_added_deck",
                    &format!("{cn} alocada ao deck {dn}"),
                );
            }
            None => log_activity(
                &con,
                "card_removed_deck",
                &format!("{cn} liberada (ficou fora de deck)"),
            ),
        }
    }
    Ok(())
}

/// Removes one physical copy. A row can stand for several identical copies (`quantity`), so this
/// decrements first and only deletes the row when it reaches zero - "delete one unit" must not
/// silently discard a stack of four.
///
/// Reports whether that was the last copy of the card anywhere, so the UI can tell the difference
/// between thinning a playset and dropping a card out of the collection entirely.
pub fn delete_collection_entry(entry_id: i64) -> Result<CopyRemoved> {
    let Ok(con) = open_app_db() else {
        return Err(Error::db_unavailable());
    };
    let row: Option<(String, i64, Option<i64>)> = con
        .query_row(
            "SELECT card_name, quantity, allocated_deck_id FROM collection WHERE id = ?1",
            [entry_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .ok()
        .flatten();
    let Some((card_name, quantity, deck_id)) = row else {
        return Err(Error::NotFound("Cópia não encontrada".into()));
    };

    if quantity > 1 {
        let _ = con.execute(
            "UPDATE collection SET quantity = quantity - 1 WHERE id = ?1",
            [entry_id],
        );
    } else {
        let _ = con.execute("DELETE FROM collection WHERE id = ?1", [entry_id]);
    }

    // A copy that was sleeved in a deck leaves that deck's list short by one - the deck_cards row
    // has to follow, or the deck would keep claiming a card that no longer exists.
    if let Some(did) = deck_id {
        let deck_row: Option<(i64, i64)> = con
            .query_row(
                "SELECT id, quantity FROM deck_cards
                 WHERE deck_id = ?1 AND card_name = ?2 COLLATE NOCASE",
                params![did, card_name],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .ok()
            .flatten();
        if let Some((row_id, dq)) = deck_row {
            if dq > 1 {
                let _ = con.execute(
                    "UPDATE deck_cards SET quantity = quantity - 1 WHERE id = ?1",
                    [row_id],
                );
            } else {
                let _ = con.execute("DELETE FROM deck_cards WHERE id = ?1", [row_id]);
            }
        }
    }

    let remaining: i64 = con
        .query_row(
            "SELECT COALESCE(SUM(quantity), 0) FROM collection WHERE card_name = ?1 COLLATE NOCASE",
            [&card_name],
            |r| r.get(0),
        )
        .unwrap_or(0);

    log_activity(
        &con,
        "card_removed",
        &if remaining == 0 {
            format!("{card_name} saiu da coleção (última cópia)")
        } else {
            format!("1 cópia de {card_name} removida da coleção ({remaining} restante(s))")
        },
    );
    Ok(CopyRemoved {
        remaining,
        card_name,
    })
}

/// Edits one stored copy in place.
///
/// Deliberately cannot change `card_name` or `allocated_deck_id`. Renaming would turn this copy
/// into a different card; reallocating is what [`allocate_collection`] is for. The point of
/// editing in place is that a copy sleeved in a deck can gain its set and artist without leaving
/// the deck - deleting and re-adding was the only way before, and that pulled the card out of
/// the deck.
///
/// Changing the quantity of a copy that *is* in a deck moves the deck's count with it, or the
/// deck would keep claiming cards the collection no longer says exist.
pub fn edit_collection_entry(entry_id: i64, p: CollectionEditIn) -> Result<()> {
    let Ok(con) = open_app_db() else {
        return Err(Error::db_unavailable());
    };
    let row: Option<(String, i64, Option<i64>)> = con
        .query_row(
            "SELECT card_name, quantity, allocated_deck_id FROM collection WHERE id = ?1",
            [entry_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .ok()
        .flatten();
    let Some((card_name, old_qty, deck_id)) = row else {
        return Err(Error::NotFound("Cópia não encontrada".into()));
    };

    let new_qty = p.quantity.unwrap_or(old_qty).max(1);
    // COALESCE so an omitted field keeps its stored value, while an explicitly empty string
    // clears it - "I don't know the artist" has to be expressible.
    if con
        .execute(
            "UPDATE collection SET
                set_code = COALESCE(?1, set_code),
                artist   = COALESCE(?2, artist),
                lang     = COALESCE(?3, lang),
                quantity = ?4,
                notes    = COALESCE(?5, notes)
             WHERE id = ?6",
            params![p.set_code, p.artist, p.lang, new_qty, p.notes, entry_id],
        )
        .is_err()
    {
        return Err(Error::Internal("Falha ao salvar".into()));
    }

    if let (Some(did), true) = (deck_id, new_qty != old_qty) {
        let delta = new_qty - old_qty;
        let _ = con.execute(
            "UPDATE deck_cards SET quantity = MAX(1, quantity + ?1)
             WHERE deck_id = ?2 AND card_name = ?3 COLLATE NOCASE",
            params![delta, did, card_name],
        );
    }

    log_activity(
        &con,
        "card_edited",
        &format!("Detalhes de {card_name} atualizados"),
    );
    Ok(())
}
