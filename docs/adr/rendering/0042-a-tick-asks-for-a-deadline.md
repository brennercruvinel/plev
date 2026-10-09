---
type: adr
status: accepted
tags: [comps, engine, animation, render-on-demand, caret, showcase, winit]
date: 2026-09-23
---

# a tick asks for a deadline, not for every frame

## context

render on demand says: no visible change, no frame. a caret at rest
changes twice a second, but the only way a widget had to ask for its
flip was a `tick(dt) -> bool` meaning "keep the frames coming", so a
focused text field drew sixty frames a second for a change that happens
twice (task-73, scenario g: 120 of 120 turns drawn). with
`ControlFlow::Wait` a tick that answered `false` let the loop sleep, and
nothing woke it for the flip; hence the shortcut.

the engine's `FrameClock` made the fix impossible as it stood: its `dt`
is clamped to 0.1 s so a stall does not jump a spring to its end, and a
loop that slept 0.53 s until the flip woke up believing 0.1 s had passed.

## decision

`comps::Wake` is what a tick asks of the loop: `now` (this tick changed
something visible, draw) and `next: Option<Duration>` (when the next tick
is due; `Some(ZERO)` is continuous motion, `None` is nothing pending).
wakes compose with `merge`: any `now` wins, the earliest `next` wins.
`comps::wake::Blink` is the caret clock the text widgets share (the phase
inside the half-period, exact through a long stall, no panic on a bad
`dt`); `TextInput`, `TextField` and `EditorView` answer `wake(dt)`; the
pages and the `ShowcaseView` merge their children.

`AnimationTick` gained `since_last`, the real unclamped seconds since the
previous tick (0.0 on the first), beside the clamped `dt` motion keeps
reading. the showcase shell passes `since_last` to the view's `wake`, and
paces the loop from the answer: `Some(ZERO)` requests the next frame,
`Some(d)` sets `ControlFlow::WaitUntil(now + d)`, `None` sets `Wait`;
`StartCause::ResumeTimeReached` invalidates and returns the flow to
`Wait`, so a missed redraw cannot spin on a deadline in the past. any
event (input, the git worker, the screen reader, the agent port) ends the
wait early, as winit documents, so nothing that asked for a frame waits
for the caret. the `PerfHud` contract (continuous frames while shown)
does not change.

## consequences

- measured: scenario g of `showcase --example measure` went from 120 of
  120 turns drawn to 4 of 120; in the real window a focused, untouched
  todo field draws 11 frames in 6 s, one per flip, where it drew 360.
  typing still draws every key; an idle page draws nothing.
- `tick -> bool` stays where a consumer outside this task reads it (the
  ide's commit form, the comps editor example, the settle loops of rec
  and headless), meaning "pending" or "in motion" as each doc comment
  says; the ide and urnaui adopt `Wake` in phase b, after task-62's lab
  left the ide surface. it stays in the showcase's Git page too: the
  page lifts the `bool` of `Workbench::tick` to `Wake::moving`
  (`apps/uigallery/src/views/apps/git/desktop/mod.rs`,
  `apps/uigallery/src/views/compositor/render.rs`), so with that page visible and
  the commit field focused, or an ide toast up, the showcase draws every
  frame again.
- an `EditorView` without a focus notion keeps a blink pending while
  visible (one flip every 0.53 s); the dock caret stays continuous with
  its panel open; the git worker and the accessibility bridge still ask
  for stray frames with nothing visible changed. all three are recorded
  in the task file as debts, not hidden.
- phase b (branch `task/82b-wake-everywhere`) supersedes the two
  bullets above for the ide, the showcase's Git page, urnaui and the git
  worker: all of them use the same mechanism. the ide's chain
  (`Workbench` -> `WorkspaceView` -> panels, commit form, toasts) and
  urnaui's eight view ticks answer `wake(dt) -> Wake`; both shells pace
  the loop as the showcase does (`since_last`, `WaitUntil`, `Wait`,
  `ResumeTimeReached`). the Git page merges `GitPage::wake` only while it
  shows, and the git worker's `UserEvent::Wake` applies its events
  whichever page shows but draws only with the Git page on screen.
  measured in a real window: a focused ide commit field draws 11 frames
  in 6 s where it drew every frame (720 at 120 Hz); the same on the Git
  page; three worker results with another page showing draw 0 frames
  where they drew 2 or 3.
- a toast at rest is a deadline too: `ToastManager::wake` asks for every
  frame while a toast springs in or out and, between, for the instant
  the first one on screen dismisses itself. the ide's
  `|| !toasts.is_empty()` is gone.
- `since_last` covers the time before an event as well: a change made
  between two frames (a switch flipped, a toast pushed, a field
  focused) would take the whole gap on the next tick, and a switch
  flipped after three idle seconds jumped to its end. so each shell
  catches up as the loop wakes: `new_events` ticks what animates with
  the real elapsed time before anything of that iteration lands (input,
  a worker result, a screen reader's or an agent's request). the
  catch-up draws nothing unless it changed something visible, and the
  next frame counts only the time after the event. a tick counts all the
  time it is given; code that drives widgets without a shell (tests,
  headless, settle loops) catches up itself before a change.
- `tick -> bool` stays only where a reader outside the shells needs it:
  `ShowcaseView::tick` and `Workbench::tick` mean "in motion" for the
  settle loops (headless `render_page`, the session recorder, the
  workbench tests), and `EditorView::tick` means "flipped" for the comps
  editor example, which keeps its own timer. `TextField::tick` and
  `ToastManager::tick` are gone.
- the idle ide drew two frames every third of a second in an ordinary
  repository: `git status` took the optional index lock inside `.git`,
  the file watcher saw it and refreshed, which ran `git status` again.
  `GitRepo` now runs every git process with `GIT_OPTIONAL_LOCKS=0`, and
  the watcher ignores an event on `.git/index.lock` alone; idle, 0
  frames and 0 git processes in 6 s where there were 34 and 17.
- still open, recorded in the task file as debts: an `EditorView`
  without a focus notion keeps a blink pending while visible; the dock
  caret stays continuous with its panel open; the accessibility bridge
  still asks for a frame with nothing visible changed; a toast's springs
  take ~1.7 s each to reach the engine spring's rest threshold, most of
  it below a pixel.
