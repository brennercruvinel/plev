---
id: adr-0028
title: "core theme: the tailwind class lists as button recipes"
project: plev
type: adr
status: accepted
date: 2026-09-21
date_basis: header
topics: [design-system, gpu]
relations: []
source: docs/adr/rendering/0028-core-theme-tailwind-classes-as-recipes.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# core theme: the tailwind class lists as button recipes

Decides that the Tailwind classes of the TypeScript kit become button recipes in tokens (one `ButtonRecipe` per variant, with rest, hover and active states), and that no visual literal enters a widget. The reason: the dark HOFF theme did not match the kit, with different radii and borders with a dark fringe. The result is two global fixes: the blending of border and fill now happens in premultiplied linear, and zero-blur shadows get anti-aliasing with a minimum sigma of 0.5 px.

## context

the design system the brenner keeps refining lives in a private
typescript kit (next + tailwind: a light kit on nine gray shades, Inter,
measured shadow stacks). plev's theme was the HOFF dark
glass transcribed from the hoff site, and its buttons did not match: the
radii disagreed, the rings looked masked, the corners carried a dark
fringe. the two systems had to converge on the typescript one, one
component at a time, deleting the old widget and rebuilding it over the
engine.

a tailwind button is a class list: a gradient fill, a `box-shadow` list
mixing drop layers, `0 0 0 1px` rings and `inset` key-lights, an
`after:` overlay on hover, an opacity when disabled, and a height / padding
/ radius / text size per size. the repo rule says no visual literal in a
widget, so that list had to become tokens.

## decision

- `Theme::hoff_core()` (`theme/hoff_core.rs`): the kit's shades and
  accents, its depths page as `ShadowTokens`, radii 8 / 10 / 12 / 16 /
  22, Inter tracked -0.02em. `Theme::named("core")`; the showcase default.
- `theme.button: ButtonTokens` (`theme/button.rs`): a `ButtonRecipe` per
  variant (primary, secondary, warning, outline, ghost, toolbar,
  toolbar_active), each a `ButtonSurface` at rest / hover / active: the
  fill gradient, the css shadow list (rings are spread-1 blur-0 layers,
  key-lights are inset layers), the overlay wash, the text color; plus
  `disabled_alpha` and the two size ladders (28 / 36 / 40 / 44 labeled,
  24 / 36 / 40 icon-only). `ButtonTokens::hoff_core` is the kit verbatim;
  `ButtonTokens::derive(&colors)` gives every other theme flat recipes so
  the widgets render everywhere.
- `ShadowSpec.inset`; `TypographyScale.family` and `tracking_em`, with
  `exact(size, line_height, weight)` for a ramp written in px pairs.
- Inter Regular / Medium / SemiBold embedded (OFL) next to Inclusive
  Sans; a theme picks its face through `family`.
- `comps::action` rewritten: `press.rs` (the press model and
  `paint_surface`, which paints one `ButtonSurface`), `Button`,
  `IconButton` (outline / ghost / toolbar with `active`), `SubmitButton`
  (primary that turns secondary on hover and while active).
  `Intent::Destructive` maps to the warning recipe; other intents keep
  the variant.
- the showcase Buttons page is the kit's ButtonsPage: group title, rule,
  cards with a chip per state.

## the two global defects this surfaced

- `rect_sdf.wgsl` mixed border against fill, and the gradient stops, in
  straight alpha. a transparent fill is transparent black, so along the
  anti-aliased inner edge of every stroke the border was dragged toward
  black: a dark gray fringe on every rounded corner (measured 189 on an
  ECECEC border), invisible on the dark theme, plain on the light one.
  every mix now happens in premultiplied linear space. the HOFF
  edge-light gradient (edge -> transparent) had the same muddy tail.
- a blur-0 shadow had no anti-aliasing in the analytic shader, so the
  kit's 1px rings stair-stepped at the corners. `shadow_sigma` floors at
  `MIN_SHADOW_SIGMA` (0.5px).

## consequences

- the kit migrates page by page (cards, inputs, menu, modal, toolbar,
  tooltip...). HOFF dark glass stays until the last page moves, then goes.
- the theme clone per frame now carries `Vec<ShadowSpec>` lists; cheap,
  but a recipe is not a `Copy` type.
- the binary grew 1.2 MB with Inter.
