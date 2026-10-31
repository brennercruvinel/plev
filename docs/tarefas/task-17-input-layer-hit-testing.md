---
id: task-17
title: "task-17: input <-> layer hit-testing"
project: plev
type: tarefa
status: concluido
date: 2021-11-08
date_basis: cabecalho
topics: [input, renderizacao]
relations: [task-07, task-09]
source: [docs/log/steps/task-17.md, docs/log/changes/task-17.md]
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# task-17: input <-> layer hit-testing

Registra o hit-testing respeitando a ordem de z dos layers: as regiões são percorridas de trás para frente, e layers invisíveis ou com opacidade zero não recebem toques. O `InputState` recebe o layer atual por `set_current_layer()`, e o compositor não mudou. Os seis testes novos cobrem layers invisíveis, opacidade zero, a sobreposição e o reset por frame, e a suíte chegou a 216 testes passando.

## objetivo
hit-testing respeitar z_order dos layers.

## dependências
- task-07 (layer system)
- task-09 (input system)

## checklist
- [x] hit-test reverso por layer (z_order decrescente), natural via registration order + reverse iteration
- [x] layers invisíveis ignorados no hit-test
- [x] layers com opacity = 0 ignorados no hit-test
- [x] inputstate recebe informação de layer via set_current_layer()
- [x] testes: click em element de layer superior acerta apenas esse layer
- [x] testes: click sem element no layer superior passa para inferior
- [x] testes: layer invisível não recebe hits
- [x] 216 testes passando

## arquivos modificados
- `src/input/mod.rs` (hitregion fields, inputstate layer state, filtering, 6 tests)

## Registro de entrega

### sessão 1 (2021-11-03)

#### análise
- hitregion é flat vec, reverse iteration = last registered first
- compositor layers sorted by z_order ascending, natural registration order
- não precisa re-sort: se regions registradas na ordem dos layers, reverse iteration respeita z-order
- só precisa: layer visibility/opacity filtering no hit_test

#### implementado
- add `layer_visible: bool` + `layer_opacity: f32` fields to hitregion
- add `current_layer_visible` + `current_layer_opacity` to inputstate (defaults: true, 1.0)
- add `set_current_layer(visible, opacity)` to inputstate for tagging regions
- `register_hit_region` automatically tags with current layer state
- `hit_test()` filters: skip regions where `!layer_visible || layer_opacity <= 0.0`
- `hit_test_focusable()` same filtering
- `begin_frame()` resets layer state to defaults
- 6 new tests: invisible layer skip, zero-opacity skip, visible layer on top, focusable skip, defaults, begin_frame reset

#### resultado
- 216 testes passando (150 plev + 12 integration + 54 macro)
- zero mudanças em compositor.rs (layer info pushed via inputstate API)
- backward compatible: sem set_current_layer, regions são visible/opacity=1.0
