---
id: task-98
title: "task-98: workspace reorganization"
project: plev
type: task
status: done
date: 2026-10-09
date_basis: git
topics: [workspace, git]
relations: [adr-0080, task-51, task-52]
source: [reconstruction, docs/tasks/task-98-workspace-reorganization/change-record.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
evidence: [3161bb89, d3e65544, 9e31564e, 6c4d6ff3]
---

# task-98: workspace reorganization

The October reorganization gathered libraries into `rust/`, applications into
`apps/`, demonstrations into `demo/` and tools into `tool/`. The sequence kept
moves and content changes in separate commits: commit `d3e65544` contains only
R100 renames, 1,861 files.

## Problem and execution

The previous structure gathered libraries and applications in `crates/`:
`crates/engine` and `crates/comps` became `rust/engine`, `crates/showcase`
became `apps/uigallery` and `crates/urnaui` became `apps/urnadbapp`
([change record](change-record.md#mapa-de-crates), "mapa de crates"). The migration
map set the destination of the files before the moves (`3161bb89`). Then came
manifests (`9e31564e`), module declarations, paths, literals, tests and docs.
The widgets of `comps` moved to the engine (`engine::widgets`); the importers
were gathered into `import`; the names of crates and modules are in
[TERMS](../../TERMS.toml).

## Result and limits

The [delivery report](change-record.md) records 1,861 R100
renames, the API comparisons and the tests of the round. These are historical
results, not a new run of the suite. The report also records that Android and
the `.urna` explorer leg were not verified on that machine. Task 99 covers later
work on this integration; it does not retroactively prove the Android build.

## Rust lessons

Preserving behavior does not make a reorganization compatible with every
consumer: changing the public path of a type can break imports. The report's API
comparison separated three cases: a false positive from the normalizer, in
functions named after a renamed module; new group modules, additions only; and
`comps` inside the engine ([proofs](change-record.md#provas)). The
[ADR 0080](../../adr/structure/0080-workspace-by-responsibility.md) records the
structure adopted.

## Evidence

- [`3161bb89`](https://github.com/brennercruvinel/plev/commit/3161bb89b0bb6b490b83d10b72c3f2e50d85fd72)
- [`d3e65544`](https://github.com/brennercruvinel/plev/commit/d3e6554467ad0a84ab9b6eaba01a996c17527354)
- [`9e31564e`](https://github.com/brennercruvinel/plev/commit/9e31564eb75aa261d87f2a9f734a7ebdac97193c)
- [`6c4d6ff3`](https://github.com/brennercruvinel/plev/commit/6c4d6ff3cf292b9d9e815394843933de03c26918)
