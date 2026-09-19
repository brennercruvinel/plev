---
type: adr
status: accepted
tags: [ui, widgets, design-system, theme, tokens, responsive, comps]
date: 2026-09-18
---

# comps: one design system crate, every value a token

## context

the ui lived in four places that did not talk to each other. the engine
carried the retained widgets (`engine::ui::widgets`), the text input, the
overlay stack, the charts and the editor, plus a legacy immediate builder
with its own palette (`UiTheme`). the ide carried a second copy of the
HOFF tokens (493 lines, with the radii table and the shadow stacks the
engine did not have) and a second implementation of button, tabs, modal,
checkbox and context menu, plus avatar, badge, separator and panel header
that existed nowhere else. the showcase drew its own sidebar, header and
text fields by hand; urnaui drew its own field, panel and group label;
four demos had palettes of their own.

inside the widgets the numbers were literals: 36 / 40 / 44 / 52 control
heights (the engine and the ide disagreed on `Sm`), 240 menu width, 400
modal width, 248 sidebar width, `TextStyle::new(32.0).with_line_height(40.0)`
in the card, `blur_radius: 16.0, offset: [2.0, 4.0]` in three places for
the same key-light. nothing knew a viewport width: no breakpoint, no
gutter, no sidebar mode; a phone got the desktop layout at a third of the
width. docs/catalog.md holds the full inventory that opened this change.

## decision

- one crate, `crates/comps`, is the design system. `engine::ui`,
  `text_input`, `overlay`, `charts`, the `GraphView` widget and the editor
  view moved into it (git history preserved), organized by category:
  `core`, `recipe`, `icons`, `action`, `form`, `nav`, `content`,
  `feedback`, `overlay`, `charts`, `graph`, `editor`, `shell`. the engine
  keeps rendering, text, layout, input, animation, platform and the
  tokens. the legacy `Ui` builder and `UiTheme` are gone.
- every visual value is a token on `engine::theme::Theme`. the theme grew
  the scales the widgets were improvising: `shape` (radius by role),
  `control` (heights, paddings, icon sizes, rims, focus ring, toggle
  geometry, scrollbar, divider), `size` (menu, modal, sidebar, rail, card,
  readable width, toast, tooltip, dropdown), `duration`, `layout`
  (breakpoints, gutters, columns, sidebar mode, `fit_columns`), `shadows`
  (the four HOFF stacks by `Elevation`), and in `glass` the tones the
  widgets were deriving by multiplication (`text_active`, `text_default`,
  `tabs`, `disabled_alpha`, `wash_alpha`). the ramp gained `headline`,
  `mono` and `readout`. `Theme::hoff_light` is the ink inversion the ide
  carried. a widget never restates a number; a widget that lays out from
  tokens takes `&Theme` on `handle_event` too, so events and pixels come
  from one geometry.
- the pieces the apps drew by hand are widgets now: avatar, badge,
  separator, panel header, nav link, sidebar (full / rail / drawer by
  breakpoint, pinned footer links), breadcrumb (collapses when narrow),
  panel, stat, code block, skeleton, table (columns share free width by
  weight and drop by priority when narrow), text field (the glass field
  over the editing core), and `AppShell` (sidebar + header + content rect
  + menu button + safe area). the showcase, the ide and urnaui consume
  them; the ide's theme and components directories are deleted, the
  `todo` and `text_input` demos are absorbed by the showcase sections
  that already showed the same thing, the other demo palettes are views
  over `Theme::default()`.
- the matrix test (`crates/comps/src/tests/matrix.rs`) renders every
  widget under every built-in theme at a phone width and a desktop width.
  a new widget joins it in the same change.

## consequences

- one place to look for a component, one place to change a number.
  rebranding is a theme; a phone is a breakpoint.
- the apps lost their local tokens and helpers (ide: 2 160 lines; the
  showcase chrome: 300; urnaui field: 100) and gained widgets with tests.
- signature churn: `preferred_size(&theme)`, and `handle_event(...,
  &theme)` on select, tabs, slider, tree, list, table, modal, menu, toast,
  text field, empty state. the showcase sections pass `&self.theme`
  through their `layout`.
- `SplitPane::new` and `Table::new` take the theme at construction (their
  minimums and divider geometry come from it); `Card::width` is an option,
  the default is `size.card_w`.

## avoid

- a literal in a widget "because it is just this once": it becomes a
  token in `scales.rs` with its spec value in `tests_scales.rs`.
- a second theme, palette or helper module in an app. the ide's file
  status colors are the shape an app-owned token may take: domain
  semantics mapped onto theme intents.
- a widget that only works at a desktop width: phone width is a viewport
  in the matrix, and the shell is how a screen meets it.
