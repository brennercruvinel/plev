---
id: architecture-layout-engine
title: "layout engine - decisions and knowledge"
project: plev
type: reference
status: reference
date: 2021-04-15
date_basis: header
topics: [layout, performance]
relations: []
source: docs/ref/layout-engine.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# layout engine - decisions and knowledge

Records the choice of taffy 0.9 as the layout engine instead of an in-house implementation, with the measurement of 89 µs for 1000 nodes in release, and the notes on the API used. The result is a `LayoutEngine` that computes absolute coordinates in two phases per frame: collection of the layout items and then rendering with the `ComputedBounds`. The design matters because the dependency and the per-frame cost determine the engine's layout budget.

## decisao: taffy 0.9 (nao implementacao custom)

- 89us para 1000 nodes em release mode (bem abaixo do <1ms requisito)
- zero deps externas, rust puro
- battle-tested: zed, bevy, servo, slint, lapce
- API simples: taffytree + style struct + compute_layout
- implementacao custom seria 300-500 loc basico, 3000+ para edge cases

## arquitetura: two-phase rendering

```
Frame:
1. Walk View tree → collect LayoutItems (DFS, pre-order indexing)
2. LayoutEngine::compute() → Vec<ComputedBounds> (absolute coords)
3. Walk View tree → pass ComputedBounds via ViewContext → render SceneNodes
4. Push nodes to compositor → resolve → render pass (inalterado)
```

## API taffy 0.9.2 - notas importantes

- `Dimension`, `LengthPercentage`, `LengthPercentageAuto` sao structs (nao enums)
  - usar `.length()`, `.percent()`, `.auto()` constructors
- alignment fields sao `Option<T>`: `Some(AlignItems::Center)`, nao bare
- `JustifyContent` = type alias para `AlignContent`
- `Layout.location` e relativo ao parent - precisa dfs acumulando offsets para absolute
- `gap` e `Size<LengthPercentage>` com `.width` = column gap, `.height` = row gap
- `Rect` fields: `left, right, top, bottom` (nao start/end)
- root node com auto size = 0px - precisa explicit size ou ser child com flex_grow

## tipos plev (wrapping taffy)

- `Direction`: row | column
- `Align`: start | center | end | stretch
- `Justify`: start | center | end | spacebetween | spacearound | spaceevenly
- `LayoutStyle`: direction, align, justify, padding[4], gap, width/height/min/max, flex_grow/shrink
- `ComputedBounds`: x, y, width, height (absolute screen coords)
- `LayoutItem`: style + children indices
- `LayoutEngine`: owns taffytree, `.compute()` returns vec<computedbounds>

## performance

- debug mode: ~34ms para 1001 nodes (taffy sem otimizacoes)
- release mode: <1ms para 1001 nodes (conforme benchmark oficial)
- taffytree.clear() reutiliza entre frames (evita realocacao)
