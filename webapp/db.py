"""Access to the two databases: app.db (the owner's data) and mtg.sqlite (card database, read-only)."""
import os
import sqlite3

HERE = os.path.dirname(os.path.abspath(__file__))
APP_DB = os.path.join(HERE, "app.db")
CARDS_DB = os.path.join(os.path.dirname(HERE), "data", "mtg.sqlite")

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


def init_db():
    con = get_app_db()
    con.executescript(SCHEMA)
    run_migrations(con)
    con.commit()
    con.close()


if __name__ == "__main__":
    init_db()
    print(f"Schema aplicado em {APP_DB}")
