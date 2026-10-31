---
id: adr-0058
title: "a native dialog answers the same contract: the alert is a person's capability, the confirmation is the same confirm or decline"
project: plev
type: adr
status: accepted
date: 2026-09-24
date_basis: header
topics: [platform, semantics, agents]
relations: [adr-0055, adr-0059, task-58, task-76, task-92]
source: docs/adr/interface/0058-native-dialogs-answer-the-same-contract.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# a native dialog answers the same contract: the alert is a person's capability, the confirmation is the same confirm or decline

Records that the native alert and confirmation answer to the same contract as the file dialogs: the alert is a person-only capability, and native confirmation is an app option that goes through the same `confirmed_by_person` or `declined_by_person`. The result is that the person answers through the drawn modal, through the system dialog or through a screen reader, with the same stages in the receipt, and no agent opens or answers a panel. It comes from item 8 of task-76, with the human confirmation of task-58.

## context

the engine had no native alert ("the disk is full") and no native
confirmation ("delete this file?"). the ide asks a person to confirm a
request another actor made (a discard by an agent, `Confirm::ForAgents`;
an agent's question, `Request::ask`, task-92) in a modal it draws
itself (`apps/idestudio/src/capability/person.rs`: the request in
`AwaitingConfirmation` opens `open_confirm_modal(id, title, body,
confirm)`, and the modal's buttons answer through `confirmed_by_person`
and `declined_by_person`, which call `Dispatcher::confirm` and
`decline` as `Actor::Human`). the program (task-76, item 8; the thesis,
sections 6 and 22) asks for the native dialogs in `engine::platform` by
the same mould as the file dialogs and the menu, and for the person's
confirmation of task-58 to be offerable in the system's dialog, as an
option of the app, without touching the contract.

the probe (the handoff of item 8 in
[task-76](../../tasks/task-76-platform-integration/README.md)) measured:

- `rfd::MessageDialog::show` is synchronous and modal: on macOS it runs
  `NSAlert::runModal()` on the main thread, a nested modal loop, so the
  winit loop stands still until the person answers; with `set_parent` it
  opens as the window's sheet. custom button labels come back as
  `MessageDialogResult::Custom(label)`; Escape is `Cancel`.
- `set_parent<W: HasWindowHandle + HasDisplayHandle>(&W)` takes a sized
  type; `Arc<winit::window::Window>` has both handles
  (`raw-window-handle` 0.6.2, the one version in the lock, implements
  them for `Arc<H>`), and winit re-exports the crate, so the engine
  gains no dependency.
- `rfd` has a message dialog on macOS, windows, gtk3 and (through
  `zenity`) the xdg portal, and `window.alert`/`confirm` on wasm; none on
  android or ios. it is in the engine only for the desktop that is not
  mobile.
- what the apps show today on their own: the ide's toast (`notify`, the
  words of `notice.rs`, projected as `notices.*` for the agent too), the
  Files page's `files.status`. widgets, never a native panel.

## decision

- **`engine::platform::alerts`**: `Alert { level: Level (info, warning,
  error), title, body }`, `Confirm { title, body, confirm_label,
  cancel_label, destructive }`; the trait `Alerts` with `alert(&Alert,
  parent: Option<&dyn Parent>) -> Shown<()>` and `confirm(&Confirm,
  parent) -> Shown<Answered { Confirmed | Declined }>`, where `Shown<T>`
  is `Result<T, AlertError::Unsupported(kind)>`. `Parent` is a marker
  over `HasWindowHandle + HasDisplayHandle`, implemented by every type
  with both handles, so the shell's `Arc<Window>` goes in as it stands.
  `NativeAlerts` wraps `rfd::MessageDialog` on the desktop (the level as
  the icon, a destructive confirmation as a warning, the two labels as
  `OkCancelCustom`, the answer read by the button's label, `set_parent`
  when a parent is given); `FakeAlerts` answers what a test loaded, in
  order, and keeps every dialog shown and whether it had a parent (shared
  through `Arc<Mutex>`); `NoAlerts` is wasm and mobile; `default_alerts()`
  chooses by target. the same shape as `files.rs`, line for line.
- **the native confirmation is the app's option over the same
  contract.** `Workbench::confirm_natively(alerts, parent)` makes the ide
  ask the platform's dialog in the modal's place: the same title, body
  and button (`confirmation(&request)` is the one source of the words),
  `destructive` as the node's action policy says, the window as parent;
  the answer goes through the same `confirmed_by_person` or
  `declined_by_person`, so the dispatcher runs the same second check
  against the live state, and the receipt, the recording and the
  inspector see the same stages (`confirmed { by: human }`, or `rejected
  { declined { by: human } }`). the shell turns it on with
  `PLEV_NATIVE_CONFIRM=1`; the drawn modal stays the default. a target
  whose dialogs are `Unsupported` falls back to the modal. an agent
  through the port never opens, answers or sees a dialog: its request
  waits for a person as before, and `confirm` by an agent is
  `ConfirmByMachine` as before. nothing in `sem` changed.
- **the native alert is a capability of a person alone.** the showcase's
  Files page gained `files.warn` (button, action `warn`, policy `Actors {
  human, assistive, test }`, the same as `open_file`): a person's click
  or Enter asks `Alerts::alert` through the gallery's dispatcher, the
  panel says how many files the page holds and that nothing was written,
  and the body shown is projected as `files:alert`, so an agent reads
  what the person saw and is refused the button as `NotAuthorized`
  before `execute` runs. the toast and the status stay the app's truth;
  the panel is another representation of it, never a second source.
- **what the recording keeps of a native answer.** the receipt's stages,
  through the sink already on the dispatcher; no `Input`, since no click
  or key reached the window (the dialog is the platform's). `plev-inspect`
  names who answered from the stage (`by human`), as it does for the
  modal. `rec` and `record.rs` did not change.

## reason

- one contract for the answer: a person confirms in the modal, in the
  platform's dialog or through a screen reader, and the dispatcher sees
  one `confirm` by `Actor::Human` or `Assistive`, with the same guards
  (`require_person`, the second check). a second path for the native
  dialog would be a second rule about who may confirm.
- the option belongs to the app, not to the request: which surface asks
  the person is presentation; the contract says only that a person
  answers.
- a fake by construction: every test of the ide's native confirmation
  and of the Files page's alert runs over `FakeAlerts`, and the pages are
  over the fake in the crates' tests, so no test can open a native panel
  by mistake, and no panel opened on the machine this was built on.
- synchronous, like the file dialogs: `rfd`'s dialog blocks the thread
  that asked, and the apps' one thread is the main one. an asynchronous
  dialog (`AsyncMessageDialog`) is another contract, left named.
- a person-only policy on the node is the contract's way to say who may
  raise a panel on someone's screen; a check inside `execute` would be a
  second, hidden rule.

## consequences

- the ide loop stands still while the native dialog is open: an agent's
  act through the port whose request waits for a person is answered when
  the dialog closes, with its final stage, instead of `awaiting_confirmation`
  right away as with the modal; git results and frames queue meanwhile.
  the modal keeps the loop alive, which is why it stays the default.
- the native dialog on the ide is asked from inside `dispatch`
  (`report` -> `ask_next_confirmation`); one dialog at a time, the next
  request waiting is asked when the answer lands, as the modal does.
- the Files page's panel opens without a parent: the page has no window
  (the shell owns the windows and `app.rs` did not change), so it is a
  free-standing panel, not a sheet.
- the Files page's projection gained `files.warn` and `files:alert`: the
  golden `rust/plevir/tests/fixtures/uigallery-files.snapshot.json` and the two
  regress baselines of the page (`files@core`, `files@hoff`) were
  blessed for it; the other 159 baselines are unchanged.
- `destructive` on the native confirmation follows the node's policy (a
  discard is, a stage with `--ask` is not); the drawn modal still shows
  every confirmation with the destructive intent.
- windows: custom button labels need `rfd`'s `common-controls-v6`
  feature; not a target, not enabled. linux: the message dialog goes
  through gtk3 or `zenity`, the bring-up's choice.
- what the later items take from here: notifications (item 5) are an
  `Alert` the platform shows without blocking and without an answer, or
  a `Clicked` of the menu when they carry an action; the tray reuses the
  person-only policy. an `AsyncAlerts` over `rfd::AsyncMessageDialog`
  would keep the loop alive during a confirmation and is the change that
  would make the native dialog a candidate for the default.

## alternatives

- Diálogo nativo assíncrono (`AsyncMessageDialog`): seria outro contrato e fica nomeado. Um `AsyncAlerts` é a mudança que tornaria o diálogo nativo candidato ao padrão (*decision*, *consequences*).
- Caminho próprio para a confirmação nativa, com regra própria de quem confirma: seria uma segunda regra. A resposta passa por `confirmed_by_person` ou `declined_by_person` (*reason*).
- Alerta como segunda fonte de verdade: o que o painel disse é estado projetado, e o toast ou o status continuam dizendo (*avoid*).
- Painel aberto por pedido de agente: o pedido segue esperando uma pessoa, como antes (*decision*, *consequences*).

## avoid

- a native dialog opened by an agent's request, or a policy that lets
  one.
- a confirmation answered by anything but `Dispatcher::confirm` or
  `decline` as a person.
- a test that constructs `NativeAlerts` and shows anything.
- the alert as a second truth: what the panel said is projected state,
  and the toast or the status keeps saying it.
