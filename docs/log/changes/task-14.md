---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2021-09-22
domain: changelog
---

# changelog, task-14: plev_narrate! DSL verbal

## 2021-09-22

### sessão 1, implementação completa (fases 1-4)

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

### sessão 2, rebase e finalização

- rebase sobre master atual (pós task-13)
- todos os 197 testes do workspace passando (131 core + 66 narrate)
- zero warnings
- task movida para checked, deploy report criado
