# Spellbook — MTG Collection & Deck Manager

Local prototype for managing a Magic: The Gathering collection and decks (mono-black Commander). Runs 100% offline after initial setup — no Node.js, just Python (FastAPI + SQLite) and a static HTML/CSS/JS frontend.

Full design-decision documentation lives in the Obsidian vault: `MTG/App - Protótipo Spellbook.md` (in Portuguese — that's the owner's personal knowledge base).

## Why this repository doesn't include the data

- `data/` (Scryfall bulk data + EDHREC cache) is ~430 MB — over GitHub's file size limit, and fully reconstructible from Scryfall's public API.
- `webapp/app.db` is the real collection/decks — personal data, stays local only.

Both are in `.gitignore`. To run on any machine, rebuild both with the steps below.

## Setup from scratch

```bash
# 1. Python dependencies (no Node.js needed)
pip3 install --user fastapi uvicorn "pydantic<3" python-multipart pymupdf

# 2. download the Scryfall card database (official bulk data, free, no API key)
python3 scryfall.py bulk oracle_cards --download
python3 scryfall.py bulk all_cards --download   # ~370MB, includes PT names + images
mv oracle-cards-*.jsonl.gz all-cards-*.jsonl.gz data/

# 3. build the local index (SQLite, ~14s)
python3 mtgdb.py build

# 4. (optional) cache EDHREC synergy for your commanders
python3 edhrec.py fetch "Syr Konrad, the Grim" --combos
python3 edhrec.py fetch "Toshiro Umezawa" --combos

# 5. populate the app database with your decks (edit webapp/seed.py with your own collection)
cd webapp
python3 seed.py

# 6. run
python3 -m uvicorn server:app --port 8420
```

Visit **http://127.0.0.1:8420**.

## Why there's no GitHub Pages

GitHub Pages only serves static files — it can't run the Python (FastAPI) backend the whole app depends on (decks, collection, scanner, games). Running locally with `uvicorn` is how this gets used on any computer.

## Structure

```
scryfall.py     — Scryfall API CLI client (search, decklist validation, bulk data)
mtgdb.py        — builds/queries the local SQLite card index (offline)
edhrec.py       — EDHREC synergy/combo cache per commander
webapp/
  server.py     — FastAPI API (decks, collection, scanner, games, activity log)
  db.py         — app database schema (SQLite)
  seed.py       — populates the database with real decks/collection
  static/       — frontend (plain HTML/CSS/JS, no build step)
```

## Tools used by the app (none require an API key)

- Card database: Scryfall bulk data (38k+ cards, official PT names, images)
- Deck synergy: EDHREC (cached locally per commander)
- Scanner OCR: Tesseract.js (loaded from CDN on first use, then browser-cached)
