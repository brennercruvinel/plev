---
id: adr-0080
title: "the workspace separates libraries, apps, demos and tools"
project: plev
type: adr
status: accepted
date: 2026-10-09
date_basis: git
recorded: 2026-10-31
topics: [workspace, git]
relations: [task-98]
evidence: [d3e65544, 9e31564e, 6c4d6ff3]
source: reconstruction
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# the workspace separates libraries, apps, demos and tools

## Context

The tree under `crates/` gathered libraries and applications (`crates/engine`, `crates/comps`, `crates/showcase` and `crates/urnaui`, among others; the crate map in the [change record of task-98](../../../Tasks/task-98-workspace-reorganization/change-record.md#mapa-de-crates)). The migration also had to keep imports, manifests, scripts and tests consistent.

## Decision

Libraries live in `rust/`, applications in `apps/`, demos in `demo/` and tools in `tool/`. The widgets belong to the engine (`rust/engine/src/widgets`); imported formats converge on `import` (`rust/import`). The [ATLAS](../../../ATLAS.toml) describes the tree and the [TERMS](../../../TERMS.toml) records the renames.

## Consequences

Public paths change even when the item's behavior stays the same: the report records that no behavior changed and that 13 of 19 crates have an API identical line by line, with the other 6 explained in the report itself (lines 49 and 61 to 69). Comparing the API requires translating the paths, without hiding incompatibilities from consumers. Separate move commits make it possible to trace where the files came from: `d3e65544` contains only R100 renames. The proofs and limits are in [task 98](../../../Tasks/task-98-workspace-reorganization/task-98-workspace-reorganization.md).

## Alternatives

Keeping libraries and apps mixed together would preserve the old paths, but would not meet the target structure documented in the migration.

## Evidence

- [`d3e65544`](https://github.com/brennercruvinel/plev/commit/d3e6554467ad0a84ab9b6eaba01a996c17527354)
- [`9e31564e`](https://github.com/brennercruvinel/plev/commit/9e31564eb75aa261d87f2a9f734a7ebdac97193c)
- [`6c4d6ff3`](https://github.com/brennercruvinel/plev/commit/6c4d6ff3cf292b9d9e815394843933de03c26918)

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0088: libraries are plain, and a mobile build asks for its crate type](../0088-libraries-are-plain-and-a-mobile-build-asks-for-its-crate-type/0088-libraries-are-plain-and-a-mobile-build-asks-for-its-crate-type.md)
- [task-98: workspace reorganization](../../../Tasks/task-98-workspace-reorganization/task-98-workspace-reorganization.md)
<!-- /related -->
