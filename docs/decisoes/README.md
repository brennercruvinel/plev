---
id: indice-decisoes
title: "Decisões"
project: plev
type: indice
status: ativo
date: null
date_basis: gerado
topics: [navegacao]
relations: []
source: tool/tasks/docbase.py
source_revision: gerado
revised: gerado
review: gerado
---

# Decisões

Os ADRs: contexto, decisão, consequências e alternativas, numerados numa sequência só.

## estrutura

| Data | Documento | Estado |
|---|---|---|
| 2024-05-21 | [modularizacao por single responsibility principle](estrutura/0015-srp-modularization.md) | aceito |
| 2025-08-18 | [engine at root, libraries and apps in crates, demos in examples](estrutura/0013-workspace-engine-at-root-libs-in-crates-demos-in-examples.md) | aceito |
| 2026-04-01 | [git backend: gix for reads, git CLI for mutations, a worker thread for the UI](estrutura/0020-git-backend-gix-reads-cli-mutations.md) | aceito |
| 2026-07-13 | [one knowledge tree, and a minimal root](estrutura/0022-one-knowledge-tree-and-a-minimal-root.md) | aceito |
| 2026-08-30 | [official app pattern: state in structs, retained widgets, explicit invalidation](estrutura/0023-official-app-pattern.md) | aceito |
| 2026-09-18 | [comps: one design system crate, every value a token](estrutura/0027-comps-one-design-system-crate.md) | aceito |
| 2026-09-21 | [showcase: one page per component, the sidebar as a folding tree](estrutura/0030-showcase-one-page-per-component.md) | aceito |
| 2026-09-24 | [the silences app is the fixed screen the generated one is compared against](estrutura/0053-the-silences-app-is-the-fixed-screen.md) | aceito |
| 2026-09-24 | [a window is a host of what dies with it; the app owns the state](estrutura/0056-windows-are-hosts-the-app-owns-the-state.md) | aceito |
| 2026-09-26 | [local llm inference in process: candle or burn](estrutura/0072-ai-native-single-process.md) | aceito |
| 2026-10-09 | [O workspace separa bibliotecas, apps, demos e ferramentas](estrutura/0080-workspace-por-responsabilidade.md) | aceito |
| 2026-10-27 | [Um grafo separa dados, aparência e layout](estrutura/0081-grafo-dados-aparencia-layout.md) | aceito |

## geracao

| Data | Documento | Estado |
|---|---|---|
| 2026-09-24 | [the plev IR is typed, references only declared capabilities, and runs only once verified](geracao/0049-plev-ir-typed-and-verified.md) | aceito |
| 2026-09-24 | [a surface materializes only from verified IR, projects from the IR and asks through the app's dispatcher](geracao/0051-materialized-surfaces-from-verified-ir.md) | aceito |
| 2026-09-24 | [proposers propose, the runtime verifies](geracao/0052-proposers-propose-the-runtime-verifies.md) | aceito |
| 2026-09-24 | [adaptive transformations are small verified operations that preserve identity](geracao/0067-adaptive-transforms-preserve-identity.md) | aceito |
| 2026-09-25 | [each demo's method is written before it runs, and the result keeps its negatives](geracao/0070-generative-runtime-results.md) | proposto |

## gravacao

| Data | Documento | Estado |
|---|---|---|
| 2026-09-23 | [session recording: replay, not re-execution](gravacao/0037-session-recording-replay-not-reexecution.md) | aceito |
| 2026-09-23 | [the inspector reads a session, it never runs it](gravacao/0046-the-inspector-reads-a-session-never-runs-it.md) | aceito |
| 2026-09-24 | [sessions become regression tests](gravacao/0050-sessions-become-regression-tests.md) | aceito |
| 2026-09-24 | [the scene travels as data](gravacao/0065-scene-travels-as-data.md) | aceito |

## importacao

| Data | Documento | Estado |
|---|---|---|
| 2025-05-14 | [the monster animation format: keyframes plus discovered deltas](importacao/0010-binary-animation-format-with-discovered-deltas.md) | aceito |
| 2025-06-10 | [import foreign formats by conversion, never by embedding a foreign runtime](importacao/0011-import-foreign-formats-by-conversion-not-embedding.md) | aceito |
| 2025-07-08 | [the transpiler reports every unmapped construct, never drops silently](importacao/0012-transpiler-reports-every-unmapped-construct.md) | aceito |
| 2026-09-18 | [urnaui: decode media frames in place with ffmpeg's subfile protocol](importacao/0025-urnaui-media-frames-in-place-via-ffmpeg-subfile.md) | aceito |
| 2026-09-24 | [ui asset compiler: an asset leaves the runtime on a measured number](importacao/0066-ui-asset-compiler.md) | aceito |
| 2026-09-25 | [importers target plev IR, with a report in four categories](importacao/0069-importers-target-plev-ir.md) | aceito |

## interface

| Data | Documento | Estado |
|---|---|---|
| 2025-01-27 | [touch input synthesizes pointer events into the existing mouse path](interface/0009-touch-as-synthesized-pointer-events.md) | aceito |
| 2026-03-04 | [the text editing core follows helix: document is rope plus selections plus history](interface/0021-text-editing-core-follows-helix.md) | aceito |
| 2026-09-23 | [accessibility projects the semantic snapshot](interface/0038-accessibility-projects-the-semantic-snapshot.md) | aceito |
| 2026-09-24 | [file dialogs and file drops are capabilities of the engine, and a native dialog opens only for a person](interface/0055-file-dialogs-and-drops-are-capabilities.md) | aceito |
| 2026-09-24 | [a native menu is another view of the app's capabilities](interface/0057-native-menus-are-another-view-of-capabilities.md) | aceito |
| 2026-09-24 | [a native dialog answers the same contract: the alert is a person's capability, the confirmation is the same confirm or decline](interface/0058-native-dialogs-answer-the-same-contract.md) | aceito |
| 2026-09-24 | [the tray and the notifications are capabilities out of the window](interface/0059-tray-and-notifications-are-capabilities-out-of-the-window.md) | aceito |
| 2026-09-24 | [capture is a job the person turns on](interface/0062-capture-is-a-job-the-person-turns-on.md) | aceito |
| 2026-09-24 | [a link is not a person](interface/0063-a-link-is-not-a-person.md) | aceito |
| 2026-09-25 | [accessibility is production-ready per platform](interface/0068-accessibility-is-production-ready-per-platform.md) | proposto |
| 2026-09-25 | [text moves by screen order and shapes by line](interface/0075-text-moves-by-screen-order-and-shapes-by-line.md) | aceito |
| 2026-09-25 | [the keyboard walks the projection](interface/0076-the-keyboard-walks-the-projection.md) | aceito |
| 2026-09-26 | [the person's composition and clipboard stay the person's](interface/0077-the-persons-text-and-clipboard-stay-the-persons.md) | aceito |

## processos

| Data | Documento | Estado |
|---|---|---|
| 2024-02-06 | [adotamos shader hot reload via notify file watcher sobre dylib reloading](processos/0019-shader-hot-reload.md) | aceito |
| 2024-02-18 | [adotamos runtime parser + override map para hot reload de plev_narrate! DSL](processos/0018-narrate-hot-reload.md) | aceito |
| 2024-04-14 | [adotamos pleverror como tipo de erro unificado e eliminamos debt de lint](processos/0017-error-handling-lint-cleanup.md) | aceito |
| 2026-09-22 | [hot reload on by default: shaders and literals in place, code by rebuild](processos/0031-hot-reload-on-by-default-in-gpu-context.md) | aceito |
| 2026-09-23 | [regression by meaning before pixels](processos/0045-regression-by-meaning-before-pixels.md) | aceito |
| 2026-09-23 | [A integração na main é um pull request fundido por merge commit](processos/0085-integracao-por-merge-commit.md) | aceito |
| 2026-09-24 | [view source for pixels: a control leads to its projection, a text to its literal](processos/0048-view-source-for-pixels.md) | aceito |
| 2026-09-24 | [what identical means](processos/0064-what-identical-means.md) | aceito |
| 2026-10-27 | [A união de features reduz compilações duplicadas](processos/0083-unificacao-de-features.md) | aceito |
| 2026-10-27 | [Cada alvo web tem um comando e uma faixa de portas](processos/0084-alvos-web-e-portas.md) | aceito |

## renderizacao

| Data | Documento | Estado |
|---|---|---|
| 2024-06-17 | [adotamos projecao ortografica em logical pixels para apps hidpi](renderizacao/0016-hidpi-projection.md) | aceito |
| 2024-09-18 | [container geometry derives from available space, never from constants](renderizacao/0002-content-driven-layout-not-fixed-constants.md) | aceito |
| 2024-10-07 | [render on demand makes invalidation a correctness contract](renderizacao/0008-render-on-demand-requires-explicit-invalidation.md) | aceito |
| 2024-10-29 | [one TextStyle per text run, shared by measurement and drawing](renderizacao/0006-one-text-style-for-measurement-and-drawing.md) | aceito |
| 2024-11-20 | [linearize colors before they reach the GPU](renderizacao/0004-linearize-colors-before-the-gpu.md) | aceito |
| 2024-12-04 | [render into an sRGB view when the surface format cannot be sRGB](renderizacao/0007-render-into-an-srgb-view-format.md) | aceito |
| 2024-12-19 | [design tokens come from measurement of the live reference, not from eyes or static CSS](renderizacao/0005-measured-design-tokens-over-eyeballed-values.md) | aceito |
| 2025-01-08 | [embed every font weight the UI uses, and pin the default families](renderizacao/0003-embed-every-font-weight-in-use.md) | aceito |
| 2025-02-10 | [async GPU initialization and one wasm entry point per module](renderizacao/0001-async-gpu-init-and-single-wasm-entry.md) | aceito |
| 2025-09-24 | [motion trails come from position history, not framebuffer accumulation](renderizacao/0014-motion-trails-by-position-history.md) | aceito |
| 2026-08-31 | [glyph raster identity and atlas isolation](renderizacao/0024-glyph-raster-identity-and-atlas-isolation.md) | aceito |
| 2026-09-18 | [background results wake the event loop; child pipes drain from the start](renderizacao/0026-worker-results-wake-the-event-loop.md) | aceito |
| 2026-09-21 | [core theme: the tailwind class lists as button recipes](renderizacao/0028-core-theme-tailwind-classes-as-recipes.md) | aceito |
| 2026-09-21 | [embed an app view by giving it a frame origin](renderizacao/0029-embed-an-app-view-by-frame-origin.md) | aceito |
| 2026-09-23 | [headless render shares the frame path](renderizacao/0034-headless-render-shares-the-frame-path.md) | aceito |
| 2026-09-23 | [the svg export reads the scene, not the pixels](renderizacao/0041-svg-export-reads-the-scene.md) | aceito |
| 2026-09-23 | [a tick asks for a deadline, not for every frame](renderizacao/0042-a-tick-asks-for-a-deadline.md) | aceito |
| 2026-09-24 | [a lost GPU device is recovered in place, and the next frame draws the same](renderizacao/0054-gpu-device-loss-recovers-in-place.md) | aceito |
| 2026-09-24 | [a covered window does not draw, and wakes with a new frame](renderizacao/0060-a-covered-window-does-not-draw-and-wakes-new.md) | aceito |
| 2026-09-24 | [printing is the frame on paper](renderizacao/0061-printing-is-the-frame-on-paper.md) | aceito |
| 2026-09-25 | [embed Noto Sans subsets for the non-Latin scripts](renderizacao/0071-embedded-fonts-by-script.md) | aceito |
| 2026-09-26 | [a renderer upgrade is proven by a full capture](renderizacao/0074-a-renderer-upgrade-is-proven-by-a-full-capture.md) | aceito |
| 2026-09-29 | [design for ranges, not screens and devices](renderizacao/0073-design-for-ranges-not-screens-and-devices.md) | aceito |
| 2026-10-27 | [the web build fetches the Noto faces when a text first needs them](renderizacao/0078-the-web-fetches-the-noto-faces-on-demand.md) | aceito |
| 2026-10-27 | [a desktop draws the scripts with the embedded faces, not the system's](renderizacao/0079-a-desktop-draws-the-scripts-with-the-embedded-faces.md) | aceito |
| 2026-10-27 | [Uma view hospedada usa a janela e o relógio da engine](renderizacao/0082-view-hospedada-sob-demanda.md) | aceito |

## semantica

| Data | Documento | Estado |
|---|---|---|
| 2026-09-23 | [semantic contract as a pure projection of app state](semantica/0032-semantic-contract-as-pure-projection.md) | aceito |
| 2026-09-23 | [actions carry the state their actor observed](semantica/0033-actions-carry-observed-state.md) | aceito |
| 2026-09-23 | [semantic binding beside the scene, and the click as a request](semantica/0035-semantic-binding-beside-the-scene.md) | aceito |
| 2026-09-23 | [a commit checks the index it observed](semantica/0036-commit-checks-the-index-it-observed.md) | aceito |
| 2026-09-23 | [agents act through the semantic contract](semantica/0039-agents-act-through-the-semantic-contract.md) | aceito |
| 2026-09-23 | [the semantic form is lean, and one rule redacts a secret](semantica/0040-lean-semantic-form-and-one-redaction.md) | aceito |
| 2026-09-23 | [a delta carries the node it adds, and a stale refusal names who acted](semantica/0043-a-delta-carries-the-node-a-refusal-names-who-acted.md) | aceito |
| 2026-09-23 | [a terminal stage carries the revision the app is at](semantica/0044-a-terminal-stage-carries-the-revision.md) | aceito |
| 2026-09-23 | [an act rests on what its actor was shown](semantica/0047-an-act-rests-on-what-its-actor-was-shown.md) | aceito |

[Acervo plev](../README.md)
