"""Access to the two databases: app.db (the owner's data) and mtg.sqlite (card database, read-only)."""
import os
import sqlite3
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
APP_DB = os.path.join(HERE, "app.db")
CARDS_DB = os.path.join(ROOT, "data", "mtg.sqlite")

# mtgdb.py (repo root) owns the card-index schema, including fold_text() — the same normalization
# used to populate the folded columns at build time must be used to query them. Same sys.path
# pattern as data_update.py / deck_wizard.py.
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

SCHEMA = """
CREATE TABLE IF NOT EXISTS decks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    commander_name TEXT NOT NULL,
    philosophy TEXT,
    tags TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS deck_cards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    deck_id INTEGER NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
    card_name TEXT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1,
    is_commander INTEGER NOT NULL DEFAULT 0,
    oracle_id TEXT
);

CREATE TABLE IF NOT EXISTS collection (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    card_name TEXT NOT NULL,
    set_code TEXT,
    lang TEXT NOT NULL DEFAULT 'en',
    quantity INTEGER NOT NULL DEFAULT 1,
    allocated_deck_id INTEGER REFERENCES decks(id) ON DELETE SET NULL,
    notes TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    oracle_id TEXT
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
"""


def log_activity(con, type_, description):
    """Records an event in the app's activity log (Overview screen)."""
    con.execute("INSERT INTO activity (type, description) VALUES (?, ?)", (type_, description))


def get_app_db():
    con = sqlite3.connect(APP_DB)
    con.row_factory = sqlite3.Row
    con.execute("PRAGMA foreign_keys = ON")
    return con


def get_cards_db():
    """Returns None if the card index hasn't been built yet (fresh install, before the first data update)."""
    if not os.path.exists(CARDS_DB):
        return None
    con = sqlite3.connect(f"file:{CARDS_DB}?mode=ro", uri=True)
    con.row_factory = sqlite3.Row
    return con


MIGRATIONS = [
    ("collection", "artist", "ALTER TABLE collection ADD COLUMN artist TEXT"),
    ("decks", "commander_name_2", "ALTER TABLE decks ADD COLUMN commander_name_2 TEXT"),
    # Distinguishes cards that share a printed name but are actually different game objects
    # (e.g. "Phyrexian Hydra" the 5-mana creature vs. the token of the same name) — lets the
    # deck/collection remember exactly which oracle_id was picked instead of always resolving
    # an ambiguous name to whichever row the cards index happens to return first.
    ("deck_cards", "oracle_id", "ALTER TABLE deck_cards ADD COLUMN oracle_id TEXT"),
    ("collection", "oracle_id", "ALTER TABLE collection ADD COLUMN oracle_id TEXT"),
    # Free-text, comma-separated custom labels the owner assigns per deck (e.g. "Competitivo,
    # Orçamento baixo") — purely descriptive, no relation to the EDHREC card tags used for
    # grouping cards within a single deck (see the /decks/{id}/tags endpoint in server.py).
    ("decks", "tags", "ALTER TABLE decks ADD COLUMN tags TEXT"),
]


def run_migrations(con):
    """Adds columns to tables that already existed before the column was introduced (CREATE TABLE IF NOT EXISTS won't do this)."""
    for table, column, ddl in MIGRATIONS:
        cols = [row["name"] for row in con.execute(f"PRAGMA table_info({table})")]
        if column not in cols:
            con.execute(ddl)


# Schema the app's queries rely on that a card index built by an older mtgdb.py won't have.
# Applying it here (rather than only in mtgdb.py's schema) means an existing install self-heals
# on the next server start instead of needing a full ~400MB re-download and rebuild.
CARDS_INDEXES = [
    "CREATE INDEX IF NOT EXISTS idx_pt_oracle ON names_pt(oracle_id)",
    "CREATE INDEX IF NOT EXISTS idx_name_folded ON cards(name_folded)",
    "CREATE INDEX IF NOT EXISTS idx_pt_folded ON names_pt(printed_name_folded)",
]

# (table, column, DDL, source column to backfill from) — accent-folded copies of the name columns,
# used for forgiving lookups. See mtgdb.fold_text().
CARDS_COLUMNS = [
    ("cards", "name_folded", "ALTER TABLE cards ADD COLUMN name_folded TEXT", "name"),
    ("names_pt", "printed_name_folded", "ALTER TABLE names_pt ADD COLUMN printed_name_folded TEXT", "printed_name"),
]


def ensure_cards_indexes():
    """Best-effort: brings an already-built card index up to the schema the app expects (folded
    name columns + indexes). Silently skips when the index hasn't been built yet or the file isn't
    writable — it's an optimization, never a requirement, and the app works without it."""
    if not os.path.exists(CARDS_DB):
        return
    try:
        from mtgdb import fold_text
    except ImportError:
        return
    try:
        con = sqlite3.connect(CARDS_DB)
        con.row_factory = sqlite3.Row
        try:
            for table, column, ddl, src in CARDS_COLUMNS:
                cols = [r["name"] for r in con.execute(f"PRAGMA table_info({table})")]
                if column not in cols:
                    con.execute(ddl)
                # Backfill anything still NULL — covers both a freshly added column and a
                # previous run that was interrupted partway through.
                todo = con.execute(
                    f"SELECT rowid, {src} FROM {table} WHERE {column} IS NULL AND {src} IS NOT NULL"
                ).fetchall()
                if todo:
                    con.executemany(
                        f"UPDATE {table} SET {column} = ? WHERE rowid = ?",
                        [(fold_text(r[src]), r["rowid"]) for r in todo],
                    )
            for ddl in CARDS_INDEXES:
                con.execute(ddl)
            con.commit()
        finally:
            con.close()
    except sqlite3.Error:
        pass  # read-only file, locked by another process, etc. — the app still works, just slower


def init_db():
    con = get_app_db()
    con.executescript(SCHEMA)
    run_migrations(con)
    con.commit()
    con.close()


if __name__ == "__main__":
    init_db()
    print(f"Schema aplicado em {APP_DB}")
