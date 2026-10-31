---
id: task-99
title: "task-99: explorer integration and platform fixes"
project: plev
type: task
status: done
date: 2026-10-16
date_basis: git
recorded: 2026-10-31
topics: [urna, platform, process]
relations: [adr-0026, task-76, task-98]
evidence: [05fe993c, 701d2993, c836ea73, ac2906a2, 14fd20dd, 7f4b9994]
source: reconstruction
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-99: explorer integration and platform fixes

After the reorganization, the explorer had to follow the paths of the urna project, and the workspace still had differences between the native builds.

## Implementation

The query embedder looks for the scripts `potionqry.py` and `presetqry.py` in `rust/bridge/python/urna` (the repository path) and in `<data root>/urna/python`, where `urna setup` installs them ([embed.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/apps/urnadbapp/src/backend/workers/embed.rs), lines 34 and 175 to 177; `701d2993`). In the current tree there is no versioned `potionqry.py`: the search is in the code and the scripts come from the urna project. The received vector is still subject to validation of model, dimension and hash (`embed.rs`, lines 215 to 247). The interface receives messages from the worker; creating an embedding process does not happen on the thread that draws the window (`embed.rs`, line 18).

The shared executor drains stdout and stderr from the start, on separate threads. Waiting for the child's output before consuming the pipes can create a deadlock when the output exceeds the system buffer. The timeout kills and reaps the process. The executor was born in `05fe993c` (2026-09-18), in `crates/urnaui/src/model/subprocess.rs`, with this draining; `d3e65544` moved it to `apps/urnadbapp`. Code: [apps/urnadbapp/src/backend/workers/subprocess.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/apps/urnadbapp/src/backend/workers/subprocess.rs).

## Platform and reproduction

The fixes also selected the xdg-portal backend of `rfd` on Linux (`c836ea73`), restricted the background import to macOS (`ac2906a2`), handled the case where the reduced-motion preference is missing on macOS (`7f4b9994`), and made the staged pages of the baselines (silences) stop offering the Claude installed on the host (`14fd20dd`).

These changes address different causes and do not amount to certifying all platforms. The commits below delimit each fix.

## Lesson in Rust

A `cfg` has to keep up with both the implementation and the imports, as `ac2906a2` shows. A task moved to a worker can still block that worker: subprocesses require an output contract, concurrent draining and controlled termination. See [ADR 0026](../../adr/rendering/0026-worker-results-wake-the-event-loop.md).

## Evidence

- [`05fe993c`](https://github.com/brennercruvinel/plev/commit/05fe993ce2eac37e92f9b79bb523329da3ffdf6e)
- [`701d2993`](https://github.com/brennercruvinel/plev/commit/701d2993266b090acdfc13f641d88df2d57c74e6)
- [`c836ea73`](https://github.com/brennercruvinel/plev/commit/c836ea732afd915e14f8109144ebded99a4ccf34)
- [`ac2906a2`](https://github.com/brennercruvinel/plev/commit/ac2906a27cea431a90551f5818fda94840ad1dd7)
- [`14fd20dd`](https://github.com/brennercruvinel/plev/commit/14fd20dd77f405d7ca71270c07067f4be55e054b)
- [`7f4b9994`](https://github.com/brennercruvinel/plev/commit/7f4b99941b987b3fca31333804423a6689daf91f)
