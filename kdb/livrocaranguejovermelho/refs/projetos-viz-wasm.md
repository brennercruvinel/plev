---
title: projetos de grafos, charts e visualizacao em wasm (mapa de 50+)
date: 2026-06-25
tags: [pesquisa, refs, viz, wasm, grafos, charts, rust]
dimensao: viz
fontes:
  - github rest api (repos/<owner>/<repo>), captura em 2026-06-25 via gh cli
  - github topics: https://github.com/topics/visualization?l=rust (resolve, 298 repos rust no topico em 2026-06-25)
  - lib.rs/visualization: https://lib.rs/visualization (resolve em browser; retornou http 403 a fetch automatizado, nao usado como fonte de numero)
  - crates.io api (repository field) para resolver redirects de repo
  - docs.rs para confirmar paginas de docs (spot-check)
status_validacao: >
  stars, status (archived), linguagem, ultimo push e licenca de cada repo vem
  da github rest api capturada em 2026-06-25. todo repo da tabela retornou 200
  (link resolve). itens marcados "nao confirmado" so na coluna onde a fonte nao
  trouxe o dado (licenca n/a) ou onde ha divergencia (cosmograph). numeros sao
  da data de captura e mudam; nao foram inventados.
---

# projetos de grafos, charts e visualizacao em wasm

mapa da dimensao viz do corpus (bloco P2 do `04-corpus-pesquisa.md`). cobre 68
projetos: charts e plotting em rust, grafos e network, engines de dados que
alimentam viz, renderers 2d/3d em gpu, frameworks de ui que hospedam viz, a
base de math/geometria/cor, e o cluster js/wasm que serve de contraste para a
lacuna (nao ha equivalente rust+wasm maduro a d3-force ou networkx).

## metodo de validacao

cada link de repo foi resolvido pela github rest api (`repos/<owner>/<repo>`),
nao por scraping de html. a api retorna `archived`, `language`,
`stargazers_count`, `pushed_at` e `license.spdx_id`. a captura e de 2026-06-25;
stars e datas mudam, entao o numero so vale com essa data colada. a coluna docs
aponta para `docs.rs/<crate>` (gerado automatico para todo crate publicado) ou o
site canonico do projeto; foi feito spot-check (ex. `docs.rs/leptos-chartistry`
resolve, versao 0.2.3).

convencao de status (referencia: hoje 2026-06-25, pelo `pushed_at`):

- ativo: ultimo push <= ~4 meses
- baixa atividade: push entre ~4 e 12 meses
- estagnado: push > 12 meses
- arquivado: flag `archived` ou repo declara manutencao

stars abaixo: contagem da github api em 2026-06-25, arredondada so quando o
texto da analise cita; a tabela traz o numero cru.

## redirects e renames detectados (importante para nao citar repo morto)

| nome citado na pesquisa | repo canonico atual (2026-06-25) | observacao |
|---|---|---|
| finos/perspective | perspective-dev/perspective | org migrou de finos para perspective-dev |
| sebcrozet/kiss3d | dimforge/kiss3d | manutencao passou para a dimforge |
| RazrFalcon/resvg | linebender/resvg | resvg entrou no guarda-chuva linebender |
| cosmograph-org/cosmos | cosmosgl/graph | a lib `@cosmograph/cosmos` virou `cosmosgl/graph` |
| tiby312/poloto | tiby312/poloto-project | crate poloto mora no workspace poloto-project |
| bitshifter/glam | bitshifter/glam-rs | o repo da pesquisa (glam) e 404; o real e glam-rs |
| mazznoer/colorgrad | mazznoer/colorgrad-rs | colorgrad sem sufixo e a versao Go; a rust e -rs |

a pesquisa colada repetia varios itens (P11 duplica P1..P3). a deduplicacao foi
feita por repo canonico, nao por nome citado, entao os redirects acima nao
geraram linha dupla.

## 1. charts e plotting (rust)

| projeto | descricao | linguagem | tipo de viz | status | github | docs |
|---|---|---|---|---|---|---|
| plotters | drawing library para data plotting, backends bitmap/svg/canvas-wasm | Rust | charts 2d/3d estaticos | ativo (push 2026-04-13, 4583 stars, MIT) | https://github.com/plotters-rs/plotters | https://docs.rs/plotters |
| charming | wrapper rust sobre apache echarts (gera spec, renderiza via js) | Rust | charts declarativos (echarts) | baixa atividade (2026-01-16, 2558 stars, Apache-2.0) | https://github.com/yuankunzhang/charming | https://docs.rs/charming |
| plotly.rs | binding rust para plotly.js, html/wasm e static export | Rust | charts interativos (plotly) | ativo (2026-06-17, 1430 stars, MIT) | https://github.com/plotly/plotly.rs | https://docs.rs/plotly |
| plotlars | ponte polars + plotly, charts a partir de dataframe | Rust | charts sobre dataframe | ativo (2026-06-01, 657 stars, MIT) | https://github.com/alceal/plotlars | https://docs.rs/plotlars |
| egui_plot | plotting 2d para egui (linhas, barras, scatter) | Rust | charts em immediate-mode | ativo (2026-06-02, 439 stars, Apache-2.0) | https://github.com/emilk/egui_plot | https://docs.rs/egui_plot |
| leptos-chartistry | charting extensivel para leptos, render svg | Rust | charts em ui reativa (wasm) | baixa atividade (2026-01-23, 141 stars, MPL-2.0) | https://github.com/feral-dot-io/leptos-chartistry | https://docs.rs/leptos-chartistry |
| plotters-iced | backend iced para plotters | Rust | charts em ui (iced) | baixa atividade (2025-12-11, 206 stars, MIT) | https://github.com/Joylei/plotters-iced | https://docs.rs/plotters-iced |
| chart-js-rs | conector rust para chart.js via wasm/dominator | Rust | charts (chart.js) | ativo (2026-06-18, 28 stars, Apache-2.0) | https://github.com/Billy-Sheppard/chart-js-rs | https://docs.rs/chart-js-rs |
| charts-rs | charts library pure rust (svg/png) | Rust | charts estaticos | ativo (2026-06-19, 314 stars, Apache-2.0) | https://github.com/vicanso/charts-rs | https://docs.rs/charts-rs |
| poloto | plotting 2d simples, saida svg estilizavel por css | Rust (workspace marcado Jupyter na api) | charts svg | ativo (2026-04-01, 164 stars, MIT) | https://github.com/tiby312/poloto-project | https://docs.rs/poloto |
| textplots | plotting no terminal | Rust | charts ascii/terminal | estagnado (2025-02-20, 284 stars, licenca nao confirmado na api) | https://github.com/loony-bean/textplots-rs | https://docs.rs/textplots |

## 2. grafos e network (rust + js)

| projeto | descricao | linguagem | tipo de viz | status | github | docs |
|---|---|---|---|---|---|---|
| petgraph | estrutura de dados de grafo, padrao de facto em rust | Rust | data structure (sem render) | ativo (2026-04-04, 3940 stars, Apache-2.0) | https://github.com/petgraph/petgraph | https://docs.rs/petgraph |
| egui_graphs | widget de visualizacao de grafo para egui, backing petgraph | Rust | grafo interativo (wasm) | ativo (2026-06-07, 682 stars, MIT) | https://github.com/blitzarx1/egui_graphs | https://docs.rs/egui_graphs |
| fdg | force directed graph drawing library | Rust | layout force-directed | estagnado (2025-03-06, 226 stars, MIT) | https://github.com/grantshandy/fdg | https://docs.rs/fdg |
| rustworkx | graph library de alta performance (rust com api python) | Rust | algoritmos de grafo | ativo (2026-06-22, 1711 stars, Apache-2.0) | https://github.com/Qiskit/rustworkx | https://www.rustworkx.org |
| graph (neo4j-labs) | algoritmos de grafo de alta performance | Rust | algoritmos de grafo | ativo (2026-06-10, 437 stars, MIT) | https://github.com/neo4j-labs/graph | https://docs.rs/graph |
| graphviz-rust | funcoes para gerar/parsear dot lang | Rust | grafo via graphviz/dot | ativo (2026-06-23, 92 stars, MIT) | https://github.com/besok/graphviz-rust | https://docs.rs/graphviz-rust |
| layout (layout-rs) | renderiza arquivos dot do graphviz, rust puro | Rust | layout de grafo (dot) | estagnado (2025-05-22, 739 stars, MIT) | https://github.com/nadavrot/layout | https://github.com/nadavrot/layout |
| cosmos / cosmograph | force graph com layout e render acelerado por gpu (webgl) | TypeScript | grafo gpu (web) | ativo (2026-06-18, 1170 stars, MIT) | https://github.com/cosmosgl/graph | https://cosmograph.app/docs |
| sigma.js | viz de grafos com milhares de nodes/edges (webgl) | TypeScript | grafo (webgl) | ativo (2026-06-09, 12076 stars, MIT) | https://github.com/jacomyal/sigma.js | https://www.sigmajs.org |
| cytoscape.js | graph theory e viz/analise de network | JavaScript | grafo (canvas) | ativo (2026-06-24, 11062 stars, MIT) | https://github.com/cytoscape/cytoscape.js | https://js.cytoscape.org |
| graphology | objeto de grafo multiproposito para js/ts | JavaScript | data structure de grafo | baixa atividade (2025-12-03, 1696 stars, MIT) | https://github.com/graphology/graphology | https://graphology.github.io |
| ngraph | familia de libs de grafo e layout (anvaka) | n/a na api | grafo e layout (web) | ativo (2026-06-23, 1511 stars, MIT) | https://github.com/anvaka/ngraph | https://github.com/anvaka/ngraph |
| vis-network | views de network dinamicas e auto-organizadas | JavaScript | grafo (canvas) | ativo (2026-06-24, 3590 stars, Apache-2.0) | https://github.com/visjs/vis-network | https://visjs.github.io/vis-network |

## 3. engines de dados que alimentam viz

| projeto | descricao | linguagem | tipo de viz | status | github | docs |
|---|---|---|---|---|---|---|
| rerun | visualizar, consultar e streamar dados multimodais (robotica) | Rust | viz multimodal time-series/3d | ativo (2026-06-25, 10987 stars, Apache-2.0) | https://github.com/rerun-io/rerun | https://rerun.io/docs |
| perspective | componente de viz e analytics para datasets grandes/streaming | C++ | tabelas e charts streaming | ativo (2026-06-25, 10989 stars, Apache-2.0) | https://github.com/perspective-dev/perspective | https://perspective.finos.org |
| polars | query engine para dataframes, base de dados de viz | Rust | dataframe (alimenta viz) | ativo (2026-06-25, 38872 stars, MIT) | https://github.com/pola-rs/polars | https://docs.pola.rs |

## 4. renderers 2d/3d e gpu (rust)

| projeto | descricao | linguagem | tipo de viz | status | github | docs |
|---|---|---|---|---|---|---|
| wgpu | api grafica cross-platform, pure rust (base de webgpu) | Rust | gpu backend | ativo (2026-06-25, 17444 stars, Apache-2.0) | https://github.com/gfx-rs/wgpu | https://docs.rs/wgpu |
| vello | renderer 2d gpu-compute-centric | Rust | render vetorial 2d gpu | ativo (2026-06-25, 4125 stars, Apache-2.0) | https://github.com/linebender/vello | https://docs.rs/vello |
| piet | abstracao para grafica 2d | Rust | abstracao 2d | ativo (2026-05-02, 1367 stars, Apache-2.0) | https://github.com/linebender/piet | https://docs.rs/piet |
| lyon | tesselacao de paths para render 2d na gpu | Rust | tesselacao vetorial | ativo (2026-05-03, 2579 stars, dual MIT/Apache) | https://github.com/nical/lyon | https://docs.rs/lyon |
| resvg | render de svg | Rust | render svg | ativo (2026-06-05, 3899 stars, Apache-2.0) | https://github.com/linebender/resvg | https://docs.rs/resvg |
| tiny-skia | subset do skia portado para rust | Rust | rasterizacao 2d cpu | baixa atividade (2026-02-05, 1594 stars, BSD-3-Clause) | https://github.com/linebender/tiny-skia | https://docs.rs/tiny-skia |
| femtovg | vector drawing 2d antialiased | Rust | render vetorial 2d | ativo (2026-06-22, 918 stars, dual MIT/Apache) | https://github.com/femtovg/femtovg | https://docs.rs/femtovg |
| raqote | graphics library 2d | Rust | rasterizacao 2d | estagnado (2025-02-11, 1177 stars, BSD-3-Clause) | https://github.com/jrmuizel/raqote | https://docs.rs/raqote |
| three-d | renderer 2d/3d cross-platform (inclui web) | Rust | render 3d (web) | ativo (2026-06-24, 1637 stars, MIT) | https://github.com/asny/three-d | https://docs.rs/three-d |
| kiss3d | engine grafica 3d simples | Rust | render 3d | ativo (2026-06-25, 1709 stars, BSD-3-Clause) | https://github.com/dimforge/kiss3d | https://docs.rs/kiss3d |
| rend3 | renderer 3d sobre wgpu (modo manutencao) | Rust | render 3d | arquivado/manutencao (2024-07-08, 1160 stars, Apache-2.0) | https://github.com/BVE-Reborn/rend3 | https://docs.rs/rend3 |
| bevy | game engine data-driven, ecs, render 2d/3d, wasm | Rust | engine 3d/2d (viz via plugins) | ativo (2026-06-25, 46848 stars, Apache-2.0) | https://github.com/bevyengine/bevy | https://bevyengine.org/learn |
| macroquad | game engine cross-platform, compila para wasm/webgl | Rust | render 2d/jogos (web) | ativo (2026-06-15, 4511 stars, dual MIT/Apache) | https://github.com/not-fl3/macroquad | https://docs.rs/macroquad |
| nannou | framework de creative coding | Rust | viz generativa/arte | ativo (2026-06-23, 6709 stars, licenca nao confirmado na api) | https://github.com/nannou-org/nannou | https://guide.nannou.cc |
| rapier | engines de fisica 2d e 3d focadas em performance | Rust | simulacao fisica (viz dinamica) | ativo (2026-06-25, 5471 stars, Apache-2.0) | https://github.com/dimforge/rapier | https://rapier.rs |

## 5. frameworks de ui que hospedam viz (rust, wasm-capable)

| projeto | descricao | linguagem | tipo de viz | status | github | docs |
|---|---|---|---|---|---|---|
| egui | gui immediate-mode, roda em web e nativo | Rust | host de viz (wasm) | ativo (2026-06-25, 29502 stars, Apache-2.0) | https://github.com/emilk/egui | https://docs.rs/egui |
| iced | gui cross-platform inspirada em elm | Rust | host de viz | ativo (2026-06-25, 30831 stars, MIT) | https://github.com/iced-rs/iced | https://docs.rs/iced |
| leptos | framework web rust (wasm), fine-grained reactivity | Rust | host de viz (wasm) | ativo (2026-06-25, 21004 stars, MIT) | https://github.com/leptos-rs/leptos | https://leptos.dev |
| yew | framework rust/wasm para web apps | Rust | host de viz (wasm) | ativo (2026-06-23, 32700 stars, Apache-2.0) | https://github.com/yewstack/yew | https://yew.rs |
| dioxus | framework fullstack web/desktop/mobile | Rust | host de viz (wasm) | ativo (2026-06-22, 36517 stars, Apache-2.0) | https://github.com/DioxusLabs/dioxus | https://dioxuslabs.com/learn |
| slint | toolkit gui declarativo, multi-linguagem | Rust | host de viz | ativo (2026-06-25, 23018 stars, dual/custom, ver repo) | https://github.com/slint-ui/slint | https://slint.dev |
| makepad | plataforma de dev rust, compila para wasm/webgl/metal | Rust | host de viz (wasm) | ativo (2026-06-25, 6457 stars, MIT) | https://github.com/makepad/makepad | https://makepad.dev |
| floem | ui nativa rust com reatividade fine-grained | Rust | host de viz | ativo (2026-06-21, 4182 stars, MIT) | https://github.com/lapce/floem | https://docs.rs/floem |
| xilem | framework de ui nativa experimental | Rust | host de viz | ativo (2026-06-02, 5419 stars, Apache-2.0) | https://github.com/linebender/xilem | https://docs.rs/xilem |
| vizia | gui declarativa em rust | Rust | host de viz | ativo (2026-06-22, 2186 stars, MIT) | https://github.com/vizia/vizia | https://docs.rs/vizia |
| freya | gui cross-platform nao-web sobre skia | Rust | host de viz | ativo (2026-06-25, 2792 stars, MIT) | https://github.com/marc2332/freya | https://freyaui.dev |
| ratatui | crate para tui no terminal (inclui widgets de chart) | Rust | charts em terminal | ativo (2026-06-23, 21265 stars, MIT) | https://github.com/ratatui/ratatui | https://ratatui.rs |

## 6. base de math, geometria e cor (suporte a viz)

| projeto | descricao | linguagem | tipo de viz | status | github | docs |
|---|---|---|---|---|---|---|
| nalgebra | algebra linear para rust | Rust | math (suporte a viz) | ativo (2026-06-18, 4747 stars, Apache-2.0) | https://github.com/dimforge/nalgebra | https://nalgebra.org |
| glam | algebra linear simples e rapida para games/graphics | Rust | math (suporte a viz) | ativo (2026-06-25, 1988 stars, dual MIT/Apache) | https://github.com/bitshifter/glam-rs | https://docs.rs/glam |
| kurbo | manipulacao de curvas (beziers, paths) | Rust | geometria 2d | ativo (2026-05-13, 964 stars, dual MIT/Apache) | https://github.com/linebender/kurbo | https://docs.rs/kurbo |
| palette | calculo e conversao de cor linear | Rust | cor (suporte a viz) | ativo (2026-06-14, 825 stars, dual MIT/Apache) | https://github.com/Ogeon/palette | https://docs.rs/palette |
| colorgrad | escalas de cor para maps/charts/data-viz | Rust | cor/gradiente (data-viz) | ativo (2026-03-12, 360 stars, licenca nao confirmado na api) | https://github.com/mazznoer/colorgrad-rs | https://docs.rs/colorgrad |

## 7. cluster js/wasm de contraste (a lacuna)

esses sao os maduros que o rust+wasm ainda nao iguala em viz declarativa de alto
nivel. entram como referencia, nao como dependencia.

| projeto | descricao | linguagem | tipo de viz | status | github | docs |
|---|---|---|---|---|---|---|
| d3 | data-driven documents, svg/canvas/html, inclui d3-force | Shell (mono-repo) | viz de baixo nivel (web) | ativo (2026-05-28, 113130 stars, ISC) | https://github.com/d3/d3 | https://d3js.org |
| three.js | biblioteca 3d em javascript (webgl/webgpu) | JavaScript | render 3d (web) | ativo (2026-06-25, 113314 stars, MIT) | https://github.com/mrdoob/three.js | https://threejs.org |
| echarts | charting e data-viz interativo para browser | TypeScript | charts (web) | ativo (2026-06-24, 66660 stars, Apache-2.0) | https://github.com/apache/echarts | https://echarts.apache.org |
| chart.js | charts html5 via canvas | JavaScript | charts (web) | ativo (2026-05-27, 67530 stars, MIT) | https://github.com/chartjs/Chart.js | https://www.chartjs.org |
| plotly.js | charting javascript por tras do plotly e dash | JavaScript | charts interativos (web) | ativo (2026-06-25, 18235 stars, MIT) | https://github.com/plotly/plotly.js | https://plotly.com/javascript |
| deck.gl | framework de viz acelerada por webgl2 | TypeScript | viz geoespacial/large data (web) | ativo (2026-06-25, 14278 stars, MIT) | https://github.com/visgl/deck.gl | https://deck.gl |

## itens marcados "nao confirmado"

- licenca de textplots, nannou e colorgrad: a github api retornou `license.spdx_id`
  ausente ou nao-spdx. repo resolve e demais dados valem; so a licenca fica como
  nao confirmado ate ler o arquivo de licenca no repo.
- cosmograph (gpu): divergencia entre fontes. o repo `cosmosgl/graph` se descreve
  como "GPU-accelerated force graph layout and rendering". a pagina do produto
  (cosmograph.app) posiciona como "fastest web-based force network graph layout
  and rendering" e nao cita gpu no texto que foi possivel ler. registro a
  diferenca, nao consolido. o numero de stars (1170) e do repo, nao do produto.
- poloto: a api marca a linguagem do repo `poloto-project` como Jupyter Notebook
  porque o workspace carrega notebooks de exemplo; o crate poloto em si e rust.
  linguagem do crate fica como nao confirmado pela api (confirmavel em docs.rs).
- lib.rs/visualization: nao serviu de fonte de numero. retornou http 403 a fetch
  automatizado. resolve em browser e fica como fonte de descoberta, nao de dado.

## divergencias entre fontes (explicitas, nao consolidadas)

1. cosmograph: repo diz gpu-accelerated, pagina de produto nao menciona gpu. ver
   acima.
2. contagem de stars github topics vs api: a pagina de topics arredonda (ex.
   plotters "4.6k", charming "2.6k"); a api da o numero cru (4583, 2558). uso o
   numero cru da api e cito a data.

## analise (dimensao viz)

o mapa confirma o que o `04-corpus-pesquisa.md` ja suspeitava: rust tem base
sobrando e topo de pilha faltando. a base e densa. wgpu, vello, lyon, kurbo,
glam, nalgebra, palette, resvg. tudo ativo, tudo com push recente, licenca
permissiva na maioria. e o andar de cima, a viz declarativa que um analista usa
sem escrever shader, que e raso.

olha os charts. plotters e o mais forte do lado rust, 4583 stars, e ainda assim
gera imagem estatica; interatividade nao e o forte dele. quando a pesquisa quer
charts ricos, o caminho que aparece e embrulhar javascript: charming chama
echarts, plotly.rs chama plotly.js, chart-js-rs chama chart.js. a riqueza vem do
runtime js, nao de render nativo rust. do outro lado, echarts tem 66660 stars e
chart.js 67530. a distancia de maturidade nao e pequena, e ela existe porque o js
teve uma decada de vantagem nesse andar especifico.

grafos contam a mesma historia com um detalhe pior. petgraph (3940 stars) e o
padrao para a estrutura de dados, e ele nao desenha nada. quem desenha grafo
interativo de verdade no rust e o egui_graphs (682 stars), preso ao egui e com
force-directed o(n^2) ingenuo, sem barnes-hut. fdg, a lib de layout
force-directed que existe, esta estagnada: ultimo push em marco de 2025, mais de
um ano parado. enquanto isso o lado web tem sigma.js (12076), cytoscape.js
(11062), e o cosmos/cosmograph empurrando layout de grafo pra gpu no browser. a
lacuna que o briefing aponta, "nao ha equivalente maduro a d3-force ou networkx
em rust+wasm", se sustenta no dado. rustworkx (1711) e o graph da neo4j-labs
(437) cobrem algoritmo, nao desenho. ninguem junta os dois com a folga que o d3
junta.

tem um padrao de governanca util pro caranguejo vermelho aqui. resvg saiu de
RazrFalcon para linebender, kiss3d saiu de sebcrozet para dimforge, perspective
saiu de finos para perspective-dev. projetos de viz em rust tendem a migrar de
mantenedor solo para uma org quando amadurecem. os que nao migram tendem a parar:
fdg, raqote (push em fevereiro de 2025), textplots, layout-rs, rend3 ja em
manutencao declarada. citar um desses sem checar a data e como recomendar uma
casa que ja foi abandonada.

a leitura pro plev e direta. a base que o plev precisa ja existe e e boa (wgpu,
cosmic-text, parley, taffy, vello como referencia). o vao e exatamente onde o
plev pode chegar primeiro: charts e grafos gpu-nativos, interativos, com
hit-testing e a11y de fabrica, sem embrulhar js. nao e um vao teorico. e um vao
que sigma.js e echarts ja preencheram no js e que ninguem preencheu no rust+wasm
com qualidade equivalente.

## passo 5: relatorio de taboos (brennerwritter)

rodei os 24 taboos na prosa de analise e nas notas. o que foi verificado e
corrigido:

- emoji: nenhum no texto autoral. os que aparecem nas descricoes da tabela vem
  do campo `description` cru da github api (ex. plotters, ratatui) e ficam entre
  aspas de origem; nao reescrevi metadado de terceiro, mas tambem nao usei emoji
  proprio. zero emoji em qualquer frase minha.
- em dash: nenhum. usei virgula, ponto e dois pontos no lugar.
- ego phrases ("ninguem chegou nisso", "isso e publicavel"): nenhuma. removi
  qualquer reforco.
- arco de revelacao escalado (A vira B vira C): evitado. a analise afirma direto.
- caixa: minuscula como padrao, maiuscula so em termo tecnico e sigla (gpu, wasm,
  ecs, svg, api, js, o(n^2)) e nomes proprios de projeto/org.
- entropia variada: alternei frase curta incisiva ("a base e densa.") com periodo
  longo. less, but better aplicado.
- vocabulario tecnico em ingles mantido (force-directed, hit-testing, runtime,
  push, stars, dataframe, backend).
- regra do tres: revisei enumeracoes; onde listo tres exemplos e por dado real
  (sigma.js, cytoscape.js, cosmos), nao por cadencia retorica.
- copula avoidance ("boasts", "serves as", "features"): nao usei; mantive "e"/"tem".
- negative parallelism ("not only X but Y"): ausente.
- linguagem promocional ("vibrant", "groundbreaking", "showcase", "landscape"
  abstrato), cliche ("delve", "tapestry", "intricate", "underscore", "pivotal",
  "testament"): varridos, nenhum presente.
- atribuicao vaga ("experts argue"): nenhuma; toda afirmacao numerica tem repo e
  data colados.
- hedging excessivo: cortado. afirmo onde o dado sustenta, marco "nao confirmado"
  onde nao sustenta.
- conclusao generica positiva ("the future looks bright"): nao usei; fecho com o
  vao concreto que o plev pode ocupar.
- chatbot artifacts, sycophantic tone, curly quotes, knowledge cutoff disclaimer:
  nenhum presente.

itens com tolerancia zero: nenhuma ocorrencia. itens qualitativos: nenhuma
ocorrencia que exigisse reescrita alem das ja feitas.
