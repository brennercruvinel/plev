# Spellbook — MTG Collection & Deck Manager

Native macOS app for managing a Magic: The Gathering collection and Commander decks — any commander, any colors, any player. Rust + Tauri, with an embedded SQLite card index, so it runs offline once the card data is downloaded.

Everything is created from the UI: use "**Novo Deck**" (Dashboard/Meus Decks) to start a deck for any commander — optionally toggle "**Montar automaticamente**" to have it filled out deterministically (lands/ramp/draw/removal ratios + EDHREC synergy for that commander — no LLM involved, every card is verified against the local index before being added) — and "**+ Adicionar Carta**" (Dashboard/Coleção) to add cards to your collection, one at a time or by pasting a list of names. No editing Python files or seed data required — those only exist to load the original author's own decks as a worked example.

Full design-decision documentation lives in the Obsidian vault: `MTG/App - Protótipo Spellbook.md` (in Portuguese — that's the owner's personal knowledge base).

## Why this repository doesn't include the data

- The Scryfall bulk data + card index is ~430 MB — over GitHub's file size limit, and fully reconstructible from Scryfall's public API.
- Your decks/collection are personal data and stay on your machine.

Both live outside the repo (see "Where your data lives") and are in `.gitignore`.

## Running it (macOS)

Spellbook is a native macOS app. Build it once and it installs like any other app:

```bash
# one-time: Rust toolchain + Tauri CLI
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
cargo install tauri-cli --version "^2" --locked

cd desktop/src-tauri
cargo tauri build --bundles app
cp -R target/release/bundle/macos/Spellbook.app ~/Applications/
```

Then open it from Launchpad or the Dock. On first launch macOS asks for access to
your Documents folder only if you point it at data kept there — by default it doesn't
need to.

### Where your data lives

Everything the app owns is under `~/Library/Application Support/Spellbook/`:

```
app.db            your decks / collection / games
data/mtg.sqlite   the Scryfall card index
data/edhrec/      cached per-commander synergy
config.json       optional — override any of the paths above
```

The frontend ships inside the .app bundle, so an installed copy doesn't depend on this
repository being present at all. The card index is not bundled (~31MB, and it's yours):
it's rebuilt from Scryfall's public bulk data.

Until the in-app updater is ported to Rust, build the index with the Python CLIs at the
repo root and copy it into place:

```bash
pip3 install --user -r requirements.txt
python3 scryfall.py bulk oracle_cards --download
python3 scryfall.py bulk all_cards --download   # includes PT names + images
mv oracle-cards-*.jsonl.gz all-cards-*.jsonl.gz data/
python3 mtgdb.py build
cp data/mtg.sqlite ~/Library/Application\ Support/Spellbook/data/
```

## Status of the port

The desktop app is the product; the FastAPI web version under `webapp/` is retired and
kept only for the CLI/data-building code it shares.

Working natively: decks (create / edit / delete, incl. partner commanders), collection
(add, allocate, free), games, card search and detail with accent-insensitive matching,
and auto-build with ownership modes.

Not ported yet — use the Python CLIs meanwhile: the in-app data update, decklist import,
EDHREC synergy fetch, and the scanner.

## Structure

```
desktop/src-tauri/     the app
  src/main.rs          window + embedded HTTP server on a loopback port
  src/api.rs           read endpoints (cards, decks, collection, games)
  src/writes.rs        write endpoints
  src/wizard.rs        deterministic deckbuilder, with ownership modes (no LLM)
  src/db.rs            SQLite access, schema + migrations, accent folding
  src/paths.rs         where data and the frontend are resolved from

webapp/static/         the frontend (plain HTML/CSS/JS, no build step);
                       bundled into the .app at build time

scryfall.py            Scryfall API CLI (search, decklist validation, bulk data)
mtgdb.py               builds/queries the SQLite card index
edhrec.py              EDHREC synergy/combo cache per commander
webapp/*.py            retired FastAPI version, kept for the shared data-building code
```

## Tools used by the app (none require an API key)

- Card database: Scryfall bulk data (38k+ cards, official PT names, images)
- Deck synergy: EDHREC (cached locally per commander)
- Scanner OCR: Tesseract.js (CDN on first use — the one remaining online dependency)
