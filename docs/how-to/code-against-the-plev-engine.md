---
type: how-to
tags: [plev, engine, widgets, layout, text, color, clipping, checklist]
date: 2026-06-10
commit: bb34a1c
---

# how to code against the plev engine

this is the operating manual distilled from every defect found and fixed
while building showcase and ide on plev (wgpu 28, winit 0.30,
cosmic-text 0.18, taffy 0.9). each rule below was violated at least once
in this repository, shipped a visible defect, and was then root-caused.
agents and developers should treat violations as defects even when the
screen currently looks correct.

## the official app pattern

state lives in plain structs, ui is built from retained widgets
(`comps`, the design system crate: `comps::prelude::*`), and every
visible mutation invalidates (`EventResult::changed` / request redraw).
the showcase is the template app. the declarative builder
(`engine::builder`) is supported for prototypes and demos; `view`,
`component`, `signal` and `narrate` are experimental: do not start app
code on them (docs/adr/official-app-pattern.md).

## tokens, never literals

every widget takes `&Theme` and reads colors from `theme.colors` and
`theme.glass`, radii from `theme.shape` (by role: card, pill, nav,
tooltip, micro), heights and paddings from `theme.control`
(`ControlSize::Xs..Xl`, icon sizes, rims, focus ring, toggle geometry),
container widths from `theme.size`, timed transitions from
`theme.duration`, breakpoints and gutters from `theme.layout`, shadow
stacks from `theme.shadows` (by `Elevation`), and text styles from the
`theme.typography` ramp (`headline`, `title`, `base_*`, `caption_*`,
`small_*`, `mono`, `readout`). a widget that lays out from tokens takes
`&Theme` on `handle_event` too (select, tabs, slider, tree, list, modal,
menu, toast, text field), so the hit test and the pixels come from one
geometry. a number a screen needs that is not a token yet becomes one in
`crates/engine/src/theme/scales.rs`, with its spec value in
`tests_scales.rs`; it never lands in the screen.

an app frames its screens with `comps::shell::AppShell`: it hands the
screen the content rect for the current breakpoint (full sidebar, rail,
or a drawer behind a menu button on a phone) and routes the chrome's
events. the only tokens an app may own are domain semantics derived from
the theme (the ide's file-status colors).

every comps widget renders under every built-in theme at a phone and a
desktop width (`crates/comps/src/tests/matrix.rs`); a new widget joins
that matrix in the same change.

## before writing anything

check whether the engine or comps already provides the capability. the
engine had flex layout with text measure functions, percent dimensions,
touch recognition and real text measurement while both apps reimplemented
these poorly by hand; the ide carried a second copy of the theme and of
five widgets until comps absorbed them (docs/catalog.md records what
moved where). the single most expensive failure pattern in this project
was app-layer reimplementation of an existing feature. grep
crates/engine/src and crates/comps/src first; also look for one screen
that already does it right (the showcase sections are the in-repo proof
of correct responsive layout).

## text

- one `TextStyle` per text run, used for both `TextMeasurer::measure_styled`
  and `TextNodeKey::from_style`. never construct measurement and drawing
  parameters separately (see ADR one-text-style-for-measurement-and-drawing)
- never estimate text width arithmetically. `TextMeasurer` is GPU free and
  cached; there is no performance excuse
- a family+weight pair resolves only if that exact face is embedded
  (src/text/fonts.rs). if a new weight is introduced, embed it, then add
  it to the face-resolution test
- letter spacing, weight and line height all change advance. if any new
  typography attribute is added, plumb it through measurement and drawing
  in the same commit
- single-line truncation is `TextMeasurer::truncate_to_width(text, style,
  max_width)`, the same style you draw with, grapheme-safe, cached; paths
  elide by segment with `TextMeasurer::elide_path`. do not port per-app
  ellipsis helpers (the ide and nestui both grew one before this landed)

## small app utilities

- clipboard: `engine::clipboard` (`SystemClipboard` on desktop,
  `LocalClipboard` for tests/wasm/mobile, `default_clipboard()` picks per
  target). the editor widget in comps borrows the trait from here
- charts live in `comps::charts`: pure geometry (`line_chart`,
  `bar_chart`, `stacked_area`, `donut`) + `charts::draw` scene emission.
  charts carry no widget state: the owning view keeps the optional
  reveal `Tween<f32>`; `charts::draw::meter`/`hbars` cover the simple
  bar-meter patterns. network canvases: `engine::graph` (deterministic
  force layout, `ViewTransform`, BFS subsampling over 2.5k nodes) +
  `comps::graph::GraphView` (pan/zoom/hover/selection; the app owns
  tooltips and detail panels via `hovered()`/`selected()`)
- text fields are `comps::form::TextField` (glass look, caret, selection,
  click-to-caret through the same `TextStyle` it draws with); the editing
  core `comps::form::TextInput` draws nothing. the editor widget is
  `comps::editor::EditorView` with `EditorConfig::from_theme` /
  `EditorTheme::from_theme`
- the app frame is `comps::shell::AppShell` (sidebar, header, content
  rect, menu button on a phone, safe area); tables, panels, stats,
  avatars, badges, breadcrumbs, skeletons and code blocks are in
  `comps::content` / `comps::nav` / `comps::action`
- simple global shortcuts (`cmd+o`, `cmd+1..6`): build a
  `engine::actions::shortcuts::ShortcutMap` (`bind("cmd-o", "open")`) and
  match with `match_logical_key(&key_event.logical_key, modifiers)` where
  the shell tracks `ModifiersState` from `WindowEvent::ModifiersChanged`.
  reach for the full `KeymapMatcher`/`ActionRegistry` only when you need
  contexts, multi-stroke sequences or user-configurable keymaps

## color

- hex and theme values are sRGB. anything entering GPU memory must be
  linear: `to_linear_array()` for clear colors and uniforms; vertex colors
  are linearized in-shader
- surface render targets are created exclusively through
  `gpu.surface_render_view(&output)`. a default `create_view` skips gamma
  on the web target
- validate color work by sampling pixels, never by inspection. desktop
  background must measure 48,48,48 (and so must web)

## layout and responsiveness

- container geometry derives from `content.w` or from `LayoutEngine`.
  constants only as min, max and gap
- grids: `cols = floor((content.w + gap) / (min_w + gap)).max(1)`, then
  stretch the column width to fill, clamped by a readability maximum
- intrinsic rows wrap against `content.w`
- user-adjusted sizes store desired separately from effective; clamp at
  read time
- keep `layout()` functions pure (no GPU, no window). write viewport
  regression tests at an explicit narrow and wide width for every new
  screen (see showcase view tests for the pattern)

## events and invalidation

- any handler that changes visible state must invalidate (return true /
  request redraw). under render on demand a missed invalidation is a
  frozen app, not a glitch
- clipping rects pushed in logical pixels must be scaled by
  `gpu.clip_scale()` when they become physical scissor rects (HiDPI)
- after `gpu.resize`, the logical projection must be reapplied. the engine
  App does this; standalone event loops must call `set_projection`
  themselves

## platform

- no blocking executor on any path reachable from wasm; GPU init follows
  the spawn_local plus EventLoopProxy pattern (see ADR
  async-gpu-init-and-single-wasm-entry)
- touch arrives as synthesized pointer events; widgets need no special
  handling. multi-finger gestures come from the recognizer when needed
- `cargo check --target wasm32-unknown-unknown -p showcase` must stay
  green; it is the cheapest cross-platform guard in the repo

## definition of done for visual work

1. workspace tests green, including new regression tests for the change
2. wasm check green
3. the affected screen sampled by pixel at two window widths
4. no new constant that encodes a viewport assumption
5. measurement and drawing provably share one style object for any text
   touched
