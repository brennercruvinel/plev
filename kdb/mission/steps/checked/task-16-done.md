---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: task-tracking
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
