---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-06-12
domain: task-tracking
---

# task-43: monster, codec binario de animacao v0

## objetivo
formato binario proprio para animacao vetorial que seja pequeno, seekable, e toque na mesma engine que desenha a ui. nem json parseado a cada frame (lottie), nem replay O(n) no rewind (swf), nem video que joga fora a natureza vetorial.

## dependencias
- task-31 (vector paths via lyon, fornece a geometria tessellada)
- task-27 (animation tick + easing, o player reusa as curvas)
- compositor da engine (o player baixa ir para scenenode no render)

## contexto
a sub-area mais madura do experimento mon. as opcoes existentes falham cada uma num ponto: lottie e json (1.9 a 321 KB/s, runtime estrangeiro), swf assa uma tag por objeto por frame e nao tem acesso aleatorio, webm perde o vetor e a semantica. precisava de um formato com seek O(1) e delta que so paga pelo que muda.

## o que foi entregue
- crate `monster`, magic `MON0`, frozen v0. ir proprio (`ir.rs`) desacoplado de `engine::SceneNode`, o formato congelado nao persegue o enum interno; o player baixa ir para scenenode no render (`lower.rs`).
- container (`container.rs`): header, tabelas de asset e easing, indice de secao com sha256 por secao, quantizacao na wire (twips para coordenadas, rgba8 para cor) em `quant.rs`.
- dois encoders: mode A authored timeline (`write.rs`) e mode B discover (`discover.rs`, `discover_fit.rs`): snapshots por amostra viram ops, viram segmentos lineares por propriedade, viram keyframes.
- keyframe e snapshot de cena inteira (acesso aleatorio, seek O(1)); interframe e delta descoberto (place, modify, replace, remove + segmentos com easing e duracao). um no que nao muda custa zero bytes, a licao da display list do swf na granularidade de shape.
- optimizer (`optimize.rs`) idempotente: static collapse, reducao de keyframe por RDP, fusao de colineares.
- decoder estrito (`read.rs`, `read_sec.rs`) e player deterministico (`play.rs`, `play_eval.rs`): dirigido por AnimationTick, sem wall clock, avaliacao em janela.
- description track utf-8 opcional por keyframe, a semente de a11y e busca que o flash nunca teve, distinta da arvore de widget do accesskit.

## numeros honestos
- corpus de 5 arquivos: cards 0.36x e explosion 0.74x do tamanho do json lottie (ganha). girl 6.5x, snake 42x, money 53x. o movimento de corpo inteiro a 60fps paga o custo do v0 porque morph = re-tessela cada shape em movimento num asset novo por amostra.
- a alavanca v1 e morph track: guardar a curva, nao as amostras (licao do DefineMorphShape do swf).
- a comparacao e contra o json do lottie, nao contra bytes de swf. "mon bate o swf" e gate de design (swf mediu ~1.7 KB/s), nao head-to-head medido no corpus.
- gate mode B e2e: max deviation 0.0375 px / 0.0035 por canal.
- 34 arquivos .rs, ~7130 LOC, 124 testes (incluindo proptest de round-trip), bench `benches/codec.rs`.

## referencias
- adr [binary-animation-format-with-discovered-deltas](../../../adr/binary-animation-format-with-discovered-deltas.md)
- spec [monster-format-v0](../../../adr/monster-format-v0.md)
- commits a4ad0c0 (rebrand anm->monster), aa802ff (codec v0)

## fora de escopo
- morph track (v1): guardar a curva do path, nao as amostras
- script sidecar (feature `script`, vazia no v0): playback de tween nunca pode depender de script
