---
id: index-Research
title: Research
project: plev
type: index
status: active
date: null
date_basis: generated
topics: [navigation]
relations: []
source: tool/tasks/docbase.py
source_revision: generated
revised: generated
review: generated
---

# Research

What we know, by the themes the ADRs use: the references studied, the comparisons, the thesis and the program, how each subsystem was designed and how to work on the repository.

## Ecosystem

| Date | Document | Status |
|---|---|---|
| 2022-06-15 | [competitive positioning, plev](Ecosystem/competitive-positioning.md) | reference |
| 2022-06-15 | [competitor analysis, gui/rendering frameworks](Ecosystem/competitors.md) | reference |
| 2022-06-15 | [reference analysis: emulators and WASM runtimes](Ecosystem/emulators-wasm-runtimes.md) | reference |
| 2022-06-15 | [integration candidates, ranking](Ecosystem/integration-candidates.md) | reference |
| 2022-06-15 | [linebender ecosystem, analysis for plev](Ecosystem/linebender-ecosystem.md) | reference |
| 2022-06-15 | [technology radar, plev](Ecosystem/technology-radar.md) | reference |
| 2022-06-15 | [reference analysis: WASM tooling ecosystem](Ecosystem/wasm-tooling.md) | reference |
| 2022-10-21 | [competitors by tier](Ecosystem/competitors-by-tier.md) | reference |
| 2022-11-19 | [makepad visual feature inventory vs plev gap analysis](Ecosystem/makepad-gap-analysis.md) | reference |
| 2022-12-10 | [technical report: plev vs makepad](Ecosystem/plev-vs-makepad-report.md) | reference |
| 2023-04-06 | [extracted patterns, 56 reference repos](Ecosystem/extracted-patterns.md) | reference |
| 2023-04-12 | [task-34 extraction: phase e (WASM runtime) + phase f (competitors)](Ecosystem/extraction-fase-ef.md) | reference |

## Generative

| Date | Document | Status |
|---|---|---|
| 2026-09-23 | [program: from GPU UI engine to generative interaction runtime](Generative/generative-runtime-program.md) | active |
| 2026-09-23 | [plev: thesis of the generative interaction runtime](Generative/generative-runtime-thesis.md) | active |

## Importing

| Date | Document | Status |
|---|---|---|
| 2025-03-26 | [monster format v0 specification](Importing/monster-format-v0.md) | reference |
| 2026-06-12 | [mon experiment](Importing/mon-experiment-notes.md) | reference |

## Interface

| Date | Document | Status |
|---|---|---|
| 2021-05-27 | [input system design (task-09)](Interface/input-system-design.md) | reference |
| 2021-08-30 | [mobile specifics - safe areas, IME, lifecycle](Interface/mobile-specifics.md) | reference |
| 2022-06-15 | [reference analysis: accessibility (accesskit)](Interface/accessibility.md) | reference |
| 2022-06-15 | [reference analysis: tui patterns](Interface/tui-patterns.md) | reference |
| 2022-10-27 | [touch & gesture recognition design](Interface/touch-gesture-design.md) | reference |
| 2023-04-12 | [pattern extraction: tui apps (yazi, television, bottom)](Interface/pattern-extraction-tui-apps.md) | reference |

## Rendering

| Date | Document | Status |
|---|---|---|
| 2021-04-15 | [layout engine - decisions and knowledge](Rendering/layout-engine.md) | reference |
| 2021-06-16 | [layer system - technical decisions](Rendering/layer-system.md) | reference |
| 2021-07-04 | [effects architecture (task-08)](Rendering/effects-architecture.md) | reference |
| 2021-09-17 | [WASM/webgpu validation - knowledge](Rendering/wasm-webgpu-validation.md) | reference |
| 2022-02-24 | [animation pattern, frame-based exponential lerp](Rendering/animation-pattern-lerp.md) | reference |
| 2022-06-15 | [reference analysis: animation and motion](Rendering/animation-motion.md) | reference |
| 2022-06-15 | [reference analysis: charts and visualization](Rendering/charts-visualization.md) | reference |
| 2022-06-15 | [reference analysis: math, physics and geometry](Rendering/math-physics-geometry.md) | reference |
| 2023-04-12 | [pattern extraction: parley, lyon, glam-rs](Rendering/pattern-extraction-parley-lyon-glam.md) | reference |
| 2023-06-20 | [parley vs cosmic-text - factual comparison for plev](Rendering/parley-vs-cosmic-text.md) | reference |
| 2026-06-10 | [hoff visual tokens (measured)](Rendering/hoff-visual-tokens.md) | reference |

## Structure

| Date | Document | Status |
|---|---|---|
| 2021-03-06 | [view trait - design decisions](Structure/view-trait-design.md) | reference |
| 2021-03-24 | [component state design (task-02)](Structure/component-design.md) | reference |
| 2021-05-02 | [signal system design](Structure/signal-system-design.md) | reference |
| 2021-07-26 | [DSL narrate, design decisions](Structure/dsl-narrate-design.md) | reference |
| 2023-07-05 | [WASM plugin architecture - research doc](Structure/wasm-plugin-architecture.md) | reference |
| 2024-05-23 | [SRP modularization with a 300-line limit](Structure/srp-modularization.md) | reference |
| 2026-09-18 | [ui component catalog](Structure/component-catalog.md) | active |
| undated | [plev — Technical Implementation Plan](Structure/technical-plan.md) | reference |

## Workflows

| Date | Document | Status |
|---|---|---|
| 2021-08-15 | [ios build - decisions and knowledge](Workflows/ios-build.md) | reference |
| 2022-01-19 | [android emulator deploy](Workflows/android-emulator-deploy.md) | reference |
| 2024-01-22 | [hot reload design - plev](Workflows/hot-reload-design.md) | reference |
| 2024-07-09 | [resolving 107 clippy warnings to zero](Workflows/clippy-zero-warnings.md) | reference |
| 2026-09-23 | [operating mode: autonomous orchestrator](Workflows/autonomy.md) | active |
| 2026-09-23 | [how to build and serve the web target](Workflows/build-and-serve-the-web-target.md) | active |
| 2026-09-23 | [how to code against the plev engine](Workflows/code-against-the-plev-engine.md) | active |
| 2026-09-23 | [how to validate visuals by pixel](Workflows/validate-visuals-by-pixel.md) | active |
| 2026-09-23 | [autonomous cycle journal](Workflows/autonomous-cycle-journal.md) | reference |
| 2026-09-26 | [how to orchestrate coding agents on this repo](Workflows/orchestrate-coding-agents-on-this-repo.md) | active |
| 2026-09-29 | [orchestrator and agent guidelines](Workflows/generative-runtime-guidelines.md) | active |
| 2026-10-27 | [how to run the tools](Workflows/run-the-tools.md) | active |
| 2026-10-31 | [Conventions of the docs archive](Workflows/docs-conventions.md) | active |

[plev docs](../README.md)
