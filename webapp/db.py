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
    con = sqlite3.connect(f"file:{CARDS_DB}?mode=ro", uri=True)
    con.row_factory = sqlite3.Row
    return con


def init_db():
    con = get_app_db()
    con.executescript(SCHEMA)
    con.commit()
    con.close()


if __name__ == "__main__":
    init_db()
    print(f"Schema aplicado em {APP_DB}")
