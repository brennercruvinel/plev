---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2025-07-16
domain: task-tracking
---

# task-45: parser, transpiler poc (react/gpui para builder)

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
- adr [transpiler-reports-every-unmapped-construct](../../../adr/transpiler-reports-every-unmapped-construct.md)
- commits 5eecb0a (poc do parser), f93a57e (preview ao vivo), aafc091 (rebrand prs->parser)

## fora de escopo
- cobertura de um arquivo de ui real completo (a taxa de drop em input nao curado segue como pergunta de estudo)
- masks/mattes/expressions e qualquer construcao que dependa de runtime js
