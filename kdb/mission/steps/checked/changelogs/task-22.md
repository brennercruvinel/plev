---
project: phi
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-08
domain: changelog
---

# changelog, task-22: error messages & DX

## sessao 1 (2026-03-08)

### implementado
- levenshtein distance engine em `parse/keywords.rs`
- `suggest_similar()` com threshold: distance <= 1 para palavras curtas (<=3 chars), <= 2 para longas
- 4 const arrays: element_names, modifier_names, event_names, block_keywords
- element errors: 3 tiers (typo -> modifier-as-element -> fallback com lista completa)
- modifier errors: typo detection + context-specific missing value messages
- event errors: typo detection + js-style detection (`onclick` -> "use `on click`")
- block keyword errors: typo detection
- 33 novos testes (21 em mod.rs, 16 em keywords.rs)

### nao feito
- trybuild tests (requer crate adicional + arquivos .rs de expected stderr)
- builder warnings para combinacoes invalidas (requer mudancas em builder.rs)
