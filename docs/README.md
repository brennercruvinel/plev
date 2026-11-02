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
| see what was done, task by task | [tasks](Tasks/Tasks.md), one folder per task, its note named like the folder |
| understand why the code is the way it is | the [ADRs](adr/adr.md), one folder per decision, with what it taught beside it; [adr/TEMPLATE.md](adr/TEMPLATE.md) for a new one |
| what was measured, how, and its limits | the experiments, in [Research/Exploring](Research/Research.md#exploring) |
| what we know about a subject | [research](Research/Research.md), by the same themes as the ADRs, with the [generative runtime thesis](Research/Synthesis/generative-runtime-thesis.md) and its [program](Research/Synthesis/generative-runtime-program.md) |
| what went wrong and what stayed | the learnings, beside the decision they speak to, in the ADR's folder; what taught the way of working in [Research/Workflows](Research/Workflows/autonomous-cycle-learnings.md) |
| work on the code today | [Research/Workflows](Research/Research.md#workflows), starting with the [engine manual](Research/Workflows/code-against-the-plev-engine.md) |
| the architecture on one page | [ATLAS.toml](ATLAS.toml); the old names and today's in [TERMS.toml](TERMS.toml) |

## How the pieces connect

The archive has two trees with the same themes (Ecosystem, Importing, Interface,
Recording, Rendering, Semantics, Structure, Synthesis, Workflows): `adr/<Theme>/` holds
what was decided and `Research/<Theme>/` what is known about the same subject;
`Research/Exploring/` holds the experiments, what was measured. A research
note raises a hypothesis; a task implements it; an ADR fixes the decision that stayed; an
experiment measures it; a learning says what changed in the understanding, and lives in
the folder of the ADR it led to. Each page points to the others in the `relations` field of its header, and
ends with a Related list written from those relations in both directions, as plain
links: Quartz, Obsidian, Zola or a reader on GitHub follow and draw the same graph; [CATALOG.json](CATALOG.json) gathers all of them, with their headings, for
whoever wants to query the archive as data. The shape of every page is in the
[conventions](Research/Workflows/docs-conventions.md).

## Reading an old record

A historical page speaks with the names and paths of its time (`sem`, `comps`,
`showcase`, `crates/`…); TERMS.toml leads each one to today's name. A command or a
number in a task holds for the revision it was written at. git only keeps the history
from June 2026; before that, the date of a page comes from its header, and
[reading the dates](CHANGELOG.md#reading-the-dates) says where it was revised after the fact.
