---
type: reference
audience: [agents, contributors]
status: living
domain: process
date: 2026-06-24
tags: [conventions, rust, contract, tests, naming, lint, errors, docs]
---

# rust conventions

the operating contract for code on this repository: tests, naming, hygiene,
lint, errors, docs, sync. the single instruction source is AGENTS.md; this
file holds the conventions it points to. read it at task start, follow it,
and update it whenever a new convention is established.

this used to be `doc/.conventions/conventions.lua` (a luajit graph node). it
was migrated to markdown so there is no extra build/parse step in the app;
the content is unchanged. the keys below are the same quick keys the lua
table exposed.

## tests

real, no mocks. every change ships executable proof: happy path, error path,
and one edge case minimum, against real artifacts (rendered scenes, measured
pixels, golden fixtures). nothing merges without it. visual claims require
pixel measurement, not "looks the same" (kdb/how-to/validate-visuals-by-pixel.md).

## backend before ui

no ui before the logic behind it is implemented and tested. chart geometry,
state machines, codecs, parsers: pure modules with unit tests first, pixels
second.

## naming

the dividing line, so agents do not oscillate: under `src/` (any crate)
module directories and files are snake_case, rust mandates it and the
compiler gives no real choice. `kdb/`, `doc/`, assets and every other
non-source path are kebab-case english. a crate package name in Cargo.toml
may be kebab (`narrate-macro`), but the lib name normalizes to snake, so the
import is `use narrate_macro`. prefer short, global names (3 to 9 chars where
it stays clear); never sacrifice clarity for brevity. renames are `mv`
commands: fix every touched import, run the suite.

## file hygiene

hard limit 369 lines per source file, target ~220 for new files. split
oversize production code by single responsibility. exempt from the count:
`#[cfg(test)]` blocks (in-file unit tests are idiomatic rust and must never
force a split; a test block pushing a file to 380 is not a single-
responsibility signal), generated files, lockfiles. the limit is a tax on
big production units, never a tax on testing well.

## lint

table stakes for rust, the lint catches the rest for free. the gate:
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt
--check`, `cargo test --workspace`, and the wasm check. allow a lint per
item with a one-line reason, never crate-wide. shared lint levels live in
`[workspace.lints]`; crates inherit with `[lints] workspace = true`.

## errors

one error posture so every agent's error path looks the same. library
crates: typed `Result` with thiserror; library code never panics on a
user-input path (degrade: log and empty output). binaries: anyhow, or a
plain message plus non-zero exit for simple apps. panic only for an internal
invariant that user input cannot reach.

## unsafe

forbidden by default. any unsafe block or unsafe fn carries a `// SAFETY:`
comment naming the invariant that makes it sound; that is the one comment
kind allowed beyond stating a constraint. `unsafe_op_in_unsafe_fn` is on in
`[workspace.lints]`.

## crate boundary

something becomes its own crate (not a module under `src/`) when it has a
distinct identity and name, its own dependencies, is independently reusable
or testable, or must compile separately (proc-macro). short of that it stays
a module. edition 2024, rust-version 1.85, both inherited from
`[workspace.package]`; every crate is `publish = false` with a description.

## docs

diataxis style. all lowercase except acronyms. no emoji. no em-dash (use a
comma, semicolon, period or hyphen). no decorative markdown. every doc opens
with yaml frontmatter (type, tags, date, commit or status). a design note
that turns out wrong gets a correction on top, never a delete.

## arc sync

the source of truth for what exists is Cargo.toml members and the
`crates/engine/examples/` dir, never a doc. `crates/engine/tests/arc_sync_guard.rs`
enforces it: every workspace crate must be named in arc.yaml, arc.md and
README, every example in arc.yaml, or the build fails naming the missing
one. so drift is a red test, not a silent lie. arc.yaml is the canonical
machine map, arc.md the human projection, arc.mmd the frame-flow view; on a
wording divergence arc.yaml wins. keep them small.

## doc sync

keep README.md current after user-facing changes (new crate, new run
command, new format behavior). keep this conventions file current when a new
convention is established.

## audit on finish

on finishing a task: full audit of every session change, no summary, from
devops, code quality and secops angles. write a temporary markdown manifest
under tmp to track executed tasks. find dead code, stale generated files,
items in wrong folders; fix or report. run tests after.

## style

commit messages: plain english or portuguese, no conventional-commits
prefix; the body explains the why, the diff shows the what. agents do not
commit; the orchestrator commits thematically. code comments state
constraints the code cannot show (and `// SAFETY:` for unsafe), nothing else.

## learnings

experiments live in refs/, the gitignored working area (see AGENTS.md):
examples, demos, clones of other apps, proofs of concept. importing those and
testing them against plev is the main source of new rules. when an experiment
closes and its work migrates to an example or a crate, fold what it taught
back into this file and the sibling rule files (clippy.toml, typos.toml,
nextest.toml) in the same change: a new typing or naming rule, a spell-checker
false positive, a lint threshold, a flaky-test finding. these files are
living; an undocumented learning is a learning lost.

## engine manual

engine rules live in the manual; the short list is in AGENTS.md. the manual:
kdb/how-to/code-against-the-plev-engine.md.
