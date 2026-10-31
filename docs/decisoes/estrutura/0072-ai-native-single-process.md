---
id: adr-0072
title: "local llm inference in process: candle or burn"
project: plev
type: decisao
status: aceito
date: 2026-09-26
date_basis: cabecalho
topics: [llm, gpu, web-wasm]
relations: [task-41, task-78]
source: docs/adr/structure/0072-ai-native-single-process.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# local llm inference in process: candle or burn

Escolhe a biblioteca de inferência local do app de IA nativo da task-78 e mede candle e burn com o mesmo modelo, o mesmo laço de decodificação e o mesmo canal de streaming. A decisão de Brenner, de 2026-09-26, mantém as duas implementações atrás do trait `Model` e leva o plev ao wgpu 29, para que o burn rode no dispositivo do renderizador. O resultado é um processo, um dispositivo wgpu e uma fila: o candle decodifica mais rápido, e o burn é o único com caminho GPU no navegador.

## context

task-78 wants an app that answers from a local model in streaming, in the
same rust process as the ui and the renderer, with no browser, no ipc and
no backend ([`task-78-ai-native-apps.md`](../../tarefas/task-78-ai-native-apps.md)). the
first step is the inference library. brenner picked the two candidates on
2026-09-25: candle (with tokio for the streaming) and burn. the choice is
his (decision 15 of the program); this adr measures and recommends.

both run the same model, TinyLlama 1.1B Chat v1.0 (Apache-2.0, bf16
safetensors, outside the repository), behind one interface in
`rust/llmrun`, its own workspace like urnaui:

- `llm::Model`: reset the kv cache, feed ids at a position, get the greedy
  next id. `llm::stream::spawn` runs the decode loop on its own thread and
  hands out a `Stream`: tokens one by one on a tokio mpsc channel, awaited
  or blocked on, a `Cancel` that stops before the next token, and always a
  final `Done(Eos | Length | Cancelled)` or `Failed`. the channel and the
  loop are the same for both libraries, so the numbers compare only the
  inference.
- `llm::text`: the tokenizer (`tokenizers` 0.22), an incremental
  detokenizer that never splits a character, and the zephyr chat template.
- `llm::candle`: `candle-transformers`' llama, which reads the Hugging Face
  checkpoint as it is.
- `llm::burn`: burn 0.21 has no llama. the model code is adapted from
  burn-lm's `burn-lm-llama` (commit a53baab, MIT OR Apache-2.0,
  `rust/llmrun/NOTICE`), about 300 lines after the port: the backend
  parameter back, the checkpoint's field names, weights read by
  burn-store from the safetensors with bf16 cast on the way in, and the
  Hugging Face rope with f32 tables (burn's own rope builds its angles in
  the backend float, which in f16 loses the position). burn-lm was
  archived on 2026-09-04.

the two produce the same 32 first tokens, id for id, on Metal f16 and on
the cpu in f32.

## the measurement

Apple M4 (4 P and 6 E cores, 10-core gpu), 16 GB, macOS 26.6.2, release
with thin lto. the same prompt (37 tokens), 128 new tokens per round,
greedy; one warm-up process, then three processes of five rounds each,
one at a time; the median of the three medians. method and the command
that reproduces it are in the task file, "fase 1: a medição".

| | candle, Metal f16 | burn, Metal f16 | candle, cpu f32 | burn, cpu f32 |
| --- | --- | --- | --- | --- |
| load, file to ready | 4.33 s | 3.49 s | 3.29 s | 6.14 s |
| first token, first round of a process | 90 ms | 911 ms | 990 ms | 1489 ms |
| first token, median | 52 ms | 60 ms | 409 ms | 922 ms |
| decode, median | 37.1 tok/s | 26.2 tok/s | 6.7 tok/s | 1.9 tok/s |
| peak rss | 3264 MB | 2779 MB | 4626 MB | 5212 MB |
| peak footprint | 5182 MB | 4325 MB | 4753 MB | 5644 MB |
| bench binary, this library alone | 6.8 MB | 33.0 MB | same | same |
| `wasm32-unknown-unknown`, cpu features | compiles | compiles | | |

cold, the first process after a build: candle 9.0 s to load and 5.2 s to
the first token (Metal shaders compiled once); burn 4.6 s and 10.8 s, and
10.7 tok/s in that first round, because the autotune runs then. burn keeps
the autotune results in `target/autotune` next to the nearest
`Cargo.toml`, or in the user cache directory for an installed app; each
new process still compiles its kernels, the 0.9 s of the second row.

burn's `Linear` layout was measured both ways: keeping the checkpoint's
[out, in] and transposing the view (`Col`) wins on Metal (26.3 against
24.4 tok/s, and a load without copying); transposing the bytes at load
(`Row`) wins on the cpu (1.8 against 0.3 tok/s). `llm::burn::layout_for`
picks per device.

## wasm

with only the cpu features, both compile for `wasm32-unknown-unknown`
once `getrandom` gets the browser backend (feature `wasm_js` on 0.3 and
0.4, and `--cfg getrandom_backend="wasm_js"` for 0.3); without it both
stop there first. the tokenizer does not compile for wasm (its regex
engine, onig, is C; the pure-rust path wants a crate not in the cache),
so a browser build needs a different tokenizer path for either library.
on the gpu in the browser only burn has a path, its `webgpu` backend;
candle has no wgpu backend, so in the browser it is cpu only.

## the renderer's gpu (scope item 4)

- burn runs on wgpu, through cubecl, and accepts an existing wgpu device
  (`init_device(WgpuSetup { instance, adapter, device, queue })` in
  cubecl-wgpu). sharing the renderer's device is the intended path, but
  burn 0.21 is on wgpu 29 and plev on 28: it needs plev on 29 first.
- candle talks to Metal directly and creates its own device and queue,
  with no constructor that takes an existing one. sharing buffers with
  the renderer would be unsafe interop through wgpu-hal, on the same gpu
  with separate queues; wgpu-hal 29 uses the same metal bindings as candle
  (objc2-metal), 28 does not.

## options

1. candle. faster where the app spends its time: 1.4 times the decode rate
   on Metal, 3.5 times on the cpu, a first token in 90 ms in a fresh
   process instead of 0.9 s, a fifth of the binary. the llama, and the
   other architectures in `candle-transformers`, are maintained upstream.
   costs: more peak memory (5.2 against 4.3 GB of footprint), its own
   Metal device, cpu only in the browser.
2. burn. less memory, the only one of the two with a gpu path in the
   browser and a sanctioned way onto the renderer's wgpu device. costs:
   slower on every row that is speed, a first token near 1 s in each new
   process and 11 s on the first run of an install, a binary five times
   bigger, and the model code is ours to keep: burn has no llama and
   burn-lm is archived. burn-lm's own notice says its llama was adapted
   from Meta's Llama 3 reference code, under the Llama 3 community
   license; that chain would need a look before shipping it.

## recommendation

candle, for the reference app. the app's job is to stream an answer, and
candle does it faster, from the first token of a fresh process to the
last, with a model code base someone else maintains. what burn offers
over it, the shared wgpu device and webgpu in the browser, is not usable
today (wgpu 29 against 28) and belongs to later items; the gpu compute of
item 4, like projecting embeddings for the graph, can be plev's own wgpu
compute on the renderer's device without an ml framework. the `Model`
trait keeps the door open: burn stays a feature of `rust/llmrun` until the
decision, and can come back if the shared device or the browser gpu
becomes the priority.

## decision (brenner, 2026-09-26)

both stay. read the numbers above, then brenner decided: "vamos manter o
burn e candle", and, about burn needing a newer wgpu than plev has,
"implementa". so:

- `rust/llmrun` keeps the two implementations behind the one `Model`
  trait, each behind its feature. the reference app can run either one.
- plev moves from wgpu 28 to 29, the version burn 0.21 is on, so that
  burn can run on the renderer's own device (`WgpuSetup`), as scope item
  4 asks. that is the next step of task-78. it lands before the
  reference app, and pixel identity has to hold.
- the licence chain of the burn model code (burn-lm, adapted from Meta's
  Llama 3 reference code, `rust/llmrun/NOTICE`) is looked at before any
  public release (task-41).

## the shared device (task-78 phase 2)

plev moved to wgpu 29.0.4, and the 1312 desktop frames of `regress
capture` (82 pages, 4 themes, 4 scales) came out equal in every byte to
the wgpu 28 capture taken before the move. then burn went onto the
renderer's device:

- the engine hands out what cubecl's `init_device` takes:
  `GpuContext::instance()` and `adapter()`, by addition, beside the
  `device` and `queue` fields it already had.
- `RenderConfig::compute`, `None` by default, asks the device for the
  adapter's limits and for the non-experimental features the adapter has.
  cubecl picks its kernels by the adapter's features (f16, i64, subgroups)
  and sizes buffers by the device's limits, so a device with the
  renderer's defaults would reject them (TinyLlama's embedding alone
  outgrows the default 256 MB buffer). without it the renderer asks for
  what it asked before.
- `rust/llmrun`, feature `plev`: `llm::plev::device` wraps the four
  handles in a `WgpuSetup`, and `llm::plev::load` loads the model on that
  device through burn's wgsl path.

## consequences

what it shows. one process, one wgpu device, one queue. in
`rust/llmrun/tests/engine.rs` the headless renderer's device goes to burn,
a matmul on it reads back equal to the cpu's, and the same renderer then
draws a frame equal in every byte to one from a renderer burn never
touched. with the model present (`tests/real.rs`, ignored by default),
TinyLlama answers "The capital of France is Paris." in f16 on the
engine's device, then the same device draws a frame. two runs of that
process, load and 16 tokens included, took 8.7 and 7.6 s, with a peak
footprint of 4.3 and 4.0 GB.

what it does not show:

- no tensor reaches a frame. the test reads the matmul back to the cpu
  to compare it; drawing from a burn buffer without that copy, the second
  half of scope item 4, is not built.
- no speed. the phase 1 numbers are burn on its own device through msl;
  this is the wgsl path on a shared device, not measured, and neither is
  a frame and a decode taking turns on one queue.
- burn with the `metal` feature does not run here: it compiles to msl
  through passthrough shaders, an experimental wgpu 29 feature the
  engine's device does not enable. `llm::plev::device` refuses that pair.
- after a device loss the engine rebuilds its device, and the device
  burn holds is gone with the old one. the caller has to register the new
  one; nothing tests that with burn yet.
- only the headless path ran. a window's device comes from the same
  descriptor, but no windowed app has burn on it.
- candle still makes its own Metal device.

## alternatives

- candle sozinho, como recomendava `## recommendation`: não foi a decisão. Brenner escolheu manter as duas implementações atrás do mesmo trait `Model`.
- burn sozinho: teria menos memória e o único caminho GPU no navegador, mas perderia em todas as linhas de velocidade e teria um binário cinco vezes maior, com código de modelo próprio a manter (`## options`, item 2).
- compartilhar buffers do renderizador com o candle por wgpu-hal: rejeitado como interop insegura entre filas separadas no mesmo GPU (`## the renderer's gpu (scope item 4)`).
- dispositivo do renderizador para o candle: indisponível, porque o candle não tem construtor que receba um device existente (`## the renderer's gpu (scope item 4)`).
- burn com a feature `metal` no dispositivo compartilhado: recusado pela função `device` de `rust/llmrun/src/engine.rs:42` (o corpo a chama de `llm::plev::device`), porque depende de passthrough shaders que o device do engine não habilita (`## the shared device (task-78 phase 2)`).
