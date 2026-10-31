---
id: docs
title: "plev docs"
project: plev
type: index
status: active
date: null
date_basis: generated
topics: [navigation]
relations: [docs-conventions, timeline, dates-and-sources]
source: docs/README.md
source_revision: ec0a224c
revised: 2026-10-31
review: reviewed
---

# plev docs

plev began in 2021 as a way to learn Rust by drawing on the GPU and became a
compositing engine with a semantic contract that people, screen readers and agents
operate through the same path. These docs keep that path: the tasks, the decisions,
the research, the measurements and what each mistake taught, with the code that
backs each claim.

## Where to start

| To | Start at |
|---|---|
| follow the story in time | the [timeline](log/timeline.md), and [dates and sources](log/dates-and-sources.md) to read the dates |
| see what was done, task by task | [tasks](tasks/README.md), one folder per task |
| understand why the code is the way it is | the [ADRs](adr/README.md); [adr/TEMPLATE.md](adr/TEMPLATE.md) for a new one |
| what was measured, how, and its limits | [experiments](experiments/README.md) |
| what was studied before building | [research](research/README.md), with the [generative runtime thesis](research/generative-runtime-thesis.md) |
| what went wrong and what stayed | [learnings](learnings/README.md) |
| work on the code today | [reference](ref/README.md), starting with the [engine manual](ref/operations/code-against-the-plev-engine.md) |
| what happened, in order | the [log](log/README.md): journal, changelog, programs, project state |
| the architecture on one page | [ATLAS.toml](ATLAS.toml); the old names and today's in [TERMS.toml](TERMS.toml) |

## How the pieces connect

A research note raises a hypothesis; a task implements it; an ADR fixes the decision
that stayed; an experiment measures it; a learning says what changed in the
understanding. Each page points to the others in the `relations` field of its header,
and [CATALOG.json](CATALOG.json) gathers all of them, with their headings, for
whoever wants to query the archive as data. The shape of every page is in the
[conventions](ref/operations/docs-conventions.md).

## Reading an old record

A historical page speaks with the names and paths of its time (`sem`, `comps`,
`showcase`, `crates/`…); TERMS.toml leads each one to today's name. A command or a
number in a task holds for the revision it was written at. git only keeps the history
from June 2026; before that, the date of a page comes from its header, and
[dates and sources](log/dates-and-sources.md) says where it was revised after the fact.
