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

from fastapi import FastAPI, File, HTTPException, UploadFile
from fastapi.staticfiles import StaticFiles
from fastapi.responses import FileResponse, Response
from pydantic import BaseModel

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from db import get_app_db, get_cards_db, init_db, log_activity  # noqa: E402
import data_update  # noqa: E402
import deck_wizard  # noqa: E402
import decklist_import  # noqa: E402

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


def _lookup_card_in(cdb, name: str):
    """Same matching rules as lookup_card(), but reuses an already-open cards
    connection — used by decklist import, which looks up many names in a row."""
    r = cdb.execute("SELECT * FROM cards WHERE name = ? COLLATE NOCASE", (name,)).fetchone()
    if r:
        return card_row_to_dict(r), "exata"
    row = cdb.execute(
        "SELECT DISTINCT oracle_id FROM names_pt WHERE printed_name = ? COLLATE NOCASE", (name,)
    ).fetchall()
    if len(row) == 1:
        r = cdb.execute("SELECT * FROM cards WHERE oracle_id = ?", (row[0]["oracle_id"],)).fetchone()
        if r:
            return card_row_to_dict(r), "exata (nome em português)"
    r = cdb.execute("SELECT * FROM cards WHERE name LIKE ? COLLATE NOCASE LIMIT 1", (f"%{name}%",)).fetchone()
    if r:
        return card_row_to_dict(r), "aproximada"
    return None, None


def lookup_card(name: str):
    """Finds a card by name (exact English, exact Portuguese, or approximate)."""
    cdb = get_cards_db()
    if cdb is None:
        return None, "sem base de cartas — use 'Atualizar base de dados' na Visão Geral"
    result = _lookup_card_in(cdb, name)
    cdb.close()
    return result


# A handful of cards explicitly allow unlimited copies in a Commander deck (basic lands, plus
# cards like Relentless Rats / Persistent Petitioners / Dragon's Approach / Shadowborn Apostle /
# Nazgûl that print "A deck can have any number of cards named ..."). Everything else is capped
# at one copy by the format's singleton rule, so adding a second copy is very likely a mistake —
# see allows_unlimited_copies() / the duplicate-card confirmation in add_deck_card().
BASIC_LAND_NAMES = {"Plains", "Island", "Swamp", "Mountain", "Forest", "Wastes", "Snow-Covered Plains",
                     "Snow-Covered Island", "Snow-Covered Swamp", "Snow-Covered Mountain", "Snow-Covered Forest"}


def allows_unlimited_copies(info, card_name: str) -> bool:
    if card_name in BASIC_LAND_NAMES:
        return True
    text = ((info or {}).get("oracle_text") or "").lower()
    return "a deck can have any number of cards named" in text


# ------------------------------------------------------------------ cards ----

@app.get("/api/cards/search")
def search_cards(q: str = "", limit: int = 30):
    cdb = get_cards_db()
    if cdb is None:
        return []
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
def get_card(name: str, oracle_id: Optional[str] = None):
    """Looks up a card by name, or — when the caller already knows exactly which printing it
    wants (picked from a disambiguation list, e.g. two different cards both named "Phyrexian
    Hydra") — by oracle_id directly, bypassing the ambiguous name match entirely."""
    if oracle_id:
        cdb = get_cards_db()
        if cdb is None:
            raise HTTPException(404, f"Carta não encontrada: {name}")
        r = cdb.execute("SELECT * FROM cards WHERE oracle_id = ?", (oracle_id,)).fetchone()
        card = card_row_to_dict(r) if r else None
        how = "exata (oracle_id)" if card else None
        if not card:
            cdb.close()
            raise HTTPException(404, f"Carta não encontrada: {name}")
    else:
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


@app.get("/api/cards/{name}/variants")
def card_variants(name: str):
    """All distinct cards (by oracle_id) that share this exact printed name — e.g. "Phyrexian
    Hydra" the 5-mana creature and "Phyrexian Hydra" the token it makes. When this returns more
    than one row, the frontend shows a picker instead of silently defaulting to one of them."""
    cdb = get_cards_db()
    if cdb is None:
        return []
    rows = cdb.execute("SELECT * FROM cards WHERE name = ? COLLATE NOCASE", (name,)).fetchall()
    cdb.close()
    return [card_row_to_dict(r) for r in rows]


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
    if cdb is None:
        return {"candidates": [], "searched_text": text}
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


# ------------------------------------------------------------- data update ----

@app.get("/api/data/info")
def data_info():
    """Current state of the local card index — powers the "Atualizar base de dados" card on the Dashboard."""
    cdb = get_cards_db()
    if cdb is None:
        return {"exists": False, "cards": 0, "pt_names": 0, "built_at": None}
    n_cards = cdb.execute("SELECT COUNT(*) FROM cards").fetchone()[0]
    n_pt = cdb.execute("SELECT COUNT(DISTINCT printed_name) FROM names_pt").fetchone()[0]
    cdb.close()
    from db import CARDS_DB
    built_at = os.path.getmtime(CARDS_DB) if os.path.exists(CARDS_DB) else None
    return {"exists": True, "cards": n_cards, "pt_names": n_pt, "built_at": built_at}


@app.post("/api/data/update")
def data_update_start():
    started = data_update.start(refresh_synergy=True)
    if not started:
        raise HTTPException(409, "Uma atualização já está em andamento.")
    return {"ok": True}


@app.get("/api/data/update/status")
def data_update_status():
    return data_update.get_status()


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
        def _card_row(commander_name):
            if cdb is None or not commander_name:
                return None
            c = cdb.execute("SELECT * FROM cards WHERE name = ? COLLATE NOCASE", (commander_name,)).fetchone()
            if not c:
                c = cdb.execute("SELECT * FROM cards WHERE name LIKE ? COLLATE NOCASE LIMIT 1", (f"{commander_name}%",)).fetchone()
            return c

        def _art_crop(row):
            # art_crop shows just the illustration (no card frame) — right fit for a small tile thumbnail
            return row["image_uri"].replace("/normal/", "/art_crop/") if row and row["image_uri"] else None

        commander_2_name = d["commander_name_2"] if "commander_name_2" in d.keys() else None
        row1 = _card_row(d["commander_name"])
        row2 = _card_row(commander_2_name)
        commander_image = _art_crop(row1)
        commander_image_2 = _art_crop(row2)
        # Deck's overall color identity is just the commander(s)' combined identity — standard
        # EDH convention, and cheap to compute here vs. scanning every card in the deck.
        identity_letters = []
        for row in (row1, row2):
            if row and row["color_identity"]:
                identity_letters += list(row["color_identity"])
        color_identity = "".join(dict.fromkeys(l for l in "WUBRG" if l in identity_letters))
        out.append({
            **dict(d), "total_cards": total, "wins": wins, "losses": losses,
            "commander_image": commander_image, "commander_image_2": commander_image_2,
            "color_identity": color_identity,
        })
    if cdb is not None:
        cdb.close()
    con.close()
    return out


def _normalize_tags(raw: Optional[str]) -> Optional[str]:
    """Trims, drops empties, and de-dupes (case-insensitively) a comma-separated tag string —
    shared by create/edit so "Competitivo, competitivo ,, Budget" always lands the same way."""
    if not raw:
        return None
    seen = set()
    out = []
    for t in raw.split(","):
        t = t.strip()
        if t and t.lower() not in seen:
            seen.add(t.lower())
            out.append(t)
    return ", ".join(out) or None


class DeckIn(BaseModel):
    name: str
    commander_name: str
    commander_name_2: Optional[str] = None  # partner / background co-commander, if any
    philosophy: Optional[str] = None
    tags: Optional[str] = None  # comma-separated custom labels, e.g. "Competitivo, Budget"


@app.post("/api/decks")
def create_deck(payload: DeckIn):
    con = get_app_db()
    commander_2 = (payload.commander_name_2 or "").strip() or None
    tags = _normalize_tags(payload.tags)
    cur = con.execute(
        "INSERT INTO decks (name, commander_name, commander_name_2, philosophy, tags) VALUES (?, ?, ?, ?, ?)",
        (payload.name, payload.commander_name, commander_2, payload.philosophy, tags),
    )
    deck_id = cur.lastrowid
    con.execute(
        "INSERT INTO deck_cards (deck_id, card_name, quantity, is_commander) VALUES (?, ?, 1, 1)",
        (deck_id, payload.commander_name),
    )
    if commander_2:
        con.execute(
            "INSERT INTO deck_cards (deck_id, card_name, quantity, is_commander) VALUES (?, ?, 1, 1)",
            (deck_id, commander_2),
        )
    label = payload.commander_name + (f" + {commander_2}" if commander_2 else "")
    log_activity(con, "deck_built", f"Deck {payload.name} criado (comandante: {label})")
    con.commit()
    con.close()
    return {"ok": True, "id": deck_id}


class DeckEditIn(BaseModel):
    name: str
    commander_name: str
    commander_name_2: Optional[str] = None
    philosophy: Optional[str] = None
    tags: Optional[str] = None  # comma-separated custom labels, e.g. "Competitivo, Budget"


@app.put("/api/decks/{deck_id}")
def update_deck(deck_id: int, payload: DeckEditIn):
    """Renames a deck and/or swaps its commander(s) — how the UI lets you add a
    partner commander, replace one, or drop a partner after the fact.

    Reconciles the deck_cards rows flagged is_commander=1 to match the new
    commander set: cards that stop being a commander lose the flag's row
    entirely (unless already added as a normal spell, in which case it's just
    unflagged), and newly-named commanders get a 1-copy row if not already present.
    """
    con = get_app_db()
    deck = con.execute("SELECT * FROM decks WHERE id = ?", (deck_id,)).fetchone()
    if not deck:
        con.close()
        raise HTTPException(404, "Deck não encontrado")

    new_c1 = payload.commander_name.strip()
    new_c2 = (payload.commander_name_2 or "").strip() or None
    if not new_c1:
        con.close()
        raise HTTPException(400, "Comandante principal é obrigatório.")

    old_set = {n for n in (deck["commander_name"], deck["commander_name_2"] if "commander_name_2" in deck.keys() else None) if n}
    new_set = {n for n in (new_c1, new_c2) if n}

    con.execute(
        "UPDATE decks SET name = ?, philosophy = ?, commander_name = ?, commander_name_2 = ?, tags = ? WHERE id = ?",
        (payload.name, payload.philosophy, new_c1, new_c2, _normalize_tags(payload.tags), deck_id),
    )

    for old_name in old_set - new_set:
        con.execute(
            "DELETE FROM deck_cards WHERE deck_id = ? AND card_name = ? COLLATE NOCASE AND is_commander = 1",
            (deck_id, old_name),
        )
    for new_name in new_set - old_set:
        existing = con.execute(
            "SELECT id FROM deck_cards WHERE deck_id = ? AND card_name = ? COLLATE NOCASE",
            (deck_id, new_name),
        ).fetchone()
        if existing:
            con.execute("UPDATE deck_cards SET is_commander = 1 WHERE id = ?", (existing["id"],))
        else:
            con.execute(
                "INSERT INTO deck_cards (deck_id, card_name, quantity, is_commander) VALUES (?, ?, 1, 1)",
                (deck_id, new_name),
            )

    label = new_c1 + (f" + {new_c2}" if new_c2 else "")
    log_activity(con, "deck_built", f"Deck {payload.name} editado (comandante: {label})")
    con.commit()
    con.close()
    return {"ok": True}


class DeckAutoBuildIn(BaseModel):
    name: str
    commander_name: str
    bracket: int = 3
    philosophy: Optional[str] = None


@app.post("/api/decks/auto-build")
def auto_build_deck(payload: DeckAutoBuildIn):
    """Deterministic deckbuilder (no LLM) — see deck_wizard.py. Runs as a background job like data updates."""
    started = deck_wizard.start(payload.name, payload.commander_name, payload.bracket, payload.philosophy)
    if not started:
        raise HTTPException(409, "Uma montagem automática já está em andamento.")
    return {"ok": True}


@app.get("/api/decks/auto-build/status")
def auto_build_status():
    return deck_wizard.get_status()


@app.delete("/api/decks/{deck_id}")
def delete_deck(deck_id: int):
    con = get_app_db()
    deck = con.execute("SELECT * FROM decks WHERE id = ?", (deck_id,)).fetchone()
    if not deck:
        con.close()
        raise HTTPException(404, "Deck não encontrado")
    # deck_cards cascades (ON DELETE CASCADE); collection rows just lose their allocation (ON DELETE SET NULL)
    con.execute("DELETE FROM decks WHERE id = ?", (deck_id,))
    log_activity(con, "deck_disassembled", f"Deck {deck['name']} removido")
    con.commit()
    con.close()
    return {"ok": True}


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
        r = None
        dc_oracle_id = dc["oracle_id"] if "oracle_id" in dc.keys() else None
        if cdb is not None:
            # A specific printing was picked (disambiguated, e.g. one of two cards both named
            # "Phyrexian Hydra") — resolve unambiguously by oracle_id first. Only fall back to
            # the name-based (possibly ambiguous) lookup for older rows that predate this.
            if dc_oracle_id:
                r = cdb.execute("SELECT * FROM cards WHERE oracle_id = ?", (dc_oracle_id,)).fetchone()
            if not r:
                r = cdb.execute("SELECT * FROM cards WHERE name = ? COLLATE NOCASE", (dc["card_name"],)).fetchone()
            if not r:
                r = cdb.execute("SELECT * FROM cards WHERE name LIKE ? COLLATE NOCASE LIMIT 1", (f"{dc['card_name']}%",)).fetchone()
        info = card_row_to_dict(r) if r else {"name": dc["card_name"], "type_line": "?", "cmc": 0}
        cat = "Comandante" if dc["is_commander"] else classify(info.get("type_line", ""))
        entry = {
            "card_name": dc["card_name"], "quantity": dc["quantity"], "id": dc["id"],
            "oracle_id": info.get("oracle_id"),
            "mana_cost": info.get("mana_cost"), "type_line": info.get("type_line"),
            "image_uri": info.get("image_uri"), "cmc": info.get("cmc"),
            "price_usd": info.get("price_usd"), "edhrec_rank": info.get("edhrec_rank"),
            "colors": info.get("colors"), "color_identity": info.get("color_identity"),
            "rarity": info.get("rarity"),
            "shared_with": other_map.get(dc["card_name"], []),
        }
        by_type.setdefault(cat, []).append(entry)
        total += dc["quantity"]
        if not dc["is_commander"] and "Land" not in (info.get("type_line") or ""):
            cmc = int(info.get("cmc") or 0)
            mana_curve[cmc] = mana_curve.get(cmc, 0) + dc["quantity"]
    if cdb is not None:
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
    oracle_id: Optional[str] = None  # picked from a disambiguation list, when the name is shared by 2+ cards
    confirm: bool = False  # set after the user confirms adding a card that's already in the deck


@app.post("/api/decks/{deck_id}/cards")
def add_deck_card(deck_id: int, payload: DeckCardIn):
    con = get_app_db()
    deck = con.execute("SELECT * FROM decks WHERE id = ?", (deck_id,)).fetchone()
    if not deck:
        con.close()
        raise HTTPException(404, "Deck não encontrado")

    cdb = get_cards_db()
    info = None
    if cdb is not None:
        if payload.oracle_id:
            r = cdb.execute("SELECT * FROM cards WHERE oracle_id = ?", (payload.oracle_id,)).fetchone()
        else:
            r = cdb.execute("SELECT * FROM cards WHERE name = ? COLLATE NOCASE", (payload.card_name,)).fetchone()
        info = card_row_to_dict(r) if r else None
        cdb.close()

    # Commander is singleton — only basic lands and a handful of explicitly-unlimited cards
    # (Relentless Rats, Shadowborn Apostle, etc.) can have more than 1 copy. Anything else
    # already in the deck needs an explicit confirm before a second copy is added.
    if not allows_unlimited_copies(info, payload.card_name) and not payload.confirm:
        existing_qty = con.execute(
            "SELECT COALESCE(SUM(quantity),0) FROM deck_cards WHERE deck_id = ? AND card_name = ? COLLATE NOCASE",
            (deck_id, payload.card_name),
        ).fetchone()[0]
        if existing_qty:
            con.close()
            raise HTTPException(409, {
                "needs_confirmation": True,
                "card_name": payload.card_name,
                "existing_quantity": existing_qty,
            })

    # Merge into an existing row for the same (name, oracle_id) pair rather than always
    # inserting a fresh one — otherwise adding "Swamp" one at a time via the search box builds
    # up a pile of separate 1x rows instead of a single row whose quantity actually adds up.
    # oracle_id is matched with IS (not =) so two NULLs — i.e. two adds where no specific
    # printing was picked — still count as the same card, while a disambiguated printing only
    # merges with rows that picked that exact same printing.
    existing_row = con.execute(
        "SELECT id FROM deck_cards WHERE deck_id = ? AND card_name = ? COLLATE NOCASE AND is_commander = 0 AND oracle_id IS ?",
        (deck_id, payload.card_name, payload.oracle_id),
    ).fetchone()
    if existing_row:
        con.execute("UPDATE deck_cards SET quantity = quantity + ? WHERE id = ?", (payload.quantity, existing_row["id"]))
    else:
        con.execute(
            "INSERT INTO deck_cards (deck_id, card_name, quantity, oracle_id) VALUES (?, ?, ?, ?)",
            (deck_id, payload.card_name, payload.quantity, payload.oracle_id),
        )
    con.execute(
        "INSERT INTO collection (card_name, lang, quantity, allocated_deck_id, oracle_id, notes) VALUES (?, 'en', ?, ?, ?, 'Adicionado via app')",
        (payload.card_name, payload.quantity, deck_id, payload.oracle_id),
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


# -------------------------------------------------------------- deck export ----
# Downloadable decklist, in either of two plain-text shapes:
#   "text"     — bare "qty name" per line, no zones/headers. The most universally-accepted
#                shape (also exactly what this app's own decklist_import.py reads back in).
#   "moxfield" — same, but with "Commander (n)" / "Deck (n)" section headers, which Moxfield's
#                paste-to-import recognizes (as does decklist_import.SECTION_HEADER_RE here).
# No card lookups needed — this only ever reads what's already stored in deck_cards, so it
# works even without the card index built.

@app.get("/api/decks/{deck_id}/export")
def export_deck(deck_id: int, format: str = "text"):
    con = get_app_db()
    deck = con.execute("SELECT * FROM decks WHERE id = ?", (deck_id,)).fetchone()
    if not deck:
        con.close()
        raise HTTPException(404, "Deck não encontrado")
    cards = con.execute(
        "SELECT * FROM deck_cards WHERE deck_id = ? ORDER BY is_commander DESC, card_name COLLATE NOCASE",
        (deck_id,),
    ).fetchall()
    con.close()

    commanders = [c for c in cards if c["is_commander"]]
    others = [c for c in cards if not c["is_commander"]]

    def card_line(c):
        return f"{c['quantity']} {c['card_name']}"

    if format == "moxfield":
        lines = []
        if commanders:
            lines.append(f"Commander ({sum(c['quantity'] for c in commanders)})")
            lines += [card_line(c) for c in commanders]
            lines.append("")
        lines.append(f"Deck ({sum(c['quantity'] for c in others)})")
        lines += [card_line(c) for c in others]
        body = "\n".join(lines).strip("\n") + "\n"
        suffix = "-moxfield"
    else:
        lines = [card_line(c) for c in commanders + others]
        body = "\n".join(lines) + "\n"
        suffix = ""

    filename = f"{slugify(deck['name']) or 'deck'}{suffix}.txt"
    return Response(
        content=body,
        media_type="text/plain; charset=utf-8",
        headers={"Content-Disposition": f'attachment; filename="{filename}"'},
    )


# ---------------------------------------------------------- decklist import ----
# Paste from Moxfield/Archidekt/plain text, or upload a .txt/.csv/.xlsx file.
# Two-step flow: /import/preview(-file) parses + matches against the local
# index without writing anything; /import/commit writes only what the user
# confirmed. Never silently adds a card the parser merely guessed at.

def _match_entries(entries):
    cdb = get_cards_db()
    matched, not_found = [], []
    seen = {}
    for qty, name in entries:
        key = name.lower()
        if key in seen:
            seen[key]["quantity"] += qty
            continue
        if cdb is None:
            item = {"requested_name": name, "quantity": qty}
            not_found.append(item)
            seen[key] = item
            continue
        card, how = _lookup_card_in(cdb, name)
        if card:
            item = {
                "name": card["name"], "quantity": qty, "requested_name": name, "match_type": how,
                "mana_cost": card.get("mana_cost"), "type_line": card.get("type_line"),
                "image_uri": card.get("image_uri"),
            }
            matched.append(item)
        else:
            item = {"requested_name": name, "quantity": qty}
            not_found.append(item)
        seen[key] = item
    if cdb is not None:
        cdb.close()
    return {"matched": matched, "not_found": not_found, "total_lines": len(entries)}


class ImportPreviewIn(BaseModel):
    text: str


@app.post("/api/decks/{deck_id}/import/preview")
def import_preview(deck_id: int, payload: ImportPreviewIn):
    con = get_app_db()
    deck = con.execute("SELECT id FROM decks WHERE id = ?", (deck_id,)).fetchone()
    con.close()
    if not deck:
        raise HTTPException(404, "Deck não encontrado")
    entries = decklist_import.parse_text(payload.text)
    if not entries:
        raise HTTPException(400, "Não encontrei nenhuma carta reconhecível no texto colado.")
    return _match_entries(entries)


@app.post("/api/decks/{deck_id}/import/preview-file")
async def import_preview_file(deck_id: int, file: UploadFile = File(...)):
    con = get_app_db()
    deck = con.execute("SELECT id FROM decks WHERE id = ?", (deck_id,)).fetchone()
    con.close()
    if not deck:
        raise HTTPException(404, "Deck não encontrado")
    raw = await file.read()
    fname = (file.filename or "").lower()
    try:
        if fname.endswith(".xlsx"):
            entries = decklist_import.parse_xlsx(raw)
        elif fname.endswith(".csv"):
            entries = decklist_import.parse_csv(raw.decode("utf-8", errors="replace"))
        else:
            entries = decklist_import.parse_text(raw.decode("utf-8", errors="replace"))
    except RuntimeError as e:
        raise HTTPException(400, str(e))
    if not entries:
        raise HTTPException(400, "Não encontrei nenhuma carta reconhecível no arquivo.")
    return _match_entries(entries)


class ImportCommitIn(BaseModel):
    cards: List[DeckCardIn]
    mode: str = "merge"  # "merge" adds/increments; "replace" clears non-commander cards first


@app.post("/api/decks/{deck_id}/import/commit")
def import_commit(deck_id: int, payload: ImportCommitIn):
    con = get_app_db()
    deck = con.execute("SELECT * FROM decks WHERE id = ?", (deck_id,)).fetchone()
    if not deck:
        con.close()
        raise HTTPException(404, "Deck não encontrado")

    if payload.mode == "replace":
        con.execute("DELETE FROM deck_cards WHERE deck_id = ? AND is_commander = 0", (deck_id,))

    added = 0
    for card in payload.cards:
        if not card.card_name.strip():
            continue
        existing = con.execute(
            "SELECT id, quantity FROM deck_cards WHERE deck_id = ? AND card_name = ? COLLATE NOCASE AND is_commander = 0",
            (deck_id, card.card_name),
        ).fetchone()
        if existing:
            con.execute("UPDATE deck_cards SET quantity = quantity + ? WHERE id = ?", (card.quantity, existing["id"]))
        else:
            con.execute(
                "INSERT INTO deck_cards (deck_id, card_name, quantity, is_commander) VALUES (?, ?, ?, 0)",
                (deck_id, card.card_name, card.quantity),
            )
        con.execute(
            "INSERT INTO collection (card_name, lang, quantity, allocated_deck_id, notes) VALUES (?, 'en', ?, ?, 'Importado via decklist')",
            (card.card_name, card.quantity, deck_id),
        )
        added += card.quantity

    mode_label = "substituindo cartas existentes" if payload.mode == "replace" else "mesclado com o deck atual"
    log_activity(con, "card_added_deck", f"{added} carta(s) importada(s) para o deck {deck['name']} ({mode_label})")
    con.commit()
    con.close()
    return {"ok": True, "added": added}


def _deck_commanders(deck):
    names = [deck["commander_name"]]
    c2 = deck["commander_name_2"] if "commander_name_2" in deck.keys() else None
    if c2:
        names.append(c2)
    return names


@app.get("/api/decks/{deck_id}/synergy")
def deck_synergy(deck_id: int):
    """Cached EDHREC synergy for this deck's commander(s) — 100% offline (local file).

    With a partner/background pair, synergy from both commanders' cached pages
    is merged (best synergy score kept per card) so recommendations reflect
    the whole command zone, not just the first-listed commander.
    """
    con = get_app_db()
    deck = con.execute("SELECT * FROM decks WHERE id = ?", (deck_id,)).fetchone()
    current_cards = {r["card_name"] for r in con.execute(
        "SELECT card_name FROM deck_cards WHERE deck_id = ?", (deck_id,)
    ).fetchall()}
    con.close()
    if not deck:
        raise HTTPException(404, "Deck não encontrado")

    import json
    commanders = _deck_commanders(deck)
    missing = []
    merged = {}
    similar_all = []
    for name in commanders:
        slug = slugify(name)
        path = os.path.join(EDHREC_CACHE, "commanders", f"{slug}.json")
        if not os.path.exists(path):
            missing.append(name)
            continue
        data = json.load(open(path, encoding="utf-8"))
        cardlists = data.get("container", {}).get("json_dict", {}).get("cardlists", [])
        for cl in cardlists:
            if "High Synergy" in cl.get("header", "") or "Top Cards" in cl.get("header", ""):
                for v in cl.get("cardviews", []):
                    if v["name"] in current_cards:
                        continue
                    existing = merged.get(v["name"])
                    if existing is None or (v.get("synergy") or 0) > (existing.get("synergy") or 0):
                        merged[v["name"]] = {
                            "name": v["name"], "synergy": v.get("synergy"), "num_decks": v.get("num_decks"),
                            "already_owned": False,
                        }
        similar_all.extend(data.get("similar") or [])

    if len(missing) == len(commanders):
        rode = " e ".join(f'./edhrec.py fetch "{n}"' for n in missing)
        return {"cached": False, "message": f"Sem cache do EDHREC para {' e '.join(missing)}. Rode: {rode}"}

    high_synergy = sorted(merged.values(), key=lambda x: -(x["synergy"] or -999))
    similar = [s for s in dict.fromkeys(similar_all) if s not in commanders]
    return {
        "cached": True, "recommendations": high_synergy[:15], "similar_commanders": similar[:8],
        "missing_cache_for": missing,
    }


@app.post("/api/decks/{deck_id}/synergy/fetch")
def fetch_deck_synergy(deck_id: int):
    """Fetches EDHREC synergy for this deck's commander(s) — lightweight alternative to a full data update."""
    con = get_app_db()
    deck = con.execute("SELECT * FROM decks WHERE id = ?", (deck_id,)).fetchone()
    con.close()
    if not deck:
        raise HTTPException(404, "Deck não encontrado")
    errors = []
    for name in _deck_commanders(deck):
        ok, err = data_update.fetch_one_commander(name, with_combos=True)
        if not ok:
            errors.append(f"{name}: {err}")
    if errors:
        raise HTTPException(502, f"Não foi possível buscar no EDHREC: {'; '.join(errors)}")
    return {"ok": True}


# EDHREC cardlist headers that are too generic to work as a "tag" for browsing — every card in
# a High Synergy/Top Cards/New Cards list, plus these existing ones, would just end up bucketed
# together, defeating the point of grouping by theme.
GENERIC_TAG_HEADERS = {"top cards", "new cards", "high synergy cards"}


@app.get("/api/decks/{deck_id}/tags")
def deck_card_tags(deck_id: int):
    """Contextual tags for the cards actually in this deck, sourced from the EDHREC cardlist
    headers on the deck's commander(s) — e.g. "Removal", "Ramp", "Recursion". 100% offline,
    reads the same cached commander JSON as /synergy. Lets the deck page group cards by theme
    instead of just by card type.
    """
    con = get_app_db()
    deck = con.execute("SELECT * FROM decks WHERE id = ?", (deck_id,)).fetchone()
    if not deck:
        con.close()
        raise HTTPException(404, "Deck não encontrado")
    deck_card_names = {r["card_name"] for r in con.execute(
        "SELECT card_name FROM deck_cards WHERE deck_id = ?", (deck_id,)
    ).fetchall()}
    con.close()

    import json
    tag_map = {}
    any_cached = False
    for name in _deck_commanders(deck):
        slug = slugify(name)
        path = os.path.join(EDHREC_CACHE, "commanders", f"{slug}.json")
        if not os.path.exists(path):
            continue
        any_cached = True
        data = json.load(open(path, encoding="utf-8"))
        cardlists = data.get("container", {}).get("json_dict", {}).get("cardlists", [])
        for cl in cardlists:
            header = (cl.get("header") or "").strip()
            if not header or header.lower() in GENERIC_TAG_HEADERS:
                continue
            for v in cl.get("cardviews", []):
                cname = v.get("name")
                if cname in deck_card_names:
                    tag_map.setdefault(cname, set()).add(header)

    return {"cached": any_cached, "tags": {name: sorted(tags) for name, tags in tag_map.items()}}


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
        if cdb is not None:
            c = cdb.execute("SELECT type_line, mana_cost, image_uri, colors, rarity, price_usd FROM cards WHERE name = ? COLLATE NOCASE", (card_name,)).fetchone()
            if not c:
                c = cdb.execute("SELECT type_line, mana_cost, image_uri, colors, rarity, price_usd FROM cards WHERE name LIKE ? COLLATE NOCASE LIMIT 1", (f"{card_name}%",)).fetchone()
            if c:
                g.update(dict(c))
        out.append(g)
    if cdb is not None:
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


# ---------------------------------------------------------- collection: add by list ----
# "Adicionar por lista" mode of the add-card dialog — a fast path for registering many
# cards at once, always 1 unit each, edition-agnostic. Reuses decklist_import.parse_text()
# for the same tolerant line parsing the deck-import feature uses (quantity prefixes,
# trailing set/collector annotations, section headers), but deliberately ignores any
# parsed quantity and never merges duplicate lines — each line is resolved and reported
# on its own, 1:1, so the frontend can point at exactly which line failed.

class BulkResolveIn(BaseModel):
    text: str


@app.post("/api/collection/bulk-resolve")
def bulk_resolve_collection(payload: BulkResolveIn):
    """Parses + resolves each line against the local index. Writes nothing — the frontend
    commits each resolved line via the existing POST /api/collection, same as a single add."""
    entries = decklist_import.parse_text(payload.text)
    cdb = get_cards_db()
    out = []
    for _qty, name in entries:
        if cdb is None:
            out.append({"input": name, "card_name": None, "lang": None, "oracle_id": None})
            continue
        card, how = _lookup_card_in(cdb, name)
        if card:
            lang = "pt" if how and "português" in how else "en"
            out.append({"input": name, "card_name": card["name"], "lang": lang, "oracle_id": card["oracle_id"]})
        else:
            out.append({"input": name, "card_name": None, "lang": None, "oracle_id": None})
    if cdb is not None:
        cdb.close()
    return out


class CollectionIn(BaseModel):
    card_name: str
    set_code: Optional[str] = None
    artist: Optional[str] = None
    lang: str = "en"
    quantity: int = 1
    notes: Optional[str] = None
    deck_id: Optional[int] = None
    oracle_id: Optional[str] = None  # picked from a disambiguation list, when the name is shared by 2+ cards


@app.post("/api/collection")
def add_collection(payload: CollectionIn):
    con = get_app_db()
    cur = con.execute(
        "INSERT INTO collection (card_name, set_code, artist, lang, quantity, notes, allocated_deck_id, oracle_id) VALUES (?,?,?,?,?,?,?,?)",
        (payload.card_name, payload.set_code, payload.artist, payload.lang, payload.quantity, payload.notes, payload.deck_id, payload.oracle_id),
    )
    qty_label = f"{payload.quantity}x " if payload.quantity != 1 else ""
    if payload.deck_id:
        deck = con.execute("SELECT name FROM decks WHERE id = ?", (payload.deck_id,)).fetchone()
        con.execute(
            "INSERT INTO deck_cards (deck_id, card_name, quantity, oracle_id) VALUES (?, ?, ?, ?)",
            (payload.deck_id, payload.card_name, payload.quantity, payload.oracle_id),
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
