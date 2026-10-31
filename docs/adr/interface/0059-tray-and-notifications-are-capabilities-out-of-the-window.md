---
id: adr-0059
title: "the tray and the notifications are capabilities out of the window"
project: plev
type: adr
status: accepted
date: 2026-09-24
date_basis: header
topics: [platform, semantics, agents]
relations: [adr-0057, adr-0058, task-76]
source: docs/adr/interface/0059-tray-and-notifications-are-capabilities-out-of-the-window.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# the tray and the notifications are capabilities out of the window

Records that the tray and the system notifications are capabilities outside the window: the icon and the notification reuse the menu model and come back to the dispatcher as `Actor::Human`, and no agent notifies or shows the tray. The result is a click queue with two owners, a notifier that answers `NeedsBundle` outside the bundle instead of showing a notification that cannot be taken back, and projected texts that hold even when the platform shows nothing. It comes from item 5 of task-76.

## context

the tray icon and the system notification are the two integrations that
exist when the app is not in front: the window is hidden, behind another,
or nobody is looking. the program (task-76, item 5; the thesis, sections 6
and 22) asks that each one be another representation of capabilities the
projection already has, and that a click on either come back through the
app's one dispatcher as `Actor::Human`, as the bar's items do since item 4.

the probe (the handoff of item 5 in
[task-76](../../tasks/task-76-platform-integration/README.md)) measured:

- `tray-icon` 0.25.1 with its default features off resolves offline into
  the root lock as itself alone: every dependency (`muda`, `objc2` and the
  AppKit, Foundation, CoreFoundation and CoreGraphics bindings, `png`,
  `crossbeam-channel`, `once_cell`, `thiserror`, `windows-sys`) was
  already there; AppKit gains the status bar classes as features. on
  linux the crate refuses to build without a backend feature
  (`libappindicator` or `ksni`), the linux bring-up's choice.
- `objc2-user-notifications` 0.3.2 also resolves offline as itself alone.
  but `UNUserNotificationCenter.currentNotificationCenter` wants the main
  bundle's proxy and raises `NSInternalInconsistencyException`
  ("bundleProxyForCurrentProcess is nil") in a process outside an app
  bundle, which every `cargo run` of this repository is; and writing that
  path needs `objc2` and `objc2-foundation` as direct dependencies
  besides, for the delegate that receives the response and the strings.
- `tray-icon` refuses any thread but the main one, wants the loop running
  (winit: from `StartCause::Init` on), and creating the icon adds the
  `NSStatusItem` at once: nothing can be created hidden. its menu is a
  `muda` menu, so its item clicks arrive through the same one handler per
  process the bar's do, into the same queue; its own clicks
  (`TrayIconEvent`) through another one handler per process.

## decision

- **`engine::platform::tray`, the model.** `TrayModel` is pure data the
  app builds from its `Snapshot`: an image (`TrayImage`: RGBA, or
  `TrayImage::mark`, a ring the engine draws itself, as a template so the
  platform tints it), a tooltip, a menu (a `Submenu` of `platform::menu`,
  the same `MenuItem`s the bar shows) and, for a click on the icon itself,
  the one capability the app chose (`on_click`, a `Clicked`). with one, the
  left click asks it and the right click opens the menu; without, either
  opens the menu. the trait `Tray` is `install`, `update`, `poll`; `update`
  changes the icon, the tooltip, the labels, `enabled` and the marks in
  place while the menu's shape holds and shows the menu again on the same
  icon when it changed. `NativeTray` (`tray-icon`; macOS and windows,
  windows written and not compiled), `FakeTray` (the model kept, clicks
  injected on the menu and on the icon), `NoTray` (linux until its
  bring-up, wasm, mobile), `default_tray(wake)` choosing.
- **one click queue, two owners.** the tray's items carry `tray:` before
  their key in the `muda` id; the bar takes the ids without it and the
  tray the ids with it (`take_clicks`), each leaving the other's. the
  waker is the loop's proxy, the last one given.
- **`engine::platform::notify`, the model.** `Notification` is a `Level`
  (the one of `alerts`), a title, a body and at most one `Action`: a
  capability of the projection (`SemId`, action, param) with a label,
  `Action::of(snapshot, sem, action)` reading it as the projection
  declares it and refusing what a person could not press (an agent's
  capability, a disabled one). the trait `Notifier` is `notify` (returns
  at once with a `NotificationId`, or why not: `Unsupported`,
  `NeedsBundle`, `Platform`) and `poll`; a click on the notification or on
  its button is the action's `Clicked`, one without an action hands back
  none. `FakeNotifier` keeps what was sent, refuses on request and takes
  injected clicks; `NoNotifier` on wasm and mobile; `NativeNotifier` on the
  desktop.
- **the click is the button's request.** a `Clicked` from the tray or a
  notification becomes `Request::observe(seen, Actor::Human, ..)` over the
  projection as it is now, through the same dispatcher, so the same
  checks hold: stale, inert, disabled, the policy. a capability that left
  the projection meanwhile makes no request at all, as a menu item of a
  page that closed.
- **who notifies.** the app, when something happened while nobody was
  looking; a person, through a capability declared for a person alone.
  never an agent: it reads the text the app projected. the text is
  projected whether the platform showed it or not: it is the truth of
  what the app told.
- **the real notification waits for the bundle.** on macOS
  `NativeNotifier` answers `NeedsBundle` outside an app bundle (told from
  the executable's place, `<Name>.app/Contents/MacOS/`) and `Unsupported`
  inside one until the UserNotifications path is written against a bundle
  that exists; the app logs it and goes on. the crate does not enter the
  manifest unused. `osascript display notification` as a child was
  weighed and left out: it shows as another app and cannot hand a click
  back, so it would not answer the contract.
- **the showcase.** the shell projects `shell.show` (a button, `activate`,
  a person only, enabled on the desktop): the main window forward. the
  tray shows "show plev" first (the icon's own click asks it too), the
  bar's "view" entries (theme, go to, the page in a window of its own,
  the save while recording) and the platform's quit; it follows the
  gallery after every frame of the main window. the shell keeps which
  window of ours has the focus (`WindowEvent::Focused`) and tells the
  view. the Files page, when files land while none has it, asks for a
  notification "N files received" with its open button as the button and
  projects the text as `files:notice`; the shell hears the Silences
  detection end (its request completed or failed off the loop, through a
  `ReceiptSink` on the gallery's dispatcher) and, while no window has the
  focus, tells "N silences found" (or the error) with "show" as the
  button and projects it as `shell:notice`. the window keeps closing the
  app: living in the tray alone (`Windows::set_exit_when_last_closes`) is
  a later step.

## reason

- one path: the tray item, the icon, the notification's button, the bar,
  a key, a button, a screen reader and an agent reach a capability through
  the same `Request` and the same dispatcher; the policy is declared on
  the node once.
- the tray reuses the menu's model and its `muda` code instead of a second
  menu: the same items, the same clicks, one queue with two owners.
- an honest `NeedsBundle` is worth more than a notification shown by
  another app's name with no way back.
- the text projected beside the notification is what an agent reads and
  what a test asserts; the platform's banner is a representation of it.

## consequences

- proven without a platform: the models from the todo golden and a small
  shell, a fake click becoming the button's request with the same
  receipt, stale and gone refused, an agent's capability never a button,
  the shell's `shell.show` and its policy, the tray over a bare window
  with a fake, the Files page asking and projecting, the detection's end
  told with "show" and projected (`rust/engine/tests/{tray,notify}.rs`,
  `apps/uigallery/src/shell/{tray,notify}.rs`, `dispatch/shell/tests.rs`,
  `apps/files/tests.rs`). `NoTray` and `NoNotifier` compile the showcase
  for wasm.
- no icon and no notification were shown on the machine that wrote this:
  the real tray is compiled and not run, the real notification says why
  it cannot; the orchestrator sees the icon.
- the Files projection gained `files:notice`, so its golden and its two
  regress baselines moved by meaning alone (the frames are the same); the
  shell's projection gained `shell.show` and `shell:notice`.
- the tray's menu carries the bar's key equivalent for the save while
  recording; it works while that menu is open, the bar's the rest of the
  time.
- what the platform bring-up owns: the app bundle and, with it, the
  UserNotifications path (`objc2-user-notifications` with the classes it
  uses, `objc2` and `objc2-foundation` directly, a delegate for the
  response, a category for the button); a linux tray backend.

## alternatives

- `osascript display notification` como processo filho: aparece com o nome de outro app e não devolve o clique, então não atende ao contrato (*decision*).
- Notificação real no macOS antes do bundle: `NeedsBundle` é a resposta honesta; uma notificação com o nome de outro app não tem volta (*decision*, *reason*).
- Menu próprio para a bandeja: o mesmo `MenuModel` e o mesmo código `muda` evitam uma segunda lista de itens (*reason*).
- Segunda fila de cliques para a bandeja: os dois donos dividem o manipulador (*avoid*).

## avoid

- a tray item or a notification button that calls the page, the view or
  the shell directly: the handler is the dispatcher.
- an agent that notifies or shows a tray: it reads the projected text.
- a notification whose text is not projected.
- a test that constructs `NativeTray`, a `tray-icon` type or a `muda`
  type.
- a second click queue for the tray: the two owners share the one handler.
