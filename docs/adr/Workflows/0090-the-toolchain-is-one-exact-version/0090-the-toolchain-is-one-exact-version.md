---
id: adr-0090
title: "the toolchain is one exact version, read from one file"
project: plev
type: adr
status: accepted
date: 2026-11-24
date_basis: git
recorded: 2026-11-25
topics: [workspace, process, performance]
relations: [task-106, research-build-directories-in-large-rust-workspaces]
evidence: [90447a43]
source: reconstruction
source_revision: 0a8598f9
revised: 2026-11-25
review: reviewed
---

# the toolchain is one exact version, read from one file

Decides that `rust-toolchain.toml` names an exact compiler version and that every CI job installs what that file names. The first pin is 1.99.1, the compiler `stable` already was on the day ([task-106](../../../Tasks/task-106-build-weight-and-reuse/task-106-build-weight-and-reuse.md)).

## context

`rust-toolchain.toml` said `stable`, and the CI jobs installed `stable` through an action of their own. A build directory keeps its artifacts by compiler version, so each update of `stable` left a whole older copy in every directory, with nothing in the repository saying it had happened. The machine and CI could also build with different compilers on the same commit.

## decision

- `rust-toolchain.toml` names the exact version, with the components (`rustfmt`, `clippy`, `rust-src`) and the wasm32 target the gate uses.
- Every job of `.github/workflows/gatecheck.yml` runs `rustup toolchain install`, with no argument: rustup reads the same file.
- A new compiler enters by a commit that changes the file.

## consequences

A change of compiler is visible in the history and happens for every checkout at the same commit, so the day the build directories go cold is chosen. Nobody gets a new stable by accident, and nobody gets it for free either: the bump is work someone does.

Not verified: the CI side. No workflow has run on the repository since 2026-10-16.

## alternatives

- `stable`: the context above.
- The pin in the file and a different installer in CI: two places to keep equal.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [task-106: what a build directory weighs, and a warm start for a new checkout](../../../Tasks/task-106-build-weight-and-reuse/task-106-build-weight-and-reuse.md)
- [how large rust workspaces keep a build directory small](../../../Research/Ecosystem/build-directories-in-large-rust-workspaces.md)
<!-- /related -->
