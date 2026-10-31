---
id: adr-0068
title: "accessibility is production-ready per platform"
project: plev
type: adr
status: proposed
date: 2026-09-25
date_basis: header
recorded: 2026-09-25
topics: [accessibility, platform, semantics, web-wasm]
relations: [adr-0038, task-60, task-75]
source: docs/adr/interface/0068-accessibility-is-production-ready-per-platform.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# accessibility is production-ready per platform

States what a screen reader receives on each platform and what is already proven: the accessibility tree of each gallery page is tested, status messages are live regions that do not move focus, theme contrast is measured with the failures listed, and on macOS the system's reduced-motion preference is respected. The web stays open, because the choice between a DOM mirror with ARIA and nothing for now belongs to Brenner. It matters because being present in the code and being ready for production are different things, and whoever adopts plev needs to know, platform by platform, what reaches the screen reader.

## context

task-60 projects the AccessKit tree from the semantic snapshot
([`0038-accessibility-projects-the-semantic-snapshot.md`](0038-accessibility-projects-the-semantic-snapshot.md)) and VoiceOver was
checked on macOS. the thesis (section 21) asks more: being present and
being production-ready are different things, and a product adopting plev
needs to know, per platform, what a screen reader gets, that every page
holds together, that what changes without focus is heard, that the text
can be read, and that the system's "reduce motion" is honored.

task-75 phase 1 read the locked sources (accesskit 0.24.0,
accesskit_winit 0.32.2, accesskit_macos 0.26.1, accesskit_windows 0.32.1,
accesskit_unix 0.21.1 over accesskit_atspi_common 0.18.1, the consumers
0.35.0 and 0.36.0, winit 0.30.13) and every page of the gallery. the
matrix with file and line is in
[`task-75-platform-accessibility.md`](../../tasks/task-75-platform-accessibility/README.md), section
"fase 1: a sonda".

## decision

### the matrix is the lock's, not the documentation's

- macOS, Windows and Linux (AT-SPI) have an adapter in the lock:
  `accesskit_winit`'s `platform_impl/mod.rs` picks it (9-28). macOS is the
  only one whose target runs today; Windows and Linux are served the day
  their target exists.
- iOS has no adapter in `accesskit_winit` 0.32.2: it compiles `null.rs`,
  which drops every update.
- Android has one (`accesskit_android` 0.7.2), optional and off: the
  workspace asks `accesskit_winit` with its default features, so it is
  not in the lock. turning it on is a new crate in the lock and its own
  decision.
- the web has none, and the bridge does not compile on wasm32.
- no adapter of the lock implements custom actions. a node with several
  actions reaches a reader only when the page projects each as a control
  of its own.
- Windows and AT-SPI have no increment or decrement: a slider moves there
  by `SetValue`.

a version bump of AccessKit is a decision with approval; this matrix is
redone from the new lock when it happens.

### the tree of every page is tested

`apps/uigallery/tests/access_tree.rs` builds the tree of each of the 82
pages as the bridge does and holds it: a valid update; every control
named and focusable; Tab and the reader walking the same order; the page
taking the focus the reader asks for; every offered action reading back
as a request the dispatcher admits from assistive technology; every
status line live. what the pages do not give a reader yet is kept as
exact lists (the 76 pages without a projection, the actions no action of
the tree reaches, the focus a page takes and does not project, the press
offered while its action is disabled), so a change shows.

two rules entered the tree builder from what the sonda found:

- a button, a link or a menu item with none of `activate`, `toggle`,
  `submit`, `select` and exactly one action without a parameter is
  pressed by it (`propose`, `open_file`, `detect_silence`: seven buttons a
  reader could focus and not press).
- a group with no label, no description, no action and no focus is a
  `GenericContainer`, which every adapter leaves out and lifts its
  children from, so a reader never stops on an empty "group".

### live regions

a `Status` is a polite live region and an `Alert` an assertive one; the
focus never moves for them. the words are the node's value: its own,
else its `message` state, else its label, and an empty message says
nothing. macOS announces a live node's value, Windows and AT-SPI its
name, so the label stays the name: a status whose words are its label is
heard on all three, and one that keeps a message under a fixed name
(`silences.status`, `surface_page.status`) is heard on macOS alone until
its label follows the message (a page change, phase 2).

the gallery's toasts are not in the page's projection. the shell adds
them, as `comps` projects them, to the snapshot the bridge is shown
(`app/access.rs`), only while one is on screen; the page's snapshot, what
agents, tests and the baselines read, is untouched. their right home is
the view's own projection, where an agent reads them too. `shell:notice`
lives in `gallery_semantics` as a state of the shell region and is not a
live region yet: that is a change of the view's shell projection.

### contrast

`rust/engine/src/styling/theme/tests/contrast.rs` measures 301 pairs of the
four themes (every text token on every surface text sits on, the
tooltip's text on the tooltip, the headline gradient as large text, the
intents, the accent that draws the focus ring and the active border as
non-text, each button's label on its fill at rest, hovered and pressed)
as the window shows them: a translucent token composed over what it sits
on in linear light, as the GPU blends into the sRGB surface. 102 fail
WCAG 2.2 AA and are listed with their ratio; the test fails when one of
them starts passing and when a new one fails. no theme color changed:
that is the design's decision and moves every baseline.

### reduced motion

`engine::animation` is the one point every transition passes through
(`Spring::tick`, `Tween::tick`: the comps switch, scrollbar, toast and
overlay, the surface motion, the gallery's todo, dock and chart, the
engine's smooth scroll). it holds the preference per thread
(`set_reduced_motion`): with it on, a spring lands on its target and a
finite tween completes on the next tick, so a transition ends on the
first frame and asks for no more; a tween that repeats forever runs on.
a thread starts with it off, so a test, a headless frame and the
baselines never see a person's setting.

the bridge sets it on the event loop's thread from the system
(`accessibility::motion`): when it attaches and each time the window
takes the focus again. on macOS it asks `/usr/bin/defaults read
com.apple.universalaccess reduceMotion` (about 10 ms; a pair that does
not exist is off). winit 0.30 exposes nothing for it. Windows
(`SystemParametersInfo`), GNOME (`gsettings`), the browser
(`matchMedia`, a `web-sys` feature the engine does not declare), iOS and
Android wait for their target or for a dependency; they answer `None` and
nothing changes.

### the web: open

two options, the decision is brenner's:

1. a hidden DOM mirror with ARIA, generated from the same snapshot the
   tree is built from: another projection, never a second source. it
   gives the browser's screen readers what VoiceOver gets on macOS
   (roles, names, states, focus, live regions through `aria-live`), costs
   a DOM kept in step every frame a reader listens, and needs the
   `web-sys` features for elements and attributes.
2. nothing on the web for now: the canvas stays opaque to a reader, and
   the decision is taken when a product ships plev on the web.

## consequences

- the VoiceOver script of phase 1 (in the task file) is run by a person;
  the tests prove the tree, not what VoiceOver says.
- phase 2 opens the widgets and the pages: the component pages project
  their meaning, the multi-action rows and the selects project their
  actions as controls, the pages project the focus they take, the todo
  row holds a checkbox, the toasts and `shell:notice` move into the
  view's projection, the ambient loops (spinner, skeleton) read the
  preference, and the full keyboard.
- a status is announced on macOS when its node appears: moving to a page
  announces its status lines once.

## alternatives

- para a web: o espelho DOM oculto com ARIA (opção 1 de `the web: open`) ou nada por enquanto (opção 2). As duas seguem abertas; a escolha é do Brenner.
- uma segunda árvore só para os anúncios: rejeitada. As regiões vivas são nós da própria árvore, e elas não movem o foco (ver `avoid`).
- mudar uma cor do tema para passar no teste de contraste: rejeitado. A cor é decisão de design e moveria todas as linhas de base (ver `contrast`).

## avoid

- a claim about a platform taken from documentation instead of the
  locked source.
- a live region that moves the focus, or a second tree for the
  announcements.
- a theme color changed to pass a contrast test without the design's
  decision.
- reading the motion preference per frame, or a process-wide flag a test
  can leak into another.

## addition: phase 2a, the pages that project and the shell (2026-09-25)

task-75 phase 2a closed what phase 1 listed for the pages that already
project (Todo and Toggle Group, Surface with its quick todo, Silences,
Files, Capture) and for the shell; the sonda, with file and line, is in
the task file, section "fase 2a: a sonda". The component pages and the
full keyboard stay for phase 2b.

### where a fix lives

three things read a page's projection at rest and pin it: the `uir`
goldens (`rust/plevir/tests/fixtures/showcase-{todo,silences,files}.snapshot.json`,
what the IR fixtures verify against), the capability registry a
generated surface is verified against and a model is prompted with
(`uir::Capabilities::from_snapshot`), and the regress scene, which
carries the projection's revision and every binding. So:

- what the page means changes in the page: the focus a page takes is
  projected (the todo's filter tab, a surface's node that is not a
  field, the proposing row's select and buttons, the Silences page's
  buttons); `capture.level` says its value.
- what exists because the adapters lack custom actions, or read a role
  another way, is the reader's presentation and lives in the tree
  (`engine::accessibility`):
  - an item of a collection (a list item, a row, a tree item) holds, as
    controls of its own, the actions its own node does not reach
    (`present`): its toggle as a checkbox named by the key it reads, any
    other action without a parameter as a button named by the action.
    A control is `<item>[<action>]`, in the tree only, with no bounds (the
    item's rect would win the hit test); a click on it is the item's
    action, observed over what the reader was shown, through the app's
    one dispatcher as `Actor::Assistive`. The contract does not move: an
    agent and an IR still ask `toggle` and `remove` of `todo.item[n]`.
  - a select's press is its `open`, and its `close` while expanded; its
    `SetValue` is its `select` with the option.
  - a press is offered only while its action and its node are on; the
    node keeps saying whether it is disabled (a field that edits is not).
  - a group named only by the key of its own id, as a generated surface
    names an instance of a repeat, is named by the labels inside it.
  - a status's words are its name as well as its value: macOS announces
    the value, Windows and AT-SPI the name, so a status that keeps its
    words in a `message` under a fixed label is heard on all three. The
    projection keeps its label and its message.
- what only a reader may hear is in the view's reader snapshot
  (`ShowcaseView::reader_semantics`, what the bridge is given), never in
  the projection the port serves: the shell's notice as a polite status,
  and on the Silences page each accessibility preference button's `on`,
  which the tree says as a toggle button's state. A person's preference
  still never leaves through the port nor into a model's prompt (tested
  through the real port).

### the view's overlays

the toasts on screen, and the modal and the context menu while they hold
the pointer, are in the view's projection (`ShowcaseView::semantics`),
where an agent reads them too; `app/access.rs` no longer adds the toasts.
The comps widgets project themselves under the ids the view gives them
(`toasts`, `dialog`, `context_menu`), and bind their rects. The overlay
takes the focus (the dialog's cancel, the menu's first item, or where a
reader moved it); the page beneath is inert and keeps its own focus,
which is the projection's again when the overlay closes. Everything the
overlays do runs through the gallery's one dispatcher, a person's click
included. The menu bar leaves them out.

### reduced motion

the ambient loops follow the preference too: with it on, a spinner's arc
and a skeleton's pulse stand still and ask for no frame.

### what stays, and why

- the Files page's `drop_files`: a reader does not drag a file, and
  `files.open` reaches the same files.
- on Windows a select with `expanded` is expand-collapse, which the
  adapter does not implement, so it gets no Invoke: it is set by value.
  AT-SPI has no text `SetValue`: a select is set there through its
  options.
- a presented control has no bounds: a reader walks to it, a pointer
  does not land on it.

## consequences of phase 2a

- the exact lists of `apps/uigallery/tests/access_tree.rs` are down to
  the file drop; the others are empty and stay empty.
- the capture baselines moved in their semantics only (the scene's
  revision with them); every other baseline and the `uir` goldens held.
- the VoiceOver script of phase 2a, in the task file, is run by a person.

## addition: phase 2b, the component pages (2026-09-25)

task-75 phase 2b made the component pages whose widgets already project
in comps (task-59) project what they show; the sonda, with file and line,
is in the task file, section "fase 2b: a sonda".

### a component page is a `Component`

one trait (`apps/uigallery/src/views/dispatcher/component/mod.rs`) and one table,
a line per pane: the page's widgets carry the ids it gave them, so each
draw binds its rect beside the scene; it projects them under a region
named by the page's title (inert while an overlay holds the pointer); it
runs a request against its own state and says what the view does after
(a toast, a modal); it takes the focus a reader asks for. The view's
projection, its `act`, its `assist_focus` and the Escape reach a component
page through the table and nowhere else, so an agent through the port and
a screen reader meet the page in one projection, and what either asks
runs through the gallery's one dispatcher. 30 pages over 19 panes project
now; the menu bar leaves them out (samples of the design system, not what
the page does).

### the person's click stays in the widget

a comps widget changes its own state in its event (a checkbox checks, a
tab selects, a drag moves a knob), and every action it declares reads what
it changes (task-59). So a person's click on a component page stays where
it was, the projection reads the widget and follows it, and a request
decided before the click comes back stale. What a page does on top of a
click (the Dialog page's modal, the Toast page's toast, a crumb's echo) is
one method that the click and the request both call. The pages of the
apps keep putting the person's click through the dispatcher. Tested per
kind of widget: the person, a reader and an agent through a real port,
each on a gallery of its own, end in the same projection.

### what the tree learned

a table says its selected row in `selected_row`, and the tree marks that
row selected as it marks the child a container's `selected` names.
Windows still does not say it (a row is not a `SelectionItem` in
accesskit_windows 0.32.1); macOS and AT-SPI do.

### what stays, and why

- the pages whose widgets have no projection in comps (24: Theme, Avatar,
  Badge, Stat, Progress, Skeleton, Code Block, Card, Chart, Graph, Empty,
  Context Menu and Dropdown Menu, Sidebar, Drawer and Sheet, the dock
  pages, Editor, Git, Builder, Panel Header) and the ones with nothing to
  operate or read but a sample (22: Typography, Icons, Effects, Panel,
  Separator, the 17 placeholders), each with its reason in
  `apps/uigallery/tests/access_tree.rs`.
- the Resizable page's `set_ratio`: the tree sets a value only through
  `set_value` and a select's `select`, and the contract has no splitter
  role; an agent asks it, a person drags, a reader hears the share.
- `expanded` is said by no adapter of the lock (no disclosure in
  accesskit_macos 0.26.1, no `ExpandCollapse` in accesskit_windows, no
  expanded state in accesskit_atspi_common): a tree branch is heard
  through its presented "toggle expanded" control and the rows that
  appear.
- the `active` of a nav link or a toolbar button and a submit button's
  `busy` are not told yet.
- the full keyboard in the widgets (Tab in the projection's order, arrows,
  the modal's focus trap) is the next phase.

## consequences of phase 2b

- the regress baselines of the 30 pages carry their projection and their
  bindings; the page's layer is named by the roots bound in it (the
  regress names a layer that way), its content unchanged.
- a gallery page whose widget gains a projection in comps joins by one
  line in the table and its `Component`.
