# Spellbook — MTG Collection & Deck Manager

Local app for managing a Magic: The Gathering collection and Commander decks — any commander, any colors, any player. Runs 100% offline after initial setup — no Node.js, just Python (FastAPI + SQLite) and a static HTML/CSS/JS frontend.

Everything is created from the UI: use "**Novo Deck**" (Dashboard/Meus Decks) to start a deck for any commander — optionally toggle "**Montar automaticamente**" to have it filled out deterministically (lands/ramp/draw/removal ratios + EDHREC synergy for that commander — no LLM involved, every card is verified against the local index before being added) — and "**+ Adicionar Carta**" (Dashboard/Coleção) to add cards to your collection, one at a time or by pasting a list of names. No editing Python files or seed data required — those only exist to load the original author's own decks as a worked example.

Full design-decision documentation lives in the Obsidian vault: `MTG/App - Protótipo Spellbook.md` (in Portuguese — that's the owner's personal knowledge base).

## Why this repository doesn't include the data

- `data/` (Scryfall bulk data + EDHREC cache) is ~430 MB — over GitHub's file size limit, and fully reconstructible from Scryfall's public API.
- `webapp/app.db` is the real collection/decks — personal data, stays local only.

Both are in `.gitignore`. To run on any machine, rebuild both with the steps below.

## Setup from scratch

```bash
# 1. Python dependencies (no Node.js needed)
pip3 install --user -r requirements.txt

# 2. run (starts with an empty database — no card index, no decks, no collection)
cd webapp
python3 -m uvicorn server:app --port 8420
```

Requires Python 3.9 or newer.

Visit **http://127.0.0.1:8420**. The sidebar shows a "**Baixar base de dados agora**" panel — click it to download Scryfall's card database (official bulk data, ~400 MB, no API key) and build the local index in one step; needs internet only for this. After that, use "Novo Deck" / "+ Adicionar Carta" to build your own decks and collection from the UI, 100% offline.

Schema changes apply themselves on startup — if you're upgrading an install from before decklist import / partner commanders / card disambiguation existed, the new columns are added automatically the next time `server.py` starts (see `MIGRATIONS` in `webapp/db.py`). A card index built by an older `mtgdb.py` also gets any missing index added in place (see `ensure_cards_indexes()`), so no ~400 MB re-download is needed just to pick up a performance fix.

## Keeping the card data current

When a new set/block releases, click "**Atualizar base de dados**" (sidebar, visible on every page) any time to re-download and reindex — it also refreshes EDHREC synergy for the commanders already in your decks. The same thing can be done from the terminal if you prefer:

```bash
python3 scryfall.py bulk oracle_cards --download
python3 scryfall.py bulk all_cards --download   # includes PT names + images
mv oracle-cards-*.jsonl.gz all-cards-*.jsonl.gz data/
python3 mtgdb.py build
python3 edhrec.py fetch "Your Commander's Name" --combos
```

`webapp/seed.py` is optional — it's a worked example that loads the original author's own two decks (Syr Konrad, Toshiro Umezawa) instead of starting empty. Skip it unless you specifically want that sample data.

## Why there's no GitHub Pages

GitHub Pages only serves static files — it can't run the Python (FastAPI) backend the whole app depends on (decks, collection, scanner, games). Running locally with `uvicorn` is how this gets used on any computer.

## Structure

```
scryfall.py     — Scryfall API CLI client (search, decklist validation, bulk data)
mtgdb.py        — builds/queries the local SQLite card index (offline)
edhrec.py       — EDHREC synergy/combo cache per commander
webapp/
  server.py     — FastAPI API (decks, collection, scanner, games, activity log)
  db.py         — app database schema + migrations (SQLite)
  data_update.py — background job behind the "Atualizar base de dados" button
  deck_wizard.py — deterministic deckbuilder behind "Montar automaticamente" (no LLM)
  decklist_import.py — parses pasted/uploaded decklists (Moxfield/Archidekt/.txt/.csv/.xlsx)
  knowledge/    — deckbuilding_guide.json: ratios/brackets used by deck_wizard.py
  seed.py       — optional: loads the author's own decks/collection as sample data
  static/       — frontend (plain HTML/CSS/JS, no build step)
```

## Tools used by the app (none require an API key)

- Card database: Scryfall bulk data (38k+ cards, official PT names, images)
- Deck synergy: EDHREC (cached locally per commander)
- Scanner OCR: Tesseract.js (loaded from CDN on first use, then browser-cached)
