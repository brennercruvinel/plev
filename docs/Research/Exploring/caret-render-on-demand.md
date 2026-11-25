---
id: experiment-caret-render-on-demand
title: "caret blink as a deadline: frames drawn with a focused, idle field"
project: plev
type: experiment
status: reference
date: 2026-09-23
date_basis: text
recorded: 2026-10-31
topics: [rendering, performance, input]
relations: [task-73/phase-1-measurements, task-82, task-82/phase-a, task-82/phase-b, adr-0042]
evidence: [d97a375, a6fec8d]
source: reconstruction
source_revision: c2e3f8cd
revised: 2026-10-31
review: reviewed
---

# caret blink as a deadline: frames drawn with a focused, idle field

The experiment checked whether a focused text field with nothing typed in it can stop drawing 60 frames per second for a caret that changes visibility about twice a second. The blink became a deadline (`comps::Wake`): the tick says whether a visible change is due now and when the next one is, and the shell sleeps until then. In the measure harness, scenario g went from 120 of 120 turns drawn to 4 of 120, which is 2 frames per second. In a real window, the focused commit field of the ide went from 720 frames in 6 s to 11, one per blink change, in the final round. The sources record no CPU figure after the change.

## Question

Quotes from the Portuguese sources are translated here.

- The problem ([task-82 README](../../Tasks/task-82-caret-blink-render-on-demand/task-82-caret-blink-render-on-demand.md), objective): a focused, still text field draws 60 frames per second for a caret that changes visibility about twice a second. That breaks the render-on-demand rule, "no visible change, no frame".
- The acceptance criteria (same page, "aceite"): scenario g falls to 2 or 3 frames per second drawn with the field focused and still; typing keeps drawing every key; the caret keeps blinking, checked with an injected clock on the blink changes; the gate stays green.
- The sources measure frames. They state no CPU criterion for an idle caret, and the task-82 pages contain no CPU figure.

## Setup

- **Scenario g** has the same name in both measurement pages: "focused field, no typing" in the [task-73 phase 1 page](../../Tasks/task-73-incremental-dependency-system/phase-1-measurements.md), and "campo focado, nada digitado" in the [task-82 phase a page](../../Tasks/task-82-caret-blink-render-on-demand/phase-a.md). In task-73 it is the Todo page with `todo.input` focused by a click and no input afterwards.
- **Measure harness, before** (task-73 phase 1): release build, 120 measured turns after 10 warm-up turns, fixed tick of 16.67 ms, 1200x800 logical at scale 2, theme core, Apple M4.
- **Measure harness, after** (task-82 phase a): release build, 120 turns at 60 Hz (2 s), 1200x800 at scale 2, theme core. The harness now models the shell it replaces: "a deadline discounts a fixed DT per turn and draws on the turn it is due; the tick of a drawn turn receives the time since the last drawn turn." The task-82 README names the scenario `showcase --example measure`. Warm-up turns, machine and OS are not recorded.
- **Real window, showcase** (task-82 phase a): `cargo run -p uigallery Todo` with `RUST_LOG=showcase=debug`. The field was focused by a real click (CGEvent) on its centre, found through the accessibility tree; then 6 s with no input. A debug line per frame records the interval and the `Wake`. Machine: not recorded.
- **Real window, ide and Git page** (task-82 phase b): debug build (`cargo build -p idestudio -p uigallery`). The before build is base `83967a0` with one per-frame debug line added to the ide's `main.rs` and one `git worker woke` line in the showcase's `user_event`. The after build is the branch. The app opens a draft repository (one commit, one modified file, one untracked file). Keys are CGEvents posted to the app's pid after it is activated. Frame lines are stamped on arrival by the harness, and git processes are counted by a counting `git` on the `PATH`. The harness waits 4 s without hardware input and marks a round as disturbed if the mouse or keyboard moves during the window. Round 1 ran with `GIT_OPTIONAL_LOCKS=0` in the environment to isolate the caret and the toast; round 2 is the final code, without it. The display rate varied between rounds, 120 or 240 per second.
- **Commits**: phase a integrated in `main` at `d97a375` (eight commits, rebased); phase b merged at `a6fec8d` (eleven commits).

## Results

### Scenario g in the measure harness

Sources: [task-82 phase a](../../Tasks/task-82-caret-blink-render-on-demand/phase-a.md), the measurement table (its "antes" column cites task-73), and [task-73 phase 1, "summary"](../../Tasks/task-73-incremental-dependency-system/phase-1-measurements.md) for scenario g.

| scenario | before (task-73) | after (task-82 phase a) |
|---|---|---|
| g. focused field, no typing | 120 of 120 turns drawn | 4 of 120 turns drawn: 2 frames per second, the blinks at 0.53 s, 1.06 s, 1.59 s and 2.12 s |
| a. type in the field | 120 of 120 | 120 of 120 (each key draws) |
| f. page left alone | 0 frames | 0 frames |

The letter f means different things in the two pages. See Limits.

### CPU with the caret idle

Sources: [task-73 phase 1, "summary"](../../Tasks/task-73-incremental-dependency-system/phase-1-measurements.md) and its phase 1b "summary", scenario g.

- Before, phase 1 (three runs): cpu, the frame's p50 minus the gpu wait's p50, was **62 to 94 µs** per frame.
- Before, phase 1b (ten runs with timestamps): median cpu **50 µs** (46 to 423); five runs without timestamps: median **41 µs**.
- After: **not recorded**. The task-82 pages give frame counts only. Every turn was drawn before the change, and 4 of 120 are drawn after it; comparing per-frame figures across that difference is my reading of the pages, not a figure from them.

### Real window: showcase Todo page

Source: [task-82 phase a, "na janela real"](../../Tasks/task-82-caret-blink-render-on-demand/phase-a.md). The before count is from [ADR 0042](../../adr/Rendering/0042-a-tick-asks-for-a-deadline/0042-a-tick-asks-for-a-deadline.md), "consequences".

- Before the click, with the page still, the frames that appeared had `wake now false next None`. They were requested from outside (the git worker, the accessibility bridge), and none came from the tick.
- After the click, 6 s untouched: **11 frames**, all `now true`, with `since_last` between 0.529 and 0.531 s and `next` around 528 ms. One frame per blink change, "1.9 per second", and no extra frame between changes.
- Before the change, the same state: the phase a page says it "drew 60 per second" (it gives no count). ADR 0042 says a focused, untouched todo field "draws 11 frames in 6 s, one per flip, where it drew **360**."

### Real window: ide commit field and showcase Git page

Source: [task-82 phase b, "fase b: medição"](../../Tasks/task-82-caret-blink-render-on-demand/phase-b.md), rounds 1 and 2.

| case | before, round 2 (final) | after, round 2 (final) | before, round 1 | after, round 1 |
|---|---|---|---|---|
| a. ide, commit field focused by `c`, 6 s still | 720 frames, every frame; 18 git processes | 11 frames, one per change; 0 git processes | 1431 | 11 |
| c. showcase, Git page, field focused by `c`, 6 s still | 720 frames, every frame | 11 frames, one per change | 1423 | 11 |

- Round 2 also checks: five keys after `c` in the ide each drew one frame, and each key's frame asks for the next change in about 0.529 s.
- The idle ide, with no field focused and no toast (round 2, 6 s): 34 frames and 17 `git status` processes before; 0 frames and 0 processes after. This is not the caret. Phase b names the cause: the file watcher reacted to the `.git/index.lock` that `git status` took, which ran `git status` again. The fix is in the same branch.
- **The before counts differ between rounds, and the page states why.** The ide field was 1431 in round 1 and 720 in round 2; the Git page was 1423 and 720. The phase a page repeats "eram 1431" for the ide. Phase b says the continuous "before" numbers follow the display rate, not the app. ADR 0042 gives the ide case as "720 at 120 Hz".

### Tests and gate

- Phase a: gate green with 1985 tests passing, 0 failed, 6 ignored. Eleven of the added tests cover `Wake` and `Blink`, all with an injected `dt` (phase a, "testes").
- Phase b: gate green with 2128 passing, 0 failed, 12 ignored, with the urnaui leg running (phase b, "fechamento da fase b").

### Outcome

- Scenario g meets the acceptance criterion of 2 or 3 frames per second (measured: 2 per second). Typing draws every key (120 of 120). The caret keeps blinking: the real window shows one frame per change, and injected-clock tests check the changes.
- Phase b extended the same `Wake` to the ide, the Git page and urnaui. The phase b audit had found the README's promise "no change, no frame" broken in the ide: it drew every frame with the commit field focused or a toast on screen (phase b, "fase b aberta").

## Limits

- **The 4-of-120 result comes from a model of the shell.** The after harness "models the shell it replaces" (phase a). The real-window counts are the check on the real shell. Phase b notes that the `tool/bench/measure/src` example still models the shell without the catch-up; its numbers are phase a's, and phase b did not change them.
- **The letter f differs between the pages.** In task-82 phase a, f is "página parada, deixada em paz" (left alone): 0 frames before. In task-73 phase 1, the f row is "idle redraw", where each turn asks for a frame with nothing changed: 120 of 120 drawn. The same phase 1 page also says "left alone for 120 turns, the loop drew 0 frames", which is the value phase a uses. The numbers are not in conflict; the labels are.
- **The before counts in the real window come from four sources with different values**: 360 (ADR 0042, showcase Todo), "60 per second" (phase a, no count), 1431 and 720 (ide, phase b rounds 1 and 2), and 1423 and 720 (Git page, phase b rounds 1 and 2). Phase b attributes the spread to the display rate, 120 or 240 per second between rounds.
- In phase b round 1, the "before" rounds for the field and the toast were flagged because hardware input happened during the window; the page says they "already drew every frame". Round 2 is recorded as having all rounds clean.
- Warm-up turns, the machine and the OS for the task-82 measure runs are not recorded.
- The phase a page describes one real-window run on the showcase Todo page and reports no repeats; the other real-window figures are the phase b rounds listed above.
- The caret is not fixed everywhere. Per the sources, these still draw without a visible change:
  - the `EditorView` without a focus notion keeps a blink pending while visible: the Editor page asks for one flip every 0.53 s while visible, even with nobody editing (phase a, debts);
  - the dock caret in the demo asks for continuous frames while its panel is open (phase a; ADR 0042);
  - the accessibility bridge asks for frames with nothing visible changed (phase a; phase b).
  Phase b's closing lists these as debts "without a task yet", and says none of them breaks a promise with a real user today.
- A toast's spring still draws every frame for about 1.7 s in each direction (phase b). The toast is not the caret; it is listed here because it is in the same measurement.

## Records

- [task-82 README](../../Tasks/task-82-caret-blink-render-on-demand/task-82-caret-blink-render-on-demand.md): the objective, the acceptance criteria, the orchestrator's review and the two phases.
- [task-82 phase a](../../Tasks/task-82-caret-blink-render-on-demand/phase-a.md): the `Wake` and `Blink` clock, the shell in `WaitUntil`, the measurement of scenario g (120 to 4 of 120), and the real-window run on the showcase Todo page.
- [task-82 phase b](../../Tasks/task-82-caret-blink-render-on-demand/phase-b.md): the `Wake` in the ide, the Git page and urnaui; the two real-window rounds; the git change to optional locks.
- [task-73 phase 1 measurements](../../Tasks/task-73-incremental-dependency-system/phase-1-measurements.md): the before numbers for scenario g (frames and cpu) and for f.
- [ADR 0042](../../adr/Rendering/0042-a-tick-asks-for-a-deadline/0042-a-tick-asks-for-a-deadline.md): the decision the mechanism implements; the "360" figure is in its consequences.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0042: a tick asks for a deadline, not for every frame](../../adr/Rendering/0042-a-tick-asks-for-a-deadline/0042-a-tick-asks-for-a-deadline.md)
- [task-82: the caret cannot request 60 frames per second](../../Tasks/task-82-caret-blink-render-on-demand/task-82-caret-blink-render-on-demand.md)
- [task-73 phase 1: what each stage of a frame costs](../../Tasks/task-73-incremental-dependency-system/phase-1-measurements.md)
- [task-82 phase a: the caret blink becomes a deadline](../../Tasks/task-82-caret-blink-render-on-demand/phase-a.md)
- [task-82 phase b: ide, Git page and urnaui draw only when something changes](../../Tasks/task-82-caret-blink-render-on-demand/phase-b.md)
<!-- /related -->
