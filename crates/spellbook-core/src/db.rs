//! Database access — a direct port of webapp/db.py.
//!
//! Two SQLite files, same split as the Python version:
//!   app.db     — the user's decks/collection/games. Read-write. Never bundled, never shared.
//!   mtg.sqlite — the Scryfall card index. Read-only, rebuilt by the updater.
//!
//! The schema and migration list are kept byte-compatible with the Python app so the two can
//! open the same app.db interchangeably during the migration period.

use rusqlite::{Connection, OpenFlags, Result};
use std::path::Path;

use crate::paths;

pub const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS decks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    commander_name TEXT NOT NULL,
    philosophy TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS deck_cards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    deck_id INTEGER NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
    card_name TEXT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1,
    is_commander INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS collection (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    card_name TEXT NOT NULL,
    set_code TEXT,
    lang TEXT NOT NULL DEFAULT 'en',
    quantity INTEGER NOT NULL DEFAULT 1,
    allocated_deck_id INTEGER REFERENCES decks(id) ON DELETE SET NULL,
    notes TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);
-- Cards the user wants but doesn't own. Deliberately its own table rather than a flag on
-- `collection`: a wishlist entry is not cardboard, and every count in the app treats a
-- collection row as a card you physically have.
CREATE TABLE IF NOT EXISTS wishlist (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    card_name TEXT NOT NULL,
    set_code TEXT,
    artist TEXT,
    lang TEXT NOT NULL DEFAULT 'en',
    quantity INTEGER NOT NULL DEFAULT 1,
    notes TEXT,
    oracle_id TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS games (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    deck_id INTEGER NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
    played_at TEXT NOT NULL DEFAULT (date('now')),
    result TEXT NOT NULL CHECK(result IN ('vitoria','derrota','empate')),
    opponents TEXT,
    turns INTEGER,
    notes TEXT
);
CREATE TABLE IF NOT EXISTS game_highlights (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    card_name TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS activity (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts TEXT NOT NULL DEFAULT (datetime('now')),
    type TEXT NOT NULL,
    description TEXT NOT NULL
);
"#;

/// (table, column, DDL) — mirrors MIGRATIONS in webapp/db.py.
const MIGRATIONS: &[(&str, &str, &str)] = &[
    (
        "collection",
        "artist",
        "ALTER TABLE collection ADD COLUMN artist TEXT",
    ),
    (
        "decks",
        "commander_name_2",
        "ALTER TABLE decks ADD COLUMN commander_name_2 TEXT",
    ),
    (
        "deck_cards",
        "oracle_id",
        "ALTER TABLE deck_cards ADD COLUMN oracle_id TEXT",
    ),
    (
        "collection",
        "oracle_id",
        "ALTER TABLE collection ADD COLUMN oracle_id TEXT",
    ),
    ("decks", "tags", "ALTER TABLE decks ADD COLUMN tags TEXT"),
];

fn has_column(con: &Connection, table: &str, column: &str) -> Result<bool> {
    let mut stmt = con.prepare(&format!("PRAGMA table_info({table})"))?;
    let mut rows = stmt.query([])?;
    while let Some(r) = rows.next()? {
        let name: String = r.get(1)?;
        if name == column {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn open_app_db() -> Result<Connection> {
    let path = paths::app_db();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let con = Connection::open(&path)?;
    con.execute_batch("PRAGMA foreign_keys = ON;")?;
    Ok(con)
}

pub fn init_app_db() -> Result<()> {
    let con = open_app_db()?;
    con.execute_batch(SCHEMA)?;
    for (table, column, ddl) in MIGRATIONS {
        if !has_column(&con, table, column)? {
            con.execute(ddl, [])?;
        }
    }
    Ok(())
}

/// Appends to the activity feed the home screen shows. Deliberately infallible: a deck edit that
/// succeeded must not be reported as failed because its history line didn't write.
pub fn log_activity(con: &Connection, type_: &str, description: &str) {
    let _ = con.execute(
        "INSERT INTO activity (type, description) VALUES (?1, ?2)",
        rusqlite::params![type_, description],
    );
}

/// A deck's name, or None if the id doesn't exist — used for the messages in the activity log.
pub fn deck_name(con: &Connection, deck_id: i64) -> Option<String> {
    use rusqlite::OptionalExtension;
    con.query_row("SELECT name FROM decks WHERE id = ?1", [deck_id], |r| {
        r.get(0)
    })
    .optional()
    .ok()
    .flatten()
}

/// Read-only handle to the card index, or None when it hasn't been built yet (fresh install).
/// Every caller must degrade gracefully — same contract as get_cards_db() in Python.
pub fn open_cards_db() -> Option<Connection> {
    let path = paths::cards_db();
    if !Path::new(&path).exists() {
        return None;
    }
    Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY).ok()
}

/// Lowercased, accent-stripped form used for forgiving name matching — port of mtgdb.fold_text().
/// SQLite's NOCASE is ASCII-only, so without this a typed "Seance Board" never finds
/// "Séance Board" (42% of the Portuguese printed names carry accents).
pub fn fold_text(s: &str) -> String {
    // NFD-decompose, then drop the combining marks (U+0300..U+036F covers the Latin accents
    // present in card names), then lowercase.
    let decomposed = unicode_decompose(s);
    decomposed
        .chars()
        .filter(|c| !('\u{0300}'..='\u{036F}').contains(c))
        .collect::<String>()
        .to_lowercase()
}

/// Minimal NFD for the Latin-1/Latin Extended-A range actually used by MTG card names.
/// Avoids pulling in a full Unicode normalization crate for a fixed, small character set.
fn unicode_decompose(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' => {
                out.push('A');
                out.push('\u{0301}');
            }
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => {
                out.push('a');
                out.push('\u{0301}');
            }
            'È' | 'É' | 'Ê' | 'Ë' => {
                out.push('E');
                out.push('\u{0301}');
            }
            'è' | 'é' | 'ê' | 'ë' => {
                out.push('e');
                out.push('\u{0301}');
            }
            'Ì' | 'Í' | 'Î' | 'Ï' => {
                out.push('I');
                out.push('\u{0301}');
            }
            'ì' | 'í' | 'î' | 'ï' => {
                out.push('i');
                out.push('\u{0301}');
            }
            'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' => {
                out.push('O');
                out.push('\u{0301}');
            }
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' => {
                out.push('o');
                out.push('\u{0301}');
            }
            'Ù' | 'Ú' | 'Û' | 'Ü' => {
                out.push('U');
                out.push('\u{0301}');
            }
            'ù' | 'ú' | 'û' | 'ü' => {
                out.push('u');
                out.push('\u{0301}');
            }
            'Ç' => {
                out.push('C');
                out.push('\u{0301}');
            }
            'ç' => {
                out.push('c');
                out.push('\u{0301}');
            }
            'Ñ' => {
                out.push('N');
                out.push('\u{0301}');
            }
            'ñ' => {
                out.push('n');
                out.push('\u{0301}');
            }
            'Ý' => {
                out.push('Y');
                out.push('\u{0301}');
            }
            'ý' | 'ÿ' => {
                out.push('y');
                out.push('\u{0301}');
            }
            'Æ' => out.push_str("AE"),
            'æ' => out.push_str("ae"),
            'Ø' => out.push('O'),
            'ø' => out.push('o'),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::fold_text;

    #[test]
    fn folds_accents_like_the_python_version() {
        assert_eq!(fold_text("Séance Board"), "seance board");
        assert_eq!(
            fold_text("Andúril, Flame of the West"),
            "anduril, flame of the west"
        );
        assert_eq!(
            fold_text("A Ascensão da Onda Faminta"),
            "a ascensao da onda faminta"
        );
        assert_eq!(fold_text("Adéwalé"), "adewale");
        // already-plain text is only lowercased
        assert_eq!(fold_text("Sol Ring"), "sol ring");
    }
}
