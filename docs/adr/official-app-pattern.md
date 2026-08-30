---
type: adr
status: accepted
tags: [ui, patterns, apps, widgets, builder, experimental]
date: 2026-08-30
---

# official app pattern: state in structs, retained widgets, explicit invalidation

## context

the engine grew five ways to build ui: retained widgets (ui/widgets), the
declarative builder (builder/), the view/component traits with the
`#[component]` macro, the signal runtime, and the narrate dsl. nothing
said which one an app should use, so each exploration wave left its own
paradigm looking equally official.

meanwhile every app that ships (showcase, ide, nestui) converged on the
same shape: plain structs own the state, retained widgets handle events
and report `EventResult`, and every visible mutation invalidates
(render-on-demand-requires-explicit-invalidation).

## decision

- the official app pattern is **state in structs + retained widgets +
  explicit invalidation**; the showcase is the template
- the declarative builder is **supported for prototypes and demos** (the
  showcase builder tour, the parser's emit target)
- `view`, `component`, `signal` and `narrate` are **experimental**: no new
  app code on them without an adr
- module docs and crate descriptions carry the maturity label, so the
  boundary is visible from the code, not just from this record

## consequences

- new sections and apps copy the showcase wiring (one TextStyle per run,
  `EventResult` merge, tick gated by the visible section)
- experimental modules stay compiled and tested (nothing is deleted);
  promotion out of experimental requires an adr showing an app-scale use

## avoid

- do not start an app on signals/narrate because the demo reads shorter;
  the demos never show invalidation, focus routing or ime, which the
  retained path already solved
