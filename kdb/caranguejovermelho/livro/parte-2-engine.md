---
title: a engine, por dentro
parte: 2
status: stub
rastros: []
---

# parte 2, a engine, por dentro

espinha de secoes. so a estrutura, sem corpo. um capitulo por subsistema da
engine, ancorado nos adrs de kdb/adr/ que registraram cada decisao.

## 2.1 gpu, device, surface, pipelines, GpuVec

### device e surface via wgpu 28
### pipelines de quad e a reuse de buffers
### surface render target so via gpu.surface_render_view

## 2.2 cor, srgb e linearizar uma vez

### srgb na entrada, linear na gpu (linearize-colors-before-the-gpu)
### render-into-an-srgb-view-format
### to_linear_array para clears e uniforms

## 2.3 compositor e camadas

### scene nodes e dirty layers (layer-system)
### premultiplied alpha em todo o pipeline
### invalidacao, render-on-demand-requires-explicit-invalidation

## 2.4 effects, blur, shadow e o composite pass

### fragment-only blur e shadow (effects-architecture)
### texturepool grow-only
### full-screen triangle no composite pass

## 2.5 text, shaping, atlas, uma TextStyle por run

### shaping cache e atlas de glifos
### uma TextStyle por run (one-text-style-for-measurement-and-drawing)
### TextMeasurer, medir e desenhar com o mesmo estilo

## 2.6 fontes, embed de cada peso em uso

### por que embutir, embed-every-font-weight-in-use
### pesos em uso vs pesos sinteticos

## 2.7 layout, taffy e a geometria que deriva do espaco

### taffy 0.9 wrappado, two-phase rendering (layout-engine)
### geometria deriva do espaco (content-driven-layout-not-fixed-constants)
### measure-fns e computed bounds absolutos

## 2.8 tokens medidos e a projecao hidpi

### measured-design-tokens-over-eyeballed-values, oklch
### adr-004-hidpi-projection, set_projection em retina

## 2.9 input, fila de eventos e hit-testing

### event queue em vez de closures (input-system-design)
### hit-testing linear reverso, click-to-focus
### modifierstate proprio

## 2.10 touch e gesto, o pointer sintetizado

### touch-as-synthesized-pointer-events
### 6-state gesturerecognizer (touch-gesture-design)
### instant explicito para testabilidade

## 2.11 animation, tween, spring, frameclock

### frame-based lerp e pixel-snap (animation-pattern-lerp)
### tween, spring analitico, frameclock
### motion-trails-by-position-history

## 2.12 signal, o reativo push-pull

### push-pull hibrido, slotmap runtime (signal-system-design)
### rc closures para borrow safety
### RAII observer guard, fxindexset, peek

## 2.13 view, component e builder

### a arvore declarativa de elementos
### view-trait-design e component-design
### dsl narrate como acucar verbal (dsl-narrate-design)

## 2.14 render on demand, a invalidacao explicita

### render-on-demand-requires-explicit-invalidation
### todo handler que muda estado visivel invalida

## 2.15 hot reload, shader e narrate

### shader-hot-reload-adr, notify watcher e pipeline recreation
### narrate-hot-reload-adr, file watcher e override map
### a analise de fronteira, hot-reload-design

## 2.16 perf, PerfMonitor e o hud

### PerfMonitor, janelas rolantes de fps e percentis
### hud implica frames continuos
### pure, sem gpu
