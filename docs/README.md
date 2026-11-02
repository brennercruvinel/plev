---
id: docs
title: "plev docs"
project: plev
type: index
status: active
date: null
date_basis: generated
recorded: 2026-10-09
topics: [navigation]
relations: [docs-conventions, changelog]
source: docs/README.md
source_revision: ec0a224c
revised: 2026-11-02
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
| follow the story in time | the [changelog](CHANGELOG.md), newest first, one section per date; [reading the dates](CHANGELOG.md#reading-the-dates) says how far each date holds |
| see what was done, task by task | [tasks](Tasks/README.md), one folder per task |
| understand why the code is the way it is | the [ADRs](adr/README.md), one folder per decision, with what it taught beside it; [adr/TEMPLATE.md](adr/TEMPLATE.md) for a new one |
| what was measured, how, and its limits | [experiments](Experiments/README.md) |
| what we know about a subject | [research](Research/README.md), by the same themes as the ADRs, with the [generative runtime thesis](Research/Generative/generative-runtime-thesis.md) and its [program](Research/Generative/generative-runtime-program.md) |
| what went wrong and what stayed | beside its decision, in the ADR's folder; what belongs to no single decision in [learnings](Learnings/README.md) |
| work on the code today | [Research/Workflows](Research/README.md#workflows), starting with the [engine manual](Research/Workflows/code-against-the-plev-engine.md) |
| the architecture on one page | [ATLAS.toml](ATLAS.toml); the old names and today's in [TERMS.toml](TERMS.toml) |

## How the pieces connect

The archive has two trees with the same themes (Ecosystem, Generative, Importing,
Interface, Recording, Rendering, Semantics, Structure, Workflows): `adr/<Theme>/` holds
what was decided and `Research/<Theme>/` what is known about the same subject. A research
note raises a hypothesis; a task implements it; an ADR fixes the decision that stayed; an
experiment measures it; a learning says what changed in the understanding, and lives in
the folder of the ADR it led to. Each page points to the others in the `relations` field of its header,
and [CATALOG.json](CATALOG.json) gathers all of them, with their headings, for
whoever wants to query the archive as data. The shape of every page is in the
[conventions](Research/Workflows/docs-conventions.md).

## Reading an old record

A historical page speaks with the names and paths of its time (`sem`, `comps`,
`showcase`, `crates/`…); TERMS.toml leads each one to today's name. A command or a
number in a task holds for the revision it was written at. git only keeps the history
from June 2026; before that, the date of a page comes from its header, and
[reading the dates](CHANGELOG.md#reading-the-dates) says where it was revised after the fact.
