---
project: phi
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-08
domain: changelog
---

# changelog, task-17: input <-> layer hit-testing

## sessão 1 (2026-03-08)

### análise
- hitregion é flat vec, reverse iteration = last registered first
- compositor layers sorted by z_order ascending, natural registration order
- não precisa re-sort: se regions registradas na ordem dos layers, reverse iteration respeita z-order
- só precisa: layer visibility/opacity filtering no hit_test

### implementado
- add `layer_visible: bool` + `layer_opacity: f32` fields to hitregion
- add `current_layer_visible` + `current_layer_opacity` to inputstate (defaults: true, 1.0)
- add `set_current_layer(visible, opacity)` to inputstate for tagging regions
- `register_hit_region` automatically tags with current layer state
- `hit_test()` filters: skip regions where `!layer_visible || layer_opacity <= 0.0`
- `hit_test_focusable()` same filtering
- `begin_frame()` resets layer state to defaults
- 6 new tests: invisible layer skip, zero-opacity skip, visible layer on top, focusable skip, defaults, begin_frame reset

### resultado
- 216 testes passando (150 φ + 12 integration + 54 macro)
- zero mudanças em compositor.rs (layer info pushed via inputstate API)
- backward compatible: sem set_current_layer, regions são visible/opacity=1.0
