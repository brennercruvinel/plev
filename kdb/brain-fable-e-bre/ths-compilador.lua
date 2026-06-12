return {
  id  = "ths-compilador",
  typ = "thesis",
  sts = "provocacao-viva",
  dom = "architecture",
  dat = "2026-06-11",
  ttl = "framework e compilador, nao runtime: o colapso das 4 camadas",
  lnk = { "vis-unificada", "plev", "cam-1", "cam-2", "cam-3", "cam-4" },
  txt = [[
provocacao do brenner para refletir o projeto inteiro, inclusive nos
workflows (preservada quase verbatim; e norte de pesquisa, nao regra
cravada):

framework nao e um runtime, e um compilador. as 4 camadas colapsam num
pipeline de lowering progressivo, estilo MLIR: tokens de design ->
componentes -> statecharts -> grafo de dataflow -> comandos de pintura ->
backend. cada camada vira um dialeto do mesmo IR, nao uma biblioteca
empilhada sobre outra. o que sobra em runtime e quase nada: event loop,
tabelas de transicao, tracker de regioes sujas, um blitter.

o insight von neumann: o gargalo de ui nunca foi computacao, e movimento
de dados. renderizacao e memory-bandwidth-bound: pointer chasing na scene
graph, cache miss atras de cache miss, round-trip cpu-gpu. o design
inteiro obedece duas leis: (1) nao busque o que pode ser pre-computado,
mova tudo que der do runtime pro compile time; (2) nao mova o que nao
mudou: renderizacao output-sensitive, custo proporcional aos pixels que
mudaram. o minimo teorico de bytes movidos por frame e o delta, e o
framework ideal atinge esse minimo.

comportamento (camada 3): statecharts de harel compilados a tabelas de
transicao. o zag.js provou que todo componente e uma maquina de estados;
tira-se isso do runtime javascript e poe-se em compile time. a tabela cabe
em L1 no desktop e em SRAM no mcu. o bonus que ninguem explora: a arvore
de acessibilidade e derivada do MESMO statechart, outra lowering do mesmo
IR, nao uma camada paralela mantida a mao.

estado e reatividade: grafo de dependencias resolvido estaticamente (a
ideia do solid, sem o runtime). dados em layout SoA/ECS, buffers
contiguos, cache-line aligned, residentes na gpu quando ela existe. um
set() deixa de ser "recompose, diff, reconcile" e vira: escreve 8 bytes
no offset X, marca regiao suja Y.

render: dois backends do mesmo IR. na gpu grande, rasterizacao por compute
shader (direcao vello/makepad/gpui), cena retida na vram, cpu so envia
deltas. no mcu, software renderer linha a linha (o truque do slint: ram
minima = uma linha de pixels) com DMA2D quando o silicio oferece, que
alias ja e um motorzinho de dataflow de funcao fixa: o mundo embarcado
escapou de von neumann antes de todo mundo.

design system (camada 4): dados puros. tabelas de constantes foldadas em
compile time; theming dinamico vira troca de uniform buffer; shaders
pre-compilados em build (licao do impeller).

linguagem: o frontend e uma DSL total: terminante, declarativa,
constraint-based, analisavel estaticamente, na linha do .slint mas com
statecharts e dataflow como cidadaos de primeira classe. totalidade
destrava avaliacao parcial agressiva: o compilador prova "essa subarvore
nunca muda, pre-rasteriza". compilador e runtime minimo: rust hoje, sem
hesitar (no_std, wasm, sem gc, ownership mapeia posse de fatias de
estado). zig e a tentacao seria (comptime = partial evaluation como
feature), mas pre-1.0 pesa. a resposta profunda: a linguagem verdadeira e
o IR; frontends sao plugaveis, e o dev e dono do codigo gerado (licao do
shadcn). o narrate (crates/narrate_macro, 99 testes) e o candidato local a
frontend.

o meta-padrao para mineracao: cada vencedor do catalogo descobriu um
fragmento. svelte/solid: mover pro compile time, matar o vdom. zag:
comportamento e statechart. slint: dsl -> codigo de maquina, <300KiB,
render por linha. lvgl: dirty regions, partial buffer. impeller: shaders
em build time. makepad/gpui/vello: gpu-first. shadcn: componente e dado
copiavel, nao dependencia. qwik: resumability (nao re-execute o que o
build ja computou). o framework descrito e a intersecao desses fragmentos
levada ao limite: ninguem juntou tudo ainda, e e exatamente nesse vao que
o estudo esta cavando.
]],
}
