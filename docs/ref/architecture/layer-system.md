---
id: architecture-layer-system
title: "layer system - technical decisions"
project: plev
type: reference
status: reference
date: 2021-06-16
date_basis: header
topics: [rendering, gpu]
relations: []
source: docs/ref/layer-system.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# layer system - technical decisions

Records the technical decisions of the layer system: premultiplied alpha across the whole pipeline, one offscreen texture per layer and a composition pass with a full-screen triangle. The result is that a layer with no changes generates no render pass, geometry or shaping, and each visible layer costs one draw in the composite; the page estimates about 8 MB of VRAM per layer at 1920×1080. The design matters because it is the basis of the steady-state cost of composition.

## premultiplied alpha
- mudança de `SrcAlpha/OneMinusSrcAlpha` para `One/OneMinusSrcAlpha` em todo o pipeline
- shaders outputam `rgb * a, a` (premultiplied)
- necessário para o operador `over` funcionar corretamente na composição de layers
- visualmente idêntico para cores opacas

## arquitetura de layers
- cada layer tem: textura offscreen (RGBA), hash de cena próprio, buffers quad/text próprios
- texturas criadas lazily no resolve(), recriadas no resize
- default layer (id=0) sempre existe, `push()` vai para ela
- layers ordenadas por z_order

## composite pass
- full-screen triangle via vertex_index (sem vb, 3 verts)
- shader em `shaders/composite.wgsl`
- bind group 0: textura da layer + sampler
- bind group 1: opacity uniform (f32)
- um draw(0..3) por layer visível

## textsystem
- `resolve_for_layer()` limpa staging, emite glyphs, retorna `(Vec<TextVertex>, Vec<u32>)`
- layer copia esses dados para seus próprios buffers
- `finish_frame()` faz eviction do shaping cache
- atlas de glifos continua global e compartilhado

## performance em steady state
- layer sem mudanças = 0 render passes, 0 geometry rebuild, 0 shaping
- apenas 1 draw call no composite pass (full-screen triangle)
- vram: ~8mb por layer em 1920x1080

## gpuvec compartilhado
- extraído para `src/gpu_vec.rs`, usado por compositor (per-layer) e text system
- grow-only, never shrink, partial writes via `queue.write_buffer`
