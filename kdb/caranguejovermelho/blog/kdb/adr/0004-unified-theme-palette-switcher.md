# 0004. unified theme and palette switcher

- status: accepted
- date: 2026-06-23

## context

the nav started with two separate controls: one button for theme mode (light, dark, system) and a second button for the palette. two dropdowns side by side, redundant and crowded.

## decision

one dropdown. the three theme-mode icons sit in a row at the top, a divider, then the palette swatches as a vertical list. `static/theme-switcher.js` owns the `.theme-modes` buttons and `static/palette-switcher.js` owns the `[data-palette-id]` buttons, with selectors scoped so the two never fight over the `.active` class. the standalone palette icon (`--icon-palette`) became orphan and was removed.

## consequences

- one entry in the nav, two axes visually grouped but still independent.
- the default for each axis renders server-side, choice persists in `localStorage`, no flash.
- two small scripts share one container, so the scoped selectors are now a constraint: a rename on one side has to keep the other side's scope intact.
