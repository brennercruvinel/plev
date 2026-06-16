---
project: phi
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-08
domain: changelog
---

# changelog, task-21: documentacao DSL narrate

## sessao 1 (2026-03-08)

### criado
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

### nao feito
- comparacao com JSX (item do checklist original)
