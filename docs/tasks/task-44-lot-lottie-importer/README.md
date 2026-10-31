---
id: task-44
title: "task-44: lot, lottie importer and bridge to .monster"
project: plev
type: task
status: done
date: 2025-06-17
date_basis: reconstructed
recorded: 2026-06-12
topics: [formats, animation]
relations: [adr-0011, task-31, task-43]
evidence: [a044613, c5535c8]
source: [docs/log/steps/task-44-lot-lottie-importer.md, docs/log/changes/task-44.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-44: lot, lottie importer and bridge to .monster

Records the task for `lot`, the Lottie importer that reads the JSON once, offline, and converts it to `.monster`, without shipping a foreign runtime. The result is a declared subset (shapes, transforms, precomps, fills and strokes), in which masks, mattes, text and expressions are skipped with a log entry and never cause a panic. It was validated by the examples `lot2monsters` and `monster_player`, which plays the `.monster` without Lottie linked.

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
- adr [import-foreign-formats-by-conversion-not-embedding](../../adr/importing/0011-import-foreign-formats-by-conversion-not-embedding.md)
- commits a044613 (lot cnv), c5535c8 (ponte lot->cnv, payload Path)

## fora de escopo
- paridade com a superficie completa do lottie (masks, mattes, trim, text, images, expressions)
- playback do lottie ao vivo como produto: `rnd` existe para validacao, o caminho de produto e a conversao

## Delivery record

### modelo e leitura
- [x] modelo bodymovin via serde (mdl.rs)
- [x] avaliacao de keyframe com bezier easing (kfr.rs)
- [x] helpers de matriz para transform (gem.rs)

### subset suportado
- [x] shape layers (ty 4), null (ty 3), precomps (ty 0)
- [x] transforms static e keyframed
- [x] shapes gr/sh/el/rc, fills, strokes
- [x] gradient fill/stroke aproximado para solido
- [x] nao suportado pulado com log, nunca panic (masks, mattes, trim, text, images, expressions)

### render direto
- [x] rnd.rs: player que desenha o lottie em TessellatedPath sem converter (validacao)

### conversao
- [x] cnv.rs: amostra o lottie uma vez
- [x] dedup de payloads tessellados por bytes quantizados exatos (shape estatico = 1 asset, zero delta)
- [x] descobre deltas e encoda .monster
- [x] stage WxH no description track

### validacao
- [x] bench convert.rs (lottie json -> .monster)
- [x] examples lot2monsters (cli) e monster_player (toca sem lottie linkado)
