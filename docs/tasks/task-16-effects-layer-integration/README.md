---
id: task-16
title: "task-16: effects <-> layer integration"
project: plev
type: task
status: done
date: 2021-10-24
date_basis: header
topics: [effects, rendering]
relations: [task-07, task-08]
source: [docs/log/steps/task-16.md, docs/log/changes/task-16.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-16: effects <-> layer integration

Records the plan to integrate blur and shadow into the layers, with effects applied through the API and processed between offscreen rendering and composition. The body contains only the analysis of session 1 (2021-10-20), and the checklist remains open; the code has `set_layer_effects` and `LayerEffect`, and the phase map marks the task as completed. Only the code and the map support the `done` in the header, because the body does not record the closing.

## objetivo
effects (blur, shadow) aplicáveis diretamente a layers via API.

## dependências
- task-07 (layer system)
- task-08 (effects system)

## checklist
- [ ] adicionar `effects: Vec<LayerEffect>` ao struct `Layer` em compositor.rs
- [ ] API: `compositor.set_layer_effects(layer_id, vec![LayerEffect::Blur { radius: 8.0 }])`
- [ ] no render loop: após renderizar layer para offscreen, aplicar effectprocessor antes do composite
- [ ] testes: layer com blur processada antes de composite
- [ ] testes: layer com shadow (silhouette + blur)
- [ ] testes: layer sem effects, pipeline idêntico ao atual (sem regressão)

## arquivos críticos
- `src/compositor.rs`
- `src/effects.rs`
- `src/window.rs`

## Delivery record

### sessão 1 (2021-10-20)

#### análise
- effectprocessor é serviço standalone: apply_blur, apply_shadow, composite_pass
- compositor só faz cpu-side prep (resolve), effects precisam de GPU passes
- texturepool grow-only, keyed por (width, height, format)
- layer tem offscreen texture, effects aplicados entre layer render e composite pass
- integration: add layereffect enum ao layer, aplicar no render loop
