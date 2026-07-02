---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-06-12
domain: task-tracking
---

# task-44: lot, importer lottie e ponte para .monster

## objetivo
importar animacao lottie (bodymovin) sem embarcar o runtime estrangeiro. ler o json uma vez, offline, e converter para .monster, de modo que o playback rode so na engine do plev e nenhum codigo lottie execute em runtime.

## dependencias
- task-43 (monster, o destino da conversao)
- task-31 (vector paths, a geometria tessellada de saida)

## contexto
import por conversao, nunca por embedding. o lottie ja embarcou um motor js inteiro para tocar clipe com script; o plev nao vai por esse caminho. o json morre na porta: depois da conversao, quem toca e o `monster::MonsterPlayer`.

## o que foi entregue
- crate `lot`, le o modelo bodymovin via serde (`mdl.rs`). dois caminhos de saida:
  - `rnd.rs`: render direto para TessellatedPath (player que desenha o lottie sem converter)
  - `cnv.rs`: conversao para .monster
- avaliacao de keyframe (`kfr.rs`) e helpers de matriz (`gem.rs`).
- subset suportado, declarado e honesto: shape layers (ty 4), null layers (ty 3), precomps (ty 0), transforms static e keyframed com bezier easing, shapes gr/sh/el/rc, fills, strokes, gradient fill/stroke aproximado para solido.
- nao suportado (masks, mattes, trim paths, text, images, expressions): pulado com log, nunca panic.
- na conversao: amostra o lottie uma vez, dedup de payloads tessellados na tabela de asset por bytes quantizados exatos (um shape estatico e um asset e zero delta bytes), descobre deltas, encoda .monster. o stage WxH viaja no description track.

## numeros honestos
- 6 arquivos .rs, ~1039 LOC, bench `benches/convert.rs`.
- validado pelos examples `lot2monsters` (conversor cli) e `monster_player` (toca o .monster sem lottie linkado).
- o valor do importer depende do subset: ele cobre o caminho de shape/transform/precomp, nao a superficie inteira do lottie. masks e mattes ficam de fora por design.

## referencias
- adr [import-foreign-formats-by-conversion-not-embedding](../../../adr/import-foreign-formats-by-conversion-not-embedding.md)
- commits a044613 (lot cnv), c5535c8 (ponte lot->cnv, payload Path)

## fora de escopo
- paridade com a superficie completa do lottie (masks, mattes, trim, text, images, expressions)
- playback do lottie ao vivo como produto: `rnd` existe para validacao, o caminho de produto e a conversao
