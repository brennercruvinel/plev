"""
Spellbook — local prototype for collection/deck management.

Runs 100% on localhost. Card data (38k+ cards, official Portuguese names,
images) comes from the local mtg.sqlite database (Scryfall bulk data, already
downloaded and indexed — see MTG/API Scryfall - Acesso e Uso.md in the vault).
No network call is needed for core data; only optional live price lookups use
the Scryfall API.

Note: user-facing strings returned by this API (error messages, activity log
descriptions, etc.) are intentionally in Portuguese — that's the app's actual
interface language for its owner. Only this file's own comments/docstrings
follow the repository's English convention.
"""
import os
import re
import sys
import unicodedata
from typing import List, Optional

from fastapi import FastAPI, HTTPException
from fastapi.staticfiles import StaticFiles
from fastapi.responses import FileResponse
from pydantic import BaseModel

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from db import get_app_db, get_cards_db, init_db, log_activity  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
EDHREC_CACHE = os.path.join(os.path.dirname(HERE), "data", "edhrec")

app = FastAPI(title="Spellbook")
init_db()


def slugify(name: str) -> str:
    name = unicodedata.normalize("NFKD", name).encode("ascii", "ignore").decode()
    name = name.lower()
    name = re.sub(r"[^a-z0-9]+", "-", name)
    return name.strip("-")


def card_row_to_dict(r):
    d = dict(r)
    return d


def lookup_card(name: str):
    """Finds a card by name (exact English, exact Portuguese, or approximate)."""
    cdb = get_cards_db()
    r = cdb.execute("SELECT * FROM cards WHERE name = ? COLLATE NOCASE", (name,)).fetchone()
    if r:
        cdb.close()
        return card_row_to_dict(r), "exata"
    row = cdb.execute(
        "SELECT DISTINCT oracle_id FROM names_pt WHERE printed_name = ? COLLATE NOCASE", (name,)
    ).fetchall()
    if len(row) == 1:
        r = cdb.execute("SELECT * FROM cards WHERE oracle_id = ?", (row[0]["oracle_id"],)).fetchone()
        if r:
            cdb.close()
            return card_row_to_dict(r), "exata (nome em português)"
    r = cdb.execute("SELECT * FROM cards WHERE name LIKE ? COLLATE NOCASE LIMIT 1", (f"%{name}%",)).fetchone()
    if r:
        cdb.close()
        return card_row_to_dict(r), "aproximada"
    cdb.close()
    return None, None


# ------------------------------------------------------------------ cards ----

@app.get("/api/cards/search")
def search_cards(q: str = "", limit: int = 30):
    cdb = get_cards_db()
    q_like = f"%{q}%"
    rows = cdb.execute(
        """
        SELECT c.* FROM cards c
        WHERE c.name LIKE ? COLLATE NOCASE
           OR c.oracle_id IN (SELECT oracle_id FROM names_pt WHERE printed_name LIKE ? COLLATE NOCASE)
        ORDER BY (CASE WHEN c.edhrec_rank IS NULL THEN 999999 ELSE c.edhrec_rank END) ASC
        LIMIT ?
        """,
        (q_like, q_like, limit),
    ).fetchall()
    cdb.close()
    return [card_row_to_dict(r) for r in rows]


@app.get("/api/cards/{name}")
def get_card(name: str):
    card, how = lookup_card(name)
    if not card:
        raise HTTPException(404, f"Carta não encontrada: {name}")
    cdb = get_cards_db()
    pts = cdb.execute(
        "SELECT DISTINCT printed_name, set_code FROM names_pt WHERE oracle_id = ?",
        (card["oracle_id"],),
    ).fetchall()
    cdb.close()
    card["match_type"] = how
    card["pt_names"] = [dict(p) for p in pts]
    return card


@app.post("/api/scan/recognize")
def scan_recognize(payload: dict):
    """Takes text (extracted via OCR in the browser) and returns the closest-matching cards.

    This is NOT visual card recognition — it's a fuzzy text match against the
    38k names (English + official Portuguese) in the local database. It's fast
    and works offline; accuracy depends on the OCR quality on the captured frame.
    """
    text = (payload.get("text") or "").strip()
    if not text:
        return {"candidates": []}
    cdb = get_cards_db()
    words = [w for w in re.split(r"[^a-zA-ZÀ-ÿ]+", text) if len(w) >= 3]
    candidates = {}
    for w in words[:6]:
        like = f"%{w}%"
        rows = cdb.execute(
            """
            SELECT c.name, c.mana_cost, c.type_line, c.image_uri, c.oracle_id,
                   (SELECT printed_name FROM names_pt WHERE oracle_id = c.oracle_id LIMIT 1) as pt_name
            FROM cards c
            WHERE c.name LIKE ? COLLATE NOCASE
               OR c.oracle_id IN (SELECT oracle_id FROM names_pt WHERE printed_name LIKE ? COLLATE NOCASE)
            LIMIT 8
            """,
            (like, like),
        ).fetchall()
        for r in rows:
            candidates[r["oracle_id"]] = card_row_to_dict(r)
    cdb.close()
    return {"candidates": list(candidates.values())[:10], "searched_text": text}


# ----------------------------------------------------------------- decks ----

TYPE_ORDER = ["Land", "Creature", "Planeswalker", "Battle", "Artifact", "Enchantment", "Instant", "Sorcery"]


def classify(type_line: str) -> str:
    if not type_line:
        return "Outro"
    for t in TYPE_ORDER:
        if t in type_line:
            return t
    return "Outro"


@app.get("/api/decks")
def list_decks():
    con = get_app_db()
    decks = con.execute("SELECT * FROM decks ORDER BY id").fetchall()
    cdb = get_cards_db()
    out = []
    for d in decks:
        total = con.execute(
            "SELECT COALESCE(SUM(quantity),0) FROM deck_cards WHERE deck_id = ?", (d["id"],)
        ).fetchone()[0]
        wins = con.execute(
            "SELECT COUNT(*) FROM games WHERE deck_id = ? AND result = 'vitoria'", (d["id"],)
        ).fetchone()[0]
        losses = con.execute(
            "SELECT COUNT(*) FROM games WHERE deck_id = ? AND result = 'derrota'", (d["id"],)
        ).fetchone()[0]
        c = cdb.execute("SELECT image_uri FROM cards WHERE name = ? COLLATE NOCASE", (d["commander_name"],)).fetchone()
        if not c:
            c = cdb.execute("SELECT image_uri FROM cards WHERE name LIKE ? COLLATE NOCASE LIMIT 1", (f"{d['commander_name']}%",)).fetchone()
        commander_image = c["image_uri"] if c else None
        out.append({**dict(d), "total_cards": total, "wins": wins, "losses": losses, "commander_image": commander_image})
    cdb.close()
    con.close()
    return out


@app.get("/api/decks/{deck_id}")
def get_deck(deck_id: int):
    con = get_app_db()
    deck = con.execute("SELECT * FROM decks WHERE id = ?", (deck_id,)).fetchone()
    if not deck:
        con.close()
        raise HTTPException(404, "Deck não encontrado")
    dcards = con.execute(
        "SELECT * FROM deck_cards WHERE deck_id = ? ORDER BY is_commander DESC, card_name", (deck_id,)
    ).fetchall()

    # cards allocated to ANOTHER deck under the same name (cross-deck duplicates)
    other_alloc = con.execute(
        """
        SELECT card_name, decks.name as other_deck, decks.id as other_deck_id
        FROM collection JOIN decks ON decks.id = collection.allocated_deck_id
        WHERE collection.allocated_deck_id != ?
        """,
        (deck_id,),
    ).fetchall()
    other_map = {}
    for r in other_alloc:
        other_map.setdefault(r["card_name"], []).append({"deck": r["other_deck"], "deck_id": r["other_deck_id"]})
    con.close()

    cdb = get_cards_db()
    by_type = {}
    total = 0
    mana_curve = {}
    for dc in dcards:
        r = cdb.execute("SELECT * FROM cards WHERE name = ? COLLATE NOCASE", (dc["card_name"],)).fetchone()
        if not r:
            r = cdb.execute("SELECT * FROM cards WHERE name LIKE ? COLLATE NOCASE LIMIT 1", (f"{dc['card_name']}%",)).fetchone()
        info = card_row_to_dict(r) if r else {"name": dc["card_name"], "type_line": "?", "cmc": 0}
        cat = "Comandante" if dc["is_commander"] else classify(info.get("type_line", ""))
        entry = {
            "card_name": dc["card_name"], "quantity": dc["quantity"], "id": dc["id"],
            "mana_cost": info.get("mana_cost"), "type_line": info.get("type_line"),
            "image_uri": info.get("image_uri"), "cmc": info.get("cmc"),
            "price_usd": info.get("price_usd"), "edhrec_rank": info.get("edhrec_rank"),
            "shared_with": other_map.get(dc["card_name"], []),
        }
        by_type.setdefault(cat, []).append(entry)
        total += dc["quantity"]
        if not dc["is_commander"] and "Land" not in (info.get("type_line") or ""):
            cmc = int(info.get("cmc") or 0)
            mana_curve[cmc] = mana_curve.get(cmc, 0) + dc["quantity"]
    cdb.close()

    return {
        **dict(deck),
        "total_cards": total,
        "is_valid_100": total == 100,
        "by_type": by_type,
        "mana_curve": mana_curve,
    }


class DeckCardIn(BaseModel):
    card_name: str
    quantity: int = 1


@app.post("/api/decks/{deck_id}/cards")
def add_deck_card(deck_id: int, payload: DeckCardIn):
    con = get_app_db()
    deck = con.execute("SELECT * FROM decks WHERE id = ?", (deck_id,)).fetchone()
    if not deck:
        con.close()
        raise HTTPException(404, "Deck não encontrado")
    con.execute(
        "INSERT INTO deck_cards (deck_id, card_name, quantity) VALUES (?, ?, ?)",
        (deck_id, payload.card_name, payload.quantity),
    )
    con.execute(
        "INSERT INTO collection (card_name, lang, quantity, allocated_deck_id, notes) VALUES (?, 'en', ?, ?, 'Adicionado via app')",
        (payload.card_name, payload.quantity, deck_id),
    )
    log_activity(con, "card_added_deck", f"{payload.card_name} entrou no deck {deck['name']}")
    con.commit()
    con.close()
    return {"ok": True}


@app.delete("/api/decks/{deck_id}/cards/{card_id}")
def remove_deck_card(deck_id: int, card_id: int):
    con = get_app_db()
    deck = con.execute("SELECT * FROM decks WHERE id = ?", (deck_id,)).fetchone()
    dc = con.execute("SELECT * FROM deck_cards WHERE id = ? AND deck_id = ?", (card_id, deck_id)).fetchone()
    con.execute("DELETE FROM deck_cards WHERE id = ? AND deck_id = ?", (card_id, deck_id))
    if dc and deck:
        log_activity(con, "card_removed_deck", f"{dc['card_name']} saiu do deck {deck['name']}")
    con.commit()
    con.close()
    return {"ok": True}


@app.get("/api/decks/{deck_id}/synergy")
def deck_synergy(deck_id: int):
    """Cached EDHREC synergy for this deck's commander — 100% offline (local file)."""
    con = get_app_db()
    deck = con.execute("SELECT * FROM decks WHERE id = ?", (deck_id,)).fetchone()
    current_cards = {r["card_name"] for r in con.execute(
        "SELECT card_name FROM deck_cards WHERE deck_id = ?", (deck_id,)
    ).fetchall()}
    con.close()
    if not deck:
        raise HTTPException(404, "Deck não encontrado")

    slug = slugify(deck["commander_name"])
    path = os.path.join(EDHREC_CACHE, "commanders", f"{slug}.json")
    if not os.path.exists(path):
        return {"cached": False, "message": "Sem cache do EDHREC para este comandante. "
                                              f"Rode: ./edhrec.py fetch \"{deck['commander_name']}\""}
    import json
    data = json.load(open(path, encoding="utf-8"))
    cardlists = data.get("container", {}).get("json_dict", {}).get("cardlists", [])
    high_synergy = []
    for cl in cardlists:
        if "High Synergy" in cl.get("header", "") or "Top Cards" in cl.get("header", ""):
            for v in cl.get("cardviews", []):
                if v["name"] not in current_cards:
                    high_synergy.append({
                        "name": v["name"], "synergy": v.get("synergy"), "num_decks": v.get("num_decks"),
                        "already_owned": v["name"] in current_cards,
                    })
    similar = data.get("similar") or []
    return {"cached": True, "recommendations": high_synergy[:15], "similar_commanders": similar}


# --------------------------------------------------------------- collection ----

@app.get("/api/collection")
def list_collection(status: str = "all", q: str = ""):
    """One row per card (grouped) — sums units and lists which decks they're allocated to.

    The same card can have multiple entries in the `collection` table (one per
    copy/deck). This grouping is what matters for "how many units do I actually
    own", as opposed to "how many separate entries exist" — see
    [[Colecao - Inventario Geral]] in the vault.
    """
    con = get_app_db()
    sql = """
        SELECT collection.*, decks.name as deck_name
        FROM collection LEFT JOIN decks ON decks.id = collection.allocated_deck_id
        WHERE 1=1
    """
    params = []
    if status == "free":
        sql += " AND collection.allocated_deck_id IS NULL"
    elif status == "allocated":
        sql += " AND collection.allocated_deck_id IS NOT NULL"
    if q:
        sql += " AND collection.card_name LIKE ? COLLATE NOCASE"
        params.append(f"%{q}%")
    sql += " ORDER BY collection.card_name"
    rows = con.execute(sql, params).fetchall()
    con.close()

    grouped = {}
    for r in rows:
        d = dict(r)
        g = grouped.setdefault(d["card_name"], {
            "card_name": d["card_name"], "total_quantity": 0, "decks": [], "entry_ids": [],
        })
        g["total_quantity"] += d["quantity"]
        g["entry_ids"].append(d["id"])
        g["decks"].append({
            "deck_id": d["allocated_deck_id"], "deck_name": d["deck_name"] or "Livre",
            "quantity": d["quantity"], "lang": d["lang"], "set_code": d["set_code"],
        })

    cdb = get_cards_db()
    out = []
    for card_name, g in grouped.items():
        c = cdb.execute("SELECT type_line, mana_cost, image_uri, colors, rarity, price_usd FROM cards WHERE name = ? COLLATE NOCASE", (card_name,)).fetchone()
        if not c:
            c = cdb.execute("SELECT type_line, mana_cost, image_uri, colors, rarity, price_usd FROM cards WHERE name LIKE ? COLLATE NOCASE LIMIT 1", (f"{card_name}%",)).fetchone()
        if c:
            g.update(dict(c))
        out.append(g)
    cdb.close()
    out.sort(key=lambda x: x["card_name"])
    return out


@app.get("/api/collection/total")
def collection_total():
    """Total units registered in the collection, counting duplicates (allocated + free)."""
    con = get_app_db()
    total = con.execute("SELECT COALESCE(SUM(quantity), 0) FROM collection").fetchone()[0]
    distinct = con.execute("SELECT COUNT(DISTINCT card_name) FROM collection").fetchone()[0]
    con.close()
    return {"total_units": total, "distinct_cards": distinct}


@app.get("/api/activity")
def list_activity(limit: int = 30):
    """App activity log — new cards, decks joined/left, decks built, etc."""
    con = get_app_db()
    rows = con.execute(
        "SELECT * FROM activity ORDER BY ts DESC, id DESC LIMIT ?", (limit,)
    ).fetchall()
    con.close()
    return [dict(r) for r in rows]


@app.get("/api/collection/duplicates")
def collection_duplicates():
    """Cards allocated to 2+ decks at once — 2 owned copies, one per deck."""
    con = get_app_db()
    rows = con.execute(
        """
        SELECT collection.card_name, GROUP_CONCAT(decks.name, ' + ') as decks, COUNT(*) as n
        FROM collection JOIN decks ON decks.id = collection.allocated_deck_id
        GROUP BY collection.card_name
        HAVING n > 1
        ORDER BY collection.card_name
        """
    ).fetchall()
    con.close()
    return [dict(r) for r in rows]


class CollectionIn(BaseModel):
    card_name: str
    set_code: Optional[str] = None
    artist: Optional[str] = None
    lang: str = "en"
    quantity: int = 1
    notes: Optional[str] = None
    deck_id: Optional[int] = None


@app.post("/api/collection")
def add_collection(payload: CollectionIn):
    con = get_app_db()
    cur = con.execute(
        "INSERT INTO collection (card_name, set_code, artist, lang, quantity, notes, allocated_deck_id) VALUES (?,?,?,?,?,?,?)",
        (payload.card_name, payload.set_code, payload.artist, payload.lang, payload.quantity, payload.notes, payload.deck_id),
    )
    qty_label = f"{payload.quantity}x " if payload.quantity != 1 else ""
    if payload.deck_id:
        deck = con.execute("SELECT name FROM decks WHERE id = ?", (payload.deck_id,)).fetchone()
        con.execute(
            "INSERT INTO deck_cards (deck_id, card_name, quantity) VALUES (?, ?, ?)",
            (payload.deck_id, payload.card_name, payload.quantity),
        )
        log_activity(con, "card_new", f"{qty_label}{payload.card_name} adicionada à coleção e ao deck {deck['name'] if deck else '?'}")
    else:
        log_activity(con, "card_new", f"{qty_label}{payload.card_name} adicionada à coleção")
    con.commit()
    entry_id = cur.lastrowid
    con.close()
    return {"ok": True, "id": entry_id}


class AllocateIn(BaseModel):
    deck_id: Optional[int] = None


@app.patch("/api/collection/{entry_id}/allocate")
def allocate_collection(entry_id: int, payload: AllocateIn):
    con = get_app_db()
    entry = con.execute("SELECT * FROM collection WHERE id = ?", (entry_id,)).fetchone()
    con.execute("UPDATE collection SET allocated_deck_id = ? WHERE id = ?", (payload.deck_id, entry_id))
    if entry:
        if payload.deck_id:
            deck = con.execute("SELECT name FROM decks WHERE id = ?", (payload.deck_id,)).fetchone()
            log_activity(con, "card_added_deck", f"{entry['card_name']} alocada ao deck {deck['name'] if deck else '?'}")
        else:
            log_activity(con, "card_removed_deck", f"{entry['card_name']} liberada (ficou fora de deck)")
    con.commit()
    con.close()
    return {"ok": True}


# --------------------------------------------------------------- games ----

class GameIn(BaseModel):
    deck_id: int
    played_at: str
    result: str
    opponents: Optional[str] = None
    turns: Optional[int] = None
    notes: Optional[str] = None
    highlights: List[str] = []


@app.get("/api/games")
def list_games(deck_id: Optional[int] = None):
    con = get_app_db()
    sql = "SELECT games.*, decks.name as deck_name FROM games JOIN decks ON decks.id = games.deck_id"
    params = []
    if deck_id:
        sql += " WHERE games.deck_id = ?"
        params.append(deck_id)
    sql += " ORDER BY games.played_at DESC"
    rows = con.execute(sql, params).fetchall()
    out = []
    for r in rows:
        h = con.execute("SELECT card_name FROM game_highlights WHERE game_id = ?", (r["id"],)).fetchall()
        out.append({**dict(r), "highlights": [x["card_name"] for x in h]})
    con.close()
    return out


@app.post("/api/games")
def add_game(payload: GameIn):
    con = get_app_db()
    cur = con.execute(
        "INSERT INTO games (deck_id, played_at, result, opponents, turns, notes) VALUES (?,?,?,?,?,?)",
        (payload.deck_id, payload.played_at, payload.result, payload.opponents, payload.turns, payload.notes),
    )
    game_id = cur.lastrowid
    for card_name in payload.highlights:
        con.execute("INSERT INTO game_highlights (game_id, card_name) VALUES (?, ?)", (game_id, card_name))
    con.commit()
    con.close()
    return {"ok": True, "id": game_id}


@app.get("/api/games/stats")
def games_stats(deck_id: Optional[int] = None):
    con = get_app_db()
    where = "WHERE deck_id = ?" if deck_id else ""
    params = [deck_id] if deck_id else []
    rows = con.execute(f"SELECT result, COUNT(*) as n FROM games {where} GROUP BY result", params).fetchall()
    tally = {r["result"]: r["n"] for r in rows}

    hl_sql = """
        SELECT game_highlights.card_name, COUNT(*) as n
        FROM game_highlights JOIN games ON games.id = game_highlights.game_id
    """
    if deck_id:
        hl_sql += " WHERE games.deck_id = ?"
    hl_sql += " GROUP BY game_highlights.card_name ORDER BY n DESC LIMIT 10"
    top_cards = con.execute(hl_sql, params).fetchall()
    con.close()
    total = sum(tally.values())
    wins = tally.get("vitoria", 0)
    return {
        "total_games": total,
        "wins": wins,
        "losses": tally.get("derrota", 0),
        "draws": tally.get("empate", 0),
        "win_rate": round(wins / total * 100, 1) if total else None,
        "top_highlight_cards": [dict(r) for r in top_cards],
    }


# ------------------------------------------------------------ static/spa ----

class NoCacheStaticFiles(StaticFiles):
    """Prevents the browser from caching JS/CSS across edits during development."""

    async def get_response(self, path, scope):
        response = await super().get_response(path, scope)
        response.headers["Cache-Control"] = "no-store"
        return response


STATIC_DIR = os.path.join(HERE, "static")
app.mount("/assets", NoCacheStaticFiles(directory=STATIC_DIR), name="assets")


@app.get("/")
def index():
    return FileResponse(os.path.join(STATIC_DIR, "index.html"))
