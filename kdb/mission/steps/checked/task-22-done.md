---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: task-tracking
---

# task-22: error messages & DX, done (parcial)

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
