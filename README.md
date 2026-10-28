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

rust edition 2024 through [rustup](https://rustup.rs), which reads
`rust-toolchain.toml` (stable, rustfmt, clippy, the wasm32 target). the
workspace needs rust 1.87 or later, the floor of wgpu 29. a Homebrew rust
ignores that file and ships no wasm32 std, so the web legs only work with
rustup.

```
cargo build --release --workspace

cargo run -p uigallery [page] [theme] [scroll]     # the design-system gallery, one page per component (title as the sidebar shows it, "Data Table")
cargo run -p uigallery -- --render Todo --theme hoff --width 1280 --height 800 --out todo.png   # any page to png, no window
cargo run -p uigallery -- --render Todo --svg todo.svg --semantics todo.json   # the same frame as svg (each approximation reported on stderr) and the page's semantic projection as the json `plev observe` serves
cargo run -p regress -- check   # structural regression over tool/bench/regress/baselines: meaning, then layer hashes, then pixels only where a layer moved; `-- bless` accepts a change
cargo run -p regress -- capture   # every page in 4 themes at 4 scales through the real frame path; `--against <dir>` compares byte for byte with an earlier capture
cargo run -p idestudio [path]          # plev-native git client; with PLEV_AGENT=1 (debug) its window opens the agent port as app ide, with PLEV_RECORD=1 it records its session (cmd-shift-s, on close, on a panic) for plev-inspect
cargo run -p agenteval -- --arm scripted   # the task-62 lab (unix): the ide's workbench with no window, on a temp repo, served to an agent through the port, recorded, checked against git
cargo run -p primes                    # prime-coherence particle swarm
cargo run -p arcade                    # any of the windowed demos (demo/*)
cargo run -p socket --bin plev -- observe --scope todo --app showcase   # an app opened with PLEV_AGENT=1 (debug), as an agent sees it
cargo run -p socket --bin plev -- act todo.item[1] toggle --from-revision <rev>   # act on what that observation saw; exit 0 done, 1 rejected, 2 failed, 3 error
cargo run -p replay --bin plev-inspect -- <session.monster> [--timeline] [--why <request>] [--control <id> --at <ms>] [--json]   # a recorded session read back: a summary, every event, a request's causal chain, a control at an instant; it never runs anything
cargo run -p inspector --bin plev-inspector -- <session.monster>   # the inspector's window: the timeline and the scrub, the recorded frame with its controls, the thesis answers about the control clicked; it never runs anything
cargo run -p intent -- --app todo "show the filter as chips and hide the counts"   # an intent to a verified IR over the todo's (or --app ide, the workbench's) golden projection, proposed by the Claude through the local claude -p (PLEV_CLAUDE names another program); prints each attempt, the time and the IR
```

every debug build is its own dev loop: save a `.wgsl`, a string or a hex
color anywhere in the sources and the open window shows it, state intact;
save code and the bin rebuilds and relaunches (same pid, same terminal).
in the gallery, cmd+click on a control or a text opens the line of source
that made it, and cmd+shift+click on a text edits that string literal in
place: Enter writes it back to the `.rs` file, only while the file still
holds the code the build came from. there is no flag, macro or watcher
tool to set up, and release builds do none of it.

web, the desktop's pixels in the browser (`cargo install trunk` first):

```
./tool/tasks/webtarget.sh
```

urnadbapp, the `.urna` explorer, is its own workspace because its native
backend path-depends on a sibling checkout of
[urna](https://github.com/hoffresearch/urna) (v0.4.0 or later) at
`../urna`. with urna cloned next to plev:

```
cargo run --manifest-path apps/urnadbapp/Cargo.toml [file.urna]   # drop a file to open, or Browse…
./tool/tasks/webtarget.sh urnadbapp                                              # web target on :8081
```

it opens any `.urna` (and the pre-rename `.nest` files) and surfaces the
whole format: exact / ann / graph / hybrid search and one entry per
multimodal space, the explain panel with the rerank honesty line, chunk
filter by text, id or `urna://` citation, validate, media blobs with a
hash-verified export, and the frame of every card or image straight out of
the inlined av1 stream. two optional tools: text search shells out to the
urna offline embedders (`URNA_PYTHON` picks the interpreter; a multimodal
space needs that model's deps, e.g. `open_clip_torch` for clip), and frame
previews need `ffmpeg` on PATH (the blob is read in place, never copied).
the reference corpus is the 38k-card
[mtg-urna-benchmark](https://huggingface.co/datasets/brennercruvinel/mtg-urna-benchmark).

`rust/llmrun` runs a language model inside the app's own process and
streams its answer token by token, with candle or burn behind one
interface. it is its own workspace too, because burn brings a dependency
tree the engine does not need. the weights stay outside the repository:
point `PLEV_LLM_MODEL` or `--model` at a directory with a llama-family
checkpoint as Hugging Face ships it (`config.json`, `tokenizer.json`,
`model.safetensors`); the measurements used TinyLlama 1.1B Chat. with the `plev`
feature, burn runs on the renderer's own gpu device.

```
cargo test --manifest-path rust/llmrun/Cargo.toml --features candle,burn
cargo run --release --manifest-path rust/llmrun/Cargo.toml --features candle,metal --bin llm-bench -- --impl candle --model <dir> --device metal
```

mobile: the gallery runs natively on iOS and Android from the same
engine. there is no kotlin or swift ui, every pixel is drawn by plev. iOS
needs xcode, the `aarch64-apple-ios-sim` target and `xcodegen`; Android
needs the sdk/ndk, a jdk 17 and `cargo-ndk`.

```
tool/tasks/mobileapp.sh ios && tool/tasks/mobileapp.sh ios-run   # simulator: build, install, launch, screenshot
tool/tasks/mobileapp.sh android                                  # -> apps/uigallery/android/app/build/outputs/apk/debug/app-debug.apk
tool/tasks/screencap.sh web --all                            # every gallery page in the browser, each classified against the desktop's frame
tool/tasks/screencap.sh ios --all                            # the same on the simulator
```

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

```
./tool/tasks/fullcheck.sh       # test (the wgsl shaders included), clippy, fmt, the wasm check and the docs links, plus the urnadbapp leg when ../urna is there; stops on the first red
cargo bench -p editor           # criterion, one crate
```

benches sit on the hot path of each crate: `engine` scene build, `editor`
edits, `motion` codec, `import` lottie conversion and gpui transpile.
`rust/llmrun` measures inference with its own `llm-bench`.

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
