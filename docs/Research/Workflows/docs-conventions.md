---
id: docs-conventions
title: "Conventions of the docs archive"
project: plev
type: manual
status: active
date: 2026-10-31
date_basis: editorial
recorded: 2026-10-31
topics: [editorial, process]
relations: [changelog]
source: reconstruction
source_revision: c854ebf3
revised: 2026-11-02
review: reviewed
---

# Conventions of the docs archive

The archive keeps the path of plev since 2021: what was decided, researched,
measured and learned, and where that lives in the code. This page is the contract
for the shape of every page in `docs/`. `tool/tasks/docbase.py check` verifies the
mechanical part of it in the gate, and `tool/tests/test_structure.rs` holds the
top-level folders and the shape of `adr/` and `Research/`.

## Where each thing lives

| Folder | What it holds | Identity |
|---|---|---|
| `adr/<Theme>/NNNN-<slug>/` | one folder per architecture decision record: `NNNN-<slug>.md` is the record, numbered in one sequence across the themes; the learnings that led to it or came out of it sit beside it | `adr-NNNN`, a learning `learning-<name>` |
| `Research/<Theme>/` | what is known about a subject: references studied, comparisons, the thesis and the program of the generative runtime, how each subsystem was designed, the manuals of the repository and of this archive | `research-<name>`, `architecture-…`, `operations-…`, `platform-…`, and the names of the thesis, program and plan |
| `Tasks/task-NN-<slug>/` | one folder per task: `README.md` with the plan, execution, delivery and what stayed open; the phases of a long task as pages beside it | `task-NN`, a phase `task-NN/<page>` |
| `Experiments/<slug>/` | one folder per experiment: question, setup, results, limits, and the task pages that hold the full record | `experiment-<slug>` |
| `Learnings/` | what a problem taught that belongs to no single decision | `learning-<name>` |
| `img/` | images the pages show | the path |

`CHANGELOG.md`, beside this tree, is the history in time: newest first, one section per
date, each with the entries written when the change landed and the records of that
day linked. A new task, ADR, experiment, research note or learning gets its line under
its date, and the check fails when one is missing.

`adr/` and `Research/` share their themes, so a decision and what is known about its
subject carry the same name in both trees:

| Theme | What it covers |
|---|---|
| `Ecosystem` | what plev compares with, takes from and depends on: competitors, reference repositories, libraries chosen |
| `Generative` | the generative runtime: plev IR, verification, proposers, generated and adapted surfaces |
| `Importing` | other formats into plev: lottie, svg, figma, react, html, gpui, the .monster codec |
| `Interface` | the person at the interface: input, touch, text, keyboard, accessibility, platform capabilities |
| `Recording` | recording sessions and reading them back |
| `Rendering` | the GPU, color, text raster, layout, effects, motion, frames, devices |
| `Semantics` | the semantic contract: identity, actions, receipts, agents |
| `Structure` | the shape of the repository, its crates, its components and its apps |
| `Workflows` | how the work is done: dev loop, builds, gates, regression, agents, this archive |

A new ADR copies [`adr/TEMPLATE.md`](../../adr/TEMPLATE.md) into its own folder and is
listed in [`adr/README.md`](../../adr/README.md). Task and ADR numbers are identity, not
chronology: the order in time comes from the `date` field.

A page in an ADR's folder other than the record is a learning (`type: learning`) whose
`relations` name that ADR; the ADR lists it under `## learnings`, and the other ADRs it
speaks to name it in their `relations` and link it the same way.

A page inside a task or experiment folder other than its `README.md` is a section:
`type: section` and `parent:` the id of that folder's `README.md`. Its file name says
what it records (`phase-2-proposer.md`, `item-6-deep-links.md`, `closing.md`), and
the task's `README.md` lists the sections in order.

## The header

YAML with one `key: value` per line. A value is a bare word, a string in double
quotes, `null`, or an inline list (`[a, "b c"]`).

| Field | What it says |
|---|---|
| `id` | stable identity, unique in the archive; relations use it |
| `title` | the subject, said the way a reader looks for it |
| `project` | always `plev` |
| `type` | `task`, `adr`, `research`, `experiment`, `learning`, `reference`, `manual`, `record`, `program`, `thesis`, `draft`, `template`, `section`, `index` |
| `parent` | only on a `section`: the `id` of the page it belongs to |
| `status` | task: `pending`, `in-progress`, `done`, `superseded`; ADR: `proposed`, `accepted`, `superseded`; the others: `reference`, `active`, `draft` |
| `date` | when what the page records happened, as precise as known: `2021`, `2021-03` or `2021-03-06`; `null` when unknown |
| `date_basis` | where the date came from: `header` (the source header), `text` (a date written in the body), `git` (a commit), `reconstructed` (the reconstruction of the history lost before June 2026, or a date inferred from several sources), `editorial`, `undated` |
| `recorded` | the day the page's text was first written: before the date for a plan, after it for a record written later |
| `topics` | subjects in kebab-case, few and shared between pages |
| `relations` | `id`s of linked pages: dependencies, the ADR a task took, the research behind it |
| `evidence` | commits (short sha) that back what the page states |
| `source` | the source path (or list) at `source_revision`; `reconstruction` when the page was written from code and commits |
| `source_revision` | the commit at which the source was read |
| `revised` | the date of the last editorial revision |
| `review` | `pending` (migrated, not yet read whole), `reviewed` (read whole and checked), `generated` (an index) |

The history before June 2026 was lost with an old Forgejo instance and written
down again between March and June 2026; commit `704412f1` gave those records the
dates of the work itself. They keep that date with `date_basis: reconstructed`,
and `recorded` keeps the day they were written; [reading the dates](../../CHANGELOG.md#reading-the-dates)
explains it and lists where the reconstruction was brought in step. A date only
changes when another record contradicts it, and then `date_basis` says why.

## The body

- It opens with the title (`# …`) and a paragraph that says what the page records,
  the result and why it matters. New text is written in English. The text that came
  from the history of the project stays in the language it was written in, and so do
  the titles of the ADRs.
- The project is named plev. Crate, module, format and command names are the ones in
  the code: `TERMS.toml` maps the old names (`sem`, `comps`, `showcase`…) to today's,
  and an old record keeps the names of its time.
- A page speaks of what happened, not of how the archive was organized: no "moved
  from", "reorganized into" or notes about the migration. The origin is in the header.
- Process instructions that only held for the file workflow of their time (move the
  file to `ongoing/`, rename it to `-DONE`) are dropped; what teaches something
  technical stays.
- A reconstructed statement says what it rests on (code at a commit, a test, a
  recorded measurement). What has no source is not stated.
- Links are relative and point to pages of the archive or to the code of the
  repository; code cited as historical proof points to the commit.

## Indexes

The `README.md` of `Tasks/`, `Experiments/`, `Research/` and `Learnings/`, and
`CATALOG.json`, are generated by `tool/tasks/docbase.py index`; they are not edited by
hand. `adr/README.md` is the ADR index kept by hand, and the check fails when an ADR is
missing from it.
