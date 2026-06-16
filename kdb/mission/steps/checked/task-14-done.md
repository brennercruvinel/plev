---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: task-tracking
---

# task-14: plev_narrate!, DSL verbal

## objetivo
criar a macro `plev_narrate!` que permite descrever UI em sintaxe verbal "prosa-like". a macro des-sugara para a API declarativa (task-05) em compile-time.

## contexto
a DSL verbal é o diferencial conceitual do plev, nenhum framework existente tem isso. a ideia é que código UI leia como prosa estruturada, reduzindo cognitive load para humanos e sendo mais preciso para geração por llms. a macro transforma sintaxe verbal em chamadas à builder API.

## dependências
- task-05 (API declarativa, a DSL des-sugara para builders) concluída
- task-04 (signal system, DSL precisa suportar bindings reativos) concluída
- task-09 (input system, DSL precisa suportar event handlers) concluída

## checklist de conclusão
- [x] proc-macro `plev_narrate!` em crate separada
- [x] parsing de sintaxe verbal básica (definição de componentes, layout, estilo)
- [x] des-sugaring para builder API existente
- [x] suporte a signals dentro da DSL (via `{expr}` interpolation e `bind`)
- [x] suporte a event handlers dentro da DSL (`on click { ... }`)
- [x] error messages claros quando sintaxe é inválida
- [x] exemplo funcional: 12 testes de integração end-to-end com plev_narrate!
- [x] `cargo build` passa sem warnings
- [x] `cargo test` passa (66 testes: 54 unit + 12 integration)

## status: concluída
