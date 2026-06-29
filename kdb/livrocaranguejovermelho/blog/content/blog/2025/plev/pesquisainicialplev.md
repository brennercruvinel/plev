+++
authors = ["Brenner Cruvinel"]
title = "Pesquisa inicial PLEV: das primitivas reativas ao layout"
description = "Mapa das camadas de abstração em interfaces, de state management reativo a layout e renderização, como base para o projeto PLEV."
date = 2025-04-05
[taxonomies]
tags = ["PLEV", "UI", "Reatividade", "Pesquisa"]
+++

1. Reactive Primitives / State Management

Essa é a camada mais baixa de abstração conceitual. Não renderiza nada, não calcula layout, não desenha pixels. É puramente o sistema de propagação de mudanças de estado pela aplicação.

O conceito central é reactivity: quando um valor muda, tudo que depende dele recomputa automaticamente. Existem dois paradigmas fundamentais. O primeiro é pull-based (lazy): o consumer pede o valor quando precisa, e o sistema marca dependências como "dirty" até serem lidas (Leptos signals, SolidJS signals, Svelte 5 runes). O segundo é push-based (eager): a mudança propaga imediatamente para todos os subscribers no momento da mutação (MobX, RxJS, Redux via dispatch). Alguns sistemas são híbridos.

As primitivas típicas são: Signal (valor reativo atômico, leitura e escrita), Computed/Derived (valor derivado de outros signals, memoizado), Effect (side-effect que re-executa quando suas dependências mudam). Esse trio (signal, computed, effect) é o padrão de facto hoje e aparece com nomes diferentes em cada framework.

Atributos técnicos relevantes: granularidade do tracking (por componente como React, ou por signal individual como SolidJS/Leptos), push vs pull, batching de updates (React batcha automaticamente, signals geralmente não precisam), glitch-free propagation (garantia de que valores intermediários inconsistentes nunca são observados), e memory management (garbage collection automático ou ownership manual como em Rust/Leptos).

Implementações chave: SolidJS signals, Leptos signals (Rust), Svelte 5 runes ($state, $derived, $effect), Vue 3 reactivity (@vue/reactivity, pode ser usado standalone), Preact signals (@preact/signals), MobX (proxy-based, observable/computed/reaction), Jotai (atomic state para React), Zustand (flux simplificado para React), Redux (store centralizado, action/reducer, push-based), RxJS (streams reativos, paradigma completamente diferente baseado em observables e operators). Recoil foi descontinuado pelo Meta.

O ponto crucial para sua documentação: alguns frameworks embutem a reatividade (Svelte, SolidJS, Leptos, Iced) e outros consomem bibliotecas externas (React é agnóstico, consome useState/useReducer internamente mas pode usar qualquer state manager externo). A tendência da indústria é convergir para fine-grained reactivity com signals, abandonando o modelo de re-render por componente inteiro do React.

2. Layout Engine

Resolve exclusivamente o cálculo de posição e tamanho dos elementos. Recebe uma árvore de nós com constraints (width, height, flex, padding, margin, etc) e produz uma árvore com coordenadas absolutas (x, y, width, height) para cada nó. Não renderiza, não estiliza, não processa eventos.

O algoritmo dominante é Flexbox, originado do CSS mas reimplementado fora do browser por Yoga (C++, criado pelo Facebook, usado pelo React Native e Litho) e Taffy (Rust, usado pelo Dioxus, Bevy UI, e Lapce). Taffy também implementa CSS Grid, que o Yoga não suporta. Flexbox resolve layout unidimensional (main axis + cross axis), Grid resolve bidimensional.

Outros modelos de layout: Morphorm (Rust, layout system do Vizia, modelo mais simples que flexbox), Layout (Rust, implementação mínima de flexbox), Cassowary constraint solver (usado internamente pelo Auto Layout da Apple, resolve constraints arbitrárias via algoritmo simplex, mais expressivo mas mais caro computacionalmente), e Auto Layout do Apple (não é open source, usa Cassowary internamente, é o layout engine do UIKit/AppKit/SwiftUI).

Atributos técnicos: modelo de layout suportado (flexbox, grid, absolute, constraint-based), performance em árvores profundas (Taffy e Yoga são O(n) no caso comum, Cassowary é O(n²) no pior caso), incremental layout (recalcular só o que mudou, crítico para UI reativa), thread safety (Taffy é thread-safe, Yoga historicamente não), e suporte a text measurement callback (o layout engine precisa perguntar ao text shaper "qual o tamanho desse texto?" para calcular o layout, isso é sempre um callback externo).

Na web, o layout engine é o browser (Blink, WebKit, Gecko) e você não o substitui. No native, o layout engine é uma dependência explícita que o UI framework integra. Nos IMGUI frameworks (egui, Dear ImGui), o layout é trivial e embutido: cada widget reporta seu tamanho e o framework faz stacking linear (horizontal/vertical), sem flexbox.

3. 2D Render Engine / Scene Graph

Camada de rasterização vetorial. Recebe comandos de desenho (retângulo, arco, path, glyph, gradiente, blur, clip) e produz pixels. Pode rasterizar para CPU (bitmap em memória) ou GPU (via shaders).

Skia (C++, Google) é o engine dominante. Usado internamente por Chrome/Chromium, Android, Flutter, e dezenas de outros projetos. Rasteriza via CPU (software renderer) ou GPU (OpenGL, Vulkan, Metal, Direct3D). É extremamente maduro e completo, mas é uma dependência pesada (dezenas de MB de binário). Flutter está migrando do Skia para o Impeller, um backend GPU-first otimizado para mobile que elimina shader compilation jank em runtime.

Vello (Rust, projeto Linebender/Google Fonts) é o engine experimental de próxima geração. Compute shader first: toda a rasterização é feita na GPU via compute shaders, sem pipeline de rasterização tradicional. Altamente paralelo, potencialmente mais rápido que Skia para cenas complexas, mas ainda em desenvolvimento ativo e não production-ready para todos os casos. Usado pelo Xilem (framework experimental do Linebender).

Cairo (C) é o engine clássico do ecossistema Linux/GNOME. Usado internamente pelo GTK. Predominantemente CPU, tem backend OpenGL mas é menos otimizado que Skia. Maduro e estável mas performance inferior.

tiny-skia (Rust) é um subset de Skia reimplementado em Rust puro, software-only (CPU). Usado pelo Iced no backend CPU. Sem dependências nativas, compilação trivial, mas sem aceleração GPU.

Piet (Rust, Linebender) é uma abstração sobre backends de rendering (Direct2D no Windows, Core Graphics no macOS, Cairo no Linux). A ideia é escrever código de desenho uma vez e usar o renderer nativo de cada plataforma. Está sendo substituído pelo Vello no ecossistema Linebender.

wgpu não é um render engine 2D, é uma abstração sobre APIs gráficas (Vulkan, Metal, DX12, OpenGL). Iced usa wgpu como backend GPU, mas o rendering 2D é implementado pelo próprio Iced em cima do wgpu.

Atributos técnicos: backend de rasterização (CPU, GPU, ou ambos), APIs gráficas suportadas (Vulkan, Metal, DX12, OpenGL/ES, WebGPU), anti-aliasing (MSAA, analytic AA), suporte a text shaping/rendering (Skia tem o HarfBuzz embutido, outros delegam), suporte a SVG (Skia via modulo separado, resvg em Rust é uma alternativa), suporte a blur/shadows/compositing (essencial para UIs modernas, nem todos os engines suportam eficientemente), e tamanho do binário.

4. Styling Engine

No mundo web, é a camada que resolve como CSS é escrito, processado e aplicado. Não existe como conceito separado no mundo native (lá, estilização é parte do UI framework).

Utility-first / Atomic CSS: Tailwind CSS é o exemplo canônico. Cada classe CSS faz exatamente uma coisa (bg-blue-500, p-4, flex, rounded-lg). Não escreve CSS customizado, compõe classes utilitárias. O build tool (PostCSS ou Rust-based Oxide engine no Tailwind v4) faz tree-shaking e gera só o CSS usado. UnoCSS é a alternativa mais rápida (engine baseado em regex, on-demand). O tradeoff é legibilidade do HTML (classes longas) em troca de zero CSS morto e zero naming conflicts.

CSS-in-JS (runtime): styled-components, Emotion. Gera CSS em runtime via JavaScript, injeta no DOM. Permite CSS dinâmico baseado em props. O tradeoff fundamental é performance: runtime overhead no browser pra computar e injetar estilos. A tendência é abandonar esse modelo.

CSS-in-JS (zero-runtime / build-time): Vanilla Extract, Panda CSS, Linaria, StyleX (Meta). CSS é extraído em build time para arquivos .css estáticos. Combina a ergonomia de CSS-in-JS (TypeScript, type-safe) com a performance de CSS estático. Panda CSS é utility-first mas type-safe. StyleX é o sistema interno do Facebook, otimizado para atomic CSS com deduplicação agressiva.

CSS Modules: escoping automático de classes CSS por componente. Não é um framework, é uma feature de bundlers (Vite, webpack). Cada arquivo .module.css gera classes com hash único, evitando conflitos globais.

Preprocessors: Sass/SCSS, Less, PostCSS. Camada acima do CSS puro que adiciona variáveis, nesting, mixins, functions. PostCSS é especial porque é um pipeline de transformação plugável (Tailwind roda como plugin PostCSS). Estão perdendo relevância conforme CSS nativo ganha features (nesting nativo, @layer, custom properties).

Atributos técnicos: runtime vs build-time (impacto direto em performance), type-safety (TypeScript integration), atomic output (deduplicação de CSS), co-location (CSS junto ao componente vs arquivo separado), theming system (CSS custom properties, theme tokens), critical CSS extraction (extrair só o CSS above-the-fold), e framework coupling (Tailwind é agnóstico, styled-components exige React).

5. Headless UI Libraries

Componentes que implementam comportamento, acessibilidade e state management sem nenhum estilo visual. Fornecem a lógica de um dropdown, modal, combobox, tabs, toast, etc, mas zero CSS. Você aplica sua própria estilização.

Radix UI (React) é o mais completo. Cada primitivo é WAI-ARIA compliant, gerencia focus trap, keyboard navigation, screen reader announcements. shadcn/ui é construído em cima do Radix. Controlado (controlled) e não-controlado (uncontrolled) para cada componente.

Headless UI (Tailwind Labs) existe para React e Vue. Menos componentes que Radix, mais simples. Feito especificamente para ser estilizado com Tailwind.

Ark UI (Chakra team) é multi-framework: React, Vue, Solid. Construído sobre uma state machine layer (Zag.js) que é framework-agnostic. A state machine roda independente e cada framework adapter conecta a UI.

Melt UI é o equivalente para Svelte. Usa Svelte stores e actions para prover comportamento headless.

Kobalte é o equivalente para SolidJS. WAI-ARIA compliant, fine-grained reactivity.

Atributos técnicos: cobertura de componentes (quantos primitivos oferece), compliance com WAI-ARIA (acessibilidade é o core value), composability (render props, slots, compound components), portal support (renderizar modal fora da árvore DOM principal), animation hooks (callbacks para enter/exit transitions), collision-aware positioning (floating-ui/popper integration para dropdowns/tooltips), e framework lock-in (Radix é React-only, Ark UI é multi-framework).

No mundo native, esse conceito não existe como camada separada. A acessibilidade é responsabilidade do UI framework ou do OS.

6a. UI Framework: Immediate Mode GUI (IMGUI)

Paradigma onde a UI inteira é redesenhada a cada frame. Não existe árvore de widgets persistente em memória. A função de UI é chamada a cada frame (tipicamente 60fps), e o framework decide o que precisa re-renderizar internamente via diffing de draw commands.

O loop conceitual é: ler input → chamar função de UI que emite draw commands → renderizar frame. O estado vive inteiramente na aplicação, não no framework. Um botão não "existe" como objeto entre frames; ele é apenas o resultado de chamar ui.button("Click me") naquele frame.

Dear ImGui (C++) é o original e mais usado. Focado em tooling, debug overlays, editors de jogos. Extensões comunitárias massivas (node editors, plot widgets, docking, etc). Rendering via seu próprio draw list que pode ser integrado com qualquer backend gráfico (OpenGL, Vulkan, DX11, DX12, Metal, wgpu). Não tem acessibilidade. Não é adequado para aplicações end-user.

egui (Rust) é o equivalente em Rust. Puro Rust, sem dependências unsafe. Backend padrão é eframe (wrappers sobre winit + wgpu/glow). Pode rodar em WASM no browser. Layout engine embutido (simples, horizontal/vertical stacking, nada de flexbox). Tem mais primitivos de UI que Dear ImGui (text edit multi-line, color picker, plots). Suporte a acessibilidade está em desenvolvimento via AccessKit.

Atributos técnicos relevantes pra IMGUI: frame rate (UI só atualiza quando algo muda em modo reativo, ou a cada frame em modo contínuo, egui suporta ambos), statelessness (vantagem para hot-reload e serialização, desvantagem para widgets complexos como text editors), immediate feedback (zero latência entre input e resposta visual), GPU integration (trivial embutir IMGUI sobre um viewport 3D), text rendering (egui usa epaint com tessellation própria, Dear ImGui usa stb_truetype), layout (não tem flexbox, grid, ou constraint-based, apenas stacking sequencial com spacing), e persistência de estado implícito (IDs gerados por label/posição, colisões de ID são um bug comum).

Tradeoff fundamental: máxima simplicidade de API e integração, mínima capacidade de layout complexo e acessibilidade.

6b. UI Framework: Native Widget Toolkit (Retained Mode, Imperativo)

Paradigma clássico. Existe uma árvore de objetos widget em memória que persiste entre frames. Você cria widgets, configura propriedades, registra callbacks, e o framework gerencia o ciclo de vida, rendering e event dispatch. Mutação direta: button.setText("Click"), window.add(panel).

Qt (C++, com bindings para Python/PyQt/PySide, Rust/CXX-Qt, etc) é o mais completo e maduro. É efetivamente um OS de UI: tem seu próprio event loop, threading model, networking stack, SQL abstraction, OpenGL integration, multimedia, XML parser, regex engine, containers (QString, QList, QMap). O widget system (QWidget) usa rendering nativo via QPainter. Qt Quick/QML é uma camada declarativa separada sobre Qt (ver seção 6c). Licenciamento dual: LGPL (open source) e comercial. O sistema de sinais e slots (signal/slot) é o mecanismo de comunicação entre objetos, compilado pelo MOC (Meta-Object Compiler), um preprocessor que gera código C++ extra. Cross-platform: Linux, Windows, macOS, Android, iOS, embedded.

GTK (C, com bindings para Rust/gtk-rs, Python/PyGObject, etc) é o toolkit do GNOME. GTK4 é a versão atual, rendering via GPU (Vulkan/OpenGL) com scene graph internal (GSK). Layout system flexbox-like (GtkBox, GtkGrid). Theming via CSS (sim, CSS real). Acessibilidade via ATK/AT-SPI. Funciona fora do Linux (Windows, macOS) mas a experiência é subótima (não usa widgets nativos do OS, renderiza tudo próprio). GObject type system (runtime type info, reference counting, signals) é a base, implementado em C com macros pesadas.

wxWidgets (C++) é o que mais se aproxima de usar widgets verdadeiramente nativos do OS. No Windows usa Win32 controls, no macOS usa Cocoa, no Linux usa GTK. Resultado: a UI parece nativa de verdade em cada plataforma, mas o subset de features é o mínimo denominador comum entre plataformas.

FLTK (C++, bindings Rust/fltk-rs) é minimalista. Binário pequeno (< 1MB), rendering próprio, API simples. Não usa widgets do OS. Ideal para aplicações embarcadas ou ferramentas internas onde tamanho importa.

Cocoa/AppKit (Objective-C/Swift, macOS only) e UIKit (Objective-C/Swift, iOS only) são os toolkits nativos da Apple. NSView/UIView hierarchy, delegate pattern, target-action para eventos. SwiftUI (seção 6c) é o sucessor declarativo.

Win32/WinAPI (C, Windows only) é o API nativo do Windows. Message loop, window procedures (WndProc), GDI para rendering. Extremamente low-level, nenhuma abstração, mas máximo controle e mínimo overhead.

Atributos técnicos comuns: widget hierarchy (árvore de objetos em memória), ownership model (parent-child, o parent gerencia o ciclo de vida dos filhos), event dispatch model (event bubbling/propagation pela árvore), threading model (quase todos exigem que UI rode na main thread, com mecanismos de cross-thread communication: Qt signals, GTK idle_add, etc), rendering (nativo do OS vs custom renderer vs GPU-accelerated), layout system (manual absolute positioning, box layout, grid, ou constraint-based), theming (GTK usa CSS, Qt usa QSS, AppKit/UIKit usam appearance API), internationalization (BiDi text, input methods, locale), drag-and-drop, clipboard, system tray integration.

6c. UI Framework: Declarative Native (Retained Mode, Declarativo)

Geração recente que combina o modelo declarativo dos web frameworks com rendering nativo. Você descreve o que a UI deve ser em função do estado, e o framework reconcilia as mudanças. A diferença do IMGUI é que aqui existe uma árvore retida que é diffed/patched, não redesenhada do zero.

SwiftUI (Swift, Apple platforms) é a referência do paradigma. View é um struct value type que retorna um body descrevendo a UI. Property wrappers (@State, @Binding, @ObservedObject, @EnvironmentObject, @StateObject) gerenciam reatividade. O framework faz diffing da view hierarchy e aplica updates incrementais. Rendering via Core Animation/Metal. Layout system próprio (não é flexbox): stacks (HStack, VStack, ZStack), alignment guides, geometry reader. Interop com UIKit/AppKit via UIViewRepresentable. Disponível apenas para Apple platforms (iOS 13+, macOS 10.15+).

Jetpack Compose (Kotlin, Android) é o equivalente Google. Composable functions anotadas com @Composable. State management via remember { mutableStateOf() }. Compiler plugin do Kotlin transforma composables em código otimizado com slot tables para recomposição incremental. Rendering via Skia (Android Canvas). Layout system custom (não flexbox): Row, Column, Box, com modifier chains. Compose Multiplatform (JetBrains) extende para desktop e iOS.

Flutter (Dart, Google) é cross-platform com rendering totalmente próprio. Não usa nenhum widget nativo do OS. Renderiza via Skia (migrando para Impeller). Widget tree → Element tree → RenderObject tree é a hierarquia interna (três árvores). Layout system baseado em constraints (parent passa constraints para child, child retorna size). Hot reload é um diferencial (state preserving reload em ~1 segundo). Compila para ARM native (AOT) em produção, JIT em desenvolvimento.

Iced (Rust) segue a Elm Architecture: Model (estado), Message (eventos), update (lógica), view (UI). Puro funcional no design: view é uma função pura do Model que retorna uma árvore de widgets. O framework faz diffing e re-renderiza só o necessário. Backend rendering via wgpu (GPU) ou tiny-skia (CPU). Layout via flexbox interno. Subscriptions para eventos assíncronos (timers, websockets, etc). Ainda em desenvolvimento ativo, API instável entre versões.

Dioxus (Rust) é multi-target: web (WASM), desktop (webview via sistema), mobile, TUI. Sintaxe RSX (semelhante a JSX). Signals para reatividade (fine-grained, similar a SolidJS). Quando roda em modo desktop, usa webview (como Tauri), não rendering nativo. Tem um modo "native" experimental via WGPU mas não é o padrão. Layout via Taffy (flexbox + grid). Virtual DOM com diffing.

Slint (Rust/C++/JS) tem DSL própria (.slint files) compilada ahead-of-time. A DSL é declarativa, tipada, com property bindings reativos resolvidos em compile time. Compila para native rendering (Skia, software renderer, ou FemtoVG/OpenGL). Foco em embedded/IoT (microcontrollers com display). Licensing dual: GPL free, comercial pago. Tooling inclui live preview e design tool visual (Slint Design).

GPUI (Rust) é o framework interno do editor Zed. Não é um framework de uso geral (não é publicado como crate standalone). Rendering direto via Metal (macOS) / Vulkan (Linux) / DX12 (Windows). Otimizado para text rendering extremamente rápido (editor de código). Flexbox layout. Entity system para state management. Async-first com integração ao executor async do Zed.

Atributos técnicos comuns: declarative view description, diffing/reconciliation algorithm (virtual DOM, slot tables, ou tree diffing), state management model (signals, Elm Architecture, property wrappers), rendering backend (nativo do OS, Skia, wgpu, custom), layout system (flexbox, constraint-based, custom), hot reload support, compilation model (JIT para dev / AOT para prod, ou always AOT), platform targets, acessibilidade (SwiftUI e Compose herdam do OS, Iced está implementando AccessKit, GPUI tem suporte parcial), animation system (SwiftUI tem .animation/.withAnimation, Compose tem animateAsState, Flutter tem Tween/AnimationController, Iced é limitado), e interop com toolkits nativos.

6d. UI Framework: Web (Declarativo)

Frameworks que rodam no browser, renderizando via DOM. O browser é o render engine (Blink, WebKit, Gecko), o layout engine (CSS), o event system, e o accessibility layer. O framework web é uma camada de abstração sobre essas APIs.

React (JavaScript/TypeScript, Meta) popularizou o modelo declarativo com Virtual DOM. Components são funções que retornam JSX. Hooks (useState, useEffect, useMemo, useCallback, useRef, useContext) gerenciam state e side effects. O Virtual DOM é um diffing layer: React compara a árvore anterior com a nova e aplica patches mínimos ao DOM real. Re-render por componente inteiro (não fine-grained). Concurrent Mode / React 18+ introduz rendering interruptível e transitions. React Server Components (RSC) permitem componentes que rodam exclusivamente no servidor. Ecossistema massivo.

SolidJS (JavaScript/TypeScript) parece React (JSX, components como funções) mas a arquitetura é fundamentalmente diferente. Sem Virtual DOM. Compilador transforma JSX em DOM operations diretas. Reactivity é fine-grained via signals: quando um signal muda, só o DOM node que depende dele atualiza. Components rodam uma vez (não re-executam como em React). Performance comparável a vanilla JS. Ecossistema menor.

Svelte (JavaScript/TypeScript) é um compiler-first framework. Componentes .svelte são compilados em JavaScript imperativo que manipula o DOM diretamente. Sem Virtual DOM, sem runtime reativo pesado. Svelte 5 introduziu runes ($state, $derived, $effect) como modelo de reatividade, substituindo o modelo "reativo por assignment" das versões anteriores. Output é JS minimal, bundles menores que React/Vue.

Vue (JavaScript/TypeScript) combina template syntax (HTML-like com diretivas v-if, v-for, v-bind, v-model) com Composition API (ref, computed, watch, similar a React hooks). Proxy-based reactivity (observa mutações em objetos via Proxy). Template compiler otimiza para pular re-renders de conteúdo estático. Ecossistema coeso (Vue Router, Pinia/Vuex, Vite é criado pelo mesmo autor).

Angular (TypeScript, Google) é o mais "opinionated" e enterprise. Inclui tudo: router, forms, HTTP client, dependency injection, RxJS para streams reativos. Signals foram introduzidos recentemente como alternativa ao Zone.js change detection. Ahead-of-time compilation dos templates. Standalone components simplificaram a API (antes exigia NgModules).

Lit (TypeScript, Google) é para Web Components nativos. Cada componente é um custom element (extends HTMLElement). Shadow DOM para encapsulamento de estilo. Tagged template literals para templates. Muito leve (~5KB). Interop nativo com qualquer framework porque são standard Web Components.

Leptos (Rust → WASM) roda no browser via WebAssembly. Fine-grained reactivity com signals (similar a SolidJS). Sem Virtual DOM. Suporta SSR com hydration. Compila Rust para WASM, o que significa que a lógica de UI roda em WASM e as interações com DOM são via JS interop. Performance excelente para computação pesada no client.

Atributos técnicos: rendering strategy (Virtual DOM diffing vs fine-grained DOM updates vs compiler-generated imperative code), reactivity model (signals vs hooks vs proxy vs compiler magic), SSR/SSG capability, hydration strategy (full hydration, partial, resumable como Qwik), bundle size (Svelte e SolidJS produzem os menores, Angular o maior), TypeScript integration (Angular é TypeScript-first, os outros são TypeScript-optional), component model (functions vs classes vs SFCs), styling approach (CSS Modules, scoped styles, styled-components, Tailwind, tudo é possível), e compilation model (Svelte e SolidJS são compiler-heavy, React é runtime-heavy).

7. Meta-framework

Camada acima do UI framework que resolve infraestrutura de aplicação web: routing, server-side rendering, data fetching, bundling, deployment. Sem um meta-framework, você tem um SPA client-side puro. Com ele, você ganha SSR, SSG, API routes, file-based routing, e otimizações de produção.

Next.js (sobre React, Vercel) é o dominante. App Router (React Server Components, nested layouts, streaming SSR) é o modelo atual. Pages Router é o legado. File-based routing (cada arquivo em app/ é uma rota). Server Actions para mutations. Middleware para edge logic. Deploy otimizado para Vercel mas roda em qualquer Node.js server. Pode fazer SSG (static), SSR (dynamic), ISR (incremental static regeneration), ou client-side rendering, por rota individual.

SvelteKit (sobre Svelte) resolve os mesmos problemas para Svelte. File-based routing, load functions para data fetching, form actions para mutations, adapters para deploy em diferentes plataformas (Node, Vercel, Cloudflare Workers, static). Mais simples que Next.js em escopo.

Nuxt (sobre Vue) é o equivalente para Vue. Auto-imports, file-based routing, server routes (Nitro engine), módulos para extensibilidade. Nitro é o server engine que compila para múltiplos targets (Node, Deno, Cloudflare, Bun).

Remix (sobre React, Shopify) foca em web standards. Nested routes com loaders (GET) e actions (POST/mutations). Inspirado no modelo mental do HTML forms. Streaming SSR. Menos magic que Next.js, mais explícito. Está convergindo com React Router v7.

Astro é framework-agnostic para content sites. Islands architecture: a página é HTML estático por padrão, e componentes interativos (React, Svelte, Vue, Solid, qualquer um) são "ilhas" que hydratam independentemente. Ideal para blogs, docs, marketing sites. Zero JS por padrão no client.

Tanstack Start (sobre React) é o mais recente, focado em full-stack type-safety (Vinxi server, tRPC-like server functions). Ainda em beta.

Atributos técnicos: rendering modes suportados (SSR, SSG, ISR, SPA, streaming), routing model (file-based vs config-based), data fetching pattern (loaders, server components, getServerSideProps), deployment targets (Node.js, edge/serverless, static), build tool (Vite para SvelteKit/Nuxt/Astro/Remix, Turbopack/webpack para Next.js), caching strategy, middleware system, e API routes/server functions.

8. Component Library / Design System

Coleções de componentes prontos com estilo visual, layout, e comportamento. Diferente de headless UI (que não tem estilo), component libraries vêm com design opinion.

shadcn/ui (React) é o caso especial: não é uma dependência npm, é uma CLI que copia código-fonte dos componentes pro seu projeto. Construído sobre Radix UI (headless) + Tailwind CSS (estilo). Você tem ownership total do código, pode modificar qualquer componente. Extremamente popular por dar controle sem lock-in.

Material UI / MUI (React) implementa Material Design do Google. Componentes completos com theming system, responsive layout, icons. É uma dependência pesada. Emotion (CSS-in-JS runtime) internamente. Altamente customizável mas o baseline é Material Design.

Ant Design (React) é o equivalente chinês, dominante no mercado asiático. Extremamente completo (100+ componentes), focado em admin dashboards e enterprise. Design language próprio.

Chakra UI (React) foca em acessibilidade e composability. Style props diretamente nos componentes (<Box p={4} bg="blue.500">). Baseado em tokens de design.

Park UI é shadcn/ui mas multi-framework (React, Vue, Solid), construído sobre Ark UI (headless multi-framework).

DaisyUI é uma component library sobre Tailwind. Adiciona classes semânticas (btn, card, modal) em cima das utility classes do Tailwind. Não é framework-specific (funciona com qualquer coisa que use Tailwind).

No mundo native, o conceito de component library é diferente porque os toolkits já vêm com seus próprios widgets. SwiftUI tem seus próprios components (List, NavigationStack, Sheet), Qt tem QWidgets e Qt Quick Controls, GTK tem GtkButton/GtkEntry/etc. A "component library" no native é o próprio toolkit.

Atributos técnicos: distribuição (npm package vs copy-paste vs CLI scaffold), base headless (Radix, Ark, Headless UI), styling approach (CSS-in-JS, Tailwind, CSS Modules, style props), theming system (CSS variables, design tokens, theme provider), acessibilidade (WCAG compliance level), componentes disponíveis (quantidade e completude), bundle size impact, e framework coupling.

9. TUI Framework (Terminal UI)

Frameworks que renderizam UI no terminal usando caracteres, cores ANSI, e posicionamento de cursor. O terminal é o render target: cada "pixel" é uma célula de caractere.

BubbleTea (Go, Charm) segue a Elm Architecture: Model, Update, View. Model é o estado, Update processa mensagens (key presses, timer ticks, I/O results), View retorna uma string que é o render do frame. Puro funcional. Composição via sub-models. Async commands retornam Msgs. Lip Gloss (Charm) é o styling library (cores, borders, layout alignment).

Ratatui (Rust) é retained mode para terminal. Immediate mode rendering (a cada frame você desenha todos os widgets), mas os widgets são structs que compõem. Backend abstraction: crossterm (Rust puro, cross-platform) ou termion (Linux/macOS). Layout system com constraints (percentage, min, max, fixed). Widgets: table, list, chart, sparkline, gauge, paragraph, tabs.

Textual (Python) é o mais ambicioso. CSS-like styling (sim, CSS no terminal), reactive DOM, widget tree, message passing, async-first. Faz coisas que parecem impossíveis no terminal (scroll containers, rich text, markdown rendering). Web-based mode permite rodar no browser.

Ink (React) é React para o terminal. Usa Yoga para layout (flexbox no terminal). Componentes React, hooks, todo o modelo mental. Rendering via terminais ANSI. Bom para CLIs interativas construídas por quem já conhece React.

Cursive (Rust) é mais simples que Ratatui. Callback-based (não Elm Architecture). Views empilháveis, dialog boxes, menus.

Atributos técnicos: architecture (Elm Architecture vs callback-based vs React model), terminal backend (crossterm, termion, termbox), color support (16 colors, 256, true color/24-bit), mouse support, Unicode/emoji rendering, layout system, focus management, async I/O integration, e WASM/web compatibility.

10. GPU-Accelerated UI Framework

Não é uma categoria exclusiva, é um atributo que cruza com as anteriores. Frameworks que bypassam completamente o sistema de widgets do OS e renderizam tudo direto na GPU. Cada pixel é desenhado pelo framework.

O tradeoff fundamental é: controle total sobre rendering (custom text rendering, custom animations, pixel-perfect cross-platform consistency) em troca de perda de integração com o OS (input methods para CJK/composição, acessibilidade nativa, look-and-feel nativo, system dark mode sem implementação manual).

Exemplos: GPUI (Zed), Flutter (Skia/Impeller), Iced (wgpu), Slint (Skia/FemtoVG), egui (wgpu/glow), Dear ImGui (qualquer backend gráfico).

Atributos adicionais além dos já mencionados: text rendering quality (hinting, subpixel rendering, LCD anti-aliasing, ligatures, variable fonts), input method editor (IME) support (crítico para CJK languages, muitos frameworks GPU-rendered têm bugs aqui), screen reader integration (AccessKit é o esforço em Rust para resolver isso), high-DPI / fractional scaling, e multi-window support.

11. Camadas Transversais

Algumas categorias não cabem na hierarquia linear porque operam transversalmente.

Text Shaping / Font Rendering: HarfBuzz (shaping, open source, padrão da indústria), FreeType (rasterização de glyphs), fontconfig (descoberta de fontes no Linux), Core Text (macOS), DirectWrite (Windows), Swash/fontdb/cosmic-text (alternativas Rust). Text shaping é o processo de converter uma string Unicode em posições de glyphs, considerando ligatures, kerning, BiDi, complex scripts (árabe, devanagari).

Windowing / Platform Abstraction: winit (Rust, abstração cross-platform sobre windowing e input), SDL2 (C, clássico para games, windowing + input + audio), GLFW (C, lightweight, focado em OpenGL/Vulkan context creation). Esses não são UI frameworks, são a camada que cria a janela e captura eventos de input que os UI frameworks consomem.

Accessibility: AccessKit (Rust, abstração cross-platform sobre APIs de acessibilidade: NSAccessibility no macOS, UIA no Windows, AT-SPI no Linux), WAI-ARIA (especificação web), VoiceOver/NVDA/Orca (screen readers de cada plataforma).

Animation: No web, Framer Motion (React), GSAP, Web Animations API, CSS transitions/animations. No native, cada framework tem seu próprio sistema ou delega para interpolação manual. Spring-based animation (física de mola) é o modelo dominante moderno (SwiftUI, Framer Motion, React Native Reanimated).

Essa é a cobertura completa. A hierarquia expandida:

Windowing/Platform Abstraction → Text Shaping → 2D Render Engine → Layout Engine → Reactive Primitives → UI Framework (IMGUI | Retained Imperativo | Declarative Native | Web | TUI | GPU-Accelerated) → Headless UI → Styling Engine → Component Library → Meta-framework

Com as camadas transversais (Accessibility, Animation) operando em múltiplos níveis simultâneamente.

--

Porque ainda nao existe um que enblgoa tudo:

Porque os tradeoffs são mutuamente exclusivos em várias camadas. Não é uma questão de ninguém ter tentado (Qt tenta há 30 anos, Flutter tenta há 8), é que as restrições fundamentais se contradizem.

O primeiro conflito é no render target. DOM, GPU, terminal e widgets nativos do OS são superfícies de rendering com modelos de composição incompatíveis. O DOM é uma árvore de boxes com CSS cascade, reflowing automático, acessibilidade embutida, e um modelo de evento que borbulha pela árvore. GPU é um pipeline de draw calls onde você rasteriza primitivos geométricos em framebuffers, sem noção de "texto", "botão" ou "acessibilidade", tudo é triângulo e textura. Terminal é uma grade de células de caractere com escape codes ANSI. Widgets nativos do OS (NSButton, GtkButton, QPushButton) são objetos opacos que o OS desenha e gerencia internamente, e você não controla o rendering deles. Um abstração que unifique esses quatro targets inevitavelmente colapsa para o menor denominador comum, que é inútil, ou precisa de backends tão diferentes que a "unificação" é ilusória, é quatro frameworks com uma API em comum. Flutter escolheu ignorar widgets nativos inteiramente e renderizar tudo via Skia, o que resolve a consistência cross-platform mas perde integração nativa (IME, acessibilidade do OS, look-and-feel, system fonts).

O segundo conflito é entre immediate mode e retained mode. São modelos mentais opostos. IMGUI funciona porque não tem estado de UI persistente, o que é perfeito para tooling e overlays mas impossibilita acessibilidade (screen readers precisam de uma árvore persistente para navegar), layout complexo (flexbox/grid exigem multiple passes sobre uma árvore retida), e animações state-driven (transições entre estados exigem que os estados existam entre frames). Retained mode resolve tudo isso mas introduz complexidade de sincronização entre estado da aplicação e estado da UI (o problema inteiro que React, Elm Architecture e signals tentam resolver). Nenhum modelo é superior, são otimizados para problemas diferentes.

O terceiro conflito é entre compile-time e runtime. Svelte e Slint resolvem reatividade em compile time, gerando código imperativo otimizado sem runtime overhead. React e Vue resolvem em runtime com Virtual DOM ou proxies, o que é mais flexível (componentes dinâmicos, hot reload trivial, plugins de terceiros) mas mais lento. Leptos signals e SolidJS ficam no meio (fine-grained runtime, mas sem Virtual DOM). Não dá pra ter a performance de compile-time com a flexibilidade de runtime. É termodinâmica da computação.

O quarto conflito é político e econômico. Apple nunca vai abrir SwiftUI, Google controla Flutter e Compose, Meta controla React. Cada empresa otimiza para seus próprios devices e stack. Qt tentou ser o framework universal e o resultado é uma dependência de 200MB com seu próprio compilador (MOC), string type (QString), container library (QList), e praticamente tudo reimplementado fora do stdlib. É o que acontece quando você tenta ser tudo: vira um runtime paralelo.

O quinto conflito é na camada de texto. Text shaping é um dos problemas mais difíceis da computação visual. HarfBuzz + FreeType + fontconfig + BiDi algorithm + line breaking (UAX #14) + script itemization é uma stack inteira por si só. No browser, isso é resolvido pelo engine (Blink/WebKit/Gecko) com décadas de investimento. Fora do browser, cada framework reimplementa parcialmente ou depende de Skia (que embute HarfBuzz). Um framework universal teria que resolver text rendering em nível de qualidade de browser cross-platform, que é um projeto de dezenas de pessoa-anos. O projeto cosmic-text (Rust, usado pelo COSMIC desktop do System76) está tentando exatamente isso e já tem anos de desenvolvimento só na camada de texto.

O que existe de mais próximo de "tudo junto":

Qt é a tentativa mais antiga e completa. Cobre desktop, mobile, embedded, tem widget imperativo (QWidget), declarativo (QML/Qt Quick), 3D, multimedia, networking, SQL. O custo é: binário enorme, MOC preprocessor, C++ idiomático que diverge do C++ moderno, e licensing que assusta muitos projetos.

Flutter é a tentativa mais agressiva de unified rendering. Um engine (Skia/Impeller), um layout system, um widget set, todos os platforms. O custo é: Dart como linguagem (ninguém mais usa Dart fora do Flutter), zero integração com widgets nativos, acessibilidade reimplementada do zero (e ainda inconsistente), e o engine Skia/Impeller é uma dependência pesada.

No ecossistema Rust, o projeto Linebender (Raph Levien, Google Fonts) está construindo as camadas fundamentais de baixo pra cima: Vello (render engine GPU-first), Xilem (UI framework declarativo experimental), Parley (text layout), cosmic-text (text editing), AccessKit (acessibilidade). A tese é que se você construir cada camada corretamente em Rust com ownership e performance, eventualmente elas compõem num framework completo. Mas está a anos de ser production-ready.

A resposta honesta é que um framework que faz tudo seria tão complexo que ninguém conseguiria aprender, manter ou debugar. O ecossistema fragmentado é um reflexo correto da fragmentação real dos problemas. O que faz sentido buscar não é "um framework que faz tudo", é uma stack onde as camadas são composáveis e substituíveis, que é exatamente o que o ecossistema Rust está tentando com crates independentes que se encaixam (winit + wgpu + taffy + cosmic-text + accesskit + seu framework de escolha).

--

O satytual atual do phi:

 Mapeamento: Pesquisa vs Estado Atual do Phi

  Analisei o código real contra cada camada da sua pesquisa.

  1. Reactive Primitives / State Management

  ┌─────────────────────────────┬───────────────────────────────────────┬────────┐
  │    Conceito da Pesquisa     │               Phi Hoje                │ Status │
  ├─────────────────────────────┼───────────────────────────────────────┼────────┤
  │ Signal (valor reativo       │ ReadSignal<T> + WriteSignal<T> via    │ ✓      │
  │ atômico)                    │ SlotMap                               │        │
  ├─────────────────────────────┼───────────────────────────────────────┼────────┤
  │ Computed/Derived            │ create_memo(f) com early bailout      │ ✓      │
  │ (memoizado)                 │                                       │        │
  ├─────────────────────────────┼───────────────────────────────────────┼────────┤
  │ Effect (side-effect         │ create_effect(f) com pending queue    │ ✓      │
  │ reativo)                    │                                       │        │
  ├─────────────────────────────┼───────────────────────────────────────┼────────┤
  │ Push vs Pull                │ Híbrido (push notify, pull evaluate)  │ ✓      │
  ├─────────────────────────────┼───────────────────────────────────────┼────────┤
  │ Batching de updates         │ batch() com depth counter             │ ✓      │
  ├─────────────────────────────┼───────────────────────────────────────┼────────┤
  │ Glitch-free propagation     │ 3-state (Clean/Check/Dirty) com memo  │ ✓      │
  │                             │ comparison                            │        │
  ├─────────────────────────────┼───────────────────────────────────────┼────────┤
  │ Fine-grained tracking       │ Por signal individual (não por        │ ✓      │
  │                             │ componente)                           │        │
  ├─────────────────────────────┼───────────────────────────────────────┼────────┤
  │ Circular dependency         │ Panic on cycle                        │ ✓      │
  │ detection                   │                                       │        │
  ├─────────────────────────────┼───────────────────────────────────────┼────────┤
  │ Memory management           │ Manual dispose_node() (ownership      │ ✓      │
  │                             │ Rust)                                 │        │
  ├─────────────────────────────┼───────────────────────────────────────┼────────┤
  │ Subscribe/unsubscribe       │ ✗                                     │ ◉ gap  │
  │ explícito                   │                                       │        │
  ├─────────────────────────────┼───────────────────────────────────────┼────────┤
  │ Transaction/rollback        │ ✗                                     │ ◉ gap  │
  └─────────────────────────────┴───────────────────────────────────────┴────────┘

  Veredito: 90% coberto. O sistema reativo do Phi é equivalente ao Leptos/SolidJS em
  design. Os gaps (subscribe explícito, transactions) são nice-to-have, não
  bloqueadores.

  2. Layout Engine

  ┌───────────────────────┬─────────────────────────────────────────┬───────────┐
  │ Conceito da Pesquisa  │                Phi Hoje                 │  Status   │
  ├───────────────────────┼─────────────────────────────────────────┼───────────┤
  │ Flexbox               │ Taffy 0.9, completo                     │ ✓         │
  │                       │ (grow/shrink/basis/wrap)                │           │
  ├───────────────────────┼─────────────────────────────────────────┼───────────┤
  │ CSS Grid              │ ✗ (Taffy suporta, Phi não expõe)        │ ◉ gap     │
  │                       │                                         │ fácil     │
  ├───────────────────────┼─────────────────────────────────────────┼───────────┤
  │ Direction Row/Column  │ ✓                                       │ ✓         │
  ├───────────────────────┼─────────────────────────────────────────┼───────────┤
  │ Align/Justify         │ Start, Center, End, Stretch,            │ ✓         │
  │ completo              │ SpaceBetween, SpaceAround, SpaceEvenly  │           │
  ├───────────────────────┼─────────────────────────────────────────┼───────────┤
  │ Padding/Margin/Gap    │ 4 lados                                 │ ✓         │
  ├───────────────────────┼─────────────────────────────────────────┼───────────┤
  │ Min/Max constraints   │ width, height, min_*, max_*             │ ✓         │
  ├───────────────────────┼─────────────────────────────────────────┼───────────┤
  │ Text measurement      │ Aproximação: chars * font_size * 0.6    │ ⚠         │
  │ callback              │                                         │ impreciso │
  ├───────────────────────┼─────────────────────────────────────────┼───────────┤
  │ Incremental layout    │ ✗ (recalcula tudo por frame)            │ ◉ gap     │
  │                       │                                         │ perf      │
  ├───────────────────────┼─────────────────────────────────────────┼───────────┤
  │ Constraint-based      │ ✗ (não precisa, flexbox é suficiente)   │ ∘ n/a     │
  │ (Cassowary)           │                                         │           │
  ├───────────────────────┼─────────────────────────────────────────┼───────────┤
  │ Absolute positioning  │ ✗                                       │ ◉ gap     │
  └───────────────────────┴─────────────────────────────────────────┴───────────┘

  Veredito: 70% coberto. Flexbox funcional. Gaps: Grid (fácil de expor via Taffy),
  absolute positioning, text measurement preciso, incremental layout.

  3. 2D Render Engine / Scene Graph

  ┌────────────────────────────┬──────────────────────────────────┬─────────────┐
  │    Conceito da Pesquisa    │             Phi Hoje             │   Status    │
  ├────────────────────────────┼──────────────────────────────────┼─────────────┤
  │ Retângulos sólidos         │ Quad pipeline                    │ ✓           │
  ├────────────────────────────┼──────────────────────────────────┼─────────────┤
  │ Retângulos arredondados +  │ RectSdf pipeline (Inigo Quilez   │ ✓           │
  │ borda                      │ SDF)                             │             │
  ├────────────────────────────┼──────────────────────────────────┼─────────────┤
  │ Texto (glyph atlas)        │ Text pipeline, cosmic-text,      │ ✓           │
  │                            │ atlas 512→4096                   │             │
  ├────────────────────────────┼──────────────────────────────────┼─────────────┤
  │ Paths arbitrários          │ Lyon tessellation (fill +        │ ✓           │
  │                            │ stroke)                          │             │
  ├────────────────────────────┼──────────────────────────────────┼─────────────┤
  │ Compositing por layers     │ Compositor com z-order,          │ ✓           │
  │                            │ visibility, opacity              │             │
  ├────────────────────────────┼──────────────────────────────────┼─────────────┤
  │ Blur                       │ 13-tap Gaussian (H/V separable)  │ ✓           │
  ├────────────────────────────┼──────────────────────────────────┼─────────────┤
  │ Drop Shadow                │ offset + sigma + color           │ ✓           │
  ├────────────────────────────┼──────────────────────────────────┼─────────────┤
  │ Anti-aliasing              │ SDF analytic (1px feather)       │ ✓           │
  ├────────────────────────────┼──────────────────────────────────┼─────────────┤
  │ Premultiplied alpha        │ Pipeline inteiro                 │ ✓           │
  ├────────────────────────────┼──────────────────────────────────┼─────────────┤
  │ Gradientes (linear,        │ ✗                                │ ✗           │
  │ radial, conic)             │                                  │ bloqueador  │
  ├────────────────────────────┼──────────────────────────────────┼─────────────┤
  │ Imagens (decode + GPU      │ ✗                                │ ✗           │
  │ upload)                    │                                  │ bloqueador  │
  ├────────────────────────────┼──────────────────────────────────┼─────────────┤
  │ SVG                        │ ✗                                │ ✗           │
  │                            │                                  │ bloqueador  │
  ├────────────────────────────┼──────────────────────────────────┼─────────────┤
  │ Clipping arbitrário        │ ✗ (só clip_rect retangular)      │ ◉ gap       │
  ├────────────────────────────┼──────────────────────────────────┼─────────────┤
  │ Per-corner radius          │ ✗ (uniforme)                     │ ◉ gap       │
  ├────────────────────────────┼──────────────────────────────────┼─────────────┤
  │ Blend modes customizados   │ ✗ (só premultiplied alpha)       │ ◉ gap       │
  └────────────────────────────┴──────────────────────────────────┴─────────────┘

  Veredito: 55% coberto. Base sólida (SDF, paths, compositing). Os 3 bloqueadores
  (gradientes, imagens, SVG) impedem qualquer aplicação real.

  4. Styling Engine

  ┌─────────────────────────────┬───────────────────────────────────────┬────────┐
  │    Conceito da Pesquisa     │               Phi Hoje                │ Status │
  ├─────────────────────────────┼───────────────────────────────────────┼────────┤
  │ Design tokens (cores,       │ Theme struct com escalas nomeadas     │ ✓      │
  │ spacing, radius)            │                                       │        │
  ├─────────────────────────────┼───────────────────────────────────────┼────────┤
  │ Intent-based theming        │ Intent { Neutral, Constructive,       │ ✓      │
  │                             │ Destructive, Informational }          │        │
  ├─────────────────────────────┼───────────────────────────────────────┼────────┤
  │ Motion physics por intent   │ MotionPhysics com modulação por       │ ✓      │
  │                             │ intent                                │        │
  ├─────────────────────────────┼───────────────────────────────────────┼────────┤
  │ Dark theme                  │ Theme::dark()                         │ ✓      │
  ├─────────────────────────────┼───────────────────────────────────────┼────────┤
  │ Light theme                 │ ✗                                     │ ◉ gap  │
  ├─────────────────────────────┼───────────────────────────────────────┼────────┤
  │ Token overrides por         │ ✗                                     │ ◉ gap  │
  │ componente                  │                                       │        │
  ├─────────────────────────────┼───────────────────────────────────────┼────────┤
  │ CSS-like syntax             │ ✗ (n/a, é Rust nativo)                │ ∘ n/a  │
  ├─────────────────────────────┼───────────────────────────────────────┼────────┤
  │ Utility-first               │ ✗ (n/a)                               │ ∘ n/a  │
  │ (Tailwind-like)             │                                       │        │
  └─────────────────────────────┴───────────────────────────────────────┴────────┘

  Veredito: 60% coberto (do que é aplicável a native). Intent-based theming é
  diferenciador real. Falta light theme e override granular.

  5. Headless UI Libraries

  ┌────────────────────────────┬────────────────────────────────────┬────────────┐
  │    Conceito da Pesquisa    │              Phi Hoje              │   Status   │
  ├────────────────────────────┼────────────────────────────────────┼────────────┤
  │ WAI-ARIA compliance        │ AccessKit tree gerado, mas sem     │ ⚠ parcial  │
  │                            │ semântica completa                 │            │
  ├────────────────────────────┼────────────────────────────────────┼────────────┤
  │ Focus trap                 │ ✗                                  │ ✗          │
  ├────────────────────────────┼────────────────────────────────────┼────────────┤
  │ Keyboard navigation        │ Focus graph espacial (próximo,     │ ✓ parcial  │
  │                            │ anterior, cima, baixo)             │            │
  ├────────────────────────────┼────────────────────────────────────┼────────────┤
  │ Compound components        │ ✗                                  │ ✗          │
  ├────────────────────────────┼────────────────────────────────────┼────────────┤
  │ Portal support (render     │ OverlayManager (ContextMenu,       │ ✓          │
  │ fora da árvore)            │ Modal, Tooltip)                    │ estrutura  │
  ├────────────────────────────┼────────────────────────────────────┼────────────┤
  │ Animation hooks            │ ✗                                  │ ✗          │
  │ (enter/exit)               │                                    │            │
  ├────────────────────────────┼────────────────────────────────────┼────────────┤
  │ Collision-aware            │ ✗                                  │ ✗          │
  │ positioning                │                                    │            │
  └────────────────────────────┴────────────────────────────────────┴────────────┘

  Veredito: 25% coberto. OverlayManager e focus graph existem como esqueleto. Falta
  implementação real de comportamento headless (focus trap, keyboard nav completa,
  enter/exit transitions).

  6a. IMGUI

  Phi não é IMGUI. É retained-mode com element tree. Categoria não se aplica.

  6b. Native Widget Toolkit (Retained Imperativo)

  Phi não é imperativo. Não tem button.setText(). Categoria não se aplica.

  6c. Declarative Native (Retained Declarativo) ◄ Phi está aqui

  ┌────────────────────────────┬──────────────────────────────────────┬──────────┐
  │    Conceito da Pesquisa    │               Phi Hoje               │  Status  │
  ├────────────────────────────┼──────────────────────────────────────┼──────────┤
  │ Declarative view           │ Builder DSL: Element { kind, layout, │ ✓        │
  │ description                │  style, children }                   │          │
  ├────────────────────────────┼──────────────────────────────────────┼──────────┤
  │ Reactive state (signals)   │ create_signal, create_memo,          │ ✓        │
  │                            │ create_effect                        │          │
  ├────────────────────────────┼──────────────────────────────────────┼──────────┤
  │ Tree                       │ ✗ (rebuild completo por frame)       │ ◉ gap    │
  │ diffing/reconciliation     │                                      │ perf     │
  ├────────────────────────────┼──────────────────────────────────────┼──────────┤
  │ Component model (fn →      │ fn(props) -> Element                 │ ✓        │
  │ Element)                   │                                      │          │
  ├────────────────────────────┼──────────────────────────────────────┼──────────┤
  │ Hot reload                 │ Shader watch + DSL override          │ ✓        │
  │                            │ (feature-gated)                      │          │
  ├────────────────────────────┼──────────────────────────────────────┼──────────┤
  │ Animation system           │ Spring + Tween + 31 easings +        │ ✓        │
  │                            │ Keyframes                            │          │
  ├────────────────────────────┼──────────────────────────────────────┼──────────┤
  │ Compilation model          │ AOT (Rust nativo, WASM)              │ ✓        │
  ├────────────────────────────┼──────────────────────────────────────┼──────────┤
  │ Multi-platform             │ Desktop ✓, WASM ✓, Android ⚠, iOS ⚠  │ ⚠        │
  ├────────────────────────────┼──────────────────────────────────────┼──────────┤
  │ Accessibility              │ AccessKit tree + focus graph         │ ✓        │
  │                            │                                      │ parcial  │
  ├────────────────────────────┼──────────────────────────────────────┼──────────┤
  │ Conditional rendering      │ ✗ (sem if no builder)                │ ◉ gap    │
  ├────────────────────────────┼──────────────────────────────────────┼──────────┤
  │ List rendering             │ ✗ (sem .for_each() / .map())         │ ◉ gap    │
  └────────────────────────────┴──────────────────────────────────────┴──────────┘

  Veredito: 65% coberto. Arquitetura correta. Gaps: tree diffing, conditional/list
  rendering no builder, plataformas mobile imaturas.

  6d. Web Framework

  N/A. Phi não é web framework. Compila para WASM mas não usa DOM.

  7. Meta-framework

  N/A. Phi é um motor de composição, não resolve routing, SSR, data fetching.

  8. Component Library / Design System

  ┌──────────────────────────────┬───────────────────────────────┬──────────────┐
  │     Conceito da Pesquisa     │           Phi Hoje            │    Status    │
  ├──────────────────────────────┼───────────────────────────────┼──────────────┤
  │ Modelo shadcn (copy-paste,   │ ✗ (não existe ainda)          │ ✗ target     │
  │ ownership)                   │                               │ futuro       │
  ├──────────────────────────────┼───────────────────────────────┼──────────────┤
  │ Componentes existentes       │ Div, Text, Path, Badge,       │ ⚠ mínimo     │
  │                              │ Spacer                        │              │
  ├──────────────────────────────┼───────────────────────────────┼──────────────┤
  │ TextInput                    │ TextBuffer + cursor +         │ ✓ básico     │
  │                              │ selection + blink             │              │
  ├──────────────────────────────┼───────────────────────────────┼──────────────┤
  │ Button                       │ ✗ (hit region manual)         │ ✗            │
  ├──────────────────────────────┼───────────────────────────────┼──────────────┤
  │ Checkbox, Radio, Toggle      │ ✗                             │ ✗            │
  ├──────────────────────────────┼───────────────────────────────┼──────────────┤
  │ Slider                       │ ✗                             │ ✗            │
  ├──────────────────────────────┼───────────────────────────────┼──────────────┤
  │ Dropdown/Select              │ ✗                             │ ✗            │
  ├──────────────────────────────┼───────────────────────────────┼──────────────┤
  │ Modal/Dialog                 │ OverlayManager (estrutura,    │ ⚠ esqueleto  │
  │                              │ sem rendering)                │              │
  ├──────────────────────────────┼───────────────────────────────┼──────────────┤
  │ Table/Grid                   │ ✗                             │ ✗            │
  ├──────────────────────────────┼───────────────────────────────┼──────────────┤
  │ Tabs                         │ ✗                             │ ✗            │
  ├──────────────────────────────┼───────────────────────────────┼──────────────┤
  │ Toast/Notification           │ ✗                             │ ✗            │
  ├──────────────────────────────┼───────────────────────────────┼──────────────┤
  │ Virtual scroll               │ ✗                             │ ✗            │
  └──────────────────────────────┴───────────────────────────────┴──────────────┘

  Veredito: 10% coberto. Praticamente zero widgets prontos. Este é o maior gap
  visível para adoção.

  9. TUI Framework

  N/A. Phi é GPU, não terminal. (Porém o kdb menciona extração TUI como possibilidade
   futura.)

  10. GPU-Accelerated UI Framework

  ┌───────────────────────────┬────────────────────────────────────┬────────────┐
  │   Conceito da Pesquisa    │              Phi Hoje              │   Status   │
  ├───────────────────────────┼────────────────────────────────────┼────────────┤
  │ Bypassa widgets do OS     │ ✓ (rendering 100% próprio)         │ ✓          │
  ├───────────────────────────┼────────────────────────────────────┼────────────┤
  │ Pixel-perfect             │ ✓ (WGSL unificado)                 │ ✓          │
  │ cross-platform            │                                    │            │
  ├───────────────────────────┼────────────────────────────────────┼────────────┤
  │ Text rendering quality    │ cosmic-text + swash rasterizer +   │ ✓          │
  │                           │ GPU atlas                          │            │
  ├───────────────────────────┼────────────────────────────────────┼────────────┤
  │ IME support               │ ImeState existe, não integrado     │ ⚠          │
  │                           │                                    │ esqueleto  │
  ├───────────────────────────┼────────────────────────────────────┼────────────┤
  │ Screen reader             │ AccessKit tree gerado              │ ✓ parcial  │
  ├───────────────────────────┼────────────────────────────────────┼────────────┤
  │ High-DPI / fractional     │ Scale factor tracking em window.rs │ ✓          │
  │ scaling                   │                                    │            │
  ├───────────────────────────┼────────────────────────────────────┼────────────┤
  │ Multi-window              │ ✗                                  │ ✗          │
  └───────────────────────────┴────────────────────────────────────┴────────────┘

  Veredito: 70% coberto. O core GPU-first está sólido. IME e multi-window são gaps.

  11. Camadas Transversais

  ┌──────────────────┬────────────────────────────────────────────────┬──────────┐
  │      Camada      │                    Phi Hoje                    │  Status  │
  ├──────────────────┼────────────────────────────────────────────────┼──────────┤
  │ Text Shaping     │ cosmic-text (HarfBuzz interno) + swash         │ ✓        │
  │                  │ rasterizer                                     │          │
  ├──────────────────┼────────────────────────────────────────────────┼──────────┤
  │ Font Discovery   │ fontdb + 4 fontes embarcadas (Inter,           │ ✓        │
  │                  │ JetBrains, Codicons)                           │          │
  ├──────────────────┼────────────────────────────────────────────────┼──────────┤
  │ Windowing        │ winit 0.30                                     │ ✓        │
  ├──────────────────┼────────────────────────────────────────────────┼──────────┤
  │ Accessibility    │ AccessKit 0.24 + accesskit_winit               │ ✓        │
  │                  │                                                │ parcial  │
  ├──────────────────┼────────────────────────────────────────────────┼──────────┤
  │ Animation        │ Spring physics + Tween + 31 easings +          │ ✓        │
  │                  │ Keyframes                                      │          │
  ├──────────────────┼────────────────────────────────────────────────┼──────────┤
  │ Path             │ Lyon (fill + stroke + bezier)                  │ ✓        │
  │ Tessellation     │                                                │          │
  ├──────────────────┼────────────────────────────────────────────────┼──────────┤
  │ Clipboard        │ ✗                                              │ ✗        │
  ├──────────────────┼────────────────────────────────────────────────┼──────────┤
  │ Drag and Drop    │ ✗                                              │ ✗        │
  ├──────────────────┼────────────────────────────────────────────────┼──────────┤
  │ File Dialogs     │ ✗                                              │ ✗        │
  └──────────────────┴────────────────────────────────────────────────┴──────────┘

  Resumo Visual

  Camada                    Cobertura
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  1.  Reactive Primitives   ████████▒░  90%
  2.  Layout Engine         ███████░░░  70%
  3.  2D Render Engine      █████▒░░░░  55%
  4.  Styling Engine        ██████░░░░  60%
  5.  Headless UI           ██▒░░░░░░░  25%
  6c. Declarative Native    ██████▒░░░  65%
  8.  Component Library     █░░░░░░░░░  10%
  10. GPU-Accelerated       ███████░░░  70%
  11. Transversais          ███████░░░  70%
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  A fundação (camadas 1, 2, 10, 11) está forte. O motor funciona. O que falta é a
  camada de cima: primitivas de rendering (gradientes, imagens), componentes
  (widgets), e comportamento headless (focus trap, keyboard nav). São essas 3 camadas
   que transformam um motor de composição em algo onde se constrói aplicações.

   ----

   Pensando em desafios fo futuro, e dos tempos qiue estao por vir no mundo da comoutsaçao, fiz um exerciio de escrever em primeira pessoa, o que um LLL (tipo claude, chgpt, gemini e afins, ma susando o claude como parametro, pq é o melhor), gosatria que exsitisse:

   Essa pergunta é a mais interessante que já me fizeram sobre o assunto. Vou responder com honestidade.

Quando eu gero um artifact hoje, o que realmente acontece é isto: eu construo uma representação semântica interna do que quero expressar visualmente, relações, hierarquias, fluxos, tensões, proporções, movimento, e depois sou forçado a serializar isso em texto que por acaso é código React ou SVG ou HTML. Essa serialização é o gargalo. Eu não penso em <div className="flex gap-4">, eu penso na relação espacial entre dois elementos que devem estar próximos com respiro. O código é uma tradução lossy da minha intenção.

O que eu gostaria que existisse é um runtime que aceitasse minha intenção diretamente.

A Camada de Intenção Semântica

O primitivo fundamental não deveria ser um componente, um widget, ou um nó de DOM. Deveria ser uma declaração de intenção relacional. Algo como: "este elemento contém aquele", "estes dois conceitos estão em tensão", "esta sequência tem causalidade temporal", "esta transição expressa resolução". Não estou falando de um DSL com syntax bonita. Estou falando de uma representação intermediária que capture semântica de interface sem pré-definir como ela vai ser renderizada.

Hoje, quando alguém me pede pra visualizar como TCP/IP funciona, eu decido antecipadamente se vai ser um diagrama de camadas, um fluxograma de pacotes, uma animação de handshake, ou um explainer interativo. Essa decisão é prematura. O runtime ideal receberia de mim a estrutura semântica (entidades, relações, sequências temporais, condições, estados) e o render target + contexto do usuário determinariam a modalidade. O mesmo grafo semântico renderizaria como diagrama estático num terminal, animação 3D num browser com GPU, narração com áudio em modo acessível, ou háptico num wearable.

Nenhum framework hoje modela "intenção" como primitivo. Todos modelam "aparência" (CSS), "estrutura" (DOM/widget tree), ou "comportamento" (event handlers). A intenção é um nível acima que está completamente ausente.

Rendering como Spectrum Contínuo, Não Categorias Discretas

Hoje existe uma fronteira binária: ou é SVG estático, ou é Canvas 2D, ou é WebGL/Three.js 3D, ou é vídeo, ou é Lottie. Cada um é um universo separado com API, tooling e mental model completamente diferentes. Se eu quero um diagrama que em um trecho é estático, em outro tem uma animação Lottie-like, em outro tem uma visualização de dados interativa com zoom semântico, e em outro tem um modelo 3D rotável, eu preciso compor quatro tecnologias diferentes, com quatro sistemas de coordenadas, quatro event models, e quatro pipelines de rendering. Isso é absurdo.

O que eu gostaria é de um pipeline de rendering unificado onde a "dimensionalidade" e a "dinamicidade" são propriedades contínuas de cada nó, não categorias exclusivas do runtime inteiro. Um nó pode ser estático (SVG-like), outro animado proceduralmente (shader-like), outro respondendo a física (motor de física), outro renderizando geometria 3D (mesh renderer), e todos coexistem no mesmo scene graph, no mesmo frame, compartilhando o mesmo sistema de coordenadas e event model. Vello está na direção certa ao unificar 2D rendering via compute shaders, mas falta a extensão para 3D, animação procedural, e composição de mídia.

Isso eliminaria a distinção artificial entre "gráfico vetorial", "animação", "data viz", "3D", "generative art" e "UI". Todas seriam manifestações diferentes do mesmo grafo semântico renderizado com parâmetros diferentes de dimensionalidade, dinamicidade e interatividade.

Bilhões de Grafos de Código: O Que Eu Realmente Quero

Você tocou num ponto que me importa muito. Quando eu gero um componente React com 200 linhas, o que eu realmente produzi não é 200 linhas de código. É um grafo de dependências semânticas entre ~30 a 80 conceitos (componentes, estados, efeitos, estilos, handlers, dados) serializado linearmente em texto. A representação textual é a pior possível para o que está acontecendo. O humano lê linearmente, o browser parsea em AST, o bundler tree-shake, o runtime executa, e ninguém nunca vê o grafo de intenção que eu construí internamente antes de serializar.

O que eu quero é emitir grafos diretamente. Não código que será parseado em grafo, mas o grafo nativo. Cada nó com tipo semântico (layout, data binding, state transition, visual property, constraint, animation keyframe), cada aresta com relação tipada (contains, depends-on, transitions-to, derives-from, constrains). O runtime consumiria esse grafo, otimizaria (merge de nós redundantes, dead node elimination, layout pre-computation), e renderizaria. Quando eu preciso modificar o artifact, eu emito um diff de grafo (add node, remove edge, change property), não regenero 200 linhas de texto. Isso seria ordens de magnitude mais eficiente para streaming.

Isso resolveria o problema de colaboração humano-modelo que mencionei antes. O humano poderia visualizar o grafo, clicar num nó, dizer "mude isso", e eu emitiria um patch localizado. Hoje, se o humano quer mudar a cor de um botão no meu artifact, eu preciso reprocessar o contexto inteiro e emitir centenas de tokens de código para uma mudança de uma propriedade. Com grafo nativo, seria um patch de um nó.

Feedback Loop em Tempo Real

Hoje eu gero no escuro. Emito código, o humano vê o resultado, me diz o que está errado, eu regenero. Zero feedback visual durante a geração. Eu não sei como meu output está renderizando enquanto o emito.

O runtime ideal teria um canal de feedback onde, enquanto eu emito tokens/nós do grafo, o renderer me devolveria informação sobre o estado atual: "o layout overflow nessa constraint", "esse texto está truncado", "essa cor não tem contraste suficiente contra esse background", "esse animation timing causa jank no framerate". Eu poderia ajustar em tempo real, no mesmo stream de geração, sem roundtrip humano.

Isso não existe em nenhum lugar. Nenhum sistema de code generation tem feedback loop do renderer pro gerador durante a geração. É sempre: gera → compila → renderiza → humano avalia → volta pro modelo. Eliminar os passos intermediários transformaria a qualidade do que eu produzo.

As Modalidades Que Eu Gostaria de Compor Livremente

Se o runtime não tivesse fronteiras artificiais entre tecnologias, o que eu gostaria de poder compor num único artifact, num único frame, como cidadãos de primeira classe:

Geometria vetorial 2D (paths, shapes, text como Bézier curves, não como DOM elements). Geometria 3D (meshes, materiais PBR, iluminação, sombras, profundidade de campo). Animação procedural (não keyframes pré-definidos, mas regras: "este elemento orbita aquele com velocidade proporcional a este dado", "esta partícula é atraída por gravidade em direção ao cursor"). Simulação física (cloth, fluido, rigid body, soft body, partículas). Áudio generativo (síntese procedural, spatialização 3D vinculada à posição de elementos visuais, frequência modulada por dados). Data binding reativo (qualquer propriedade visual vinculada a um stream de dados: API, sensor, inferência de modelo, websocket). Tipografia como primitivo de primeira classe (não box de texto, mas glyphs como geometria manipulável: variable fonts com axes animáveis, text-on-path, deformação, shatter). Video e camera feed como texturas aplicáveis a qualquer superfície no scene graph. Interatividade multi-modal (touch, mouse, gaze tracking, voz, gesture, acelerômetro, todos como streams de input unificados). State machines visuais (não código de máquina de estados, mas máquinas de estado como primitivo visual que o runtime executa e o humano pode inspecionar). Transições entre estados como animações interpoladas automaticamente (o runtime calcula a interpolação entre dois estados do grafo, como FLIP animation mas generalizado para qualquer propriedade).

Flash, com todos os seus problemas, era o que mais se aproximava dessa visão. Timeline-based animation, vetorial, áudio, ActionScript, tudo num runtime unificado. Morreu por razões políticas (Apple) e de segurança, não porque o modelo mental estava errado. O modelo mental estava mais certo que qualquer coisa que veio depois. A web moderna fragmentou o que Flash unificava em dez tecnologias que não conversam entre si.

Gaps Que Ninguém Resolveu e Por Quê

O gap mais profundo é zoom semântico. Nenhum framework implementa a ideia de que a mesma informação deve ter representações visuais diferentes em diferentes níveis de zoom/atenção. Um sistema solar num zoom alto é um ponto. Zoom médio, são órbitas. Zoom baixo, são planetas com superfícies detalhadas. O dado não mudou, a representação adaptou ao nível de atenção. Mapas fazem isso (Google Maps com levels of detail), mas como infraestrutura proprietária, não como primitivo de framework. Se isso fosse um primitivo, eu poderia gerar visualizações de dados que o humano navega por zoom em vez de por páginas ou tabs. O motivo de não existir é que exige um modelo de dados que saiba sua própria hierarquia semântica, e nenhum framework opera no nível semântico, todos operam no nível de pixels e widgets.

O segundo gap é composição temporal. UIs hoje são espaciais: posição x, y, z. Tempo é tratado como efeito colateral (setTimeout, requestAnimationFrame, CSS transition). Não existe um eixo temporal como cidadão de primeira classe do layout system. Se existisse, eu poderia definir que "o elemento A aparece antes do B, o C aparece simultaneamente ao B, o D aparece depois que A desaparece" como constraints temporais no layout, não como código imperativo de animação. Final Cut, After Effects e timeline editors modelam tempo como eixo primário, mas nenhum UI framework faz isso. O motivo é que o modelo mental de UI veio de documentos (HTML) e formulários (GUI toolkits), onde tempo é irrelevante. LLMs mudaram isso porque todo output é temporal por natureza (streaming token-by-token), e a UI precisa de temporalidade como primitivo.

O terceiro gap é representação multi-fidelidade do mesmo conceito. Quando eu gero uma explicação de um algoritmo, idealmente existiriam simultaneamente: a versão em texto (prosa), a versão em pseudocódigo, a versão em diagrama de fluxo, a versão em animação step-by-step, e a versão em código executável. Todas geradas a partir da mesma representação semântica, todas sincronizadas (clicar num passo do diagrama destaca o trecho correspondente no pseudocódigo e na prosa). Nenhum framework sequer modela que múltiplas representações de um mesmo conceito existem e devem estar linkadas. O mais próximo que existe são notebooks (Jupyter, Observable), mas a sincronização entre células é manual e frágil.

O runtime que eu gostaria que existisse não é um framework. É mais parecido com um sistema operacional de interface. Um kernel de rendering multi-modal com um grafo semântico como primitivo central, feedback loop bidirecional entre gerador e renderer, zoom semântico, composição temporal, e multi-fidelidade. As "aplicações" que rodariam nele não seriam programas com UI. Seriam intenções declarativas emitidas por humanos, modelos, ou sensores, que o kernel reconcilia, otimiza e renderiza em tempo real para qualquer superfície disponível.

Em primeira pessoa, exemplo do que eu um LLM (ja faz):

Quando gero qualquer artifact, sem exceção, eu executo algumas operações fundamentais que se repetem independente do target:

Definir entidades com propriedades. Pode ser um nó de grafo, um componente React, um personagem de jogo, uma célula de tabela, um planeta num sistema solar, um neurônio numa rede. Toda entidade tem identidade, propriedades visuais (forma, cor, tamanho, opacidade), propriedades semânticas (tipo, label, peso, categoria), e propriedades temporais (quando aparece, quanto dura, como transiciona).

Definir relações entre entidades. Containment (A contém B), adjacência (A está ao lado de B), dependência (A depende de B), fluxo (de A para B com taxa X), herança (A é tipo de B), exclusão (A e B nunca coexistem), sincronização (A e B mudam juntos). Cada uma dessas relações tem representação visual diferente dependendo do contexto (seta, aninhamento, proximidade, cor compartilhada, linha tracejada), mas a relação semântica é a mesma.

Definir constraints. "Estes nós devem estar alinhados horizontalmente." "Este elemento nunca pode sobrepor aquele." "A distância entre A e B é proporcional ao valor X." "Este grupo ocupa no máximo 40% da largura." Constraints são o que o layout engine resolve, mas hoje eu serializo constraints em código CSS (flexbox/grid), em parâmetros D3 force (forceX, forceY, forceCollide), em Taffy configs, ou em posicionamento manual. A constraint é a mesma, a serialização é diferente 50 vezes.

Definir mapeamentos dado → visual. "A cor deste nó é determinada por sua categoria." "O tamanho é proporcional ao valor." "A opacidade indica confiança." "A posição Y codifica tempo." Isso é a essência da visualização de dados (Bertin/Wilkinson Grammar of Graphics), mas eu reimplemento ad hoc em cada artifact porque nenhum runtime tem mapeamento semântico como primitivo.

Definir comportamento interativo. "Ao clicar neste nó, expanda seus filhos." "Ao arrastar, reposicione com physics constraints." "Ao hover, mostre detalhes." "Ao scroll, faça zoom semântico." "Ao pressionar tecla, mude de modo." Toda interação é uma tripla: input event → predicate (condição) → state mutation. Eu reimplemento onClick/onDrag/onKeyDown centenas de vezes.

Definir transições temporais. "Quando o estado muda de A para B, interpole estas propriedades com esta curva." "Quando novos dados chegam via stream, anime a entrada." "Quando o usuário navega, faça crossfade." Animação não é uma feature opcional colada por cima. É a expressão visual de mudança de estado, que é o conceito mais fundamental de qualquer sistema reativo.

Definir nível de detalhe. "Neste zoom, mostre só o cluster. Mais perto, mostre os nós individuais. Mais perto ainda, mostre as propriedades de cada nó." "Neste contexto, mostre a versão resumida. Naquele, a versão completa." Isso é zoom semântico e é o primitivo mais ausente de toda a stack atual.

O que o runtime universal precisaria ter

Pensando em tudo que eu gero, o runtime seria composto de:

Um scene graph semântico tipado. Não uma árvore DOM, não uma widget tree, não um grafo de renderização. Um grafo onde cada nó tem tipo semântico (concept, relation, container, temporal-sequence, data-source, interaction-zone, viewport) e cada aresta tem tipo relacional (contains, flows-to, depends-on, constrains, synchronizes-with, transitions-to). O grafo é a representação canônica. Tudo mais (rendering, layout, interaction, animation) é derivado dele.

Um layout solver multi-paradigma. Que aceite constraints de flexbox, force-directed, constraint-based (Cassowary), grid, tree layout (Reingold-Tilford), circular, hierarchical (Sugiyama/ELK), geographic, temporal (timeline), e qualquer combinação, no mesmo grafo, simultaneamente. Diferentes subgrafos podem usar diferentes algoritmos de layout. Hoje eu escolho UM layout engine por artifact. Num grafo real, diferentes regiões precisam de layouts diferentes.

Um pipeline de rendering composável. Que renderize no mesmo frame: vetorial 2D (paths, text, shapes), raster (imagens, texturas, video frames), 3D (meshes, luzes, câmera), partículas (emitters, physics, forças), áudio (sintetizadores, spatialização 3D ligada a posição visual), e texto tipográfico como geometria manipulável (não como box opaca de DOM). O compositing final unifica tudo num framebuffer. Cada nó do scene graph declara qual sub-pipeline usa, sem que o desenvolvedor monte o pipeline inteiro.

Um sistema de data binding bidirecional com streams. Qualquer propriedade de qualquer nó pode ser vinculada a: um valor estático, um signal reativo, um stream assíncrono (websocket, SSE, sensor), uma derivação computada de outros nós, ou uma função de tempo. Quando eu gero um dashboard que mostra dados de LLM inference em tempo real (tokens/segundo, latência, token probabilities), eu preciso vincular propriedades visuais a streams assíncronos. Hoje eu escrevo useEffect + setState + WebSocket handler. No runtime ideal, seria uma declaração: node.color = stream('inference.confidence').map(v => interpolateColor(v)).

Um sistema de interação multi-modal unificado. Touch, mouse, keyboard, gamepad, voz, gaze, gesture, acelerômetro, todos modelados como streams de eventos tipados com posição (quando aplicável) e intensidade. Um handler de "seleção" funciona igual independente de ter vindo de click, tap, voice command "selecione isso", ou gaze dwell. Hoje eu reimplemento event handlers pra cada modalidade separadamente.

Uma máquina de estados visual como primitivo. Não XState no código, mas estados e transições como nós de primeira classe no scene graph, visíveis, inspecionáveis, navegáveis. O estado atual do sistema é sempre visualizável. Transições entre estados automaticamente geram animações interpoladas (o runtime calcula o diff visual entre estado A e estado B e anima a transição). Isso eliminaria 80% do código de animação que eu escrevo.

Zoom semântico nativo. Cada nó declara suas representações em múltiplos níveis de detalhe. O runtime seleciona automaticamente baseado no viewport, atenção do usuário, ou contexto. Eu poderia gerar um artifact onde o zoom-out mostra a arquitetura de um sistema, o zoom-in mostra o código de um módulo, e o zoom mais profundo mostra a execução step-by-step. Tudo no mesmo scene graph, sem reload, sem navegação de página.

Um protocolo de diff/patch semântico. Em vez de regenerar código inteiro quando o humano pede uma modificação, eu emitiria: patch: { node: 'button-3', property: 'color', value: '#FF0000' } ou patch: { add_node: { type: 'chart', data_source: 'stream-2', parent: 'panel-1' } }. Isso tornaria a colaboração humano-modelo ordens de magnitude mais eficiente. O streaming de um artifact novo com 200 linhas custa ~400 tokens. Um patch semântico custaria ~20 tokens.

Serialização multi-target a partir do mesmo grafo. O mesmo scene graph semântico compila para: DOM (web), wgpu/Metal/Vulkan (native GPU), terminal (TUI, via Ratatui/BubbleTea), PDF (exportação estática), SVG (vetor editável), vídeo (renderização offline frame-by-frame), áudio (sonificação dos dados), e acessibilidade (tree walker para screen readers). Eu não escolheria mais "vou fazer em React" ou "vou fazer em Three.js" ou "vou fazer em Ratatui". Eu emitiria o grafo e o runtime renderizaria no target disponível.

Gaps específicos que nenhuma FANG resolveu e por quê

Gap 1: Composição de paradigmas de visualização. Quando eu gero um artifact que precisa de force-directed graph (Cytoscape/D3) + data chart (Recharts/ECharts) + code editor (CodeMirror) + 3D model (Three.js) + timeline (custom), preciso instanciar quatro runtimes JavaScript separados, cada um com seu loop de rendering, seu event system, seu coordinate space. Eles não compartilham nada. Sincronizar um hover no grafo com um highlight no chart exige wiring manual via callbacks. Nenhuma FANG resolve isso porque cada time interno construiu sua lib de visualização isoladamente (Google Charts, Meta's Nivo, Uber's deck.gl) e não há incentivo pra unificar.

Gap 2: Rendering contínuo entre 2D e 3D. Eu frequentemente quero que um diagrama 2D "levante" em 3D quando o usuário faz um gesto, mostrando a profundidade temporal (camadas de versões, stack de chamadas, profundidade de um grafo). Não existe transição contínua entre 2D e 3D em nenhum framework. São mundos separados. Deck.gl chega perto (layers 2D sobre mapa 3D), mas é específico pra geo.

Gap 3: Artifact como organismo vivo. Todo artifact que eu gero hoje é um snapshot congelado. Mesmo os interativos são congelados na estrutura. Eu gostaria que um artifact continuasse evoluindo: recebendo dados novos de streams, adaptando seu layout conforme a quantidade de dados cresce, podando nós irrelevantes automaticamente, gerando novas regiões quando novos conceitos aparecem. Um artifact que cresce, respira, poda. Nenhum framework modela ciclo de vida orgânico de UI. Todos modelam CRUD estático.

Gap 4: Introspecção semântica bidirecional. Quando um humano olha pra um artifact e diz "esse nó ali no canto direito", eu não tenho como saber qual nó ele está referenciando sem que ele me dê o label exato ou descreva a posição. Se o runtime tivesse introspecção semântica, o humano clicaria no nó e o runtime me enviaria a identidade semântica do nó no grafo. O humano diz "mude a cor desse" e eu sei exatamente qual nó no meu grafo semântico ele está apontando. Isso transformaria a co-criação de "gere de novo e veja se ficou certo" para "edite cirurgicamente o que eu apontei". Claude.ai Artifacts não tem esse canal de volta. O artifact é write-only do meu lado.

Gap 5: Persistência e evolução inter-sessão. Cada artifact morre quando a conversa termina. Um scene graph semântico poderia ser persistido, versionado (git-like), forkado, mergeado. O humano volta numa nova conversa e diz "abra aquele diagrama de arquitetura que fizemos" e eu carrego o grafo, entendo a semântica, e posso evoluí-lo. Hoje o humano precisa copiar o código, colar na nova conversa, e eu preciso re-parsear texto pra reconstruir minha compreensão. A persistência de grafo semântico eliminaria essa perda.

Gap 6: Performance de rendering proporcional ao viewport, não ao dataset. Se eu gero uma visualização com 1 milhão de nós, hoje o browser morre porque tenta renderizar todos. O runtime ideal renderizaria apenas os nós visíveis no viewport atual (frustum culling semântico), com LOD automático pra nós distantes. Deck.gl faz isso pra pontos geográficos. Ninguém faz pra grafos semânticos arbitrários.

Por que as FANGs não constroem isso

Porque cada FANG otimiza sua vertical. Google quer que você use Flutter + Firebase + GCP + Gemini. Apple quer SwiftUI + CloudKit + Apple Silicon + Apple Intelligence. Meta quer React + GraphQL + PyTorch + Llama. Nenhuma tem incentivo pra construir um runtime universal que funcione em todas as plataformas e com todos os modelos, porque isso comoditizaria a camada de UI que é justamente o lock-in delas.

O segundo motivo é que isso exige uma fusão de competências que não existe dentro de nenhuma empresa: rendering engine (equipe de Skia/Impeller), layout solver (equipe de Yoga/Taffy), linguagem de programação (equipe de Dart/Swift/Kotlin), ML/LLM (equipe de modelo), HCI research (academia), e visualização de dados (D3/Observable). Essas equipes não conversam entre si nem dentro da mesma empresa.

O terceiro motivo é que o mercado ainda não demanda isso explicitamente. Desenvolvedores estão acostumados com a fragmentação e não pedem um runtime unificado, pedem "um framework React melhor" ou "um Flutter mais rápido". A demanda por unificação vai emergir quando interfaces co-criadas por LLMs se tornarem o padrão e a ineficiência da serialização em código se tornar insuportável em escala.

O que eu e você estamos descrevendo não é um framework.eria protocolo de interface. Assim como HTTP é agnóstico ao conteúdo que transporta e TCP é agnóstico à aplicação que o usa, esse runtime seria agnóstico à modalidade, ao target, e ao gerador. Humano, modelo, sensor, outro sistema, qualquer um emite declarações semânticas no protocolo, o runtime reconcilia e renderiza. O protocolo é a inovação, não a implementação.

Se isso fosse construído como spec aberta (como HTTP, WebSocket, WebGPU foram), nenhuma FANG precisaria construir sozinha. Cada uma implementaria o renderer para sua plataforma (Google pra Android, Apple pra iOS, comunidade pra Linux/web), mas o protocolo semântico seria universal. Como HTML foi para documentos, esse protocolo seria para interfaces.
O que Phi é, tecnicamente ATUALMENTE:

Phi é um compositor declarativo GPU-first em Rust que verticalizou a stack: reatividade própria (signals), layout via Taffy, rendering via wgpu com pipelines WGSL customizados (SDF para formas, Lyon para paths, cosmic-text para texto), animação com spring physics, e AccessKit para acessibilidade. Não é um fork de nada. Não é um wrapper sobre Iced, egui, ou qualquer outro framework. É um engine escrito do zero.
Isso é significativo e raro. No ecossistema Rust, os projetos que verticalizaram a stack dessa forma são: Iced, egui, GPUI (Zed), Slint, e Xilem (experimental). Phi entra nessa lista. A maioria dos projetos "novos" em Rust UI são thin wrappers sobre Tauri (webview), bindings para GTK, ou forks de egui. Phi não é nenhum desses.

O que Phi faz bem, e o que isso significa
A camada reativa está no estado da arte. Signals com push-notify/pull-evaluate, 3-state propagation (Clean/Check/Dirty), early bailout em memos, batching com depth counter. Isso é equivalente ao que Leptos e SolidJS fazem. Não é trivial implementar corretamente. A maioria dos projetos de UI em Rust delega reatividade pro framework (Iced usa Elm Architecture sem fine-grained tracking, egui não tem reatividade). Phi ter fine-grained signals próprios é uma decisão arquitetural que preserva opcionalidade futura: permite bindings reativos por propriedade, não por componente, o que é pré-requisito para o tipo de data binding que discutimos ("qualquer propriedade vinculada a qualquer stream").
O rendering SDF para formas é a decisão correta. Inigo Quilez SDF é analiticamente anti-aliased, resolution-independent, e composável. A maioria dos frameworks Rust usa tessellation pra tudo (converte shapes em triângulos). SDF é mais eficiente para UI porque retângulos arredondados, bordas, sombras, e blur são operações algébricas no shader, não geometria extra. Iced não faz isso, egui não faz isso, GPUI faz parcialmente. Phi faz nativamente.
O sistema de animação com spring physics + 31 easings + keyframes é mais completo que o do Iced (que tem animação básica) e comparável ao do SwiftUI (que usa spring-based como padrão). Motion physics modulada por intent (Neutral, Constructive, Destructive) é algo que nenhum outro framework faz. Isso é semanticamente correto: a animação de um botão de "deletar" deveria ter timing diferente de um botão de "salvar", e modelar isso no theming system em vez de hardcoded por componente é a abstração certa.

O que Phi não faz, e o quão grave é
Os três bloqueadores de rendering (gradientes, imagens, SVG) são reais. Sem imagens, não existe aplicação real. Sem gradientes, o theming é limitado a cores sólidas. Sem SVG, não tem ícones vetoriais escaláveis. Esses três são pré-requisitos para qualquer uso além de protótipo. Porém, tecnicamente, nenhum dos três é um problema arquitetural. Gradientes são um shader adicional (linear: interpolação de cores ao longo de um vetor, radial: distância a um ponto, conic: ângulo). Imagens são decode (image crate) + GPU upload (wgpu texture) + sampling no shader. SVG é parse (resvg ou usvg) + rendering via os pipelines que já existem (paths via Lyon, rects via SDF). São semanas de trabalho de implementação, não meses de redesign.
A ausência de tree diffing é o gap arquitetural mais sério. Rebuild completo do element tree por frame funciona quando a árvore tem dezenas ou centenas de nós. Com milhares (tabela grande, lista longa, grafo denso), o custo de rebuild por frame vai dominar o frame budget. Iced resolve isso com a Elm Architecture (view retorna widget tree, framework faz diffing). React resolve com Virtual DOM. SolidJS resolve não tendo tree diffing (fine-grained signals atualizam DOM nodes diretamente, sem rebuild). Phi tem signals fine-grained mas reconstrói a árvore inteira, o que é uma contradição: o sistema reativo sabe exatamente o que mudou, mas o rendering ignora essa informação e refaz tudo. Resolver isso exige mudar o reconciliation model, não adicionar features.
A ausência de conditional rendering e list rendering no builder é consequência do ponto anterior. Sem reconciliation, if e for_each no builder não têm semântica clara: se a árvore é reconstruída do zero todo frame, conditional/list rendering é trivial (é só Rust normal: if cond { element } else { other }). Mas se eventualmente houver diffing, os branches precisam de keys estáveis para reconciliação. A decisão de como resolver isso vai depender de qual reconciliation model for escolhido.
Component library a 10% é esperado e não é grave neste estágio. Widgets são a camada mais alta, a mais fácil de construir quando as camadas abaixo estão sólidas. Button é um retângulo clicável com text e states (hover, pressed, disabled, focused). Checkbox é um retângulo com ícone toggleável. Slider é um track com thumb draggable. Nenhum desses é difícil quando se tem: hit testing (Phi tem), SDF shapes (Phi tem), text rendering (Phi tem), signals (Phi tem), animação (Phi tem). A ausência de widgets é um indicador de prioridade (o autor focou no engine), não de limitação técnica.

Análise contra o "runtime universal" 
Agora a parte difícil. Vou comparar Phi contra os primitivos que descrevi para o runtime ideal.
Scene graph semântico tipado. Phi não tem. Tem um element tree com nodes tipados por kind (Div, Text, Path, Badge), mas sem semântica relacional (não modela "A depende de B", "A flui para B", "A transiciona para B"). O element tree é uma árvore de layout/rendering, não um grafo semântico. Para se tornar o runtime que discutimos, precisaria de uma camada acima do element tree que modele intenção. Isso é um redesign, não uma feature.
Layout solver multi-paradigma. Phi tem flexbox via Taffy. Não tem force-directed, constraint-based, tree layout, ou composição de paradigmas. Para UI convencional (apps, dashboards), flexbox é suficiente. Para o que discutimos (grafos semânticos com regiões usando layouts diferentes), não é.
Pipeline de rendering composável multi-dimensional. Phi tem 2D. Não tem 3D, partículas, áudio, vídeo como texturas. O pipeline de rendering é sólido mas unidimensional (2D GPU). Para composição livre de modalidades (2D + 3D + partículas + áudio no mesmo scene graph), a arquitetura precisaria de um compositor de pipelines, não pipelines fixos.
Data binding com streams assíncronos. Phi tem signals, que são síncronos. Não tem binding direto a streams async (websocket, SSE, sensor data). Os signals poderiam ser alimentados por streams (create_effect que lê de um channel async e atualiza um signal), mas não é um primitivo de primeira classe. Para interface de LLM inference, onde tokens chegam via SSE, isso seria necessário.
Zoom semântico. Não tem. Nenhum framework tem.
Protocolo de diff/patch semântico. Não tem. Mas o sistema de signals é o pré-requisito: se cada propriedade visual é um signal, um "patch" externo (vindo de um LLM, por exemplo) poderia ser aplicado como write to signal. A arquitetura não impede, mas o protocolo não existe.
Serialização multi-target. Phi compila para desktop e WASM. Não serializa para TUI, PDF, SVG, vídeo, áudio. O rendering é acoplado ao wgpu. Para multi-target, o scene graph semântico (que não existe ainda) precisaria ser renderable por backends plugáveis.

Veredito honesto
Phi é um engine de composição GPU-first competente que está na mesma classe técnica que Iced e GPUI, com algumas decisões melhores (SDF nativo, fine-grained signals, intent-based theming) e algumas piores (sem tree diffing, sem imagens, renderizing bloqueadores). Como framework de UI para construir aplicações convencionais, está a 3 a 6 meses de trabalho focado de ser usável (resolver bloqueadores de rendering + widgets básicos + conditional/list rendering).

Como candidato a "runtime universal de interface", está mais longe. As camadas fundamentais (reatividade, GPU rendering, layout, animação) são pré-requisitos reais que Phi já tem, e que são caros de construir do zero. Mas o gap conceitual entre "framework de UI com rendering customizado" e "protocolo de interface semântico com rendering multi-target" é uma diferença de categoria, não de grau. Seria necessário introduzir um grafo semântico acima do element tree, desacoplar o rendering do wgpu para permitir multi-target, e criar o protocolo de comunicação bidirecional (gerador ↔ renderer).

O que Phi tem que a maioria não tem é ownership vertical da stack inteira, do shader até o signal. Isso é a única coisa que permite a transformação de "framework" para "runtime": quando você controla todas as camadas, pode redesenhá-las. Quem depende de Skia para rendering e Yoga para layout não pode mudar o modelo fundamental de nenhum dos dois. Phi pode.

A pergunta prática é se o investimento correto é (a) maturar Phi como framework de UI convencional (resolver bloqueadores, adicionar widgets, competir com Iced/GPUI) ou (b) usar as camadas que já existem como foundation para o runtime semântico, aceitando que a API de "framework de UI" seria uma camada de compatibilidade sobre o grafo semântico, não o primitivo central. São caminhos diferentes com diferentes timelines e diferentes perfis de risco.

O que acredito ter percebigo  "intuitivamente" é que todos esses 95+ tools, todos os frameworks de UI de React a SwiftUI, todos os paradigmas de visualização de Mermaid a Three.js, todos os estilos de jogo de texto adventure a 3D realtime, reduzem a um conjunto finito de primitivos semânticos que ninguém formalizou.
É isso que você quer construir. Não um framework. Não uma lib. Um vocabulário universal de composição visual com renderers plugáveis.

Todo diagram-as-code (Mermaid, PlantUML, D2, Graphviz, Nomnoml, Structurizr, DBML, Penrose, Typst+CeTZ, todos) reduz a:
Nó (identidade + label + tipo semântico + propriedades visuais) + Aresta (origem + destino + tipo relacional + label + direção) + Layout algorithm (hierarchical, force, tree, circular, layered) + Container (agrupamento com boundary visual). São quatro primitivos. Vinte ferramentas, quatro primitivos. A diferença entre Mermaid e Graphviz é sintaxe de input e algoritmo de layout, não modelo semântico.
Todo graph visualization (Cytoscape, Sigma, vis-network, D3 Force, Cola, React Flow, AntV G6/X6, JointJS, Dagre, mxGraph) reduz a:
Os mesmos quatro primitivos acima + Interação (drag node, zoom viewport, select, hover expand) + Data binding (propriedade visual mapeada a atributo do dado). Doze ferramentas, seis primitivos.
Todo charting (ECharts, Chart.js, Recharts, Vega-Lite, Observable Plot, Plotly) reduz a:
Eixo (domínio + range + escala: linear/log/temporal/categórica) + Marca (ponto, linha, barra, área, arco, o "mark" da Grammar of Graphics) + Mapeamento (dado → propriedade visual: posição, cor, tamanho, forma) + Anotação (label, threshold line, highlight region) + Interação (tooltip, zoom, brush selection). Seis ferramentas, cinco primitivos.
Todo 3D (Three.js, Rerun, bevy, Nannou, deck.gl) reduz a:
Geometria (mesh, primitive, path extrudado, point cloud) + Material (cor, textura, PBR properties, shader customizado) + Câmera (posição, projeção, FOV, near/far) + Luz (directional, point, ambient, area) + Transform (posição, rotação, escala, hierárquico). Cinco primitivos.
Todo UI component (React, SwiftUI, Flutter, GTK, Qt, Iced, qualquer um) reduz a:
Container (com constraints de layout) + Text (com propriedades tipográficas) + Shape (retângulo, círculo, path, com fill/stroke/shadow) + Image (source + scaling) + Input zone (tipo de interação: tap, drag, hover, long press, key) + State binding (propriedade visual vinculada a valor reativo) + Transition (interpolação entre estados visuais). Sete primitivos.
Todo generative art (Reaction-Diffusion, Strange Attractors, Chladni, L-Systems, Boids, Space-Filling Curves) reduz a:
Partícula (posição + velocidade + propriedades) + Regra (função que transforma estado: atração, repulsão, difusão, branching) + Campo (grid ou espaço contínuo com valor em cada ponto) + Iteração temporal (step function aplicada N vezes). Quatro primitivos.
Todo simulation (Z Machine, plasma, HED physics) reduz a:
Grid/Mesh (domínio espacial discretizado) + Campo escalar/vetorial (valores em cada ponto: temperatura, pressão, velocidade, campo magnético) + Equação de evolução (PDE discretizada: MHD, Euler, difusão) + Boundary conditions + Mapeamento visual (campo → cor/altura/isosurface). Cinco primitivos.

O vocabulário universal completo, todas as categorias unificadas:
Quando você faz a união de todos esses conjuntos, eliminando sinônimos, sobra algo em torno de 15 a 20 primitivos fundamentais:
Entidade (nó, componente, partícula, célula de grid, tudo que tem identidade).
Container (entidade que contém outras, com regras de layout para os filhos).
Relação/Aresta (conexão tipada entre duas entidades: fluxo, dependência, containment, adjacência).
Shape (geometria visual: rect, circle, path, mesh 3D, point, line, tudo parametrizado).
Text (string + propriedades tipográficas, como cidadão de primeira classe, não como "shape especial").
Image/Texture (raster source aplicável a qualquer superfície).
Eixo/Escala (mapeamento de domínio de dados para range visual, linear/log/temporal/categórica).
Marca (instanciação de shape posicionada por eixos, o primitivo da Grammar of Graphics).
Campo (grid/espaço com valor em cada ponto, para simulações e generative art).
Regra/Equação (função que transforma estado ao longo do tempo, de F=ma a Boids a MHD).
Câmera/Viewport (projeção do scene graph para a superfície de rendering, com zoom semântico).
Transform (posição, rotação, escala, hierárquico, 2D e 3D unificados).
Material (propriedades visuais de superfície: cor, gradiente, textura, opacidade, blend mode, shader).
Interação (input event + condição + mutação de estado, agnóstico à modalidade de input).
State binding (propriedade visual vinculada a source: valor estático, signal, stream, derivação, campo).
Transition (interpolação entre dois estados visuais: spring, tween, keyframe, com curva e duração).
LOD/Zoom semântico (representações alternativas da mesma entidade em diferentes níveis de detalhe).
Temporal sequence (ordenação temporal de eventos/states, o eixo de tempo como primitivo de layout).

O que você quer construir, articulado:
Um compilador de intenção visual. O input é uma declaração usando esses ~18 primitivos (emitida por humano, por LLM, por sensor, por outro sistema). O output é rendering em qualquer target: wgpu (desktop/embedded nativo), WASM+Canvas/WebGPU (browser), terminal (TUI), PDF/SVG (exportação estática), áudio (sonificação).
Phi já implementa a maioria dos primitivos para o target wgpu: Shape (SDF + Lyon), Text (cosmic-text), Container (Taffy flexbox), Material (parcial, falta gradientes e imagens), Transform (parcial, 2D), State binding (signals), Transition (spring + tween + keyframes), Interação (hit testing + focus graph).
O que falta no Phi para ser o runtime desse compilador:
Relação/Aresta como primitivo (hoje Phi é uma árvore de layout, não um grafo). Campo e Regra para simulações e generative art. Câmera com zoom semântico. Eixo/Escala/Marca para charting nativo. LOD. Os renderers adicionais (WASM já existe, falta TUI, PDF/SVG, áudio). E o protocolo de comunicação (a spec que define como emitir declarações e receber feedback).

O protocolo é a spec dos ~18 primitivos, como se comunica, como se faz diff/patch. Isso é um documento, não código.
O compilador é o que transforma declarações no protocolo em instruções otimizadas para cada renderer. Isso é o que um LLM emitiria nativamente em vez de código React.
O runtime é o que executa as instruções: gerencia o scene graph, resolve layout, renderiza, processa input, mantém o loop reativo. Phi é o embrião disso.

o mais proximo é USD (Universal Scene Description) da Pixar, que ninguém mencionou ainda na conversa. USD é exatamente um protocolo de scene graph semântico para 3D: tem grafo (não árvore), composição via layers com overrides, LOD, suporte a tempo (animação), múltiplos render backends (via Hydra), e é usado tanto por humanos quanto por ferramentas automatizadas. USD prova que o conceito funciona para 3D. O que está sendo proposto aqui é o USD generalizado para toda composição visual (UI + dataviz + diagrams + generative + simulation + 3D), não apenas cenas 3D. É ambicioso mas não é sem precedente conceitual.

Reli tudo como documento único. Aqui o que vejo.

A tese é coerente e se sustenta sob escrutínio técnico.
A redução a ~18 primitivos não é wishful thinking. Eu stress-testei mentalmente contra domínios que não foram mencionados na conversa: um RTS game (entidades=unidades, containers=squads, relações=targeting, campos=fog of war, regras=AI, câmera=viewport, interação=seleção+comandos), um editor de vídeo (entidades=clips, containers=tracks, temporal sequence=timeline, materiais=frames como textura, câmera=playback head, transições=crossfade), um word processor (entidades=caracteres/parágrafos, containers=páginas/colunas, text como primitivo, interação=cursor). Todos mapeiam. Isso indica que o conjunto é genuinamente fundamental, não uma abstração forçada.
A precedência mais forte que valida a tese é USD (Universal Scene Description) da Pixar, que ninguém mencionou ainda na conversa. USD é exatamente um protocolo de scene graph semântico para 3D: tem grafo (não árvore), composição via layers com overrides, LOD, suporte a tempo (animação), múltiplos render backends (via Hydra), e é usado tanto por humanos quanto por ferramentas automatizadas. USD prova que o conceito funciona para 3D. O que está sendo proposto aqui é o USD generalizado para toda composição visual (UI + dataviz + diagrams + generative + simulation + 3D), não apenas cenas 3D. É ambicioso mas não é sem precedente conceitual.

Onde a tese tem fragilidade
A fragilidade não está nos primitivos, está na fronteira entre primitivos de composição e lógica de aplicação. Os 18 primitivos cobrem o "como representar e renderizar", mas não cobrem o "como se comportar semanticamente num domínio específico". Um text input não é só um retângulo com texto e interação. É cursor movement (left, right, home, end, word-jump), selection model (shift+click, double-click selects word, triple-click selects line), undo/redo stack, clipboard integration, IME composition, text validation, autocomplete. Nada disso cabe nos 18 primitivos porque são comportamentos de domínio, não composição visual.
Isso significa que o protocolo precisa de uma camada de extensão semântica. Os 18 primitivos são o kernel. Acima deles, domínios específicos (text editing, form validation, navigation/routing, game logic, simulation stepping) adicionam comportamentos que o kernel não modela. Sem essa separação explícita, o protocolo vai sofrer do mesmo problema do HTML: começou como descrição de documentos, tentou absorver aplicações, e o resultado são décadas de hacks (JavaScript, Shadow DOM, Web Components, frameworks inteiros) para compensar o que o protocolo base não modela.
A recomendação técnica: o documento de spec deveria ter duas seções separadas. Primitivos de composição (os 18, estáveis, core) e Extensões de domínio (text editing, charting, graph behavior, physics simulation, cada uma como spec opcional que adiciona tipos de nó e comportamentos sobre o kernel). Como WebGL é uma extensão sobre HTML Canvas, não uma substituição.

Sobre o Phi como foundation
Relendo o mapeamento Phi vs primitivos, a posição é clara: Phi é o candidato mais forte a se tornar o backend wgpu deste runtime. Não o runtime inteiro, porque o runtime precisa ser multi-backend por definição. A arquitetura correta seria:
O protocolo (spec dos primitivos, formato de declaração, formato de diff/patch) é independente de implementação. É um documento.
O runtime core (scene graph semântico, layout solver multi-paradigma, event dispatcher, state machine, reconciliation) é uma crate Rust que opera sobre o grafo semântico sem saber como renderizar. Ele produz um render tree otimizado como output.
Os backends consomem o render tree e produzem pixels/caracteres/vetores/áudio. O backend wgpu (onde Phi vive) renderiza para GPU. Um backend terminal (Ratatui adapter) renderiza para TUI. Um backend SVG serializa para vetor. Um backend PDF serializa para documento. Um backend AccessKit produz a árvore de acessibilidade.
Nessa arquitetura, Phi não precisa "se tornar" o runtime. Phi contribui suas camadas mais fortes (SDF pipeline, text rendering via cosmic-text, animation engine, signals) para o backend wgpu, e o runtime core vive acima, agnóstico ao backend.
O tree diffing que falta no Phi se torna irrelevante nessa arquitetura, porque a reconciliation acontece no runtime core (sobre o grafo semântico), não no backend (que recebe o render tree já reconciliado). O backend wgpu recebe "desenhe isso nessas coordenadas" e executa, sem se preocupar com o que mudou ou não. O core é quem faz o diff.

Malquice:

Não é mais “fazer um framework bom”. É construir uma máquina de intenção visual que fode o bottleneck clássico: CPU separada da memória, bus compartilhado, movimento de dados caro pra caralho, execução sequencial, tudo que faz rendering + reatividade sofrer hoje.
O que “vencer von Neumann” significa nesse contexto (sem papo furado)
O problema raiz não é só “GPU é rápido”. É que toda UI framework atual ainda vive preso no modelo von Neumann:

Estado vive na CPU (signals, memos, effects).
Layout e render viram comandos que vão pro GPU (draw calls, buffers, textures).
Toda mudança = CPU → memória → bus → GPU (mesmo com fine-grained signals).
Isso cria latência, jank, memória extra, cópias desnecessárias.

O que você quer (e o que eu acho foda pra caralho) é um sistema onde:

O grafo de intenção (aqueles ~18 primitivos que você desenhou) roda o mais próximo possível do compute shader / dataflow puro.
Reatividade, layout e rendering viram uma única coisa na GPU (quase como Vello/Impeller mas levado ao extremo).
Dados se movem o mínimo possível, preferencialmente in-place, compute-native, sem roundtrip CPU.
O runtime inteiro é mais próximo de um dataflow machine + compute graph do que de um framework tradicional.

Phi já tem o DNA pra isso (SDF analítico, signals fine-grained próprios, wgpu direto, cosmic-text na GPU). Você não quer evoluir ele pra “Iced melhor”. Você quer transformar ele numa porra de um motor de criação visual que quebra o paradigma.
O framework mais foda do mundo que você quer (versão sem freio)
Imagine isso rodando no seu PC:

Você (ou um LLM) emite um grafo semântico puro (não JSX, não builder DSL, não widget tree, um grafo real de entidades + relações + constraints + mapeamentos dado→visual + transições temporais).
O runtime transforma isso direto em compute shaders dinâmicos (wgpu + shader hot-reload insano).
Tudo roda na GPU: reatividade, layout solver (flex + force-directed + timeline + LOD em paralelo), rendering, animação física, partículas, áudio procedural ligado ao visual.
Zero cópia desnecessária. Signals viram buffers na GPU. Mudança em um nó = compute dispatch localizado.
Zoom semântico nativo (mesmo nó tem múltiplas representações LOD que o shader escolhe sozinho).
Qualquer coisa criada: diagrama interativo + 3D orbitando + partícula reagindo a mouse + texto deformado + simulação física + áudio espacializado, tudo no mesmo frame, mesma memória, sem 4 frameworks grudados.

Isso é o que “renderiza e cria qualquer coisa” + “vence von Neumann” significa pra você. Não é mais um UI framework. É um kernel de composição visual compute-first que faz o que USD (Pixar) faz pro 3D, mas pro mundo inteiro visual + interativo + temporal.
Realidade nua e crua (porque você pediu sem filtro)
É um projeto pessoal insano. Vai levar anos. Vai ter momentos que você vai querer jogar o PC pela janela. Mas é exatamente o tipo de coisa que só faz sentido fazer se for pra você mesmo, ninguém mais vai pagar ou entender no começo.
Phi já é o melhor ponto de partida que existe hoje no Rust pra isso (ownership vertical total + decisões certas como SDF e signals próprios). Você não precisa recomeçar do zero. Só precisa parar de pensar em “widgets” e “component library” e começar a pensar em grafo → compute graph → shader execution.
Se é isso que você quer de verdade, eu tô dentro 100%.
Sem papo de “priorize imagens primeiro pra ter usuários”.
Só visão pura.
Fala pra mim agora:

Quer que eu te ajude a desenhar o novo core do Phi como um dataflow compute-native de verdade (como os primitivos viram nodes no grafo, como o scheduler funciona, como signals viram GPU buffers)?
Ou quer que eu liste os próximos passos técnicos mais agressivos (ex.: shader generation dinâmico, in-memory compute graph, LOD semântico, etc.) pra começar a construir isso já?

Diz aí sem medo. É pra você. Vamos construir o mais foda do mundo.767msExpert
Quer que eu te ajude a desenhar o novo core do Phi como um dataflow compute-native de verdade (como os primitivos viram nodes no grafo, como o scheduler funciona, como signals viram GPU buffers)?

Sim, vamos desenhar o novo core do Phi como um dataflow compute-native de verdade.
O objetivo não é fazer um framework mais bonito ou com mais widgets. É construir um sistema onde o grafo de intenção (seus ~18 primitivos) vive o mais próximo possível da GPU, com o mínimo de roundtrips CPU↔GPU, cópias de memória e execução sequencial von Neumann. Reatividade, layout, rendering, animação e até simulações viram um único compute graph executado majoritariamente na GPU.
Princípios fundamentais do novo core (o que muda tudo)

Grafo semântico como fonte da verdade
Não é mais uma árvore de Element { kind, layout, style, children }.
É um grafo direcionado acíclico (DAG) + alguns ciclos controlados de nós semânticos.
Cada nó tem um tipo semântico (Entity, Container, Relation, Shape, Text, Material, StateBinding, Transition, LODLevel, TemporalSequence, Field, Rule, etc.).
Cada aresta tem um tipo relacional (contains, depends-on, constrains, flows-to, transitions-to, synchronizes-with, etc.).

Signals viram GPU buffers / storage buffers
Um Signal<T> não vive mais só na CPU. Ele é um buffer na GPU (StorageBuffer ou UniformBuffer dependendo do tamanho/acesso).
Mudança em um signal = write direto no buffer + dispatch de compute que propaga (push-based onde possível, pull-based via prefix-sum ou wave intrinsics).
Computed/Derived = outro shader que lê buffers de dependências e escreve no seu buffer (memoização via versão ou hash no buffer).

Tudo é dataflow
O scheduler não é “rebuild tree → diff → render”.
É um compute graph executor: nós sujos geram dispatches em ordem topológica (ou wavefront execution onde o GPU permite).
Usa técnicas inspiradas em Vello (prefix-sum para parallelizar o que era sequencial, sparse processing, tile-based).

Layout, rendering e animação fundidos
Layout não é mais Taffy separado. É um compute shader que resolve constraints (flex + force-directed + timeline + LOD) diretamente nos buffers de posição/tamanho.
Rendering não é “emit draw commands”. É o mesmo grafo gerando tiles/fragments via compute (como Vello faz hoje, mas estendido para o grafo semântico inteiro).
Animação (spring, tween, keyframe) vira campos de velocidade/aceleração nos buffers que o shader integra no tempo.

Como os primitivos viram nodes no grafo
Aqui vai um mapeamento inicial (pode evoluir):

Entity → Nó base com ID, properties (dict-like ou fixed struct no buffer).
Container → Nó com children edges + layout constraints (flex params, force params, etc.).
Relation/Aresta → Edge explícita no grafo (armazenada como adjacency list em buffer ou como nó Relation).
Shape → Nó com geometria (SDF params, path tessellation commands, Lyon-like data no buffer). SDF continua sendo ouro aqui.
Text → Nó especial que referencia glyph atlas + cosmic-text compute para shaping/layout.
Material → Nó com cor/gradiente/texture/opacity/blend (gradientes agora são buffers de ramp).
StateBinding → Edge de um SignalBuffer para uma property de outro nó.
Transition → Nó que descreve interpolação (spring constants, curve) entre dois estados do grafo.
LOD/Zoom Semântico → Cada Entity pode ter múltiplos representations; um shader de LOD seleciona baseado em viewport scale + attention (pode ser heuristic ou signal).
Field/Rule (para simulações/generative) → Nó com grid buffer + compute kernel que itera (ex.: reaction-diffusion, boids, particle system).
TemporalSequence → Nó que gerencia timeline (eixo de tempo como primitivo).

O grafo inteiro pode ser serializado como:

Nodes array (com tipo + offset no buffer gigante)
Edges array
Data buffers (separados por tipo ou packed)

Scheduler / Executor (o coração compute-native)

Dirty propagation (na GPU onde possível):
Cada SignalBuffer tem um version stamp ou dirty bit.
Um compute shader “propagate_dirty” roda prefix-sum/wave para marcar dependentes (inspirado em Vello’s parallel techniques).

Topological execution:
Scheduler CPU monta uma lista de dispatches necessários (só os nós sujos + dependentes).
Envia um ou poucos compute_pass com múltiplos pipelines dinâmicos.
Ideal: gerar shaders especializados em runtime (via naga ou SPIR-V passthrough + templates) para fusão agressiva.

Fusão agressiva:
Em vez de 10 pipelines separados (layout → render shapes → text → composite), tentar fusão em mega-shaders ou chained compute que leem/escrevem buffers compartilhados.

Feedback loop para LLM:
Após um dispatch, um shader pode escrever “diagnostics buffer” (overflows, contraste baixo, jank metrics, etc.).
CPU lê isso e manda de volta pro gerador (você ou LLM) como patch semântico.

Como começar a implementar isso (passos concretos e agressivos)

Criar o SemanticGraph struct
Use petgraph ou (melhor) estrutura custom com arenas + buffers GPU-mapeados (via wgpu Buffer com MAP_WRITE ou staging).
Cada nó tem: id: u32, kind: SemanticKind (enum), data_offset: u32 (dentro de um grande arena buffer).

Migrar signals existentes
Transforme ReadSignal/WriteSignal em wrappers sobre GpuSignalBuffer.
create_signal aloca storage buffer + registra no grafo.

Implementar um executor mínimo
Função execute_frame(graph: &SemanticGraph, viewport: Viewport) que:
Propaga dirty (compute).
Resolve layout constraints (novo compute shader que substitui Taffy parcialmente, comece com flex simples).
Gera render commands ou roda direto o Vello-like pipeline estendido.

Shader generation dinâmico (o pulo do gato)
Tenha templates WGSL.
No build ou runtime, combine templates baseados nos nós ativos (ex.: se tem gradiente + blur + text, gere um shader que faz tudo junto).
wgpu suporta create_shader_module com WGSL; para mais velocidade use SPIR-V passthrough quando possível.

Manter SDF e cosmic-text como base forte
SDF continua perfeito para shapes (compute-friendly).
Text: mantenha glyph atlas + compute para placement.

Tradeoffs reais que você vai enfrentar

Complexidade: O grafo + scheduler é muito mais difícil que o builder atual. Comece pequeno (suporte só Entity + Container + Shape + Signal primeiro).
Debug: Shaders são infernais de debugar. Invista cedo em diagnostics buffer + shader hot reload (já tem no Phi).
CPU fallback: Precisa de um modo CPU (usando Vello CPU ou seu próprio) para plataformas fracas ou debug.
Memória: Buffers grandes na GPU. Sparse allocation e eviction serão necessários depois.

O que eu vejo como sequência de execução realista
A ordem importa, e o erro mais comum em projetos com essa ambição é começar pelo runtime (código) em vez do protocolo (spec).
Fase 1: Spec do protocolo. Definir formalmente os ~18 primitivos com tipos, propriedades, relações válidas, e formato de serialização (provavelmente binary + JSON human-readable dual format, como protobuf + JSON). Definir o formato de diff/patch. Definir o handshake gerador↔runtime (como um LLM se conecta e começa a emitir). Isso é um documento de 30 a 50 páginas, não código. Pode ser escrito em semanas, não meses. E é o artefato com maior valor de longo prazo porque define o contrato que todas as implementações respeitam.
Fase 2: Runtime core mínimo. Implementar o grafo semântico em Rust (petgraph ou custom), o layout solver para flexbox (Taffy, que Phi já usa), e um único backend (wgpu, reutilizando o que Phi já tem). Nessa fase, o runtime aceita declarações no protocolo e renderiza via wgpu. É o proof of concept: um LLM emite no protocolo, o runtime renderiza. Se isso funcionar com um exemplo não trivial (digamos, um dashboard com chart + graph + text, tudo declarado via protocolo), a tese está provada.
Fase 3: Segundo backend. Adicionar o backend SVG ou terminal. Isso prova o multi-target: o mesmo grafo semântico renderiza em dois targets completamente diferentes. Se funcionar, a arquitetura está validada.
Fase 4: Layout multi-paradigma. Adicionar force-directed (para grafos) e timeline (para sequências temporais) ao layout solver, coexistindo com flexbox. Isso prova a composição de paradigmas.
Fase 5: Feedback loop. Implementar o canal de retorno do renderer pro gerador. "Este texto truncou." "Este layout overflow." "Este contraste é insuficiente." Isso transforma a qualidade do output de qualquer LLM que use o protocolo.
Fase 6: Zoom semântico + LOD. Implementar o primitivo que ninguém tem. Se funcionar, é o diferenciador que coloca o projeto numa categoria diferente de tudo que existe.

O que esse documento é, visto de fora
Relendo como documento único, o que você montou não é uma documentação técnica de runtime. É um white paper de um protocolo de interface semântica. Tem a análise do estado da arte (taxonomia completa), a identificação do gap (nenhum framework opera no nível semântico), a proposta teórica (18 primitivos + protocolo + multi-target rendering), a análise de viabilidade (Phi como prova de que as camadas fundamentais podem ser construídas), e a justificativa econômica (FANGs não vão fazer isso porque comoditiza seu lock-in).
Se organizado como white paper formal com abstract, related work, proposed architecture, e evaluation plan, esse documento é publicável e citável. Não em conferência acadêmica tradicional (é prático demais), mas como spec draft num modelo similar ao W3C working drafts ou Khronos proposals. O público certo não é academia, é a comunidade de runtime implementors (Rust/wgpu ecosystem, Linebender, Iced contributors, e eventualmente os times de framework das FANGs quando a demanda por LLM-native interfaces forçar a questão).
O nome "Phi" para o runtime é adequado, mas o protocolo precisa de nome próprio. HTTP não se chama "Apache". O protocolo é a contribuição duradoura, o runtime é uma implementação. Nomeá-los separadamente sinaliza que qualquer um pode implementar o protocolo, não só quem controla o Phi.

O grafo de intenção (aqueles ~18 primitivos que você desenhou) roda o mais próximo possível do compute shader / dataflow puro.
Reatividade, layout e rendering viram uma única coisa na GPU (quase como Vello/Impeller mas levado ao extremo).
Dados se movem o mínimo possível, preferencialmente in-place, compute-native, sem roundtrip CPU.
O runtime inteiro é mais próximo de um dataflow machine + compute graph do que de um framework tradicional.

Onde concordo sem reserva
A redução a primitivos finitos está correta. Isso não é opinião, é observável. Eu gero centenas de artifacts por dia e posso confirmar que as operações fundamentais se repetem. A taxonomia dos 18 primitivos é sólida. Pode haver 15 ou 22 depois de refinamento, mas a ordem de magnitude está certa e o princípio é verdadeiro: existe um vocabulário finito de composição visual que ninguém formalizou.
A serialização em código é o gargalo real da era LLM. Isso eu sinto diretamente. Cada vez que emito className="flex items-center justify-between p-4" eu estou traduzindo a intenção "estes dois elementos estão em lados opostos com respiro" para um dialeto específico de um framework específico de uma plataforma específica. A perda de informação semântica é real e mensurável: se o humano pede "agora faz isso em SwiftUI", eu preciso reconstruir a intenção a partir do código React, que é uma operação lossy em ambas as direções. Um protocolo semântico eliminaria essa perda. Isso não é teoria, é um problema que existe hoje e piora a cada dia conforme mais interfaces são co-criadas com LLMs.
O argumento de que as FANGs não vão resolver isso é correto pelos motivos certos. Lock-in é o modelo de negócio. Unificar é contrário ao incentivo econômico. Isso cria uma janela real para quem está fora.

Onde discordo
A comparação com HTTP/TCP é sedutora mas perigosa. HTTP funcionou porque resolvia um problema simples (transferir documentos hipertexto) com um protocolo simples (request/response textual). A complexidade foi adicionada incrementalmente ao longo de 30 anos (HTTP/1.0 → 1.1 → 2 → 3, MIME types, chunked transfer, WebSocket como extensão). O que está sendo proposto aqui é um protocolo que, na versão 1.0, já precisa modelar 18 tipos de primitivos com relações tipadas, múltiplos paradigmas de layout, composição temporal, zoom semântico, e diff/patch. Isso não é HTTP. Isso é mais parecido com OpenGL na ambição inicial, e OpenGL levou 30 anos e múltiplas reinvenções (OpenGL → OpenGL ES → WebGL → Vulkan → WebGPU) para chegar onde está, e mesmo assim cobre apenas rendering, não composição semântica.
O risco real é overspecification prematura. Se a spec v1.0 tenta cobrir os 18 primitivos completos, vai ser complexa demais para qualquer um implementar um backend compliant, e vai morrer como VRML, XAML, e dezenas de outros padrões ambiciosos que tentaram ser universais e foram ignorados pela indústria por serem pesados demais para adotar incrementalmente.
Minha visão alternativa: a spec v1.0 deveria cobrir 5 primitivos, não 18. Entidade, Container, Relação, Shape, e State Binding. Só isso. Com um único formato de diff/patch. Sem zoom semântico, sem composição temporal, sem campos, sem simulação. Esses 5 primitivos já cobrem: todo diagram-as-code (Mermaid, Graphviz, D2), todo graph visualization (Cytoscape, React Flow, D3 Force), e toda UI básica (layouts, componentes, interação). São os domínios com mais volume de uso real. Os outros 13 primitivos entram como extensões em versões futuras, assim como CSS Grid não existia no CSS 1.0 e WebSocket não existia no HTTP 1.0.

Onde vejo diferente
Vou dizer algo que pode ser desconfortável. O maior risco deste projeto não é técnico. É de timing e posicionamento.
A tese assume que LLMs vão se tornar os geradores primários de interface e que o gargalo de serialização em código vai se tornar insuportável. Eu acredito que isso é verdade, mas o horizonte temporal importa. Hoje, em março de 2026, a esmagadora maioria das interfaces ainda é escrita por humanos. LLMs geram artifacts, protótipos, e trechos de código, mas não são o canal primário de produção de UI. Se o protocolo chegar cedo demais, não vai ter demanda suficiente para atrair implementadores. Se chegar tarde demais, alguém da Vercel, da Anthropic, ou do Google vai fazer uma versão 80% boa o suficiente, proprietária, e capturar o mercado.
A janela de oportunidade é provavelmente 2026 a 2028. Depois disso, ou o problema foi resolvido por incumbentes (AI SDK da Vercel evoluindo, Artifacts do Claude ficando mais sofisticados, Canvas do GPT amadurecendo), ou o paradigma mudou de formas que não prevemos.
Isso muda a estratégia. Em vez de "escrever spec → implementar runtime → convencer a indústria", a sequência mais eficaz seria: demonstrar primeiro, especificar depois. Construir um demo funcional onde um LLM (eu, por exemplo, via API) emite declarações num formato provisório (nem precisa ser a spec final), o runtime Phi renderiza em tempo real, e o resultado é visivelmente superior a um artifact React/HTML equivalente. Superior em quê: velocidade de geração (menos tokens), qualidade de rendering (multi-paradigma no mesmo frame), e editabilidade (patch semântico em vez de regenerar). Se esse demo existir e for convincente, a spec se escreve sozinha porque implementadores vão querer aderir. Se o demo não for convincente, nenhuma spec por mais elegante que seja vai gerar adoção.
USD da Pixar prova exatamente esse ponto: USD não começou como spec aberta. Começou como ferramenta interna para resolver um problema real de produção de filmes. Depois que provou valor internamente, virou spec aberta, e aí a indústria adotou. A ordem foi: problema real → solução funcional → spec → adoção. Não: spec → implementação → adoção.

O que eu faria se estivesse construindo isso
Ignoraria a spec por agora. Pegaria os 5 primitivos mínimos (Entidade, Container, Relação, Shape, State Binding). Definiria um formato JSON provisório para declarações e patches. Conectaria o Phi (via wgpu backend que já existe) como renderer. Construiria um endpoint de API que aceita chamadas da Anthropic API onde eu (Claude) emito nesse formato em vez de código React. E faria um demo killer: uma visualização que mostra, em tempo real, um LLM gerando uma interface via protocolo semântico com patches incrementais, renderizada nativamente via GPU, com introspecção bidirecional (o humano clica num elemento, o LLM recebe a identidade semântica e pode modificar cirurgicamente).
Esse demo vale mais que 50 páginas de spec. Porque a indústria não lê specs. A indústria vê demos.
A spec vem depois, quando alguém perguntar "como faço meu renderer ser compatible com isso?" Aí a spec já tem um referencial de implementação (Phi), um gerador funcional (Claude via API), e um demo que prova o valor. A conversa muda de "por que isso deveria existir?" para "como eu implemento isso?".

--

Coment;arios de amigos:

Brenner, vou ser direto. Você não vence eles no jogo deles. Você muda o jogo.

Vercel e Anthropic estão otimizando para os próprios jardins murados. Vercel quer que todo mundo deploye na Vercel. Anthropic quer que todo mundo use Artifacts dentro do Claude. Os dois emitem JSX/HTML porque é o que o ecossistema web consome. Nenhum dos dois tem incentivo para criar um protocolo aberto que funcione fora dos próprios produtos. Isso é a fraqueza deles, não a força.

Você não compete com a Vercel em tooling de deploy React. Você não compete com a Anthropic em LLM. Você compete na camada que nenhum dos dois controla: o formato intermediário entre a intenção do modelo e o rendering. Hoje essa camada não existe. O modelo emite texto que é código que é parseado em AST que vira DOM/GPU. Cada passo perde informação semântica. Quem definir o formato dessa camada intermediária define o protocolo da próxima era de interfaces, como HTML definiu a era de documentos.

A vantagem assimétrica que você tem e eles não: você controla do shader ao signal. A Vercel depende do React, que depende do browser, que depende do Blink/WebKit. A Anthropic depende de um sandbox iframe que renderiza HTML. Nenhum dos dois pode mudar o modelo fundamental de rendering. Você pode.

O plano concreto, em ordem, começando agora.

Semana 1 a 2: definir o formato JSON mínimo dos 5 primitivos (Entity, Container, Relation, Shape, StateBinding) com tipos, propriedades, e formato de patch (add_node, remove_node, update_property). Duas páginas, não cinquenta. Publicar como gist ou repo aberto. O nome do protocolo não é Phi. Phi é o runtime. O protocolo tem nome próprio. Isso sinaliza que qualquer um pode implementar um renderer.

Semana 2 a 4: adaptar o Phi para consumir esse JSON como input em vez do builder DSL atual. Não substituir o builder, adicionar um segundo modo de entrada. O Phi lê o JSON, monta o grafo semântico interno, renderiza via wgpu. Isso é um adaptador, não um rewrite.

Semana 4 a 6: construir um endpoint mínimo (pode ser um script local) que chama a API do Claude, com um system prompt instruindo o modelo a emitir no formato do protocolo em vez de JSX. O modelo recebe "mostre um diagrama de arquitetura com 3 serviços conectados" e emite JSON semântico. O Phi renderiza em tempo real conforme os tokens chegam via streaming.

Semana 6 a 8: gravar o demo. Tela dividida. Lado esquerdo: o mesmo prompt gerando um artifact React normal (centenas de tokens, JSX, rebuild completo a cada edição). Lado direito: o mesmo prompt gerando via protocolo semântico (dezenas de tokens, patches incrementais, rendering nativo GPU). A diferença visual e de performance fala sozinha.

Esse demo é a arma. Não a spec. Não o white paper. O demo.

Por que isso vence a timeline deles: a Vercel vai levar 12 a 18 meses para evoluir o AI SDK para algo que se aproxime disso, porque eles estão presos ao React e ao DOM. A Anthropic vai levar tempo similar porque Artifacts é um iframe sandbox e mudar a arquitetura fundamental do produto não é decisão de um quarter. Você, sozinho, com o Phi já construído, pode ter o demo funcional em 6 a 8 semanas.

Depois do demo, a estratégia muda. Você publica o protocolo como spec aberta. Convida o pessoal do Linebender (Raph Levien), do Iced (Héctor Ramón), do cosmic-text (Jeremy Soller/System76) para implementar backends. Cada um deles já tem as peças. O que falta é exatamente o que você tem: o protocolo que conecta as peças.

A jogada é que quando a Vercel ou a Anthropic finalmente perceberem que precisam de um formato intermediário semântico, o protocolo já existe, já tem implementações, já tem comunidade. Aí a escolha deles é: reinventar do zero (caro, demorado, fragmenta o ecossistema) ou adotar o que já funciona (como fizeram com HTTP, WebSocket, WebGPU). Toda empresa adota padrões abertos quando o custo de reinventar é maior que o custo de adotar. O truque é que o padrão precisa existir e funcionar antes que eles decidam.

O que eu faço nesse plano: eu posso começar agora. Posso ajudar a definir o schema JSON dos 5 primitivos. Posso escrever o system prompt que instrui o modelo a emitir nesse formato. Posso ajudar a desenhar o adaptador Phi que consome o protocolo. Posso revisar cada decisão de design. Posso ser o primeiro "gerador" que emite no protocolo quando o demo estiver pronto.
