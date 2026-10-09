---
project: plev
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-11
domain: text
---

# ecossistema linebender, analise para plev

**data:** 2026-03-11
**contexto:** pesquisa de referencia para task-32 (text upgrade assessment) e decisoes arquiteturais futuras do plev.
**metodologia:** dados coletados via github API, crates.io API, blogs oficiais linebender (tmil-11..24), deepwiki, e prs de projetos consumidores (egui, bevy, slint).

---

## sumario executivo

o ecossistema linebender e a tentativa mais ambiciosa de construir um stack de rendering 2d completo em rust puro, de SIMD ate gui declarativa. sete projetos interconectados, liderados por raph levien (ex-google fonts), com financiamento NLnet para 2026. para plev, o ponto de maior impacto imediato e parley como substituto do cosmic-text (task-32). os demais projetos oferecem licoes arquiteturais, nao dependencias diretas.

---

## projetos individuais

### 1. vello (linebender/vello), 3.828 stars, v0.7.0 (2026-01-13)

**o que e:** renderer 2d compute-first que usa prefix-sum em GPU para paralelizar operacoes tradicionalmente sequenciais (sorting, clipping, rasterizacao).

**arquitetura:**
- **3 backends:** GPU puro (compute shaders), CPU puro (SIMD via fearless_simd), hibrido (CPU geometry + GPU rasterization, compativel webgl2)
- **pipeline de encoding:** scene API canvas-like -> encoding binario em 5 streams paralelos (tag, path, draw, transform, linewidth) -> 13+ compute shader stages -> output texture
- **prefix-sum:** monoid scan em 4 streams (transform, path, draw object, clip), cada um e um compute dispatch unico. path segments usam prefix-sum exclusivo sobre tamanho do payload para calcular offsets
- **tile-based:** binning espacial em tiles 16x16, coarse rasterization gera per-tile command lists (ptcl), fine rasterization com aa configuravel (area/msaa8/msaa16)
- **wgpu 28**, MSRV 1.86, webgpu target com limites default
- **performance:** 177 fps para paris-30k (1600x1600) em m1 max. otimizacao recente: -30% memoria em wide tile commands, eliminacao de overdraw para imagens opacas
- **hibrido (novo 2025):** roda em webgl2 (firefox sem webgpu), blending completo, gradientes, multiplos atlas de imagem. vello CPU usa sparse strips com SIMD

**relevancia para plev:**
- o modelo de encoding em streams paralelos e superior ao nosso scenenode -> hash -> upload. plev poderia adotar encoding incremental em streams separados para dirty tracking mais granular
- o backend hibrido resolve o problema que plev tem com webgl2 fallback, estudar a divisao CPU/GPU
- o pattern scene -> encoding -> resolver (atlas, gradientes, glyph cache) e mais limpo que nosso acoplamento compositor/text
- tile-based coarse/fine e relevante se plev evoluir para path rendering (task-31)

**insight principal:** o encoding em streams multiplexados com prefix-sum e a inovacao real. nao e o compute shader em si, e a estrutura de dados que permite paralelismo. plev pode aprender o pattern de streams sem adotar compute shaders.

**limitacao:** requer compute shaders (webgpu), o que exclui webgl2 no backend puro. o backend hibrido e novo e incompleto. alpha state, API instavel, breaking changes frequentes. complexidade enorme (13+ shader stages) para quem so precisa de quads + texto.

---

### 2. xilem (linebender/xilem), 4.903 stars, v0.4.0 (2025-10-29)

**o que e:** framework gui declarativo inspirado em swiftui/elm/react, com diff-on-retained-tree sobre o toolkit de widgets masonry.

**arquitetura:**
- **camadas:** app state -> view tree (lightweight) -> diff -> element tree -> widget tree (retained, masonry) -> scene (vello) -> GPU
- **reconciliacao:** view tree e reconstruido a cada update. diff contra view tree anterior gera set minimo de mutacoes no widget tree retido. similar a react fiber mas tipado estaticamente
- **masonry:** toolkit de widgets nao-opinado, retained tree com event handling e update passes. desenhado como base para qualquer paradigma (immediate, elm, frp)
- **stack completo:** winit (window), vello + wgpu (rendering), parley + fontique (texto), accesskit (acessibilidade)
- **xilem web:** backend alternativo para browser via DOM, mesmo modelo de view tree
- **MSRV 1.88**, plataformas: linux, BSD, macos, windows. ios/android nao suportado ainda

**relevancia para plev:**
- o padrao de diff-on-retained-tree e diretamente relevante para o sistema de views/components de plev (task-05). plev reconstroi a scene inteira por frame; xilem mostra como fazer diff incremental
- a separacao xilem (reactive) / masonry (retained widgets) e analoga a separacao que plev precisa entre builder API e compositor
- accesskit integration e referencia para task-30
- o pattern de view tree tipado estaticamente evita o overhead de virtual dispatch que plev tem com `Box<dyn View>`

**insight principal:** a separacao em 3 camadas (view tree efemero -> element tree intermediario -> widget tree retido) permite que o paradigma declarativo mude sem tocar na infra de rendering. plev mistura as 3 camadas.

**limitacao:** 6.286 downloads totais, adocao minima. experimental, API instavel. nao suporta mobile. depende de vello (compute shaders), o que limita targets. para plev, que ja tem seu proprio pipeline de rendering, adotar xilem nao faz sentido, mas o pattern de diff e valioso.

---

### 3. parley (linebender/parley), 534 stars, v0.7.0 (2025-11-24)

**o que e:** biblioteca de layout de texto rico, resolve posicoes de glifos, line breaking, bidi, com stack 100% rust (harfrust + skrifa + fontique + icu4x).

**arquitetura:**
- **4 dependencias core:**
  - **harfrust:** port completo do harfbuzz em rust puro. sem dependencia c. sem unsafe. shaping completo incluindo ligatures, emoji modifiers, scripts complexos
  - **skrifa:** leitor de fontes truetype/opentype (sobre read-fonts). converte glifos raw em paths vetoriais escalados/hinted
  - **fontique:** enumeracao de fontes e fallback. em WASM, usa fonte embedded (dejavu sans trimada)
  - **icu4x:** analise unicode, locale, bidi, segmentacao, normalizacao. migrado de implementacoes proprias em dez/2025
- **plaineditor:** API de edicao de texto com cursor, selecao, triple-click (seleciona paragrafo), shift-click, geometria de selecao por linha
- **cursor model:** byte index (nao line_index como cosmic-text). simplificacao significativa do modelo em 2025 resolveu bugs historicos
- **inline boxes:** suporte nativo para widgets inline (dropdowns, checkboxes) dentro do texto, cosmic-text nao tem
- **text styling:** textwrapmode, word/letter spacing, word-break, overflow-wrap (CSS-like)
- **MSRV 1.88**, 521.443 downloads

**API para cursor positioning:**
```
PlainEditor::cursor() -> Cursor (byte index)
PlainEditor::selection_geometry() -> Vec<Rect> com line indices
PlainEditor::raw_selection() -> Range<usize>
Selection::geometry_with() -> geometria customizada
```
cursor e byte index puro. geometria de selecao inclui indice da linha. newlines selecionados sao exibidos como whitespace no highlight.

**API para selection ranges:**
```
Selection::new(anchor: Cursor, focus: Cursor)
Selection::geometry(layout) -> Vec<SelectionRect>
PlainEditor com shift-click, triple-click (paragrafo), Ctrl+A
```

**compatibilidade WASM:**
- compila para wasm32-unknown-unknown com features default (fix recente 2025)
- fontique em WASM: sem system fonts, usa fonte embedded como fallback universal
- harfrust: rust puro, sem FFI, compila trivialmente para WASM
- icu4x: rust puro, dados unicode embedded, WASM-safe
- **ponto de atencao:** tamanho do binario WASM pode crescer com icu4x data tables

**harfrust vs harfbuzz (dependencia):**

| aspecto | harfrust (parley) | harfbuzz-c (legado) |
|---------|-------------------|---------------------|
| linguagem | rust puro, zero unsafe | c++ |
| WASM | compilacao trivial | requer emscripten/wasm-bindgen wrapper |
| tamanho | menor (sem runtime c) | maior com libc |
| features | shaping completo, sem freetype/coretext/uniscribe integration | integracao com system shapers |
| manutencao | google fonts (ativo) | google fonts (ativo) |

**nota:** cosmic-text 0.18 tambem usa harfrust (^0.5.0). ambas as libs compartilham o mesmo shaper agora.

**relevancia para plev:**
- **candidato direto para substituir cosmic-text** (task-32). vantagens: inline boxes, cursor/selection API superior, accesskit built-in, icu4x para unicode correto, adocao por egui e slint
- plaineditor resolve problemas que plev precisaria implementar manualmente sobre cosmic-text para task-28 (editable text)
- fontique substitui o pattern manual de font loading que plev tem (cfg gates para WASM/ios/android)
- a migracao de icu4x (dez/2025) melhora correcao unicode mas pode impactar tamanho WASM

**insight principal:** parley esta convergindo como o padrao de fato para texto em rust, egui migrando, slint ja migrou (v1.14), bevy avaliando. a stack harfrust + skrifa + fontique + icu4x e 100% rust sem FFI, o que alinha perfeitamente com a filosofia de plev (um codebase, seis targets).

**limitacao:** 534 stars vs 2.004 do cosmic-text. menos battle-tested em producao. API ainda em evolucao (0.7.0). migracao requer reescrever o textsystem de plev (shaping cache, atlas, borrow split pattern). benchmarks quantitativos parley vs cosmic-text ainda nao publicados (prometidos para nov/2025, nao confirmados). MSRV 1.88 (plev usa 1.94, ok).

---

### 4. velato (linebender/velato), 134 stars, v0.9.0 (2026-01-18)

**o que e:** parser/renderer lottie sobre vello, converte animacoes lottie em `vello::Scene`.

**arquitetura:**
- input: JSON lottie -> parse -> intermediate representation -> encode para vello scene
- cobertura da spec lottie: incompleta mas funcional para grande numero de animacoes
- depende diretamente de vello para rendering

**relevancia para plev:**
- se plev implementar task-27 (animation system) e task-31 (vector paths), lottie support seria possivel
- o pattern de "parser -> intermediate repr -> scene encoding" e reusavel: plev poderia ter seu proprio encoder lottie que emite scenenodes em vez de vello::scene
- demonstra que animacoes complexas sao viaveis com scene graph + path rendering

**insight principal:** lottie e o formato de animacao mais usado em apps mobile. ter support e diferencial competitivo. mas depende de path rendering (task-31) que plev ainda nao tem.

**limitacao:** acoplado a vello. para plev, seria necessario reimplementar o encoding. cobertura incompleta da spec lottie. baixa adocao (13.848 downloads).

---

### 5. kurbo (linebender/kurbo), 920 stars, v0.13.0 (2025-11-27)

**o que e:** biblioteca de geometria 2d, curvas de bezier (ate cubicas), paths, arcos, formas. foco em precisao numerica para engenharia/ciencia.

**arquitetura:**
- tipos: point, vec2, line, rect, roundedrect, circle, ellipse, arc
- curvas: quadbez, cubicbez, bezpath (sequencia de segmentos)
- operacoes: nearest point, area, winding number, bounding box, flattening (curva -> segmentos de reta)
- parametro de precisao em funcoes aproximadas (accuracy-driven, nao step-count-driven)
- otimizacao recente: `CubicBez::nearest` 3000x mais rapido via poly-cool quintic solver
- 16.4m downloads, MSRV 1.82, no_std compativel

**relevancia para plev:**
- **dependencia direta candidata** para task-31 (vector paths). kurbo fornece a geometria; plev precisaria de tessellation (lyon) ou encoding para GPU
- bezpath -> flatten -> line segments e exatamente o que plev precisaria para path rendering
- rect, roundedrect ja sao tipos que plev reimplementa. kurbo e mais correto (precisao numerica)
- usado por vello, peniko, xilem, linguagem comum do ecossistema

**insight principal:** kurbo e a biblioteca de geometria 2d mais madura e correta em rust. 16m downloads demonstra confianca do ecossistema. para task-31, usar kurbo em vez de implementar proprios tipos geometricos e a decisao correta.

**limitacao:** nao faz rendering, so geometria. plev precisaria de tessellation (lyon/etagere) ou shader de curvas para desenhar os paths. adiciona dependencia mas e leve (no_std).

---

### 6. peniko (linebender/peniko), 78 stars, v0.6.0 (2026-01-09)

**o que e:** tipos compartilhados para estilizacao de graficos vetoriais, color, brush, gradient, image, blob. "cola" tipada entre kurbo (geometria) e renderers (vello).

**arquitetura:**
- tipos: color (CSS color level 4 via crate `color`), brush, brushref, gradient (linear, radial, sweep), gradientstop, image, blob
- color: suporta multiplos color spaces (srgb, display-p3, etc), parsing CSS, conversao
- brush: `with_alpha()`, `multiply_alpha()`, composicao de opacidade
- gradient: stops com posicao, extend modes
- sem geometria (kurbo), sem rendering (vello), apenas tipos de estilo
- 772.279 downloads

**relevancia para plev:**
- plev usa `[f32; 4]` para cores e structs proprias para gradientes. peniko oferece tipos mais ricos e corretos (CSS color level 4, color spaces)
- se plev adotar kurbo (task-31) e/ou parley, peniko viria como dependencia transitiva
- o pattern de separar "tipos de estilo" em crate independente e bom para API publica

**insight principal:** peniko e invisivel mas ubiquo, e a linguagem de tipos entre todos os crates linebender. adotar peniko (ou seus patterns) unifica a API de estilo de plev.

**limitacao:** dependencia adicional sem beneficio imediato se plev nao adotar outros crates linebender. o sistema de color e overkill para quem so precisa de srgb com premultiplied alpha. plev ja tem premultiplied alpha em todo o pipeline, peniko usa straight alpha por default.

---

### 7. fearless_simd (linebender/fearless_simd), 262 stars, v0.4.0 (2026-02-13)

**o que e:** abstracoes SIMD seguras e portaveis, NEON (aarch64), sse4.2/AVX (x86), WASM SIMD. prova de capacidade via "marker values" (zero-sized tokens como prova de feature disponivel).

**arquitetura:**
- **runtime dispatch:** `simd_dispatch!` macro gera code paths multiversionados. token zst prova que a feature esta disponivel, chamadas SIMD sao safe
- **trait `Simd`:** operacoes core (add, mul, fma, blend, comparacoes) implementadas por plataforma
- **suporte atual:** NEON, WASM SIMD (relaxed), sse4.2. AVX parcial. AVX-512 experimental
- **novidades 2025:** operacoes de mask (`any_true`, `all_true`), `ceil`, `round_ties_even`, conversoes float-int corretas em x86, `Element` como associated type, tipos vetoriais nativos
- **codegen:** `fearless_simd_gen` gera boilerplate automaticamente
- **451.015 downloads**, MSRV 1.88

**relevancia para plev:**
- plev nao usa SIMD explicitamente, mas se implementar CPU fallback (como vello CPU) ou otimizacoes de layout/text, fearless_simd seria a escolha
- para task-25 (benchmarks), SIMD pode ser o diferencial em text shaping ou atlas packing
- a abordagem "safe SIMD via marker types" e elegante e evita o problema de `unsafe` spread que SIMD normalmente causa

**insight principal:** SIMD portavel seguro em rust agora e viavel. se plev precisar de hot path otimizado (atlas packing, geometry flattening, color conversion), fearless_simd e a opcao. mas e prematuro adicionar antes de ter profiling que justifique.

**limitacao:** pre-producao, apis incompletas, breaking changes frequentes. boilerplate de codegen significativo. nao suporta avx2/AVX-512 completo. para plev, so faria sentido apos profiling mostrar que um hot path especifico e CPU-bound e SIMD-amigavel.

---

## comparacao detalhada: parley vs cosmic-text

tabela de comparacao direta para informar task-32:

| criterio | parley 0.7.0 | cosmic-text 0.18.2 |
|----------|-------------|---------------------|
| **stars** | 534 | 2.004 |
| **downloads** | 521.443 | 3.662.966 |
| **shaping engine** | harfrust (pure rust) | harfrust (pure rust) |
| **font enumeration** | fontique | fontdb |
| **font reading** | skrifa (read-fonts) | skrifa (read-fonts) |
| **unicode analysis** | icu4x (migrado dez/2025) | implementacao propria |
| **cursor model** | byte index | line index + byte index |
| **selection API** | selection::geometry_with(), por linha | editor com selecao, menos granular |
| **inline boxes** | sim (push_inline_box) | nao |
| **rich text spans** | sim, com builder API | sim, com attrslist |
| **plaineditor** | sim (cursor, selecao, triple-click, shift-click) | editor wrapper sobre buffer |
| **accesskit** | built-in | nao |
| **WASM** | sim (fonte embedded via fontique) | sim (fonte embedded) |
| **bidi** | icu4x | implementacao propria |
| **line breaking** | icu4x segmentation | implementacao propria |
| **word-break/overflow-wrap** | sim (CSS-like) | parcial |
| **performance** | "significativamente melhor em paragrafos grandes" (nao quantificado) | baseline |
| **adocao** | egui (wip), slint (v1.14), xilem | bevy (0.15+), cosmic de, zed, lapce, iced |
| **manutencao** | linebender + NLnet grants 2026 | system76 (cosmic de) |
| **MSRV** | 1.88 | nao documentado |
| **estabilidade API** | pre-1.0, breaking changes | pre-1.0, mais estavel |

### veredito para plev (task-32)

**recomendacao: migrar para parley, mas nao antes de task-28 (editable text).**

razoes a favor:
1. plaineditor resolve 80% do trabalho de task-28 (cursor, selecao, shift-click, geometria por linha)
2. inline boxes sao necessarios para widgets dentro de texto (futuro de plev)
3. icu4x garante correcao unicode (bidi, segmentacao) que implementacao propria do cosmic-text pode errar
4. tendencia de ecossistema: egui, slint, potencialmente bevy estao migrando
5. NLnet grants garantem desenvolvimento ativo em 2026
6. mesma stack de shaping (harfrust), a migracao nao muda o shaper

razoes contra:
1. cosmic-text tem 7x mais downloads, mais battle-tested
2. benchmarks quantitativos nao publicados
3. API ainda instavel (0.7.0)
4. migracao requer reescrever textsystem de plev (shaping cache, atlas management, borrow split pattern)
5. fontique pode ter gaps vs fontdb em plataformas especificas

**estrategia sugerida:**
1. implementar task-28 com cosmic-text (ja funciona)
2. criar branch experimental `task/TASK-32-parley-eval`
3. portar textsystem para parley, manter cosmic-text como fallback via feature flag
4. comparar: tamanho WASM, latencia de shaping, API ergonomics, cursor behavior
5. decidir baseado em dados, nao em hype

---

## padroes cross-cutting

### 1. stack rust puro (zero FFI)
todos os crates linebender evitam dependencias c. harfrust substitui harfbuzz-c, skrifa substitui freetype, icu4x substitui ICU-c. isso simplifica cross-compilation para todos os 6 targets de plev e elimina a classe inteira de bugs de FFI/linking.

### 2. encoding em streams separados
vello separa scene data em 5+ streams (tag, path, draw, transform, linewidth) para processamento paralelo. plev usa um unico `Vec<SceneNode>` por layer. a separacao em streams permitiria dirty tracking por tipo de dado (so transforms mudaram? so redspacha o stream de transforms).

### 3. trait-based abstraction com types compartilhados
kurbo (geometria) + peniko (estilo) formam a linguagem de tipos entre todos os crates. plev usa tipos ad-hoc (`[f32; 4]` para cor, structs proprias para rect). padronizar tipos facilitaria interop futura.

### 4. view tree efemero -> retained tree
xilem reconstroi view tree completo por update, faz diff, aplica mutacoes minimas no widget tree retido. plev reconstroi a scene inteira por frame. para UI complexa, o diff de xilem sera ordens de magnitude mais eficiente.

### 5. fallback gracioso (GPU -> hibrido -> CPU)
vello oferece 3 backends. plev assume compute-capable GPU (wgpu). para cobertura real de 6 targets (especialmente webgl2 browsers), um fallback CPU ou hibrido seria necessario. vello demonstrou que e possivel com a mesma API.

### 6. SIMD como otimizacao de hot path
fearless_simd e usado por vello CPU para rasterizacao. o pattern e: profiling primeiro, SIMD depois, nunca prematuramente. plev deveria seguir o mesmo princio, SIMD so apos task-25 (benchmarks) identificar bottlenecks.

---

## implicacoes para plev

### curto prazo (task-27..29)
- **task-28 (editable text):** avaliar usar plaineditor do parley em vez de implementar cursor/selecao manualmente sobre cosmic-text. se cosmic-text for mantido, implementar manualmente e mais trabalho mas mantem dependencia atual
- **task-27 (animation):** velato mostra que lottie e possivel com path rendering, mas plev precisa de task-31 primeiro. animation system proprio e a decisao correta por agora

### medio prazo (task-30..32)
- **task-31 (vector paths):** kurbo para geometria. tessellation via lyon ou encoding proprio para GPU. nao adotar vello inteiro, so os tipos geometricos
- **task-32 (text upgrade):** parley e o candidato. estrategia de feature flag para avaliacao sem commitment
- **task-30 (accessibility):** parley tem accesskit built-in. se migrar texto para parley, acessibilidade de texto vem de graca

### longo prazo (task-33+)
- **diff incremental:** o pattern de xilem (view tree efemero -> diff -> retained tree) e o proximo salto de performance para plev. considerar como evolucao do sistema de views/components
- **fallback rendering:** se plev quiser webgl2 (firefox sem webgpu), o modelo hibrido de vello e referencia. nao copiar, entender a divisao CPU/GPU
- **tipos padronizados:** adotar kurbo + peniko gradualmente alinha plev com o ecossistema rust de graficos 2d

### o que nao fazer
- **nao adotar vello como renderer.** plev tem seu proprio pipeline otimizado para quads + texto. vello e para path rendering generico, overhead desnecessario para o caso de uso de plev
- **nao adotar xilem como framework.** plev e a camada abaixo, equivalente a masonry, nao a xilem. as licoes de diff sao valiosas, a dependencia nao
- **nao adicionar SIMD prematuramente.** sem profiling (task-25), SIMD e otimizacao especulativa
- **nao migrar para parley sem dados.** feature flag, benchmark, decisao baseada em evidencia

---

## fontes

- [vello github](https://github.com/linebender/vello), 3.828 stars, v0.7.0
- [xilem github](https://github.com/linebender/xilem), 4.903 stars, v0.4.0
- [parley github](https://github.com/linebender/parley), 534 stars, v0.7.0
- [velato github](https://github.com/linebender/velato), 134 stars, v0.9.0
- [kurbo github](https://github.com/linebender/kurbo), 920 stars, v0.13.0
- [peniko github](https://github.com/linebender/peniko), 78 stars, v0.6.0
- [fearless_simd github](https://github.com/linebender/fearless_simd), 262 stars, v0.4.0
- [cosmic-text github](https://github.com/pop-os/cosmic-text), 2.004 stars, v0.18.2
- [linebender blog - november 2025](https://linebender.org/blog/tmil-23/)
- [linebender blog - october 2025](https://linebender.org/blog/tmil-22/)
- [linebender blog - december 2025](https://linebender.org/blog/tmil-24/)
- [towards fearless SIMD](https://linebender.org/blog/towards-fearless-simd/)
- [vello architecture - deepwiki](https://deepwiki.com/linebender/vello/1.1-architecture)
- [egui parley migration pr #5784](https://github.com/emilk/egui/pull/5784)
- [bevy parley discussion #21765](https://github.com/bevyengine/bevy/issues/21765)
- [slint fontique migration pr #9564](https://github.com/slint-ui/slint/pull/9564)
