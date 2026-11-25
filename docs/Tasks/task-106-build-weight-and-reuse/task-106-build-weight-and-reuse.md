---
id: task-106
title: "task-106: what a build directory weighs, and a warm start for a new checkout"
project: plev
type: task
status: in-progress
date: 2026-11-24
date_basis: text
recorded: 2026-11-24
topics: [workspace, performance, process, agents]
relations: [task-102, operations-orchestrate-coding-agents-on-this-repo]
source: "https://github.com/brennercruvinel/plev/issues/185"
source_revision: f7695626
revised: 2026-11-24
review: pending
---

# task-106: what a build directory weighs, and a warm start for a new checkout

A build directory of plev reached 8 GB with the tests compiled alone, and every agent's checkout has its own. This task makes each directory hold less and lets a new checkout start from the dependencies already compiled. The release profile and what is distributed did not change.

[POCS] `cargo test --workspace --no-run` in an empty build directory, before and after: 88 test executables where there were 186, 5.0 GB where there were 8.1 GB.

## What weighed

Running one demo after another in the same checkout already compiled nothing twice: `camera` after `arcade` built 1 crate in 2 s, the work of `rust/unions` ([task-102](../task-102-builds-and-web-commands/task-102-builds-and-web-commands.md)). The weight was the workspace's own crates. In a `target/` of 5.2 GB, `cargo clean --workspace` removed 4.5 GB; the dependencies were 724 MB.

`cargo test --workspace --no-run` in an empty directory, on the branch point (`f7695626`), took 8.1 GB:

| part | weight |
|---|---|
| 186 linked test executables | 3.2 GB |
| incremental data | 1.8 GB |
| `libengine.a` and `libuigallery.a`, written by a desktop build | 523 MB |

The workspace had 38 crates and 127 integration test targets: cargo links every file under `tests/` as an executable of its own, each with the engine inside (60 to 80 MB for a gallery test). For scale, read on their main branches on 2026-11-24: Zed has 261 crates and 11 such targets, uv 91 and 8, Bevy 83 and 10.

## What changed

- **The engine is a plain library.** As a `cdylib` and `staticlib` of its own it linked a 196 MB archive and a dylib on every desktop build, and nothing loaded either: the mobile apps link the engine into their own library. The extra crate types also took the hash out of `libengine.rlib`'s file name, the cause of the collision between the two workspaces that the gate's urnadbapp leg works around. A rebuild of the engine for one demo went from 14 s and 485 MB to 11 s and 296 MB.
- **The apps ask for their mobile crate type when a mobile build runs.** `uigallery` and `urnadbapp` are plain libraries; `tool/tasks/mobileapp.sh` runs `cargo rustc --crate-type staticlib` for iOS and hands the same to cargo-ndk for Android, and the artifact names Xcode and Gradle read are the same.
- **One engine for every member.** `demo/graphs` was the one member with the engine's default features off, which compiled a second engine, and it did not compile at all: `styling/theme/adapt.rs` read a module of the `accessibility` feature without its `cfg`. The engine compiles without its default features again, the gate and CI check that shape (`cargo clippy -p engine --no-default-features -- -D warnings`), and graphs takes the engine the others take.
- **A crate's integration tests are one executable.** In each crate with several files under `tests/`, the files are modules of `tests/suite.rs` (`autotests = false` and one `[[test]]`). No file moved. 127 targets became 29.
- **`tool/tasks/builddir.sh`.** `size` says what each build directory weighs; `trim` removes what the workspace's crates wrote when a directory is over a limit and keeps the dependencies (it took a `target/` from 12.3 GB to 1.5 GB); `seed` gives another checkout a build directory cloned copy on write from the main one, with the workspace's own artifacts removed from the copy.

## One file's tests

A command an earlier document gives as `cargo test -p <crate> --test <file>` is now:

```
cargo test -p <crate> --test suite <file>::
cargo test -p <crate> --test suite <file>::<test name>
```

The documents under `docs/Tasks` and `docs/adr` keep the commands they were written with; they record what was run then. The comments and messages inside the test files were updated.

## What a shared process showed

Cargo runs one test executable after another, so files that were executables apart never shared a process. As modules of one executable they do, and three things depended on the old shape:

- `uigallery`'s `edit_literal` rewrites the process's literal baseline (`engine::runtime::hot_reload::literals`) and `view_source` reads the same title's literal. `edit_literal` keeps an executable of its own.
- `engine`'s `device_loss` and `text_audit` each install their own recorder as the process's logger and assert on what the engine logged; `log::set_logger` takes one per process. Both keep an executable of their own.
- A test that runs its own executable again as a child, by test name, names the module now: `mirror::app_child` in the gallery, `env::from_env_in_a_child` and `env::scene_from_env_in_a_child` in `socket`.

A manifest says why each target apart is apart. `set_reduced_motion` is per thread and needed nothing. The folders that were already one executable and whose modules name each other by `crate::` (`idestudio`'s `rerun`, `motion`'s `gate_bytes`, `record`'s `session`, `replay`'s `queries`, `inspector`'s `ui`) stay as they were, and so do the five tests of `tool/tests`, which the check scripts run by name and which weigh 1 to 1.6 MB each.

## The same tests, before and after

Counted crate by crate, integration tests only, the old layout against the new:

| crate | executables | passed | failed | ignored |
|---|---|---|---|---|
| uigallery | 24 to 2 | 200 | 4 | 6 |
| engine | 18 to 3 | 79 | 1 | 5 |
| screen | 14 to 1 | 48 | 3 | 1 |
| idestudio | 12 to 2 | 73 | 1 | 1 |
| socket | 10 to 1 | 95 | 0 | 0 |
| accord | 10 to 1 | 167 | 0 | 0 |
| plevir | 7 to 1 | 68 | 1 | 0 |
| offers | 7 to 1 | 62 | 0 | 0 |
| import | 6 to 1 | 28 | 3 | 2 |
| motion | 3 to 2 | 11 | 0 | 0 |
| gitdrv | 2 to 1 | 22 | 0 | 2 |
| agenteval | 2 to 1 | 11 | 0 | 0 |

The counts are the same on both sides, and so are the names of the tests that fail. The failures were not this task's: they failed on the branch point, before any test moved. The next section is what became of them.

## The eighteen that failed on the branch point

`cargo test --workspace` on the branch point had 18 failing tests, and no workflow had run on the repository since 2026-10-16 to show them. Each was traced to the commit that left it behind before anything was changed, and no golden was written again without that. None was a regression in what the engine draws; one shows a real defect on screen, noted below.

| tests | left behind by | what was done |
|---|---|---|
| `plevir` schema file, `idestudio` workbench fixture | `540e218d` (2026-10-27): `unions` asks `serde_json` for `preserve_order`, so a map prints in the order of insertion | the two tests sort the keys of what they print; the kept files are byte for byte what they were |
| `import` html and react reports, the `text_mid` tolerance | `08e608a1` (task-104m): the theme settles `text_dim` to AA, the gap to `text_mid` went from 51 levels to 49.13 | the tolerance reads 24.56 where it read 25.5, one line in each golden and the constant |
| `inspector` and `replay` dependency lists | `540e218d`: every member got the `unions` line | the lists name `unions`, which holds no code |
| `inspector` commit scenario | `2ca6ded9` (task-104g): the code face is JetBrains Mono, the row wraps | the test reads the drawn text across its rows |
| `tool/tests` root layout | `205fc5f8` and `f7695626` deleted `.devops`, `.secops` and `.design`, on purpose | the test no longer wants them; the lychee, nextest and typos contracts `.devops/check` held live in `.config`, and the gate's link check runs again (4056 links, none broken) |
| `tool/tests` crate homes | an empty untracked directory on one machine | the test lists what git tracks, as the rest of its file does |
| `engine` text audit | `2ca6ded9`: 82644 more bytes of fonts | the golden's first line, the only one that differed |
| `screen` slider tests (3) | `19fcf1d3` (task-104e): the slider maps its travel by the thumb | the test helpers use the thumb's width |
| `uigallery` mirror, record, view_source | `259859a0` and `48464cd3` (task-104d, 104h): the Card has no photo, the sidebar peeks | the image leg is the Attachment page, the test pins the sidebar, the text is the Tooltip page's title |
| `uigallery` scene_codec | `9b0cc681` (task-104b): the first image decoded from a PNG lands in a second slot of the store | an image node is compared by its pixels, not by its handle |

A nineteenth, `screen`'s caret test, failed or passed by the machine's speed: `84987d83` (task-104m) fades a text field's edge on the wall clock, and the helper that waits for transitions gave up after 600 simulated frames, a few milliseconds. The helper also asks for two seconds of real time now.

After these, on the task's branch: `cargo test --workspace` gives 4116 passed, none failed, 29 ignored; clippy, fmt, the wasm check, hakari, the docs checker and the link check are clean.

## Measured after

The same command in an empty directory, on the task's branch:

| | before | after |
|---|---|---|
| the directory | 8.1 GB | 5.0 GB |
| linked test executables | 186, 3.2 GB | 88, 1.1 GB |
| `deps/` | 5.4 GB | 2.8 GB |
| incremental data | 1.8 GB | 1.6 GB |
| static archives of the mobile builds | 523 MB | none |

The time is not compared: the first run used every core (155 s) and the second four build jobs (174 s), by choice, on a machine that was busy.

What is left is mostly the incremental data, which this task did not touch (an agent's directory already builds with `CARGO_INCREMENTAL=0`), and the 88 executables: the unit tests of each library and binary, and the test targets kept.

A new checkout, through `builddir.sh seed`: the seed took 5 s and no disk, and `cargo build -p arcade` there compiled 6 crates in 12 s with four jobs, against 192 crates and 87 s from an empty directory.

## What was decided against

- **One build directory shared by every checkout.** It failed here on 2026-09-23 (cargo names a workspace crate the same in every checkout and judges it fresh by mtime), cargo still locks the whole directory, and `-Z checksum-freshness` and `-Z fine-grain-locking` are unstable in cargo 1.99.1. One build directory per checkout stays the rule; `builddir.sh seed` spares the new one its cold start.
- **The engine as a dylib, or every demo in one executable.** A demo's executable is 24 MB and links in 2 s.
- **sccache.** It saves time and not disk, and the seed gives the time back without a service.
- **Mirroring the dev profile in `build-override`**, as Zed does to avoid compiling crates for the host and the target twice. Tried: the 11 crates that appear twice here are two versions of the same crate asked for from above (`objc2` 0.5 and 0.6, `bitflags` 1 and 2), not a host copy and a target copy.

## What was not verified

- A real iOS or Android build with the plain libraries. No mobile target and no cargo-ndk is installed on the machine of this task; `cargo rustc --crate-type staticlib` and `cdylib` were run on the host triple, and cargo-ndk's forwarding of a `rustc` subcommand was read in its source. Two defects a review found in the mobile script were fixed and tried on stand-ins only: the iOS build read the library from a fixed `target/` (it asks cargo now, so `CARGO_TARGET_DIR` is honored), and the Android build no longer cleared an old `libengine.so` from a reused `jniLibs`.
- Sharing one build directory between the root workspace and urnadbapp's, now that the engine's file name carries the hash. The gate's leg keeps its own subdirectory.
- The packaging of the web build of `graphs`. `tool/tasks/webtarget.sh graphs build` compiles the wasm release build to the end; trunk then needs wasm-bindgen 0.2.123, the lockfile's version, and the machine of this task has 0.2.129.
- The gate on a clean runner. The `gatecheck` workflow is disabled and its last runs, on 2026-10-16, did not start for the account's billing, so the pinned toolchain, the linux leg of the matrix and the structural regression job were not run by CI.

## Open

- The inspector at 1280x800 draws the commit's sha cut over two rows since the code face changed (`2ca6ded9`), and `value_column` in its panel takes the first space after a label, so a two-word label never finds its value column. Seen, not changed.
- `apps/urnadbapp` still builds into a directory of its own (4.0 GB). Sharing the root's, or joining the workspace, is [issue 191](https://github.com/brennercruvinel/plev/issues/191).
- ADR 0069 still states 25.5 and 51 levels for `text_mid`; the theme makes them 24.56 and 49.13 since task-104m.
- The engine's image store keeps an image decoded from a PNG and the same pixels given raw in two slots.

## Closed in the task

- `.design`, `.devops` and `.secops` are gone from the root for good, the owner's decision. The task first restored them, reading their deletion as an accident because the tree's own test, the gate, CI, the readme and the atlas still named them; that was undone. What `.devops/check` held (the lychee, nextest and typos contracts) is in `.config`, beside hakari's, and every reference points there. nextest finds its file there with no flag.

- The toolchain is pinned: `rust-toolchain.toml` names 1.99.1, the compiler `stable` already was, and every CI job runs `rustup toolchain install`, which reads the same file. The CI change is not verified: no workflow has run on the repository since 2026-10-16.
- `human_agent`'s commit author in `idestudio` followed `GIT_AUTHOR_NAME` when a session exported it. The test compares with the author git gave the commit, and the gate drops the identity variables before its tests.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [task-102: reusable builds and web commands](../task-102-builds-and-web-commands/task-102-builds-and-web-commands.md)
- [how to orchestrate coding agents on this repo](../../Research/Workflows/orchestrate-coding-agents-on-this-repo.md)
<!-- /related -->
