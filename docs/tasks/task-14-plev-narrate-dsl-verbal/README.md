---
id: task-14
title: "task-14: plev_narrate!, verbal DSL"
project: plev
type: task
status: done
date: 2021-09-27
date_basis: header
topics: [dsl, signals]
relations: [task-04, task-05, task-09]
source: [docs/log/steps/task-14.md, docs/log/changes/task-14.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-14: plev_narrate!, verbal DSL

Records the procedural macro `plev_narrate!`, which translates a verbal UI syntax into the builder API at compile time, in a crate of its own. Session 1 delivered parsing, code generation, signal interpolation and tests; session 2 rebased onto master and closed with 197 workspace tests passing. The recorded intent is for the interface to be read as structured prose and to become more precise for generation by LLMs.

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

## Delivery record

### 2021-09-22

#### sessão 1, implementação completa (fases 1-4)

**decisões:**
- `gen` renomeado para `codegen_str` nos testes (keyword reservada no rust 2024)
- sem dependência plev no crate plev_narrate (stubs autônomos até integração com task-05)
- sem bare idents como valores de modifiers (apenas literais e `{expr}`), elimina ambiguidade no parser
- modifiers divididos em flag-only vs value-required (sem "optional value", simplifica parsing)
- single root element enforçado no codegen (múltiplos roots = compile_error)
- campos `span` preservados em todos os ast nodes para error reporting futuro

**implementado:**
- workspace setup: `[workspace]` no root cargo.toml, 2 novos crates
- `plev_narrate_macro`: proc-macro completa com parsing e codegen
  - parsing: elementos (row/col/div/text/button/image/spacer/pascalcase), modifiers (30+ keys), show, on, bind, when/otherwise, each/keyed by
  - codegen: elemento->constructor, modifier->method chain, show->.child() com format interpolation, on->.on_event(closure), when->.child_if/_else, each->.children_each/_keyed
  - 54 unit tests (parse + codegen + interpolation)
- `plev_narrate`: re-export + builder stubs com métodos genéricos
- 12 integration tests end-to-end

#### sessão 2, rebase e finalização

- rebase sobre master atual (pós task-13)
- todos os 197 testes do workspace passando (131 core + 66 narrate)
- zero warnings
- task movida para checked, deploy report criado
