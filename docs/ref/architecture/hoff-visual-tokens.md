---
id: hoff-visual-tokens
title: "hoff visual tokens (measured)"
project: plev
type: record
status: reference
date: 2026-06-10
date_basis: header
topics: [design-system, text, rendering]
relations: [adr-0005]
source: kdb/reference/hoff-visual-tokens.md
source_revision: d0958da0
revised: 2026-10-30
review: reviewed
evidence: [9c5fbd3]
---

# hoff visual tokens (measured)

Canonical table of the visual tokens of the hoff language, measured on the live render of the reference (hoff-research-social) and not copied from style sheets. The code that implements it was at `src/theme/hoff.rs` in the 2026-06 tree and is now at `rust/engine/src/styling/theme/hoff.rs`, according to docs/ATLAS.toml. The record includes color, typography and effect values with verification anchors, and a product decision still open about the page background.

canonical token table for the hoff design language as implemented in
src/theme/hoff.rs. all values were measured from the live rendered
reference (hoff-research-social), not transcribed from stylesheets. see
ADR measured-design-tokens-over-eyeballed-values for the protocol.

## color

| token | value | provenance |
|---|---|---|
| page background | #303030 (48,48,48) | live composite of rgba(#282828,.7) over body; the body's own #444444 never reaches the screen |
| sidebar, columns | #2E2E2E | rgba(#282828,.8) composite |
| card surface lift | #343434 | post card rgba(248,248,248,.02) over page |
| popover, menu | #3B3B3B, radius 32 | measured |
| text primary | white alpha .95 | reference variables.sass |
| text body | white alpha .76 | reference variables.sass |
| text meta | white alpha .50 | reference variables.sass |

## typography

| token | value |
|---|---|
| UI family | rubik 400/500/600/700, embedded, pinned as sans-serif default |
| monospace | jetbrains mono, pinned |
| fallback | inter (embedded, secondary) |

## effects

| token | rule |
|---|---|
| backdrop blur (glass) | pills, search, menus only; content cards are never frosted |
| shadows | analytic (evan wallace formulation), plus inset variant |

## verification anchors

- expected pixel sample for page background on any platform: (48,48,48)
- linear value of #303030 after `to_linear_array()`: 0.0296
- guarding tests: `hoff_page_is_measured_graphite_not_black`,
  `to_linear_array_darkens_srgb_midtones`,
  `default_family_resolves_rubik_faces_for_all_ui_weights`

## open design decision

whether to keep #303030 (faithful to the reference) or adopt a darker
page (#1A1A1A) for higher contrast remains a product owner decision,
tracked in docs/status.md.
