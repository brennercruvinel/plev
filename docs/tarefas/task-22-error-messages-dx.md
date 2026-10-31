---
id: task-22
title: "task-22: error messages & DX"
project: plev
type: tarefa
status: concluido
date: 2021-12-11
date_basis: cabecalho
topics: [dsl, processo]
relations: [task-14, task-05]
source: [docs/log/steps/task-22.md, docs/log/changes/task-22.md]
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# task-22: error messages & DX

Registra a melhoria das mensagens de erro do `plev_narrate!`: sugestões "did you mean?" para typos em elementos, modifiers, eventos e keywords de bloco, detecção de sintaxe estilo JS (`onclick` vira `on click`) e de modifier usado como elemento. Os testes novos ficaram em `parse/mod.rs` e `parse/keywords.rs`, caminhos que não existem mais no repositório; trybuild e os avisos do builder para combinações inválidas ficaram de fora por escopo. Importa porque o erro passa a sugerir a correção em vez de apenas rejeitar a entrada.

## objetivo
melhorar mensagens de erro e developer experience.

## dependencias
- task-14 (DSL)
- task-05 (builder)

## checklist
- [x] "did you mean?" suggestions para typos em keywords/modifiers
- [ ] trybuild tests para mensagens de erro legiveis, nao feito
- [ ] builder: warnings para combinacoes invalidas, nao feito

## o que foi feito
- levenshtein distance engine em `parse/keywords.rs`
- sugestoes para typos em: elementos, modifiers, eventos, block keywords
- deteccao de js-style (`onclick` -> use `on click`)
- deteccao de modifier usado como elemento (`bg` -> "is a modifier, not an element")
- mensagens context-specific para missing values
- 33 novos testes de error messages (21 em mod.rs + 16 em keywords.rs)

## arquivos modificados
- `crates/plev_narrate_macro/src/parse/keywords.rs` (levenshtein, suggest_similar, const arrays, 16 testes)
- `crates/plev_narrate_macro/src/parse/element.rs` (3-tier error messages)
- `crates/plev_narrate_macro/src/parse/modifier.rs` (typo detection, context-specific messages)
- `crates/plev_narrate_macro/src/parse/block_item.rs` (event typos, block keyword typos)
- `crates/plev_narrate_macro/src/parse/mod.rs` (21 novos testes de error cases)

## o que nao foi feito e por que
- trybuild: requer adicionar crate trybuild como dev-dependency e criar arquivos .rs de teste com expected stderr. decisao de pular para nao aumentar escopo.
- builder warnings: requer mudar builder.rs para emitir warnings em runtime para combinacoes como text().col() ou div().font_size(). nao foi implementado.

## Registro de entrega

### sessao 1 (2021-12-07)

#### implementado
- levenshtein distance engine em `parse/keywords.rs`
- `suggest_similar()` com threshold: distance <= 1 para palavras curtas (<=3 chars), <= 2 para longas
- 4 const arrays: element_names, modifier_names, event_names, block_keywords
- element errors: 3 tiers (typo -> modifier-as-element -> fallback com lista completa)
- modifier errors: typo detection + context-specific missing value messages
- event errors: typo detection + js-style detection (`onclick` -> "use `on click`")
- block keyword errors: typo detection
- 33 novos testes (21 em mod.rs, 16 em keywords.rs)

#### nao feito
- trybuild tests (requer crate adicional + arquivos .rs de expected stderr)
- builder warnings para combinacoes invalidas (requer mudancas em builder.rs)
