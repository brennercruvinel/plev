---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2021-10-06
domain: changelog
---

# changelog, task-15: pipeline DSL -> builder -> compositor

## sessão 1 (2021-10-06)

### análise
- codegen usa `::plev_narrate::builder::*` com construtores no-arg (`text()`, `button()`)
- `show "Hello"` gera `.child("Hello")`, precisa `IntoView for &str/String`
- DSL gera `.gap(4)` (int literal), real builder aceita f32, precisa trait numérico
- stubs aceitam qualquer tipo via `<V>`, real builder precisa métodos tipados
- método `centered()` no DSL é alias de `center()`
