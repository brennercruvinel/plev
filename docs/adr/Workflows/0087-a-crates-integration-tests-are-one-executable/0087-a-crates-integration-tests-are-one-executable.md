---
id: adr-0087
title: "a crate's integration tests are one executable"
project: plev
type: adr
status: accepted
date: 2026-11-24
date_basis: git
recorded: 2026-11-25
topics: [workspace, performance, process]
relations: [task-106, adr-0083, research-build-directories-in-large-rust-workspaces]
evidence: [df245521, aff2388a, 57e62523, 92c1cd0a]
source: reconstruction
source_revision: 0a8598f9
revised: 2026-11-25
review: reviewed
---

# a crate's integration tests are one executable

Decides that the files under a crate's `tests/` are modules of one test target, `tests/suite.rs`, instead of an executable each. The workspace went from 127 integration test targets to 29 with the same tests passing, and the test executables of a build from 186 and 3.2 GB to 88 and 1.1 GB ([task-106](../../../Tasks/task-106-build-weight-and-reuse/task-106-build-weight-and-reuse.md)).

## context

cargo links every file directly under `tests/` as an executable of its own, and each one carries the crate and everything under it: 60 to 80 MB for a test of the gallery. With 38 crates the workspace had 127 such targets; the gallery's 27 alone weighed 1.5 GB. Read on their main branches on 2026-11-24, Zed has 261 crates and 11 such targets, uv 91 and 8, Bevy 83 and 10.

## decision

- A crate with more than one file under `tests/` sets `autotests = false` and declares one `[[test]]` named `suite`, at `tests/suite.rs`. Each file is a module of it (`mod capture;`); a folder that was a target enters by `#[path = "x/main.rs"] mod x;`. No file moves.
- A helper the files share (`tests/support/`) is declared once, in `suite.rs`, and a file reaches it by `use crate::support;`.
- One file's tests run by the module's name: `cargo test -p engine --test suite capture::`. A test that runs itself as a child process names itself the same way (`--exact mirror::app_child`).
- A target stays apart only for a reason its manifest states. Today: the ones that need a process of their own (`edit_literal` in the gallery rewrites the process's literal baseline; `device_loss` and `text_audit` in the engine each install the process's logger), and the folders that were already one executable and whose modules name each other by `crate::` (`rerun`, `gate_bytes`, `session`, `queries`, `ui`).
- The five targets of `tool/tests` stay, because the check scripts run them by name.

## consequences

A build links one test executable per crate, so a change to a library relinks one file where it relinked up to 27.

The tests of a crate now share a process. What is global to a process is shared between files that used to be alone: the literal baseline, the logger, the environment. A test that depends on one of those either stops depending on it or gets a target of its own, with the reason written in the manifest.

A new file under `tests/` does nothing until `suite.rs` names it. The compiler does not warn about a file nobody names; the count of tests before and after is the check.

## alternatives

- Tests inside each crate, as Zed writes them: it would move every file and turn tests of the public interface into tests with access to the private one.
- A folder with its own `main.rs`, as uv's `tests/it/`: the same executable, at the cost of moving the files. A root file beside them names them where they are, and each file keeps its history.
- Leaving it: the weight is paid again in every build directory, and every agent's checkout has one.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0083: feature unification reduces duplicate compilations](../0083-feature-unification/0083-feature-unification.md)
- [task-106: what a build directory weighs, and a warm start for a new checkout](../../../Tasks/task-106-build-weight-and-reuse/task-106-build-weight-and-reuse.md)
- [how large rust workspaces keep a build directory small](../../../Research/Ecosystem/build-directories-in-large-rust-workspaces.md)
<!-- /related -->
