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

rust through [rustup](https://rustup.rs), which reads `rust-toolchain.toml`
(stable, rustfmt, clippy, the wasm32 target; rust 1.87 or later). the web
needs [trunk](https://trunkrs.dev) (`cargo install trunk`) and a browser with
WebGPU (Chrome or Edge 113+). every command runs from the repository root.

apps, desktop:

```
cargo run -p uigallery                                   # the design-system gallery, one page per component
cargo run -p uigallery -- Typography hoff                # one page, one theme (core, hoff, dark, light)
cargo run -p idestudio -- .                              # the git client, on a repository
cargo run --manifest-path apps/urnadbapp/Cargo.toml      # the .urna explorer (urna cloned at ../urna)
cargo run -p inspector --bin plev-inspector -- <session.monster>   # a recorded session, in a window
```

demos, desktop:

```
cargo run -p graphs -- lesmis                            # d3's Les Misérables, the force simulation live: drag a node
cargo run -p graphs -- compound                          # cytoscape's compound nodes: drag a group and its children follow
cargo run -p primes                                      # prime-coherence particle swarm
cargo run -p arcade                                      # snake, played by an AI
cargo run -p shapes                                      # sdf shapes: cards, borders, shadows
cargo run -p layers                                      # layers, opacity and effects, in two windows
cargo run -p camera                                      # a 3d scene behind a hud of components
cargo run -p inputs                                      # text fields on the gpu: selection, the IME
cargo run -p mobile                                      # touch, safe areas, the soft keyboard
cargo run -p ticker                                      # a counter on the component lifecycle
cargo run -p lottie                                      # a lottie played directly (data/media/lottie/snakeloop, or a path)
cargo run -p import --bin convert -- data/media/lottie/cardstack/animation.json /tmp/cardstack.monster   # a lottie to .monster
cargo run --release -p player -- /tmp/cardstack.monster  # a .monster, played
cargo run -p mockup -- rust/import/tests/fixtures/react/index.tsx rust/import/tests/fixtures/react/NoteCard.module.sass rust/import/tests/fixtures/react/note-card-variables.sass   # a react component, transpiled live
cargo run -p commit                                      # the thesis scenario, in the terminal
cargo run -p intent -- --app todo "show the filter as chips"   # an intent to a verified surface, by the local claude -p
```

web, in the browser (each one serves on its own port, so all run at once;
resize the window and the layout follows):

```
tool/tasks/webtarget.sh                                  # the gallery, http://127.0.0.1:8080 (?page=Typography&theme=hoff)
tool/tasks/webtarget.sh urnadbapp                        # the .urna explorer, http://127.0.0.1:8081
(cd demo/graphs && trunk serve)                          # the graphs, http://127.0.0.1:8082 (?page=compound)
(cd demo/primes && trunk serve)                          # the swarm, http://127.0.0.1:8083
tool/tasks/webtarget.sh build --release                  # the gallery's static build, into apps/uigallery/web/dist
python3 -m http.server 8080 -d apps/uigallery/web/dist   # that build, served as it is
```

mobile:

```
tool/tasks/mobileapp.sh ios && tool/tasks/mobileapp.sh ios-run   # the gallery on the iOS simulator (xcode, xcodegen)
tool/tasks/mobileapp.sh android                          # the gallery's apk (sdk/ndk, jdk 17, cargo-ndk)
```

tools:

```
./tool/tasks/fullcheck.sh                                # the gate: test, clippy, fmt, wasm check, docs links, urnadbapp leg
cargo run -p uigallery -- --render Todo --theme hoff --out todo.png   # any page to png, no window
cargo run -p regress -- check                            # structural regression: meaning, then layers, then pixels
cargo run --release -p measure -- --out measurements.md  # what each stage of a frame costs
cargo run -p agenteval -- --arm scripted                 # the agent lab: the ide's workbench served to an agent
PLEV_AGENT=1 cargo run -p uigallery -- Todo              # the gallery with its agent port open, then:
cargo run -p socket --bin plev -- observe --scope todo --app showcase   # what an agent sees
cargo run -p socket --bin plev -- act 'todo.item[1]' toggle --from-revision <rev>   # an act on what it saw
PLEV_AGENT=1 PLEV_MIRROR=1 cargo run -p uigallery         # the gallery sharing its scene, then:
cargo run -p mirroring -- --show                         # its window, mirrored in another process
cargo run -p replay --bin plev-inspect -- <session.monster>   # a recorded session, read back as text
cargo bench -p editor                                    # criterion benches, one crate
cargo test --manifest-path rust/llmrun/Cargo.toml --features candle,burn   # the in-process language model (own workspace)
cargo run --release --manifest-path rust/llmrun/Cargo.toml --features candle,metal --bin llm-bench -- --impl candle --model <dir> --device metal   # its benchmark, weights outside the repo
```

a debug build reloads itself: save a `.wgsl`, a string or a hex color and the
open window shows it, state intact; save code and the bin rebuilds and
relaunches. in the gallery, cmd+click opens the source line of a control.

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

## gate

`./tool/tasks/fullcheck.sh` runs every check a change passes (the first line
of tools above); CI runs the same parts as separate jobs.

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
