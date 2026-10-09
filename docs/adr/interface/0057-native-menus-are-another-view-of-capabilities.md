---
type: adr
status: accepted
tags: [platform-integration, menus, muda, semantic-contract, dispatcher, agents, showcase, winit]
date: 2026-09-24
---

# a native menu is another view of the app's capabilities

## context

no app of plev had a native menu; on macOS winit installed its own
default bar (About, Services, Hide, Quit) and nothing of the app was in
it. the program (task-76, item 4; the thesis, sections 6 and 22) asks
that every platform integration which is an action be a capability of
the contract: the item of a menu and the button on the screen are two
representations of one capability, and the click on the item goes
through the dispatcher as `Actor::Human`, like the click on the button.

the showcase's shell had capabilities that existed as keys alone: `T`
cycled the theme, the digits and the sidebar picked a page, `D` put the
page in a window of its own (item 2 left `shell.detach` out of the
projection for lack of an operable shell), cmd-shift-s saved the
recording. none of them was projected, so no menu, no screen reader and
no agent could ask for them.

the probe (the handoff of item 4 in
docs/log/steps/task-76-platform-integration.md) measured:

- `muda` 0.20 with its default features off resolves offline into the
  root lock as `muda` and `keyboard-types` alone; the features only pick
  the linux backend (gtk3, libxdo), the linux bring-up's choice.
- a click arrives on the platform's thread as a `MenuEvent { id }`, on a
  global channel or through the one handler a process may set; with a
  handler set the channel gets nothing. `muda` tells winit users to set
  the handler and wake the loop through the proxy.
- on macOS the bar is the application's (`NSApp.mainMenu`), one for
  every window; winit installs its default bar right before `resumed`,
  so an app that calls `init_for_nsapp` in `resumed` replaces it without
  touching the loop. on windows a bar belongs to a window
  (`init_for_hwnd`). `muda::Menu::new` panics off the main thread on
  macOS, so no test can build one.
- `ShowcaseView::semantics` is what the todo golden, the 161 regress
  baselines, the accessibility tree, the recording and cmd+click read;
  a node added there moves all of them.

## decision

- **`engine::platform::menu`, the model.** `MenuModel` is pure data the
  app builds from its `Snapshot`: menus of entries (an item, a
  separator, a submenu, a platform item such as About or Quit). a
  `MenuItem` is a capability, (`SemId`, action, optional param), with the
  label, the shortcut, `enabled` and a check mark as the projection says:
  `MenuItem::of` reads one action of one node (the action's label or the
  node's; enabled when node and action are and the node is not inert; a
  toggle's `on` or a checkbox's `checked` as the mark), `MenuItem::options`
  one item per accepted option of a select (the option as text param,
  the mark on the `selected` one), `MenuItem::plain` every action without
  a parameter that a person may ask for. the app decides which enter and
  under which menu; the platform never does.
- **the click is the button's request.** `Clicked { sem, action, param }`
  comes back from the platform and `Clicked::request(&seen)` is
  `Request::observe(seen, Actor::Human, ..)`: the same request a click on
  the button makes, checked by the same dispatcher (stale, inert,
  disabled, the policy; a destructive or confirming action asks what the
  button asks). the app dispatches it on its loop, before the frame.
- **the trait, the real one, the fake, the none.** `NativeMenu` is
  `install(model)`, `update(model)`, `poll() -> Vec<Clicked>`. `MudaMenu`
  (the desktop cfg of `arboard` and `rfd`) shows the model through
  `muda`; `update` changes labels, `enabled` and marks in place while
  the model's shape holds (`MenuModel::same_shape`: the same keys in the
  same places) and shows the bar again when it changed, so an item
  whose capability is gone leaves. `FakeMenu` keeps the model and takes
  injected clicks on enabled items, for the tests; `NoMenu` answers
  `Unsupported` on wasm and mobile; `default_menu(wake)` chooses. a
  shortcut always has a modifier: a bare letter as a key equivalent
  would take the letter away from every text field.
- **per app on macOS, per window where the platform says so.** the
  engine sets `muda`'s one handler per process to queue the item id and
  call the waker the shell gave it; the shell wakes its loop through the
  proxy (`UserEvent::Menu` in the showcase) and polls there. the shell
  that owns the windows owns the menu, beside the state it projects; on
  windows `MudaMenu::attach_window` gives it the hwnd (written, not
  compiled: windows is not a target). `WindowHost` and `Windows::route`
  did not change.
- **the showcase's shell is projected.** `shell` (region) with
  `shell.page` and `shell.theme` (selects: `selected`, editable with the
  titles and the theme names as options, `select` with a constrained
  text, any actor), `shell.detach` (toggle: `on`, `toggle` reading it;
  enabled on the desktop for a page that has something to show; a person
  only: `Actors { human, assistive, test }`, as the Files page's
  `open_file`) and, while the session is recorded, `shell.save_session`
  (button: `activate`; a person only). it runs through the gallery's one
  dispatcher, lent by the todo page as the Surface, Silences and Files
  pages do. `execute` records the ask: a page or a theme the view applies
  right after the dispatch; the window and the save stay the shell's to
  take on its loop (`take_detach_request`, `take_save_request`), as `D`
  always worked. the T key, the digits, D, the theme page's cards, the
  sidebar links and cmd-shift-s are now the person's requests on these
  nodes, with receipts.
- **where the shell shows.** `ShowcaseView::semantics` stays the page's
  alone, byte for byte: the golden, the baselines, the accessibility
  tree, the recording and cmd+click do not move. the shell enters
  `gallery_semantics`, the shell and the page in one snapshot, which the
  agent port serves and the menu is built from. an agent switches the
  theme and the page through the port and is refused the window.
- **the showcase's bar.** "plev" (about, hide, hide others, show all,
  quit: what winit's default bar had, less Services), "view" (the theme
  and the page as submenus of options with the mark on the current one,
  the page in a window of its own as a check item, the recording's save
  with cmd-shift-s while the session is recorded) and the active page's
  title with its plain actions (a button by its label, a tab by its list
  and itself, anything else by its label and the action; the rows left
  out, their actions belong to the row on screen), when it has any. the
  shell installs it once the main window is up, rebuilds the model after
  every frame of the main window and updates the bar when it moved.

## reason

- one path: a menu item, a key, a button, a screen reader and an agent
  reach a capability through the same `Request` and the same dispatcher;
  the policy is declared on the node once and holds for all of them (a
  person may open a window, an agent may not, whether it asks by menu or
  by port).
- pure model, native view: the model is testable on every target from a
  real projection (the goldens), and the platform part is the thinnest
  possible mapping, with no decision of its own.
- the platform's own words for its own items: About, Hide and Quit are
  the platform's (`PredefinedMenuItem`), never capabilities of the app.
- the shell's keys became capabilities instead of the menu becoming a
  second key handler: the item for "the page in a window of its own" and
  the `D` key are the same `shell.detach`.
- the page's projection is a contract with every agent, golden and
  baseline that reads one page; adding the shell there would have moved
  161 baselines for a node that is not in the frame. the gallery
  projection is the shell's proper scope.

## consequences

- proven without a platform: the model from the todo and the silences
  goldens; a fake click becoming the button's request and getting the
  same receipt; a moved projection disabling and removing items; the
  shell's nodes and policies; the keys, the cards and the sidebar
  leaving receipts; an agent switching the theme and the page through
  the port and refused the window; a stale pick refused; the bar over a
  bare window with a fake menu (`rust/engine/tests/menu.rs`,
  `apps/uigallery/src/views/dispatcher/shell/tests.rs`,
  `apps/uigallery/src/shell/system/menu.rs`, `dispatch/agent.rs`). `NoMenu`
  compiles the showcase for wasm; the 161 baselines and the todo golden
  are unchanged.
- `init_for_nsapp` on the live NSApp was run once with
  `PLEV_BACKGROUND=1`: the bar installed (100 items) with no panic; with
  that policy the bar is not shown and nothing was clicked, so a click
  on the real bar is the orchestrator's to see.
- the gallery projection rebuilds the page's nodes into a new snapshot
  (as the `window` node did), so their `source` is the shell's; the
  port drops `source` unless asked. a `SnapshotBuilder::extend` in `sem`
  remains the change that would keep it.
- one `MudaMenu` per process: the click queue is the process's.
- on macOS one bar for the app: the page's own window shows the same
  page, so the same bar. a second window showing another page would need
  the bar to follow the key window; nothing does today.
- windows: `attach_window` and `init_for_hwnd` are written against the
  `muda` and `raw-window-handle` APIs and not compiled here.
- the sidebar links, the digits and the theme cards now leave a receipt
  each in the gallery's dispatcher (retained up to 256).
- what the later items take from here: the tray (item 5) is a
  `MenuModel` shown by another platform object with the same `Clicked`
  and the same dispatch; the native dialogs (item 8) reuse the
  person-only policy and the trait-with-a-fake shape, with the parent
  window from item 2.

## avoid

- a menu item that calls the page, the view or the shell directly: the
  handler is the dispatcher.
- a capability that exists as a key alone: a key is a person's request
  on a projected node, or it is not a capability.
- a bare letter as a menu accelerator.
- adding the shell to the page's projection: the goldens and the
  baselines are the contract with everyone who reads one page.
- a test that constructs `MudaMenu` or any `muda` type.
