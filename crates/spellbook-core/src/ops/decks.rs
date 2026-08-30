//! Decks: the list, one deck's contents, the cards in it, auto-build, decklist import and
//! EDHREC synergy.
//!
//! Deck contents live in two places on purpose. `deck_cards` is the *list* - what the deck is
//! meant to contain. `collection` is the *cardboard* - one row per physical copy, pointing at
//! the deck it is sleeved in. Every write here has to keep those two honest with each other:
//! adding a card claims a copy, removing one releases it back to free.

use std::collections::HashMap;

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::db::{deck_name, log_activity, open_app_db, open_cards_db};
use crate::error::{Error, Result};
use crate::ops::cards::{CARD_COLS, MatchKind, lookup_card_in};
use crate::types::{Card, ImageRef};
use crate::wizard::{BuildMeta, CardOwnership};

const TYPE_ORDER: &[&str] = &[
    "Land",
    "Creature",
    "Planeswalker",
    "Battle",
    "Artifact",
    "Enchantment",
    "Instant",
    "Sorcery",
];

fn classify(type_line: &str) -> String {
    for t in TYPE_ORDER {
        if type_line.contains(t) {
            return (*t).to_string();
        }
    }
    "Outro".into()
}

/// One deck as the deck grid lists it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeckSummary {
    pub id: i64,
    pub name: String,
    pub commander_name: String,
    pub commander_name_2: Option<String>,
    pub philosophy: Option<String>,
    /// Stored as a plain comma-separated string, not JSON - the tag pills split it.
    pub tags: Option<String>,
    pub created_at: Option<String>,
    pub total_cards: i64,
    pub wins: i64,
    pub losses: i64,
    pub commander_image: Option<ImageRef>,
    pub commander_image_2: Option<ImageRef>,
    pub color_identity: Option<String>,
}

/// One card in a deck, as the deck view draws it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeckCard {
    /// `deck_cards.id` - the row to delete when the card is removed.
    pub id: i64,
    pub card_name: String,
    pub quantity: i64,
    pub oracle_id: Option<String>,
    pub mana_cost: Option<String>,
    /// `"?"` when the index has no row for this name, exactly as before.
    pub type_line: String,
    pub image: Option<ImageRef>,
    pub image_back: Option<ImageRef>,
    pub cmc: Option<f64>,
    pub price_usd: Option<String>,
    pub edhrec_rank: Option<i64>,
    pub colors: Option<String>,
    pub color_identity: Option<String>,
    pub rarity: Option<String>,
    pub shared_with: Vec<SharedWith>,
}

/// Another deck holding a copy of the same card, by name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedWith {
    pub deck: String,
    pub deck_id: i64,
}

/// One deck with its contents, grouped and counted for the deck screen.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeckDetail {
    pub id: i64,
    pub name: String,
    pub commander_name: String,
    pub commander_name_2: Option<String>,
    pub philosophy: Option<String>,
    pub tags: Option<String>,
    pub created_at: Option<String>,
    pub total_cards: i64,
    pub is_valid_100: bool,
    /// Category ("Comandante", "Land", "Creature", ..., "Outro") -> the cards in it.
    pub by_type: HashMap<String, Vec<DeckCard>>,
    /// Converted mana cost -> copies at that cost. Lands and commanders excluded.
    pub mana_curve: HashMap<i64, i64>,
    /// Per-card ownership, keyed by card name, as `wizard::deck_ownership`
    /// computes it.
    pub ownership: HashMap<String, CardOwnership>,
}

/// Every deck, with the counts and commander art the grid shows.
///
/// Answers an empty list rather than an error when app.db will not open: the deck grid is
/// reachable before the first data download, and an empty grid reads correctly there.
pub fn list_decks() -> Vec<DeckSummary> {
    let Ok(con) = open_app_db() else {
        return Vec::new();
    };
    let cdb = open_cards_db();

    type DeckRow = (
        i64,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    );
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

        // Commander art: the tile shows just the illustration, so it draws the `art_crop`
        // variant of this reference (`ImageRef::variant`) rather than the whole card.
        // Partner decks have a second commander, whose art the tile shows alongside the first.
        let art_of = |c: &Connection, nm: &str| -> (Option<ImageRef>, Option<String>) {
            match lookup_card_in(c, nm) {
                Some((card, _)) => (card.image, card.color_identity),
                None => (None, None),
            }
        };
        let mut commander_image = None;
        let mut commander_image_2 = None;
        let mut color_identity = None;
        if let Some(ref c) = cdb {
            let (img, ci) = art_of(c, &commander);
            commander_image = img;
            color_identity = ci;
            if let Some(ref c2) = commander2
                && !c2.is_empty()
            {
                commander_image_2 = art_of(c, c2).0;
            }
        }

        out.push(DeckSummary {
            id,
            name,
            commander_name: commander,
            commander_name_2: commander2,
            philosophy,
            tags,
            created_at,
            total_cards: total,
            wins,
            losses,
            commander_image,
            commander_image_2,
            color_identity,
        });
    }
    out
}

/// One deck, its cards grouped by type, and everything the deck screen counts.
pub fn get_deck(deck_id: i64) -> Result<DeckDetail> {
    let missing = || Error::NotFound("Deck não encontrado".into());
    let con = open_app_db().map_err(|_| missing())?;
    let header = con.query_row(
        "SELECT id, name, commander_name, commander_name_2, philosophy, tags, created_at
         FROM decks WHERE id = ?1",
        [deck_id],
        |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, Option<String>>(6)?,
            ))
        },
    );
    let Ok((id, name, commander_name, commander_name_2, philosophy, tags, created_at)) = header
    else {
        return Err(missing());
    };

    // (id, card_name, quantity, is_commander, oracle_id)
    let dcards: Vec<(i64, String, i64, i64, Option<String>)> = (|| -> rusqlite::Result<Vec<_>> {
        let mut stmt = con.prepare(
            "SELECT id, card_name, quantity, is_commander, oracle_id FROM deck_cards
             WHERE deck_id = ?1 ORDER BY is_commander DESC, card_name",
        )?;
        let rows = stmt.query_map([deck_id], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get::<_, Option<String>>(4)?,
            ))
        })?;
        rows.collect()
    })()
    .unwrap_or_default();

    // Cards allocated to another deck under the same name (cross-deck duplicates).
    let mut other_map: HashMap<String, Vec<SharedWith>> = HashMap::new();
    if let Ok(mut stmt) = con.prepare(
        "SELECT card_name, decks.name, decks.id FROM collection
         JOIN decks ON decks.id = collection.allocated_deck_id
         WHERE collection.allocated_deck_id != ?1",
    ) && let Ok(rows) = stmt.query_map([deck_id], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, i64>(2)?,
        ))
    }) {
        for row in rows.flatten() {
            other_map.entry(row.0).or_default().push(SharedWith {
                deck: row.1,
                deck_id: row.2,
            });
        }
    }

    let cdb = open_cards_db();
    let mut by_type: HashMap<String, Vec<DeckCard>> = HashMap::new();
    let mut total = 0i64;
    let mut mana_curve: HashMap<i64, i64> = HashMap::new();

    for (row_id, card_name, quantity, is_commander, oracle_id) in dcards {
        let info: Option<Card> = cdb.as_ref().and_then(|c| {
            oracle_id
                .as_deref()
                .and_then(|oid| {
                    c.prepare(&format!(
                        "SELECT {CARD_COLS} FROM cards WHERE oracle_id = ?1"
                    ))
                    .ok()
                    .and_then(|mut s| s.query_row([oid], Card::from_row).ok())
                })
                .or_else(|| lookup_card_in(c, &card_name).map(|(card, _)| card))
        });

        let type_line = info
            .as_ref()
            .and_then(|c| c.type_line.clone())
            .unwrap_or_else(|| "?".to_string());
        let cat = if is_commander == 1 {
            "Comandante".to_string()
        } else {
            classify(&type_line)
        };

        by_type.entry(cat).or_default().push(DeckCard {
            id: row_id,
            oracle_id: info.as_ref().and_then(|c| c.oracle_id.clone()),
            mana_cost: info.as_ref().and_then(|c| c.mana_cost.clone()),
            type_line: type_line.clone(),
            image: info.as_ref().and_then(|c| c.image.clone()),
            image_back: info.as_ref().and_then(|c| c.image_back.clone()),
            cmc: info.as_ref().and_then(|c| c.cmc),
            price_usd: info.as_ref().and_then(|c| c.price_usd.clone()),
            edhrec_rank: info.as_ref().and_then(|c| c.edhrec_rank),
            colors: info.as_ref().and_then(|c| c.colors.clone()),
            color_identity: info.as_ref().and_then(|c| c.color_identity.clone()),
            rarity: info.as_ref().and_then(|c| c.rarity.clone()),
            shared_with: other_map.get(&card_name).cloned().unwrap_or_default(),
            card_name,
            quantity,
        });

        total += quantity;
        if is_commander == 0 && !type_line.contains("Land") {
            let cmc = info.as_ref().and_then(|c| c.cmc).unwrap_or(0.0) as i64;
            *mana_curve.entry(cmc).or_insert(0) += quantity;
        }
    }

    Ok(DeckDetail {
        id,
        name,
        commander_name,
        commander_name_2,
        philosophy,
        tags,
        created_at,
        total_cards: total,
        is_valid_100: total == 100,
        by_type,
        mana_curve,
        // Per-card ownership so the deck view can flag what isn't owned (or is on loan from
        // another deck). Computed on read so it tracks the collection as it changes.
        ownership: crate::wizard::deck_ownership(&con, deck_id),
    })
}

/// The EDHREC picks for a deck's commander, plus the commanders it plays like.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Synergy {
    /// `false` when there is nothing on disk for this commander, which is the
    /// state the "Buscar sinergia agora" button exists to resolve.
    pub cached: bool,
    /// Still `Value`: these come straight out of `edhrec::recommendations` and
    /// get typed when that module is ported.
    pub recommendations: Vec<Value>,
    pub similar_commanders: Vec<Value>,
    /// Set only when there is no cache, explaining which commander is missing.
    pub message: Option<String>,
}

/// Cached EDHREC synergy for this deck's commander - reads local files only, never the network.
pub fn deck_synergy(deck_id: i64) -> Synergy {
    let Ok(con) = open_app_db() else {
        return Synergy::default();
    };
    let commander: Option<String> = con
        .query_row(
            "SELECT commander_name FROM decks WHERE id = ?1",
            [deck_id],
            |r| r.get(0),
        )
        .ok();
    let Some(commander) = commander else {
        return Synergy::default();
    };

    let in_deck: Vec<String> = (|| -> rusqlite::Result<Vec<String>> {
        let mut stmt = con.prepare("SELECT card_name FROM deck_cards WHERE deck_id = ?1")?;
        let rows = stmt.query_map([deck_id], |r| r.get::<_, String>(0))?;
        rows.collect()
    })()
    .unwrap_or_default();

    match crate::edhrec::recommendations(&commander, &in_deck) {
        Some((recs, similar)) => Synergy {
            cached: true,
            recommendations: recs,
            similar_commanders: similar,
            message: None,
        },
        None => Synergy {
            cached: false,
            recommendations: Vec::new(),
            similar_commanders: Vec::new(),
            message: Some(format!("Sem cache do EDHREC para {commander}.")),
        },
    }
}

/// Per-card EDHREC theme tags, card name -> tags. Still the stub the HTTP version
/// was: it answers nothing, and the deck view falls back to grouping by type.
pub fn deck_tags(_deck_id: i64) -> HashMap<String, Vec<String>> {
    HashMap::new()
}

/// Trims, drops empties, de-dupes case-insensitively - port of _normalize_tags().
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
    if out.is_empty() {
        None
    } else {
        Some(out.join(", "))
    }
}

/// The editable fields of a deck, shared by create and update.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeckIn {
    pub name: String,
    pub commander_name: String,
    #[serde(default)]
    pub commander_name_2: Option<String>,
    #[serde(default)]
    pub philosophy: Option<String>,
    #[serde(default)]
    pub tags: Option<String>,
}

/// Creates a deck and its commander row(s). Answers the new deck's id.
pub fn create_deck(p: DeckIn) -> Result<i64> {
    let Ok(con) = open_app_db() else {
        return Err(Error::db_unavailable());
    };
    let c2 = p
        .commander_name_2
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let tags = normalize_tags(p.tags.as_deref());

    if con
        .execute(
            "INSERT INTO decks (name, commander_name, commander_name_2, philosophy, tags)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![p.name, p.commander_name, c2, p.philosophy, tags],
        )
        .is_err()
    {
        return Err(Error::Internal("Falha ao criar o deck".into()));
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
    log_activity(
        &con,
        "deck_built",
        &format!("Deck {} criado (comandante: {label})", p.name),
    );
    Ok(deck_id)
}

/// Renames a deck, rewrites its text fields, and reconciles its commander rows.
pub fn update_deck(deck_id: i64, p: DeckIn) -> Result<()> {
    let Ok(con) = open_app_db() else {
        return Err(Error::db_unavailable());
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
        return Err(Error::NotFound("Deck não encontrado".into()));
    };

    let new_c1 = p.commander_name.trim().to_string();
    if new_c1.is_empty() {
        return Err(Error::BadRequest(
            "Comandante principal é obrigatório.".into(),
        ));
    }
    let new_c2 = p
        .commander_name_2
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let _ = con.execute(
        "UPDATE decks SET name = ?1, philosophy = ?2, commander_name = ?3, commander_name_2 = ?4,
                          tags = ?5 WHERE id = ?6",
        params![
            p.name,
            p.philosophy,
            new_c1,
            new_c2,
            normalize_tags(p.tags.as_deref()),
            deck_id
        ],
    );

    // Reconcile the is_commander rows to the new commander set.
    let old_set: Vec<String> = [Some(old_c1), old_c2].into_iter().flatten().collect();
    let new_set: Vec<String> = [Some(new_c1.clone()), new_c2.map(str::to_string)]
        .into_iter()
        .flatten()
        .collect();
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
    log_activity(
        &con,
        "deck_built",
        &format!("Deck {} editado (comandante: {label})", p.name),
    );
    Ok(())
}

/// What deleting a deck does to the cardboard it was holding.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeleteMode {
    /// Keeps the deck's cards in the collection, just unallocated - the deck was taken apart
    /// but you still own the cards. The default, which keeps `Remove` strictly opt-in.
    #[default]
    Free,
    /// Deletes those collection rows too, for a deck whose cards were never physically owned
    /// (e.g. an auto-built list used as a shopping plan).
    Remove,
}

/// Deletes a deck. Answers how many collection rows went with it.
pub fn delete_deck(deck_id: i64, mode: DeleteMode) -> Result<usize> {
    let Ok(con) = open_app_db() else {
        return Err(Error::db_unavailable());
    };
    let Some(name) = deck_name(&con, deck_id) else {
        return Err(Error::NotFound("Deck não encontrado".into()));
    };
    let remove_from_collection = mode == DeleteMode::Remove;

    let mut removed = 0usize;
    if remove_from_collection {
        // Only the rows this deck owns - anything already free, or allocated elsewhere, is
        // untouched. Runs before the DELETE, since the FK would otherwise null the link first.
        removed = con
            .execute(
                "DELETE FROM collection WHERE allocated_deck_id = ?1",
                [deck_id],
            )
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
    Ok(removed)
}

/// One card being put into a deck.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeckCardIn {
    pub card_name: String,
    #[serde(default = "one")]
    pub quantity: i64,
    #[serde(default)]
    pub oracle_id: Option<String>,
    /// Set once the user has answered the "already in this deck" question.
    #[serde(default)]
    pub confirm: bool,
}

fn one() -> i64 {
    1
}

/// Basic lands plus the handful of cards that print "A deck can have any number of cards named...".
fn allows_unlimited_copies(oracle_text: Option<&str>, card_name: &str) -> bool {
    const BASICS: &[&str] = &[
        "Plains",
        "Island",
        "Swamp",
        "Mountain",
        "Forest",
        "Wastes",
        "Snow-Covered Plains",
        "Snow-Covered Island",
        "Snow-Covered Swamp",
        "Snow-Covered Mountain",
        "Snow-Covered Forest",
    ];
    if BASICS.iter().any(|b| b.eq_ignore_ascii_case(card_name)) {
        return true;
    }
    oracle_text
        .map(|t| {
            t.to_lowercase()
                .contains("a deck can have any number of cards named")
        })
        .unwrap_or(false)
}

/// Puts `quantity` copies of a card into a deck, taking free ones from the collection first.
///
/// Adding a card to a deck used to INSERT a collection row unconditionally, so putting a card you
/// already owned into a deck invented a second copy: own one, add it, and the collection reported
/// two. Cardboard doesn't appear because a deck listed it - a copy is claimed if one is sitting
/// free, and only genuinely new cardboard is recorded as new.
///
/// A free row can stand for several identical copies, so claiming one out of a stack splits it:
/// the free row loses one and an allocated row gains it.
fn claim_copies(
    con: &Connection,
    deck_id: i64,
    card_name: &str,
    quantity: i64,
    oracle_id: Option<&str>,
    note: &str,
) {
    let mut still_needed = quantity;

    while still_needed > 0 {
        let free: Option<(i64, i64)> = con
            .query_row(
                "SELECT id, quantity FROM collection
                 WHERE allocated_deck_id IS NULL AND card_name = ?1 COLLATE NOCASE
                 ORDER BY id LIMIT 1",
                params![card_name],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .ok()
            .flatten();
        let Some((row_id, have)) = free else { break };

        let take = have.min(still_needed);
        if take == have {
            // The whole row moves into the deck; no split needed.
            let _ = con.execute(
                "UPDATE collection SET allocated_deck_id = ?1 WHERE id = ?2",
                params![deck_id, row_id],
            );
        } else {
            let _ = con.execute(
                "UPDATE collection SET quantity = quantity - ?1 WHERE id = ?2",
                params![take, row_id],
            );
            let _ = con.execute(
                "INSERT INTO collection (card_name, lang, quantity, allocated_deck_id, oracle_id, notes)
                 VALUES (?1, 'en', ?2, ?3, ?4, ?5)",
                params![card_name, take, deck_id, oracle_id, note],
            );
        }
        still_needed -= take;
    }

    // Whatever the collection couldn't cover is cardboard the user is telling us they have.
    if still_needed > 0 {
        let _ = con.execute(
            "INSERT INTO collection (card_name, lang, quantity, allocated_deck_id, oracle_id, notes)
             VALUES (?1, 'en', ?2, ?3, ?4, ?5)",
            params![card_name, still_needed, deck_id, oracle_id, note],
        );
    }
}

/// Adds a card to a deck and claims a physical copy for it.
///
/// Answers `Error::Conflict` when the card is already in the deck and `p.confirm` is unset. That
/// is not a failure: the deck screen turns it into a confirmation dialog and calls again with
/// `confirm: true` if the user agrees. The message carries the card and the count that dialog
/// names, since the message is all a conflict can hand back.
pub fn add_deck_card(deck_id: i64, p: DeckCardIn) -> Result<()> {
    let Ok(con) = open_app_db() else {
        return Err(Error::db_unavailable());
    };
    let Some(dname) = deck_name(&con, deck_id) else {
        return Err(Error::NotFound("Deck não encontrado".into()));
    };
    // Same reason as add_collection: store the index's spelling, so a card entered here and a
    // card entered there end up as one card rather than two.
    let card_name = open_cards_db()
        .and_then(|cdb| crate::ops::cards::canonical_name(&cdb, &p.card_name))
        .unwrap_or_else(|| p.card_name.clone());

    let oracle_text: Option<String> = open_cards_db().and_then(|cdb| {
        let sql = match p.oracle_id {
            Some(_) => format!("SELECT {CARD_COLS} FROM cards WHERE oracle_id = ?1"),
            None => format!("SELECT {CARD_COLS} FROM cards WHERE name = ?1 COLLATE NOCASE"),
        };
        let key: &dyn rusqlite::ToSql = p
            .oracle_id
            .as_ref()
            .map(|o| o as &dyn rusqlite::ToSql)
            .unwrap_or(&p.card_name);
        cdb.query_row(&sql, [key], |r| r.get::<_, Option<String>>("oracle_text"))
            .ok()
            .flatten()
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
            return Err(Error::Conflict(format!(
                "{} já está neste deck ({existing_qty}x). Adicionar mais uma?",
                p.card_name
            )));
        }
    }

    // Merge into the row for the same (name, oracle_id) pair - `IS` so two NULLs match - rather
    // than accumulating separate 1x rows when adding the same card repeatedly.
    let existing_row: Option<i64> = con
        .query_row(
            "SELECT id FROM deck_cards WHERE deck_id = ?1 AND card_name = ?2 COLLATE NOCASE
             AND is_commander = 0 AND oracle_id IS ?3",
            params![deck_id, card_name, p.oracle_id],
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
                params![deck_id, card_name, p.quantity, p.oracle_id],
            );
        }
    }
    claim_copies(
        &con,
        deck_id,
        &card_name,
        p.quantity,
        p.oracle_id.as_deref(),
        "Adicionado via app",
    );
    log_activity(
        &con,
        "card_added_deck",
        &format!("{card_name} entrou no deck {dname}"),
    );
    Ok(())
}

/// Takes one `deck_cards` row out of a deck and frees the copy it stood for.
pub fn remove_deck_card(deck_id: i64, card_id: i64) -> Result<()> {
    let Ok(con) = open_app_db() else {
        return Err(Error::db_unavailable());
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
        log_activity(
            &con,
            "card_removed_deck",
            &format!("{cn} saiu do deck {dn} (cópia voltou para as livres)"),
        );
    }
    Ok(())
}

/// What a finished auto-build produced.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AutoBuildResult {
    pub deck_id: i64,
    /// The build's own report: counts per category, the shopping list, and the
    /// cards it could only find in other decks.
    pub meta: BuildMeta,
}

/// Where the last auto-build got to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildState {
    /// Nothing has been built in this session.
    #[default]
    Idle,
    Done,
    Error,
}

/// Mirrors deck_wizard.get_status(): the dialog polls this and opens `result.deck_id`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AutoBuildStatus {
    pub state: BuildState,
    pub task: Option<String>,
    pub percent: i64,
    pub error: Option<String>,
    pub result: Option<AutoBuildResult>,
}

static LAST_BUILD: std::sync::Mutex<Option<AutoBuildResult>> = std::sync::Mutex::new(None);
static LAST_ERROR: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// Synchronous, unlike the Python version's background job: the whole build is a few local
/// SQLite queries and finishes well inside one call, so there is no progress to poll. The
/// dialog's polling loop still works - it sees "done" on its first tick.
pub fn auto_build(p: crate::wizard::AutoBuildIn) -> Result<()> {
    match crate::wizard::build(&p) {
        Ok(out) => {
            let mut st = LAST_BUILD.lock().unwrap();
            *st = Some(AutoBuildResult {
                deck_id: out.deck_id,
                meta: out.meta,
            });
            Ok(())
        }
        Err(e) => {
            let mut st = LAST_BUILD.lock().unwrap();
            *st = None;
            // The status dialog shows the message, so keep it; the caller still
            // gets the error itself, classified as the wizard classified it.
            *LAST_ERROR.lock().unwrap() = Some(e.detail().to_string());
            Err(e)
        }
    }
}

/// The state of the last auto-build, for the progress dialog.
pub fn auto_build_status() -> AutoBuildStatus {
    if let Some(result) = LAST_BUILD.lock().unwrap().clone() {
        return AutoBuildStatus {
            state: BuildState::Done,
            task: Some("Concluído.".into()),
            percent: 100,
            error: None,
            result: Some(result),
        };
    }
    if let Some(err) = LAST_ERROR.lock().unwrap().clone() {
        return AutoBuildStatus {
            state: BuildState::Error,
            task: None,
            percent: 0,
            error: Some(err),
            result: None,
        };
    }
    AutoBuildStatus::default()
}

/// Fetches the EDHREC page for this deck's commander(s) on demand - the "Buscar sinergia agora"
/// button. Needs network; everything downstream of it reads the cache offline.
pub fn fetch_deck_synergy(deck_id: i64) -> Result<()> {
    let Ok(con) = open_app_db() else {
        return Err(Error::db_unavailable());
    };
    let commander: Option<String> = con
        .query_row(
            "SELECT commander_name FROM decks WHERE id = ?1",
            [deck_id],
            |r| r.get(0),
        )
        .optional()
        .ok()
        .flatten();
    let Some(commander) = commander else {
        return Err(Error::NotFound("Deck não encontrado".into()));
    };
    crate::edhrec::fetch(&commander, true).map_err(Error::Upstream)
}

/// One line of a pasted decklist that was placed in the index.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImportMatch {
    /// The index's spelling, which is what gets stored on commit.
    pub name: String,
    pub quantity: i64,
    /// What the pasted line actually said, so an approximate match stays visible.
    pub requested_name: String,
    pub match_type: MatchKind,
    pub mana_cost: Option<String>,
    pub type_line: Option<String>,
    pub image: Option<ImageRef>,
    pub image_back: Option<ImageRef>,
}

/// A line the index has no card for.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportMiss {
    pub requested_name: String,
    pub quantity: i64,
}

/// What a pasted decklist would do, before it does it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ImportPreview {
    pub matched: Vec<ImportMatch>,
    pub not_found: Vec<ImportMiss>,
    pub total_lines: usize,
}

/// Parses + matches without writing anything. The user confirms the preview before any insert,
/// so a name the parser guessed wrong is surfaced rather than silently added.
pub fn import_preview(deck_id: i64, text: &str) -> Result<ImportPreview> {
    let Ok(con) = open_app_db() else {
        return Err(Error::db_unavailable());
    };
    if deck_name(&con, deck_id).is_none() {
        return Err(Error::NotFound("Deck não encontrado".into()));
    }
    let entries = crate::decklist::parse_text(text);
    if entries.is_empty() {
        return Err(Error::BadRequest(
            "Não encontrei nenhuma carta reconhecível no texto colado.".into(),
        ));
    }

    let cdb = open_cards_db();
    let (mut matched, mut not_found): (Vec<ImportMatch>, Vec<ImportMiss>) =
        (Vec::new(), Vec::new());
    let mut seen: HashMap<String, usize> = HashMap::new();
    let total = entries.len();

    for (qty, name) in entries {
        let key = name.to_lowercase();
        // Repeated lines for the same card add up rather than becoming separate preview rows.
        if let Some(&i) = seen.get(&key) {
            if let Some(item) = matched.get_mut(i) {
                item.quantity += qty;
            }
            continue;
        }
        let found = cdb.as_ref().and_then(|c| lookup_card_in(c, &name));
        match found {
            Some((card, how)) => {
                seen.insert(key, matched.len());
                matched.push(ImportMatch {
                    name: card.name,
                    quantity: qty,
                    requested_name: name,
                    match_type: how,
                    mana_cost: card.mana_cost,
                    type_line: card.type_line,
                    image: card.image,
                    image_back: card.image_back,
                });
            }
            None => not_found.push(ImportMiss {
                requested_name: name,
                quantity: qty,
            }),
        }
    }
    Ok(ImportPreview {
        matched,
        not_found,
        total_lines: total,
    })
}

/// One confirmed line of an import.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportCard {
    pub card_name: String,
    #[serde(default = "one")]
    pub quantity: i64,
}

/// What an import does to the cards already in the deck.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImportMode {
    /// Adds to the current list; repeated cards become 2x, 3x...
    #[default]
    Merge,
    /// Trades the whole list for this one. The commanders stay.
    Replace,
}

/// Writes a confirmed import into the deck. Answers how many cards went in.
pub fn import_commit(deck_id: i64, cards: &[ImportCard], mode: ImportMode) -> Result<i64> {
    let Ok(con) = open_app_db() else {
        return Err(Error::db_unavailable());
    };
    let Some(dname) = deck_name(&con, deck_id) else {
        return Err(Error::NotFound("Deck não encontrado".into()));
    };
    let replace = mode == ImportMode::Replace;
    if replace {
        // Commanders are kept - replacing the list shouldn't decapitate the deck.
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
    for card in cards {
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
        claim_copies(
            &con,
            deck_id,
            name,
            card.quantity,
            None,
            "Importado via decklist",
        );
        added += card.quantity;
    }

    let label = if replace {
        "substituindo cartas existentes"
    } else {
        "mesclado com o deck atual"
    };
    log_activity(
        &con,
        "card_added_deck",
        &format!("{added} carta(s) importada(s) para o deck {dname} ({label})"),
    );
    Ok(added)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A card's category is the first entry of `TYPE_ORDER` its type line
    /// mentions, so an Artifact Creature is a Creature and a card the index
    /// could not place is "Outro" rather than being dropped from the list.
    #[test]
    fn classify_follows_the_type_order() {
        assert_eq!(classify("Legendary Creature - Human Wizard"), "Creature");
        assert_eq!(classify("Land - Forest"), "Land");
        assert_eq!(classify("Artifact Creature - Golem"), "Creature");
        assert_eq!(classify("?"), "Outro");
    }

    /// Tags are a comma-separated string typed by hand, so blanks and repeats
    /// are ordinary input rather than errors.
    #[test]
    fn tags_are_trimmed_deduped_and_never_empty() {
        assert_eq!(
            normalize_tags(Some(" cEDH , budget ,, CEDH ")).as_deref(),
            Some("cEDH, budget")
        );
        assert_eq!(normalize_tags(Some("  , ,")), None);
        assert_eq!(normalize_tags(None), None);
    }

    /// The singleton rule only bends for basics and for the cards that print
    /// the "any number" clause; everything else needs a confirmation first.
    #[test]
    fn only_basics_and_any_number_cards_are_unlimited() {
        assert!(allows_unlimited_copies(None, "Forest"));
        assert!(allows_unlimited_copies(None, "snow-covered island"));
        assert!(allows_unlimited_copies(
            Some("A deck can have any number of cards named Persistent Petitioners."),
            "Persistent Petitioners"
        ));
        assert!(!allows_unlimited_copies(Some("Draw a card."), "Sol Ring"));
    }

    /// Keeping the cards is the default on purpose: deleting a deck must not
    /// take cardboard out of the collection unless it was asked to.
    #[test]
    fn the_non_destructive_modes_are_the_defaults() {
        assert_eq!(DeleteMode::default(), DeleteMode::Free);
        assert_eq!(ImportMode::default(), ImportMode::Merge);
    }
}
