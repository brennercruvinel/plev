---
project: plev
audience: [ai-agents, contributors]
status: active
last-updated: 2026-09-21
domain: index
---

# knowledge base, plev

indice dos architecture decision records do plev. cada linha aponta para o arquivo, com status e data reais do frontmatter. a ordem e cronologica de proposito: o git so preservou de 2026-06-10 em diante (expurgo de historico antes do primeiro push), entao este indice e a fonte da linha do tempo que o git perdeu. fundacao em 2021, ondas seguintes ano a ano ate 2026.

status usados: `reference` (conhecimento congelado de uma sessao), `accepted` (decisao em vigor), `draft-v0` (spec em rascunho), `living` (atualizado continuamente), `active` (este indice).

## fundacao, fase 0 (2021)

| data | adr | status | resumo |
|------|-----|--------|--------|
| 2021-03-06 | [view-trait-design](view-trait-design.md) | reference | viewcontext sem ref ao compositor, views retornam vec<scenenode>, &mut para extensao futura |
| 2021-03-24 | [component-design](component-design.md) | reference | lifecycle trait separada de view, component<l> com estado generico, campos disjuntos para o borrow checker |
| 2021-04-15 | [layout-engine](layout-engine.md) | reference | taffy 0.9 wrappado, two-phase rendering, computedbounds absolutos, <1ms/1000 nodes em release |
| 2021-05-02 | [signal-system-design](signal-system-design.md) | reference | push-pull hibrido, slotmap runtime, rc closures para borrow safety, comparefn type-erased para memos |
| 2021-05-27 | [input-system-design](input-system-design.md) | reference | event queue (nao closures), hit-testing linear reverso, click-to-focus, modifierstate proprio |
| 2021-06-16 | [layer-system](layer-system.md) | reference | premultiplied alpha, per-layer dirty tracking, offscreen textures, composite pass, gpuvec compartilhado |
| 2021-07-04 | [effects-architecture](effects-architecture.md) | reference | blur/shadow/composite fragment-only, texturepool grow-only, 13-tap gaussian, premultiplied alpha |
| 2021-07-26 | [dsl-narrate-design](dsl-narrate-design.md) | reference | gramatica hibrida verbal, disambiguacao no parser, modifier categories, format interpolation, builder stubs |
| 2021-08-15 | [ios-build](ios-build.md) | reference | font loading embedded, metal via primary, lifecycle handlers, requisitos de info.plist, script de simulador |
| 2021-08-30 | [mobile-specifics](mobile-specifics.md) | reference | safe area insets (#[cfg] android), IME state machine, lifecycle callbacks, heuristica de teclado, scale factor |
| 2021-09-17 | [wasm-webgpu-validation](wasm-webgpu-validation.md) | reference | eventloopproxy para GPU init async, limits default vs webgl2, build guards |

## mobile e animacao inicial (2022)

| data | adr | status | resumo |
|------|-----|--------|--------|
| 2022-01-19 | [android-emulator-deploy](android-emulator-deploy.md) | reference | gradle + cargo-ndk, gameactivity 2.0.2, GPU host obrigatorio (swiftshader trava), showcase renderizando |
| 2022-02-24 | [animation-pattern-lerp](animation-pattern-lerp.md) | reference | frame-based exponential lerp, pixel-snap com round(), font size fixo evita jitter de re-shaping |

## pesquisa, benchmarks e competitivo (2022 a 2023)

| data | adr | status | resumo |
|------|-----|--------|--------|
| 2022-04-26 | [benchmark-results](benchmark-results.md) | reference | criterion m4 mac: push_rects 159-222m/s, dirty tracking 3.3us/1000, tessellation 1.5-3.7us/shape, signals 67ns/cycle |
| 2022-05-24 | [arxiv-paper-outline](arxiv-paper-outline.md) | reference | 11 secoes: architecture, cross-platform, DSL, accessibility, vector paths, evaluation |
| 2022-07-06 | [extracted-patterns](extracted-patterns.md) | reference | 38 patterns de 17 repos em 6 fases, top 10 priorizado com LOC, blueprints para task-30/31 |
| 2022-08-11 | [parley-vs-cosmic-text](parley-vs-cosmic-text.md) | reference | comparacao factual: cursor API, selection geometry, inlinebox, harfrust, WASM, custo. recomendacao: wait |
| 2022-09-08 | [wasm-plugin-architecture](wasm-plugin-architecture.md) | reference | wasmtime vs wasmer vs extism, host function interface draft, 10-15ns/call. recomendacao: wait (p4) |
| 2022-10-27 | [touch-gesture-design](touch-gesture-design.md) | reference | 6-state gesturerecognizer, touchtracker fxhashmap, explicit instant para testabilidade, separado do mouse |
| 2022-11-19 | [makepad-gap-analysis](makepad-gap-analysis.md) | reference | inventario de features visuais do makepad vs plev, 9 categorias, gaps e prioridades |
| 2022-12-10 | [plev-vs-makepad-report](plev-vs-makepad-report.md) | reference | relatorio detalhado plev vs makepad: core, learnings, philosophy, priority roadmap |
| 2023-01-14 | [brief-strengths](brief-strengths.md) | reference | bullets dos diferenciais tecnicos: dirty tracking, text atlas, signal system, spring solver, SDF |
| 2023-02-09 | [arxiv-paper-draft](arxiv-paper-draft.md) | reference | rascunho do paper: abstract + 2 secoes, motivacao, gap analysis |

## hot reload (2024)

| data | adr | status | resumo |
|------|-----|--------|--------|
| 2024-01-22 | [hot-reload-design](hot-reload-design.md) | reference | 7 implementacoes estudadas, 3 tiers: shader (vello), DSL (makepad), rust (subsecond). dylib descartado (typeid UB) |
| 2024-02-06 | [shader-hot-reload-adr](shader-hot-reload-adr.md) | reference, adr-3 reversed | shader reload via notify watcher + pipeline recreation, errorscopeguard; channel polling in about_to_wait replaced by request_redraw (2026-09-22) |
| 2024-02-18 | [narrate-hot-reload-adr](narrate-hot-reload-adr.md) | reference | narrate DSL reload via runtime parser + override map, re-parse on access |
| 2026-09-22 | [hot-reload-on-by-default-in-gpu-context](hot-reload-on-by-default-in-gpu-context.md) | accepted | hot reload on em todo debug build desktop: shader e literal (string, cor hex) trocam na janela aberta, estado intacto, sem macro; código rebuilda e re-exec. ideia do rsx-hotreload do dioxus, sem dioxus; aprendizados em docs/mission/steps/checked/sessao-hot-reload-learnings.md |

## lint e modularizacao (2024)

| data | adr | status | resumo |
|------|-----|--------|--------|
| 2024-04-14 | [error-handling-lint-cleanup](error-handling-lint-cleanup.md) | reference | pleverror como tipo de erro unificado, setup_wasm_canvas(), consolidacao de #[allow(dead_code)] |
| 2024-05-21 | [adr-003-srp-modularization](adr-003-srp-modularization.md) | reference | limite de 300 linhas/arquivo, 44 monolitos em submodulos, API publica inalterada, 271 .rs total |
| 2024-05-23 | [srp-modularization](srp-modularization.md) | reference | mesma decisao do adr-003 com detalhes de sessao: padrao de divisao, armadilhas, metricas antes/depois |
| 2024-06-17 | [adr-004-hidpi-projection](adr-004-hidpi-projection.md) | reference | set_projection() para apps com layout em logical pixels em displays retina/hidpi |
| 2024-07-09 | [clippy-zero-warnings](clippy-zero-warnings.md) | reference | 107 warnings resolvidos em 12 categorias: float precision, collapsible if, default impls, too many args |

## onda responsividade, web e fidelidade (2024 a 2025)

| data | adr | status | resumo |
|------|-----|--------|--------|
| 2024-09-18 | [content-driven-layout-not-fixed-constants](content-driven-layout-not-fixed-constants.md) | accepted | geometria do container deriva do espaco disponivel, nunca de constantes; formula de grid; min/max so como limites |
| 2024-10-07 | [render-on-demand-requires-explicit-invalidation](render-on-demand-requires-explicit-invalidation.md) | accepted | event handler que muda estado visual deve retornar true; invalidacao e contrato de correcao, nao otimizacao |
| 2024-10-29 | [one-text-style-for-measurement-and-drawing](one-text-style-for-measurement-and-drawing.md) | accepted | uma TextStyle por run, compartilhada por measure_styled e textnodekey; heuristica chars*0.58 deletada |
| 2024-11-20 | [linearize-colors-before-the-gpu](linearize-colors-before-the-gpu.md) | accepted | cores sRGB linearizadas: to_linear_array() para clears/uniforms, vertex colors no shader; bug #303030 |
| 2024-12-04 | [render-into-an-srgb-view-format](render-into-an-srgb-view-format.md) | accepted | view sRGB quando a surface nao pode ser sRGB; surface_render_view(); web bate (48,48,48) com o desktop |
| 2024-12-19 | [measured-design-tokens-over-eyeballed-values](measured-design-tokens-over-eyeballed-values.md) | accepted | token entra no tema so com medicao do render vivo, nao de stylesheet nem de olho; graphite #303030 pinned por teste |
| 2025-01-08 | [embed-every-font-weight-in-use](embed-every-font-weight-in-use.md) | accepted | familia unica embutida (inclusive sans 300-700, desde 2026-08; antes rubik 400-700), familias default pinadas, determinismo |
| 2025-01-27 | [touch-as-synthesized-pointer-events](touch-as-synthesized-pointer-events.md) | accepted | touch sintetiza pointer events no caminho de mouse existente, sem vocabulario paralelo de evento |
| 2025-02-10 | [async-gpu-init-and-single-wasm-entry](async-gpu-init-and-single-wasm-entry.md) | accepted | spawn_local + eventloopproxy para GPU init async, feature web-entry, canvas 100vw/100vh |
| 2025-02-19 | [responsiveness-multiplatform-and-fidelity](responsiveness-multiplatform-and-fidelity.md) | reference | registro de sessao: text spill, distribuicao responsiva, gamma do browser, touch, licoes |

## onda formatos: monster, lot, parser (2025)

| data | adr | status | resumo |
|------|-----|--------|--------|
| 2025-03-26 | [monster-format-v0](monster-format-v0.md) | draft-v0 | spec do formato: i-frames (keyframes), delta ops (place/modify/replace/remove), tabela de easing, secoes com sha256 |
| 2025-05-14 | [binary-animation-format-with-discovered-deltas](binary-animation-format-with-discovered-deltas.md) | accepted | a decisao do .monster: h264 para vetores, seek O(1), delta descoberto, no que nao muda custa zero bytes |
| 2025-06-10 | [import-foreign-formats-by-conversion-not-embedding](import-foreign-formats-by-conversion-not-embedding.md) | accepted | lot le o json do lottie uma vez offline e converte para .monster; nenhum runtime estrangeiro embarcado |
| 2025-07-08 | [transpiler-reports-every-unmapped-construct](transpiler-reports-every-unmapped-construct.md) | accepted | o parser emite codigo + droplist com file:line, contagens congeladas em teste, nunca dropa em silencio |

## organizacao do workspace (2025)

| data | adr | status | resumo |
|------|-----|--------|--------|
| 2025-08-18 | [workspace-engine-at-root-libs-in-crates-demos-in-examples](workspace-engine-at-root-libs-in-crates-demos-in-examples.md) | accepted | tres tiers: engine na raiz, libs e apps em crates/, demos em examples/; workspace.package/dependencies/lints; shaders em src/gpu/shaders |

## organizacao do workspace (2026)

| data | adr | status | resumo |
|------|-----|--------|--------|
| 2026-07-13 | [one-knowledge-tree-and-a-minimal-root](one-knowledge-tree-and-a-minimal-root.md) | accepted | docs/ como arvore unica de conhecimento (kdb renomeado, arc absorvido de doc/), web/ dono do alvo web, script/ dono dos comandos, raiz so com o que as ferramentas exigem la |

## padroes de app (2026)

| data | adr | status | resumo |
|------|-----|--------|--------|
| 2026-08-30 | [official-app-pattern](official-app-pattern.md) | accepted | padrao oficial: estado em structs + widgets retidos + invalidacao explicita (template: showcase); builder para prototipos/demos; view/component/signal/narrate experimentais |
| 2026-09-21 | [core-theme-tailwind-classes-as-recipes](core-theme-tailwind-classes-as-recipes.md) | accepted | Theme::hoff_core (o kit typescript: shades, accents, sombras, Inter, raios) e theme.button: cada class list de botao virou receita (fill, shadow list com ring e inset, overlay, texto) por estado; Button / IconButton / SubmitButton reescritos; showcase Buttons e a ButtonsPage; dois defeitos globais: mix em alpha reto no rect_sdf (franja escura em toda quina) e sombra blur 0 sem AA; aprendizados em docs/mission/steps/checked/sessao-core-buttons-learnings.md |
| 2026-09-21 | [embed-an-app-view-by-frame-origin](embed-an-app-view-by-frame-origin.md) | accepted | view de app ganha origem de frame (WorkspaceView::set_frame, Sidebar.origin); begin_frame e do host; ide vira lib + bin com GitSession; showcase embute Editor e Git como app sections sem gutter |
| 2026-09-21 | [showcase-one-page-per-component](showcase-one-page-per-component.md) | accepted | o showcase vira uma pagina por widget do comps: tabela `view::pages::PAGES` em tres grupos (Foundations, Components alfabetico, Apps Demo), pagina propria, alias (mostra o exemplo vizinho com a nota do que falta) ou placeholder (widget que o comps ainda nao tem); `comps::nav::Sidebar` ganha `NavGroup` (grupos que dobram); foco de teclado por pagina; aprendizados em docs/mission/steps/checked/sessao-showcase-pages-learnings.md |
| 2026-09-18 | [comps-one-design-system-crate](comps-one-design-system-crate.md) | accepted | crates/comps e o design system (widgets, receitas, icones, charts, overlay, editor, shell) sobre o engine; todo valor visual e token do Theme (shape, control, size, duration, layout, shadows); breakpoints e AppShell (sidebar cheia / rail / drawer); ide, showcase e urnaui consomem, tema e componentes locais apagados; matriz tema x viewport testa todo widget; inventario em docs/catalog.md; aprendizados da sessao em docs/mission/steps/checked/sessao-comps-learnings.md |

## runtime de interação generativa (2026)

o programa em docs/mission/steps/ongoing/generative-runtime-program.md; uma linha por decisão das tasks 57 a 80.

| data | adr | status | resumo |
|------|-----|--------|--------|
| 2026-09-23 | [semantic-contract-as-pure-projection](semantic-contract-as-pure-projection.md) | accepted | crate `sem`, puro: todo controle tem `SemId` persistente escolhido pelo app (nunca contador nem posição), role, estado, ações com as `reads` de que dependem, policy e constraints; o app implementa `Semantics::project` sob demanda a partir do estado vivo; `finish` reporta todas as violações juntas; `Revision` estável (FNV-1a 64, ordem de árvore, sem `source`) e `diff` vazio exatamente quando os snapshots são iguais; `secret` guarda só tamanho (task-57) |
| 2026-09-23 | [actions-carry-observed-state](actions-carry-observed-state.md) | accepted | toda ação carrega o estado que o ator observou (`Request::observe`) e passa por um `Dispatcher` único para humano, agente, tecnologia assistiva e teste; `check` puro decide em ordem fixa e `Stale` compara só as `reads` da ação, devolvendo o delta; execução só depois de aceita ou confirmada por uma pessoa; `Receipt` com estágios e transições legais, tempo passado por quem chama; validação em dois pontos quando a verdade mora fora da UI; replay nunca cria request; `decline` recusa uma confirmação (task-58, 58b) |
| 2026-09-23 | [semantic-binding-beside-the-scene](semantic-binding-beside-the-scene.md) | accepted | tabela de ligação `SemId -> Binding { rect, layer, clip, order, view }` ao lado da cena, zerada por frame, fora do hash de layer; clip lido dos nós da própria layer, sem tocar no desenho; `hit(x, y)` devolve o id do topo; `TextField`, `Checkbox` e `Button` com `.sem(id)` e helper de projeção; o todo do showcase inteiro pelo dispatcher, fundo inerte com overlay aberto; custo medido em 50 a 60 ns por ligação (task-59 fase a); adendo da fase b: os 16 widgets interativos restantes com `SemId` e projeção, e a regra "toda ação lê o estado que ela muda" |
| 2026-09-23 | [headless-render-shares-the-frame-path](headless-render-shares-the-frame-path.md) | accepted | `GpuContext::new_headless` e o construtor da janela terminam no mesmo `from_device`; `window::draw_frame` desenha em qualquer view srgb e `present_frame` vira acquire + draw + present; `engine::headless` lê de volta para png; o probe de texto passa por ele (sai o segundo pipeline); `showcase --render` e `monster_player --frame`; frame de monster headless igual ao da janela por sha256 (task-66) |
| 2026-09-23 | [commit-checks-the-index-it-observed](commit-checks-the-index-it-observed.md) | accepted | o git client na prova da tese: um `Workbench` junta view, sessão git e dispatcher; o commit é conferido no dispatcher e de novo no worker contra o index real (`CommitIf`), com o delta real na recusa; o receipt termina com o sha e os arquivos de `git show`; recusa vira toast e o `discard` de agente espera uma pessoa num modal; resta a corrida entre a leitura do index e o `git commit` por outro processo (task-62) |
| 2026-09-23 | [accessibility-projects-the-semantic-snapshot](accessibility-projects-the-semantic-snapshot.md) | accepted | a árvore AccessKit sai do mesmo snapshot e das mesmas ligações que agente e teste usam; NodeId estável derivado do `SemId`; ação do leitor de tela entra no dispatcher como `Actor::Assistive`; foco é navegação, não capacidade; ordem de foco semântica; `Bridge` num shell de app; o caminho derivado de hit region saiu (task-60) |
| 2026-09-23 | [session-recording-replay-not-reexecution](session-recording-replay-not-reexecution.md) | accepted | o crate `rec` grava a sessão como dado num anel limitado (trilha semântica e visual) e a persiste num `.monster` de magic `MONS`, contêiner irmão do de animação: seções com sha256, trilha semântica em JSON lines, chunks visuais como bytes do monster; replay nunca cria request; segredos falham fechado (task-63) |
| 2026-09-23 | [lean-semantic-form-and-one-redaction](lean-semantic-form-and-one-redaction.md) | accepted | a forma serde do `sem` omite campo no valor padrão e lê o ausente como padrão (observação do todo de 6419 para 2039 bytes; revisão não muda; forma antiga continua lendo); `source` fica na forma para o `rec` e sai da resposta do agent port a não ser que peça (`observe {source: true}`, `plev observe --source`); a regra de redação de segredo mora uma vez só em `sem::redact`, união das duas cópias que divergiam (task-57b) |
| 2026-09-23 | [svg-export-reads-the-scene](svg-export-reads-the-scene.md) | accepted | `engine::headless::svg` escreve um elemento por nó da cena na ordem do composite, texto como contorno dos glifos que o mesmo shaping do frame produziu (um caminho só), e tudo que não é exato entra num relatório por nó; `showcase --render` ganha `--svg`, `--svg-report`, `--semantics` e `--source` (task-66b) |
| 2026-09-23 | [agents-act-through-the-semantic-contract](agents-act-through-the-semantic-contract.md) | accepted | um agente fora do processo observa e age só pelo contrato: socket unix local (0600, desligado por padrão), protocolo v1 em JSON lines, observação por escopo e por delta, `act` a partir de uma revisão vista, estágios do receipt de volta pela conexão; confirmar e recusar nunca por agente; tudo que toca o dispatcher roda na thread da UI; `plev observe` e `plev act` (task-61) |

## urnaui, o explorador de .urna (2026)

| data | adr | status | resumo |
|------|-----|--------|--------|
| 2026-09-18 | [urnaui-media-frames-in-place-via-ffmpeg-subfile](urnaui-media-frames-in-place-via-ffmpeg-subfile.md) | accepted | frame de midia decodificado no lugar: ffmpeg le o byte range do blob dentro do .urna via protocolo subfile, nada e copiado; export de blob sha256-verificado antes de gravar; ffmpeg opcional, probe por open |
| 2026-09-18 | [worker-results-wake-the-event-loop](worker-results-wake-the-event-loop.md) | accepted | resultado do worker acorda o loop winit via EventLoopProxy (nao espera o proximo mouse move); um helper de subprocess drena stdout/stderr desde o spawn, com timeout e kill (fim do hang por pipe cheio) |

## demos (2025)

| data | adr | status | resumo |
|------|-----|--------|--------|
| 2025-09-24 | [motion-trails-by-position-history](motion-trails-by-position-history.md) | accepted | trails de particula via ring buffer de posicoes, nao acumulacao de framebuffer; cena segue limpa por frame (prime) |

## documentos vivos

| arquivo | status | resumo |
|---------|--------|--------|
| [changelog](../CHANGELOG.md) | living | log de versoes commit a commit: prime creatures, workspace tiers, kdb consolidado, qa pass, ponte monster |

## notas

- duplicados conscientes: [adr-003-srp-modularization](adr-003-srp-modularization.md) (decisao formal) e [srp-modularization](srp-modularization.md) (detalhes de sessao) cobrem o mesmo limite de 300 linhas por angulos diferentes. o trio de hot reload ([hot-reload-design](hot-reload-design.md) e a pesquisa, [shader-hot-reload-adr](shader-hot-reload-adr.md) e [narrate-hot-reload-adr](narrate-hot-reload-adr.md) sao as duas decisoes).
- idioma: ADRs ate meados de 2024 estao em portugues, do fim de 2024 em diante em ingles, casando com o idioma do codigo e dos commits da respectiva onda. mantido de proposito.
- a spec do monster ([monster-format-v0](monster-format-v0.md)) e o registro tecnico do formato; a decisao por tras dela esta em [binary-animation-format-with-discovered-deltas](binary-animation-format-with-discovered-deltas.md).
