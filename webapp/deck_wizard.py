"""
Deterministic Commander deckbuilder — no LLM involved, by explicit choice
(see the "Novo Deck" auto-build toggle). Every card in the result was read
straight out of the local Scryfall index; nothing is generated or guessed.

Three inputs, all already used elsewhere in this project:
  1. Ratio targets and bracket rules from knowledge/deckbuilding_guide.json
     (itself a structured version of MTG/Deckbuilding - Guia e Proporcoes.md
     in the vault).
  2. The local Scryfall card index (db.get_cards_db()) — filtered to the
     commander's color identity and commander legality, then bucketed into
     the guide's "skeleton" categories (ramp/draw/removal/wipes/protection)
     by oracle_text/type_line heuristics. This is where the real, official
     `game_changer` field (see mtgdb.py) and the bracket rules gate what's
     eligible.
  3. Cached EDHREC synergy for the commander (fetched on demand via
     data_update.fetch_one_commander if missing) — fills everything past
     the skeleton with the commander's actual signature payoffs/wincons.

Runs as a background job (same pattern as data_update.py) since step 3 can
hit the network; progress is exposed via get_status() for the frontend.
"""
import json
import os
import re
import sys
import threading
import time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
sys.path.insert(0, ROOT)

import edhrec  # noqa: E402
import data_update  # noqa: E402

from db import get_app_db, get_cards_db, log_activity  # noqa: E402

GUIDE_PATH = os.path.join(HERE, "knowledge", "deckbuilding_guide.json")
_guide_cache = None


def load_guide():
    global _guide_cache
    if _guide_cache is None:
        with open(GUIDE_PATH, encoding="utf-8") as fh:
            _guide_cache = json.load(fh)
    return _guide_cache


# Fixed slot targets (midpoints of the guide's ranges) — sum to 99 nonland+land
# slots; +1 commander = 100. "synergy_fill" absorbs whatever's left, which is
# also where win conditions land (EDHREC synergy tends to surface those anyway).
TARGETS = {
    "lands": 37, "ramp": 11, "draw": 10, "removal_spot": 6,
    "board_wipes": 3, "protection": 4,
}
TARGETS["synergy_fill"] = 99 - sum(TARGETS.values())

GAME_CHANGER_BUDGET = {1: 0, 2: 0, 3: 3, 4: 10**6, 5: 10**6}
ALLOW_MASS_LAND_DENIAL = {1: False, 2: False, 3: False, 4: True, 5: True}
BASIC_LAND_NAMES = {"W": "Plains", "U": "Island", "B": "Swamp", "R": "Mountain", "G": "Forest"}

RAMP_RE = re.compile(r"\badd \{|\badd one mana|\badd mana|search your library for a.*land.*battlefield", re.I)
DRAW_RE = re.compile(r"draws? (a|two|three|four|x) cards?", re.I)
REMOVAL_SPOT_RE = re.compile(r"destroy target|exile target|target player sacrifices|-\d+/-\d+ until end of turn", re.I)
WIPE_RE = re.compile(r"destroy all creatures|exile all creatures|deals? \d+ damage to each creature|all creatures get -\d+/-\d+", re.I)
PROTECTION_RE = re.compile(r"hexproof|indestructible|protection from|counter target spell", re.I)
MLD_RE = re.compile(r"destroy all lands|sacrifices? (a|an|\d+|all)? ?lands?\b.*(all|each)|nonbasic lands? .* destroy", re.I)


class DeckWizardError(ValueError):
    """User-facing error (bad commander name, no card index yet, etc.)."""


def _color_subset(card_ci, commander_ci):
    return all(c in commander_ci for c in (card_ci or ""))


def _classify(card):
    type_line = card.get("type_line") or ""
    text = card.get("oracle_text") or ""
    if "Land" in type_line:
        return "land"
    if RAMP_RE.search(text):
        return "ramp"
    if WIPE_RE.search(text):
        return "board_wipes"
    if REMOVAL_SPOT_RE.search(text):
        return "removal_spot"
    if PROTECTION_RE.search(text):
        return "protection"
    if DRAW_RE.search(text):
        return "draw"
    return "other"


def _sort_key(card, synergy_map):
    syn = synergy_map.get(card["name"].lower())
    rank = card.get("edhrec_rank")
    return (-(syn if syn is not None else -999), rank if rank is not None else 10 ** 9)


def _load_synergy_map(commander_name, on_fetch_needed=None):
    """Returns {lowercased card name: best synergy score seen}. Fetches the EDHREC cache on demand if missing."""
    slug = edhrec.slugify(commander_name)
    cache_path = edhrec._cache_path("commanders", slug)
    if not os.path.exists(cache_path):
        if on_fetch_needed:
            on_fetch_needed()
        data_update.fetch_one_commander(commander_name, with_combos=False)
    if not os.path.exists(cache_path):
        return {}
    with open(cache_path, encoding="utf-8") as fh:
        edh_data = json.load(fh)
    synergy_map = {}
    for cl in edh_data.get("container", {}).get("json_dict", {}).get("cardlists", []):
        for v in cl.get("cardviews", []):
            nm = v["name"].lower()
            syn = v.get("synergy") or 0
            if nm not in synergy_map or syn > synergy_map[nm]:
                synergy_map[nm] = syn
    return synergy_map


def build_deck_list(commander_name, bracket=3, on_fetch_needed=None):
    """Pure-ish function (only network side effect: on-demand EDHREC fetch).

    Returns (chosen_cards: list[dict], lands: dict[name -> qty], meta: dict).
    Raises DeckWizardError for anything the user needs to fix (bad commander name, no card index).
    """
    guide = load_guide()  # noqa: F841 — loaded for future use (bracket descriptions, etc.); ratios above already mirror it
    bracket = bracket if bracket in GAME_CHANGER_BUDGET else 3

    cdb = get_cards_db()
    if cdb is None:
        raise DeckWizardError("Base de cartas ainda não construída — use 'Atualizar base de dados' na Visão Geral primeiro.")

    commander = cdb.execute("SELECT * FROM cards WHERE name = ? COLLATE NOCASE", (commander_name,)).fetchone()
    if not commander:
        cdb.close()
        raise DeckWizardError(f"Comandante não encontrado na base local: {commander_name}")
    commander = dict(commander)
    type_line = commander.get("type_line") or ""
    if "Legendary" not in type_line or not ("Creature" in type_line or "Planeswalker" in type_line):
        cdb.close()
        raise DeckWizardError(f"'{commander['name']}' não parece ser uma carta lendária elegível para comandante.")
    commander_ci = commander.get("color_identity") or ""

    rows = cdb.execute("SELECT * FROM cards WHERE commander_legal='legal'").fetchall()
    pool = [dict(r) for r in rows if r["name"] != commander["name"] and _color_subset(r["color_identity"] or "", commander_ci)]
    cdb.close()

    synergy_map = _load_synergy_map(commander["name"], on_fetch_needed)

    buckets = {"ramp": [], "draw": [], "removal_spot": [], "board_wipes": [], "protection": [], "other": []}
    for card in pool:
        cat = _classify(card)
        if cat == "land":
            continue  # basics are handled separately below (v1 scope: no curated nonbasic lands)
        buckets[cat].append(card)
    for key in buckets:
        buckets[key].sort(key=lambda c: _sort_key(c, synergy_map))

    gc_budget = [GAME_CHANGER_BUDGET.get(bracket, 3)]
    allow_mld = ALLOW_MASS_LAND_DENIAL.get(bracket, False)
    chosen, chosen_names = [], set()

    def _eligible(card):
        if card["name"] in chosen_names:
            return False
        if not allow_mld and MLD_RE.search(card.get("oracle_text") or ""):
            return False
        if card.get("game_changer"):
            if gc_budget[0] <= 0:
                return False
        return True

    def _accept(card):
        if card.get("game_changer"):
            gc_budget[0] -= 1
        chosen.append(card)
        chosen_names.add(card["name"])

    def _take(bucket_key, n):
        taken = 0
        for card in buckets[bucket_key]:
            if taken >= n:
                break
            if _eligible(card):
                _accept(card)
                taken += 1
        return taken

    _take("ramp", TARGETS["ramp"])
    _take("draw", TARGETS["draw"])
    _take("removal_spot", TARGETS["removal_spot"])
    _take("board_wipes", TARGETS["board_wipes"])
    _take("protection", TARGETS["protection"])

    # EDHREC-driven fill — the commander's actual signature payoffs/wincons
    synergy_pool = [c for c in pool if c["name"].lower() in synergy_map and "Land" not in (c.get("type_line") or "")]
    synergy_pool.sort(key=lambda c: _sort_key(c, synergy_map))
    taken = 0
    for card in synergy_pool:
        if taken >= TARGETS["synergy_fill"]:
            break
        if _eligible(card):
            _accept(card)
            taken += 1

    # backfill (narrow color identity, or a commander with a thin/no EDHREC page)
    total_nonland_target = sum(v for k, v in TARGETS.items() if k != "lands")
    generic_pool = sorted(
        (c for c in pool if "Land" not in (c.get("type_line") or "")),
        key=lambda c: _sort_key(c, synergy_map),
    )
    for card in generic_pool:
        if len(chosen) >= total_nonland_target:
            break
        if _eligible(card):
            _accept(card)

    lands = _build_manabase(commander_ci, chosen, TARGETS["lands"])

    meta = {
        "commander": commander["name"],
        "color_identity": commander_ci,
        "bracket": bracket,
        "game_changers_used": sum(1 for c in chosen if c.get("game_changer")),
        "synergy_cache_used": bool(synergy_map),
        "nonland_count": len(chosen),
        "land_count": sum(lands.values()),
    }
    return chosen, lands, meta


def _build_manabase(commander_ci, chosen, land_count):
    """Basic lands only (v1), split proportionally by colored-pip weight of the chosen spells."""
    if not commander_ci:
        return {"Wastes": land_count}
    pip_counts = {c: 0 for c in commander_ci}
    for card in chosen:
        cost = card.get("mana_cost") or ""
        for c in commander_ci:
            pip_counts[c] += cost.count("{" + c + "}")
    total_pips = sum(pip_counts.values())
    lands = {}
    if total_pips == 0:
        per = land_count // len(commander_ci)
        for c in commander_ci:
            lands[BASIC_LAND_NAMES[c]] = per
        lands[BASIC_LAND_NAMES[commander_ci[0]]] += land_count - per * len(commander_ci)
        return lands
    assigned = 0
    for c in commander_ci:
        share = round(land_count * pip_counts[c] / total_pips)
        lands[BASIC_LAND_NAMES[c]] = share
        assigned += share
    lands[BASIC_LAND_NAMES[commander_ci[0]]] += land_count - assigned
    return lands


def save_deck(name, commander_name, bracket, chosen, lands, philosophy=None):
    """Writes the built list to app.db exactly like a manually-created deck — same tables, same shape."""
    if not philosophy:
        philosophy = (
            f"Montado automaticamente (bracket {bracket}) a partir da base local do Scryfall, "
            f"sinergia do EDHREC e das proporções em [[Deckbuilding - Guia e Proporcoes]]."
        )
    con = get_app_db()
    cur = con.execute(
        "INSERT INTO decks (name, commander_name, philosophy) VALUES (?, ?, ?)",
        (name, commander_name, philosophy),
    )
    deck_id = cur.lastrowid
    con.execute(
        "INSERT INTO deck_cards (deck_id, card_name, quantity, is_commander) VALUES (?, ?, 1, 1)",
        (deck_id, commander_name),
    )
    for card in chosen:
        con.execute(
            "INSERT INTO deck_cards (deck_id, card_name, quantity, is_commander) VALUES (?, ?, 1, 0)",
            (deck_id, card["name"]),
        )
    for land_name, qty in lands.items():
        if qty > 0:
            con.execute(
                "INSERT INTO deck_cards (deck_id, card_name, quantity, is_commander) VALUES (?, ?, ?, 0)",
                (deck_id, land_name, qty),
            )
    log_activity(con, "deck_built", f"Deck {name} montado automaticamente (bracket {bracket}, comandante: {commander_name})")
    con.commit()
    con.close()
    return deck_id


# --------------------------------------------------------------- background job ----

_status = {
    "state": "idle",  # idle | running | done | error
    "task": None,
    "percent": 0,
    "error": None,
    "result": None,  # {"deck_id": int, "meta": {...}} on success
}
_lock = threading.Lock()


def get_status():
    with _lock:
        return dict(_status)


def is_running():
    with _lock:
        return _status["state"] == "running"


def _progress(task, percent):
    with _lock:
        _status["task"] = task
        _status["percent"] = max(0, min(100, round(percent, 1)))


def _run(name, commander_name, bracket, philosophy):
    try:
        with _lock:
            _status["state"] = "running"
            _status["task"] = None
            _status["percent"] = 0
            _status["error"] = None
            _status["result"] = None

        _progress("Lendo a base local de cartas…", 10)

        def on_fetch_needed():
            _progress(f"Buscando sinergia no EDHREC: {commander_name}…", 30)

        chosen, lands, meta = build_deck_list(commander_name, bracket, on_fetch_needed=on_fetch_needed)
        _progress("Selecionando cartas por categoria…", 70)
        _progress("Montando terrenos…", 85)
        deck_id = save_deck(name, meta["commander"], bracket, chosen, lands, philosophy)
        _progress("Concluído.", 100)
        with _lock:
            _status["state"] = "done"
            _status["result"] = {"deck_id": deck_id, "meta": meta}
    except DeckWizardError as e:
        with _lock:
            _status["state"] = "error"
            _status["error"] = str(e)
    except BaseException as e:
        with _lock:
            _status["state"] = "error"
            _status["error"] = f"Erro inesperado: {e}"


def start(name, commander_name, bracket=3, philosophy=None):
    if is_running():
        return False
    threading.Thread(target=_run, args=(name, commander_name, bracket, philosophy), daemon=True).start()
    return True
