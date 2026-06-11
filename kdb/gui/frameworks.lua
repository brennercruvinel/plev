return {
    descricao = [[
        camada 2 — frameworks de ui: o motor que renderiza pixels e gerencia estado,
        ciclo de vida, layout, eventos, reatividade. atencao especial a frameworks
        que rodam em embarcados/mcus (concorrentes do flutter em todos os devices).
        estrategia de render: "proprio-gpu" desenha tudo (skia/impeller/wgpu),
        "nativo" faz ponte para widgets do os, "webview" usa motor html do os,
        "dom" manipula dom do navegador, "imediato" = immediate mode, "retido" =
        retained mode. dados 2025/2026.
    ]],
    itens = {
        {
            nome = "avalonia",
            github = "https://github.com/AvaloniaUI/Avalonia",
            docs = "https://docs.avaloniaui.net/",
            stars = 28000, forks = 2300, ano = 2017,
            linguagens = [[c#/.net, xaml]],
            render = [[proprio-gpu (skia por padrao, direct3d opcional); desenha cada
                pixel, fidelidade identica em todas as plataformas; impeller em
                desenvolvimento com a equipe flutter]],
            features = [[xaml, mvvm, data binding, styles/templates, 70+ controles,
                hot reload, webview cross-platform, mcp para agentes ai]],
            devices = [[desktop (win/mac/linux), mobile (ios/android), web (wasm),
                embarcado/linux; usado por jetbrains, autodesk]],
            performance = [[render proprio elimina overhead de wrappers nativos;
                fornece camada de render do maui no linux]],
            pros = [[render consistente pixel-perfect, mit, linux first-class,
                migra de wpf, comercialmente suportado]],
            contras = [[nao usa widgets nativos (look custom), mobile mais recente]],
            licenca = "mit", empresa = "avaloniaui (oss)", estado = "ativo",
        },
        {
            nome = "compose_multiplatform",
            github = "https://github.com/JetBrains/compose-multiplatform",
            docs = "https://www.jetbrains.com/lp/compose-multiplatform/",
            stars = 17500, forks = 1200, ano = 2021,
            linguagens = [[kotlin]],
            render = [[proprio-gpu (skia/skiko) em ios/desktop/web; em android usa
                jetpack compose nativo]],
            features = [[declarativo, recomposicao reativa, interop swiftui/uikit,
                hot reload (compose hot reload 1.0 jan/2026), navegacao, kmp]],
            devices = [[android, ios (estavel mai/2025), desktop (jvm), web (wasm,
                beta)]],
            performance = [[startup comparavel a nativo; scroll on par com swiftui
                em devices de alta taxa de atualizacao (jetbrains 1.8.0)]],
            pros = [[compartilha ui+logica, kotlin, interop nativo ios, adotado por
                netflix/mcdonalds/duolingo (via kmp)]],
            contras = [[jvm/skiko pesado, web em beta, nao para embarcado]],
            licenca = "apache-2.0", empresa = "jetbrains + google",
            estado = "ativo",
        },
        {
            nome = "dear_imgui",
            github = "https://github.com/ocornut/imgui",
            docs = "https://github.com/ocornut/imgui/wiki",
            stars = 65000, forks = 11000, ano = 2014,
            linguagens = [[c++ (bindings c/cimgui, muitas linguagens)]],
            render = [[imediato (immediate mode); gera vertex buffers, render-agnostico
                (opengl/dx/vulkan/metal/webgpu); reconstroi ui a cada frame]],
            features = [[bloat-free, sem dependencias, ideal para tools/debug em game
                engines, sem acessibilidade/i18n, docking/tabs/multi-viewport]],
            devices = [[desktop, embarcado (qualquer lugar que renderize triangulos),
                consoles; nao foca end-user apps]],
            performance = [[redesenha tudo por frame mas em batches eficientes;
                footprint minimo; bom o suficiente para gamedev]],
            pros = [[trivial de integrar, rapido para tooling, battle-tested na
                industria de jogos]],
            contras = [[sem acessibilidade, sem look nativo, immediate mode tem
                paradoxo de layout, nao para apps de usuario final]],
            licenca = "mit", empresa = "omar cornut (oss)", estado = "ativo",
        },
        {
            nome = "dioxus",
            github = "https://github.com/DioxusLabs/dioxus",
            docs = "https://dioxuslabs.com/learn/",
            stars = 36267, forks = 1690, ano = 2021,
            linguagens = [[rust (rsx, html-like)]],
            render = [[multiplo: web-sys (dom), webview (desktop/mobile), ssr,
                liveview, renderer wgpu experimental; vdom coarse-grained]],
            features = [[inspirado em react, rsx, hot-patching subsecond, tailwind,
                fullstack com axum, binarios <5mb desktop, web <50kb]],
            devices = [[web (wasm), desktop (win/mac/linux), mobile (ios/android via
                jni/webview)]],
            performance = [[wasm proximo de js vanilla em benchmarks; binarios
                portateis <3mb desktop]],
            pros = [[react-like em rust, multiplataforma, dx excelente, fullstack]],
            contras = [[mobile via webview (nao nativo puro), ecossistema jovem]],
            licenca = "mit / apache-2.0", empresa = "dioxus labs", estado = "ativo",
        },
        {
            nome = "egui",
            github = "https://github.com/emilk/egui",
            docs = "https://docs.rs/egui/",
            stars = 29300, forks = 2100, ano = 2020,
            linguagens = [[rust]],
            render = [[imediato (immediate mode); render-agnostico via integracao
                (eframe/egui_glow/egui-wgpu); repaint so em interacao/animacao]],
            features = [[immediate mode puro, facil integrar em game engines, roda
                em web+native, epaint (api de pintura 2d), patrocinado por rerun]],
            devices = [[desktop (win/mac/linux), web (wasm/webgl), embedded (com
                integracao custom)]],
            performance = [[tipicamente 1-2 ms por frame; 60fps suave em resize sem
                custo extra de cpu; repaint sob demanda economiza cpu]],
            pros = [[simples, rapido, otimo para tools/games, web+native]],
            contras = [[nao parece nativo, immediate mode (paradoxo de layout),
                interfaces ainda em fluxo]],
            licenca = "mit / apache-2.0", empresa = "emil ernerfeldt (oss)",
            estado = "ativo",
        },
        {
            nome = "electron",
            github = "https://github.com/electron/electron",
            docs = "https://www.electronjs.org/docs/latest",
            stars = 116000, forks = 16000, ano = 2013,
            linguagens = [[javascript/typescript, html, css; nucleo c++]],
            render = [[webview/dom (chromium bundlado + node.js); cada app embute
                browser inteiro]],
            features = [[chromium completo, acesso total ao node/os, ecossistema npm,
                cross-platform desktop, usado por vscode/slack/discord]],
            devices = [[desktop (win/mac/linux)]],
            performance = [[pesado: embute chromium (centenas de mb, alto consumo de
                ram); tauri usa 1/10-1/100 do tamanho]],
            pros = [[maturidade, ecossistema, render consistente, dx web]],
            contras = [[binarios enormes, ram alta, sem mobile, sem embarcado]],
            licenca = "mit", empresa = "openjs foundation", estado = "ativo",
        },
        {
            nome = "emwin",
            github = "n/a (proprietario, segger)",
            docs = "https://www.segger.com/products/user-interface/emwin/",
            stars = 0, forks = 0, ano = 1996,
            linguagens = [[c, c++]],
            render = [[proprio (software/2d), independente de controlador de display
                e cpu; framebuffer]],
            features = [[appwizard (gui builder), 2d, fontes, bitmaps, widgets,
                window manager, animacoes; roda em rtos (embos) ou bare metal]],
            devices = [[embarcado/mcus (muitos), rtos e bare metal]],
            performance = [[footprint pequeno, otimizado para mcu; sem gpu necessaria]],
            pros = [[comercialmente suportado, appwizard, maduro]],
            contras = [[caro (us$3k-15k por dev), proprietario, nao cross-platform]],
            licenca = "comercial", empresa = "segger microcontroller",
            estado = "ativo",
        },
        {
            nome = "flutter",
            github = "https://github.com/flutter/flutter",
            docs = "https://docs.flutter.dev/",
            stars = 177000, forks = 30500, ano = 2017,
            linguagens = [[dart]],
            render = [[proprio-gpu; desde 3.29 impeller e o unico renderer no ios
                ("skia support has been removed from the ios backend e o flag
                FLTEnableImpeller opt-out nao funciona mais"); android api29+ usa
                impeller (vulkan, fallback opengles); web usa skia (canvaskit/skwasm).
                impeller pre-compila shaders em build time (overhead ~100 kb por
                arquitetura)]],
            features = [[widgets material+cupertino, hot reload, aot, stateful hot
                reload no web (2025), genui sdk (ai), ai toolkit, mcp server]],
            devices = [[mobile (ios/android), desktop (win/mac/linux), web (wasm/js),
                embedded linux (flutter-pi); nao roda em mcu/bare metal]],
            performance = [[60/120fps estavel via impeller; impeller elimina jank de
                compilacao de shader; impeller pode consumir mais gpu/energia que
                skia em alguns casos (issue 164607: +15% corrente em vetores)]],
            pros = [[ui pixel-perfect consistente, 1M+ devs, motor proprio, mais
                usado em cross-platform por 4 anos seguidos (jetbrains)]],
            contras = [[dart nicho, nao roda em mcu, binarios maiores, equipe pequena
                vs base de usuarios (fork flock em 2024)]],
            licenca = "bsd-3", empresa = "google", estado = "ativo",
        },
        {
            nome = "gpui",
            github = "https://github.com/zed-industries/zed",
            docs = "https://www.gpui.rs/",
            stars = 84941, forks = 9012, ano = 2021,
            linguagens = [[rust]],
            render = [[proprio-gpu (acelerado por gpu, metal/etc); hybrid immediate/
                retained; criado para o editor zed]],
            features = [[gpu-first, alto desempenho para apps como zed, api em rust;
                stars referem-se ao repo zed (gpui vive dentro dele)]],
            devices = [[desktop (mac maduro, linux/windows em progresso)]],
            performance = [[projetado para latencia minima e 120fps no editor zed]],
            pros = [[performance extrema, gpu-first, usado em producao (zed)]],
            contras = [[acoplado ao zed, docs/ecossistema imaturos, sem mobile/web/
                embarcado]],
            licenca = "apache-2.0 / gpl (partes)", empresa = "zed industries",
            estado = "ativo",
        },
        {
            nome = "gtk",
            github = "https://gitlab.gnome.org/GNOME/gtk",
            docs = "https://docs.gtk.org/",
            stars = 1800, forks = 400, ano = 1998,
            linguagens = [[c (bindings rust/python/vala/c++)]],
            render = [[proprio (gsk/cairo/vulkan no gtk4); desenha widgets proprios]],
            features = [[toolkit completo, widgets, css-like styling, base do gnome,
                bindings multilinguagem]],
            devices = [[desktop (linux primario, win/mac portados)]],
            performance = [[maduro, razoavel; gtk4 com gpu via vulkan]],
            pros = [[maduro, base do gnome, foss, bindings]],
            contras = [[foco linux (win/mac segundo plano), look nao-nativo fora do
                gnome]],
            licenca = "lgpl-2.1", empresa = "gnome foundation", estado = "ativo",
        },
        {
            nome = "iced",
            github = "https://github.com/iced-rs/iced",
            docs = "https://docs.rs/iced/",
            stars = 30600, forks = 1600, ano = 2019,
            linguagens = [[rust]],
            render = [[proprio-gpu (wgpu; fallback tiny-skia software); retained,
                inspirado em elm (model-view-update)]],
            features = [[arquitetura elm, reativo, type-safe, cross-platform,
                modular (pluga em qualquer lugar)]],
            devices = [[desktop (win/mac/linux), web (wasm experimental), embedded
                possivel via wgpu/software]],
            performance = [[retained mode eficiente, wgpu acelerado]],
            pros = [[arquitetura elm limpa, type-safe, render proprio consistente]],
            contras = [[ecossistema jovem, web imaturo, menos widgets que toolkits
                maduros]],
            licenca = "mit", empresa = "hector ramos / oss", estado = "ativo",
        },
        {
            nome = "jetpack_compose",
            github = "https://github.com/androidx/androidx (compose)",
            docs = "https://developer.android.com/jetpack/compose",
            stars = 0, forks = 0, ano = 2021,
            linguagens = [[kotlin]],
            render = [[proprio em android (desenha via skia no canvas android);
                declarativo, recomposicao reativa]],
            features = [[@composable, recomposicao por estado, interop com xml views,
                material 3, preview ao vivo, base do compose multiplatform]],
            devices = [[android (nativo); wear os; e estende para multiplatform]],
            performance = [[otimizado para android; recomposicao inteligente]],
            pros = [[oficial android, declarativo, kotlin, material 3 nativo]],
            contras = [[so android (sem multiplatform = jetbrains), biblioteca (nao
                bundlada no os)]],
            licenca = "apache-2.0", empresa = "google", estado = "ativo",
        },
        {
            nome = "kivy",
            github = "https://github.com/kivy/kivy",
            docs = "https://kivy.org/doc/stable/",
            stars = 17000, forks = 3000, ano = 2011,
            linguagens = [[python (kv language)]],
            render = [[proprio (opengl es 2); desenha widgets proprios]],
            features = [[multi-touch, kv lang declarativa, kivymd (material), apps
                inovadoras/touch]],
            devices = [[linux, win, mac, android, ios, raspberry pi]],
            performance = [[opengl acelerado; python tem overhead, distribuicao
                trabalhosa]],
            pros = [[python, multi-touch, multiplataforma, mit]],
            contras = [[look nao-nativo, empacotamento dificil, gc python]],
            licenca = "mit", empresa = "kivy organization (oss)", estado = "ativo",
        },
        {
            nome = "lvgl",
            github = "https://github.com/lvgl/lvgl",
            docs = "https://docs.lvgl.io/",
            stars = 23700, forks = 4200, ano = 2016,
            linguagens = [[c (compativel c++); bindings micropython/rust]],
            render = [[proprio (software, partial buffering); flush callback envia
                areas alteradas ao display; gpu opcional (vg-lite, pxp, dave2d,
                dma2d)]],
            features = [[30+ widgets, ~100 props de estilo, flexbox/grid (css-like),
                utf-8/cjk/rtl, anti-aliasing, animacoes, lvgl pro (xml editor),
                simulador pc, integra zephyr/nuttx/rt-thread]],
            devices = [[qualquer mcu/mpu com display (spi/parallel/rgb), monochrome/
                epaper/oled/tft, rtos ou bare metal; usado pela xiaomi em muitos
                devices]],
            performance = [[per docs oficiais: flash > 64kb essencial (>180kb
                recomendado), ram estatica ~2kb, heap >2kb (>48kb recomendado com
                muitos widgets); partial buffering minimiza ram]],
            pros = [[mit (comercial-friendly), footprint minimo, sem dependencias,
                portavel a qualquer mcu, simulador pc->deploy identico]],
            contras = [[so embarcado/2d, sem ecossistema desktop/mobile/web amplo,
                c puro (sem memory safety)]],
            licenca = "mit", empresa = "lvgl llc", estado = "ativo",
        },
        {
            nome = "lynx",
            github = "https://github.com/lynx-family/lynx",
            docs = "https://lynxjs.org/",
            stars = 12000, forks = 700, ano = 2025,
            linguagens = [[typescript/javascript (reactlynx), css; tooling rust;
                motor c++/primjs]],
            render = [[proprio (custom renderer que adapta a primitivas nativas:
                view->uiview/viewgroup); dual-thread (ui thread + logic thread)]],
            features = [[arquitetura dual-thread, instant first-frame rendering (ifr),
                main-thread scripting (mts), css real (animacoes/gradientes/vars),
                framework-agnostic (react/vue/svelte), rspeedy (rspack/rust)]],
            devices = [[android, ios, web; usado no tiktok (search, studio, shop)]],
            performance = [[lancamento ~2.5x mais rapido que react native (segundo
                bytedance); ifr elimina telas em branco; 60fps em ui thread]],
            pros = [[battle-tested no tiktok, performance, css real, multi-framework,
                apache-2.0]],
            contras = [[ecossistema nascente, docs finas, sem libs de terceiros,
                setup so estavel em macos]],
            licenca = "apache-2.0", empresa = "bytedance / tiktok", estado = "ativo",
        },
        {
            nome = "makepad",
            github = "https://github.com/makepad/makepad",
            docs = "https://makepad.dev/",
            stars = 6200, forks = 314, ano = 2019,
            linguagens = [[rust (live dsl)]],
            render = [[proprio-gpu (shaders); compila para wasm/webgl, mac/metal,
                win/dx11, linux/opengl]],
            features = [[widgets de alta performance, live coding/dsl, abstracoes de
                plataforma low-overhead, parte do projeto robius]],
            devices = [[desktop (mac/win/linux), web (wasm/webgl), mobile (em
                progresso)]],
            performance = [[gpu-first, shaders custom, foco em performance]],
            pros = [[performance gpu, live coding, rust]],
            contras = [[ecossistema muito jovem, comunidade pequena, docs limitadas]],
            licenca = "mit / apache-2.0", empresa = "makepad (oss)", estado = "ativo",
        },
        {
            nome = "maui",
            github = "https://github.com/dotnet/maui",
            docs = "https://learn.microsoft.com/dotnet/maui/",
            stars = 22000, forks = 1800, ano = 2022,
            linguagens = [[c#/.net, xaml]],
            render = [[nativo (handlers fazem ponte para controles nativos do os);
                sucessor do xamarin.forms]],
            features = [[handlers, xaml, mvvm, hot reload, blazor hybrid; render via
                native controls (catalyst no mac)]],
            devices = [[android, ios, mac (catalyst), windows (winui); linux so
                comunidade (via avalonia)]],
            performance = [[criticado: overhead de wrappers nativos; mac via catalyst
                (app ios no desktop), startup android lento]],
            pros = [[oficial microsoft, c#/.net, reaproveita skills xamarin]],
            contras = [[performance inferior a avalonia, sem linux oficial, historico
                de churn da microsoft]],
            licenca = "mit", empresa = "microsoft", estado = "ativo",
        },
        {
            nome = "nuklear",
            github = "https://github.com/Immediate-Mode-UI/Nuklear",
            docs = "https://immediate-mode-ui.github.io/Nuklear/doc/index.html",
            stars = 9500, forks = 600, ano = 2015,
            linguagens = [[c (ansi c, single-header)]],
            render = [[imediato (immediate mode); render-agnostico (gera comandos)]],
            features = [[single-header, sem dependencias, sem alocacao, ideal para
                gamedev/embedded debug ui]],
            devices = [[desktop, embarcado, qualquer backend de render]],
            performance = [[minimo, sem alocacao de heap obrigatoria]],
            pros = [[zero dependencias, single-header, portavel]],
            contras = [[sem acessibilidade, look custom, immediate mode]],
            licenca = "mit / public domain", empresa = "oss", estado = "ativo",
        },
        {
            nome = "qt_qml",
            github = "https://github.com/qt/qtbase",
            docs = "https://doc.qt.io/",
            stars = 2000, forks = 800, ano = 1995,
            linguagens = [[c++, qml, javascript]],
            render = [[proprio (qt quick scene graph, opengl/vulkan/metal/d3d);
                qwidgets usa estilo nativo]],
            features = [[framework completo, qml declarativo, qt quick, qt creator,
                qt design studio, automotive suite, base de hmi embarcado linux]],
            devices = [[desktop (win/mac/linux), mobile (ios/android), embarcado
                linux, automotive, tv; (mcus via produto separado)]],
            performance = [[maduro e otimizado; pesado para embarcado pequeno
                (requer yocto/openembedded)]],
            pros = [[completo, maduro, bem documentado, cross-platform total]],
            contras = [[custo (licenca comercial), pesado, curva de aprendizado,
                muitas classes]],
            licenca = "lgpl-3 / comercial", empresa = "the qt company", estado = "ativo",
        },
        {
            nome = "qt_for_mcus",
            github = "n/a (proprietario)",
            docs = "https://www.qt.io/product/develop-software-microcontrollers-mcu",
            stars = 0, forks = 0, ano = 2020,
            linguagens = [[c++, qml (qt quick ultralite)]],
            render = [[proprio (qt quick ultralite engine, otimizado para mcu);
                gpu/framebuffer/linebuffer conforme hardware]],
            features = [[qml em mcu, ui smartphone-like em low-end, keyframe/property
                animation, pixel formats 8-32bit, png/rle, fontes runtime/compile]],
            devices = [[mcus (nxp, renesas, infineon, stm), rtos ou bare metal]],
            performance = [[engine otimizado para clock de mhz e ram pequena; muito
                mais leve que qt normal]],
            pros = [[qml em mcu, escala de mcu a mpu, suporte de vendors]],
            contras = [[proprietario/comercial, subset do qml, custo]],
            licenca = "comercial", empresa = "the qt company", estado = "ativo",
        },
        {
            nome = "react",
            github = "https://github.com/facebook/react",
            docs = "https://react.dev/",
            stars = 240000, forks = 49000, ano = 2013,
            linguagens = [[javascript/typescript (jsx)]],
            render = [[dom (virtual dom + reconciliacao); web (react-dom)]],
            features = [[vdom, hooks, server components, concurrent, maior ecossistema
                ui do mundo, base de rn/next.js]],
            devices = [[web; (mobile via react native, desktop via electron/tauri)]],
            performance = [[vdom tem overhead vs signals (solid/svelte) mas "rapido o
                suficiente"; 44.7% de uso entre devs (so2025)]],
            pros = [[ecossistema gigante, pool de devs, maduro, battle-tested,
                80% das fortune 500]],
            contras = [[bundle/runtime maior que svelte/solid, vdom overhead,
                muitas decisoes arquiteturais]],
            licenca = "mit", empresa = "meta", estado = "ativo",
        },
        {
            nome = "react_native",
            github = "https://github.com/facebook/react-native",
            docs = "https://reactnative.dev/",
            stars = 125000, forks = 24500, ano = 2015,
            linguagens = [[javascript/typescript (jsx)]],
            render = [[nativo (ponte/jsi para widgets nativos; nova arquitetura com
                fabric/turbomodules)]],
            features = [[componentes nativos, fast refresh, metro bundler, nova
                arquitetura (fabric), expo, ecossistema npm]],
            devices = [[ios, android; (web via rn-web, desktop via portais)]],
            performance = [[near-native com nova arquitetura; depende de ponte js
                em casos]],
            pros = [[reaproveita skills react/js, npm, contratacao facil, maduro]],
            contras = [[ponte historica (mitigada), look depende de nativo, menos
                consistente que flutter]],
            licenca = "mit", empresa = "meta", estado = "ativo",
        },
        {
            nome = "slint",
            github = "https://github.com/slint-ui/slint",
            docs = "https://docs.slint.dev/",
            stars = 22900, forks = 900, ano = 2020,
            linguagens = [[dsl .slint; logica em rust, c++, javascript, python]],
            render = [[proprio; backends configuraveis em compile time: femtovg
                (opengl es 2), skia, e software renderer (cpu, sem dependencias).
                SoftwareRenderer.render_by_line() envia ui linha a linha via spi
                (ram minima = uma linha de pixels); dma2d/framebuffer/gpu conforme
                hardware]],
            features = [[dsl declarativo, live preview, figma plugin, api 1.x estavel,
                material 3, componentes em regiao de memoria unica, lsp/vscode,
                testes de ui]],
            devices = [[embarcado/mcu (bare metal: rp2040/pico foi o primeiro port,
                esp32, stm32h7), embedded linux/qnx, desktop (win/mac/linux),
                mobile (android/win/mac/linux), web (wasm)]],
            performance = [[runtime cabe em menos de 300KiB ram (site oficial slint.dev);
                compila para codigo de maquina; render line-by-line para devices com
                pouca ram]],
            pros = [[escala mcu->desktop, render proprio, 300KiB runtime, dsl limpo,
                multi-linguagem, api estavel]],
            contras = [[licenca dupla complexa (gpl/royalty-free/comercial), ecossistema
                jovem, comunidade menor que qt]],
            licenca = "gpl-3 / royalty-free / comercial", empresa = "slint (sixtyfps)",
            estado = "ativo",
        },
        {
            nome = "solidjs",
            github = "https://github.com/solidjs/solid",
            docs = "https://www.solidjs.com/",
            stars = 34631, forks = 1000, ano = 2021,
            linguagens = [[javascript/typescript (jsx)]],
            render = [[dom (reatividade fine-grained com signals, sem virtual dom);
                atualizacoes cirurgicas]],
            features = [[signals, fine-grained reactivity, jsx react-like, sem vdom,
                solidstart (fullstack)]],
            devices = [[web; (mobile via solid-native experimental)]],
            performance = [[entre os mais rapidos (proximo de js vanilla); #1 em
                satisfacao por anos]],
            pros = [[performance top, react-like, bundles pequenos, signals]],
            contras = [[ecossistema menor que react, comunidade em crescimento]],
            licenca = "mit", empresa = "ryan carniato / oss", estado = "ativo",
        },
        {
            nome = "svelte",
            github = "https://github.com/sveltejs/svelte",
            docs = "https://svelte.dev/docs",
            stars = 84956, forks = 4000, ano = 2016,
            linguagens = [[javascript/typescript (.svelte)]],
            render = [[dom (compilador: gera js imperativo direto, sem vdom em
                runtime; svelte 5 usa runes/signals)]],
            features = [[compiler-first, sem vdom, bundles pequenos, sveltekit
                (fullstack), runes (svelte 5)]],
            devices = [[web; (mobile/desktop via capacitor/tauri)]],
            performance = [[bundles menores e runtime rapido (compile-time);
                62.4% de admiracao (so2025)]],
            pros = [[bundles pequenos, sintaxe simples, performance, alta satisfacao]],
            contras = [[ecossistema menor que react, menos jobs]],
            licenca = "mit", empresa = "svelte / vercel", estado = "ativo",
        },
        {
            nome = "swiftui",
            github = "n/a (closed-source, apple)",
            docs = "https://developer.apple.com/documentation/swiftui",
            stars = 0, forks = 0, ano = 2019,
            linguagens = [[swift]],
            render = [[proprio/declarativo sobre uikit/appkit; bundlado no os;
                recompoe por estado]],
            features = [[declarativo, @state/bindings, previews ao vivo (xcode),
                integra combine, adapta automatico a apple tv/watch, interop uikit]],
            devices = [[ios, ipados, macos, watchos, tvos, visionos]],
            performance = [[arc (sem gc), integracao nativa profunda; bundlado no os]],
            pros = [[declarativo, integracao apple total, adapta a todos os devices
                apple, sem gc]],
            contras = [[closed-source, so ecossistema apple, docs/comunidade menor
                que uikit, bundlado no os (sem update independente)]],
            licenca = "proprietaria", empresa = "apple", estado = "ativo",
        },
        {
            nome = "tauri",
            github = "https://github.com/tauri-apps/tauri",
            docs = "https://v2.tauri.app/",
            stars = 104000, forks = 3200, ano = 2019,
            linguagens = [[frontend: qualquer (js/ts/html/css); backend: rust;
                integra swift/kotlin]],
            render = [[webview do os (wry: webkit no mac, webview2 no win, webkitgtk
                no linux); nao embute browser]],
            features = [[webview nativo, backend rust, qualquer framework front,
                binarios minimos, mobile (ios/android desde v2), tao+wry]],
            devices = [[desktop (win/mac/linux), mobile (ios/android) — 6 plataformas]],
            performance = [["a minimal tauri app can be less than 600kb in size"
                (docs oficiais); 1/5-1/8 da ram do electron]],
            pros = [[binarios minusculos, rust seguro no backend, qualquer front,
                muito mais leve que electron]],
            contras = [[render depende do webview do os (inconsistencia entre
                plataformas), sem embarcado]],
            licenca = "mit / apache-2.0", empresa = "tauri (oss)", estado = "ativo",
        },
        {
            nome = "touchgfx",
            github = "n/a (proprietario, st)",
            docs = "https://www.st.com/en/embedded-software/x-cube-touchgfx.html",
            stars = 0, forks = 0, ano = 2018,
            linguagens = [[c, c++]],
            render = [[proprio (gpu chrom-art/dma2d quando disponivel, ou software);
                otimizado para stm32]],
            features = [[touchgfx designer (gui builder), generator (cubemx plugin),
                engine c++, widgets, animacoes; roda em rtos ou bare metal]],
            devices = [[microcontroladores stm32 (somente)]],
            performance = [[carga minima de cpu e memoria, otimizado para stm32;
                gratuito]],
            pros = [[gratuito, suportado pela st, designer wysiwyg, performance em
                stm32]],
            contras = [[so stm32, proprietario, nao cross-platform]],
            licenca = "proprietaria (gratis)", empresa = "stmicroelectronics",
            estado = "ativo",
        },
        {
            nome = "uikit",
            github = "n/a (closed-source, apple)",
            docs = "https://developer.apple.com/documentation/uikit",
            stars = 0, forks = 0, ano = 2008,
            linguagens = [[swift, objective-c]],
            render = [[nativo/retido (uiview hierarchy); bundlado no os]],
            features = [[imperativo, interface builder/storyboards ou programatico,
                maduro, base historica do ios, interop com swiftui]],
            devices = [[ios, ipados, tvos (uikit), catalyst no mac]],
            performance = [[nativo, maduro, otimizado]],
            pros = [[maduro, controle total, enorme base de codigo/comunidade]],
            contras = [[imperativo (verboso), closed-source, so apple, apple migra
                para swiftui]],
            licenca = "proprietaria", empresa = "apple", estado = "manutencao/ativo",
        },
        {
            nome = "uno_platform",
            github = "https://github.com/unoplatform/uno",
            docs = "https://platform.uno/docs/",
            stars = 9000, forks = 800, ano = 2018,
            linguagens = [[c#/.net, xaml (winui)]],
            render = [[duplo: controles nativos do os OU skia (escolha do dev);
                pixel-perfect opcional]],
            features = [[winui em todas plataformas, render nativo ou skia, figma
                plugin, 2 mcps para agentes ai, integra windows community toolkit]],
            devices = [[web (wasm), desktop (win/mac/linux), mobile (ios/android)]],
            performance = [[skia para fidelidade ou nativo para look-and-feel]],
            pros = [[winui everywhere, escolha de render, linux+web, c#/.net]],
            contras = [[complexidade, ecossistema menor que maui/flutter]],
            licenca = "apache-2.0", empresa = "uno platform", estado = "ativo",
        },
        {
            nome = "vue",
            github = "https://github.com/vuejs/core",
            docs = "https://vuejs.org/",
            stars = 52244, forks = 8500, ano = 2014,
            linguagens = [[javascript/typescript (sfc .vue)]],
            render = [[dom (virtual dom + reatividade; vue 3 com proxy reactivity)]],
            features = [[sfc, composition api, reatividade, nuxt (fullstack),
                docs excelentes, curva de aprendizado suave]],
            devices = [[web; (mobile via capacitor/nativescript-vue)]],
            performance = [[performance solida, bundles medios, melhor baseline que
                react em cenarios client-heavy]],
            pros = [[facil de aprender, docs otimas, equilibrio dx/performance,
                ecossistema maduro]],
            contras = [[overhead de vdom vs solid/svelte, menos jobs que react]],
            licenca = "mit", empresa = "evan you / oss", estado = "ativo",
        },
        {
            nome = "wxwidgets",
            github = "https://github.com/wxWidgets/wxWidgets",
            docs = "https://docs.wxwidgets.org/",
            stars = 6500, forks = 1800, ano = 1992,
            linguagens = [[c++ (bindings python/ruby/lua/perl)]],
            render = [[nativo (usa api nativa de cada os, nao emula); look nativo
                real]],
            features = [[controles nativos, cross-platform, bindings multilinguagem,
                maduro, pode delegar ao qt (wxqt)]],
            devices = [[desktop (win/mac/linux); mobile so via wxqt experimental]],
            performance = [[nativo, leve]],
            pros = [[look nativo real, foss, maduro, licenca permissiva]],
            contras = [[sem mobile real, equipe pequena, suporte dark mode limitado
                no win, gtk4 ainda transicionando]],
            licenca = "wxwindows (lgpl-like)", empresa = "wxwidgets (oss)",
            estado = "ativo",
        },
        {
            nome = "yew",
            github = "https://github.com/yewstack/yew",
            docs = "https://yew.rs/",
            stars = 32700, forks = 1500, ano = 2017,
            linguagens = [[rust]],
            render = [[dom via wasm (virtual dom, inspirado em react/elm)]],
            features = [[vdom, componentes, html! macro, wasm, hooks-like]],
            devices = [[web (wasm)]],
            performance = [[wasm; mais lento que leptos/dioxus em alguns benchmarks
                (vdom)]],
            pros = [[rust no frontend, react-like, maduro entre os rust web]],
            contras = [[vdom overhead, wasm bundle, ecossistema menor]],
            licenca = "mit / apache-2.0", empresa = "yewstack (oss)", estado = "ativo",
        },
    },
    tabela_performance = {
        descricao = [[
            tabela comparativa de performance por cenario de device. valores tipicos/
            documentados; benchmarks variam por hardware. footprint = tamanho aprox
            do binario/runtime; ram = uso tipico; render = aceleracao.
        ]],
        desktop = {
            { framework = "flutter", ram = "moderado-alto", render = "gpu (impeller/skia)", footprint = "medio-grande", startup = "rapido (aot)" },
            { framework = "electron", ram = "alto (chromium)", render = "gpu (chromium)", footprint = "grande (100s mb)", startup = "lento" },
            { framework = "tauri", ram = "baixo (1/5-1/8 do electron)", render = "webview os", footprint = "<600kb", startup = "rapido" },
            { framework = "avalonia", ram = "baixo-moderado", render = "gpu (skia)", footprint = "medio", startup = "rapido (aot)" },
            { framework = "egui", ram = "baixo", render = "gpu (wgpu/glow)", footprint = "pequeno", startup = "rapido" },
            { framework = "qt_qml", ram = "moderado", render = "gpu (scene graph)", footprint = "medio-grande", startup = "moderado" },
        },
        mobile = {
            { framework = "flutter", ram = "moderado", render = "gpu (impeller metal/vulkan)", footprint = "medio", startup = "rapido (aot)" },
            { framework = "react_native", ram = "moderado", render = "nativo (fabric)", footprint = "medio", startup = "moderado" },
            { framework = "lynx", ram = "1/4 do rn (claim bytedance)", render = "proprio dual-thread", footprint = "medio", startup = "muito rapido (ifr)" },
            { framework = "compose_multiplatform", ram = "moderado (jvm/skiko)", render = "gpu (skia)", footprint = "medio", startup = "comparavel a nativo" },
            { framework = "swiftui", ram = "baixo (arc, bundlado no os)", render = "proprio", footprint = "minimo (no os)", startup = "rapido" },
        },
        web = {
            { framework = "react", ram = "moderado", render = "dom (vdom)", footprint = "medio-grande", startup = "moderado" },
            { framework = "svelte", ram = "baixo", render = "dom (compilado)", footprint = "pequeno", startup = "rapido" },
            { framework = "solidjs", ram = "baixo", render = "dom (signals)", footprint = "pequeno", startup = "muito rapido" },
            { framework = "flutter_web", ram = "alto", render = "skia (canvaskit/skwasm)", footprint = "grande (wasm+skia)", startup = "lento" },
            { framework = "dioxus", ram = "baixo", render = "dom (wasm)", footprint = "<50kb", startup = "rapido" },
        },
        embarcado_mcu = {
            { framework = "lvgl", ram = "estatica ~2kb, heap >2kb (>48kb rec)", render = "software/partial buffer", footprint = "flash >64kb (>180kb rec)", startup = "instantaneo" },
            { framework = "slint", ram = "<300kib runtime", render = "software line-by-line / gpu", footprint = "pequeno", startup = "rapido (codigo de maquina)" },
            { framework = "qt_for_mcus", ram = "pequeno (ultralite)", render = "gpu/framebuffer/linebuffer", footprint = "pequeno", startup = "rapido" },
            { framework = "touchgfx", ram = "minimo (stm32)", render = "gpu chrom-art/dma2d ou sw", footprint = "pequeno", startup = "rapido" },
            { framework = "emwin", ram = "pequeno", render = "software", footprint = "pequeno", startup = "rapido" },
            { framework = "dear_imgui", ram = "baixo", render = "imediato (vertex buffers)", footprint = "pequeno", startup = "instantaneo" },
        },
    },
}