---
id: experiment-local-inference-candle-vs-burn
title: "candle against burn for local LLM inference in process, TinyLlama 1.1B"
project: plev
type: experiment
status: reference
date: 2026-09-26
date_basis: text
recorded: 2026-10-31
topics: [llm, gpu, performance, web-wasm]
relations: [task-78, adr-0072]
evidence: [94109cd3, f6ff13fb, ddd0928c, 496c6ffa, 002debd4]
source: reconstruction
source_revision: c2e3f8cd
revised: 2026-10-31
review: reviewed
---

# candle against burn for local LLM inference in process, TinyLlama 1.1B

task-78 needed the inference library of an AI-native app that answers from a local model in streaming, inside the same Rust process as its UI and renderer. ADR 0072 measured candle and burn with the same model (TinyLlama 1.1B), the same decode loop and the same streaming channel: candle decoded faster and had the smaller binary, while burn used less memory on Metal and was the only one with a GPU path in the browser. The ADR recommended candle; brenner kept both behind one `Model` trait, and burn later ran on the renderer's own wgpu device in a headless test, with no tensor yet reaching a frame.

## Question

task-78's scope, item 2, asks for in-process inference: a comparison of candle (with tokio for the streaming) and burn, the two that brenner chose on 2026-09-25, by size, speed and wasm, with the decision recorded in an ADR together with brenner. The phase opened on 2026-09-26 to measure candle against burn with the same model and to write an ADR with a recommendation. The choice of library is brenner's (decision 15 of the program).

## Setup

- **Machine.** Apple M4 (4 performance and 6 efficiency cores, 10-core GPU), 16 GB, macOS 26.6.2, rustc 1.98.1. The Mac was not idle: Finder and other processes ran, with a load average near 5 (task-78).
- **Build.** Release profile of the workspace `rust/llmrun`, thin LTO and one codegen unit (task-78; the ADR says "release with thin lto"). One `llm-bench` binary per implementation, with CPU and Metal inside: `llm-bench --impl candle|burn --device metal|cpu`, built with the features `candle,metal` or `burn,metal`. Commits: `094d53da` (llm-bench and the burn layout), `94109cd3` (the candle measurement), `f6ff13fb` (the burn measurement).
- **Libraries.** candle 0.11 (`candle-*-0.11.0`, its llama) and burn 0.21 (`burn-*-0.21.0`). burn 0.21 has no llama, so the burn model code is adapted from burn-lm at commit `a53baab`. The tokenizer (`tokenizers` 0.22) is shared by both.
- **Model.** TinyLlama 1.1B Chat v1.0 (Apache-2.0), from the Hugging Face repository at commit `fe8a4ea1`. `model.safetensors`: 2,200,119,864 bytes, sha256 `6e6001da2106d4757498752a021df6c2bdc332c650aae4bae6b0c004dcf14933`. It lives outside the repository. The weights are bf16 in the file, f16 on Metal and f32 on the CPU.
- **Prompt and decoding.** "Write a short story about a lighthouse keeper who finds a message in a bottle." in the chat template: 37 tokens. 128 new tokens per round, greedy decoding; the end-of-sequence token does not stop a round.
- **Protocol.** One warm-up process, discarded (its line is the cold start). Then three processes, one at a time, each with five rounds; the table gives the median of the three medians. Before each process at least 40% of memory is free (checked with `memory_pressure`). Compilation and measurement never run at the same time.
- **Definitions.** Load: from the file to the model ready on the device (config, mmap, dtype conversion, upload, sync), without the tokenizer (0.03 s). TTFT: from the call to the first token received on the channel, prefill of the 37 tokens included. Decode: the 127 tokens between the first and the last token. Peak RSS: `ru_maxrss`. Peak footprint: `ri_lifetime_max_phys_footprint`, which counts Metal memory.
- **Command.** From the repository root: `cargo build --offline --release --manifest-path rust/llmrun/Cargo.toml --features candle,metal --bin llm-bench`, then `llm-bench --impl candle --model <model directory> --device metal`.
- **Not recorded.** The time of each run. The phase carries the date 2026-09-26.

## Results

### Measurement

Values as ADR 0072 writes them. task-78's "fase 1: a medição" table has the same values with decimal commas.

| | candle, Metal f16 | burn, Metal f16 | candle, CPU f32 | burn, CPU f32 |
|---|---|---|---|---|
| load, file to ready | 4.33 s | 3.49 s | 3.29 s | 6.14 s |
| first token, first round of a process | 90 ms | 911 ms | 990 ms | 1489 ms |
| first token, median | 52 ms | 60 ms | 409 ms | 922 ms |
| decode, median | 37.1 tok/s | 26.2 tok/s | 6.7 tok/s | 1.9 tok/s |
| peak RSS | 3264 MB | 2779 MB | 4626 MB | 5212 MB |
| peak footprint | 5182 MB | 4325 MB | 4753 MB | 5644 MB |
| bench binary, this library alone | 6.8 MB | 33.0 MB | same | same |
| `wasm32-unknown-unknown`, CPU features | compiles (see Wasm) | compiles (see Wasm) | | |

ADR 0072 puts candle's decode rate at 1.4 times burn's on Metal and 3.5 times on the CPU, and its first token in a fresh process at 90 ms against 0.9 s.

The two produce the same 32 first tokens, id for id, on Metal f16 and on the CPU in f32 (ADR 0072). task-78 says the 32 tokens are identical in the four columns and in every round.

Checks in phase 1: `rust/llmrun` with both features passed 24 tests (the model interface with a fake model, the template, the detokenizer, the rope, the prefill against the step). The two model tests, ignored by default, ran on Metal in release and answered "The capital of France is Paris." clippy and fmt were green.

### Cold start and layout

Cold, the first process after a build: candle loaded in 9.0 s and gave the first token in 5.2 s (Metal shaders compiled once). burn, with its autotune cache empty, loaded in 4.6 s, gave the first token in 10.8 s, and decoded the first round at 10.7 tok/s, because autotune runs then. From the second process the first round falls to near 1 s (ADR 0072 says 0.9 s, the 911 ms row), which is kernel compilation.

burn's `Linear` layout was measured both ways. On Metal, keeping the checkpoint's [out, in] layout with a transposed view (`Col`) decodes at 26.3 tok/s and loads without copying. Transposing the weights at load (`Row`) decodes at 24.4 tok/s and loads in 7.8 s (task-78 gives the load time; the ADR gives only the rates). On the CPU it is the reverse: `Col` 0.3 tok/s, `Row` 1.8 tok/s. `llm::burn::layout_for` picks the layout per device.

### Wasm

With only the CPU features, both compile for `wasm32-unknown-unknown` once `getrandom` has its browser backend: feature `wasm_js` on 0.3 and 0.4, and `--cfg getrandom_backend="wasm_js"` for 0.3. Without it, both stop there first. The tokenizer does not compile for wasm: its regex engine, onig, is C, and the pure-Rust path needs a crate that is not in the cache. A browser build therefore needs another tokenizer path for either library. On the GPU in the browser only burn has a path (its `webgpu` feature); candle has no wgpu backend, so in the browser it is CPU only.

### Recommendation and decision

ADR 0072's recommendation was candle for the reference app: it streams faster from the first token of a fresh process to the last, and its model code is maintained upstream. brenner read the numbers and decided on 2026-09-26, in his words translated from Portuguese, to "keep burn and candle", and, about burn needing a newer wgpu than plev had, to "implement it". Both stay behind the `Model` trait. plev moves from wgpu 28 to 29. ADR 0072 was accepted with that decision (decision 16 of the program).

### After the decision: burn on the renderer's device

These results are from task-78 phase 2 and ADR 0072's "the shared device" section. They concern burn only; candle has no constructor that takes an existing device.

| check | result |
|---|---|
| wgpu 28 to 29 (a precondition: burn 0.21 runs on wgpu 29) | the 1312 desktop frames of `regress capture` (82 pages, 4 themes, 4 scales) came out equal to the wgpu 28 capture in every byte: 1312 of 1312 `equal` |
| headless test on the engine's device | ADR 0072 names `rust/llmrun/tests/engine.rs`; task-78 describes the same test as `tests/plev.rs`, and the two pages do not say which name is current. A matmul on the engine's device reads back equal to the CPU's, and the same device then draws a frame equal in every byte to one from a renderer burn never touched |
| TinyLlama on the engine's device (ignored by default, model present) | answers "The capital of France is Paris." in f16, then the same device draws a frame. ADR 0072: two runs, load and 16 tokens included, took 8.7 and 7.6 s, with a peak footprint of 4.3 and 4.0 GB. task-78 gives the 7.6 s run, with 4.0 GB |
| gate | `rust/llmrun` with `candle,burn,plev`: 27 passed and 3 ignored (the ones that need the model). The root gate went from 3268 to 3272 passed; the four new tests are the engine's |

The sharing works through the engine, which now hands out the instance and adapter; `RenderConfig::compute` (off by default) asks the device for the adapter's limits and features, because cubecl sizes its buffers by the device's limits. `llm::plev::load` then loads the model on that device through burn's wgsl path.

### Negative and failed results

The sources record these results against burn or against the path:

- burn is slower on every speed row of the table.
- burn's cold first token is 10.8 s (ADR 0072 says 11 s on the first run of an install), against 5.2 s for candle.
- burn with the `metal` feature does not run on the engine's device. `llm::plev::device` refuses that pair, because it depends on passthrough shaders that the engine's device does not enable (ADR 0072 cites `rust/llmrun/src/engine.rs:42` for the function).
- candle cannot share the renderer's device: it has no constructor that takes an existing one, and it keeps its own Metal device.
- In the browser, candle is CPU only.
- The tokenizer does not compile for wasm, for either library.

## Limits

- **Not compared.** Electron is excluded from the comparison by brenner's decision of 2026-09-25 (task-78).
- **One machine, not idle.** The measurement ran on one M4 with 16 GB, with Finder and other processes running and a load average near 5.
- **One model, one prompt.** TinyLlama 1.1B, one prompt, greedy, 128 new tokens. The sources measure nothing else.
- **Memory.** Peak footprint includes the mapped model file during load. The memory that remains after load is not measured (task-78 pending items).
- **Autotune.** burn keeps autotune results in `target/autotune` beside the nearest `Cargo.toml`, or in the user cache for an installed app. task-78 notes that an installed app needs a place of its own. Each new process still compiles its kernels.
- **Size of burn's model code.** The task-78 probe estimated "about 600 lines of production code" for the port, with the `<B>` parameter back, the loader and the rope. The handoff in task-78 and ADR 0072 say "about 300 lines" after the port. Both are recorded here; the later count is the one the ADR uses.
- **Memory claim in the ADR.** ADR 0072's option 2 says burn uses "less memory". The table shows that only on Metal: on the CPU, burn's peak RSS and footprint are higher (5212 MB against 4626 MB, and 5644 MB against 4753 MB).
- **Licence.** burn-lm, which the burn model code is adapted from, has its llama adapted from Meta's Llama 3 reference code under the Llama 3 community license, according to burn-lm's own notice. ADR 0072 says that chain "would need a look before shipping it", and its decision puts the check before any public release (task-41). burn-lm was archived on 2026-09-04, so the burn model code is now the project's to keep.
- **Shared device, what it does not show.** No speed is measured on the shared device: the phase 1 numbers are burn on its own device through msl, and this is the wgsl path on a shared device. No tensor reaches a frame without going back to the CPU; the second half of item 4 is not built. Device loss is not tested: after a loss, the caller must register the new device. Only the headless path ran; no windowed app has burn on its device.
- **Wasm.** Only compile checks were run; no browser run.
- **Date.** The phase carries the date 2026-09-26 in both sources. The individual runs are not dated.
- **No single winner.** The recommendation is candle for the reference app, but the decision keeps both libraries. This page records that decision and does not pick one.

## Records

- [task-78 README](../../Tasks/task-78-ai-native-apps/task-78-ai-native-apps.md): the program; the phase 1 probe, method, measurement table, decision and handoff; phase 2, with the wgpu 29 move, the pixel proof and the shared device.
- [ADR 0072](../../adr/Ecosystem/0072-ai-native-single-process/0072-ai-native-single-process.md): the context, the options, the measured table, the recommendation, the brenner decision, and the shared device with what it does not show.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0072: local llm inference in process: candle or burn](../../adr/Ecosystem/0072-ai-native-single-process/0072-ai-native-single-process.md)
- [task-78: AI-native applications](../../Tasks/task-78-ai-native-apps/task-78-ai-native-apps.md)
<!-- /related -->
