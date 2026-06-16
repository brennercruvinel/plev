---
project: plev
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-11
domain: competitive
---

# posicionamento competitivo, plev

**data:** 2026-03-11
**baseado em:** refs/competitors.md, refs/linebender-ecosystem.md, mission/plan/readme.md secoes 5-6

---

## matriz comparativa atualizada

### por modelo de rendering

| framework | modelo | GPU backend | text quality | dirty tracking | camadas | targets |
|-----------|--------|-------------|-------------|----------------|---------|---------|
| **plev** | hybrid (scene graph/frame) | wgpu (metal/vulkan/dx12/webgpu) | top (cosmic-text atlas GPU) | per-layer fxhash | independentes + composite | 6 (macos, ios, linux, android, windows, WASM) |
| makepad | hybrid (immediate/retained) | custom (metal/dx11/opengl/webgl) | basica (implementacao propria, falhas cjk) | shader-level | nao documentado | 6 (webgl, nao webgpu) |
| iced | elm architecture | wgpu | boa (cosmic-text + glyphon) | parcial (re-render full) | sem camadas independentes | nativo + WASM (experimental) |
| egui | immediate mode | wgpu (backend) | mediocre (CPU-side) | nenhum (flat) | nenhuma | nativo + WASM |
| GPUI (zed) | hybrid | wgpu (metal/vulkan) | top (core-text atlas) | sim (camadas 120fps) | independentes | macos, linux (sem WASM) |
| slint | retained + DSL | skia/femtovg/software | variavel por backend | retained auto | sem acesso GPU direto | nativo + WASM (degradado) |

### por modelo de aplicacao

| framework | nivel | stars | proposta | relacao com plev |
|-----------|-------|-------|----------|------------------|
| dioxus | app framework | 35.236 | "react do rust" fullstack | plev e a camada abaixo |
| leptos | app framework (web) | 20.370 | signals fine-grained, ssr | referencia de reatividade |
| yew | app framework (web) | 32.469 | vdom legacy | irrelevante |
| ribir | gui framework | 1.665 | wgpu single-framework | pre-alpha, anti-pattern de scope creep |
| compose mp | app framework (kotlin) | 18.888 | multi-platform comercial | referencia de modelo, ecossistema separado |
| xilem | gui framework | 4.903 | swiftui-inspired, vello | referencia de diff pattern |

---

## posicionamento do plev

### nicho unico
plev ocupa uma posicao que nenhum outro projeto preenche: **compositing engine GPU-first standalone**, como skia e para flutter/chrome, mas em rust puro com wgpu e shaders identicos em todos os targets.

nao e um framework de aplicacao (dioxus/leptos/slint), nao e um game engine (bevy), nao e um editor acoplado (GPUI/zed). e a camada de rendering que qualquer um desses poderia usar.

### vantagens competitivas confirmadas pela pesquisa

1. **dirty tracking per-layer via fxhash**, nenhum competidor wgpu-based tem isso (iced re-renderiza tudo, egui e flat, makepad nao documenta)
2. **text rendering de producao**, cosmic-text com atlas GPU, qualidade zed/sublime. makepad tem falhas cjk, egui e CPU-side, iced tem glyphon issues
3. **webgpu real**, mesmos shaders WGSL em 6 targets. makepad usa webgl (inferior). GPUI nao tem WASM. iced tem web experimental
4. **scene graph com hash de estado**, frame identico = zero GPU work. nenhum competidor immediate-mode tem isso
5. **premultiplied alpha em todo o pipeline**, correto desde o shader ate o composite pass

### onde competidores sao superiores (e relevancia)

| competidor | superioridade | impacto no plev |
|------------|---------------|-----------------|
| makepad | live editing de shaders | baixo, dx feature, nao core rendering |
| dioxus | ecossistema (35k stars, hot-reload) | nenhum, camada diferente |
| leptos | reatividade granular testada em producao | medio, inspiracao para signal system |
| slint | maturidade comercial (1.15.x, empresa) | medio, referencia de estabilidade |
| GPUI | performance insana em native | baixo, plev precisa de WASM, GPUI nao tem |
| xilem | diff-on-retained-tree | medio, pattern futuro para otimizacao |
| compose mp | suporte comercial jetbrains | nenhum, ecossistema kotlin |

### riscos identificados

1. **acessibilidade zero**, makepad tambem tem zero e sofre criticas pesadas. plev deve resolver antes da 1.0 (task-30)
2. **ecossistema jovem**, 0 stars (privado). precisa de demo app convincente (task-29) e paper (task-26)
3. **cosmic-text pode ficar atras do parley**, avaliar em task-32 apos task-28
4. **sem documentacao de DSL**, makepad morreu na adocao por falta de docs. plev_narrate deve ter docs excelentes (task-21 parcial)

---

## conclusao

plev resolve um problema real que ninguem resolveu: compositing unificado GPU-first com dirty tracking, text de producao, e shaders identicos em 6 targets. o risco nao e tecnico, e de execucao: acessibilidade, documentacao, e demo app sao os 3 gates para relevancia.
