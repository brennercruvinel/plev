---
id: adr-0056
title: "a window is a host of what dies with it; the app owns the state"
project: plev
type: adr
status: accepted
date: 2026-09-24
date_basis: header
recorded: 2026-09-24
topics: [rendering, gpu, platform]
relations: [task-76, learning-cycle-0056]
source: docs/adr/structure/0056-windows-are-hosts-the-app-owns-the-state.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# a window is a host of what dies with it; the app owns the state

Decides that each window is a host (`WindowHost`) for whatever dies with it, and that the app keeps the state the windows share: the theme, the state, the single dispatcher and the agent's port. The motivation is the task-76 probe: two GPU contexts do not share atlases or layers, and a window's shell only discarded the first one when the second was created. The result is an on-demand frame per window, in which closing one does not close the other, and, with a single window, the projection stays byte-for-byte identical.

## context

every app had one window, and its shell kept the window's things beside
the app's in one struct: the engine `App` (`rust/engine/src/runtime/window`)
held `window: Option<Arc<Window>>`, a `GpuState` with the context, the
text system, the effects and the texture pool, a `Compositor`, an
`InputState`, the IME, the scale factor, the safe area, the frame clock
and the perf monitor next to the theme, the frame signal and the
lifecycle; the showcase, the ide and the urnaui shells repeated the form
and ignored the `WindowId` winit hands them with every `WindowEvent`. an
application (the thesis, section 22) opens a second window (a detached
panel, an inspector, a dialog of its own) and closes one without closing
the app.

the probe (task-76 item 2, the handoff in
[task-76-platform-integration.md](../../../Tasks/task-76-platform-integration/task-76-platform-integration.md)) measured
what a second window would share with the first: two `GpuContext`s with
two compositors drew two scenes in one loop and each frame showed only
what was pushed into it (the glyph atlas, the layer textures, the
pipelines are per context); the image store is the one global log, and a
handle loaded before the second context existed drew there from the log
on its next frame (the phase 1 design). nothing in `gpu/**` needed to
change. winit 0.30 delivers every `WindowEvent` with the id of its
window; a shell that keeps `Option<Arc<Window>>` and calls
`create_window` twice drops the first window (macOS closes it) and
routes the second's events as if they were the first's.

## decision

- **`engine::window::WindowHost`** groups what dies with a window: the
  window, its `GpuContext` (the surface, and with it the device loss and
  its recovery, per context as phase 1 left it), the `TextSystem`, the
  `EffectProcessor` and the `TexturePool` built on that device, the
  `Compositor` (its layers, and its binding table: two windows are two
  tables, by frame and by window), the `InputState` (hit regions, hover,
  focus, the cursor), the touch tracker and pointer synth, the IME, the
  scale factor and the safe area, the frame clock, the perf monitor and
  HUD, the events buffered for its next frame, and its own pending
  redraw and deadline. what the app keeps is what the windows share: the
  theme, the state and its one dispatcher, the frame signal, the
  lifecycle, the event loop proxy, the recording, the agent port.
- a host is built in steps so every target fits: `bare()` (no window,
  no device: a scene that never draws, for tests without a GPU),
  `attach_window`, `attach_gpu`; `open` does the three at once on native
  targets, blocking on the adapter as the constructors always did; on
  wasm the shell spawns the device request and attaches the result.
  `headless()` is a host without a window drawing into a texture.
  `present` is the shared frame tail (`present_frame`, or `draw_frame`
  into the headless target); the engine `App`'s own tail with the layer
  effects is `render`, on the host.
- **`engine::window::Windows`** keeps hosts by `WindowId` in opening
  order. `open` creates the window and its context, each context with
  its own `wgpu::Instance` (what the constructors make; a recovery asks
  that instance for the next adapter, and a surface belongs to the
  instance that made it). `route(id, event)` hands an event to its
  owner: input, IME, drops, resizes and scale changes are the host's;
  `CloseRequested` and `RedrawRequested` come back as the answer, since
  only the caller knows whether to close and what to draw. `close(id)`
  drops one window and its context alone; `should_exit` is true once a
  window was opened, none is left and `exit_when_last_closes` is on (the
  default; off, the loop runs without a window). an id of a window that
  closed routes as `Unknown` (the platform's `Destroyed` arrives after).
- **render on demand is per window.** `request_pending_redraws` asks the
  platform for a frame of every host that has a reason (a change in its
  scene, a request, an animation, its perf HUD) and of no other; a host
  at rest costs nothing while another animates. a shell paces the loop
  from each window's `Wake`: continuous motion draws that window's next
  frame, a deadline is that window's, and the loop waits until the
  earliest of all (`earliest_deadline`).
- **the engine `App` and the examples change no behaviour with one
  window.** the `App` drives one host through `Windows`; `GpuState` is
  gone (the host holds the device); `AppEvent::GpuReady` names the
  window. the `layers` example opens a second window with another scene,
  at rest, that draws nothing while the first animates; closing either
  keeps the other, closing both exits.
- **the showcase's page in a window of its own.** `D` outside a text
  field opens a second window for the active page (desktop only; on
  wasm and mobile the request is logged, one window there): the same
  pane state and the same dispatcher, laid out in that window's size and
  drawn into that window's compositor with layers of its own there (a
  layer id names a layer of one compositor). the main window keeps the
  chrome and the page: a change made in either window shows in the
  other on its next frame; a hover or a scroll draws the window it
  happened in alone. Escape in it, its close button or `D` again closes
  it; closing the main window quits.
- **the projection says where the page shows.** while the page has a
  window of its own, `ShowcaseView::semantics` adds one node, `window`
  (`Role::Window`, the window's title, state `page` and `shows`, the
  page roots shown in it), beside the page's nodes, which keep their
  ids, tree, state and actions; the agent port's bounds come from that
  window's compositor. with one window the projection is the page's
  alone, byte for byte: the goldens, the regress baselines and every
  agent that reads one window see no change.

## consequences

- proven without a window (`rust/engine/tests/windows.rs`, headless
  hosts under `WindowId::dummy()` and a second id): an event reaches its
  owner and no other; a frame is drawn for the window that asked and no
  other while another animates; closing one keeps the other and the
  last closes the app (or not, when told); a device lost under one
  window leaves the other's frame byte-identical and recovers on its own
  next frame; the earliest deadline across windows. and
  (`apps/uigallery/tests/detached.rs`): the same page bound in two
  compositors with two layouts, a click in the page's window toggling
  the item and the main window's next frame showing it (and back), the
  `window` node while open and the same revision as before once closed.
- what the page's own window does not get, recorded as limits: the
  accessibility bridge (the main window's tree is the only one, and its
  page nodes are placed by the main window's bindings); the recording's
  frames (that window's pointer inputs are recorded in its own
  coordinates); the right click; the overlays a page opens (a modal, a
  toast) draw in the main window, where the overlay layers live. a
  hover in one window is shared state drawn in the other on its next
  frame, not at once.
- the `window` node is rebuilt into a new snapshot through
  `SnapshotBuilder`, since `sem` has no way to carry a finished snapshot
  into another; the rebuild keeps everything but each node's `source`
  (the builder records its call site). a `SnapshotBuilder::extend` that
  keeps sources is the change `sem` would need; until then, while the
  page's window is open, cmd+click and the recording's state see the
  showcase's view as the projection site.
- the showcase does not route pointer input through
  `Windows::route`: it hands `WidgetEvent`s to its view at once, as
  before, and asks the host for resizes, scale changes and frames. the
  engine `App` and the examples route everything.
- a second `wgpu::Instance` per window: an adapter enumeration per
  context on open, the cost the constructors always paid once. reusing
  one instance would need a constructor in `gpu/**`; not this task's.
- `Windows::suspend` and `resume` keep the android round trip (surface
  dropped, device kept, a new window per host), unmeasured on a device.
- what the later items take from here: a menu bar (item 4) is per
  window on macOS, and a host is where its items would bind; sleep and
  wake (item 7) pause and resume per host (`set_animating`, the
  deadline, the recovery already per context); a dialog with a parent
  (item 8) is a window opened through `Windows::open` with the parent's
  id kept by the shell.

## alternatives

- campo `window: Option<Arc<Window>>` ao lado do estado do app: rejeitado. Um segundo `create_window` descarta a primeira janela e roteia os eventos dela para a cena errada (`## context`, `## avoid`).
- uma `wgpu::Instance` para todas as janelas: adiado. Exigiria um construtor em `gpu/**`, fora desta tarefa; cada contexto mantém a própria instância, e o custo de enumerar adaptadores é o que os construtores já pagavam (`## consequences`).
- rotear o ponteiro do showcase por `Windows::route`: não feito. A casca entrega os `WidgetEvent` direto à view, como antes; o engine `App` e os exemplos roteiam tudo (`## consequences`).
- pedir um quadro a todas as janelas a cada mudança: rejeitado. Uma janela em repouso só desenha pelos próprios motivos (`## avoid`).
- acrescentar a janela da página à projeção de uma janela: rejeitado. Goldens e baselines são o contrato com todo agente que lê uma janela (`## avoid`).
- um `SnapshotBuilder::extend` que preserve as fontes: não feito. Sem ele, o nó `window` é reconstruído e perde o `source` de cada nó; o `sem` precisaria dessa mudança (`## consequences`).

## avoid

- a shell field `window: Option<Arc<Window>>` beside the app's state: a
  second `create_window` overwrites the first and routes its events to
  the wrong scene.
- a layer id from one compositor used in another: the layers of a
  window are made in its compositor, on its first frame.
- one redraw request for every window on any change: a window at rest
  draws only for its own reasons.
- adding the window to the one-window projection: the goldens and the
  baselines are the contract with every agent that reads one window.

## learnings

- [what the autonomous cycle taught: a window is a host of what dies with it; the app owns the state](learned-windows-are-hosts-the-app-owns-the-state.md)

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0057: a native menu is another view of the app's capabilities](../../Interface/0057-native-menus-are-another-view-of-capabilities/0057-native-menus-are-another-view-of-capabilities.md)
- [what the autonomous cycle taught: a window is a host of what dies with it; the app owns the state](learned-windows-are-hosts-the-app-owns-the-state.md)
- [task-76: platform integration](../../../Tasks/task-76-platform-integration/task-76-platform-integration.md)
- [task-76, closing: what went in and what remains](../../../Tasks/task-76-platform-integration/closing.md)
- [task-76, item 2: multiple windows](../../../Tasks/task-76-platform-integration/item-2-multiple-windows.md)
<!-- /related -->
