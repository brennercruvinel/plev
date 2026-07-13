---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2025-06-12
domain: changelog
---

# task-44 changelog: lot, importer lottie e ponte para .monster

## modelo e leitura
- [x] modelo bodymovin via serde (mdl.rs)
- [x] avaliacao de keyframe com bezier easing (kfr.rs)
- [x] helpers de matriz para transform (gem.rs)

## subset suportado
- [x] shape layers (ty 4), null (ty 3), precomps (ty 0)
- [x] transforms static e keyframed
- [x] shapes gr/sh/el/rc, fills, strokes
- [x] gradient fill/stroke aproximado para solido
- [x] nao suportado pulado com log, nunca panic (masks, mattes, trim, text, images, expressions)

## render direto
- [x] rnd.rs: player que desenha o lottie em TessellatedPath sem converter (validacao)

## conversao
- [x] cnv.rs: amostra o lottie uma vez
- [x] dedup de payloads tessellados por bytes quantizados exatos (shape estatico = 1 asset, zero delta)
- [x] descobre deltas e encoda .monster
- [x] stage WxH no description track

## validacao
- [x] bench convert.rs (lottie json -> .monster)
- [x] examples lot2monsters (cli) e monster_player (toca sem lottie linkado)
