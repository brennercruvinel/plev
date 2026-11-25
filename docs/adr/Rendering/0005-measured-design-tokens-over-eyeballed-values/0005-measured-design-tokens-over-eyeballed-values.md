---
id: adr-0005
title: "design tokens come from measurement of the live reference, not from eyes or static CSS"
project: plev
type: adr
status: accepted
date: 2024-12-19
date_basis: reconstructed
recorded: 2026-06-10
topics: [design-system, rendering]
relations: []
evidence: [9c5fbd3]
source: docs/adr/rendering/0005-measured-design-tokens-over-eyeballed-values.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# design tokens come from measurement of the live reference, not from eyes or static CSS

Decides that a token enters the theme only with a measurement of the live reference behind it: computed styles and pixels sampled at several points, and not stylesheet literals or human perception. The reason is that three values were circulating for the page background, and the screens were rejected against the first two before the measurement. The result is the background measured at #303030, with the fidelity dispute reduced to a numeric comparison. The values now live in `rust/engine/src/styling/theme/hoff.rs`, where the tests pin them, according to docs/ATLAS.toml.

## context

three competing values circulated for the reference page background: a
spec document written from visual inspection said near black (#0E0E0E),
the reference's static stylesheet declared #444444 on the body, and the
actual rendered page composes translucent layers over the body to a
graphite #303030. screens were built and rejected against the first two
values before the third was established. the discrepancy consumed several
rework cycles and one entire branch (rejected for shipping #121212).

## decision

a token enters the theme only with a measurement of the live rendered
reference behind it: computed styles and screen pixels sampled at multiple
points, not stylesheet literals and not human perception. the measured
values are recorded in the theme source (src/theme/hoff.rs) and pinned by
tests (`hoff_page_is_measured_graphite_not_black`).

measured token set: page #303030, sidebar #2E2E2E, card lift #343434,
popover #3B3B3B radius 32, text white at alphas .95/.76/.50, rubik
400-700, backdrop blur restricted to pills, search and menus (the
reference does not frost content cards).

## consequences

- disagreements about fidelity reduce to measurements, which ends
  re-litigation. a claim of "too light" or "too dark" is answered with a
  pixel sample on both sides
- a stylesheet literal is treated as a hypothesis about the render, never
  as the render. translucent stacking, blend modes and gamma all separate
  the two

## avoid

- do not transcribe colors from a screenshot by eye. the #0E0E0E error
  came from exactly that
- do not copy the first plausible value out of a stylesheet. verify what
  actually composes on screen
- do not let a verifier approve "looks the same". the working protocol is
  numeric: sample, compare, state the delta

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [hoff visual tokens (measured)](../../../Research/Rendering/hoff-visual-tokens.md)
<!-- /related -->
