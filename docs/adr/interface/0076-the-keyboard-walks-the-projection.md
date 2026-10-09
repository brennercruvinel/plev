---
type: adr
status: accepted
tags: [accessibility, keyboard, focus, wai-aria, showcase, comps, sem]
date: 2026-09-25
---

# the keyboard walks the projection

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

## rejected

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
