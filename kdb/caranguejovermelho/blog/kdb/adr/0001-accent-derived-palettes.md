# 0001. accent-derived palette system

- status: accepted
- date: 2026-06-23

## context

i wanted more than one color theme on the site. the first attempts overrode every color variable per palette: `--bg-color`, `--fg-color`, the muted scale, glass, the alert colors, all set to fixed hexes per theme. the result was opaque, washed out, and the dark backgrounds all collapsed into the same near-black. meanwhile the original orange theme looked good for a reason i had been ignoring: it derives the background, the glow, and the auto-contrast text from a single accent through `color-mix` and OKLCH.

## decision

a palette only sets `--accent-color`, one value for light and one for dark. everything else stays derived in `sass/_variables.scss`: background is `color-mix` of the accent, glow is the accent at low alpha, links, titles, active states are the accent, button text is the OKLCH-derived contrast color. the palette selectors are `:root[data-palette="..."]` so they outrank the inline default that `variables.html` writes onto `:root`.

the accents are real rungs from curated ramps (see `palettes/`), never derived on the fly, except where contrast forces a deeper or brighter tone (see adr 0002).

## consequences

- adding a palette is two hexes in the `$palettes` map.
- the themes stay coherent because they share one derivation, instead of drifting as a pile of hand-tuned variants.
- the original behavior that made the orange theme good is preserved.
- in dark mode the distinction between palettes comes mostly from the accent and the glow. the background is a tinted near-dark by design, same as the original.
- a fully pastel palette has no dark rung to use as a light-mode accent, so its light accent shifts hue (raindrops light is magenta, not the signature pink).
- later i cut the set from 11 to 6 (default, orchid, raindrops, emerald, logseq, candy) and renamed steel to logseq. fewer, but each one carries its weight.
