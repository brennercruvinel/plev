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
revised: 2026-10-31
review: reviewed
---

# Conventions of the docs archive

The archive keeps the path of plev since 2021: what was decided, researched,
measured and learned, and where that lives in the code. This page is the contract
for the shape of every page in `docs/`. `tool/tasks/docbase.py check` verifies the
mechanical part of it in the gate, and `tool/tests/test_structure.rs` holds the
top-level folders.

## Where each thing lives

| Folder | What it holds | Identity |
|---|---|---|
| `adr/<theme>/` | architecture decision records, numbered in one sequence across the themes | `adr-NNNN` |
| `tasks/task-NN-<slug>/` | one folder per task: `README.md` with the plan, execution, delivery and what stayed open; the phases of a long task as pages beside it | `task-NN`, a phase `task-NN/<page>` |
| `experiments/<slug>/` | one folder per experiment: question, setup, results, limits, and the task pages that hold the full record | `experiment-<slug>` |
| `research/` | references studied, comparisons, the thesis and the program of the generative runtime, the first technical plan, the journal of the autonomous cycle | `research-<name>`, and the names of the thesis, program and plan |
| `learnings/` | what a problem or a delivery taught | `learning-<name>` |
| `ref/` | `architecture/` of each subsystem, `operations/` of the repository and of this archive, `platforms/` | `architecture-…`, `operations-…`, `platform-…` |
| `img/` | images the pages show | the path |

`CHANGELOG.md`, beside this tree, is the history in time: newest first, one section per
date, each with the entries written when the change landed and the records of that
day linked. A new task, ADR, experiment, research note or learning gets its line under
its date, and the check fails when one is missing.

The ADR themes are `generated`, `importing`, `interface`, `recording`, `rendering`,
`semantics`, `structure` and `workflows`. A new ADR copies
[`adr/TEMPLATE.md`](../../adr/TEMPLATE.md) and is listed in
[`adr/README.md`](../../adr/README.md). Task and ADR numbers are identity, not
chronology: the order in time comes from the `date` field.

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
| `date_basis` | where the date came from: `header` (the source header), `text` (a date written in the body), `git` (a commit), `reconstructed` (inferred from several sources), `editorial`, `undated` |
| `topics` | subjects in kebab-case, few and shared between pages |
| `relations` | `id`s of linked pages: dependencies, the ADR a task took, the research behind it |
| `evidence` | commits (short sha) that back what the page states |
| `source` | the source path (or list) at `source_revision`; `reconstruction` when the page was written from code and commits |
| `source_revision` | the commit at which the source was read |
| `revised` | the date of the last editorial revision |
| `review` | `pending` (migrated, not yet read whole), `reviewed` (read whole and checked), `generated` (an index) |

A source header whose date was revised after the fact (commit `704412f1` rewrote
dates from 2026 back to 2021) keeps that date with `date_basis: header`;
[reading the dates](../../CHANGELOG.md#reading-the-dates) explains the case. A date only
changes when another source contradicts it, and then `date_basis` says which.

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

The `README.md` of `tasks/`, `experiments/`, `research/`, `learnings/` and `ref/`, and
`CATALOG.json`, are generated by `tool/tasks/docbase.py index`; they are not edited by
hand. `adr/README.md` is the
ADR index kept by hand, and the check fails when an ADR is missing from it.
