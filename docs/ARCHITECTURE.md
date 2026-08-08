# Architecture

Spellbook is a native macOS app: a Tauri shell that opens one window onto an embedded axum
server running on a loopback port. There is no web deployment and no build step — the frontend
is plain ES modules, served straight from disk.

```
Tauri window ──http──> axum (in-process) ──> app.db      (your decks/collection/games, read-write)
                                         └─> mtg.sqlite  (Scryfall card index, read-only)
```

Serving over `http://127.0.0.1:<port>` rather than Tauri's asset protocol is deliberate: the
frontend talks to relative `/api/*` paths with `fetch()`, so no code is aware it's in a desktop
shell. The port is chosen by binding to 0 and reading back what the OS assigned, so a second
copy of the app can't collide with the first.

## Where things go

```
desktop/src-tauri/src/
  main.rs           window + server wiring. Nothing domain-specific lives here.
  http.rs           the response envelope every handler answers in ({"detail": ...}).
  db.rs             connections, schema, migrations, accent folding, activity log.
  paths.rs          where data and the frontend are resolved from.
  routes/           one module per domain — each owns BOTH its reads and its writes.
    cards.rs        card-index lookups (read-only; never touches app.db)
    decks.rs        decks, deck cards, auto-build, decklist import, synergy
    collection.rs   physical copies: list, counts, add, allocate
    games.rs        games played + the activity feed
    data.rs         card-database freshness and refresh
  wizard.rs         deterministic deckbuilder (no LLM — every card comes from a local query)
  update.rs         Scryfall bulk download + index rebuild + EDHREC refresh
  edhrec.rs         per-commander synergy cache (fetch on demand, read offline)
  decklist.rs       decklist text parsing (Moxfield / Archidekt / plain text)

desktop/ui/
  index.html        the shell: sidebar buttons carry data-route
  css/style.css
  js/
    app.js          entry point — the only module that knows the full set of pages
    router.js       hash routing + sidebar chrome; knows nothing about the pages
    util.js         formatting, the h`` escaper, job polling
    api.js          every /api call, one function each
    icons.js        inline SVG + mana symbols
    deck-bits.js    deck presentation shared by the deck list and the deck page
    sidebar.js      the data-update panel in the sidebar footer
    views/          one module per page (home, decks, deck-detail, collection, games)
    ui/             modals reused across pages (add-card, card-modal)
```

## The two rules that keep it from drifting

**1. Group by domain, not by mechanism.** The first cut of the API split `api.rs` (every GET)
from `writes.rs` (every POST). Every feature then straddled two files, and it ended with a read
handler importing from the write module to get a status it needed. Routes are now grouped by the
noun they operate on. Adding an endpoint means editing one file.

**2. Nothing imports its own caller.** `router.js` doesn't import views — views register
themselves through `registerRoutes` in `app.js`. Shared deck rendering lives in `deck-bits.js`
rather than in whichever view happened to define it first. The module graph is checked to be
acyclic; keep it that way.

## Adding things

- **An endpoint** → find the `routes/*.rs` module that owns the noun, add the handler next to
  its siblings, register it in that module's `router()`. `routes/mod.rs` doesn't change.
- **A page** → write `views/<name>.js` exporting a render function, add it to `registerRoutes`
  in `app.js`, add a `data-route` button in `index.html`.
- **A shared widget** → `ui/<name>.js`. If two views need it, it does not belong to either.
- **A schema change** → append to `MIGRATIONS` in `db.rs`. Never edit `SCHEMA` for an existing
  table: `SCHEMA` only runs on a fresh database, so an edit there silently skips every install
  that already exists.

## Two invariants worth knowing

**Copies are physical.** Every row in `collection` is real cardboard. A card sleeved in two
decks is two rows, because it is two cards. `deck_cards` is the *list* a deck should contain;
`collection.allocated_deck_id` says which deck each physical copy is currently in. Any write
that changes one has to keep the other honest — adding a card claims a copy, removing one
releases it back to free. Counts shown to the user are always copies, never distinct names.

**The card index may not exist.** On a fresh install `mtg.sqlite` isn't there until the first
data update. `open_cards_db()` returns `Option` for exactly that reason, and every caller has to
degrade to something useful rather than error out.

## Caching

Asset URLs carry `?v=N` and are cacheable. `index.html` is served `no-store`, because the
webview's HTTP cache survives app restarts — a cached index would keep asking for the previous
`?v=` URLs, and an update would install but never appear. Bump the `?v=` on any asset you change.

## Debugging

A release build has no devtools. Run with `SPELLBOOK_LOG_REQUESTS=1` and every request is printed
with its status, which separates "the frontend never ran" from "it ran and the API answered
wrongly" in one glance. `SPELLBOOK_APP_DB_DIR` and `SPELLBOOK_DATA_DIR` point the app at a
throwaway copy of the databases, which is how changes get tested without touching real data.
