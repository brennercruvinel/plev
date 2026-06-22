---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: task-tracking
---

# task-34: exploracao e extracao de ideias dos repos de referencia, P2

## objetivo
analisar o codigo-fonte dos 56 repositorios clonados em `/Users/aac/Dev/bc/bunker/repos/` e extrair padroes, tecnicas e ideias concretas aplicaveis ao plev. nao e para copiar codigo, e para estudar, entender e documentar decisoes arquiteturais que informem o desenvolvimento futuro.

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
1. [x] um documento `mission/knowledge/extracted-patterns.md` com patterns concretos extraidos do codigo
2. [x] cada pattern inclui: repo fonte, arquivo/funcao, descricao, aplicabilidade ao plev, decisao (adotar/adaptar/ignorar)
3. [x] pelo menos 15 patterns documentados cobrindo rendering, a11y, animation, text, UX, **38 patterns extraidos**

## resultado
- **38 patterns** extraidos de **17 repos** em **6 fases**
- documento principal: `mission/knowledge/extracted-patterns.md`
- documentos auxiliares: `refs/pattern-extraction-parley-lyon-glam.md`, `refs/pattern-extraction-tui-apps.md`, `refs/extraction-fase-ef.md`
- top 10 prioridades de implementacao documentadas com estimativa de LOC

## fora de escopo
- copiar codigo de terceiros (violar licencas)
- modificar o plev nesta task (apenas documentar)
- analisar todos os 56 repos (foco nos 17 da checklist)
