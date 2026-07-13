---
project: plev
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-11
domain: competitive
---

# analise de competidores, frameworks gui/rendering

**data:** 2026-03-11
**contexto:** pesquisa de mercado para posicionamento tecnico do plev como engine de composicao GPU-first em rust.

---

## 1. makepad (makepad/makepad), 6.222 stars, v1.0.0 (maio 2025)

**o que e:** plataforma de desenvolvimento criativo em rust com runtime GPU-first, linguagem de design live-editable, e compilacao para 6 alvos (metal, dx11, opengl, webgl, ios, android).

**arquitetura:**
- **rendering:** GPU-centric com instanced arrays. widgets que usam o mesmo shader sao agrupados no mesmo drawcall. quase zero de matrizes, so camera matrix. dados GPU gerados com posicao absoluta durante o draw. clipping via vertex-shader (sem stencil state). basicamente instanced-array drawcalls em indexed triangles.
- **layout (turtle):** layout executa *simultaneamente* ao draw, sem passo de "measure" separado. turtles sao retangulos aninhados que "caminham" pelo espaco calculando bounding boxes. apos o turtle "terminar", ele fornece bounding info ao pai, que pode mover dados GPU ja gerados (ex: alinhamento a direita = desenha esquerda, depois reposiciona).
- **shaders:** escritos em rust via macros, live-editable. compilam para multiplos backends graficos. variaveis referenciadas por nome tanto no programa quanto no shader. substituem CSS/SVG para styling e animacao.
- **estado:** hibrido immediate/retained. retained para UI estatica, immediate para updates por frame. sem virtual DOM.
- **texto:** implementacao propria (nao cosmic-text). survey de 2025 reporta que fonte default nao tem cobertura hiragana/kanji; IME funciona mas estados provisorios mostram glyphs de caractere ausente. qualidade tipografica basica.
- **alvos:** macos/metal, windows/dx11, linux/opengl, ios, android, web/webgl+WASM.
- **acessibilidade:** nenhuma. sem suporte a screen reader.

**relevancia para plev:**
- o modelo turtle (layout + draw simultaneos, sem measure pass) e uma alternativa radical ao two-phase do taffy. plev nao deve copiar, mas o conceito de mover dados GPU pos-emissao (para alinhamento) e valioso, potencialmente aplicavel em otimizacoes futuras do compositor.
- a abordagem de agrupar widgets por shader no mesmo drawcall confirma que a estrategia do plev (dois pipelines: quad + text) e valida e ate mais simples/previsivel.
- shaders live-editable sao killer feature para dx criativo, mas adicionam complexidade de compilador significativa.

**insight principal:** makepad prova que um framework rust GPU-first pode funcionar cross-platform com performance real, mas ao custo brutal de acessibilidade zero e documentacao minima. a 1.0 foi lancada em maio 2025 apos 6+ anos de desenvolvimento. o framework e construido primariamente para o time makepad, utilidade externa e coincidencia (citacao direta do survey 2025).

**limitacao:** DSL proprietaria sem documentacao publica, acessibilidade inexistente, texto com lacunas de cobertura unicode, ecossistema fechado. "built for the makepad team."

---

## 2. dioxus (dioxuslabs/dioxus), 35.236 stars, v0.7.3 (jan 2026)

**o que e:** framework fullstack cross-platform em rust com macro `rsx!`, virtual DOM, signals, e multiplos renderers (web, desktop, mobile, ssr).

**arquitetura:**
- **rendering:** multiplos backends. web: DOM via webassembly (~50kb bundle). desktop: webview (webview2/webkitgtk, "diet electron"). nativo experimental: blitz (wgpu + HTML/CSS renderer customizado). mobile: webview ou blitz experimental.
- **blitz (renderer nativo):** modular, usa stylo (CSS parsing do firefox), taffy (layout), parley (text layout), vello (2d rendering), wgpu. binario ~12mb. incrementa tree construction para performance. componentes do ecossistema servo/mozilla.
- **layout:** taffy (CSS flexbox/grid), mesmo engine usado por zed e bevy UI.
- **estado:** signals (inspirado em react + solid + svelte). `use_signal` retorna getter/setter reativo. hot-patching de codigo rust em runtime (subsegundo) desde v0.7.
- **texto:** via browser engine (webview) ou parley/cosmic-text (blitz). IME completo com estados provisorios inline. qualidade de texto excelente no modo webview (herda do browser).
- **alvos:** web (WASM), desktop (macos/linux/windows), mobile (ios/android), ssr, liveview.

**relevancia para plev:**
- blitz demonstra que construir um renderer HTML/CSS em cima de wgpu + taffy e viavel, mas o resultado e ~12mb e complexidade massiva (stylo, parley, vello, etc). plev e correto em nao tentar ser um browser engine.
- o modelo de hot-patching de rust code e revolucionario para dx. vale investigar se algo similar e possivel para o DSL narrate do plev (live reload de templates sem recompilacao completa).
- a arquitetura de signals do dioxus (combinando react/solid/svelte) valida a decisao do plev de usar readsignal/writesignal com runtime thread-local.
- taffy como layout engine e usado tanto por dioxus/blitz quanto pelo plev, confirmacao de escolha correta.

**insight principal:** dioxus e o framework rust com maior momentum (35k+ stars). mas sua proposta e fundamentalmente diferente do plev: dioxus quer ser "o react do rust" (framework de aplicacao completo), enquanto plev e a camada de rendering *abaixo* de um framework. a existencia de blitz como renderer experimental separado prova que ha demanda por engines de rendering independentes.

**limitacao:** desktop via webview traz todas as limitacoes de um browser embutido (memoria, startup time, inconsistencias de rendering entre plataformas). o renderer nativo (blitz) ainda e experimental e depende de 6+ crates externos. nao e uma engine de composicao, e um framework de aplicacao.

---

## 3. leptos (leptos-rs/leptos), 20.370 stars, v0.8.17 (mar 2026)

**o que e:** framework web full-stack isomorfico em rust com reatividade fine-grained (signals, effects, memos), ssr com hydration, e streaming http.

**arquitetura:**
- **rendering:** DOM direto, sem virtual DOM. componentes rodam uma vez, criam nos DOM reais, e configuram sistema reativo para updates. quando um signal muda, atualiza um unico text node ou classe sem nenhum outro codigo rodando.
- **reatividade:** grafo reativo push-pull. `signal(value)` retorna `(ReadSignal<T>, WriteSignal<T>)`. signals sao root nodes, effects sao leaf nodes, memos sao intermediarios. quando signal atualiza, marca dirty e propaga check aos descendentes. effects sao re-executados com frequencia minima.
- **ssr:** HTML rendering no servidor + hydration no browser. suporta streaming http (out-of-order e in-order). server functions eliminam necessidade de API rest separada.
- **layout/texto:** n/a, delega ao browser (CSS/DOM).
- **alvos:** web only. desktop nativo foi explorado ("generic rendering") mas abandonado por limitacoes do compilador com generics extensivos.

**relevancia para plev:**
- **o sistema de signals do leptos e a referencia de ouro.** readsignal/writesignal, grafo reativo com propagacao lazy (mark dirty -> propagate check -> re-evaluate on access). plev ja usa esse modelo (slotmap runtime, push-pull hybrid). confirma que a abordagem esta correta.
- o conceito de "rodar componente uma vez e configurar reatividade" vs "re-render completo" e o que plev deve seguir: montar scene graph uma vez, atualizar nodes individuais via signals.
- leptos abandonou native gui por complexidade de compilacao. isso valida que um engine separado (como plev) e necessario, frameworks web nao escalam para GPU rendering.

**insight principal:** a reatividade fine-grained do leptos (atualizar um unico text node sem re-render) e exatamente o modelo que plev deve seguir no scene graph: dirty tracking per-node, nao per-tree. o sistema de signals do plev ja implementa isso conceitualmente, mas pode aprender com a granularidade do leptos.

**limitacao:** web only. sem rendering proprio. irrelevante como competidor direto, mas extremamente relevante como referencia de arquitetura reativa.

---

## 4. yew (yewstack/yew), 32.469 stars, v0.23.0 (mar 2026)

**o que e:** framework front-end em rust para web apps multi-threaded com webassembly, inspirado em react/elm com virtual DOM.

**arquitetura:**
- **rendering:** virtual DOM com diff/patch no DOM real do browser. macro `html!` para declarar HTML interativo com expressoes rust.
- **performance:** minimiza chamadas a DOM API por render. offload de processamento para background web workers.
- **estado:** component-based (similar a react class components). hooks system. context API para state compartilhado. ecossistema de crates comunitarias para state management.
- **interop:** suporta npm packages e integracao com apps javascript existentes.
- **layout/texto:** n/a, delega ao browser (CSS/DOM).
- **alvos:** web only (WASM). targets incluem emscripten e asm.js.

**relevancia para plev:**
- yew e relevante como contra-exemplo. o modelo virtual DOM com re-render e diff e exatamente o que plev *nao* deve fazer. o overhead do vdom e mensuravel e frameworks mais recentes (leptos, solid) provaram que fine-grained reactivity supera vdom.
- o suporte a web workers para processamento pesado e um pattern interessante: plev poderia eventualmente usar workers para shaping de texto ou tessellation em WASM.

**insight principal:** yew tem stars altas (32k) por ser um dos primeiros rust WASM frameworks (desde 2017), mas momentum esta desacelerando. survey 2025 reportou problemas de release management (versoes anunciadas mas nao publicadas no crates.io). virtual DOM esta sendo superado por fine-grained reactivity em todos os ecossistemas.

**limitacao:** web only, virtual DOM com overhead inerente, sinais de manutencao irregular, nao-1.0 com breaking changes frequentes. irrelevante como competidor ou referencia tecnica para plev.

---

## 5. slint (slint-ui/slint), 21.957 stars, v1.15.1 (fev 2026)

**o que e:** toolkit gui declarativo open-source para apps nativas em rust, c++, javascript e python, com foco em embedded e desktop.

**arquitetura:**
- **DSL (.slint):** linguagem propria para descrever elementos graficos, hierarquia, property bindings, e fluxo de dados. compilada (lexing -> parsing -> 32+ passes de otimizacao -> code generation). expressoes sao funcoes puras que o compilador pode inlinar e eliminar constantes.
- **rendering (3 backends):**
  - femtovg: opengl es 2.0
  - skia: via bindings
  - software: CPU puro, zero dependencias externas
  - qt: integracao opcional para widgets nativos
- **layout:** componentes, elementos, items e propriedades em regiao unica de memoria para reduzir alocacoes. foco em footprint minimo (<300kb ram para runtime).
- **reatividade:** property<t> com change tracking e notification. dependencias registradas automaticamente durante avaliacao de bindings. re-avaliacao lazy (mark dirty, evaluate on access). operador `<=>` para two-way binding.
- **texto:** IME correto (incluindo japones). font default sem fullwidth latin/hiragana mas kanji correto pos-conversao. herda qualidade do backend (skia > femtovg > software).
- **alvos:** embedded (stm32, rp2040, raspberry pi), desktop (windows/macos/linux), web (WASM). multi-linguagem: rust, c++, javascript, python.
- **acessibilidade:** em progresso, melhorou significativamente desde 2024.

**relevancia para plev:**
- **a compilacao de DSL com 32+ passes de otimizacao e referencia para o plev_narrate.** slint prova que uma DSL compilada pode eliminar overhead de runtime significativo (inlining de propriedades constantes, dead code elimination).
- o modelo de propriedades reativas com lazy evaluation (dirty -> evaluate on access) e identico ao padrao que plev usa (fxhash dirty tracking). validacao mutua.
- o backend de software renderer (CPU, zero deps) e interessante para embedded, mas irrelevante para plev (GPU-first).
- o operador `<=>` para two-way binding e uma idea de dx elegante que plev_narrate poderia adotar.
- multi-linguagem (rust/c++/JS/python) via bindings e um modelo de distribuicao que plev deveria considerar a longo prazo.

**insight principal:** slint e o framework rust gui mais maduro (1.15.x, empresa por tras, licenciamento comercial para embedded). a separacao clara entre DSL compilada e runtime e exemplar. o foco em embedded (<300kb ram) demonstra que performance real requer decisoes de arquitetura desde o dia zero, nao e algo que se adiciona depois.

**limitacao:** licenciamento dual (gplv3 para open-source, comercial para embedded). tres backends de rendering significam tres superficies de bugs. nao e GPU-first, e GPU-optional. para plev, o modelo de "adaptar ao hardware" e o oposto da tese de "GPU everywhere."

---

## 6. ribir (ribirx/ribir), 1.665 stars, v0.4.0-alpha.60 (mar 2026)

**o que e:** framework gui nao-intrusivo em rust com wgpu como backend default, tessellation via lyon, sistema de composicao de eventos com bubbling/capture.

**arquitetura:**
- **rendering:** wgpu como GPU backend default. view -> 2d paths via painter -> tessellation (lyon) -> triangulos -> GPU. separacao entre logica de UI e rendering.
- **layout:** inspirado no flutter sublinear layout, mas com implementacao divergente. fatobj mecanismo para atributos built-in (margin, background, border, on_tap) em qualquer widget.
- **estado (nao-intrusivo):** dados do usuario sao convertidos em "estado ouvivel" (listenable state). view atualiza de acordo com mudancas no estado. estruturas de dados independentes da UI, sem camadas intermediarias.
- **widgets:** 4 tipos: function widget, compose (composicao), render (layout/paint customizado), composechild (controle pai-filho). 20+ widgets basicos, todos em estagio rough.
- **texto:** basico, tipografia e IME em estagio usavel mas rough. IME composer oculto. fonte default sem suporte kanji.
- **alvos:** desktop (linux/windows/macos) e web (WASM) em CI. ios/android compila mas UI nao adaptada.

**relevancia para plev:**
- **ribir e o competidor mais direto em termos de arquitetura** (wgpu backend, rust, cross-platform). mas com 1.6k stars e API pre-alpha, nao representa ameaca de mercado.
- o pipeline view -> painter -> tessellation (lyon) -> GPU e diferente do plev (scenenode -> quad/text buffers direto). tessellation via lyon adiciona overhead CPU que plev evita com instanced quads.
- o modelo "nao-intrusivo" (dados do usuario nao precisam implementar traits especificos) e uma boa decisao de dx que plev pode aprender: components do plev nao devem forcar o usuario a wrappear seus tipos.
- o mecanismo fatobj (atributos built-in em qualquer widget) e similar ao pattern de builder do plev, confirmacao de abordagem.

**insight principal:** ribir valida que wgpu + rust + cross-platform e viavel, mas tambem demonstra os riscos: com 60+ alpha releases e API instavel, a falta de foco (tentar ser framework completo antes de estabilizar o core) fragmenta o esforco. plev deve evitar esse anti-pattern, estabilizar engine primeiro, framework depois.

**limitacao:** pre-alpha (60+ alpha releases), API instavel, texto rough, mobile nao adaptado, documentacao desatualizada (docs para 0.2.x quando ja esta em 0.4.0-alpha). comunidade pequena.

---

## 7. compose multiplatform (jetbrains/compose-multiplatform), 18.888 stars, v1.10.0-stable / v1.11.0-alpha04 (mar 2026)

**o que e:** framework UI declarativo em kotlin para apps cross-platform (android, ios, desktop, web) com rendering via skia/skiko.

**arquitetura:**
- **rendering:** skiko (kotlin bindings para skia) em todas as plataformas. canvas-based rendering. desktop e web usam canvas direto. android usa jetpack compose nativo. ios estavel desde maio 2025 (v1.8.0).
- **web/WASM:** kotlin/wasm compilation. canvas tag renderiza compose UI no browser. `skiko.js` sendo removido (redundante para kotlin/wasm). compose for web em beta.
- **layout:** compose layout system (measure + place, similar ao flexbox mas declarativo). single-pass measurement com constraints propagation.
- **estado:** compose state management, `remember { mutableStateOf(value) }`. recomposition automatica quando state muda. modelo derivado do react mas com compilador kotlin otimizando (skip recomposition de subtrees inalteradas).
- **texto:** via skia text engine. directwrite (windows), coretext (macos), freetype+harfbuzz (linux). problemas reportados com cjk no web e emoji renderizando como quadrados. font fallback precisa ser configurado manualmente para web.
- **alvos:** android (nativo), ios (estavel), desktop jvm (windows/macos/linux), web (kotlin/wasm beta, kotlin/JS). total: 5 plataformas.
- **hot reload:** compose hot reload estavel desde v1.10.0.

**relevancia para plev:**
- **skia como rendering engine universal e a abordagem mainstream.** flutter, compose, e outros usam skia. plev escolheu wgpu diretamente (sem skia), isso e mais trabalho mas da controle total sobre o pipeline e elimina a dependencia de ~15mb do skia.
- o modelo de recomposition do compose (skip subtrees inalteradas) e analoga ao dirty tracking por layer do plev. a diferenca: compose opera em arvore de componentes, plev opera em scene graph de render nodes.
- problemas de texto cjk/emoji no web demonstram que mesmo com skia, text rendering cross-platform e dificil. plev com cosmic-text tem os mesmos riscos.
- compose prova que WASM via canvas (nao DOM) e viavel para UI frameworks. plev faz o mesmo com webgpu.

**insight principal:** compose multiplatform e a referencia de "como fazer cross-platform comercial certo", empresa dedicada (jetbrains), budget de engenharia massivo, kotlin como linguagem de alto nivel. o fato de *ainda* ter problemas com texto cjk no web e uma validacao de que este problema e genuinamente dificil, nao uma limitacao do plev.

**limitacao:** kotlin (nao rust), jvm dependency para desktop, binarios grandes, skia como dependencia pesada. modelo completamente diferente do plev. nao e competidor direto, e referencia de produto.

---

## padroes cross-cutting

### 1. reatividade convergiu para signals
todos os frameworks modernos (dioxus, leptos, slint, ribir, compose) usam alguma forma de reatividade fine-grained. virtual DOM (yew) esta sendo abandonado. o modelo de signals com readsignal/writesignal, dirty tracking, e lazy evaluation e o padrao dominante. plev ja implementa isso.

### 2. layout engines sao comoditizados
taffy aparece em dioxus (blitz), plev, zed, e bevy. CSS flexbox/grid como modelo de layout e o padrao de facto. nenhum framework inventou um layout engine superior, exceto makepad (turtle), que sacrifica flexibilidade por performance.

### 3. texto cross-platform continua sendo o problema mais dificil
todos os frameworks reportam problemas com cjk, emoji, ou IME. mesmo compose (com skia e budget de engenharia da jetbrains) tem issues abertos. nao existe solucao perfeita, cosmic-text, parley, harfbuzz, e skia text sao as melhores opcoes disponiveis e todas tem lacunas.

### 4. gpu-first e minoria
apenas makepad e plev sao genuinamente GPU-first. slint e GPU-optional (3 backends). dioxus/blitz e GPU via wgpu mas como browser engine. ribir usa wgpu mas com tessellation CPU (lyon). a maioria dos frameworks delega rendering ao browser (DOM) ou ao OS (webview). GPU-first com wgpu nativo e um diferencial real.

### 5. DSL propria e arriscada mas poderosa
makepad, slint, e plev (plev_narrate) tem dsls proprias. dioxus usa `rsx!` (JSX-like). leptos usa `view!` (similar). o risco: DSL sem documentacao mata adocao (caso makepad). o beneficio: DSL compilada pode otimizar (caso slint, 32+ passes). plev deve priorizar documentacao do plev_narrate.

### 6. acessibilidade e o elefante na sala
makepad: zero. ribir: nao documentado. slint: em progresso. dioxus (webview): herda do browser (excelente). dioxus (blitz): via accesskit. yew/leptos: herda do browser. compose: via plataforma nativa. para frameworks GPU-first que renderizam tudo customizado, acessibilidade requer integracao com accesskit ou equivalente, e trabalho significativo que plev precisara enderecar.

### 7. hot reload/patching e expectativa de dx
makepad (live shader editing), dioxus (hot-patching de rust), compose (hot reload), slint (live preview). frameworks modernos esperam iteracao sem recompilacao completa. plev deve considerar hot reload do plev_narrate como feature futura.

---

## implicacoes para plev

### recomendacoes concretas

1. **manter GPU-first como diferencial.**
   apenas makepad compete nesse espaco, e makepad tem problemas graves de acessibilidade e documentacao. plev com wgpu (6 backends via abstraction) e arquitetura mais limpa (2 pipelines vs n shaders agrupados por tipo) esta bem posicionado.

2. **nao tentar ser framework de aplicacao.**
   dioxus (35k stars) ja ocupa esse espaco com ssr, routing, state management completo. plev deve ser a *engine de rendering* que frameworks como dioxus poderiam usar como backend (analogo a como blitz/vello sao usados). o posicionamento correto e: "wgpu rendering engine for rust UI frameworks."

3. **investir pesado em texto.**
   todos os competidores lutam com texto. cosmic-text + atlas de glyph do plev e uma abordagem solida, mas precisa de: (a) fallback fonts para cjk/emoji, (b) teste extensivo de IME em todas as plataformas, (c) benchmarks comparativos com skia text.

4. **documentar plev_narrate obsessivamente.**
   makepad morreu (em adocao) pela falta de documentacao da DSL. slint prosperou (22k stars, empresa, clientes) porque a .slint language tem docs completos. cada keyword, cada modifier, cada pattern do plev_narrate precisa de documentacao com exemplos.

5. **accesskit como prioridade futura.**
   quando plev sair de alpha, acessibilidade sera blocker para adocao real. accesskit (usado por egui, dioxus/blitz, slint parcialmente) e o caminho. planejar integracao desde agora (reservar ids, manter arvore de nodes acessivel).

6. **considerar hot reload do DSL.**
   makepad e dioxus provaram que live editing e killer feature para dx. para plev_narrate, considerar: recarregar templates .plev sem recompilar o engine. isso poderia ser o diferencial de dx que atrai contribuidores.

7. **dirty tracking por layer esta correto.**
   confirmado por compose (skip recomposition de subtrees), slint (lazy property evaluation), leptos (fine-grained signal updates). o modelo do plev (fxhash per-layer, unchanged layer = zero GPU work) e state-of-the-art.

8. **evitar o anti-pattern do ribir.**
   60+ alpha releases sem API estavel e sem documentacao atualizada. plev deve estabilizar o core engine (compositor + text + effects + input) antes de expandir para widgets e framework features. profundidade > amplitude.

### posicionamento competitivo

| aspecto | plev | makepad | dioxus | slint |
|---------|------|---------|--------|-------|
| rendering | wgpu (6 backends) | metal/dx11/gl/webgl | webview + wgpu experimental | femtovg/skia/software |
| GPU-first | sim | sim | nao | nao |
| layout | taffy (flexbox) | turtle (custom) | taffy (via blitz) | custom (compilado) |
| texto | cosmic-text + atlas | custom (lacunas) | browser/parley | backend-dependente |
| DSL | plev_narrate (proc-macro) | live DSL (macros) | rsx! (JSX-like) | .slint (compilada) |
| signals | readsignal/writesignal | hibrido imm/retained | use_signal | property<t> bindings |
| maturidade | alpha | 1.0 (maio 2025) | 0.7.3 | 1.15.1 |
| acessibilidade | nenhuma (planejada) | nenhuma | via webview/accesskit | em progresso |
| stars | ~0 (privado) | 6.2k | 35.2k | 22.0k |

### o nicho do plev

plev ocupa um nicho unico: **engine de composicao GPU-first em rust via wgpu com scene graph, dirty tracking por layer, e premultiplied alpha pipeline**. nenhum dos competidores oferece isso como biblioteca standalone. makepad e o mais proximo, mas e um framework completo (nao uma engine). blitz (dioxus) e um browser engine. slint e um toolkit. plev e a *camada mais baixa*, o que fica entre wgpu e qualquer framework UI.

o modelo mental: **plev e para rust UI o que skia e para flutter/chrome**, engine de rendering que outros frameworks consomem.
