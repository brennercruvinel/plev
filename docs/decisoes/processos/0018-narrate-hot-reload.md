---
id: adr-0018
title: "adotamos runtime parser + override map para hot reload de plev_narrate! DSL"
project: plev
type: decisao
status: aceito
date: 2024-02-18
date_basis: cabecalho
topics: [hot-reload, dsl]
relations: []
source: docs/adr/workflows/0018-narrate-hot-reload.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# adotamos runtime parser + override map para hot reload de plev_narrate! DSL

Decide o hot reload do `plev_narrate!` com um parser de runtime escrito à mão e um mapa de sobrescritas: o bloco DSL é reparseado ao ser acessado, sem recompilar. Entre três abordagens (literal pool, reparse completo e checagem manual), escolheu-se o reparse completo, porque o literal pool era complexo e a checagem manual era limitada. O resultado é que cores, tamanhos, layout e texto estático mudam em cerca de 500 ms, enquanto os blocos `on`, `when`, `each` e `bind` não são interpretados.

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
