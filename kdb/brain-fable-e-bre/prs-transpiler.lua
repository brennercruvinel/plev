return {
  id  = "prs-transpiler",
  typ = "poc",
  sts = "fila",
  dom = "parser-transpiler",
  dat = "2026-06-11",
  ttl = "qualquer ui entra, plev sai: o parser universal",
  lnk = { "vis-unificada", "ths-compilador", "shc-absorcao", "ref-study" },
  txt = [[
o que tira o sono do brenner: uma lib que transpoe ui real de um framework
para outro. pegar qualquer ui de alto nivel (react/next, gpui, depois
outras) e emitir plev: do wasm ao ios, performando mais, com fidelidade e
glass. ninguem tem isso. ajudaria milhoes (pessoas e llms) na tarefa de
migracao que hoje e reescrita a mao. entregavel 1 do programa.

por que e plausivel aqui: ja fizemos uma vez A MAO (hoff-research-social
next/sass reproduzido pixel a pixel no plev, medido). as regras que
descobrimos manualmente (computed styles e nao stylesheet, TextStyle
unico, layout content-driven) sao exatamente o que o passe automatico
codifica. o builder do plev (div().flex().col()...) ja tem o formato do
modelo css/flexbox, e taffy E o algoritmo do browser sem o dom.

motor: tree-sitter (gramaticas prontas pra tsx/css/rust) como parser;
topiary como estudo de regras declarativas sobre a arvore; gritql para
transformacoes em massa (o brenner ja usou muito pra react). verificacao
TRIPLA do output: golden source tests, diff estrutural (tecnica
difftastic/weave: comparar arvores, nao texto), e pixel-compare do render
browser vs plev (nossa tecnica de validacao por pixel).

corpus do poc:
1. gpui-component (ja clonado em ref/parsecomponentes/gpui/): rust->rust,
   mede o gap semantico gpui->plev
2. os 4 hoff-research-cards (react/next em ref/parsecomponentes/
   UIvisualREFs/cards/): jsx+css -> plev, pixel-compare contra o browser

regras: o emissor obedece o manual (kdb/how-to/code-against-the-plev-
engine.md): texto medido, layout content-driven, um TextStyle por run. o
parser e um USUARIO das nossas regras. escopo honesto: componentes
apresentacionais primeiro (layout, estilo, texto, imagem, estado simples):
isso ja cobre design systems, que e o mercado. css de cauda longa: cobrir
o subconjunto que a engine fala e REPORTAR o que foi dropado, nunca
silencioso. react geral (hooks/effects/portais) fica explicitamente fora
do poc.

graphify (ref doc-only): grafos de codebase; talvez refatorar a ideia para
visualizar a arvore de elementos transpilada (diagnostico do parser).

nota corpus (2026-06-11, ordem do brenner: usar, ler, enxergar os
padroes): o corpus completo do parser e MAIOR que o poc e fica
registrado para as fases seguintes: (a) hoff-research-briefs (o segundo
app next real dele, ainda nao tocado) junto do social como alvo de
transpilacao de TELAS inteiras, nao so cards; (b) gpui-component traz
alem dos widgets: themes/ + .theme-schema.json (o PADRAO de theming
serializado a minerar para o nosso tema), docs/ e examples/ (a
estrutura de documentacao e galeria), skills/ (como eles instruem
agentes); (c) ref/render-texts (vello/parley/glyphon/swash/floem/slint)
e a fila da tese de texto/render; (d) OpenUSD e a fila do 3D/cenas.
nenhum desses pode ser esquecido: cada um vira consumidor nomeado em
workflow proprio.
]],
}
