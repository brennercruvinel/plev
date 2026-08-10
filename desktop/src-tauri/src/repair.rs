//! One-off data repairs that need both databases.
//!
//! Lives apart from `db.rs` (which owns app.db) and `routes/cards.rs` (which owns the index)
//! because it is the only thing that reaches into both, and putting it in either would make
//! those two modules depend on each other.

use rusqlite::Connection;

use crate::db::{open_app_db, open_cards_db};
use crate::routes::cards::canonical_name;

/// Rewrites stored card names to the index's own spelling.
///
/// The collection and deck lists key on a free-text name, and the write paths don't agree on
/// which spelling to store: an Adventure card arrives as "Murderous Rider" from one path and
/// "Murderous Rider // Swift End" from another, and a name typed without accents keeps them off.
/// Anything that groups or matches by that string then sees one card as two — two tiles in the
/// collection for the same card, and a deck that can't see the free copy sitting right there.
///
/// Idempotent and cheap (a few hundred rows), so it runs at startup: names can also change under
/// the app when the index is rebuilt after a Scryfall update.
///
/// Only exact matches are rewritten — see `canonical_name`. A name the index can't place is left
/// exactly as the user typed it.
pub fn canonicalise_card_names() -> usize {
    let (Ok(con), Some(cdb)) = (open_app_db(), open_cards_db()) else {
        return 0;
    };

    let mut fixed = 0;
    for table in ["collection", "deck_cards"] {
        let names: Vec<String> = (|| -> rusqlite::Result<Vec<String>> {
            let mut stmt = con.prepare(&format!("SELECT DISTINCT card_name FROM {table}"))?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
            rows.collect()
        })()
        .unwrap_or_default();

        for raw in names {
            let Some(canon) = canonical_name(&cdb, &raw) else { continue };
            // COLLATE NOCASE would also catch rows differing only in case, which is the same bug.
            let n = con
                .execute(
                    &format!("UPDATE {table} SET card_name = ?1 WHERE card_name = ?2 COLLATE NOCASE"),
                    rusqlite::params![canon, raw],
                )
                .unwrap_or(0);
            if n > 0 {
                println!("[repair] {table}: {raw:?} -> {canon:?} ({n} linha(s))");
                fixed += n;
            }
        }
    }
    merge_duplicate_deck_rows(&con);
    fixed
}

/// Renaming can leave a deck holding two rows for what is now the same card — one entered as
/// "Murderous Rider", one as its full name. They have to become one row carrying the sum, or the
/// deck would list the card twice.
fn merge_duplicate_deck_rows(con: &Connection) {
    let dupes: Vec<(i64, String)> = (|| -> rusqlite::Result<Vec<(i64, String)>> {
        let mut stmt = con.prepare(
            "SELECT deck_id, card_name FROM deck_cards
             GROUP BY deck_id, card_name COLLATE NOCASE, is_commander
             HAVING COUNT(*) > 1",
        )?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        rows.collect()
    })()
    .unwrap_or_default();

    for (deck_id, name) in dupes {
        let _ = con.execute(
            "UPDATE deck_cards SET quantity = (
                 SELECT SUM(quantity) FROM deck_cards
                 WHERE deck_id = ?1 AND card_name = ?2 COLLATE NOCASE
             )
             WHERE id = (
                 SELECT MIN(id) FROM deck_cards
                 WHERE deck_id = ?1 AND card_name = ?2 COLLATE NOCASE
             )",
            rusqlite::params![deck_id, name],
        );
        let _ = con.execute(
            "DELETE FROM deck_cards
             WHERE deck_id = ?1 AND card_name = ?2 COLLATE NOCASE
               AND id > (SELECT MIN(id) FROM deck_cards
                         WHERE deck_id = ?1 AND card_name = ?2 COLLATE NOCASE)",
            rusqlite::params![deck_id, name],
        );
        println!("[repair] deck {deck_id}: linhas duplicadas de {name:?} unidas");
    }
}
