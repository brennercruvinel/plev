---
id: task-98
title: "task-98: workspace reorganization"
project: plev
type: task
status: done
date: 2026-10-09
date_basis: git
recorded: 2026-10-31
topics: [workspace, git]
relations: [adr-0080, task-51, task-52]
evidence: [3161bb89, d3e65544, 9e31564e, 6c4d6ff3]
source: [reconstruction, docs/tasks/task-98-workspace-reorganization/change-record.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
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
renames (the map's 1,870 moves less the nine files that kept their path,
counted in `migration.toml` at `3161bb89`), the API comparisons and the tests of the round. These are historical
results, not a new run of the suite. The report also records that Android and
the `.urna` explorer leg were not verified on that machine. Task 99 covers later
work on this integration; it does not retroactively prove the Android build.

## Rust lessons

Preserving behavior does not make a reorganization compatible with every
consumer: changing the public path of a type can break imports. The report's API
comparison separated three cases: a false positive from the normalizer, in
functions named after a renamed module; new group modules, additions only; and
`comps` inside the engine ([proofs](change-record.md#provas)). The
[ADR 0080](../../adr/Structure/0080-workspace-by-responsibility/0080-workspace-by-responsibility.md) records the
structure adopted.

## Evidence

- [`3161bb89`](https://github.com/brennercruvinel/plev/commit/3161bb89b0bb6b490b83d10b72c3f2e50d85fd72)
- [`d3e65544`](https://github.com/brennercruvinel/plev/commit/d3e6554467ad0a84ab9b6eaba01a996c17527354)
- [`9e31564e`](https://github.com/brennercruvinel/plev/commit/9e31564eb75aa261d87f2a9f734a7ebdac97193c)
- [`6c4d6ff3`](https://github.com/brennercruvinel/plev/commit/6c4d6ff3cf292b9d9e815394843933de03c26918)

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0080: the workspace separates libraries, apps, demos and tools](../../adr/Structure/0080-workspace-by-responsibility/0080-workspace-by-responsibility.md)
- [task-103: runcheck, every executable in the repository runs](../task-103-runcheck-every-executable-runs/task-103-runcheck-every-executable-runs.md)
- [task-51: workspace restructure and naming rebrand](../task-51-workspace-restructure/task-51-workspace-restructure.md)
- [task-52: modularization by single responsibility principle](../task-52-srp-modularization/task-52-srp-modularization.md)
- [task-99: explorer integration and platform fixes](../task-99-integration-and-portability/task-99-integration-and-portability.md)
- [Reorganization of the repository tree (issue #130, PR #131)](change-record.md)
<!-- /related -->
