---
id: task-103
title: "task-103: runcheck, every executable in the repository runs"
project: plev
type: task
status: done
date: 2026-10-27
date_basis: git
recorded: 2026-10-31
topics: [process, platform, workspace]
relations: [task-98, task-102]
evidence: [205fc5f8]
source: reconstruction
source_revision: c4f6c343
revised: 2026-10-30
review: reviewed
---

# task-103: runcheck, every executable in the repository runs

Records `tool/tasks/runcheck.sh`, the check that runs each program in the repository (applications, demos, command-line crates and tools) to confirm that it starts and does its work on this machine. The gate proves compilation and tests; runcheck proves what the gate does not prove. The script entered main through commit `205fc5f8` (2026-10-27), under the message "Ogrnaize folders", and no page in the archive described it before this one. The archive keeps no run of it and no report from it.

## Problem and execution

The script header states the difference: the gate (`tool/tasks/fullcheck.sh`) proves that the code compiles and the tests pass; runcheck proves that each program starts and does its work (`tool/tasks/runcheck.sh:2-5`).

- GUI windows (`gui`, `runcheck.sh:99-139`): the program passes if it is still alive after `WAIT` seconds, defaulting to 6 (`:28`). On macOS, each window is captured with `screencapture` and counted by distinct colors on a grid below the title bar; a window of a single color does not pass (`:116-137`). Each capture gets up to three attempts, three seconds apart (`:118-122`).
- Command lines (`cli`, `runcheck.sh:141-156`): passes with exit code 0, or with an expected line when the `expect` argument is given. The limit is 300 s (`:147`).
- Inputs: the script uses the repository's fixtures (`:41-43`). Two programs have no input, and the script says so instead of failing: `llm-bench` without `PLEV_LLM_MODEL` (`:188-198`) and `urnadbapp` without the sibling checkout `../urna` (`:215-218`). `intent` runs only the rules, with `PLEV_CLAUDE` pointing to a nonexistent program, so as not to spend a model call (`:21-22`, `:177`).
- Coverage: the workspace is compiled first (`:158-167`); tools and command-line crates (`:170-180`); the wasm build of `uigallery-web` (`:181-187`); eight demos and the windowed applications (`:206-214`); the agent bridge with `PLEV_MIRROR=1` (`:226-238`).
- Report: `target/runcheck/report.md`, with one line per program: its type, result and evidence (`:240-251`). The exit code is 1 when something fails.

## Limits

- Window capture exists only on macOS. Elsewhere, a window passes only by staying alive (`:129`).
- A window that passes showed at least two colors on a grid; that does not prove the content (`:134-135`).
- Command lines without `expect` pass on the exit code and do not check the output.
- No run result is recorded in the archive; this page describes the script, not a run of it.

## Lesson in shell

The script uses `perl -e 'alarm shift; exec @ARGV'` as its time limit, with a comment saying it is a timeout that every Unix has (`runcheck.sh:50-51`). The script does not say why it does not use `timeout`; the likely reading is that the command does not come by default on macOS, which is an inference and is not written in the code.

## Evidence

- `205fc5f8`: adds `tool/tasks/runcheck.sh` (251 lines), together with the removal of `.devops/check/*.toml` and the addition of `rust/llmrun/Cargo.lock` (2026-10-27). It is the only commit that touches the file (`git log -- tool/tasks/runcheck.sh`).

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [task-102: reusable builds and web commands](../task-102-builds-and-web-commands/task-102-builds-and-web-commands.md)
- [task-98: workspace reorganization](../task-98-workspace-reorganization/task-98-workspace-reorganization.md)
<!-- /related -->
