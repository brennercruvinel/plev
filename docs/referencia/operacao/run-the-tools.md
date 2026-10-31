---
id: manual-run-the-tools
title: "how to run the tools"
project: plev
type: manual
status: ativo
date: 2026-10-27
date_basis: cabecalho
topics: [run, tools, render, regress, measure, agent, mirror, record, inspect, import, llm, web, mobile]
relations: []
source: docs/ref/run-the-tools.md
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# how to run the tools

the README runs the apps, the demos and the checks. this page has the rest,
one command a line, from the repository root. a word in `<angle brackets>`
is yours to fill in.

## a page without a window

```
cargo run -p uigallery -- --render Todo --theme hoff --width 1280 --height 800 --out todo.png
cargo run -p uigallery -- --render Todo --svg todo.svg --semantics todo.json
cargo run -p uigallery -- Typography hoff
```

the first draws a gallery page to png, the second as svg (each approximation
reported on stderr) and its semantic projection as json; the third opens the
window on one page in one theme (`core`, `hoff`, `dark`, `light`).

## regression and cost

```
cargo run -p regress -- check
cargo run -p regress -- bless --page Typography --theme hoff
cargo run -p regress -- capture
cargo run --release -p measure -- --out measurements.md
cargo bench -p editor
```

`check` compares every page with its baseline (meaning, then layers, then
pixels where a layer moved), `bless` accepts a change you meant, `capture`
draws every page in four themes at four scales. `measure` writes what each
stage of a frame costs; the criterion benches sit on each crate's hot path.

## the web, captured and compared

```
tool/tasks/webtarget.sh uigallery build --release
tool/tasks/screencap.sh web --page Typography --theme hoff
tool/tasks/screencap.sh web --all
```

`screencap.sh` needs python playwright and its chromium
(`python3 -m pip install playwright`, then
`python3 -m playwright install chromium-headless-shell`).

## an agent on the app

in one terminal, the gallery with its agent port open:

```
PLEV_AGENT=1 cargo run -p uigallery -- Todo
```

in a second terminal, what the agent sees, then an act on it (the revision
comes from the observation):

```
cargo run -p socket --bin plev -- observe --scope todo --app showcase
cargo run -p socket --bin plev -- act 'todo.item[1]' toggle --from-revision <revision>
```

the laboratory runs the git client's workbench, with no window, for an agent:

```
cargo run -p agenteval -- --arm scripted
```

## the mirror

in one terminal, the gallery sharing its scene; in a second, its window
drawn by another process:

```
PLEV_AGENT=1 PLEV_MIRROR=1 cargo run -p uigallery
cargo run -p mirroring -- --show
```

## a recorded session

`PLEV_RECORD=1` records the gallery or the git client; cmd-shift-s saves the
last seconds under `$TMPDIR/plev/sessions/`. read one back as text, or in a
window:

```
PLEV_RECORD=1 cargo run -p idestudio -- .
cargo run -p replay --bin plev-inspect -- <session.monster>
cargo run -p inspector --bin plev-inspector -- <session.monster>
```

the repository keeps two: `apps/idestudio/tests/fixtures/sessions/bug84.monster`
is one.

## the importers

```
cargo run -p import --bin convert -- data/media/lottie/cardstack/animation.json /tmp/cardstack.monster
cargo run --release -p player -- /tmp/cardstack.monster
cargo run -p mockup -- rust/import/tests/fixtures/react/index.tsx rust/import/tests/fixtures/react/NoteCard.module.sass rust/import/tests/fixtures/react/note-card-variables.sass
```

a lottie to `.monster` and played; a react component transpiled and shown
live.

## an intent, by the Claude

needs the `claude` command line of Claude Code, logged in:

```
cargo run -p intent -- --app todo "show the filter as chips and hide the counts"
```

## the language model in process

its own workspace; the weights stay outside the repository (a llama-family
checkpoint as Hugging Face ships it):

```
cargo test --manifest-path rust/llmrun/Cargo.toml --features candle,burn
cargo run --release --manifest-path rust/llmrun/Cargo.toml --features candle,metal --bin llm-bench -- --impl candle --model <dir> --device metal
```
