---
id: task-34
title: "task-34: exploracao e extracao de ideias dos repos de referencia"
project: plev
type: tarefa
status: concluido
date: 2022-07-12
date_basis: cabecalho
topics: [pesquisa, competicao]
relations: [task-ref, task-27, task-30, task-31, task-32, pesquisa-extracted-patterns, pesquisa-pattern-extraction-parley-lyon-glam, pesquisa-pattern-extraction-tui-apps, pesquisa-extraction-fase-ef]
source: [docs/log/steps/task-34-exploration-extraction.md, docs/log/changes/task-34.md]
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# task-34: exploracao e extracao de ideias dos repos de referencia

Registra a leitura do código-fonte de repositórios de referência, clonados fora deste repositório, para extrair padrões de accesskit, parley, lyon, vello, makepad, xilem, animação, apps de TUI, WASM e frameworks concorrentes. O objetivo era estudar e documentar decisões de arquitetura, sem copiar código de terceiros. O resultado é o catálogo de padrões, com a decisão para cada um (adotar, adaptar ou ignorar), e a lista das dez prioridades de implementação; o catálogo principal está em [extracted-patterns.md](../pesquisas/extracted-patterns.md).

## objetivo
analisar o codigo-fonte dos 56 repositorios clonados em `bunker/repos/` (colecao local de clones de estudo, fora do repo) e extrair padroes, tecnicas e ideias concretas aplicaveis ao plev. nao e para copiar codigo, e para estudar, entender e documentar decisoes arquiteturais que informem o desenvolvimento futuro.

## justificativa
as 12 analises em `mission/knowledge/refs/` documentaram apis, features e conceitos de alto nivel. mas o valor real esta no codigo-fonte: como accesskit constroi a arvore, como makepad faz layout+draw simultaneo, como egui integra acessibilidade, como keyframe implementa o trait cantween, como vello faz encoding em streams. ter os repos clonados sem estudar o codigo e desperdicar a referencia.

## dependencias
- task-ref (research briefing), concluida
- repos clonados em bunker/repos/, concluido

## checklist por categoria

### fase a: prioridade imediata (informa task-30, 31, 32)

- [x] **accesskit (accesskit):** lazy activation protocol, per-frame treeupdate accumulator, widget-to-role mapping, viewid->nodeid cast, focus routing via actionrequest, null platform adapter. 6 patterns (ak1-ak6).
- [x] **parley (parley):** byte-index cursor com affinity, selection geometry via callback, plaineditordriver pattern, inlinebox. 4 patterns (a1-a4).
- [x] **lyon (lyon):** geometrybuilder trait (drop-in no quad pipeline!), lyon+wgpu integration template. 2 patterns (a5-a6).
- [x] **glam (glam-rs):** vec2 bytemuck pod avaliado. ignorado por agora (SIMD so beneficia vec4+). 1 pattern (a7).

### fase b: patterns de rendering e compositing

- [x] **vello (vello):** stream-of-arrays encoding, scene::append() fragment caching, resolver pattern com epoch-based glyph cache eviction. 3 patterns (b1-b3).
- [x] **makepad (makepad):** turtle layout (cursor-based layout+draw simultaneo), instanced draw call batching via manyinstances. 2 patterns (b4-b5).
- [x] **xilem (xilem):** view/element/widget tree separation, memoize com partialeq + zero-size closure check, dirty flag bubbling via merge_up, per-widget scene caching. 4 patterns (b6-b9).

### fase c: animacao e motion (validar task-27)

- [x] **natura (natura):** analytical spring solver com coeficientes pre-computados. bug encontrado: plev usa euler (frame-rate dependent), natura usa solucao analitica (incondicionalmente estavel). 1 pattern (c1).
- [x] **keyframe (keyframe):** keyframesequence com easing per-segment, const-generic array cantween, step/hold easing. 3 patterns (c2, c5, c6).
- [x] **mina (mina):** stateanimator com transition blending, timeline repeat/reverse/delay. 2 patterns (c3, c4).

### fase d: UX patterns de tui apps

- [x] **yazi (yazi):** event batching + render throttle, partial render flag, layer-based action routing, priority task scheduler, two-tier plugin isolation. 5 patterns (d1-d5).
- [x] **television (television):** channel/injector data abstraction, render gating por tipo de acao. 2 patterns (d6-d7).
- [x] **bottom (bottom):** auto-navigation from layout geometry, widget maximize/restore, configurable update rate. 3 patterns (d8-d10).

### fase e: WASM runtime patterns

- [x] **waforth (waforth):** shared table+memory interop. 1 pattern (e1).
- [x] **extism (extism):** plugin lifecycle com fuel+epoch sandboxing. 1 pattern (e2).

### fase f: competidores (leitura seletiva)

- [x] **dioxus (dioxus):** peek() untracked read + drop-guard write, delega rendering ao vello. 2 patterns (f2, f5).
- [x] **slint (slint):** lazy binding + constant-signal sentinel. 1 pattern (f3).
- [x] **leptos (leptos):** fxindexset para subscriber ordering, RAII observer drop guard. 2 patterns (f1, f4).

## criterios de aceite
1. [x] um documento [`extracted-patterns.md`](../pesquisas/extracted-patterns.md) com patterns concretos extraidos do codigo
2. [x] cada pattern inclui: repo fonte, arquivo/funcao, descricao, aplicabilidade ao plev, decisao (adotar/adaptar/ignorar)
3. [x] pelo menos 15 patterns documentados cobrindo rendering, a11y, animation, text, UX, **38 patterns extraidos**

## resultado
- **38 patterns** extraidos de **17 repos** em **6 fases**
- documento principal: [`extracted-patterns.md`](../pesquisas/extracted-patterns.md)
- documentos auxiliares: [`pattern-extraction-parley-lyon-glam.md`](../pesquisas/pattern-extraction-parley-lyon-glam.md), [`pattern-extraction-tui-apps.md`](../pesquisas/pattern-extraction-tui-apps.md), [`extraction-fase-ef.md`](../pesquisas/extraction-fase-ef.md)
- top 10 prioridades de implementacao documentadas com estimativa de LOC

## fora de escopo
- copiar codigo de terceiros (violar licencas)
- modificar o plev nesta task (apenas documentar)
- analisar todos os 56 repos (foco nos 17 da checklist)

## Registro de entrega

### 2022-07-06

#### iniciado
- branch: `task/TASK-34-exploration-extraction`
- task movida de `pending/` para `ongoing/`
- 6 agentes paralelos lancados para explorar repos por fase

#### fase a completa, accesskit + parley + lyon + glam
- 13 patterns extraidos (ak1-ak6, a1-a7):
  - **ak1-ak6** accesskit: lazy activation, treeupdate accumulator, widget-to-role, id mapping, focus routing, null adapter WASM
  - **a1-a4** parley: cursor affinity, selection geometry callback, plaineditordriver, inlinebox
  - **a5-a6** lyon: geometrybuilder trait (drop-in no quad pipeline!), wgpu integration template
  - **a7** glam: ignorado (SIMD so beneficia vec4+)
- descoberta chave: lyon reutiliza o quad pipeline existente via fillvertexconstructor, task-31 nao precisa de shader novo

#### fase b completa, rendering e compositing (vello, makepad, xilem)
- 9 patterns extraidos (b1-b9):
  - **b1** stream-of-arrays encoding (vello), ignore soa, adapt transform dedup
  - **b2** scene fragment caching via append (vello), **adapt** (high priority)
  - **b3** epoch-based cache eviction (vello), adapt for shaping cache
  - **b4** turtle layout (makepad), ignore (taffy is correct for plev)
  - **b5** instanced draw call batching (makepad), adapt post-task-31
  - **b6** view/element/widget tree separation (xilem), adapt for component trait
  - **b7** memoize + partialeq data (xilem), **adapt** (component-level)
  - **b8** dirty flag bubbling merge_up (masonry), adapt when component tree matures
  - **b9** per-widget scene caching (masonry), **adapt** (highest priority for perf)

#### fase c completa, animacao (natura, keyframe, mina)
- 6 patterns extraidos (c1-c6):
  - **c1** analytical spring solver (natura), bug de corretude: plev euler vs analitico
  - **c2** keyframesequence com easing per-segment, maior feature gap
  - **c3** state animator com transition blending
  - **c4** timeline repeat/reverse/delay
  - **c5** const-generic array interpolate
  - **c6** step/hold easing

#### fase d completa, tui ux patterns (yazi, television, bottom)
- 10 patterns extraidos (d1-d10):
  - **d1** event batching + render throttle, 5-10x GPU reduction
  - **d8** auto-navigation from layout, essencial para task-30

#### fases e+f completas, WASM + competidores
- 7 patterns extraidos (e1-e2, f1-f5):
  - **f1** fxindexset subscribers (leptos), corretude + perf
  - **f4** RAII observer drop guard (leptos), previne corrupcao
  - **f3** constant-signal sentinel (slint), zero overhead
  - **f5** dioxus delega ao vello, validacao estrategica

#### finalizado
- **38 patterns** extraidos de **17 repos** em **6 fases**
- documento: [`extracted-patterns.md`](../pesquisas/extracted-patterns.md)
- top 10 prioridades de implementacao com LOC estimado
- todos os criterios de aceite atendidos (minimo era 15, entregou 38)
