---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-06-12
domain: changelog
---

# task-45 changelog: parser, transpiler poc (react/gpui para builder)

## parse (tree-sitter)
- [x] react tsx -> arvore crua (tsx.rs)
- [x] sass -> regras (sass.rs), ponte regras->styles (css_map.rs)
- [x] gpui widget -> arvore crua (gpui.rs, gpui_ir.rs)

## resolve
- [x] ir intermediario (ir.rs)
- [x] normaliza react+css para ir e preenche droplist (resolve_react.rs)
- [x] normaliza gpui para ir (resolve_gpui.rs)
- [x] mapeia cor para token de tema hoff

## emit
- [x] emissao deterministica de codigo rust contra engine::builder (emit.rs)
- [x] goldens byte-identical as copias do corpus
- [x] codigo emitido compila e renderiza (example parser_card)

## droplist honesto
- [x] cada construcao nao mapeada vai para o droplist com file:line e motivo
- [x] contagens congeladas em teste (gate de regressao)
- [x] nada dropa em silencio

## numeros
- [x] corpus do dono: 402 propriedades mapeadas, 709 entradas de droplist, zero crash
- [x] 20 testes, bench transpile.rs
- [x] examples transpile (cli) e preview (preview ao vivo)
- [ ] defeito conhecido: body text run ainda nao quebra linha (wrap)
