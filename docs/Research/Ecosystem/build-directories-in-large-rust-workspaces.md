---
id: research-build-directories-in-large-rust-workspaces
title: "how large rust workspaces keep a build directory small"
project: plev
type: research
status: reference
date: 2026-11-24
date_basis: text
recorded: 2026-11-25
topics: [workspace, performance, research]
relations: [task-106, adr-0083, adr-0087, adr-0088, adr-0089, adr-0090]
source: reconstruction
source_revision: 0a8598f9
revised: 2026-11-25
review: reviewed
---

# how large rust workspaces keep a build directory small

What five large Rust repositories do about the weight of `target/`, read on their main branches on 2026-11-24 while plev's own reached 8 GB with the tests compiled alone. None of them has a trick for the directory: they link few executables, keep the core library a plain rlib and put a ceiling on the directory by script. plev took those three and one of its own, the clone ([task-106](../../Tasks/task-106-build-weight-and-reuse/task-106-build-weight-and-reuse.md)).

## What was counted

Crates in the workspace and integration test targets, the files cargo links as an executable each.

| repository | crates | integration test targets | toolchain |
|---|---|---|---|
| Zed | 261 | 11 | pinned to an exact version (1.100.0) |
| uv | 91 | 8, one of them `tests/it/main.rs` | not read |
| Bevy | 83 | 10 | not read |
| ruff | 55 | 35 | not read |
| rust-analyzer | 45 | 10 | not read |
| plev, before | 38 | 127 | `stable` |
| plev, after | 38 | 29 | pinned to 1.99.1 |

plev was the one out of the curve: more test executables than the other five together, for the fewest crates.

## What each one does

- **Tests.** Zed writes its tests inside each crate, so a crate links one test executable. uv gathers its integration tests in `tests/it/main.rs`, one executable with a module per file. plev keeps the files where they are and names them from `tests/suite.rs` ([adr-0087](../../adr/Workflows/0087-a-crates-integration-tests-are-one-executable/0087-a-crates-integration-tests-are-one-executable.md)).
- **The core library.** Zed's `gpui` is a plain library with no `crate-type`; iOS lives in a crate of its own, `gpui_ios`, which is the one that declares `staticlib`. plev's engine is plain too, and the mobile build asks for the type on its command line ([adr-0088](../../adr/Structure/0088-libraries-are-plain-and-a-mobile-build-asks-for-its-crate-type/0088-libraries-are-plain-and-a-mobile-build-asks-for-its-crate-type.md)).
- **A ceiling.** Zed's CI runs `script/clear-target-dir-if-larger-than 350 200`: over 200 GB it runs `cargo clean --workspace`, over 350 GB it removes the directory. They bound the size; they do not make it small. plev's `tool/tasks/builddir.sh trim` is the first threshold, at 12 GB ([adr-0089](../../adr/Workflows/0089-a-build-directory-belongs-to-one-checkout/0089-a-build-directory-belongs-to-one-checkout.md)).
- **CI.** Zed sets `CARGO_INCREMENTAL=0` and uses sccache in a bucket, for build time. plev did not take sccache: it saves time and not disk, and a copy-on-write clone of the dependencies gives a new checkout the time back with no service.
- **Profiles.** Zed's dev profile carries reduced debug information, with a separate `dbg` profile for the full one, the shape of plev's `debugging`. uv uses `line-tables-only`, as plev does.
- **Features.** Zed has no workspace-hack crate on its main branch. plev's `unions` already made one demo reuse another's build ([adr-0083](../../adr/Workflows/0083-feature-unification/0083-feature-unification.md)).

## What was tried and does not apply

Zed mirrors its dev profile in `build-override`, so a crate needed by a build script and by the program is not compiled twice. Tried on plev: the 11 crates that appear twice stayed. They are two versions of one crate asked for from above (`objc2` 0.5 and 0.6, `bitflags` 1 and 2), not a host copy and a target copy.

## Limits

The counts are of one day and one branch of each repository. The toolchain was read for Zed only. Nothing here measures those repositories' build directories; the only sizes measured are plev's, in the task.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0083: feature unification reduces duplicate compilations](../../adr/Workflows/0083-feature-unification/0083-feature-unification.md)
- [ADR 0087: a crate's integration tests are one executable](../../adr/Workflows/0087-a-crates-integration-tests-are-one-executable/0087-a-crates-integration-tests-are-one-executable.md)
- [ADR 0088: libraries are plain, and a mobile build asks for its crate type](../../adr/Structure/0088-libraries-are-plain-and-a-mobile-build-asks-for-its-crate-type/0088-libraries-are-plain-and-a-mobile-build-asks-for-its-crate-type.md)
- [ADR 0089: a build directory belongs to one checkout, starts from a clone and has a ceiling](../../adr/Workflows/0089-a-build-directory-belongs-to-one-checkout/0089-a-build-directory-belongs-to-one-checkout.md)
- [ADR 0090: the toolchain is one exact version, read from one file](../../adr/Workflows/0090-the-toolchain-is-one-exact-version/0090-the-toolchain-is-one-exact-version.md)
- [task-106: what a build directory weighs, and a warm start for a new checkout](../../Tasks/task-106-build-weight-and-reuse/task-106-build-weight-and-reuse.md)
<!-- /related -->
