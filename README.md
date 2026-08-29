# plev


this is a personal experiment.this engine builds a scene every frame, resolves only the layers that
changed, and composites into an srgb surface. desktop draws it on metal, the
web draws it on webgpu, and the pixels are identical because the same code
runs on both. glass, backdrop blur, analytic shadows, content-driven layout,
real text shaping. the apps consume the engine. nothing bypasses it.

ompositor with
dirty-layer caching, a measured visual language, the same pixel on every
platform, and an animation format of its own. those pieces do not exist
assembled anywhere else.

## architecture

one render-on-demand loop. no change, no frame. unchanged layers cost zero
gpu work.

```mermaid
sequenceDiagram
    participant E as event (winit/touch/ime)
    participant A as app state
    participant S as scene build (per frame)
    participant C as compositor (dirty layers)
    participant T as text system (atlas)
    participant G as gpu passes
    participant P as surface (srgb view)

    E->>A: pointer / synthesized touch, hit regions
    A->>A: mutate state, invalidate
    Note over A,S: render on demand: no change, no frame
    A->>S: build scene nodes (monster Player can feed these)
    S->>C: push to layers (hash per layer)
    C->>C: resolve only dirty layers
    C->>T: resolve text (one TextStyle, measure == draw)
    C->>G: encode passes (quads, sdf, shadow, image, text, backdrop)
    G->>P: composite into srgb view (encode on write)
    P->>P: present (identical pixels on every target)
```

three tiers. the engine is its own crate at `crates/engine`; the repo root is
a virtual workspace. libraries and apps are sibling crates. demos are the
engine's examples.

| tier | where | what |
|---|---|---|
| engine | crate `engine` (crates/engine) | gpu, compositor, text, path, layout, input, animation, signal, theme, ui, builder, window, platform |
| crates | crates/ | git, ide, lot, macros, monster, svg, narrate, narrate-macro, nestui, parser, prime, rope, showcase |
| examples | crates/engine/examples/ | 16 windowed demos plus the lot2monsters and svg2monster clis |

machine map: [docs/arc/arc.yaml](docs/arc/arc.yaml). human reference:
[docs/arc/arc.md](docs/arc/arc.md). frame-flow diagram:
[docs/arc/arc.mmd](docs/arc/arc.mmd).

## binding contracts

violations are defects even when the pixels look right.

- one TextStyle per text run, shared by measurement and drawing. the only
  width source is the shaper, never an arithmetic estimate.
- srgb decode once entering gpu work, encode once at the surface write.
- container geometry derives from available space, never from constants.
- a visible state change implies invalidation. no silent redraws, no silent
  stale frames.

## install

requires rust edition 2024.

```
cargo build --release --workspace
```

web build (same pixels as desktop):

```
./script/web
```

## run

```
cargo run -p showcase                  # the design-system gallery, 11 tabs
cargo run -p nestui [file.nest]        # .nest vector-db explorer (drop a file to open)
cargo run -p ide [path]                # plev-native git client
cargo run -p prime                     # prime-coherence particle swarm (codepen port)
cargo run -p engine --example charts   # any of the 16 windowed demos
cargo run -p engine --example snake
```

the monster animation pipeline, a foreign format in and our format out:

```
cargo run -p engine --example lot2monsters in.json out.monster   # lottie -> monster
cargo run -p engine --example svg2monster  in.svg  out.monster   # svg (still) -> monster
cargo run -p engine --example monster_player out.monster         # play ours, no foreign code
```

`svg` imports a still image (usvg normalizes the document, we tessellate the
paths into one keyframe); gradients approximate to a solid color and filters,
masks, clips and text are skipped, so raster-heavy svgs reduce to their vector
core.

## mobile

the showcase runs natively on android and ios from the same engine. it is a
library (`crate-type = ["lib", "cdylib", "staticlib"]`) whose app shell exposes
one entry point per platform; there is no kotlin or swift ui, every pixel is
drawn on the gpu by plev.

ios (simulator) needs xcode, the `aarch64-apple-ios-sim` rust target and
`xcodegen`:

```
cd ios/showcase
./build_ios.sh      # cargo staticlib + xcodegen + xcodebuild
./run_ios.sh        # boot a simulator, install, launch, screenshot
```

android needs the sdk/ndk, a jdk 17 and `cargo-ndk`:

```
cd android
./build_android.sh  # cargo-ndk -> jniLibs (arm64-v8a, x86_64) + gradle apk
                    # -> app/build/outputs/apk/debug/app-debug.apk
```

a GameActivity host (`MainActivity.kt`) loads `libshowcase.so` and hands off to
the rust `android_main`; on ios a thin objc `main.m` calls `showcase_ios_main`.
the engine keeps its own `android_main` behind the `android-entry` feature (off
by default) so downstream apps ship their own without a symbol clash.

## the monster format

a binary animation format, magic `MON0`, frozen at v1. keyframes are full
scene snapshots so seek is O(1); between them only discovered deltas travel,
and a node that does not change costs zero bytes. quantized on the wire,
checksummed per section, a description track per keyframe for accessibility
and search. it plays on the same engine that draws the ui. `lot` reads a
lottie json once, converts it to `.monster`, and never embeds a foreign
runtime. discrete motion already beats the source json on size; full-body
morphs wait on the v1 morph-track lever. see
[docs/adr/monster-format-v0.md](docs/adr/monster-format-v0.md).

## the parser

`crates/parser` turns ui source from another framework into plev builder
code. it maps colors to hoff theme tokens, obeys the engine contracts, and
reports every construct it cannot represent on a droplist with file and
line. it never drops a construct silently. run it live:

```
cargo run -p parser --example transpile  index.tsx module.sass vars.sass
cargo run -p parser --example preview    index.tsx module.sass vars.sass
```

## tests

```
cargo test --workspace
cargo clippy --workspace
cargo fmt --check
./script/gate     # the full four-part gate, stops on the first red
```

a guard test scans the repo and fails if any code draws text by constructing
a raw key instead of going through the one-TextStyle path.

criterion benches live on the hot path of each crate (`engine` scene build,
`rope` edits, `monster` codec, `lot` conversion, `parser` transpile):

```
cargo bench                      # all benches
cargo bench -p rope              # one crate
```

the `monster` dense-lottie benches read large samples from `refs/lottie`
(gitignored study material); when it is absent they skip with a notice
instead of failing.

## reference

- [docs/arc/arc.md](docs/arc/arc.md) for architecture, contracts, frame flow.
- [docs/arc/arc.yaml](docs/arc/arc.yaml) for the machine-readable map agents read.
- [docs/adr/monster-format-v0.md](docs/adr/monster-format-v0.md) for the animation format spec.
- [docs/adr/](docs/adr/) for the architecture decision records.
- [docs/how-to/code-against-the-plev-engine.md](docs/how-to/code-against-the-plev-engine.md) for the operating manual.
- [.contracts/.agents/AGENTS.md](.contracts/.agents/AGENTS.md) for the single instruction source for ai agents and contributors. route every tool here, no per-tool files.

## license

Brenner Cruvinel.
(∂μfμν = jν)

MIT.

`.contracts/.agents/AGENTS.md` is the single instruction source: the operating contract for ai agents and contributors. the root carries no instruction file; every tool routes to that path on init, no per-tool files.
