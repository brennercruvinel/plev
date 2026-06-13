---
type: adr
status: accepted
tags: [workspace, cargo, organization, structure]
date: 2026-06-12
commit: c2a90f1
---

# engine at root, libraries and apps in crates, demos in examples

## context

the workspace had grown inconsistent: crate names mixed kebab and snake
case, two demos were full crates, single-purpose files sat loose at the top
of src, shaders lived in a top-level directory away from the gpu code that
loads them, and every crate pinned its own dependency versions.

## decision

three tiers, one rule each:

- the engine is the root crate `plev` (`[package]` at the workspace root,
  src is the engine). it is the thing every other tier depends on.
- `crates/` holds separate libraries and apps with their own identity and
  dependencies: git, ide, lot, monster, narrate, narrate-macro, parser,
  rope, showcase. names are short and global; source module directories
  stay snake_case because rust requires it.
- `examples/` holds demonstrations of the engine. cargo discovers them
  with no Cargo.toml; they are not products. demos that were crates
  (scene-3d, snake-game) moved here.

cargo hygiene: `[workspace.package]` for shared version, edition,
rust-version, authors, repository; `[workspace.dependencies]` as the single
source of version truth so crates use `<dep>.workspace = true` with no
drift; `[workspace.lints]` inherited by every crate; tuned `[profile]`.
every crate is `publish = false` (private) with a description. shaders moved
into `src/gpu/shaders/` next to the module that includes them. loose src
files moved into the module that owns them (gpu_vec into gpu, scroll into
input, ime and lifecycle into platform).

## consequences

- the layout answers "what is this" on sight: root is the engine, crates
  are products, examples are demos.
- one version bump in one place updates every crate. clippy and lints are
  uniform across the workspace.
- a mechanical rename pass that rewrote `crate::X` references reached one
  example module by mistake (counter's own lifecycle), caught only by
  `cargo test` because `cargo build --workspace` does not compile examples.
  the lesson is recorded: verify example-touching changes with `cargo
  test`, not `cargo build`.

## avoid

- do not put a demo in crates/ or a product in examples/.
- do not pin a dependency version in a crate when it belongs in
  `[workspace.dependencies]`.
- do not trust `cargo build --workspace` to prove example code compiles;
  only `cargo test` builds examples.
