---
project: plev
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-11
domain: research
---

# reference analysis: charts e visualization

data: 2026-03-11
status: completo

## escopo

analise de bibliotecas rust para charts e visualizacao de dados, avaliando relevancia para futuras aplicacoes construidas sobre plev.

---

## repositorios analisados

### plotters (plotters-rs/plotters), 4531 stars, v0.3.7

**o que e:** biblioteca de desenho em pure rust focada em data plotting. suporta multiplos backends (bitmap, SVG, html5 canvas/WASM, gtk/cairo, piston window) e renderizacao de graficos 2d/3d com milhoes de data points.

**arquitetura:**
- modelo backend-agnostico: `DrawingBackend` trait abstrai o alvo de renderizacao. backends sao crates independentes desde v0.3.
- hierarquia de abstraccao: `DrawingBackend` -> `DrawingArea` (layout/coordenadas) -> `ChartContext` (eixos, mesh, series) -> `Element` (primitivas composiveis).
- elementos composiveis: primitivas (circle, text, pathelement) combinam-se em `ComposedElement` via operador `+`.
- font handling dual: `ttf` feature (font-kit, carrega system fonts) ou `ab_glyph` feature (pure rust, requer registro manual de fontes, ideal para cross-compile e WASM).
- WASM: `CanvasBackend` renderiza em html5 canvas. API identica aos outros backends.
- features granulares: cada tipo de serie (line, histogram, candlestick, boxplot, errorbar, area, point) e feature flag independente. build minimo compila em <6s com apenas `itertools`.

**downloads crates.io:** 136.4m total, 21.1m recentes.

**licenca:** MIT.

**relevancia para plev:**
- plotters *nao* renderiza via GPU pipeline proprio, usa backends de rasterizacao (bitmap) ou markup (SVG/canvas). para plev, a integracao direta dos backends existentes nao faz sentido.
- o modelo de abstracoes (drawingarea com coordenadas mapeadas, chartcontext com eixos/mesh, elementos composiveis) e referencia excelente para design de API de charts.
- o pattern de `ab_glyph` feature (embedded fonts, pure rust, sem system dependencies) valida a mesma decisao tomada em plev para WASM/ios/android.
- a granularidade de features (series individuais como opt-in) e modelo a seguir para evitar bloat.

**insight principal:** a separacao `DrawingBackend` -> `DrawingArea` -> `ChartContext` -> `Element` permite que a mesma especificacao de chart renderize em qualquer backend. se plev criar um `PlottersBackend` que emita `SceneNode::Rect` e `SceneNode::Text`, toda a galeria de plotters funcionaria sobre o engine GPU sem modificacao do codigo de chart.

**limitacao:** nao tem suporte nativo a interatividade (zoom, pan, tooltips), e fundamentalmente um gerador de imagens estaticas. a composicao de elementos e CPU-bound; nao ha batching GPU. 176 issues abertas, atividade de manutencao reduzida (ultimo push fev/2026).

---

### charming (yuankunzhang/charming), 2515 stars, v0.6.0

**o que e:** wrapper rust para apache echarts. constroi especificacoes de charts via API declarativa rust e delega renderizacao ao runtime javascript do echarts.

**arquitetura:**
- a lib nao renderiza nada diretamente. gera JSON (especificacao echarts) que e consumido por um dos tres renderers:
  - `HtmlRenderer`: gera fragmento HTML com `<script>` que carrega echarts JS. renderizacao acontece no browser do usuario.
  - `ImageRenderer` (feature `ssr`): embute `deno_core` para executar echarts JS server-side e gerar png/SVG/jpeg/gif/webp/etc.
  - `WasmRenderer` (feature `wasm`): renderiza via echarts JS no runtime WASM. mutuamente exclusivo com `ssr`.
- tipos de chart: bar, line, pie, scatter, candlestick, boxplot, heatmap, radar, tree, treemap, sunburst, graph, sankey, funnel, gauge, parallel, theme river, geo/map.
- 14 temas built-in (default, dark, vintage, westeros, chalk, etc.).
- MSRV nao garantido, acompanha `deno_core` que exige latest stable.

**downloads crates.io:** 859k total, 211k recentes.

**licenca:** MIT or apache-2.0.

**relevancia para plev:**
- dependencia no runtime javascript do echarts torna integracao direta com plev inviavel. plev e GPU-first; charming e JS-first.
- a API declarativa e referencia de ergonomia excelente. a forma como `Chart::new().legend(...).series(Pie::new()...)` compoe components e analogo ao builder pattern de plev.
- o catalogo de chart types (30+) e temas (14) serve como benchmark de feature completeness para qualquer sistema de charts futuro.
- o approach `WasmRenderer` demonstra que charts echarts em WASM sao possiveis, mas com overhead de runtime JS embutido.

**insight principal:** charming valida que uma API rust declarativa pode mapear 1:1 para a riqueza do echarts. porem, a dependencia em javascript (via deno_core no server ou runtime JS no browser) e o preco dessa riqueza. para plev, o caminho seria implementar rendering nativo dos chart types mais comuns (line, bar, pie, scatter) diretamente em scenenodes, usando a API declarativa de charming como inspiracao de ergonomia.

**limitacao:** nao e uma lib de rendering, e um gerador de specs. toda renderizacao e delegada ao echarts JS. o `ImageRenderer` adiciona ~30mb de dependencia (deno_core). feature `ssr` e `wasm` sao mutuamente exclusivas. 21 issues abertas.

---

### egui_graphs (blitzarx1/egui_graphs), 659 stars, v0.29.0

**o que e:** widget de visualizacao de grafos (graph/network, nao charts) para egui, usando petgraph como estrutura de dados. foco em grafos interativos com layouts automaticos.

**arquitetura:**
- widget egui: `GraphView` e adicionado via `ui.add(&mut GraphView::new(&mut graph))`. integra-se ao loop de renderizacao do egui.
- estrutura de dados: `petgraph::StableGraph` como backing store. `egui_graphs::Graph` wrapa petgraph com metadados visuais.
- layout plugavel via trait `Layout`:
  - `LayoutRandom`: scatter aleatorio (default).
  - `LayoutHierarchical`: layout em camadas (ranked/sugiyama-like).
  - `LayoutForceDirected`: fruchterman-reingold o(n^2) com extras composiveis (e.g., center gravity).
  - custom: implementar `Layout` + `LayoutState` traits.
- interatividade: zoom, pan, click, double-click, select, drag por node/edge. event system via feature `events`.
- styling hooks: closures para customizar stroke de nodes/edges sem reimplementar draw completo. para mudancas geometricas (shapes, icons), implementar `DisplayNode`/`DisplayEdge` traits.
- WASM: suportado (web-demo disponivel).

**downloads crates.io:** 129k total, 10.4k recentes.

**licenca:** MIT.

**relevancia para plev:**
- egui_graphs resolve um problema diferente de charts, visualizacao de grafos (nodes + edges). relevante se plev precisar de network visualization, dependency graphs, knowledge graphs, etc.
- o sistema de layouts plugavel (trait + state + composable extras) e referencia de design para qualquer sistema de layout automatico em plev.
- o pattern de "styling hooks vs custom drawer" (tabela comparativa no readme) e modelo de extensibilidade que plev poderia adotar: hooks para tweaks rapidos, traits para customizacao total.
- a integracao com petgraph como data structure e pattern reutilizavel, petgraph e o standard de facto para grafos em rust.

**insight principal:** o sistema de extras composiveis para force-directed layout (`FruchtermanReingoldWithExtras<E>` onde e e tupla de extras) demonstra como parametrizar algoritmos de layout sem explosion combinatoria. cada extra acumula no displacement vector em ordem da tupla. esse pattern de composicao e aplicavel alem de graph layout.

**limitacao:** acoplado ao egui, nao funciona fora do ecossistema egui. layout force-directed e o(n^2) naive (sem barnes-hut ou similar), limitando escalabilidade para grafos grandes (>10k nodes). pre-1.0, breaking changes frequentes (v0.29 segue versao do egui). 10 issues abertas.

---

## padroes cross-cutting

1. **backend abstraction e universal.** plotters (drawingbackend trait), charming (3 renderers), egui_graphs (egui widget). todas separam especificacao visual de renderizacao. para plev, isso sugere que um chart system deveria definir chart specs independentes do rendering pipeline.

2. **WASM e first-class em todas.** plotters via canvasbackend, charming via wasmrenderer, egui_graphs via eframe WASM. nenhuma trata WASM como afterthought. plev ja tem WASM support nativo, um chart system sobre plev herdaria isso automaticamente.

3. **declarative builder API e o padrao.** todas usam builder pattern para construcao de charts/grafos. `ChartBuilder::on(&root).caption(...).build_cartesian_2d(...)` (plotters), `Chart::new().legend(...).series(...)` (charming), `GraphView::new(&mut graph).with_styles(...)` (egui_graphs). plev ja usa builder pattern (builder.rs), charts seriam extensao natural.

4. **interatividade e o divisor de aguas.** plotters nao tem. charming delega ao echarts JS. egui_graphs implementa nativamente (zoom, pan, drag, events). para plev, interatividade nativa e vantagem competitiva: charts GPU-rendered com hit-testing, gestures e signals integrados.

5. **fonts sao problema recorrente.** plotters oferece `ab_glyph` (embedded, pure rust) vs `ttf` (system fonts). charming delega ao echarts. egui_graphs usa epaint/egui fonts. plev ja resolveu isso com cosmic-text + fontes embutidas (inclusive sans) para WASM/ios/android.

---

## implicacoes para plev

### curto prazo (nao bloqueia roadmap atual)
nenhuma dessas libs se integra diretamente com plev. todas dependem de seus proprios sistemas de rendering (bitmap, SVG, canvas JS, egui). nao ha acao imediata necessaria.

### medio prazo (quando apps precisarem de charts)
1. **plottersbackend**: implementar `plotters::drawing::DrawingBackend` que emite `SceneNode::Rect` e `SceneNode::Text` para o compositor plev. isso daria acesso a toda a galeria de plotters (line, bar, histogram, candlestick, etc.) renderizada via GPU pipeline do plev. custo estimado: 1 backend adapter (~500 linhas).

2. **chart primitivas nativas**: para charts interativos (tooltips, zoom, cross-hair), implementar line/bar/pie/scatter diretamente como views/components plev, com hit-testing e signals integrados. API inspirada em charming (declarativa, composivel). custo: significativamente maior, mas resultado superior em performance e interatividade.

### longo prazo (graph visualization)
3. **graph widget**: se plev precisar de network visualization, o design de egui_graphs (layouts plugaveis, styling hooks, petgraph backing) e blueprint. a implementacao seria sobre o sistema de views/components de plev, com layout algorithms como modulos independentes.

### o que nao fazer
- nao embutir echarts JS via charming, contradiz a filosofia GPU-first de plev.
- nao tentar replicar a galeria completa de 30+ chart types de uma vez, comeca com line, bar, pie, scatter.
- nao acoplar chart specs ao rendering, manter separacao spec/render como plotters faz.
