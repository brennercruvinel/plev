---
id: task-16
title: "task-16: effects <-> layer integration"
project: plev
type: tarefa
status: concluido
date: 2021-10-24
date_basis: cabecalho
topics: [task-tracking]
relations: []
source: [docs/log/steps/task-16.md, docs/log/changes/task-16.md]
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# task-16: effects <-> layer integration

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

## Registro de entrega

### sessão 1 (2021-10-20)

#### análise
- effectprocessor é serviço standalone: apply_blur, apply_shadow, composite_pass
- compositor só faz cpu-side prep (resolve), effects precisam de GPU passes
- texturepool grow-only, keyed por (width, height, format)
- layer tem offscreen texture, effects aplicados entre layer render e composite pass
- integration: add layereffect enum ao layer, aplicar no render loop
