---
id: adr-0055
title: "file dialogs and file drops are capabilities of the engine, and a native dialog opens only for a person"
project: plev
type: adr
status: accepted
date: 2026-09-24
date_basis: header
recorded: 2026-09-24
topics: [platform, input, semantics, agents]
relations: [adr-0057, adr-0058, task-76]
source: docs/adr/interface/0055-file-dialogs-and-drops-are-capabilities.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# file dialogs and file drops are capabilities of the engine, and a native dialog opens only for a person

Records the decision that file dialogs and file drops are engine capabilities: a single dialog implementation, one point that translates the window's events, and a native dialog that only a person opens. The result is that urnaui stops using `rfd` and winit's drop event directly, and no test opens a real dialog. It matters because asking to open a file, through the menu, the button or the drop, becomes one capability of the contract. It comes from item 3 of task-76.

## context

the native file dialog (`rfd`) and the file drop lived only in the
urnaui, as code of one app: `rfd::FileDialog` called in two places of
`apps/urnadbapp/src/view/explorer/mod.rs` (the Browse open, the blob export),
and `WindowEvent::DroppedFile`, `HoveredFile` and `HoveredFileCancelled`
translated by `apps/urnadbapp/src/app.rs` into two view methods. the
program (task-76, item 3; the thesis, section 22) asks for one
implementation in `engine::platform`, the urnaui using it, and every
integration that is an action to be a capability of the contract: the
"open file" of a menu, of a button and of a drop are one capability with
several representations.

the probe (the handoff of item 3 in
[task-76](../../../Tasks/task-76-platform-integration/task-76-platform-integration.md)) measured:

- `rfd` 0.15 has a backend for macos, windows, linux and wasm, none for
  android or ios; on wasm it is asynchronous only and answers bytes (a
  `web_sys::File`), never a path; `pick_folder` and a starting directory
  do not exist there.
- winit 0.30 sends one `HoveredFile` and one `DroppedFile` per file, and
  a `HoveredFileCancelled`, with no position on any backend, and no
  `CursorMoved` arrives while a file is dragged.
- the root `Cargo.lock` had no `rfd` (the urnaui is its own workspace);
  with `rfd`'s default features the lock would gain `ashpd` and the zbus
  and wayland tree as linux-only entries.

## decision

- `engine::platform::files`: the `FileDialogs` trait (`pick_file`,
  `pick_files`, `save_file`, `pick_folder`; a `DialogRequest` with title,
  filters by extension, starting directory and suggested file name),
  synchronous and modal like `rfd::FileDialog`, answering a path:
  `Ok(None)` is the person cancelling, `Err(DialogError::Unsupported)` a
  target without a dialog. `NativeDialogs` wraps `rfd` on the desktop;
  `FakeDialogs` answers what a test loaded, in order, and keeps every
  request (shared between a test and a page through `Arc<Mutex>`);
  `NoDialogs` is wasm and mobile; `default_dialogs()` chooses by target.
  the same shape as `clipboard.rs`.
- `rfd` enters the engine in the `arboard` section (not wasm, not
  android, not ios) with `default-features = false`: the features only
  choose the linux backend (the xdg portal through `ashpd`, or gtk3),
  which is the linux bring-up's decision; until then the engine does not
  build for linux with a dialog, and linux is not a target. the root lock
  gains `rfd` and `block2 0.6`, nothing else.
- `engine::input::FileDrop`: `Hovered { paths, position }`, `Dropped {
  paths, position }`, `Cancelled`, with `from_window_event` as the one
  translation of the platform's events, used by every shell (the
  engine's `App`, the urnaui, the showcase), and `join` folding the one
  event per file the platform sends into one drop. `position` is in
  logical pixels, like the pointer's, and optional: `None` from winit
  0.30, `Some` in tests and in the winit that exposes it. the engine's
  `App` buffers the event and queues it as `InputEvent::FileDrop`, hit
  tested at its position like a click, to no view without one; a shell
  that routes events itself hands the event whole to its view.
- a drop is a request. on the showcase's Files page a drop becomes
  `drop_files` by `Actor::Human` through the page's dispatcher, with the
  paths as a list param; an agent through the port may ask the same
  action: the person's drop and the agent's drop are one capability.
  the files received are projected state (`files:received`).
- a native dialog is opened only by a person. `open_file` declares
  `Actors { human: true, assistive: true, agent: false, test: true }`:
  the dispatcher refuses an agent as `NotAuthorized` before `execute`
  runs, so no agent can raise a modal panel on someone's screen; a test
  drives the action over the fake. an app that offers a dialog as a
  capability declares it so.
- on wasm the engine answers `Unsupported`. the browser's picker is
  another capability (bytes, asynchronous); the urnaui keeps its own web
  picker for that, and an `AsyncFileDialog` in the engine is the
  candidate for a later phase.

## reason

- one implementation: the urnaui holds no `rfd` and reads no winit drop
  event; its shell forwards the engine's event and its explorer asks the
  engine's trait with the same title, filter and file name as before.
- a fake by construction: every test of a page that offers the dialog
  runs over `FakeDialogs`, and the showcase's page is over the fake in
  the crate's tests, so no test can open a native dialog by mistake.
- the trait is synchronous and answers a path because both callers want
  one (open by path, export to a path), and a modal dialog blocks the
  thread that asked, as `rfd` does; an asynchronous, bytes-only dialog
  would be a second contract, and it is left named, not half built.
- the drop's position is optional because the platform gives none
  today; a fixed position (the stale cursor) would lie: the pointer does
  not move for the window while a file is dragged.
- a person-only policy is the contract's way to say who may open a
  dialog; a check inside `execute` would be a second, hidden rule.

## consequences

- `InputEvent` gained a variant; every exhaustive match on it adds an
  arm (the engine's `input` example did).
- the showcase and the urnaui shells each have one arm that hands every
  window event they do not match to `FileDrop::from_window_event`; no
  shell names a winit drop event.
- a shell that buffers events (the engine's `App`) delivers one `Dropped`
  with every path; a shell that delivers event by event (the urnaui, the
  showcase) delivers one per file, as the urnaui always did.
- when winit exposes the drag position, `from_window_event` fills
  `position` and the Files page's zone starts refusing a drop released
  outside it; nothing else changes.
- item 4 (native menus) reuses the person-only policy and the one
  capability with several representations: a menu item that opens a
  file dispatches `open_file` by the person, as the button does. item 8
  (native alerts and confirmations) reuses the trait-with-a-fake shape
  and `rfd::MessageDialog`, already in the engine's dependency.

## alternatives

- `rfd` com as features padrão: o lock da raiz ganharia `ashpd` e a árvore de zbus e wayland como entradas só de linux. Ficou com `default-features = false`, e o backend de linux fica para o bring-up (*context*, *decision*).
- Diálogo assíncrono que só devolve bytes, como o seletor do navegador: seria um segundo contrato. Fica nomeado, sem meia implementação (*reason*).
- Posição fixa para o drop, a do último cursor: mentiria, porque o ponteiro não se move enquanto o arquivo é arrastado (*reason*).
- Checar quem pode abrir o diálogo dentro de `execute`: seria uma segunda regra escondida. A política fica declarada no nó (*reason*).

## avoid

- a second `rfd` in an app, or a shell reading `DroppedFile` itself.
- an agent that may request an action which opens a native panel.
- a position invented for a drop from the last cursor position.
- a test that constructs `NativeDialogs` and asks it anything.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0057: a native menu is another view of the app's capabilities](../0057-native-menus-are-another-view-of-capabilities/0057-native-menus-are-another-view-of-capabilities.md)
- [ADR 0058: a native dialog answers the same contract: the alert is a person's capability, the confirmation is the same confirm or decline](../0058-native-dialogs-answer-the-same-contract/0058-native-dialogs-answer-the-same-contract.md)
- [task-76: platform integration](../../../Tasks/task-76-platform-integration/task-76-platform-integration.md)
- [task-76, closing: what went in and what remains](../../../Tasks/task-76-platform-integration/closing.md)
- [task-76, item 3: file dialogs and drag and drop](../../../Tasks/task-76-platform-integration/item-3-files-and-drop.md)
<!-- /related -->
