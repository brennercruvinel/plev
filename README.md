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
vector database, ships its explorer (`crates/urnaui`) on plev. and the
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

cargo run -p showcase [page] [theme] [scroll]      # the design-system gallery, one page per component (title as the sidebar shows it, "Data Table")
cargo run -p showcase -- --render Todo --theme hoff --width 1280 --height 800 --out todo.png   # any page to png, no window
cargo run -p showcase -- --render Todo --svg todo.svg --semantics todo.json   # the same frame as svg (each approximation reported on stderr) and the page's semantic projection as the json `plev observe` serves
cargo run -p showcase --example regress -- check   # structural regression over crates/showcase/fixtures/regress: meaning, then layer hashes, then pixels only where a layer moved; `-- bless` accepts a change
cargo run -p showcase --example regress -- capture   # every page in 4 themes at 4 scales through the real frame path; `--against <dir>` compares byte for byte with an earlier capture
cargo run -p ide --bin ide [path]      # plev-native git client; with PLEV_AGENT=1 (debug) its window opens the agent port as app ide, with PLEV_RECORD=1 it records its session (cmd-shift-s, on close, on a panic) for plev-inspect
cargo run -p ide --bin ide-lab -- --arm scripted   # the task-62 lab (unix): the ide's workbench with no window, on a temp repo, served to an agent through the port, recorded, checked against git
cargo run -p prime                     # prime-coherence particle swarm
cargo run -p engine --example snake    # any of the windowed demos
cargo run -p comps --example editor    # the design-system editor demo
cargo run -p wire --bin plev -- observe --scope todo --app showcase   # an app opened with PLEV_AGENT=1 (debug), as an agent sees it
cargo run -p wire --bin plev -- act todo.item[1] toggle --from-revision <rev>   # act on what that observation saw; exit 0 done, 1 rejected, 2 failed, 3 error
cargo run -p inspect --bin plev-inspect -- <session.monster> [--timeline] [--why <request>] [--control <id> --at <ms>] [--json]   # a recorded session read back: a summary, every event, a request's causal chain, a control at an instant; it never runs anything
cargo run -p inspector --bin plev-inspector -- <session.monster>   # the inspector's window: the timeline and the scrub, the recorded frame with its controls, the thesis answers about the control clicked; it never runs anything
cargo run -p claude-model --example propose -- --app todo "show the filter as chips and hide the counts"   # an intent to a verified IR over the todo's (or --app ide, the workbench's) golden projection, proposed by the Claude through the local claude -p (PLEV_CLAUDE names another program); prints each attempt, the time and the IR
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
./script/web
```

urnaui, the `.urna` explorer, is its own workspace because its native
backend path-depends on a sibling checkout of
[urna](https://github.com/hoffresearch/urna) (v0.4.0 or later) at
`../urna`. with urna cloned next to plev:

```
cargo run --manifest-path crates/urnaui/Cargo.toml [file.urna]   # drop a file to open, or Browse…
./script/web-urnaui                                              # web target on :8081
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

`crates/llm` runs a language model inside the app's own process and
streams its answer token by token, with candle or burn behind one
interface. it is its own workspace too, because burn brings a dependency
tree the engine does not need. the weights stay outside the repository:
point `PLEV_LLM_MODEL` or `--model` at a directory with a llama-family
checkpoint as Hugging Face ships it (`config.json`, `tokenizer.json`,
`model.safetensors`); the measurements used TinyLlama 1.1B Chat. with the `plev`
feature, burn runs on the renderer's own gpu device.

```
cargo test --manifest-path crates/llm/Cargo.toml --features candle,burn
cargo run --release --manifest-path crates/llm/Cargo.toml --features candle,metal --bin llm-bench -- --impl candle --model <dir> --device metal
```

mobile: the showcase runs natively on iOS and Android from the same
engine. there is no kotlin or swift ui, every pixel is drawn by plev. iOS
needs xcode, the `aarch64-apple-ios-sim` target and `xcodegen`; Android
needs the sdk/ndk, a jdk 17 and `cargo-ndk`.

```
cd ios/showcase && ./build_ios.sh && ./run_ios.sh   # simulator: build, install, launch, screenshot
cd android && ./build_android.sh                    # -> app/build/outputs/apk/debug/app-debug.apk
script/capture-web --all                            # every gallery page in the browser, each classified against the desktop's frame
script/capture-ios --all                            # the same on the simulator
```

## workspace

the engine is one crate, the design system is one crate over it, the
repo root is a virtual workspace, libraries and apps are sibling crates,
demos are the engine's (and comps') examples.

| tier | where | what |
|---|---|---|
| engine | `crates/engine` | gpu, compositor, effects, text, path, layout, input, actions, animation, signal, theme (the tokens), graph, builder, window, headless, accessibility, clipboard, perf, hot_reload, platform |
| comps | `crates/comps` | the design system: every widget, recipe, icon, chart, overlay, the editor and the app shell, all read from `engine::theme::Theme`; the inventory is [docs/catalog.md](docs/catalog.md) |
| crates | `crates/` | claude-model, git, ide, inspect, inspector, lot, macros, monster, narrate, narrate-macro, parser, prime, propose, rec, rope, sem, showcase, surface, svg, uir, wire |
| own workspaces | `crates/urnaui`, `crates/llm` | the `.urna` explorer and the in-process language model, each with its own `Cargo.lock` |
| examples | `crates/engine/examples/`, `crates/comps/examples/` | 13 windowed engine demos, the comps editor demo, plus the lot2monsters, svg2monster and text_probe clis |

`comps` is the crate an app draws with. widgets are plain structs the app
owns; they take `&Theme` and read every color, radius, height, padding,
shadow and breakpoint from it, so a theme swap or a phone width changes
the whole app at once. the showcase and the ide are framed by
`comps::shell::AppShell`: full sidebar on a desktop, a rail in a narrow
window, a drawer behind a menu button on a phone.

the text field in `comps` composes input-method text inline, moves its
caret by what the screen shows in mixed-direction text, and copies, cuts
and pastes through the person's clipboard. the text still being composed
is only drawn: it stays out of the field's value, the recording and what
an agent reads. only the person's keys and menu reach the clipboard; an
agent can neither read it nor write to it.

`sem` is the semantic contract: each control's persistent id, role,
state, actions and constraints as pure data, projected from app state on
demand ([docs/ADR/semantic-contract-as-pure-projection.md](docs/ADR/semantic-contract-as-pure-projection.md)).
`rec` records a session as data in a bounded ring (who observed what, the
state, every receipt, the frames on screen) and reads it back; replay
never creates a request. `inspect` answers the questions a session raises
(what the agent saw, why a request was refused, what a control held at an
instant) and `inspector` is its window; neither runs anything.
`wire` lets an agent outside the process observe and operate an app
through that contract only, over a local socket in json lines: scoped and
incremental observations, and actions that go through the same dispatcher
as a click. `uir` is the plev IR: a surface described by meaning over
that contract, and only what its verifier accepts reaches the runtime.
`surface` materializes a verified IR into a live surface over an app:
laid out by the engine, drawn by the comps widgets, projected from the IR,
and every event a request through the app's own dispatcher.
`propose` turns an intent into a proposed IR, by rules over the app's
registry or by a language model behind a trait, and only a verified IR
comes out of it. `claude-model` puts the Claude behind that trait: it
runs the `claude -p` of Claude Code as a child process, with no tools and
no saved session, and kills it past a timeout; the network is the
child's, the crate holds no key.

some crates are pipelines, with a foreign format in and ours out.

`monster` is a binary animation format, magic `MON0`, container version
0 (the v0 spec), its golden fixtures frozen.
keyframes are full snapshots so seek is O(1), between them only discovered
deltas travel, and a node that does not change costs zero bytes. `lot`
converts a lottie json into it once, `svg` imports a still through the
same door, and no foreign runtime is ever embedded. the decision record is
[docs/ADR/monster-format-v0.md](docs/ADR/monster-format-v0.md).

`parser` reads ui source from React, gpui, a subset of HTML and CSS, or a
Figma export, and writes plev builder code and plev IR from it. colors
become the nearest theme token. every construct in the source lands in
the report as supported, transformed, approximated or impossible, with
its file and line, so nothing is dropped without a trace.

```
cargo run -p engine --example lot2monsters in.json out.monster
cargo run -p engine --example svg2monster  in.svg  out.monster
cargo run -p engine --example monster_player out.monster
cargo run -p engine --example monster_player out.monster --frame 0.5 --out f.png   # one frame, no window

cargo run -p parser --example transpile  index.tsx module.sass vars.sass
cargo run -p parser --example preview    index.tsx module.sass vars.sass
```

the frame flow, from event to present, is the `[diagram]` table of
[docs/arc/arc.toml](docs/arc/arc.toml).

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
./script/gate       # test (the wgsl shaders included), clippy, fmt and the wasm check, plus the urnaui leg when ../urna is there; stops on the first red
cargo bench -p rope # criterion, one crate
```

benches sit on the hot path of each crate: `engine` scene build, `rope`
edits, `monster` codec, `lot` conversion, `parser` transpile.
`crates/llm` measures inference with its own `llm-bench`.

## docs

- [docs/arc/arc.toml](docs/arc/arc.toml): the architecture in one file. tables for machines, prose for humans, the frame-flow diagram at the end.
- [docs/ADR/](docs/ADR/): the decision records.
- [docs/how-to/code-against-the-plev-engine.md](docs/how-to/code-against-the-plev-engine.md): the operating manual, read before touching ui or rendering code. its siblings in [docs/how-to/](docs/how-to/): pixel validation, the web target, orchestrating agents.
- [docs/mission/generative-runtime-program.md](docs/mission/generative-runtime-program.md): where plev is going, from GPU UI engine to generative interaction runtime, with the task graph and the state of each task. the thesis and the rules every agent follows sit next to it.
- [docs/CHANGELOG.md](docs/CHANGELOG.md): what changed, newest first.
- [.contracts/.agents/AGENTS.md](.contracts/.agents/AGENTS.md): the single instruction source for ai agents and contributors. route every tool here, no per-tool files.

## license

MIT, see [docs/LICENSE](docs/LICENSE).

made it simple, but significant.

this is maintained by brenner cruvinel (brenner@hoffresearch.com).
all contributions are welcome.
(∂μfμν = jν)

Brenner Cruvinel
