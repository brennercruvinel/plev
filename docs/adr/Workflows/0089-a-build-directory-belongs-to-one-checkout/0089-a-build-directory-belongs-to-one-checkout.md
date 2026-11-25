---
id: adr-0089
title: "a build directory belongs to one checkout, starts from a clone and has a ceiling"
project: plev
type: adr
status: accepted
date: 2026-11-24
date_basis: git
recorded: 2026-11-25
topics: [workspace, performance, agents]
relations: [task-106, operations-orchestrate-coding-agents-on-this-repo, adr-0083, research-build-directories-in-large-rust-workspaces]
evidence: [38fa0c21, cbce2d89]
source: reconstruction
source_revision: 0a8598f9
revised: 2026-11-25
review: reviewed
---

# a build directory belongs to one checkout, starts from a clone and has a ceiling

Decides how the build directories are kept: one per checkout, a new one cloned copy on write from the dependencies already compiled, and a ceiling that removes only what the workspace's own crates wrote. `tool/tasks/builddir.sh` does the three ([task-106](../../../Tasks/task-106-build-weight-and-reuse/task-106-build-weight-and-reuse.md)).

## context

A build directory of the workspace reached 8 GB with the tests compiled alone, and every agent's checkout has its own; the directories together reached 50 GB on one machine. Nine tenths of a directory is what the workspace's own crates wrote (4.5 of 5.2 GB when measured): the test executables, the incremental data and the libraries. The dependencies are the small part and the slow one to rebuild.

One directory shared by every checkout failed here on 2026-09-23: cargo names a workspace crate the same in every checkout and judges it fresh by mtime, so a build took an artifact compiled from another checkout's sources.

## decision

- One build directory per checkout stays the rule.
- `builddir.sh seed <dir>` starts a new one from the main checkout's `target/debug`, cloned copy on write (`cp -c` on APFS, `cp --reflink=always` on linux), and then removes the workspace's own artifacts from the clone. The second step is not optional, for the reason the shared directory failed.
- Where it cannot clone (another volume, a filesystem without copy on write) the seed refuses and says what a whole copy would cost. `--copy` takes the copy.
- `builddir.sh trim` is the ceiling: over 12 GB by default, it runs `cargo clean --workspace` for the host and for every target triple with a folder. Without `--yes` it only lists. The dependencies stay.
- `builddir.sh size` says what each directory weighs.

## consequences

A new worktree builds `arcade` with 6 crates in 12 s, against 192 crates and 87 s from an empty directory; the seed takes 5 s and no disk until a file changes. A trim took a `target/` from 12.3 GB to 1.5 GB, and the next build compiles the workspace only.

A seeded directory counts in full in `du`, which cannot tell a clone from a copy. The clone needs the two directories on one volume.

## alternatives

- A shared directory: the failure above; cargo also locks the whole directory, and `-Z checksum-freshness` and `-Z fine-grain-locking` are unstable in cargo 1.99.1.
- sccache, which Zed's CI uses: it saves time and not disk, and the seed gives the time back without a service.
- Removing a whole directory over a limit, the second threshold of Zed's `script/clear-target-dir-if-larger-than`: the dependencies would be compiled again for no disk the trim does not already free.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0083: feature unification reduces duplicate compilations](../0083-feature-unification/0083-feature-unification.md)
- [task-106: what a build directory weighs, and a warm start for a new checkout](../../../Tasks/task-106-build-weight-and-reuse/task-106-build-weight-and-reuse.md)
- [how large rust workspaces keep a build directory small](../../../Research/Ecosystem/build-directories-in-large-rust-workspaces.md)
- [how to orchestrate coding agents on this repo](../../../Research/Workflows/orchestrate-coding-agents-on-this-repo.md)
<!-- /related -->
