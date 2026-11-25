---
id: task-78
title: "task-78: AI-native applications"
project: plev
type: task
status: in-progress
date: 2026-09-26
date_basis: header
recorded: 2026-09-23
topics: [llm, gpu, performance, research]
relations: [adr-0025, adr-0026, adr-0072, task-66, task-73, task-76, experiment-local-inference-candle-vs-burn, learning-generative-runtime]
evidence: [6e2fce00, d095aeb5, a60b00c4, 2f91d01b, 094d53da, 94109cd3, f6ff13fb, 0dec09a9, a33b74a7, ddd0928c, 71993e9c, ca7cb9eb, 4a04d454, 8839f3c8, bcf0a205, 06df04c8, 496c6ffa, 002debd4, 9e46ce7a]
source: docs/log/steps/task-78-ai-native-apps.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# task-78: AI-native applications

Records the program that measures whether an app with a local model, embeddings, audio, video, canvas, token streaming, visualizations and GPU compute runs better as a single Rust process than on the Chromium, JS, DOM, IPC and native-backend stack. Phase 1 compared candle and burn with TinyLlama 1.1B on Metal f16 and on CPU f32 (decode of 37.1 and 26.2 tok/s on Metal; 6.7 and 1.9 on CPU); brenner kept both implementations, and ADR 0072 was accepted. Phase 2 moved plev to wgpu 29 with 1312 of 1312 frames equal to the pixel baseline of wgpu 28, and moved burn onto the engine's device, without the half that shows the result on screen; the reference app and the final measurement do not exist yet.

## objetivo

mostrar, medindo, que um app com modelo local, embeddings, áudio, vídeo,
canvas, milhares de tokens em streaming, visualizações e GPU compute roda
melhor como um processo Rust com runtime da aplicação, modelo e
renderização juntos do que como Chromium, JS heap, DOM, IPC, backend
nativo e GPU. o foco é a próxima geração de desktop AI.

## ligação com a tese

seção 17 (aplicações AI-native; um processo Rust; onde memória, latência,
GPU, streaming e interação rica fazem o custo das abstrações importar),
seção 18 (GPU-first é infraestrutura). a nota "talvez para além das
fronteiras da arquitetura de von neumann" fica registrada como horizonte de
pesquisa, sem critério de aceite neste programa.

## estado atual no código

- o urnaui já é a semente: explorer de banco vetorial `.urna` com busca
  exata, ann, graph e híbrida, embedders offline chamados em subprocesso
  python, frames de mídia por ffmpeg lidos no lugar, worker que acorda o
  loop (`apps/urnadbapp`, ADRs `urnaui-media-frames-in-place-via-ffmpeg-subfile`
  e `worker-results-wake-the-event-loop`).
- charts e grafo com layout de força em comps e engine.
- sem inferência de LLM em processo; sem captura de áudio; sem compute
  shader de app.

## escopo

1. app de referência (o urnaui estendido ou um app novo, decisão no
   briefing): pergunta sobre um corpus `.urna`, resposta de um LLM local em
   streaming com as fontes citadas, visualização dos trechos no grafo.
2. inferência em processo: comparação entre candle (com tokio para o
   streaming) e burn, as duas que o brenner escolheu (2026-09-25), por
   tamanho, velocidade e wasm; decisão por ADR com o brenner; modelos fora
   do repositório.
3. streaming de tokens para a view: shaping e layout incrementais (depende
   da 73); latência de token até pixel medida.
4. GPU compute no mesmo device do renderer para algo que a UI mostra (por
   exemplo projeção de embeddings para o grafo), sem cópia de volta para a
   CPU quando não precisa.
5. vídeo: frames decodificados direto para textura; áudio: captura e
   reprodução (o microfone vem da task-76).
6. medição do app de referência: memória, startup, latência de token até
   pixel, tempo de frame sob streaming. metodologia escrita antes de medir.
   sem comparação com Electron (decisão do brenner, 2026-09-25).

## fora de escopo

treinar modelo; hardware fora de von neumann.

## dependências

independente para começar; 73 para o streaming barato; 66 para as capturas.

## superfície sob responsabilidade

o app de referência, o módulo de inferência (crate próprio se a decisão
pedir), ADR [`0072-ai-native-single-process.md`](../../adr/Ecosystem/0072-ai-native-single-process/0072-ai-native-single-process.md). `apps/urnadbapp` só se
ele for o app escolhido.

## interfaces que consome

engine, comps, urna runtime (via urnaui), `sem` para que o app também seja
operável por agente.

## interfaces que pode alterar

as próprias.

## critérios de aceitação

- app de referência rodando no macOS com modelo local em streaming e fontes
  citadas.
- tabela de medição do app de referência, com metodologia.
- `./tool/tasks/fullcheck.sh` verde (o app, se depender de coisa fora do workspace,
  segue o modelo do urnaui: workspace próprio e perna opcional no gate).

## testes esperados

testes do módulo de inferência com um modelo minúsculo de fixture ou com a
interface falsa; testes de streaming para a view (sem frame velho).

## riscos e perguntas abertas

- tamanho de modelo e binário; wasm provavelmente fora para inferência.

## commits esperados

grupos por item, `task-78:` no início.

## pare e reporte se

- a decisão de biblioteca de inferência (sempre do brenner);
- o app escolhido exigir dependência fora do workspace.

## fase 1 aberta (orquestrador, 2026-09-26)

aberta com o ok do brenner ("sim, pode orquestrar com o agente", e o
espaço em disco liberado por ele). a fase mede o candle contra o burn
com o mesmo modelo e escreve o ADR com a recomendação; a escolha da
biblioteca é do brenner (decisão 15 do programa). é o primeiro passo do
item "app de referência rodando no macOS com modelo local em streaming e
fontes citadas".

o modelo, baixado pelo orquestrador, fica fora do repositório:
`~/.cache/plev/models/tinyllama-1.1b-chat-v1.0/`, TinyLlama 1.1B Chat
v1.0 (Apache-2.0), commit `fe8a4ea1` do repositório no Hugging Face,
`model.safetensors` com 2.200.119.864 bytes e sha256
`6e6001da2106d4757498752a021df6c2bdc332c650aae4bae6b0c004dcf14933`.
o TinyLlama porque o candle (`candle-transformers`, llama) e o burn (a
implementação do `burn-lm-llama`) rodam o mesmo modelo, e a comparação só
é justa assim.

## fase 1: a sonda

fontes: o registro do cargo nas versões travadas (`candle-*-0.11.0`,
`burn-*-0.21.0`) e o clone do burn-lm no commit `a53baab`.

(a) candle 0.11 com o TinyLlama:

- o `config.json` desserializa em `LlamaConfig` (candle-transformers
  src/models/llama.rs:39-54, GQA em :57, `into_config` em :67); o
  flash-attn é só cuda (Cargo.toml:48-51), `false` no Metal.
- os nomes do safetensors do Hugging Face entram sem remapear
  (llama.rs:515-525); um tensor faltando vira panic (`unwrap()`, :524).
- dtype: `VarBuilder::from_mmaped_safetensors` converte o bf16 no dtype
  pedido (candle-nn src/var_builder.rs:642, `unsafe` pelo mmap): f16 no
  Metal, f32 na CPU. a atenção sobe q, k, v para f32 (llama.rs:337-340) e
  o rope nasce em f32 (:200-207); o f16 não perde posição.
- device `Device::new_metal(0)` (candle-core src/device.rs:258); o kv em
  `Cache::new` (llama.rs:164); `forward` devolve os logits f32 da última
  posição (:503-512).
- o template não está no candle: vem do `tokenizer_config.json`
  (`<|user|>\n{prompt}</s>\n<|assistant|>\n`); o `<s>` é do pós-processador
  do `tokenizer.json`; o eos é o id 2.

(b) o `burn-lm-llama` no burn 0.21:

- o código segue o burn da main, sem o parâmetro de backend: `Tensor<1,
  Int>` e `Device` soltos (src/nn/llama/inference.rs:30-39), `Transformer`
  sem `<B>` (src/nn/transformer.rs:78-85). no 0.21 é `Tensor<B, D, K>`
  (burn-tensor src/tensor/api/base.rs:76-80). o port é mecânico.
- os pesos: o burn-lm lê um `.mpk` próprio (nn/llama/base.rs:237-246) ou
  `.pth` pelo `burn_import` (nn/llama/import.rs:2, :90), nunca o
  safetensors. o burn-store lê (`SafetensorsStore::from_file`,
  src/safetensors/store.rs:115), remapeia por regex (:294) e transpõe o
  `Linear` (`PyTorchToBurnAdapter`, src/adapter.rs:318, :416-418), mas não
  renomeia o peso do `RmsNorm` (:374-378) nem converte o dtype
  (src/applier.rs:240). faltam um adaptador bf16 para o float do backend e
  o remapeamento do import.rs:35-88.
- o rope do burn pareia vizinhos (burn-nn src/modules/rope_encoding.rs:185-188)
  e calcula as posições no float do backend (:249-251); o burn-lm permuta
  `wq` e `wk` para caber (import.rs:94-127). em f16 o ângulo perde precisão
  com a posição. troco pelo rope do Hugging Face com tabelas f32 feitas na
  CPU, como o candle, e some a permutação.
- quanto é: umas 600 linhas de produção com o `<B>` de volta, o carregador
  e o rope. não é reescrita, a arquitetura fica. o burn-lm foi arquivado
  (README.md:3-4): o código do modelo passa a ser nosso.

(c) a GPU:

- o burn 0.21 usa o cubecl-wgpu 0.10, que puxa o wgpu 29 (cubecl-wgpu
  Cargo.toml:183-195); o plev está no 28 (`Cargo.lock` da raiz). o burn
  aceita um device de fora, `init_device(WgpuSetup { instance, adapter,
  device, queue })` (cubecl-wgpu src/runtime.rs:210-232), mas só do mesmo
  wgpu: dividir o device do renderer pede o plev no 29.
- o candle fala com o Metal pelo objc2-metal 0.3 (candle-core
  Cargo.toml:45-50, :227) e cria o próprio device e fila
  (src/metal_backend/mod.rs:2025-2027), sem construtor para um device de
  fora. o wgpu-hal 28 usa o crate `metal` 0.33 (Cargo.toml:386-388); o 29
  passou ao objc2-metal 0.3.2 (Cargo.toml:409-410). dividir buffer seria
  interop `unsafe` pelo hal, mesma GPU, filas separadas.
- para o item 4, os dois apontam para o wgpu 29; só o burn tem porta
  oficial para o mesmo device.

## fase 1: a medição

metodologia:

- máquina: Apple M4 (4 núcleos P e 6 E, GPU de 10 núcleos, Metal 4),
  16 GB, macOS 26.6.2, rustc 1.98.1. o Mac não estava ocioso (Finder e
  outros processos, load average perto de 5).
- perfil: release do workspace `rust/llmrun` (lto thin, um codegen unit),
  um binário `llm-bench` por implementação, com CPU e Metal dentro.
- modelo: TinyLlama 1.1B Chat v1.0; bf16 no arquivo, f16 no Metal, f32 na
  CPU. geração gulosa.
- prompt: "Write a short story about a lighthouse keeper who finds a
  message in a bottle." no template de chat, 37 tokens; 128 tokens novos
  por rodada, o eos não para a rodada.
- protocolo: um processo de aquecimento, descartado (dele sai a linha "a
  frio"); depois três processos, um por vez, cada um com cinco rodadas; a
  tabela traz a mediana das três medianas. antes de cada processo, pelo
  menos 40% de memória livre (`memory_pressure`); nunca compilar e medir
  ao mesmo tempo. a CPU é referência: um processo só.
- definições: carga é do arquivo ao modelo pronto no device (config,
  mmap, conversão de dtype, upload, sync), sem o tokenizer (0,03 s). ttft
  é da chamada ao primeiro token recebido no canal, com o prefill dos 37
  tokens. decode é 127 tokens pelo tempo entre o primeiro e o último.
  pico de RSS é o `ru_maxrss`; pico de footprint é o
  `ri_lifetime_max_phys_footprint`, que conta a memória do Metal.
- comando, da raiz do repositório:

```sh
export CARGO_TARGET_DIR=target/agents/one/llm CARGO_BUILD_JOBS=4
cargo build --offline --release --manifest-path rust/llmrun/Cargo.toml \
  --features candle,metal --bin llm-bench      # ou burn,metal
$CARGO_TARGET_DIR/release/llm-bench --impl candle \
  --model ~/.cache/plev/models/tinyllama-1.1b-chat-v1.0 --device metal
```

| medida | candle, Metal f16 | burn, Metal f16 | candle, CPU f32 | burn, CPU f32 |
| --- | --- | --- | --- | --- |
| carga | 4,33 s | 3,49 s | 3,29 s | 6,14 s |
| ttft, 1ª rodada do processo | 90 ms | 911 ms | 990 ms | 1489 ms |
| ttft, mediana | 52 ms | 60 ms | 409 ms | 922 ms |
| decode, mediana | 37,1 tok/s | 26,2 tok/s | 6,7 tok/s | 1,9 tok/s |
| pico de RSS | 3264 MB | 2779 MB | 4626 MB | 5212 MB |
| pico de footprint | 5182 MB | 4325 MB | 4753 MB | 5644 MB |
| binário (só esta) | 6,8 MB | 33,0 MB | o mesmo | o mesmo |
| `wasm32-unknown-unknown`, features de CPU | passa (1) | passa (1) | | |

a frio (primeiro processo depois de compilar, página e shader frios):
candle no Metal carregou em 9,0 s e deu o primeiro token em 5,2 s. o burn
no Metal, com o cache de autotune vazio, carregou em 4,6 s, deu o
primeiro token em 10,8 s e decodificou a primeira rodada a 10,7 tok/s. o
burn grava esse cache em `target/autotune` do `Cargo.toml` mais próximo
do diretório de trabalho (ou no cache do usuário, fora de um projeto); a
partir do segundo processo, a primeira rodada cai para perto de 1 s, que
é a compilação dos kernels.

o layout do `Linear` no burn: no Metal, o peso em [out, in] com a vista
transposta (`Col`) decodifica a 26,3 tok/s e carrega sem copiar; o peso
transposto na carga (`Row`) decodifica a 24,4 tok/s e carrega em 7,8 s.
na CPU é o contrário: `Col` fica em 0,3 tok/s, `Row` em 1,8. o padrão é
`Col` no Metal e `Row` na CPU (`llm::burn::layout_for`).

(1) o `cargo check --offline --target wasm32-unknown-unknown
--no-default-features --features candle` (ou `burn`) falha de primeira no
`getrandom`, que não tem backend padrão nesse alvo: o 0.3, sob o `rand`
do candle, e o 0.4, sob o do burn. com a feature `wasm_js` dos dois
(declarada só para wasm32 no manifesto) e
`RUSTFLAGS='--cfg getrandom_backend="wasm_js"'`, os dois passam. o
tokenizer não passa: a feature `text` puxa o `onig_sys`, em C, que não
acha o `stdlib.h` no alvo wasm; o caminho do `tokenizers` para wasm
(`unstable_wasm`) pede o `fancy-regex` 0.14, que não está no cache. no
navegador só o burn tem GPU: a feature `webgpu` (burn-0.21.0
Cargo.toml:251-255); o candle não tem backend wgpu (candle-core
Cargo.toml, features `cuda`, `metal`, `mkl`, `accelerate`).

os 32 primeiros tokens, idênticos nas quatro colunas e em todas as
rodadas: "In the early hours of a crisp autumn morning, a
lighthouse keeper named Jack woke up to the sound of the waves crashing
against" (ids `797, 278, 4688, 6199, 310, 263, 2181, 11936, 1120, 1227,
7250, 29892, 263, 301, 18919, 1709, 1589, 11356, 4257, 5457, 281, 6946,
701, 304, 278, 6047, 310, 278, 20037, 8095, 292, 2750`).

## fase 1: handoff

commits na `task/78a-inference`: `6e2fce00` a sonda; `d095aeb5` o
workspace e a interface; `a60b00c4` o candle; `2f91d01b` o burn;
`094d53da` o llm-bench e o layout do burn; `94109cd3` a medição do
candle; `f6ff13fb` a do burn; `0dec09a9` o porquê do layout; `a33b74a7`
o wasm; `ddd0928c` o ADR; `71993e9c` e `ca7cb9eb` ajustes de feature nos
testes e no bench; este handoff.

a sonda: o candle lê o checkpoint como vem do Hugging Face; o burn não
tem llama, e o do burn-lm (arquivado) virou código nosso, umas 300
linhas, com carregador pelo burn-store e o rope do Hugging Face. na GPU,
o burn vai pelo wgpu 29 e aceita um device de fora; o candle usa o Metal
com device e fila próprios.

a tabela ("fase 1: a medição"): no Metal f16, candle a 37,1 tok/s e burn
a 26,2; primeiro token em 52 e 60 ms, mas 90 ms contra 911 ms na primeira
rodada de cada processo; footprint de 5,2 e 4,3 GB; binário de 6,8 e 33
MB. na CPU f32, 6,7 contra 1,9 tok/s. os 32 primeiros tokens são iguais
nas quatro colunas. os dois compilam para wasm com o getrandom
configurado; o tokenizer não.

o ADR: [`0072-ai-native-single-process.md`](../../adr/Ecosystem/0072-ai-native-single-process/0072-ai-native-single-process.md), `proposed`. recomenda o
candle para o app de referência, com o burn atrás da feature até a
decisão, que é do brenner.

o comando que reproduz está na seção da medição (`llm-bench --impl
candle|burn --device metal|cpu`, de dentro do `CARGO_TARGET_DIR`).

testes: `cargo test --offline --manifest-path rust/llmrun/Cargo.toml
--features candle,burn` com 24 passados (a interface com o modelo falso,
o template, o detokenizador, o rope, o prefill contra o passo); clippy e
fmt verdes com `text`, `candle`, `burn` e `candle,burn,metal`. os dois
testes `#[ignore]` rodaram no Metal em release e responderam "The
capital of France is Paris.".

gate da raiz: verde, 3268 passados, a mesma contagem; o `Cargo.lock` da
raiz igual ao de `origin/main`.

pendências:
- tokenizer no wasm: o `tokenizers` sem C pede o `fancy-regex` 0.14, fora
  do cache.
- o cache de autotune do burn cai em `target/autotune` perto do
  `Cargo.toml`; um app instalado precisa de um lugar próprio, se o burn
  ficar.
- se o burn ficar: a cadeia de licença do llama do burn-lm (código de
  referência da Meta, segundo o NOTICES dele).
- o device do renderer (item 4) pede o plev no wgpu 29 antes de qualquer
  divisão.
- o pico de footprint inclui o arquivo mapeado durante a carga; medir o
  residente depois da carga, se a memória pesar na decisão.
- depois da decisão, tirar a implementação que não ficar.

## fase 1 integrada (orquestrador, 2026-09-26)

validação do orquestrador: diff lido (a interface de streaming, as duas
implementações, o NOTICE do código trazido do burn-lm); os testes do
`rust/llmrun` com as duas features verdes (24, mais 2 com o modelo real,
`#[ignore]`); o gate da raiz verde com os mesmos 3268; o `Cargo.lock` da
raiz igual ao de main; nenhum peso no git.

decisão do brenner (2026-09-26), depois de ler os números: "vamos manter
o burn e candle", e, sobre o burn pedir um wgpu mais novo que o do plev,
"implementa". o ADR `ai-native-single-process` foi aceito com isso
(decisão 16 do programa).

próximo passo, na ordem: o plev do wgpu 28 para o 29 (o do burn 0.21),
sem mudar pixel, para o burn poder usar o device do renderer (item 4);
depois o app de referência (item 1 do critério).

o agente gastou uns 360 mil tokens depois da retomada; o trecho antes da
queda da sessão não ficou contado.

## fase 2: a base de pixel no wgpu 28

antes de qualquer mudança, na `main` em `82b584d6` (wgpu 28.0.0 no
`Cargo.lock`), a captura do desktop contra ele mesmo:

```sh
cargo run --offline -p regress -- capture \
  --out target/determinism/wgpu28
```

82 páginas, 4 temas, 4 escalas: 1312 frames, 1312 `equal` entre os dois
contextos de GPU do processo, em 70 s. os 1312 PNGs ficam em
`target/determinism/wgpu28` (183 MB, fora do git); o sha256 dos PNGs
concatenados em ordem de nome começa por `2b030e6038253651`. essa é a
referência de pixel da subida.

## fase 2: a sonda do wgpu 29

fontes: `wgpu-29.0.4`, `wgpu-types-29.0.4`, `wgpu-core-29.0.4`,
`wgpu-hal-29.0.4` e `wgpu-naga-bridge-29.0.4` no registro do cargo. o
que pegou o plev:

- `Surface::get_current_texture` devolve `CurrentSurfaceTexture`, não
  mais `Result<SurfaceTexture, SurfaceError>` (wgpu src/api/surface.rs:120,
  src/api/surface_texture.rs:55-87). o `suboptimal` saiu da textura e
  virou variante; `Occluded` e `Validation` são novas. no Metal, o
  `Occluded` vem quando a `NSWindow` não está visível
  (wgpu-hal src/metal/surface.rs:144-151); no 28 o frame era desenhado
  assim mesmo. no engine (`window/frame.rs:70`, `window/render.rs:219`)
  sucesso e subótimo desenham como antes, perdido e desatualizado seguem
  o caminho da task-76, e o coberto pula o frame: o
  `WindowEvent::Occluded(false)` já desenha um novo ao descobrir.
- `Instance::new` recebe o `InstanceDescriptor` por valor, e ele perdeu o
  `Default` (wgpu src/api/instance.rs:63; wgpu-types src/instance.rs:64-75,
  `new_without_display_handle`). `gpu/context.rs:295`.
- `PipelineLayoutDescriptor::bind_group_layouts` é
  `&[Option<&BindGroupLayout>]` (wgpu src/api/pipeline_layout.rs:38).
  `gpu/pipelines.rs` (7) e `effects/pipelines.rs` (4).
- `DepthStencilState` tem `Option` na escrita e na comparação
  (wgpu-types src/render.rs:816, :822). só o exemplo `scene3d`.
- o naga do gate: o wgpu-core agora faz o parse com opções e o validador
  vem do `wgpu-naga-bridge` (wgpu-core src/device/resource.rs:2278, :2349;
  wgpu-naga-bridge src/lib.rs:182). `wgpu::naga` segue reexportado
  (src/lib.rs:172) e o teste `wgsl` dá arquivo, linha e coluna como antes.
- o `rust-version` 1.87 liga o `manual_is_multiple_of` do clippy em seis
  lugares; um deles é o `rust/socket/src/framing/base64.rs:54`. o wgpu 28 já pedia
  1.92 (Cargo.toml:14) com o workspace declarando 1.85.

## fase 2: a prova de pixel

```sh
cargo run --offline -p regress -- capture \
  --out target/determinism/wgpu29 --against target/determinism/wgpu28 --no-png
```

no wgpu 29.0.4: 1312 de 1312 `equal` contra a base do wgpu 28 (os mesmos
bytes RGBA, frame a frame), e 1312 `equal` entre os dois contextos do
processo. nenhum `requested`, `silent` ou `resized`.

## fase 2: handoff

commits na `task/78b-wgpu29`: `4a04d454` a base de pixel no wgpu 28;
`8839f3c8` o wgpu 29 e as mudanças de API; `bcf0a205` a sonda e a prova
de pixel; `06df04c8` o lint do wire (com o ok do orquestrador);
`496c6ffa` o device do engine para o burn; `002debd4` o burn no device
do engine e os testes; `9e46ce7a` o teste real diz o dtype; este, o ADR e
o handoff.

a API do wgpu 29 que pegou o plev está na sonda acima: a textura da
superfície virou `CurrentSurfaceTexture` (`window/frame.rs:70`,
`window/render.rs:219`, 12 exemplos do engine, o editor do comps, o
preview do parser e o prime), o
`InstanceDescriptor` sem `Default` e por valor (`gpu/context.rs:295`,
dois testes), os layouts com `Option` (11 no engine, 1 no scene3d) e o
depth do scene3d. o `rust-version` 1.87 pediu o `is_multiple_of` em
seis lugares da raiz (um no wire) e em três do `rust/llmrun`.

a prova de pixel: 1312 de 1312 `equal` contra a base do wgpu 28, no
commit do wgpu 29 e de novo na ponta da branch, depois do device para o
burn. `regress check`: 176 iguais.

o device dividido: o engine dá `instance()` e `adapter()` por acréscimo,
e `RenderConfig::compute` (`None` por padrão) pede os limites e as
features do adaptador (`gpu/config.rs:44`, `gpu/context.rs:309`). no
`rust/llmrun`, a feature `plev` põe o engine como dependência de caminho
e `llm::plev` entrega os quatro handles ao `init_device`. o
`tests/plev.rs` roda sem janela: matmul no device do engine igual ao da
CPU, e depois o mesmo device desenha um frame igual byte a byte ao de um
renderer sem burn. o teste ignorado com o TinyLlama respondeu "The
capital of France is Paris." em f16 no device do engine, em 7,6 s, com
4,0 GB de pico de footprint. o que isso não mostra está no ADR, seção
"the shared device".

o que compila: macOS (gate), wasm (gate, raiz e urnaui), android
(`aarch64-linux-android`, com o clang do NDK 27.2 do SDK apontado por
variável, porque o NDK não está no PATH) e iOS (`aarch64-apple-ios-sim`).

gate verde com as pernas urnaui e wasm: 3272 passados (3186 na raiz e 86
no urnaui), eram 3268; os 4 a mais são os testes novos do engine. o
`rust/llmrun` com `candle,burn,plev`: 27 passados e 3 ignorados; clippy
verde também com `burn`, `candle,burn,metal` e `burn,plev,metal`.

pendências:
- o `rust-conventions.md` e o `clippy.toml` ainda dizem 1.85, e o
  `AGENTS.md:17` diz wgpu 28 (o orquestrador corrige na integração).
- o tensor ainda não chega ao frame sem voltar para a CPU (item 4).
- burn com `metal` no device do engine pede os shaders passthrough, que
  são experimentais no wgpu 29; hoje o `llm::plev::device` recusa.
- depois de uma perda de device, quem usa o burn tem que registrar o
  device novo; não há teste disso.
- avisos antigos só no android e no iOS: `event_loop_proxy` sem uso
  (`engine/src/window/mod.rs:57`) e `window` sem uso
  (`showcase/src/app.rs:408`).

## fase 2 integrada (orquestrador, 2026-09-26)

validação do orquestrador:
- diff lido. as mudanças de API do wgpu 29 são mecânicas. o frame da
  janela coberta (`Occluded`) é pulado, e o `Occluded(false)` que já
  existia redesenha.
- o `RenderConfig::compute` vem desligado; só o `rust/llmrun` com a
  feature `plev` liga. a linha do wire é só o lint, sem mudar o formato.
- gate verde com 3272 (eram 3268; os 4 a mais são do engine).
- `regress check`: 176 iguais.
- a captura refeita pelo orquestrador contra a base do wgpu 28: todo
  frame igual.
- `rust/llmrun` com `candle,burn,plev`: 27 passados e 3 ignorados (os que
  pedem o modelo).

o que isso fecha e o que não fecha: o pedido do brenner (decisão 16),
com o plev no wgpu 29 e o burn na GPU do renderer. do item 4 do escopo,
falta a metade que mostra o resultado do burn na tela sem voltar pela
CPU. próximo passo: o app de referência (item 1 do critério).

o agente gastou uns 495 mil tokens (uns 185 mil até a parada no wire,
uns 310 mil depois).

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0025: urnaui: decode media frames in place with ffmpeg's subfile protocol](../../adr/Importing/0025-urnaui-media-frames-in-place-via-ffmpeg-subfile/0025-urnaui-media-frames-in-place-via-ffmpeg-subfile.md)
- [ADR 0026: background results wake the event loop; child pipes drain from the start](../../adr/Rendering/0026-worker-results-wake-the-event-loop/0026-worker-results-wake-the-event-loop.md)
- [ADR 0072: local llm inference in process: candle or burn](../../adr/Ecosystem/0072-ai-native-single-process/0072-ai-native-single-process.md)
- [ADR 0074: a renderer upgrade is proven by a full capture](../../adr/Rendering/0074-a-renderer-upgrade-is-proven-by-a-full-capture/0074-a-renderer-upgrade-is-proven-by-a-full-capture.md)
- [Generative runtime session: what worked and what bit us](../../adr/Synthesis/0070-generative-runtime-results/generative-runtime.md)
- [task-66: headless as a first-class citizen](../task-66-headless-first-class/task-66-headless-first-class.md)
- [task-73: incremental dependency system](../task-73-incremental-dependency-system/task-73-incremental-dependency-system.md)
- [task-76: platform integration](../task-76-platform-integration/task-76-platform-integration.md)
- [candle against burn for local LLM inference in process, TinyLlama 1.1B](../../Research/Exploring/local-inference-candle-vs-burn.md)
- [plev: thesis of the generative interaction runtime](../../Research/Synthesis/generative-runtime-thesis.md)
- [program: from GPU UI engine to generative interaction runtime](../../Research/Synthesis/generative-runtime-program.md)
- [autonomous cycle journal](../../Research/Workflows/autonomous-cycle-journal.md)
<!-- /related -->
