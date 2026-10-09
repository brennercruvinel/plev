---
type: adr
status: accepted
tags: [platform-integration, deep-links, url-scheme, semantic-contract, dispatcher, agents, policy, security, showcase, macos, bring-up]
date: 2026-09-24
---

# a link is not a person

## context

the program (task-76, item 6; the thesis, sections 6 and 22) asks that
the app's URL scheme become a `Request` through the app's one dispatcher,
with the same refusal when a capability is gone and the same policy. the
bar, the tray and the notifications (items 4 and 5) already bring a
click from outside the window as `Clicked`, the button's request by
`Actor::Human`. a link comes the same way, from outside, but not from a
person: any web page, mail or other app can open one, with no one at the
screen having chosen it.

the probe (the handoff of item 6 in
docs/log/steps/task-76-platform-integration.md) found:

- windows (a registry key with `URL Protocol` and `"<exe>" "%1"`) and
  linux (a `.desktop` with `x-scheme-handler/<scheme>` and `%u`) start a
  new process with the link as its argument, the app open or not.
- macOS delivers a registered scheme only to a bundled app
  (`CFBundleURLTypes` in its `Info.plist`), as a `GURL` Apple event
  (`kInternetEventClass`, `kAEGetURL`), to the instance that runs; never
  as an argument. AppKit's own handler for that event calls the
  application delegate's `application:openURLs:`.
- winit 0.30.13, the one in the lock, registers its own application
  delegate on macOS and depends on it, whatever its `platform/macos.rs`
  doc says ("Winit guarantees that it will not register an application
  delegate", a text the 0.30 changelog does not back; it only dropped the
  iOS delegate): `EventLoop::new` sets `WinitApplicationDelegate`
  (`platform_impl/macos/event_loop.rs:240`, `app_state.rs:47-72`), and
  `ApplicationDelegate::get` (`app_state.rs:173-184`) reads `NSApp`'s
  delegate back and panics ("tried to get a delegate that was not the one
  Winit has registered") when it is not that one. it is called on every
  `kCFRunLoopAfterWaiting` and `kCFRunLoopBeforeWaiting` (`observer.rs:62`
  and `:84`) and on every `sendEvent:` (the swizzle, `app.rs:45`). a
  delegate of ours would take its place, and the loop would panic on its
  next turn. read in the source, not run.
- the engine names no objc crate: `objc2 0.6.4`, `objc2-foundation 0.3.2`
  and `objc2-app-kit 0.3.2` are in the lock through arboard, muda, rfd
  and tray-icon, not as the engine's. naming them changes three lines of
  `Cargo.lock` (the engine's dependency list; no new package, resolved
  offline, measured on a copy of the workspace).

## decision

- **the grammar, strict.** `engine::platform::links::Link::parse(url,
  scheme)` reads `<scheme>://<sem id>/<action>[?value=<text>]`: at most
  `MAX_LEN` (2048) bytes, all printable ASCII (a space, a control, a
  letter beyond ASCII travel percent-encoded); the scheme the app's, ASCII
  case aside; the id percent-decoded, then `SemId::parse`; exactly one
  `/`; the action a name of `[a-z_]`, raw; no query, or exactly
  `value=<text>` (a `+` is a plus); decoded text valid UTF-8, with no
  control character and no direction mark, override or isolate. no
  fragment, no user or password (`@`), no port (`:`), no slash more.
  `LinkError` names the first refusal, in a fixed order; `Refused`
  carries an excerpt of what came, one printable line (96 characters at
  most, then an ellipsis; anything else escaped). `Link::url(scheme)`
  writes a link back in the form the parse reads.
- **the actor.** a link's request is `Link::request(seen)`:
  `Request::observe` over the projection as it is now, by `Actor::Agent`
  named `link` (`links::ACTOR`, `links::actor()`), never `Actor::Human`
  and never a new actor: the agent column is the least the contract lets
  anyone outside ask, and `sem` does not change. so the dispatcher checks
  a link as it checks an agent: a capability for a person alone
  (`PERSON_ONLY`, the Files dialog) is `NotAuthorized`, stale, inert,
  disabled and the parameter as for anyone, and a policy that asks a
  person's confirmation of an agent (`Confirm::ForAgents`, `Always`)
  waits for a person, which the link cannot give (`ConfirmByMachine`).
- **the open list.** before any request, the app's own list of what a
  link may ask (`Link::is_in(open)`, `(id, action)` pairs): explicit,
  small, written in the app. outside it, nothing is dispatched, whatever
  the policy would let an agent do. the showcase opens `shell.page`
  `select` and `shell.theme` `select` alone: going to a page and changing
  the theme write nothing, delete nothing, open no device, dialog, file,
  window or printer, and run no process. the todo, the silences, the
  drop, the surface, which an agent may ask through the port, stay closed
  to links.
- **the refusal is seen.** a text that is not a link, a link outside the
  list, a capability gone, a refusal of the dispatcher, a failure: nothing
  runs, and the person is told in one line, `link refused: <link>: <why>`,
  as a toast on screen and as `shell:notice` (an agent reads it), and the
  log says it. a link that waits for a person's confirmation says so the
  same way. a link that runs says nothing more than what it changed.
- **the sources.** `LinkSource::poll` hands back every link delivered
  since the last poll, read, without blocking. `LaunchLinks` takes the
  arguments that start as links of the scheme (`is_link`,
  `<scheme>://`, a malformed one included, so it is refused where the
  person sees it), once; `FakeLinks` for the tests; `NoLinks` on wasm and
  mobile; `default_links(scheme, wake)` choosing: the launch argument on
  every desktop, read when called. `wake` is for a source that delivers
  while the app runs.
- **the path per platform, today.** windows and linux: the launch
  argument, the way a registered scheme arrives there. macOS: the launch
  argument too (`cargo run -p uigallery -- <link>`), until the bring-up
  gives the app a bundle; no delegate and no Apple event handler enter
  now, and the manifest and the lock stay as they are: the `GURL` event
  only reaches an app with its scheme registered, and registering needs
  the bundle, so the code would have no real path until then, like the
  notification of item 5. wasm and mobile: none.
- **the shell.** the showcase installs `default_links` in `resumed`, once
  the main window is up (a refusal can be shown), and polls it then (the
  launch link), on `UserEvent::Link` (the `wake`) and after a wake. an
  argument that starts with `plev-showcase://` is a link, never the page
  title the first argument otherwise is. asleep, a link is not polled: it
  waits in its source and is applied on the first poll awake, the
  contract of `engine::window::Background` for work beside the frames.
- **the recording.** a session recorded (`PLEV_RECORD=1`) keeps the link
  as it keeps an agent: the observation by `agent:link` of the gallery at
  the revision it saw, and every stage of its receipt through the sink on
  the dispatcher. no input is recorded: nothing was clicked or typed.

## reason

- whoever can put a link in front of the app is not the person at the
  screen. a link as `Actor::Human` would let any web page press any
  button a person may press, the camera and the printer included; as an
  agent, it gets what an agent gets, and the app's list narrows that to
  what is harmless to be asked by anyone.
- the list is the app's because the policy is written per node for the
  actors the contract knows, and a link is a narrower thing than an agent
  the person runs: the list is where an app says so without a new actor
  in `sem`.
- a refusal the person sees is the difference between "the link did
  nothing" and "the link was stopped, and here is why".
- one path: the link reaches a capability through the same `Request` and
  the same dispatcher as a key, a click, a menu item, a screen reader and
  an agent; nothing in the view or the pages knows links exist.

## consequences

- proven without registering anything and without a window: the grammar
  case by case (`rust/engine/src/runtime/platform/links/tests.rs`), the sources
  (`links/source.rs`), the request and the policy on a test app
  (`rust/engine/tests/links.rs`), and the showcase with a fake source
  and a bare window (`apps/uigallery/src/shell/system/links/tests.rs`) and with a
  headless frame (`apps/uigallery/tests/links.rs`): a page link changes
  the page by `agent:link`, the camera, the print, the open-file dialog
  and an id that does not exist are refused and nothing runs, the line
  shows on screen, a link that arrives asleep waits for the wake, the
  launch argument is a link, the recording has the observation and the
  stages and no input.
- the showcase's projection is unchanged: the refusal goes into the
  `shell:notice` that item 5 projected, and the pages know nothing of
  links.
- nothing on macOS receives a link from the system yet; `cargo run -p
  showcase -- <link>` is the way in there until the bring-up.
- `showcase::app::links` is public (`SCHEME`, `OPEN`, `follow`,
  `Followed`), so an integration test drives a link on a headless frame
  as the shell does.

## bring-up

- macOS: the app bundle with `CFBundleURLTypes` naming `plev-showcase`
  (the scheme is registered by Launch Services when the bundle is), and
  the `GURL` handler in the engine, never a delegate (winit's must stay):
  `objc2`, `objc2-foundation` (features `NSAppleEventManager`,
  `NSAppleEventDescriptor`, `NSNotification`, `NSString`) and
  `objc2-app-kit` (`NSApplication`) as the engine's direct dependencies on
  macOS, three lines of the lock; a small class (`define_class!`) that
  observes `NSApplicationWillFinishLaunchingNotification` through the
  `NSNotificationCenter` and, then, installs itself with
  `NSAppleEventManager::sharedAppleEventManager()
  .setEventHandler_andSelector_forEventClass_andEventID` for `GURL`/`GURL`
  (after AppKit's own handler is installed, before the launch's events are
  dispatched, so the link that launched the app is caught), reads the URL
  from `paramDescriptorForKeyword(keyDirectObject).stringValue()`, pushes
  it into the source's queue and calls the `wake` the app gave
  (`default_links(scheme, wake)`; the showcase sends itself
  `UserEvent::Link`). a test calls the handler's method through objc with
  a descriptor it builds, no scheme and no window. installing an Apple
  event handler is the process's own; it registers nothing with the
  system.
- windows: the registry key under `HKEY_CURRENT_USER\Software\Classes`
  by the installer; linux: the `.desktop` with `MimeType` and `Exec ...
  %u`, and `xdg-mime default`.
- the second instance (windows, linux): today it opens a second window
  with the link applied there. passing the link to the instance that runs
  (a named mutex and a pipe, D-Bus or a socket, and the new process
  leaves) is the bring-up's.
- iOS and android: their shells deliver a link through their own entry
  (`application:openURL:options:`, the intent's data) into a source of
  their own; `NoLinks` until then.
- universal links (`https`) are out: a link of the app's scheme alone.

## avoid

- a link's request by `Actor::Human`, or by an actor made for links in
  `sem`: the agent column is the contract.
- a capability opened to links that writes, deletes, opens a device, a
  dialog, a file, a window or a printer, or runs a process.
- an application delegate on macOS while winit 0.30 owns `NSApp`'s.
- a refusal told only to the log.
- a link applied while the app sleeps.
- registering the scheme, or opening a link of it, on the machine that
  builds: every test injects its links.

## integração (2026-09-24, orquestrador)

aceito sem mudança de código, com a opção 4 decidida na parada da sonda:
no macOS o link chega pelo argumento de lançamento até o bring-up do
bundle. a revisão conferiu: todo pedido sai como `Actor::Agent` de nome
`link`; fora da lista aberta nenhum request é feito; o caractere
recusado aparece escapado na mensagem (`escape_default`), e o texto
decodificado com controle ou marca de direção é recusado sem eco, então
o aviso na tela não carrega texto de fora sem checagem.

