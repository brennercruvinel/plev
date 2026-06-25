# 0002. wcag aa as a scripted gate, not eyeballed

- status: accepted
- date: 2026-06-23

## context

claiming "WCAG AA verified" by eye produced false confidence. an early pass checked two pairs (text over background, accent over background) and called it done, while buttons, secondary text, nav, and alerts went unchecked and some failed. the opposite mistake also happened: over-correcting every color toward AA washed the palettes out and fought the theme's own taste. note that the original orange link is about 2.17:1 over its background, so blindly forcing 4.5:1 everywhere is not even faithful to what already shipped.

## decision

contrast is a number, computed, not an opinion. a deterministic Python script replicates the theme's sRGB `color-mix` exactly and computes the real WCAG ratio for every visible pair: accent over background, body text, secondary text, each syntax token over the code panel, each alert color, across all palettes and both modes. colors are chosen from the curated ramps; only where a ramp cannot reach AA do we darken (light) or brighten (dark) a tone, and that derivation is called out. nothing ships until the table reads all-pass at 4.5:1.

## consequences

- the validation is reproducible and honest. no more "trust me".
- light accents end up deep enough to read as links on the near-white background; dark accents bright enough to pop on the tinted dark.
- the script, not intuition, is the source of truth for color picks.
- there are real identity tradeoffs that fall out of the math (see adr 0001), and i accept them rather than fake the number.
- the contrast helper is throwaway tooling, kept out of the repo (scratchpad), but the picked values and their ratios are recorded here and in the SCSS comments.
