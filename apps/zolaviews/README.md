# zolaviews

plev's knowledge base (`docs/`) read as a book on [Zola](https://www.getzola.org)
with the [Ametrine](https://codeberg.org/daudix/ametrine) theme, beside
`apps/quartzweb`. It runs from `apps/zolaviews` inside a plev checkout.

```
git submodule update --init --recursive apps/quartzweb apps/zolaviews/themes/ametrine
node apps/zolaviews/plev/book.mjs --serve --port 1112
```

`--serve` writes the content, runs `zola serve`, and writes again on every
change in `docs/` or in the blog (a change that touches more than 20 files
restarts `zola serve`, which is faster than its per-file rebuilds).
`node plev/book.mjs` alone writes the content once, for `zola build`.

## How it is put together

- `themes/ametrine` is the theme as its author publishes it, a submodule
  pointing at Codeberg; nothing in it is edited. The rest is a plain folder
  of plev: a static site, not a crate, which `tool/tests/test_structure.rs`
  recognizes by its `zola.toml`.
- `plev/book.mjs` writes `content/` and `static/docs/` (both generated and
  ignored) from `../../docs` and the monsters' blog in
  `../quartzweb/content/blog`. Zola has no plugin step and accepts only
  pages it can place, so the docs are written as Zola files instead of
  linked in. `sources.mjs` reads the inputs, `markdown.mjs` the text.
- The reading rules (sentence case, section names, link targets) are
  imported from quartzweb's `plugins/plev-book`, so both sites read the docs
  the same way.
- `templates/` overrides only what the book needs: `index.html` (the home),
  `section.html` (a collection of cards, or a task read as a document with
  the cards of its phases) and two partials for cards and details. Pages use
  the theme's `article.html`.
- `zola.toml` maps the docs onto the theme: a status is a category (with its
  color and icon), topics are tags, the nav groups the book's parts. The
  theme's analytics, fediverse lookups and demo content are off.

## What maps to what

| docs/ | the book |
|---|---|
| `README.md` | the home: the docs' own start, recent records, the parts |
| an index, the note named like its folder (`Tasks/Tasks.md`, `adr/adr.md`, `Research/Research.md`) | a collection: cards, newest first, its written index folded below |
| a task or an ADR with pages beside its note (phases, learnings) | a document: its text, then those pages as cards |
| a task or an ADR without pages beside it, any other page | an article: dates, reading time, status, tags, related, linked from, previous and next; the Related list docbase.py writes is dropped for the one the book draws |
| `status: superseded` | the theme's Archived statement and a red accent |
| `ATLAS.toml`, `TERMS.toml` | one page each, a section per table |
