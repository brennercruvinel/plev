---
project: plev
audience: [ai-agents, contributors]
status: ongoing
last-updated: 2026-09-26
domain: generative-runtime
task: task-78
wave: trilha
depends: []
depends-optional: [task-73, task-66]
branch: task/78-ai-native
---

# task-78: aplicações AI-native

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
  loop (`crates/urnaui`, ADRs `urnaui-media-frames-in-place-via-ffmpeg-subfile`
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
pedir), ADR `docs/ADR/ai-native-single-process.md`. `crates/urnaui` só se
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
- `./script/gate` verde (o app, se depender de coisa fora do workspace,
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
