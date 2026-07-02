---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-06-12
domain: changelog
---

# task-43 changelog: monster, codec binario de animacao v0

## spec e ir
- [x] magic MON0, container v0 congelado (header, tabelas de asset/easing, indice de secao sha256)
- [x] ir proprio desacoplado de scenenode (ir.rs), lower para scenenode no render (lower.rs)
- [x] quantizacao na wire: twips para coordenadas, rgba8 para cor (quant.rs)
- [x] payload Path definido (asset_path.rs): cor uniforme, vertices em twips, indices u16

## encoders
- [x] mode A: authored timeline lowering (write.rs)
- [x] mode B: discover deltas de frames amostrados (discover.rs, discover_fit.rs)
- [x] delta ops completas: place, modify, replace, remove + segmentos eased por propriedade

## optimizer
- [x] static collapse (no que nao muda vira asset, zero delta)
- [x] reducao de keyframe por RDP
- [x] fusao de segmentos colineares
- [x] passes idempotentes

## decoder e player
- [x] decoder estrito de volta para ir (read.rs, read_sec.rs)
- [x] player deterministico dirigido por animationtick (play.rs, play_eval.rs)
- [x] seek O(1) em keyframes, eval em janela
- [x] description track utf-8 por keyframe

## validacao
- [x] 124 testes, proptest de round-trip estrutural (encode -> decode identidade)
- [x] gate mode B e2e: max deviation 0.0375 px / 0.0035 por canal
- [x] bench codec.rs vs json/gzip/webm em 4 fixtures
- [x] medido no corpus: cards 0.36x e explosion 0.74x do json (ganha em movimento discreto)
