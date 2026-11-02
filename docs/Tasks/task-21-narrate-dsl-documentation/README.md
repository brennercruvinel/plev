---
id: task-21
title: "task-21: documentation of the narrate DSL"
project: plev
type: task
status: done
date: 2021-12-04
date_basis: reconstructed
recorded: 2026-03-13
topics: [dsl, editorial]
relations: [task-14, task-15]
source: [docs/log/steps/task-21.md, docs/log/changes/task-21.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-21: documentation of the narrate DSL

Records the documentation of the `plev_narrate!` syntax: the EBNF grammar derived from the real parser, the table of the 30 modifiers, examples by category and the comparison with the builder API. The result was the file `docs/narrate-syntax.md`, with 967 lines, but that path does not exist in the current repository; the comparison with JSX was left undone. It matters because the grammar was taken from the parser's code, and not written from memory.

## objetivo
documentacao completa da sintaxe plev_narrate!

## dependencias
- task-14 (DSL implementada)
- task-15 (bridge real, para exemplos end-to-end)

## checklist
- [x] `docs/narrate-syntax.md` com gramatica EBNF formal
- [x] tabela completa de modifiers (30 modifiers, 6 flags + 24 value)
- [x] exemplos por categoria (layout, style, events, signals, conditionals, loops)
- [x] comparacao: plev_narrate! vs builder API
- [ ] comparacao com JSX, nao feito

## arquivos criados
- `docs/narrate-syntax.md` (967 linhas)

## Delivery record

### sessao 1 (2021-11-30)

#### criado
- `docs/narrate-syntax.md` (967 linhas)
- EBNF grammar derivado do parser real (parse/mod.rs, element.rs, modifier.rs, block_item.rs, value.rs, keywords.rs)
- 7 elementos built-in + custom pascalcase documentados
- 30 modifiers (6 flag + 24 value) com tabelas completas
- 14 named colors + 8 rounded presets
- 6 event handlers com tipos
- control flow: when/otherwise, each/keyed, bind
- string interpolation rules
- codegen reference (mapeamento DSL -> builder API)
- comparacao DSL vs builder API (4 exemplos lado a lado)

#### nao feito
- comparacao com JSX (item do checklist original)
