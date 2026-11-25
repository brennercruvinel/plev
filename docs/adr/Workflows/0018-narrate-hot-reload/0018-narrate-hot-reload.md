---
id: adr-0018
title: "adotamos runtime parser + override map para hot reload de plev_narrate! DSL"
project: plev
type: adr
status: accepted
date: 2024-02-18
date_basis: reconstructed
recorded: 2026-03-22
topics: [hot-reload, dsl]
relations: [learning-hot-reload]
source: docs/adr/workflows/0018-narrate-hot-reload.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# adotamos runtime parser + override map para hot reload de plev_narrate! DSL

Decides the hot reload of `plev_narrate!` with a hand-written runtime parser and an override map: the DSL block is reparsed when accessed, without recompiling. Among three approaches (literal pool, full reparse and manual check), the full reparse was chosen, because the literal pool was complex and the manual check was limited. The result is that colors, sizes, layout and static text change in about 500 ms, while the `on`, `when`, `each` and `bind` blocks are not interpreted.

brenner@plev.engineer

## contexto

tier 1 (shader hot reload) funciona via file watcher + pipeline recreation. para tier 2 precisavamos de hot reload para plev_narrate! DSL blocks sem recompilacao.

tres abordagens avaliadas:
1. **literal pool** (dioxus pattern): so trocar valores literais, complexo de implementar
2. **full re-parse** (makepad pattern): re-parsear bloco inteiro, produzir element tree
3. **no proc-macro change**: developer adiciona check manual (limitado)

## decisao

full re-parse com override map. runtime parser hand-written em narrate_runtime.rs produz element trees diretamente via builder API. override map global armazena DSL text (nao element) e re-parsa on access.

## consequencias

ganhamos: editar cores, tamanhos, layout, texto estatico e nesting em plev_narrate! blocks sem recompilar. mudancas aparecem no proximo frame (~500ms debounce).

perdemos: on/when/each/bind blocks (rust code) nao sao interpretados. expressoes em modifiers skipped. custom components renderizados como placeholder. preservacao de estado across reload deferred.

## patterns que funcionaram

1. **narrate_resolve() no plev crate (nao plev_narrate)**: evita circular dependency. proc-macro gera `::plev::narrate_resolve(file!(), line!(), || { ... })`.

2. **re-parse on access**: element nao e clone (contem box<dyn fnmut> em eventhandlers). armazenar DSL text e re-parsear e o(microsegundos) para blocos tipicos.

3. **atomic take helpers no parser**: edition 2024 com implicit ref em patterns causa conflito de borrow se usar peek() + advance(). take_ident(), take_str(), take_f32() fazem check + consume atomicamente.

4. **extract_narrate_blocks() com brace depth tracking**: nao precisa de parser rust completo para encontrar macro invocations. simples pattern match + depth counter funciona.

## armadilhas

1. **edition 2024 ref patterns**: `let Some(Token::Ident(ref name)) = ...` falha. usar `let Some(Token::Ident(name)) = ...` (ref e implicito).

2. **file!() retorna path relativo**: watcher reporta paths absolutos, file!() retorna relativo ao crate root. strip_prefix(cargo_manifest_dir) para normalizar.

3. **plev_narrate legado**: codegen referenciava `::plev_narrate::` (nome antigo do crate). corrigido para `::plev_narrate::`.

## learnings

- [hot reload session: what worked and what bit us](../0031-hot-reload-on-by-default-in-gpu-context/hot-reload.md)

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0031: hot reload on by default: shaders and literals in place, code by rebuild](../0031-hot-reload-on-by-default-in-gpu-context/0031-hot-reload-on-by-default-in-gpu-context.md)
- [hot reload session: what worked and what bit us](../0031-hot-reload-on-by-default-in-gpu-context/hot-reload.md)
<!-- /related -->
