---
id: adr-0073
title: "design for ranges, not screens and devices"
project: plev
type: adr
status: accepted
date: 2026-09-29
date_basis: header
topics: [layout, design-system, accessibility, mobile]
relations: [adr-0002, adr-0027]
source: docs/adr/rendering/0073-design-for-ranges-not-screens-and-devices.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# design for ranges, not screens and devices

Records five design rules that apply to every visible change, on all platforms: fluid scaling driven by tokens, component response to space and content, intrinsic layout, relative and semantic values, and progressive adaptation tested over continuous ranges. The decision is to design for content, constraints, capabilities and ranges, not for screens and devices. Before, each task decided this on its own, and the rules, written for web applications, came to be read with each platform's equivalent.

## context

the contract already said where geometry and values come from. container
geometry derives from available space
(content-driven-layout-not-fixed-constants), every visual value is a
`Theme` token (comps-one-design-system-crate), and accessibility, reduced
motion, contrast and density are native requirements (the engine rules in
AGENTS.md).

it did not say how a value behaves between two sizes, what a component is
allowed to look at, or which inputs and capabilities an interface has to
hold under. each task decided that on its own.

the five rules came written for web applications, and their examples are
css mechanisms. plev also runs on desktop and phones, so every rule reads
"or the platform's equivalent", and none of them names a file, a screen
or a dimension.

## decision

five rules bind every change that touches anything visible, on every
platform:

- fluid scale: values scale continuously between limits, from a scale
  defined in design tokens
- component-driven responsiveness: a component responds to the space,
  content and context it receives; a breakpoint exists where the
  composition has to change
- intrinsic layout: the layout resolves size, distribution, wrapping and
  proportion by itself; fixed sizing only where the dimension is a real
  part of the design
- relative and semantic values: relative measures, tokens, logical
  properties and proportions derived from the geometry
- progressive adaptation: the interface adapts to size, content, input,
  accessibility and platform capabilities, and is tested over continuous
  ranges

the principle behind the five: design for content, constraints,
capabilities and ranges, not screens and devices. the text lives in
AGENTS.md, section "design for ranges, not screens", and is repeated in
the engine manual. hoje o `AGENTS.md` não está mais na árvore (foi removido em 704412f1); a seção está em [how to code against the plev engine](../../ref/operations/code-against-the-plev-engine.md). the other documents carry the short form and point
there.

## consequences

- the rules are read at task start and are part of a task's acceptance;
  the definition of done for ui work has an item for them
- the pull request template has one checkbox per rule, and the
  orchestrator's diff review has a line for them. o template `.github/pull_request_template.md` saiu da árvore em 2026-10-07 (commit `9c4bca2d`); a seção `design for ranges` estava nele desde o commit `5662b37e`
- arc.toml `[contracts]` (hoje `docs/ATLAS.toml`) and the README's contracts list name the
  principle
- two documents hold the same text. an edit to one lands in the other and
  in the checklist in the same change; nothing checks that yet
- the code that existed before this record was not audited against the
  five rules in this change

## alternatives

O texto não registra alternativas pesadas. O que a decisão recusa está em `avoid`: um número solto dentro do componente, um breakpoint para ajustar poucos pixels, uma regra escrita para um dispositivo e uma tela verificada só em predefinições de dispositivo.

## avoid

- a loose number inside a component
- a breakpoint added to adjust a few pixels
- a rule written for one device
- a screen checked only at device presets
