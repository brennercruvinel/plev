return {
  id  = "conventions",
  typ = "contract",
  sts = "living",
  dom = "process",
  dat = "2026-06-13",
  ttl = "the operating contract: tests, naming, hygiene, lint, errors, docs, sync",
  txt = [[
the single instruction source is AGENTS.md; this file holds the
conventions it points to. read it at task start, follow it, and update it
whenever a new convention is established. lua so it parses with luajit and
reads as a graph node, the project's own format.
]],

  -- every change ships real tests, no mocks: happy path, error path, one
  -- edge case minimum, against real artifacts (rendered scenes, measured
  -- pixels, golden fixtures). nothing merges without executable proof.
  -- visual claims require pixel measurement, not "looks the same"
  -- (kdb/how-to/validate-visuals-by-pixel.md).
  tests = "real, no mocks; happy + error + one edge min; pixel-measure visual claims",

  -- no ui before the logic behind it is implemented and tested. chart
  -- geometry, state machines, codecs, parsers: pure modules with unit
  -- tests first, pixels second.
  backend_before_ui = "logic implemented and tested before pixels",

  -- the dividing line, so agents do not oscillate: under src/ (any crate)
  -- module directories and files are snake_case, rust mandates it and the
  -- compiler gives no real choice. kdb/, doc/, assets and every other
  -- non-source path are kebab-case english. a crate package name in
  -- Cargo.toml may be kebab (narrate-macro), but the lib name normalizes
  -- to snake, so the import is `use narrate_macro`. prefer short, global
  -- names; never sacrifice clarity for brevity. renames are mv commands,
  -- fix every touched import, run the suite.
  naming = "src/ = snake (compiler-mandated); kdb/doc/assets = kebab; crate name kebab ok, lib/import snake; renames as mv + fix imports + test",

  -- hard limit 369 lines per source file, target ~220 for new files. split
  -- oversize PRODUCTION code by single responsibility. EXEMPT from the
  -- count: #[cfg(test)] blocks (in-file unit tests are idiomatic rust and
  -- must never force a split; a test block pushing a file to 380 is not a
  -- single-responsibility signal), generated files, lockfiles. the limit
  -- is a tax on big production units, never a tax on testing well.
  file_hygiene = "369 hard / ~220 target on production code; #[cfg(test)], generated, lockfiles exempt; split by single responsibility",

  -- table stakes for rust, the lint catches the rest for free. the gate:
  -- cargo clippy --workspace --all-targets -- -D warnings, cargo fmt
  -- --check, cargo test --workspace, and the wasm check. allow a lint per
  -- item with a one-line reason, never crate-wide. shared lint levels live
  -- in [workspace.lints]; crates inherit with [lints] workspace = true.
  lint = "clippy --all-targets -D warnings + fmt --check + test --workspace + wasm check all green; allow per-item with reason, never crate-wide",

  -- one error posture so every agent's error path looks the same. library
  -- crates: typed Result with thiserror; library code never panics on a
  -- user-input path (degrade: log and empty output). binaries: anyhow, or
  -- a plain message plus non-zero exit for simple apps. panic only for an
  -- internal invariant that user input cannot reach.
  errors = "lib: Result + thiserror, never panic on user input; bin: anyhow; panic only for internal invariants",

  -- forbidden by default. any unsafe block or unsafe fn carries a
  -- // SAFETY: comment naming the invariant that makes it sound; that is
  -- the one comment kind allowed beyond stating a constraint. unsafe_op_
  -- in_unsafe_fn is on in [workspace.lints].
  unsafe_policy = "unsafe forbidden by default; each exception a // SAFETY: comment naming the invariant",

  -- something becomes its own crate (not a module under src/) when it has a
  -- distinct identity and name, its own dependencies, is independently
  -- reusable or testable, or must compile separately (proc-macro). short of
  -- that it stays a module. edition 2024, rust-version 1.85, both inherited
  -- from [workspace.package]; every crate is publish = false with a
  -- description.
  crate_boundary = "crate when: own identity/deps/reusable/separate-compile; else module. edition 2024, rust-version 1.85 from workspace.package",

  -- diataxis style. all lowercase except acronyms. no emoji. no em-dash
  -- (comma, semicolon, period or hyphen). no decorative markdown. every doc
  -- opens with yaml frontmatter (type, tags, date, commit or status). a
  -- design note that turns out wrong gets a correction on top, never a
  -- delete.
"diataxis, lowercase, yaml frontmatter, no emoji, no em-dash; wrong notes corrected on top, not deleted",

  -- the source of truth for what exists is Cargo.toml members and the
  -- examples/ dir, never a doc. tests/arc_sync_guard.rs enforces it: every
  -- workspace crate must be named in arc.yaml, arc.md and README, every
  -- example in arc.yaml, or the build fails naming the missing one. so
  -- drift is a red test, not a silent lie. arc.yaml is the canonical
  -- machine map, arc.md the human projection, arc.mmd the frame-flow view;
  -- on a wording divergence arc.yaml wins. keep them small.
  arc_sync = "Cargo.toml + examples/ are the truth; tests/arc_sync_guard enforces every crate/example is named in the arc docs + README; arc.yaml canonical on wording",

  -- keep README.md current after user-facing changes (new crate, new run
  -- command, new format behavior). keep this conventions file current when
  -- a new convention is established.
  doc_sync = "README current after user-facing changes; this file current on a new convention",

  -- on finishing a task: full audit of every session change, no summary,
  -- from devops, code quality and secops angles. write a temporary markdown
  -- manifest under tmp to track executed tasks. find dead code, stale
  -- generated files, items in wrong folders; fix or report. run tests after.
  audit_on_finish = "full session audit (devops/quality/secops) + tmp manifest; fix or report; run tests",

  -- commit messages: plain english or portuguese, no conventional-commits
  -- prefix; body explains the why, the diff shows the what. agents do not
  -- commit; the orchestrator commits thematically. code comments state
  -- constraints the code cannot show (and // SAFETY: for unsafe), nothing
  -- else.
  style = "commit body explains why; agents never commit; comments state constraints only",

  -- engine rules live in the manual; the short list is in AGENTS.md.
  engine_manual = "kdb/how-to/code-against-the-plev-engine.md",
}
