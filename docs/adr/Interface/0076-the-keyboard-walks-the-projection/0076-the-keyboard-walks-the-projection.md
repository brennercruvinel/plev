---
id: adr-0076
title: "the keyboard walks the projection"
project: plev
type: adr
status: accepted
date: 2026-09-25
date_basis: header
recorded: 2026-09-25
topics: [accessibility, input, semantics]
relations: [adr-0068, task-75, learning-generative-runtime]
source: docs/adr/interface/0076-the-keyboard-walks-the-projection.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# the keyboard walks the projection

Records how the keyboard walks the projection of the gallery's component pages: Tab and Shift-Tab follow the focus order that a screen reader also follows, the arrows follow WAI-ARIA inside each widget, and Enter and Space act through the dispatcher as `Actor::Human`, the same request as a click. Task-75 asked for full keyboard support on these pages, with a test; 29 of the 30 pages can be walked with the keyboard, and one test walks all of them with Tab. It matters because a second focus order, separate from the projection, would create a map of the page that could diverge from what the screen reader and the agents see.

## context

task-75 asks for the full keyboard on the gallery's component pages, with
a test. the pages already project their controls for a screen reader and
for agents (phase 2b); the keyboard had to follow the same order and act
through the same dispatcher, not a second map of the page.

## decision

- Tab and Shift-Tab walk the projection's focus order, the same order a
  screen reader walks (`view/dispatch/keyboard.rs`, `focus::next`), with the
  focus ring from `comps::recipe`.
- the arrows follow WAI-ARIA inside a widget: along tabs, up and down a
  tree (right opens or enters a branch, left closes or goes to the
  parent), through a select's options, a slider's steps and a context
  menu's items, with Home and End at the ends.
- Enter and Space act through the gallery's dispatcher as
  `Actor::Human`, the same request a click makes.
- a modal traps the focus while it is open and gives it back to the
  control that opened it when it closes.
- lists and tables (Table, Data Table, Scroll Area, Item) take the focus
  one row at a time: one row is focusable, the arrows walk the rows, Home
  and End go to the ends, Enter and Space select, and the focus follows the
  scroll. the 8 baselines that changed changed their meaning only (row 0
  focusable); their PNGs are identical byte for byte.
- shift with the arrows, Home and End selects in a focused text field. a
  `Select` key that no field takes moves as the key without shift, so
  shift changes nothing outside a field.

## alternatives

one Tab stop per composite widget (tabs and tree as a single stop, the
roving tabindex WAI-ARIA recommends). the engine offers a screen reader's
`Focus` action only on a node in the Tab sequence, so taking the other
tabs and tree rows out of the sequence took the reader's `Focus` away from
them. for a screen reader user that is a regression. it needs a concept
the contract does not have yet: a node that is focusable but outside the
Tab sequence. that is a change to `sem` and waits for Brenner's ok.

## consequences

- 29 of the 30 component pages walk with the keyboard, and one test walks
  all 29 with Tab. Resizable waits for a splitter role in the contract.
- open: shortcuts announced to the reader (`sem::Node` has no shortcut
  field); PageUp and PageDown and the context menu key do not reach the
  view (the shell's `keys.rs` does not send them).

## learnings

- [Generative runtime session: what worked and what bit us](../../Synthesis/0070-generative-runtime-results/generative-runtime.md)

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0068: accessibility is production-ready per platform](../0068-accessibility-is-production-ready-per-platform/0068-accessibility-is-production-ready-per-platform.md)
- [ADR 0086: the adaptive foundation reads preferences once into the theme](../0086-the-adaptive-foundation-reads-preferences-once-into-the-theme/0086-the-adaptive-foundation-reads-preferences-once-into-the-theme.md)
- [Generative runtime session: what worked and what bit us](../../Synthesis/0070-generative-runtime-results/generative-runtime.md)
- [task-75: production-ready platform accessibility](../../../Tasks/task-75-platform-accessibility/task-75-platform-accessibility.md)
<!-- /related -->
