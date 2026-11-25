---
id: adr-0091
title: "the tools' contracts live in .config"
project: plev
type: adr
status: proposed
date: 2026-11-24
date_basis: git
recorded: 2026-11-25
topics: [workspace, process]
relations: [task-106, adr-0022, docs-conventions]
evidence: [b6759681, a1c697a3]
source: reconstruction
source_revision: 0a8598f9
revised: 2026-11-25
review: reviewed
---

# the tools' contracts live in .config

Proposes `.config/` as the one place for the contracts of the tools that check the repository: hakari's, and lychee's, nextest's and typos' beside it. The root no longer has `.design`, `.devops` or `.secops` ([task-106](../../../Tasks/task-106-build-weight-and-reuse/task-106-build-weight-and-reuse.md)).

## context

The owner removed `.devops`, `.secops` and `.design` from the root (`205fc5f8`, `f7695626`) and confirmed on 2026-11-25 that they are gone on purpose. `.devops/check` held three contracts the gate still read: `lychee.toml` (the link check), `nextest.toml` and `typos.toml`. The tree's own test, the gate, CI, the readme and the atlas kept naming the folders, so the test failed and the link check had nothing to read. The task first restored the folders (`a1c697a3`), reading the deletion as an accident; that was undone.

## decision

- `.config/` holds a tool's contract when the tool reads it from the repository: `hakari.toml`, `lychee.toml`, `nextest.toml`, `typos.toml`.
- The gate and CI name those paths. nextest finds `.config/nextest.toml` with no flag, the place it looks by default.
- `tool/tests/test_structure.rs` lists the root without the three folders.

## consequences

The link check of the gate runs again (4056 links, none broken). The root has one dot directory for tool contracts where it had two.

The decision the owner took is the removal. The place of the three contracts is this record's proposal, and the other answer is in the alternatives.

## alternatives

- Removing the three contracts with the folders: the gate loses its link check, and nextest and typos run on their defaults.
- Restoring the folders: tried and undone, against the owner's decision.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0022: one knowledge tree, and a minimal root](../0022-one-knowledge-tree-and-a-minimal-root/0022-one-knowledge-tree-and-a-minimal-root.md)
- [task-106: what a build directory weighs, and a warm start for a new checkout](../../../Tasks/task-106-build-weight-and-reuse/task-106-build-weight-and-reuse.md)
- [Conventions of the docs archive](../../../Research/Workflows/docs-conventions.md)
<!-- /related -->
