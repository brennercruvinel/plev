---
id: adr-0023
title: "official app pattern: state in structs, retained widgets, explicit invalidation"
project: plev
type: adr
status: accepted
date: 2026-08-30
date_basis: header
recorded: 2026-08-30
topics: [platform, generated-surfaces, ir]
relations: [adr-0008, adr-0051, task-69]
source: docs/adr/structure/0023-official-app-pattern.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# official app pattern: state in structs, retained widgets, explicit invalidation

Decides the official app pattern, with state in structs, retained widgets and explicit invalidation, using the showcase as the template. An addendum of 2026-09-24, from task-69, admits a second form: a surface materialized from verified IR, which composes the existing vocabulary without replacing the first pattern. The `view`, `component`, `signal` and `narrate` modules remain experimental, and no new app code goes into them without an ADR.

## addendum (2026-09-24, task-69): the second pattern, a surface materialized from verified IR

a screen can also be a surface materialized from verified plev IR
([docs/adr/Generative/0051-materialized-surfaces-from-verified-ir/0051-materialized-surfaces-from-verified-ir.md](../../Generative/0051-materialized-surfaces-from-verified-ir/0051-materialized-surfaces-from-verified-ir.md)): the `surface` crate
lays a `uir::VerifiedIr` out by the engine, draws it by the comps widgets
with the theme's tokens, projects it from the IR and turns every event
into a request through the app's dispatcher. it is the second supported
pattern, beside state in structs with retained widgets, and it never
replaces the first: the capabilities it operates are always an app's,
held in structs, projected (`sem::Semantics`) and run (`sem::Operable`)
by that app. a surface is a view over them and owns no state but its
widgets' (hover, focus, a caret).

when to use each:

- state in structs, retained widgets: every capability an app has,
  and every screen that is part of the app: long-lived, with its own
  interaction, motion, custom drawing or editing (the editor, the git
  workspace), a list that must virtualize, a layout the vocabulary of the
  IR cannot say. this is where a capability is born, projected and run.
- a surface materialized from verified IR: a screen that exists for a
  task and is composed from what apps already offer: written by a
  proposer, an agent, an importer or a person as IR, verified against the
  registry the app projects, and closed when the task ends. it composes
  the vocabulary's widgets over existing capabilities; it cannot add a
  capability, a widget or a visual value. the showcase's Surface page is
  the reference: the todo's IR beside the todo, one dispatcher.

a screen that needs a widget the vocabulary lacks is written by hand
(first pattern) until comps has the widget and an ADR adds it to the IR.

## context

the engine grew five ways to build ui: retained widgets (ui/widgets), the
declarative builder (builder/), the view/component traits with the
`#[component]` macro, the signal runtime, and the narrate dsl. nothing
said which one an app should use, so each exploration wave left its own
paradigm looking equally official.

meanwhile every app that ships (showcase, ide, urnaui) converged on the
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

- new pages and apps copy the showcase wiring (one TextStyle per run,
  `EventResult` merge, tick gated by the visible page)
- experimental modules stay compiled and tested (nothing is deleted);
  promotion out of experimental requires an adr showing an app-scale use

## avoid

- do not start an app on signals/narrate because the demo reads shorter;
  the demos never show invalidation, focus routing or ime, which the
  retained path already solved
