---
id: adr-0060
title: "a covered window does not draw, and wakes with a new frame"
project: plev
type: adr
status: accepted
date: 2026-09-24
date_basis: header
topics: [rendering, platform, performance]
relations: [adr-0042, adr-0054, task-18, task-19, task-76]
source: docs/adr/rendering/0060-a-covered-window-does-not-draw-and-wakes-new.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# a covered window does not draw, and wakes with a new frame

Decides that a covered or minimized window neither draws nor requests frames, and that, when it shows again, it presents the current state in a single frame. It separates three states (awake, covered and asleep) and tells the work that sits alongside frames, such as the `claude -p` of the Surface page and the ffmpeg jobs of the Silences page, that the app is asleep. The animation clock does not pause: the elapsed time shows up all at once on waking, with the effect it would have had. Without this rule, a covered window kept drawing on a surface nobody saw.

## context

render on demand says: no visible change, no frame. it said nothing
about a window nobody can see. the engine handled `suspended` and
`resumed` from winit (android: the surfaces dropped and recreated) and
nothing else: `WindowEvent::Occluded` was read nowhere, the system's
sleep had no hook, and the work that runs beside the frames (the `claude
-p` of the Surface page, the ffmpeg jobs of the Silences page, the git
worker of the ide) did not know the app slept.

the probe (task-76 item 7, the handoff in
[task-76-platform-integration.md](../../tasks/task-76-platform-integration/README.md)) read winit
0.30.13 and measured the engine headless:

- on macOS the window delegate's `windowDidChangeOcclusionState:` sends
  `Occluded(!visible)`, `visible` being `NSWindowOcclusionState::Visible`
  ("at least partially on screen"): minimized, hidden, or fully behind
  other windows is covered; partially visible is not. `request_redraw`
  queues a `RedrawRequested` through winit's own app delegate, whatever
  the occlusion, so a window that animates while covered draws every
  frame into a surface nobody sees. `resumed` is sent once at start,
  `suspended` never (the desktop has no application lifecycle in winit);
  the system's sleep and wake reach an app only through
  `NSWorkspaceWillSleepNotification` and `DidWake`, which no crate in
  the lock enables (the `NSWorkspace` feature of `objc2-app-kit` is off
  in both versions there, and the engine has no direct dependency on
  it). on iOS `Occluded(true)` is the background and `Occluded(false)`
  the foreground; on the web it is the canvas' intersection with the
  viewport; on android, wayland and windows it is unsupported.
- headless: `Occluded` was nobody's event, a covered window that
  animated kept asking for frames and kept its deadline, and its
  texture kept the frame from before the cover while the state moved
  on; `Windows::suspend` emptied the map, so on android frames stopped
  by absence.

## decision

three states, one rule for frames, one flag for the work beside them.

- **awake, covered, asleep.** a `WindowHost` is covered when the
  platform says so (`Occluded(true)`, taken by `handle_event`, so
  `Windows::route` delivers it; a shell that does not route calls
  `set_occluded`). a `Windows` is asleep when its shell says so
  (`sleep`, `wake`) or the platform took its windows (`suspend` sleeps,
  `resume` wakes). a host is `awake` when neither; `Windows::awake(id)`
  says it for one window. the desktop never sleeps by the platform's
  word today: without a system sleep hook, a desktop app sleeps only by
  its shell's hand, and a covered window is what the platform gives.
- **the rule for frames.** a host that is not awake asks the platform
  for nothing: `request_redraw` is withheld and the pending ask
  withdrawn, `needs_redraw` is false, `deadline` is `None`, so
  `request_pending_redraws` and `earliest_deadline` pass it by and a
  continuous animation on it becomes "waiting to show". a
  `RedrawRequested` the platform still sends for it is not drawn (the
  engine `App` and the showcase shell check `awake`). as the window
  shows again, or the app wakes, every such host is invalidated and
  asks for one frame: the first frame after a cover or a sleep is
  built from the state as it is then, never the one drawn before. a
  host that joins a sleeping app sleeps with it. render on demand does
  not change; this is one more reason not to draw.
- **the clocks jump, they do not pause.** the state moves on while a
  window is covered or an app sleeps: the shell's catch-up ticks with
  the real elapsed time (a worker's result, an agent's request wake the
  loop), so a spring that would have ended has ended and a toast that
  would have left has left when the window shows. asleep, the showcase
  ticks its clock not at all, so the whole time asleep lands on the
  first tick after the wake, with the same effect. pausing would need
  the animation clocks to know about the cover (`animation/**`, comps),
  and it would show a person coming back a flourish that belongs to the
  moment they left.
- **the contract of the work beside the frames.**
  `engine::window::Background` (hoje `engine::runtime::window::Background`, em `rust/engine/src/runtime/window/sleep.rs:50`) is the app's sleep as that work sees it:
  `Background::sleeping(bool)` is said by `Windows::sleep` and `wake`
  (and by `suspend` and `resume`), `Background::asleep()` is read by
  whoever starts a job or applies a result. one app runs per process
  and its pages run on the loop's thread, so the flag is thread-local:
  a test's app is its thread's. the contract: a result that arrives
  asleep stays where the worker put it (its channel) and is applied on
  the first wake after the app wakes; the shell neither settles the
  view nor draws meanwhile, and the pages' `settle` read nothing while
  asleep. a job asked asleep does not start, or starts as the app wakes,
  as each app decides: the Claude of the Surface page is held
  (`asking::Pending` keeps everything the thread takes and starts it on
  the first look after the wake; a cancel meanwhile ends it with no
  child); the git worker of the ide goes on (the repository's state is
  the truth, asleep or not); the ffmpeg jobs of the Silences still
  start when asked asleep, since their submits are in the page's
  `execute`, outside this item's files (recorded below). nothing
  changes in a worker.
- **the showcase.** the shell routes `Occluded` to the window's host,
  sleeps on `suspended` and wakes on a `resumed` with the window up
  (the desktop sends neither; the mobile bring-up will), draws no frame
  for a window that is not awake, and on the wake wakes the windows
  and catches up, so the answers that arrived asleep are applied, the
  held ask starts, what ended is told, and the frame that follows shows
  it all.

## consequences

- proven without a window and without sleeping the machine, with the
  platform's events injected over headless hosts:
  `rust/engine/tests/sleep_wake.rs` (a covered window asks for no
  frame and has no deadline while another animates; an uncovered window
  draws one frame equal, byte for byte, to a host that was never
  covered, and unequal to its old frame; a sleeping app draws nothing,
  a change asleep asks for nothing, a host inserted asleep sleeps, the
  wake asks for one frame per window and each equals a control's; awake
  is neither covered nor asleep, through every order of the two; the
  background flag is the thread's), `apps/uigallery/tests/sleep_wake.rs`
  (a `ShowcaseView` on the Todo page: an item toggled through the
  dispatcher while asleep, or while covered, shows on the first frame
  after, byte for byte the frame of a view that never slept),
  `apps/uigallery/src/shell/system/sleep.rs` (the shell over a bare window: a
  detection that ends asleep is applied, drawn and told only on the
  wake; a covered main window asks for no frame while a toast lives its
  time, and for one as it shows), `surface_page/page.rs`, hoje em `views/apps/surface/page.rs` (a propose
  asked asleep is held and the fake claude runs only after the wake; an
  answer that arrives asleep waits on the channel) and `silences/jobs.rs`, hoje em `views/apps/silences/fixed_page/jobs.rs`
  (a detection's answer arriving asleep is applied on the first awake
  wake).
- a window that animates while minimized or fully covered costs no GPU
  now; a person coming back sees the state as it is, on the first
  frame. the device may not survive a sleep either; that is the
  recovery before the acquire of that first frame, per context (the
  [device loss ADR](0054-gpu-device-loss-recovers-in-place.md)), and needs nothing from here.
- the system's sleep on the desktop is not heard: it needs the
  `NSWorkspace` feature of `objc2-app-kit` (a change to the engine's
  manifest) and an observer on the shared workspace's notification
  center calling `Windows::sleep` and `wake`; until then, a display that
  sleeps covers the windows, and the app follows through `Occluded`
  where the platform sends it. mobile's `suspended` reaches
  `Windows::sleep` through the showcase shell without dropping the
  surfaces; that is the mobile bring-up's (tasks 18 and 19).
- the `Background` flag is a thread-local the pages read, not a value
  the shell hands down: the view's `wake` signature and the pages'
  structs were outside this item's files. a `Background` trait the job
  owners implement, called by the shell, is the shape to move to when
  those files are open; the flag would then become theirs.
- the ffmpeg deferral is missing on purpose, not by oversight: it takes
  one field on `SilencesPage` (the jobs held) and one check in its
  `execute`, both outside this item's files. with no desktop sleep
  today and no ffmpeg off the desktop, the gap has no observable effect.

## alternatives

As alternativas recusadas estão no próprio texto. Pausar os relógios enquanto a janela está coberta ou o app dorme faria a mudança feita durante o sono cair no tick do wake, e a animação mostraria a uma pessoa que volta um movimento que pertence ao momento em que ela saiu (decision e avoid). Desenhar a janela coberta porque a plataforma mandou `RedrawRequested` é recusado, já que o winit enfileira um para todo `request_redraw`, coberta ou não (avoid). Uma flag global do processo também é recusada, porque os testes compartilham o processo (avoid).

## avoid

- drawing a covered window because the platform sent `RedrawRequested`:
  winit queues one for every `request_redraw`, occlusion or not; the
  host is where the ask is withheld.
- clearing the compositor's dirt on a cover, or keeping the animating
  flag off: the animation waits, it is not cancelled, and the uncover's
  invalidation is what makes the first frame new.
- pausing the clocks: a change made asleep would take the whole gap on
  the wake's tick, which is the [catch-up ADR](0042-a-tick-asks-for-a-deadline.md)'s own rule.
- settling a page's worker channel from anywhere while `Background`
  says asleep: the answer belongs to the wake's frame.
- a process-global flag: tests share the process, and each test's app
  is its thread's.
