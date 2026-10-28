# plev

a gpu-first compositing engine in rust. one codebase draws the same frame
on macOS (Metal), in the browser (WebGPU), on iOS (Metal) and on Android
(Vulkan). the gallery has been captured in the browser and on the iOS
simulator and compared with the desktop byte for byte. in the browser 79
of its 82 pages match, and the other three show different content on
purpose, because the browser has no Claude and no git.

a scene is rebuilt every frame, the compositor redraws only the layers
that changed, and everything lands on an srgb surface. it has glass,
backdrop blur, analytic shadows and layout that follows the content. text
is shaped for real, rasterized at the screen's scale, and covers latin,
arabic, hebrew, devanagari, chinese, japanese, korean and color emoji,
with bidi carets, input methods, and an editor that opens a million lines
in about 20 ms in a release build. when nothing changes, nothing is drawn:
a focused, untouched field costs two frames a second for its caret, and an
idle app costs none.
the apps consume the engine and nothing bypasses it.

every control an app draws is also data: an id, a role, its state and its
actions. a screen reader, an agent outside the process and a recorded
session all read that same contract. an agent acts through the same
dispatcher as a click, and only on state it was shown; an act on
something that changed since then is refused, naming who changed it. a
screen can be generated from an intent as a verified IR, and a recorded
session opens in an inspector or runs again as a test.

## why this exists

i started this about six years ago, long before LLMs, when i decided to go
deep into rust the honest way: build something real. the something real
was a children's education app for my daughter, one hundred percent rust,
and it needed an engine that didn't exist. plev is the engine underneath
that dream.

plev does not compete with [flutter](https://flutter.dev),
[egui](https://github.com/emilk/egui), [dioxus](https://dioxuslabs.com) or
[iced](https://iced.rs). those are serious projects built by serious teams,
and i am one person learning in public. some i studied more than i ever
used: [makepad](https://makepad.dev) and zed's [gpui](https://www.gpui.rs)
are among the highest-level engineering works i have seen in recent years,
and [bevy](https://bevyengine.org) taught me how a rust project can stay
both ambitious and welcoming. plev owes them most of what is good here.
what is left, the mistakes, are mine.

so this repo is exactly what it looks like: a personal experiment, done in
the open. what it takes for one person to build a small indie engine with
real memory control, performance first, everything on the GPU, running on
any device, ready for the LLM era. it already serves beyond the original
dream: [urna](https://github.com/hoffresearch/urna), a sovereign embedded
vector database, ships its explorer (`apps/urnadbapp`) on plev. and the
children's app that started all of this is still the next thing to build.

if any of this resonates, come build with me.

## run

you need [rustup](https://rustup.rs) (it reads `rust-toolchain.toml` and
installs the right rust). for the browser you also need trunk
(`cargo install trunk`) and Chrome or Edge 113+. run every command from the
repository root; the first build takes a few minutes.

### on the desktop

the design-system gallery, every component on its own page:

```
cargo run -p uigallery
```

the demos:

```
cargo run -p graphs -- lesmis
cargo run -p graphs -- compound
cargo run -p primes
cargo run -p arcade
cargo run -p shapes
cargo run -p layers
cargo run -p camera
cargo run -p inputs
cargo run -p mobile
cargo run -p ticker
cargo run -p lottie
cargo run -p commit
```

the git client, opened on this repository:

```
cargo run -p idestudio -- .
```

### in the browser

each command prints the address to open. a port already taken is skipped
for the next free one, so they can all run at once. resize the window and
the layout follows.

```
tool/tasks/webtarget.sh uigallery
tool/tasks/webtarget.sh graphs
tool/tasks/webtarget.sh primes
```

add `?page=Typography&theme=hoff` to the gallery's address to open one page
in one theme, and `?page=compound` to the graphs' for the other graph.

### on a phone

the iOS simulator needs Xcode and xcodegen; Android needs its SDK and NDK,
JDK 17 and cargo-ndk.

```
tool/tasks/mobileapp.sh ios
tool/tasks/mobileapp.sh ios-run
tool/tasks/mobileapp.sh android
```

### the .urna explorer

it reads `.urna` files with [urna](https://github.com/hoffresearch/urna),
cloned next to this repository (`../urna`, v0.4.0 or later).

```
cargo run --manifest-path apps/urnadbapp/Cargo.toml
tool/tasks/webtarget.sh urnadbapp
```

### checks

```
./tool/tasks/fullcheck.sh
cargo run -p regress -- check
```

the tools beyond these (a page to png, the cost of a frame, the agent port,
the mirror, recorded sessions, the importers, the language model) are in
[docs/ref/run-the-tools.md](docs/ref/run-the-tools.md). a debug build reloads
itself: save a `.wgsl`, a string or a color and the open window shows it.

## workspace

the repo root is a virtual workspace; every crate lives in one of four
homes by what it is.

| home | where | what |
|---|---|---|
| libraries | `rust/` | `engine` (compose, control, drawing, runtime, styling, and the design system as `engine::widgets`; the inventory is [docs/ref/component-catalog.md](docs/ref/component-catalog.md)), `accord` (the semantic contract), `plevir` (plev IR and its verifier), `screen` (surfaces materialized from the IR), `offers` (the proposers), `claude` (the Claude as a proposer), `socket` (the agent port and the plev cli), `record` (the flight recorder), `replay` (reading a session back, plev-inspect), `motion` (the .monster codec), `import` (lottie, svg, react, html, figma, gpui in), `editor` (the rope editing core), `gitdrv` (git), `markup` + `macros` (the markup dsl and the proc-macros), `unions` (no code: one feature set for every build, kept by cargo hakari), `llmrun` (own workspace) |
| apps | `apps/` | `uigallery` (the design-system gallery, desktop, web, android, ios), `idestudio` (the git client), `agenteval` (the experiment's lab), `inspector` (the session inspector window), `mirroring` (a page mirrored over the scene socket), `urnadbapp` (the `.urna` explorer, own workspace) |
| demos | `demo/` | one crate per demo: arcade, camera, commit, graphs, inputs, intent, layers, lottie, mobile, mockup, player, primes, shapes, ticker |
| tooling | `tool/` | `bench/measure` and `bench/regress` (cost tables and the structural regression), `tasks` (the scripts), `tests` (the architecture tests) |

`data/media/lottie` holds the lottie references the importer and the gates
read, and `data/graphs/<format>` the graph inputs the graphs demo draws (d3,
cytoscape); [docs/TERMS.toml](docs/TERMS.toml) maps every former name (sem,
showcase, comps, ...) to its place in this tree.

## contracts

violations are defects even when the pixels look right.

- one TextStyle per text run, shared by measurement and drawing. the
  only width source is the shaper, never an arithmetic estimate.
- srgb decode once entering gpu work, encode once at the surface write.
- container geometry derives from available space, never from constants.
- ui is designed for content, constraints, capabilities and ranges, not
  screens and devices: fluid scale, component-driven responsiveness,
  intrinsic layout, relative and semantic values, progressive adaptation.
- a visible state change implies invalidation, with no silent redraws and
  no stale frames.
- an agent acts only through the app's dispatcher, the same path as a
  click, and only on state it was shown.

a guard test scans the repo and fails if any code draws text by
constructing a raw key instead of going through the one-TextStyle path.

## docs

- [docs/ATLAS.toml](docs/ATLAS.toml): the architecture in one file. tables for machines, prose for humans, the frame-flow diagram at the end.
- [docs/adr/](docs/adr/): the decision records.
- [docs/ref/code-against-the-plev-engine.md](docs/ref/code-against-the-plev-engine.md): the operating manual, read before touching ui or rendering code. its siblings in [docs/ref/](docs/ref/): pixel validation, the web target, orchestrating agents.
- [docs/log/generative-runtime-program.md](docs/log/generative-runtime-program.md): where plev is going, from GPU UI engine to generative interaction runtime, with the task graph and the state of each task. the thesis is [docs/pub/generative-runtime-thesis.md](docs/pub/generative-runtime-thesis.md) and the rules every agent follows [docs/ref/generative-runtime-guidelines.md](docs/ref/generative-runtime-guidelines.md).
- [docs/log/changelog.md](docs/log/changelog.md): what changed, newest first.
- [docs/TERMS.toml](docs/TERMS.toml): the names of the tree, old and new, crate by crate and module by module.
- [.devops/check/](.devops/check/): the tool contracts (lychee, nextest, typos).

## license

MIT, see [LICENSE](LICENSE).

made it simple, but significant.

this is maintained by brenner cruvinel (brenner@hoffresearch.com).
all contributions are welcome.
(∂μfμν = jν)

Brenner Cruvinel
