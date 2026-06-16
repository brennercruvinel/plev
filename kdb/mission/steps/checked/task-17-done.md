---
project: phi
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: task-tracking
---

# task-17: input <-> layer hit-testing, done

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
