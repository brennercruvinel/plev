# plev

a gpu-first compositing engine in rust. one codebase, one pixel-identical
frame on every target: macOS on Metal, the browser on WebGPU, iOS on Metal,
Android on Vulkan.

a scene is rebuilt every frame, the compositor resolves only the layers
that changed, and everything lands on an srgb surface. glass, backdrop
blur, analytic shadows, content-driven layout, real text shaping, HiDPI
rasterization at native scale. render on demand: no change, no frame, and
an unchanged layer costs zero GPU work. the apps consume the engine and
nothing bypasses it.

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
`rust-toolchain.toml` (stable, rustfmt, clippy, the wasm32 target). a
Homebrew rust ignores that file and ships no wasm32 std, so the web legs
only work with rustup.

```
cargo build --release --workspace

cargo run -p showcase                  # the design-system gallery, 14 tabs
cargo run -p ide [path]                # plev-native git client
cargo run -p prime                     # prime-coherence particle swarm
cargo run -p engine --example snake    # any of the windowed demos
```

web, same pixels as desktop (`cargo install trunk` first):

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

mobile: the showcase runs natively on iOS and Android from the same
engine. there is no kotlin or swift ui, every pixel is drawn by plev. iOS
needs xcode, the `aarch64-apple-ios-sim` target and `xcodegen`; Android
needs the sdk/ndk, a jdk 17 and `cargo-ndk`.

```
cd ios/showcase && ./build_ios.sh && ./run_ios.sh   # simulator: build, install, launch, screenshot
cd android && ./build_android.sh                    # -> app/build/outputs/apk/debug/app-debug.apk
```

## workspace

the engine is one crate, the repo root is a virtual workspace, libraries
and apps are sibling crates, demos are the engine's examples.

| tier | where | what |
|---|---|---|
| engine | `crates/engine` | gpu, compositor, text, path, layout, input, animation, signal, theme, ui, charts, graph, builder, window, platform |
| crates | `crates/` | git, ide, lot, macros, monster, narrate, narrate-macro, parser, prime, rope, showcase, svg, urnaui |
| examples | `crates/engine/examples/` | 16 windowed demos plus the lot2monsters and svg2monster clis |

two of the crates are pipelines: a foreign format in, our format out.

`monster` is a binary animation format, magic `MON0`, frozen at v1.
keyframes are full snapshots so seek is O(1), between them only discovered
deltas travel, and a node that does not change costs zero bytes. `lot`
converts a lottie json into it once, `svg` imports a still through the
same door, and no foreign runtime is ever embedded. the decision record is
[docs/adr/monster-format-v0.md](docs/adr/monster-format-v0.md).

`parser` turns ui source from another framework into plev builder code.
it maps colors to theme tokens, obeys the engine contracts, and reports
every construct it cannot represent, with file and line. it never drops
a construct silently.

```
cargo run -p engine --example lot2monsters in.json out.monster
cargo run -p engine --example svg2monster  in.svg  out.monster
cargo run -p engine --example monster_player out.monster

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
- a visible state change implies invalidation. no silent redraws, no
  silent stale frames.

a guard test scans the repo and fails if any code draws text by
constructing a raw key instead of going through the one-TextStyle path.

## gate

```
./script/gate       # test, clippy, fmt, wasm check; stops on the first red
cargo bench -p rope # criterion, one crate
```

benches sit on the hot path of each crate: `engine` scene build, `rope`
edits, `monster` codec, `lot` conversion, `parser` transpile.

## docs

- [docs/arc/arc.toml](docs/arc/arc.toml): the architecture in one file. tables for machines, prose for humans, the frame-flow diagram at the end.
- [docs/adr/](docs/adr/): the decision records.
- [docs/how-to/code-against-the-plev-engine.md](docs/how-to/code-against-the-plev-engine.md): the operating manual, read before touching ui or rendering code.
- [.contracts/.agents/AGENTS.md](.contracts/.agents/AGENTS.md): the single instruction source for ai agents and contributors. route every tool here, no per-tool files.

## license

MIT, see [docs/LICENSE](docs/LICENSE).

made it simple, but significant.

this is maintained by brenner cruvinel (brenner@hoffresearch.com).
all contributions are welcome.
(∂μfμν = jν)

Brenner Cruvinel
