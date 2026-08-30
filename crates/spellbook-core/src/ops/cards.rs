//! Card index lookups - the read-only Scryfall database.
//!
//! Nothing here touches app.db: these operations only ever read `mtg.sqlite`,
//! which the updater rebuilds. Name matching is deliberately forgiving, in
//! this order: exact, accent-folded, front face of a two-faced card, official
//! Portuguese printed name, then substring - so a name typed from memory
//! still lands.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::db::{fold_text, open_cards_db};
use crate::error::{Error, Result};
use crate::types::{Card, ImageRef};

pub const CARD_COLS: &str = "oracle_id, name, mana_cost, cmc, type_line, oracle_text, power, toughness, \
     loyalty, colors, color_identity, rarity, set_code, keywords, commander_legal, price_usd, \
     reserved, edhrec_rank, uri, image_uri, game_changer, layout, image_uri_back";

/// How confidently a lookup placed the name it was given.
///
/// This used to be a Portuguese string that callers pattern-matched with
/// `how.starts_with("exata")`. `canonical_name` rewrites stored card names on
/// the strength of that test, so a reworded label would have silently started
/// renaming rows on fuzzy hits. The distinction is a rule, so it is a type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchKind {
    /// Matched the printed English name, exactly or accent-folded.
    Exact,
    /// Matched the front face of a two-faced card ("Murderous Rider" for
    /// "Murderous Rider // Swift End").
    ExactFrontFace,
    /// Matched an unambiguous localized printed name.
    ExactTranslated,
    /// Matched by oracle_id, which the caller supplied.
    ExactOracleId,
    /// Substring match. Never trustworthy enough to write back as data.
    Approximate,
}

impl MatchKind {
    /// Whether this match is certain enough to store as the card's name.
    pub fn is_exact(self) -> bool {
        !matches!(self, MatchKind::Approximate)
    }

    /// Label shown in the card modal, unchanged from the HTTP version.
    pub fn label(self) -> &'static str {
        match self {
            MatchKind::Exact => "exata",
            MatchKind::ExactFrontFace => "exata (face frontal)",
            MatchKind::ExactTranslated => "exata (nome traduzido)",
            MatchKind::ExactOracleId => "exata (oracle_id)",
            MatchKind::Approximate => "aproximada",
        }
    }
}

/// A card's name as printed in one language.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalizedName {
    pub printed_name: Option<String>,
    pub set_code: Option<String>,
    pub lang: Option<String>,
}

/// One printing: a distinct artwork of a card.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Printing {
    pub set_code: Option<String>,
    pub set_name: Option<String>,
    pub collector_number: Option<String>,
    pub artist: Option<String>,
    pub released_at: Option<String>,
    pub image: Option<ImageRef>,
    pub image_back: Option<ImageRef>,
}

/// A set, as the autocomplete lists it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SetInfo {
    pub code: String,
    pub name: Option<String>,
    pub released_at: Option<String>,
    pub set_type: Option<String>,
    pub cards: Option<i64>,
}

/// A card plus everything the card modal shows alongside it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CardDetail {
    pub card: Card,
    pub match_kind: MatchKind,
    pub pt_names: Vec<LocalizedName>,
}

/// Free-text card search, best-known cards first.
///
/// Answers an empty list rather than an error when the index is missing: the
/// search field is reachable before the first data download, and an empty
/// dropdown reads correctly there while an error toast would not.
pub fn search(q: &str, limit: i64) -> Vec<Card> {
    let Some(cdb) = open_cards_db() else {
        return Vec::new();
    };
    let like = format!("%{}%", fold_text(q));
    let sql = format!(
        "SELECT {CARD_COLS} FROM cards c
         WHERE c.name_folded LIKE ?1
            OR c.oracle_id IN (SELECT oracle_id FROM names_localized WHERE printed_name_folded LIKE ?1)
         ORDER BY (CASE WHEN c.edhrec_rank IS NULL THEN 999999 ELSE c.edhrec_rank END) ASC
         LIMIT ?2"
    );
    (|| -> rusqlite::Result<Vec<Card>> {
        let mut stmt = cdb.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params![like, limit], Card::from_row)?;
        rows.collect()
    })()
    .unwrap_or_default()
}

/// Port of _lookup_card_in(): exact, then accent-folded, then front face,
/// then Portuguese, then approximate.
pub fn lookup_card_in(cdb: &Connection, name: &str) -> Option<(Card, MatchKind)> {
    let folded = fold_text(name);
    let one = |sql: &str, p: &[&dyn rusqlite::ToSql]| -> Option<Card> {
        let mut stmt = cdb.prepare(sql).ok()?;
        stmt.query_row(p, Card::from_row).ok()
    };

    if let Some(c) = one(
        &format!("SELECT {CARD_COLS} FROM cards WHERE name = ?1 COLLATE NOCASE"),
        &[&name],
    ) {
        return Some((c, MatchKind::Exact));
    }
    if let Some(c) = one(
        &format!("SELECT {CARD_COLS} FROM cards WHERE name_folded = ?1"),
        &[&folded],
    ) {
        return Some((c, MatchKind::Exact));
    }

    // Front face of a two-faced card. Exporters and older imports write
    // "Murderous Rider" where the index carries "Murderous Rider // Swift
    // End"; this is a full match on the part before the separator, not a
    // substring guess, so it belongs with the exact matches rather than in the
    // LIKE fallback below - which would return it as approximate and leave
    // callers that only trust exact matches (canonical_name) unable to place a
    // card they can clearly name.
    let front = format!("{folded} // %");
    if let Some(c) = one(
        &format!(
            "SELECT {CARD_COLS} FROM cards WHERE name_folded LIKE ?1 ORDER BY length(name) LIMIT 1"
        ),
        &[&front],
    ) {
        return Some((c, MatchKind::ExactFrontFace));
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
    if ids.len() == 1
        && let Some(c) = one(
            &format!("SELECT {CARD_COLS} FROM cards WHERE oracle_id = ?1"),
            &[&ids[0]],
        )
    {
        return Some((c, MatchKind::ExactTranslated));
    }

    let like = format!("%{name}%");
    if let Some(c) = one(
        &format!("SELECT {CARD_COLS} FROM cards WHERE name LIKE ?1 COLLATE NOCASE LIMIT 1"),
        &[&like],
    ) {
        return Some((c, MatchKind::Approximate));
    }
    let like_folded = format!("%{folded}%");
    if let Some(c) = one(
        &format!("SELECT {CARD_COLS} FROM cards WHERE name_folded LIKE ?1 LIMIT 1"),
        &[&like_folded],
    ) {
        return Some((c, MatchKind::Approximate));
    }
    None
}

/// The index's own spelling of a card, for anything that stores a name as data.
///
/// `collection.card_name` and `deck_cards.card_name` are free text, and the
/// write paths disagree about which spelling to use: an Adventure card is
/// "Murderous Rider" from one and "Murderous Rider // Swift End" from another,
/// and a name typed without accents stays that way. Two rows for one card then
/// look like two different cards to anything grouping by the string.
///
/// Returns None for approximate matches - renaming a row on a fuzzy hit would
/// turn a typo into a confident claim about a card the user never entered.
pub fn canonical_name(cdb: &Connection, raw: &str) -> Option<String> {
    let (card, how) = lookup_card_in(cdb, raw)?;
    if !how.is_exact() {
        return None;
    }
    (card.name != raw).then_some(card.name)
}

/// One card by name, or by oracle_id when the caller already resolved it,
/// with the localized names the modal lists underneath.
pub fn get(name: &str, oracle_id: Option<&str>) -> Result<CardDetail> {
    let missing = || Error::NotFound(format!("Carta não encontrada: {name}"));
    let cdb = open_cards_db().ok_or_else(missing)?;

    let found = if let Some(oid) = oracle_id {
        cdb.prepare(&format!(
            "SELECT {CARD_COLS} FROM cards WHERE oracle_id = ?1"
        ))
        .ok()
        .and_then(|mut s| s.query_row([oid], Card::from_row).ok())
        .map(|c| (c, MatchKind::ExactOracleId))
    } else {
        lookup_card_in(&cdb, name)
    };
    let (card, match_kind) = found.ok_or_else(missing)?;

    let oid = card.oracle_id.clone().unwrap_or_default();
    let pt_names = (|| -> rusqlite::Result<Vec<LocalizedName>> {
        let mut stmt = cdb.prepare(
            "SELECT printed_name, set_code, lang FROM names_localized WHERE oracle_id = ?1
             GROUP BY lang HAVING MIN(lang_rank) ORDER BY lang_rank",
        )?;
        let rows = stmt.query_map([&oid], |r| {
            Ok(LocalizedName {
                printed_name: r.get(0)?,
                set_code: r.get(1)?,
                lang: r.get(2)?,
            })
        })?;
        rows.collect()
    })()
    .unwrap_or_default();

    Ok(CardDetail {
        card,
        match_kind,
        pt_names,
    })
}

/// Every row in the index sharing this exact name.
pub fn variants(name: &str) -> Vec<Card> {
    let Some(cdb) = open_cards_db() else {
        return Vec::new();
    };
    (|| -> rusqlite::Result<Vec<Card>> {
        let mut stmt = cdb.prepare(&format!(
            "SELECT {CARD_COLS} FROM cards WHERE name = ?1 COLLATE NOCASE"
        ))?;
        let rows = stmt.query_map([name], Card::from_row)?;
        rows.collect()
    })()
    .unwrap_or_default()
}

/// Every distinct artwork of a card, newest printing first.
///
/// Split, adventure and other single-faced "//" cards have one art per
/// printing like anything else; two-sided cards carry both faces here, so the
/// flip control keeps working per printing.
pub fn printings(name: &str) -> Vec<Printing> {
    let Some(cdb) = open_cards_db() else {
        return Vec::new();
    };
    let Some((card, _)) = lookup_card_in(&cdb, name) else {
        return Vec::new();
    };
    let Some(oid) = card.oracle_id else {
        return Vec::new();
    };
    (|| -> rusqlite::Result<Vec<Printing>> {
        let mut stmt = cdb.prepare(
            "SELECT p.set_code, p.collector_number, p.artist, p.image_uri, p.image_uri_back,
                    p.released_at, s.name
             FROM printings p LEFT JOIN sets s ON s.code = p.set_code
             WHERE p.oracle_id = ?1
             ORDER BY p.released_at DESC",
        )?;
        let rows = stmt.query_map([&oid], |r| {
            Ok(Printing {
                set_code: r.get(0)?,
                collector_number: r.get(1)?,
                artist: r.get(2)?,
                image: ImageRef::from_url(r.get::<_, Option<String>>(3)?.as_deref()),
                image_back: ImageRef::from_url(r.get::<_, Option<String>>(4)?.as_deref()),
                released_at: r.get(5)?,
                set_name: r.get(6)?,
            })
        })?;
        rows.collect()
    })()
    .unwrap_or_default()
}

/// The image of one specific printing, so a copy recorded as being from a
/// given set shows that set's art rather than whichever printing the index
/// happens to treat as canonical.
pub fn printing_image(
    cdb: &Connection,
    oracle_id: &str,
    set_code: &str,
) -> Option<(ImageRef, Option<ImageRef>)> {
    use rusqlite::OptionalExtension;
    cdb.query_row(
        "SELECT image_uri, image_uri_back FROM printings
         WHERE oracle_id = ?1 AND set_code = ?2 COLLATE NOCASE
         ORDER BY released_at DESC LIMIT 1",
        rusqlite::params![oracle_id, set_code],
        |r| {
            Ok((
                r.get::<_, Option<String>>(0)?,
                r.get::<_, Option<String>>(1)?,
            ))
        },
    )
    .optional()
    .ok()
    .flatten()
    .and_then(|(f, b)| {
        ImageRef::from_url(f.as_deref()).map(|front| (front, ImageRef::from_url(b.as_deref())))
    })
}

/// Set autocomplete. Matches the name accent-folded *and* the code, because
/// for recent sets the three-letter code is what people remember, and Scryfall
/// does not localise set names - so matching only a translated name would find
/// nothing.
///
/// `card`, when given, scopes the list to sets that actually printed it.
/// Offering the full 1,047 sets lets someone pick a set the card was never
/// printed in - which then resolves to no artwork at all, silently. Scoping
/// makes the field answer the question actually being asked: "which printing
/// of this card do I have?"
pub fn sets(q: &str, card: Option<&str>, limit: i64) -> Vec<SetInfo> {
    let Some(cdb) = open_cards_db() else {
        return Vec::new();
    };
    let read = |stmt: &mut rusqlite::Statement, params: &[&dyn rusqlite::ToSql]| -> Vec<SetInfo> {
        (|| -> rusqlite::Result<Vec<SetInfo>> {
            let rows = stmt.query_map(params, |r| {
                Ok(SetInfo {
                    code: r.get::<_, Option<String>>(0)?.unwrap_or_default(),
                    name: r.get(1)?,
                    released_at: r.get(2)?,
                    set_type: r.get(3)?,
                    cards: r.get(4)?,
                })
            })?;
            rows.collect()
        })()
        .unwrap_or_default()
    };

    let like = format!("%{}%", fold_text(q));
    let code_like = format!("{}%", q.to_lowercase());

    if let Some(card) = card.map(str::trim).filter(|c| !c.is_empty())
        && let Some(oid) = lookup_card_in(&cdb, card).and_then(|(c, _)| c.oracle_id)
    {
        let Ok(mut stmt) = cdb.prepare(
            "SELECT DISTINCT p.set_code, s.name, s.released_at, s.set_type, s.cards
             FROM printings p LEFT JOIN sets s ON s.code = p.set_code
             WHERE p.oracle_id = ?1
               AND (?2 = '' OR s.name_folded LIKE ?3 OR p.set_code LIKE ?4)
             ORDER BY s.released_at DESC
             LIMIT ?5",
        ) else {
            return Vec::new();
        };
        return read(
            &mut stmt,
            rusqlite::params![oid, q, like, code_like, limit.max(40)],
        );
    }

    // Ranked, not just filtered. A raw "newest first" buried Dominaria United
    // under its own token, art-series and promo sets - the ones nobody
    // catalogues a card into. Real sets come first, then a name that *starts*
    // with what was typed, then recency.
    let starts = format!("{}%", fold_text(q));
    let lowered = q.to_lowercase();
    let Ok(mut stmt) = cdb.prepare(
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
    ) else {
        return Vec::new();
    };
    read(
        &mut stmt,
        rusqlite::params![like, code_like, limit, lowered, starts],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `canonical_name` rewrites stored card names, so it must refuse
    /// anything short of an exact match. This is the rule the old
    /// `starts_with("exata")` string test encoded.
    #[test]
    fn only_exact_matches_are_safe_to_store() {
        assert!(MatchKind::Exact.is_exact());
        assert!(MatchKind::ExactFrontFace.is_exact());
        assert!(MatchKind::ExactTranslated.is_exact());
        assert!(MatchKind::ExactOracleId.is_exact());
        assert!(!MatchKind::Approximate.is_exact());
    }

    /// The labels are shown in the card modal; they are the strings the HTTP
    /// version answered, so a reader sees no change.
    #[test]
    fn labels_match_the_http_version() {
        assert_eq!(MatchKind::Exact.label(), "exata");
        assert_eq!(MatchKind::ExactFrontFace.label(), "exata (face frontal)");
        assert_eq!(MatchKind::ExactTranslated.label(), "exata (nome traduzido)");
        assert_eq!(MatchKind::ExactOracleId.label(), "exata (oracle_id)");
        assert_eq!(MatchKind::Approximate.label(), "aproximada");
    }
}
