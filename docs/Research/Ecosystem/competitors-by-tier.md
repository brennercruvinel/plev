---
id: research-competitors-by-tier
title: "competitors by tier"
project: plev
type: research
status: reference
date: 2022-10-21
date_basis: reconstructed
recorded: 2026-03-13
topics: [competition, research, gpu, platform]
relations: [research-makepad-gap-analysis, research-technology-radar]
source: docs/ref/competitors-by-tier.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# competitors by tier

Groups plev's competitors into five tiers, from direct competitors (makepad, ribir) to references from other industries (flutter, vello), and states in each one where it differs from plev. The positioning map summarizes the niche: a GPU-first composition engine, with native accessibility, hash-based dirty tracking and unified WebGPU shaders, and the page concludes that no other Rust project combines these properties at once. Why it matters: it fixes the comparison criterion against other Rust UI frameworks.

tier 1 - concorrentes diretos (GPU-first, rust, cross-platform)

1. makepad
   github.com/makepad/makepad

justificativa: o projeto mais próximo do plev. GPU-first, rendering próprio (metal,
dx11, opengl, webgl), cross-platform incluindo ios/android/WASM. possui DSL
live-editable e sistema de layout próprio (turtle). vocês já têm gap analysis
detalhado no kdb. diferenças fundamentais: makepad não tem acessibilidade, usa
texto customizado com problemas de unicode, e o codebase é ~1m LOC vs ~30k do plev.
plev usa wgpu (webgpu unificado) enquanto makepad mantém backends separados por
plataforma.

2. ribir
   github.com/RibirX/Ribir

ribir usa wgpu como backend, rust nativo, reativo. porém
está em estado pré-alpha (60+ releases alpha, API instável, documentação
limitada). vosso kdb já o cataloga como concorrente direto mas com maturidade
inferior.

---

tier 2 - frameworks rust UI com rendering próprio

3. iced
   github.com/iced-rs/iced

justificativa: arquitetura elm (como plev), renderiza via wgpu, cross-platform
(desktop + WASM). diferença chave: iced é retained-mode com widget tree
tradicional, não tem compositing por layers, não tem dirty tracking por hash como
plev. sem suporte mobile nativo. texto via cosmic-text (mesmo que plev).

4. xilem
   github.com/linebender/xilem

justificativa: projeto linebender (mesma equipe do druid), usa vello (GPU 2d
renderer baseado em compute shaders) + wgpu. arquitetura reativa inspirada em
swiftui. diferença: vello usa path rendering via compute shaders (abordagem
diferente dos SDF shaders do plev). ecossistema linebender inclui parley (text
layout), que o technology radar do plev identifica como possível substituto futuro
do cosmic-text.

5. floem
   github.com/lapce/floem

justificativa: do time do lapce editor. sistema reativo com signals (similar ao
plev), rendering GPU. usa peniko + vello para renderização. diferença: floem é
focado em desktop, sem target mobile. possui reactive system inspirado em leptos
(mesmo paradigma que influenciou o signal.rs do plev).

6. gpui (dentro do zed)
   github.com/zed-industries/zed

justificativa: framework GPU-UI customizado (gpui2) construído para o editor zed.
metal no macos, vulkan no linux/windows. rendering completamente próprio, sem DOM.
diferença: gpui é fortemente acoplado ao zed, não é um framework genérico. sem
WASM, sem mobile.

---

tier 3 - frameworks rust UI com abordagem diferente

7. slint
   github.com/slint-ui/slint

justificativa: DSL declarativa com 32+ passes de otimização, 3 backends de
rendering (femtovg, skia, software). cross-platform incluindo embedded. diferença
fundamental: não é GPU-first (o backend software é o principal), DSL compilada vs
builder pattern do plev. maturidade comercial superior (empresa por trás).

8. dioxus
   github.com/dioxuslabs/dioxus

justificativa: react-like em rust, cross-platform (desktop, mobile, web).
recentemente adicionou blitz (renderer nativo wgpu). diferença: plev seria uma
camada de rendering abaixo do dioxus, não um concorrente direto. dioxus opera no
nível de framework de aplicação, plev no nível de motor de composição.

9. egui
   github.com/emilk/egui

justificativa: immediate-mode gui, backend wgpu, WASM nativo. amplamente adotado.
diferença fundamental: immediate-mode (redesenha tudo todo frame) vs retained-mode
com dirty tracking do plev. sem acessibilidade nativa, sem layout flexbox, sem
compositing por layers. foco em ferramentas/debug, não em aplicações de produção.

10. vizia
    github.com/vizia/vizia

justificativa: declarativo, reativo, rendering customizado com GPU. suporte a
theming e acessibilidade. diferença: usa femtovg (opengl) como backend principal,
não wgpu. menos ambicioso em termos de plataformas.

---

tier 4 - influências arquiteturais (não concorrentes diretos)

11. leptos
    github.com/leptos-rs/leptos

justificativa: não é um framework de UI nativo, é web-only. porém o sistema reativo
de fine-grained signals (readsignal/writesignal, push-pull hybrid) é a referência
arquitetural direta do signal.rs do plev. vosso kdb confirma isso.

12. bevy (UI module)
    github.com/bevyengine/bevy

justificativa: game engine com módulo de UI baseado em ECS + wgpu. rendering
GPU-first. diferença: ECS architecture (entity-component-system) é fundamentalmente
diferente do model reativo do plev. UI é secundário ao game engine.

---

tier 5 - fora do ecossistema rust (referências cross-industry)

13. flutter (skia/impeller)
    github.com/flutter/flutter

justificativa: referência de arquitetura mais próxima em termos de conceito.
flutter também possui rendering próprio (impeller, sucessor do skia), sem DOM,
cross-platform completo (mobile, desktop, web). o posicionamento do plev como "skia
para rust" espelha o que impeller é para flutter. diferença: dart vs rust, gc vs
ownership.

14. vello
    github.com/linebender/vello

justificativa: GPU 2d renderer puro, compute shader-based (não vertex shaders como
plev). mesmo ecossistema wgpu. é o renderer do xilem. abordagem técnica oposta:
vello faz path rendering via compute, plev faz SDF + tessellation via
vertex/fragment pipelines.

---

mapa de posicionamento

                      GPU-first
                         |
                         |
            makepad      |      plev
                         |          gpui (zed)
            floem        |      iced
                         |
      framework <--------+--------> engine/compositor
                         |
            slint        |      vello
            dioxus       |
                         |
            egui         |
                         |
                     cpu/software

o nicho único do plev: motor de composição GPU-first (nível skia/impeller) com
acessibilidade nativa, dirty tracking por hash, shaders webgpu unificados e ~30k
LOC. nenhum outro projeto rust combina todas essas propriedades simultaneamente.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [makepad visual feature inventory vs plev gap analysis](makepad-gap-analysis.md)
- [technology radar, plev](technology-radar.md)
<!-- /related -->
