---
id: adr-0083
title: "feature unification reduces duplicate compilations"
project: plev
type: adr
status: accepted
date: 2026-10-27
date_basis: git
topics: [workspace, performance]
relations: [task-102]
source: reconstruction
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
evidence: [540e218d, 706dfa82]
---

# feature unification reduces duplicate compilations

## Context

Running an isolated package and testing the workspace activated different
features of the same dependencies, creating extra artifacts. In the measurement
of `540e218d`, each member recompiled crates that were already compiled in the
workspace: 54 in `-p arcade`, 65 in `-p uigallery` and 7 in `-p commit`.

## Decision

Cargo-hakari generates `rust/unions` for the configured macOS and Linux targets
(`.config/hakari.toml`, lines 24 to 27). The members depend on this crate. The
gate checks the configuration: `tool/tasks/fullcheck.sh` and a CI job run
`cargo hakari generate --diff` and `cargo hakari manage-deps --dry-run`
(`540e218d`). Wasm stays outside this platform union, because it uses another
artifact directory. `proc-macro2` is excluded to respect the difference in
`span-locations` between the engine and the proc-macros (`.config/hakari.toml`,
lines 29 to 37; `rust/engine/Cargo.toml`, lines 35 to 43).

## Consequences

New dependencies and members require regenerating and verifying the union. The
development profile keeps line tables for the local code and removes debug
information from the dependencies (`Cargo.toml`, lines 164 to 175; `706dfa82`).
This reduces the information available when debugging a dependency internally;
a session that needs variables uses `--profile debugging` (lines 177 to 181).

## Alternatives

Accepting the different sets keeps the recompilation. Cargo's own unification
option, cited in `.config/hakari.toml` (lines 6 and 7), required nightly in the
recorded round; the decision adopted the tool configured in the project.

## Evidence

- [`540e218d`](https://github.com/brennercruvinel/plev/commit/540e218d61b8529784a9c958ce7bfd6210af60ce)
- [`706dfa82`](https://github.com/brennercruvinel/plev/commit/706dfa82c907975688797748001e2236bdb8faef)
