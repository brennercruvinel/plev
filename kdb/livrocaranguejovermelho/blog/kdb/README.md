# kdb

knowledge base for brennercruvinel.com. decisions i want to remember, written for me, not docs for users (those live in the root README and on the live site).

## adr

architectural decision records, in `adr/`. each one is a decision made while building the theme: the context around it, what i chose, and what it cost. numbered and append-only. if a decision gets reversed later, i add a new adr that supersedes the old one instead of rewriting history.

| # | decision |
| --- | --- |
| [0001](adr/0001-accent-derived-palettes.md) | accent-derived palette system |
| [0002](adr/0002-wcag-aa-deterministic-gate.md) | wcag aa as a scripted gate, not eyeballed |
| [0003](adr/0003-palette-aware-code-highlighting.md) | palette-aware syntax highlighting over Zola's inline Solarized |
| [0004](adr/0004-unified-theme-palette-switcher.md) | unified theme and palette switcher |
| [0005](adr/0005-background-tint-levels.md) | background tint levels and the readability tradeoff |
| [0006](adr/0006-blog-year-folders-path-override.md) | blog in year folders with a path override |

## related

- `palettes/` at the repo root holds the curated source hexes for each ramp. it is reference, not built by Zola.
