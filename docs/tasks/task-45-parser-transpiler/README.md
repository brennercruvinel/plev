---
id: task-45
title: "task-45: parser, transpiler poc (react/gpui to builder)"
project: plev
type: task
status: done
date: 2025-07-16
date_basis: header
topics: [importing, formats]
relations: [adr-0012]
source: [docs/log/steps/task-45-parser-transpiler.md, docs/log/changes/task-45.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
evidence: [5eecb0a, f93a57e, aafc091]
---

# task-45: parser, transpiler poc (react/gpui to builder)

Records the transpiler POC task: it reads TSX with SASS and a gpui widget, emits plev builder code and reports, with file and line, everything it cannot represent. On the owner's corpus (40 components in two apps) the result was 402 mapped properties and 709 droplist entries, with no crash. The study question remains open: the coverage of a real `index.tsx`, which will tell whether the parser is a tool or a demonstration.

## objetivo
ler ui de outro framework e cuspir codigo builder do plev, mapeando cor para token de tema e reportando, com arquivo e linha, tudo que nao consegue representar. nada sai em silencio. e a materializacao parcial do grafo de equivalencias do experimento mon.

## dependencias
- engine builder (o alvo da emissao)
- theme tokens hoff (o destino do mapeamento de cor)

## contexto
um transpiler so vale pela taxa de drop. um que cospe droplist gigante em todo input real e um relatorio de incompatibilidade, nao um conversor. por isso o escopo e pequeno e honesto, e cada construcao nao mapeada e reportada, nunca engolida.

## o que foi entregue
- crate `parser`, pipeline parse-resolve-emit via tree-sitter. dois inputs reais:
  - react tsx + sass (`tsx.rs`, `sass.rs`, `css_map.rs`), a card base de pesquisa hoff
  - gpui (`gpui.rs`, `gpui_ir.rs`), um separator rotulado horizontal
- ir intermediario (`ir.rs`), resolvers que normalizam para o ir e preenchem o droplist (`resolve_react.rs`, `resolve_gpui.rs`), emissao deterministica de codigo rust contra `engine::builder` (`emit.rs`).
- mapeia cor para token de tema, obedece os contratos da engine, e reporta cada construcao nao representavel num droplist com file:line. contagens congeladas em golden de teste.

## numeros honestos
- corpus do dono (40 componentes em dois apps): 402 propriedades mapeadas e 709 entradas de droplist, zero crash. a card de teste congela mapped e dropped para travar regressao.
- a pergunta de estudo segue aberta: qual a cobertura num index.tsx de verdade, nao de brinquedo. esse numero e o que diz se o parser e ferramenta ou demo.
- defeito conhecido: o run de body text ainda nao quebra linha (wrap).
- 10 arquivos .rs, ~2242 LOC, 20 testes, examples `transpile` e `preview` (preview ao vivo do output na tela).

## referencias
- adr [transpiler-reports-every-unmapped-construct](../../adr/importing/0012-transpiler-reports-every-unmapped-construct.md)
- commits 5eecb0a (poc do parser), f93a57e (preview ao vivo), aafc091 (rebrand prs->parser)

## fora de escopo
- cobertura de um arquivo de ui real completo (a taxa de drop em input nao curado segue como pergunta de estudo)
- masks/mattes/expressions e qualquer construcao que dependa de runtime js

## Delivery record

### parse (tree-sitter)
- [x] react tsx -> arvore crua (tsx.rs)
- [x] sass -> regras (sass.rs), ponte regras->styles (css_map.rs)
- [x] gpui widget -> arvore crua (gpui.rs, gpui_ir.rs)

### resolve
- [x] ir intermediario (ir.rs)
- [x] normaliza react+css para ir e preenche droplist (resolve_react.rs)
- [x] normaliza gpui para ir (resolve_gpui.rs)
- [x] mapeia cor para token de tema hoff

### emit
- [x] emissao deterministica de codigo rust contra engine::builder (emit.rs)
- [x] goldens byte-identical as copias do corpus
- [x] codigo emitido compila e renderiza (example parser_card)

### droplist honesto
- [x] cada construcao nao mapeada vai para o droplist com file:line e motivo
- [x] contagens congeladas em teste (gate de regressao)
- [x] nada dropa em silencio

### numeros
- [x] corpus do dono: 402 propriedades mapeadas, 709 entradas de droplist, zero crash
- [x] 20 testes, bench transpile.rs
- [x] examples transpile (cli) e preview (preview ao vivo)
- [ ] defeito conhecido: body text run ainda nao quebra linha (wrap)
