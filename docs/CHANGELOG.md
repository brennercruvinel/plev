---
id: changelog
title: "plev changelog"
project: plev
type: record
status: active
date: null
date_basis: undated
recorded: 2026-06-11
topics: [process, workspace, rendering, agents]
relations: []
evidence: [176dd691, 704412f1, b1e820ba]
source: [docs/log/changelog.md, docs/log/README.md, docs/log/responsiveness-multiplatform-and-fidelity.md, reconstruction]
source_revision: ec0a224c
revised: 2026-10-31
review: reviewed
---

# plev changelog

What changed in plev since 2021, newest first, one section per date. A date gathers the entries written when the change landed, in the words of their time, and the records of that day: the tasks, the ADRs, the experiments, the research and the learnings, each linked. The project state and task map of 2026-09-26 and the responsiveness session of 2025-02-19 sit under their dates. How far each date can be trusted is the next section.

## Reading the dates

Each page carries three dates in its header. `date` is when what it records
happened, `recorded` is the day its text was written, and `revised` is its last
editorial revision; `date_basis` says where `date` came from. This file orders
the records by `date`.

### The history before June 2026

The history of plev up to 2026 lived in a Forgejo instance on an old VPS, and it
was lost there; it could not be recovered. The git history of this repository
starts on 10 June 2026 (`176dd691`). Between March and June 2026 the lost work was
written down again, and commit `704412f1` (2026-07-13) gave those records the
dates of the work itself, from 2021 on: the note on `View`, written on
2026-03-08, carries 2021-03-06, and the [layout ADR](adr/rendering/0002-content-driven-layout-not-fixed-constants.md),
written on 2026-06-10, carries 2024-09-18. Those pages have
`date_basis: reconstructed`, and `recorded` keeps the day they were written.

A record written in 2026 about earlier work speaks with what existed when it was
written: a machine, a toolchain, a later version of a library. `recorded` says
from when such a detail comes.

### The reconstruction kept in step

Where one record placed as done what another dated later, the dates were brought
in line with what the records say:

- the reports of the research briefing (task-ref) take the briefing's date;
- the extraction of task-34 and its reports come after the animation system
  (task-27) whose `Spring` and `Tween` they analyse, and after the demo app
  (task-29);
- the text assessment (task-32, with the parley vs cosmic-text comparison)
  comes after accessibility (task-30), which it treats as present, and the
  plugin research (task-33) after tasks 30 and 31, which it says just landed;
- the v0.2 benchmarks take the day of their measurement, since they measure
  the crates of tasks 43 to 46 with Rust 1.94.

A task's `date` is the day it closed; the sessions written inside it carry the
days the work was done, a few days before.

### Remapped commits

Commit `b1e820ba` remapped hash references after the history purge. For example,
`34a9893` was replaced by `55c8aa7` in the [ADR on asynchronous GPU initialization](adr/rendering/0001-async-gpu-init-and-single-wasm-entry.md). An old reference may have a recoverable match even if
the whole earlier history no longer exists.

### Recovered material

The texts on gamma, on apps bypassing the engine and on the HOFF tokens, and the first technical plan, were deleted from the tree at some point and recovered from git history. Their recovered versions are historical sources; the exact paths and revisions are in their `source` field.

## 2026-10-31

- plev is MIT (2026-10-31, task-41): the license is decided as MIT alone,
  not the MIT OR Apache-2.0 task-41 planned. the root `LICENSE` holds the
  text and every crate declares `license = "MIT"`, inherited from
  `[workspace.package]`.

- the graphs read ELK too (2026-10-31, PR #146, task-100, adr
  a-graph-separates-data-appearance-and-layout): the ELK reader and the
  `elk` and `elk-groups` pages of the graphs demo reach main, which closes
  task-100 and settles the decision that a graph separates data,
  appearance and layout.

## 2026-10-27

- a desktop draws the scripts with the embedded faces (2026-10-27, adr
  a-desktop-draws-the-scripts-with-the-embedded-faces): the engine's own
  fallback names the Noto face of each script before the platform's, so
  arabic, hebrew, devanagari, CJK and emoji no longer come from Geeza Pro,
  Hiragino or Apple Color Emoji, and the Typography page is equal in every
  byte between the desktop and the web.

- the web build fetches the Noto faces when a text first needs them
  (2026-10-27, issue #137, adr the-web-fetches-the-noto-faces-on-demand):
  the wasm embeds the latin faces only, each script's Noto files come from
  `fonts/` beside the page, checked by sha256, and the measurer and the
  renderer take them in the desktop's order and draw the text again. the
  uigallery's wasm went from 8.2 MB to 2.85 MB with gzip; the Typography
  page shows one line per embedded script; `experiment-no-noto` is gone.

- every executable of the repository runs (2026-10-27, task-103):
  `tool/tasks/runcheck.sh` starts each app, demo, crate command line and
  tool and checks that it does its work, which the gate does not prove: a
  window passes when it stays alive and its capture is not a single color,
  a command line when it exits 0 or prints the expected line. a program
  without its input (`llm-bench` without a model, `urnadbapp` without the
  `../urna` checkout) is reported as such instead of failing.

- one feature set for every build, and the web targets by one command
  (2026-10-27, task-102, adr feature-unification, adr
  web-targets-and-ports): `rust/unions`, kept by cargo hakari, asks for the
  union of the features the members use, so `-p arcade`, `-p uigallery`
  and `-p commit` recompile zero crates the workspace already built (54,
  65 and 7 before). dev builds keep line tables only: `cargo test -p engine
  --no-run` on a clean target went from 96 s and 3.8 GB to 85 s and 2.1 GB.
  `tool/tasks/webtarget.sh` serves uigallery, urnadbapp, graphs and primes
  from ports 8080 to 8083, or the next free one.

- graphs drawn live from d3 and Cytoscape data (2026-10-27, task-100, adr
  a-graph-separates-data-appearance-and-layout): `Graph` holds nodes, edges
  and groups, `Look` the appearance and `Layout` the arrangement (force or
  compound), and the graphs demo draws the gramorph corpus on the desktop
  and the web. the ELK reader and its pages wait on
  `feat/elk-ir-from-gramorph`.

- [task-100: interactive graphs and corpus import](tasks/task-100-interactive-graphs/README.md)
- [task-101: on-demand fonts and deterministic fallback](tasks/task-101-fonts-by-script/README.md)
- [task-102: reusable builds and web commands](tasks/task-102-builds-and-web-commands/README.md)
- [task-103: runcheck, every executable in the repository runs](tasks/task-103-runcheck-every-executable-runs/README.md)
- ADR 0078: [the web build fetches the Noto faces when a text first needs them](adr/rendering/0078-the-web-fetches-the-noto-faces-on-demand.md)
- ADR 0079: [a desktop draws the scripts with the embedded faces, not the system's](adr/rendering/0079-a-desktop-draws-the-scripts-with-the-embedded-faces.md)
- ADR 0082: [a hosted view uses the engine's window and clock](adr/rendering/0082-a-hosted-view-renders-on-demand.md)
- ADR 0081: [a graph separates data, appearance and layout](adr/structure/0081-a-graph-separates-data-appearance-and-layout.md)
- ADR 0083: [feature unification reduces duplicate compilations](adr/workflows/0083-feature-unification.md)
- ADR 0084: [each web target has one command and a port range](adr/workflows/0084-web-targets-and-ports.md)
- learning: [Separate data, appearance and algorithm in Rust](learnings/data-appearance-and-algorithm.md)

## 2026-10-16

- the explorer follows urna's new layout, and the platforms catch up
  (2026-10-16, task-99): the query embedder finds urna's scripts in the
  repository or where `urna setup` installs them, the shared executor
  drains stdout and stderr from the start so a large corpus no longer hangs
  it, `rfd` uses the xdg portal on Linux, the background import is
  macOS-only, and a reduced-motion preference macOS cannot find reads as
  never set.

- [task-99: explorer integration and platform fixes](tasks/task-99-integration-and-portability/README.md)
- learning: [Why waiting for a process can block reading its output](learnings/subprocesses-and-pipes.md)

## 2026-10-09

- the repository in four homes (2026-10-09, task-98, issue #130, PR #131,
  adr workspace-by-responsibility): libraries in `rust/`, apps in `apps/`,
  demos in `demo/`, tools in `tool/`. the 1,861 moves sit alone in one
  commit (`d3e65544`, every one R100), and the manifests, paths, tests and
  docs follow in their own commits. the names old and new are in
  `docs/TERMS.toml`.

- [task-98: workspace reorganization](tasks/task-98-workspace-reorganization/README.md)
- ADR 0080: [the workspace separates libraries, apps, demos and tools](adr/structure/0080-workspace-by-responsibility.md)

## 2026-09-29

- ADR 0073: [design for ranges, not screens and devices](adr/rendering/0073-design-for-ranges-not-screens-and-devices.md)

## 2026-09-26

- the repository catches up with itself (2026-09-26): four decision
  records for what the last rounds settled (the person's composition and
  clipboard stay the person's, a renderer upgrade is proven by a full
  capture, the keyboard walks the projection, text moves by screen order
  and shapes by line), the orchestration and pixel how-tos with the
  machine limits and the capture method, and a learnings file for the
  session. the changelogs of closed tasks sit with the closed tasks, the
  2022 competitor list moved to `docs/ref/`, a Figma file is never
  committed (`*.fig`), and the zed tasks and the pull request template run
  `./tool/tasks/fullcheck.sh`.

- plev on wgpu 29, and burn on the renderer's gpu (2026-09-26, task-78):
  the whole workspace moved from wgpu 28 to 29, and every gallery page in
  4 themes at 4 scales (1312 frames) draws the same bytes as before. a
  covered window on macOS now skips its frame and redraws when it shows
  again. the engine hands its own gpu device to a library that computes
  on it: with `rust/llmrun`'s `plev` feature, burn runs on that device,
  a matrix product on it matches the cpu, and TinyLlama answers there.
  the workspace now needs rust 1.87.

- a local language model in process (2026-09-26, task-78): `rust/llmrun`
  streams an answer token by token from a model on disk (TinyLlama 1.1B
  for now), with candle or burn behind one interface. on an M4's gpu,
  candle does 37 tokens a second and burn 26, and both give the same
  answer. both stay; plev moves to wgpu 29 next so burn can share the
  renderer's gpu.

### plev, project state and task map (2026-09-26)

o plev é uma engine de composição GPU-first em rust que está virando um
runtime de interação generativa: cada controle é também dado (um contrato
semântico), que a pessoa, o leitor de tela e um agente operam pelo mesmo
dispatcher. o que é, para onde vai e o estado de cada task ficam no
programa ([`generative-runtime-program.md`](research/generative-runtime-program.md), seção 12); a tese está em
[`generative-runtime-thesis.md`](research/generative-runtime-thesis.md), as regras dos agentes em
[`generative-runtime-guidelines.md`](ref/operations/generative-runtime-guidelines.md) e o modo de operação do
orquestrador em [`autonomy.md`](ref/operations/autonomy.md). na época, cada task tinha o seu arquivo em
`steps/`, e o estado dela ficava no `status:` do cabeçalho (`in-progress`,
`pending`, `done`); as pastas `ongoing/`, `pending/` e `checked/` citadas
no estado abaixo e nos registros antigos marcavam esse mesmo estado.

#### estado atual (2026-09-26)

- workspace de 23 crates membros, mais dois workspaces próprios
  (`apps/urnadbapp`, o explorador de `.urna`, e `rust/llmrun`, a inferência
  em processo); wgpu 29, rust 1.87.
- alvos: macOS e web entregues; android e iOS rodam a galeria (tasks 18,
  19 e 20 fechadas em 2026-09-25); linux e windows pendentes.
- abertas em `steps/ongoing/`: task-73 (dependências incrementais, o ADR
  de desenho espera a escolha do brenner), task-74 (texto: falta o roteiro
  de IME num campo), task-75 (accessibility: o roteiro do VoiceOver, as
  cores que falham no contraste e o web), task-78 (AI-native: o app de
  referência) e task-80 (a prova: a demo 2 e o checkpoint c).
- pendente em `steps/pending/`: task-41 (licença e release pública).
- o repositório no GitHub é privado; a licença MIT está em
  `LICENSE`.

#### mapa de tasks

##### fase 0, fundamental (p0), completa
| task | descricao | status |
|------|-----------|--------|
| task-01 | view trait + viewcontext | done |
| task-02 | component state + lifecycle | done |
| task-03 | layout engine (taffy 0.9) | done |
| task-04 | signal system (reactive) | done |
| task-05 | declarative builder API + #[component] | done |
| task-06 | WASM/webgpu validation | done |
| task-07 | layer system + composite pass | done |
| task-08 | effects (blur, shadow) | done |
| task-09 | input (keyboard, mouse, scroll) | done |
| task-10 | touch + gesture recognition | done |
| task-11 | android build + lifecycle | done |
| task-12 | ios build + lifecycle | done |
| task-13 | mobile specifics (safe areas, IME) | done |
| task-14 | plev_narrate! DSL verbal | done |

##### fase 1, integracao end-to-end (p0), completa
| task | descricao | status |
|------|-----------|--------|
| task-15 | pipeline DSL -> builder -> compositor | done |
| task-16 | effects <-> layer integration | done |
| task-17 | input <-> layer hit-testing | done |

##### fase 2, mobile/WASM testing (p1), parcial
| task | descricao | status |
|------|-----------|--------|
| task-18 | android build & device test | concluída (2026-09-25): APK no emulador, toque e lifecycle testados; arquivo em `checked/` |
| task-19 | ios simulator test | concluída (2026-09-25): o frame do simulador igual ao desktop fora da barra de status; arquivo em `checked/` |
| task-20 | WASM visual validation | concluída (2026-09-25): 12 capturas web iguais ao desktop em todo byte; arquivo em `checked/` |

##### fase 3, polish (p2), parcial
| task | descricao | status |
|------|-----------|--------|
| task-21 | documentacao DSL narrate | done (falta JSX comparison) |
| task-22 | error messages & DX | parcial (did you mean feito) |
| task-23 | CI/CD (github actions) | done (nao testado remotamente) |
| task-24 | cleanup & readme | parcial (readme feito, license TBD) |
| task-30 | accessibility (accesskit) | done (feature-gated, lazy activation, focusgraph, 9 testes) |

##### fase 4, proof of life (p2), completa
| task | descricao | status |
|------|-----------|--------|
| task-27 | animation system (easing, tweens, springs) | done (35 testes, ~500 LOC) |
| task-28 | editable text (cursor, input, IME) | done (44 testes, ~800 LOC) |
| task-29 | todo app demo (proof of life) | done (~530 LOC, funcional) |
| task-31 | vector paths (lyon tessellation) | done (~250 LOC, reusa quad pipeline, 13 testes) |
| task-32 | text upgrade assessment (parley vs cosmic-text) | done (research: wait, nao migrar agora) |

##### fase 4b, quick wins & bug fixes (p0/p1), completa (derivada de task-34)
| task | descricao | status |
|------|-----------|--------|
| task-35 | fix spring<t>, analytical solver (bug frame-rate) | done, solver analitico 3 regimes, frame-rate independent |
| task-36 | signal hardening (RAII guard, fxindexset, peek) | done, fxindexset o(1), observerguard panic-safe, peek() (f3 sentinel skipped) |
| task-37 | animation enhancements (keyframeseq, repeat, step/hold) | done, keyframesequence, repeat/reverse/delay, step/hold, const-generic |
| task-38 | event batching | done, bufferedevent enum, batch-drain before render |


##### fase 6, extensibilidade (p4), parcial
| task | descricao | status |
|------|-----------|--------|
| task-33 | WASM plugin architecture | done (fase 1 research: wait, extism quando API estavel) |
| task-34 | exploracao/extracao de ideias (56 repos) | done, 38 patterns de 18 repos, top 10 priorizado |

##### fase 7, release publica (p0), pendente
| task | descricao | status |
|------|-----------|--------|
| task-39 | scene cache memoization (b7 + b8) | pendente |
| task-41 | license + cargo.toml + release prep | pendente |

##### experiment, gitbutler port (proof of real app)
| task | descricao | status |
|------|-----------|--------|
| task-42 | gitbutler plev port (tauri/svelte -> rust GPU-native) | dispatch tipado + overlay foram para a engine; o app evoluiu para o crate `apps/idestudio` (ver task-48). resta polir context menu/modal render |

##### research
| task | descricao | status |
|------|-----------|--------|
| task-ref | research briefing (50+ repos) | done, 12 docs em refs/, technology radar |

##### fase 8, formatos e interop (2025), completa
| task | descricao | status |
|------|-----------|--------|
| task-43 | monster, codec binario de animacao v0 | done, MON0, seek O(1), delta descoberto, 124 testes |
| task-44 | lot, importer lottie + ponte para .monster | done, conversao offline, subset honesto, zero embedding |
| task-45 | parser, transpiler poc (react/gpui -> builder) | done, droplist file:line, contagens congeladas |

##### fase 9, edicao e apps reais (2026), completa
| task | descricao | status |
|------|-----------|--------|
| task-46 | rope, nucleo de edicao (document/transaction/history) | done, estilo helix, 77 testes, headless |
| task-47 | git, backend (gix reads + cli mutations + client threaded) | done, ui nunca bloqueia, 25 testes |
| task-48 | ide, git client nativo em plev | done, workspace/diff/commit, 55 testes (evolui de task-42) |

##### fase 10, demos e plataforma (2025), completa
| task | descricao | status |
|------|-----------|--------|
| task-49 | prime number creatures (port entropic life) | done, sim core puro, render fiel, desktop + web |
| task-50 | showcase multiplataforma como lib | done, entries desktop/web/android/ios, apk + simulador |

##### fase 11, organizacao profissional (2024/2026), completa
| task | descricao | status |
|------|-----------|--------|
| task-51 | workspace restructure + rebrand phi->plev | done, 3 tiers, renames, cargo workspace |
| task-52 | modularizacao SRP (limite 300 linhas) | done, 44 monolitos divididos, API inalterada, 470 testes |

---

- [task-78: AI-native applications](tasks/task-78-ai-native-apps/README.md)
- ADR 0077: [the person's composition and clipboard stay the person's](adr/interface/0077-the-persons-text-and-clipboard-stay-the-persons.md)
- ADR 0074: [a renderer upgrade is proven by a full capture](adr/rendering/0074-a-renderer-upgrade-is-proven-by-a-full-capture.md)
- ADR 0072: [local llm inference in process: candle or burn](adr/structure/0072-ai-native-single-process.md)
- experiment: [candle against burn for local LLM inference in process, TinyLlama 1.1B](experiments/local-inference-candle-vs-burn/README.md)
- learning: [Generative runtime session: what worked and what bit us](learnings/generative-runtime.md)

## 2026-09-25

- copy, cut, paste and select all in text fields (2026-09-25, task-74):
  cmd+c, x, v and a (ctrl elsewhere) work in the gallery's fields and in
  the ide's intent line and commit form; a key with cmd or ctrl never
  types its letter any more (cmd+v used to type "v"). a right-click on
  a field of the gallery's Input page opens a menu with the same four.
  the clipboard is the person's: an agent can neither read it nor pick
  those items.

- Japanese and Chinese input in text fields (2026-09-25, task-74): a
  field draws what the input method is composing, underlined, and the
  candidate window sits at the caret; the gallery turns the input method
  on only while a field or the editor has the focus. what is being
  composed is never in the field's value, the recording or what an
  agent reads.

- shift with the arrows selects in a text field (2026-09-25, task-74):
  in the gallery, the ide's intent line and urnaui, shift+left, right,
  home and end extend the selection, as they already did in the editor.
  the fields knew how; the apps never sent the shift.

- the editor holds a million lines (2026-09-25, task-74): it opens one
  in about 21 ms, scrolls and edits in the middle in under half a
  millisecond per frame, the same as with ten thousand lines; clicks near
  the end of a huge file land on the right line again.

- emoji in color (2026-09-25, task-74): Noto Color Emoji (640 emoji,
  with skin tones and families, 1.4 MB) is built in, and the glyph atlas
  draws a color glyph with its own colors. emoji used to be empty boxes on
  the web and phones, and invisible on the desktop.

- the keyboard on the gallery's component pages (2026-09-25, task-75):
  tab and shift-tab walk every control in the order a screen reader
  walks, with a focus ring; the arrows move in tabs, trees, menus, selects
  and sliders; enter and space press; a modal keeps the focus inside and
  gives it back when it closes. tables and lists take the focus one
  row at a time, and the arrows walk the rows.

- edit a text in the running gallery and it lands in the source
  (2026-09-25, task-65): in a debug build, cmd+shift+click on a text that
  one string literal holds opens a field over it; enter writes the new
  string into the `.rs` file, and `git diff` shows only that string. it
  refuses when the file's code changed since the build, and an agent can
  never do it.

- the caret and the selection follow mixed text (2026-09-25, task-74):
  in a field or the editor holding hebrew or arabic next to latin, the
  arrows move across the screen, the caret sits where the next letter will
  appear, and a selection that crosses directions highlights exactly the
  text it holds, in one or more pieces.

- arabic, hebrew, devanagari, chinese, japanese and korean text draws on
  the web and on phones (2026-09-25, task-74): Noto Sans subsets, regular
  and bold, are now built in (from 1.7 MB to 6.9 MB of fonts); these
  scripts used to show empty boxes there. the desktop keeps using the
  system's fonts for them. emoji comes later.

- the parser imports HTML/CSS and Figma (2026-09-25, task-71): a page
  (a subset: flex, color, radius, type, buttons and fields) and a Figma
  screen become plev IR with the same four-way report as react and gpui.
  `tool/tasks/figmapull.py` reads a `.fig` file with no network and no API
  token and writes one screen as the Figma API would.

- a window for a recorded session (2026-09-25, task-64):
  `plev-inspector <session.monster>` shows the timeline with a scrub, the
  frame that was on screen at that instant with every control outlined,
  and, for the control clicked, what it was, its state, who observed what,
  the request, the verdict with what changed, and the result. it only
  reads the file: nothing in it can send a request.

- every page compared on the web and on iOS (2026-09-25, task-77): the
  web build opens any page from its address (`?page=…&theme=…`), and the
  capture scripts take `--all`. of 82 pages, 79 match the desktop byte for
  byte on the web and 78 on the iPhone simulator differ only under the
  status bar; the rest show another thing on purpose (no Claude, no git
  there) or a blinking caret. the table demo drew other ids on the web
  (a 32-bit overflow), now fixed.

- the gallery runs on android (2026-09-25, task-18): a tap clicks and a
  drag scrolls (touch did nothing before), and coming back from the home
  screen no longer kills the app; rotation and a low-memory warning are
  fine. the frame matches the desktop within 2/255 on the emulator's
  software GPU.

- the gallery runs on the iPhone simulator (2026-09-25, task-19): it was
  laid out as a 1200x800 desktop window, cropped on the right; now it
  takes the phone's screen and matches the desktop frame pixel for pixel,
  except the status bar iOS draws on top. it does not yet keep out of the
  status bar.

- the gallery runs on the web (2026-09-25, task-20): the web build
  failed to start and then drew an empty page; now every page the
  keyboard reaches, in every theme, at 1x, 2x and 3x, matches the desktop
  frame byte for byte in Chrome with WebGPU, and scrolling holds 120 fps.

- the mirror answers, and a secret stays home (2026-09-25, task-79 phase
  2): a click in the mirror window goes back to the app as an action on
  the control under it, through the same contract an agent uses, never as
  a raw event; the app treats the mirror as an agent, so an action kept
  for a person is refused and says so, and a confirmation is given by the
  person at the app. what a field marked secret draws never reaches the
  mirror: a box that says "secret, not mirrored" takes its place, and
  typing in it sends nothing.

- the component pages speak to a screen reader and to agents (2026-09-25,
  task-75 phase 2b): 30 pages of the gallery (buttons, checkboxes,
  switches, chips, sliders, fields, the select, tabs, the tree, the table,
  the long list, the breadcrumb, the navigation links, the split, the
  dialog and toast pages) now say what they show. a person's click, a
  screen reader's press and an agent's act through the port reach the
  same state on each of them. the menu bar leaves them out. the other 46
  pages are listed with why (24 need their widget to learn to describe
  itself, 22 show nothing to operate).

- a screen reader reaches what the mouse reaches on the pages that
  project (2026-09-25, task-75 phase 2a): inside each row of the todo a
  reader finds a "completed" checkbox and a "remove" button, inside each
  cut of the Silences "preview segment", "apply cut" and "reject cut";
  the proposer opens and is set by its value; the focus a reader moves to
  a tab, a button or the proposer is where the page says it is; a status
  says its message as its name, so Windows and Linux readers announce it
  too; the toasts, the dialog and the context menu are part of what the
  page shows an agent and a reader, and their buttons run through the
  same dispatcher as a click; spinners and skeletons stand still with
  "reduce motion"; and a person with a screen reader hears which
  accessibility preference is on, which never leaves through the port.

- the ide's intent line moves under the panels (2026-09-25, task-96
  phase b): with no generated surface open the ide's panels have the
  whole width again, and the line sits in a strip at the bottom; the
  surface opens in a column beside the panels only while it is open. the
  diff card of the review is titled "diff" instead of the commit's id.

- code and diffs in a generated surface (2026-09-25, task-97): a
  generated surface can now show code, a line per row, and a diff with
  each line tinted by its kind (added in green, removed in red, the hunk
  header in grey) using colors the theme already had; every kind reads
  at WCAG AA in the four themes. the ide's review of three commits shows
  its diff this way: 14, 7 and 14 lines in view instead of 5, 5 and 5
  without color. against the fixed panel it still loses 4 lines of one
  commit (the surface's height is bounded by the one height token the IR
  has) and the line numbers.

- a generated surface in the ide (2026-09-25, task-96): beside its
  panels, the ide has a line where a person writes what they want to see.
  "review these three commits" (or "revise estes três commits") gives a
  verified surface with one card per commit (its message, author, files
  and a button that shows its diff) and the diff beside them. every
  click on it is a request through the ide's own dispatcher, so an agent
  can press the same button and a stale one is refused as anywhere else;
  closing it leaves nothing behind. the rules make it; the Claude can be
  chosen and is shown only the commits the review names. measured against
  the fixed panel for the same review: better at seeing the files of all
  three commits at once, worse at reading a diff, which the surface shows
  five lines at a time without color.

- the proof starts (2026-09-25, task-80 phase 1): the method of the five
  demonstrations is written down before any new one ran, each with its
  script, the alternative it is compared against and what would knock it
  down. two ran now. a real old bug (a git status read landing after the
  change it predates, fixed in task-84) was recorded as a bug report,
  read in the inspector at the exact instant, and turned into a test in
  the gate that fails with the bug and passes without it. the review of
  the silences was recorded being born from an intent, turned into a
  comparison and closed, with nothing of it left after; the recording
  shows it was never a screen of the app. a generated surface inside the
  ide (for the fifth demonstration) is a new task.

- accessibility preferences change a generated surface (2026-09-25,
  task-70 phase 4): on the Silences page, a person writes
  "accessibility" in the row and a band opens with "larger text", "more
  contrast", "denser" and "sparser". each one changes every generated
  surface, the open one and every one that opens after, by the same small
  verified steps as any other change, within the theme's tokens. with
  more contrast, the secondary text of the review goes from 4.13 to over
  18 against its background. turning one off undoes only what it
  changed, and what someone else asked meanwhile stays. only the person
  can turn them on; an agent is refused and never learns which are on.
  with the system's reduce motion on, a generated surface now changes at
  once too.

- the parser writes plev IR (2026-09-25, task-71 phase 1): next to the
  Rust builder code it writes today (unchanged, byte for byte), the parser
  now turns a React component or a gpui widget into plev IR, the same
  form a generated surface is made of. every construct of the source is
  reported, with its file and line, as supported, transformed (and by
  which rule), approximated (and by how much) or impossible (and why). a
  color or a spacing goes to the nearest theme token when it is within a
  tolerance measured from the theme itself. a button, a click handler or
  a text field becomes a control whose action the app has to declare; the
  importer never invents one. the IR and the builder code are drawn side
  by side and every pixel of difference is explained.

- a screen reader hears what changes (2026-09-25, task-75 phase 1): a
  status line and a toast are announced without taking the focus, an
  error louder than the rest; seven buttons a reader could reach and not
  press (propose, detect silences, open file) now press; an empty group
  no longer stops the reader. with the system's "reduce motion" on
  (macOS), every transition ends on its first frame. the contrast of the
  four themes is measured against WCAG AA: 102 of 301 pairs fail and are
  listed, no color changed yet. what AccessKit gives each platform is
  written down from the locked source: macOS today; Windows and Linux
  when their target runs; no adapter yet on iOS and the web.

- [task-18: android build & device test](tasks/task-18-android-device-test/README.md)
- [task-19: ios simulator test](tasks/task-19-ios-simulator-test/README.md)
- [task-20: WASM visual validation](tasks/task-20-wasm-visual-validation/README.md)
- [task-39: scene cache, memoization + dirty flag bubbling](tasks/task-39-scene-cache-memoization/README.md)
- [task-42: plev event dispatch + gitbutler UI port](tasks/task-42-plev-event-dispatch/README.md)
- [task-64: session inspector, bug report and session for testing](tasks/task-64-session-inspector-and-tests/README.md)
- [task-65: pixel to code and development inspector](tasks/task-65-pixel-to-source/README.md)
- [task-70: adaptive transformations](tasks/task-70-adaptive-transformations/README.md)
- [task-71: plev as compiler target (importers to IR)](tasks/task-71-compiler-target/README.md)
- [task-72: UI asset compiler](tasks/task-72-ui-asset-compiler/README.md)
- [task-74: text and editing](tasks/task-74-text-and-editing/README.md)
- [task-75: production-ready platform accessibility](tasks/task-75-platform-accessibility/README.md)
- [task-77: visual determinism](tasks/task-77-visual-determinism/README.md)
- [task-79: scene over socket](tasks/task-79-scene-over-socket/README.md)
- [task-80: proof of qualitatively different software](tasks/task-80-proof-qualitatively-different-software/README.md)
- [task-96: a generated surface in the ide](tasks/task-96-ide-generated-surface/README.md)
- [task-97: code and diff in the IR vocabulary](tasks/task-97-code-and-diff-in-the-ir/README.md)
- ADR 0070: [generative runtime results](adr/generated/0070-generative-runtime-results.md)
- ADR 0069: [importers target plev IR, with a report in four categories](adr/importing/0069-importers-target-plev-ir.md)
- ADR 0068: [accessibility is production-ready per platform](adr/interface/0068-accessibility-is-production-ready-per-platform.md)
- ADR 0075: [text moves by screen order and shapes by line](adr/interface/0075-text-moves-by-screen-order-and-shapes-by-line.md)
- ADR 0076: [the keyboard walks the projection](adr/interface/0076-the-keyboard-walks-the-projection.md)
- ADR 0071: [embed Noto Sans subsets for the non-Latin scripts](adr/rendering/0071-embedded-fonts-by-script.md)
- experiment: [five demos of qualitatively different software, each with its method written before it ran](experiments/five-demos/README.md)

## 2026-09-24

- a sequence you repeat becomes one action (2026-09-24, task-70 phase
  3): the Silences page notices when a person does the same actions in
  the same order on each cut ("preview segment, then apply cut") and
  offers to make them one action. only a person can accept; an agent is
  refused. once accepted, the action exists on every cut, and on the
  generated review a button runs it: each step is a request of its own,
  checked against the state the step before left, and if one no longer
  holds the sequence stops and says which step and why. nothing is
  undone and nothing is kept past the session.

- a change asked in words (2026-09-24, task-70 phase 2): with a
  generated surface open, the same propose of the row reads the intent as
  a change of it first. "too much information", "I can't compare these",
  "bigger numbers" and "undo" (and "tem informação demais", "não consigo
  comparar", "números maiores", "desfazer") become the operations of
  phase 1, by the rules at once or by the Claude off the app's thread,
  each verified again and applied through the app's dispatcher as
  whoever asked; an agent asks through the port the same way. the review
  by the rules now shows each cut's frame, its numbers named and its
  decision as a badge, and the prompt tells the model that reviewing
  items means seeing what each one is.

- a generated surface changes by small steps (2026-09-24, task-70 phase
  1): nine operations change a surface that is open without building it
  again: hide, show, group, reorder, emphasize, density, restyle, swap a
  view and promote an action. each result passes the verifier again, each
  comes with the way back to the surface before, byte for byte, and no
  node leaves without an explicit hide. the host keeps every change with
  who asked and undoes the last one; what a widget holds (a drag, a
  caret, a press on a cut) stays. the regress shows the Claude's
  first-round review with the cuts moved first and the repeated
  detection hidden.

- a generated surface is usable at the default window (2026-09-24,
  task-69 checkpoint b fixes): the checkpoint b found the review generated
  by the rules broken at 1200x800 and the Claude's with unlabeled sliders
  and raw numbers. a text in a generated surface is never narrower than
  its longest word, a row that does not fit wraps, a slider shows its
  label and value, and a number shows with at most two decimals. the
  prompt now asks for the task first, a label on every control and each
  control once. the regress gained the Silences page with the review
  open, by the rules and from a fixture of the Claude's proposal (six
  baselines).

- the asset compiler measures first (2026-09-24, task-72 phase 1): the
  work the app does at startup was measured before moving anything. the
  first headless frame takes 79 ms in debug: 26 ms opening the GPU and
  about 23 ms scanning the system's 805 fonts, twice. shaders take 2.7
  ms and icons 0.17 ms, so the icons stay in runtime. every WGSL shader
  is now validated in the gate, and an error names the file, the line
  and the column.

- the scene travels as data (2026-09-24, task-79 phase 1): a window's
  scene can be watched over the agent's local socket, layer by dirty
  layer, and the `mirror` example draws it in another process with the
  same pixels and the control bindings on top. a hover costs about 300
  bytes and an unchanged frame costs nothing. the channel exists only
  when the person launches the app with `PLEV_MIRROR=1`; a live image
  such as the camera never crosses it, and every watcher is logged.
  the engine's image store gained read-only access to an image's pixels.

- the blur follows the scale (2026-09-24, task-77 phase 1b): the glass
  blur, the layer blur and the layer shadow now reach as far in logical
  pixels at every display scale, so the same glass looks the same on a
  retina screen. scale 1 is unchanged byte for byte: it keeps the same
  shader entry and weights. other scales lay the 1x kernel over the
  physical texels. on a page with glass, a 2x frame costs about 1.55
  times what it did.

- what identical means (2026-09-24, task-77 phase 1): the regression
  tool gains `regress compare`, which says whether two frames are equal,
  different only where a product decision says so, or different with
  nothing to explain it (with the number of pixels and the color delta),
  and `regress capture`, which draws every page against itself. the
  desktop is deterministic: 1312 of 1312 frames (82 pages, 4 themes,
  scales 1, 1.25, 1.5 and 2) are equal in every byte across two GPU
  contexts and two processes. one real divergence was found: the glass
  blur does not follow the display scale. capture scripts for web, iOS
  and android are ready and wait for tools that need the network.

- a link is not a person (2026-09-24, task-76 item 6): the showcase
  follows `plev-showcase://<capability>/<action>?value=<text>` links read
  by a strict grammar. a link always acts as an outside agent named
  `link`, never as the person, and only on the capabilities the app
  opens to links (switching the page and the theme). anything else,
  such as the camera, the file dialog or printing, is refused, and the
  refusal shows on screen. for now a link arrives as a launch argument
  on every desktop; receiving it from the system on macOS waits for the
  app bundle. this closes task-76, platform integration.

- capture is a job the person turns on (2026-09-24, task-76 item 10):
  a new Capture page shows the camera and a microphone level meter.
  only a person can turn either on; an agent reads the state and is
  refused. capture runs ffmpeg as a child process, with no new
  dependency. when the window is covered, the app sleeps or the page
  is not shown, the device is released at once and comes back only if
  the person had it on. the engine gained a stream image, one slot
  whose pixels change in place, so frames do not fill the image atlas.
  no camera or microphone was opened on this machine.

- printing is the frame on paper (2026-09-24, task-76 item 9): a person
  prints the page with cmd-p or the menu. the document is the same frame
  the window draws, rendered without a window at 300 dpi, so the text is
  sharp and not a scaled bitmap. the engine talks to the system's print
  queue through `lp` and `lpstat` as child processes with a deadline,
  behind a trait with a fake for tests. the job runs beside the loop,
  and an agent reads its state but cannot print. this machine has no
  printer, so nothing was printed for real.

- a covered window does not draw, and wakes new (2026-09-24, task-76 item
  7): a minimized or fully covered window, or a sleeping app, asks for no
  frame and keeps no deadline; when it shows or wakes, the first frame is
  the state now, byte for byte, never the frame from before. background
  work knows the app sleeps: a proposal asked of the Claude while asleep
  is held and starts on the first poll awake, results wait in their
  channel, and nothing is lost. the clocks jump on wake. the system's
  own sleep notification is left for the platform bring-up.

- tray and notifications are capabilities out of the window (2026-09-24,
  task-76 item 5): the engine puts a tray icon with a menu read from the
  projection, and a system notification with at most one action a person
  may press, behind traits with fakes; a click on either comes back as a
  request of a person through the app's dispatcher. the showcase has a
  tray with the shell's menu and a `show` capability, the Files page
  notifies when files are dropped while the window is behind, and the end
  of a silence detection notifies through the dispatcher. on macOS a real
  notification needs an `.app` bundle, so the desktop answers "needs a
  bundle" until the platform bring-up; the tray works today.

- native dialogs answer the same contract (2026-09-24, task-76 item 8):
  `engine::platform::alerts` puts the system's alert and confirmation
  behind a trait, with a fake for tests and none on wasm and mobile. in
  the ide, `PLEV_NATIVE_CONFIRM=1` asks the contract's human confirmation
  in the system's dialog instead of the drawn modal, with the same words,
  answered through the same confirm and decline of a person and recorded
  the same way; without support it falls back to the modal. the Files
  page shows an alert as a capability only a person may ask. no
  dependency was added.

- native menus are another view of capabilities (2026-09-24, task-76 item
  4): the engine reads a menu model from an app's projection (every item
  is a `SemId` and an action it already declares, with its shortcut),
  `muda` mounts it on the desktop, and a click becomes a request through
  the app's dispatcher as a person, with the same checks as the button.
  the showcase shell is projected now (`shell.page`, `shell.theme`,
  `shell.detach`, `shell.save_session`), its keys go through the
  dispatcher, and its menu bar carries the theme, the pages, detach, save
  and the active page's plain actions. no menu on wasm or mobile.

- windows are hosts, the app owns the state (2026-09-24, task-76 item 2):
  the engine groups what belongs to a window (the window, its GPU
  context, compositor, input, IME, scale, clocks) in a `WindowHost` and
  keeps several in `Windows`, routed by id, each drawn only when it asks
  (render on demand per window), closed one at a time, with the app
  exiting on the last. the `layers` example opens two windows; in the
  showcase, `D` outside a text field opens the current page in a second
  window over the same state and dispatcher, and the projection says
  which window shows it. a lost device in one window recovers alone.

- file dialogs and file drops belong to the engine (2026-09-24, task-76
  item 3): `engine::platform::files` puts the native dialog (open, open
  many, save, pick a folder) behind a trait with a fake for tests and a
  no-dialog answer on wasm and mobile; a file dragged over or dropped on
  a window is an engine input event with the paths and, when the platform
  says it, the position. the urnaui opens and exports through the engine
  and no longer depends on `rfd` itself. the new Files page in the
  showcase shows both as capabilities: `open_file` only a person may ask,
  `drop_files` anyone, and the files received are projected state.

- a lost GPU device recovers in place (2026-09-24, task-76 phase 1): every
  device, windowed or headless, installs wgpu's device-lost callback;
  before a frame the engine polls it and, when the device is gone,
  recreates the adapter, device, queue, surface configuration, pipelines,
  projection, the glyph atlas, the effect processor, the texture pool and
  the compositor's GPU resources, then draws the same frame. images live
  in an append-only log with a cursor per atlas, so nothing is re-sent.
  without an adapter the frame is skipped with one error in the log and
  the app stays alive. proven headless by forcing the loss: the frame
  after it is byte for byte the frame before.

- the text coverage audit (2026-09-24, task-74 phase 1): a corpus with one
  phrase per script (latin with diacritics, ligatures and kerning,
  combining marks, arabic, hebrew, devanagari, chinese, japanese, korean,
  emoji with ZWJ, mixed bidi) shaped and rendered headless, with the
  glyphs, the notdefs, the measured width and the ink per line in
  `rust/engine/tests/fixtures/text_audit.txt`. with the embedded fonts every
  non-latin script is tofu; bidi order, combining marks and kerning work;
  no embedded face has `liga`; colour emoji has width and no ink. the
  probe reports shaping per glyph (`engine::text::probe::shape_report`).

- the showcase records its session, and the video editor demo is recorded
  (2026-09-24, task-69 phase 4c): with `PLEV_RECORD=1` in a debug build
  the gallery records what the ide records (every stage of every receipt,
  what the port showed each agent, the projected state, every frame with
  where each control shows, clicks and keys by kind, never a character)
  and saves on cmd-shift-s, on close and on a panic. `cargo run -p
  showcase --example demo` drives the video editor's script through the
  agent port without a window: detect, the intent, the generated review,
  a cut applied on it, the comparison, close. two recorded demos ship as
  fixtures (by the rules, and by the Claude), read by a test and by
  `plev-inspect`.

- the generated silences screen, with real frames (2026-09-24, task-69
  phase 4b): a surface's `image` bound to a frame ref the app projects
  (`silences.video@4.0`) draws the frame the app decoded, through the new
  `surface::Frames` trait; the Silences page has the intent row of the
  Surface page, a generated review beside the fixed screen (the video, a
  thumbnail per cut, apply and reject, the counts, detect and apply all)
  on the same dispatcher, and a close. the rules find the media by
  structure and "compare the original with the cuts" turns the review into
  a side-by-side comparison keeping every continuing id. the Claude
  produced both screens from a sentence (138 s and 40 s).

- the silences app, the fixed screen of the video editor (2026-09-24,
  task-69 phase 4a): a new Silences page in the showcase reviews the
  silences of a video with real capabilities: detect them with ffmpeg's
  `silencedetect`, change the threshold and the minimum duration, see a
  frame of each stretch, apply or reject each cut, apply all. ffmpeg runs
  as a child in a worker, one job at a time, with a deadline and a kill by
  pid, and the request ends in the wake. the test video is generated by
  ffmpeg itself. the page projects every capability, so a person, an
  agent through the port and a screen reader operate it by the same
  path; it is the fixed screen checkpoint b compares the generated one
  against. detection takes 28 ms, a frame 30 ms.

- the Surface page asks the Claude for the screen (2026-09-24, task-69
  phase 3b-ii): a person, or an agent through the port, writes an intent,
  picks the proposer (`rules` or `claude`; `claude` only where the program
  exists, and only on desktop) and presses propose. the Claude is asked
  off the app's thread: the request stays open (`Execution::Started`), the
  page keeps answering, the status line shows the attempt, the seconds
  and the cost, and a cancel button kills the child. the verified surface
  is born or changes form in place when the answer arrives. a real intent
  through the port ("the items as a list with the add field on top") took
  9.3 s and 0.04 USD.

- the Claude proposes surfaces (2026-09-24, task-69 phase 3b-i): the new
  `claude-model` crate puts the Claude behind `propose::Model` by running
  the `claude -p` of Claude Code as a child process, with no tools, no
  saved session, no MCP, no user settings, a budget cap and a timeout that
  kills the child. the network is the child's; the crate holds no key.
  `cargo run -p intent -- --app todo "<intent>"`
  turns an intent into a verified IR over the todo's or the ide's
  projection. four real intents verified on the first attempt, in 4.5 s to
  89 s. the Surface page does not use it yet (phase 3b-ii).

- the generated todo shows its items (2026-09-24, task-95): the todo
  projects `todo:items`, the ids of the visible items in the order shown,
  and each item's `text`. both forms of the Surface page repeat over that
  list, one text and one "done" checkbox per item, and checking an item on
  the generated surface runs the todo's `toggle` through its dispatcher.
  the rules proposer understands "items" and finds the list by structure,
  the same way in the todo and in the ide (`changes:files`). removal stays
  on the fixed todo: a destructive action without confirmation can only
  be bound through a dialog.

- a surface proposed from an intent (2026-09-24, task-69 phase 3a): the
  new `propose` crate turns an intent in text, the app's capability
  registry, its snapshot and the surface it shows into a proposed plev IR,
  and `propose_verified` lets only a verified IR out. `Rules` proposes
  without a model, from a small grammar (add, filter, counts, compact, two
  columns, everything) bound by the registry's structure, never by an
  app's names; the same rules read the todo and the ide's workbench. `Llm`
  asks a language model behind the `Model` trait and sends the verifier's
  errors back, three attempts at most; no provider is wired yet. the
  Surface page has an intent field and a propose button, projected
  capabilities an agent uses through the port too, and the surface changes
  in place, keeping what was typed. from intent to verified IR takes about
  0.2 ms in a debug build.

- a refusal that names a person points at the question (2026-09-24,
  task-94): when `plev act` is refused as stale and a person acted since
  the agent looked, `plev` writes one line on stderr naming the person's
  request and pointing at `--ask`; none after the person declines, when
  only agents acted, or when the act already asked. stdout and the
  protocol do not change. with it, agents that were never told about
  `--ask` asked the person (task-62, fifth round).

- a surface changes form without losing what it is (2026-09-24, task-69
  phase 2): an open surface takes a new verified IR and reconciles by
  identity. a control that keeps its `SemId` and widget keeps its focus,
  caret, selection and a drag in progress; a new one is born, a gone one
  leaves the projection at once and fades out, a moved one glides, with
  the theme's durations and the overlay's spring, and reduced motion
  makes it instant. a change of form draws 37 frames and then none. the
  Surface page switches the generated todo between a compact and a
  two-column form, and what was typed and the filter stay.

- the laboratory's person answers the agent (2026-09-24, task-93):
  `ide-lab --answer allow|decline|none --answer-after <ms>` has the scripted
  person answer, in the modal, every confirmation an agent asks for, and
  the report tells each question, the answer, how long it took and what
  the agent did next, and counts the acts that went over the person
  without asking. `--scripted ask` is an agent that asks before putting
  back what the person took out.

- the agent asks the person, and every stage reaches the agent
  (2026-09-24, task-92). a bug first (P0): a stage of an agent's request
  that ended outside a drain of the port (a person declining in the ide's
  confirmation modal, a second check refusing) never reached the agent
  until it sent another message, and was lost when `plev act` gave up.
  the port's sink now wakes the port when it hears one. then the handoff:
  an agent can ask a person to confirm an act first, with a short reason
  (`plev act ... --ask "the person took src/session.rs out; put it
  back?"`). the ide shows "Question from claude" with Allow and Cancel;
  allowed, the act runs after the second check; declined, the agent gets
  `declined`. only an agent's question waits; the text is bounded,
  redacted on a secret target, and the inspector's why shows the
  question and who answered.

- a surface materialized from verified IR (2026-09-24, task-69 phase 1,
  adr materialized-surfaces-from-verified-ir): a new crate, `surface`,
  turns a `VerifiedIr` into a live interface: the comps widgets, laid out
  by the engine's layout, styled by the theme's tokens, semantic by
  construction (the projection comes from the IR), and every click or key
  becomes a request through the dispatcher of the app that declares the
  capability. nothing else materializes: a raw `Ir` does not compile
  there. the showcase's new Surface page shows the todo written by hand
  beside the same capabilities materialized from `showcase-todo.ir.json`;
  adding an item or switching the filter on either changes both, and an
  agent operates the surface through the port like any page. from
  verified IR to the first frame's scene: under a millisecond (debug).
  the showcase's agent module left `dispatch.rs`, which is no longer
  frozen.

- a recorded session becomes a regression test (2026-09-24, task-64
  phase c, adr sessions-become-regression-tests): a stretch of a session
  (`plev-inspect <session> --export <t0> <t1> <out>`) and a small text
  file of marked values become a test. a harness reruns every recorded
  request, as recorded, against a fixture of the app in a temp dir, and
  compares each verdict and terminal stage and the marked values. the
  first one is the third round's a2ppa session (an LLM agent, a person
  unstaging a file right before its submit): with the commit's
  precondition broken on purpose it fails, naming request 7, recorded
  as stale and now accepted. the inspector itself still runs nothing.

- the plev IR and its verifier (2026-09-24, task-68, adr
  plev-ir-typed-and-verified): a new pure crate, `uir`, describes an
  interface surface by meaning, as JSON a person, an LLM or a tool can
  write: identity, a component from a closed vocabulary that mirrors the
  widgets, props only as theme tokens, layout by primitives, and state
  and actions bound to what the app already declares (the registry comes
  from the app's projection). `verify` is the only way to a `VerifiedIr`,
  and it answers every error at once with its JSON pointer and rule: a raw
  color, a pixel size, an action the app does not declare, a destructive
  action on a plain button, a control with no label. a JSON schema for
  constrained decoding is generated from the same tables. three surfaces
  verify against real projections: the ide's commit form, a generated
  "review what goes into the commit", and the todo.

- a stale refusal names who acted since the agent looked (2026-09-24,
  task-91): the port marks each observation it serves in the dispatcher's
  order (`sem::Mark`, through its receipt sink), and a refused act of the
  port names every request taken after the oldest observation its reads
  come from, the agent's own included. the ide's ordinary scene (a person
  clicks a row, then unstages the file) now names the unstage, and an
  agent refused by its own write reads "claude itself edited the commit
  message". the laboratory tells each act's form from the port's count
  (`Traffic.acts_from_revision`), not from its revision.

- a pixel leads to the code that made it (2026-09-24, task-65 phases a
  and b, adr view-source-for-pixels): in a desktop debug build, cmd+click
  on a control of the showcase opens the line that projected that
  capability (`Node.source`), and cmd+click on a text with no control
  opens the string literal that holds it (the hot reload baseline now
  knows each literal's line and column). several literals are listed,
  never guessed; a formatted text says so. the editor is `$PLEV_EDITOR`,
  else zed, else `code -g`. `PLEV_BACKGROUND=1` opens the showcase window
  without taking the focus on macOS. none of it exists in release, wasm
  or mobile.

- the laboratory measures what the agent rested on (2026-09-24,
  task-88): the stale measurement of `ide-lab` found the act by its
  revision, and a revision is a hash of what the app shows, so a world put
  back as it was counted as the old one (a false "valid"). now a run is
  valid when a submit after the person's act carried the staged files of
  before it while the app held others. a new scenario,
  `--human unstage-before-submit:<path>`, has the person act between the
  agent's decision and its submit, as the git arm's hook does; the old one
  stays for comparison. `--scripted chain` is an agent that chains on
  stage revisions without observing; the report counts the protocol's
  refusals (`not_observed`, `unknown_revision`).

- [task-63: session flight recorder](tasks/task-63-flight-recorder/README.md)
- [task-69: ephemeral and generated surfaces](tasks/task-69-ephemeral-generative-surfaces/README.md)
- [task-76: platform integration](tasks/task-76-platform-integration/README.md)
- [task-88: the lab measures what the agent rested on](tasks/task-88-the-lab-measures-what-the-agent-rested-on/README.md)
- [task-91: the dispatcher knows when the agent looked](tasks/task-91-the-dispatcher-knows-when-the-agent-looked/README.md)
- [task-92: the agent asks the person](tasks/task-92-the-agent-asks-the-person/README.md)
- [task-93: the person in the lab answers](tasks/task-93-the-lab-person-answers/README.md)
- [task-94: the refusal that names a person points to the question](tasks/task-94-the-refusal-points-at-the-question/README.md)
- [task-95: the todo projects its items as a list](tasks/task-95-the-todo-projects-its-items-as-a-list/README.md)
- ADR 0049: [the plev IR is typed, references only declared capabilities, and runs only once verified](adr/generated/0049-plev-ir-typed-and-verified.md)
- ADR 0051: [a surface materializes only from verified IR, projects from the IR and asks through the app's dispatcher](adr/generated/0051-materialized-surfaces-from-verified-ir.md)
- ADR 0052: [proposers propose, the runtime verifies](adr/generated/0052-proposers-propose-the-runtime-verifies.md)
- ADR 0067: [adaptive transformations are small verified operations that preserve identity](adr/generated/0067-adaptive-transforms-preserve-identity.md)
- ADR 0066: [ui asset compiler: an asset leaves the runtime on a measured number](adr/importing/0066-ui-asset-compiler.md)
- ADR 0055: [file dialogs and file drops are capabilities of the engine, and a native dialog opens only for a person](adr/interface/0055-file-dialogs-and-drops-are-capabilities.md)
- ADR 0057: [a native menu is another view of the app's capabilities](adr/interface/0057-native-menus-are-another-view-of-capabilities.md)
- ADR 0058: [a native dialog answers the same contract: the alert is a person's capability, the confirmation is the same confirm or decline](adr/interface/0058-native-dialogs-answer-the-same-contract.md)
- ADR 0059: [the tray and the notifications are capabilities out of the window](adr/interface/0059-tray-and-notifications-are-capabilities-out-of-the-window.md)
- ADR 0062: [capture is a job the person turns on](adr/interface/0062-capture-is-a-job-the-person-turns-on.md)
- ADR 0063: [a link is not a person](adr/interface/0063-a-link-is-not-a-person.md)
- ADR 0050: [sessions become regression tests](adr/recording/0050-sessions-become-regression-tests.md)
- ADR 0065: [the scene travels as data](adr/recording/0065-scene-travels-as-data.md)
- ADR 0054: [a lost GPU device is recovered in place, and the next frame draws the same](adr/rendering/0054-gpu-device-loss-recovers-in-place.md)
- ADR 0060: [a covered window does not draw, and wakes with a new frame](adr/rendering/0060-a-covered-window-does-not-draw-and-wakes-new.md)
- ADR 0061: [printing is the frame on paper](adr/rendering/0061-printing-is-the-frame-on-paper.md)
- ADR 0053: [the silences app is the fixed screen the generated one is compared against](adr/structure/0053-the-silences-app-is-the-fixed-screen.md)
- ADR 0056: [a window is a host of what dies with it; the app owns the state](adr/structure/0056-windows-are-hosts-the-app-owns-the-state.md)
- ADR 0048: [view source for pixels: a control leads to its projection, a text to its literal](adr/workflows/0048-view-source-for-pixels.md)
- ADR 0064: [what identical means](adr/workflows/0064-what-identical-means.md)
- experiment: [generated silences screen against the fixed screen, checkpoint b of task-69](experiments/generated-surface-checkpoint-b/README.md)
- experiment: [text coverage per writing system, and the cost of large documents in the editor](experiments/text-coverage-and-large-documents/README.md)
- experiment: [desktop gallery pages: same pixels across scales, themes and runs](experiments/visual-determinism-captures/README.md)

## 2026-09-23

- the ide records its own session, and a frame says where each control
  shows (2026-09-23, task-63 phase d): with `PLEV_RECORD=1` in a debug
  build the ide keeps a flight recorder on the workbench's clock (every
  stage of every receipt, who observed what through the port, the state
  after each change, every frame, a person's clicks and keys by kind and
  control, never the characters) and saves it to
  `$TMPDIR/plev/sessions/ide-<pid>-<n>.monster` on cmd-shift-s, on close
  and on a panic; `plev-inspect` reads it. `rec` records, per frame, the
  visible rect of each bound control (section B, session version 1;
  version 0 files still read), so a replay can say which control sat
  under a point. `PLEV_BACKGROUND=1` on macOS opens the ide's window
  without taking the focus, for runs someone watches.

- a stale refusal names the chain since the state the agent saw
  (2026-09-23, task-90): `acted` named only the other actors' actions
  checked on the exact revision the agent saw; a person who unstaged two
  files in a row was named for the first only. now it names every action
  of other actors that ran from the first request that ran on that state
  on, in order, and the ide's toast says it in one sentence per actor
  ("the person unstaged B, opened the diff of C and unstaged C"). a
  change no request made still names nobody: from outside the app, and in
  the ide a selection, typing or a toast leaving, which move the state
  outside the dispatcher (task-91 anchors on the agent's look instead).

- an act rests only on what its agent was shown (2026-09-23, task-89,
  adr an-act-rests-on-what-its-actor-was-shown): the session inspector
  found that `act --from-revision` built the values an agent "saw" from
  the whole snapshot of the revision, including state outside the scope
  and keys it observed; an agent that looked only at the commit form after
  a person unstaged a file would have committed two files believing
  three. the port now remembers, per actor (the name of `hello`, over
  every connection under it), what each observation showed, and each
  read takes the value from the last observation up to that revision
  that showed it; a read never shown is refused before the dispatcher
  with the new error `not_observed` (additive in v1, `plev` exits 3). a
  terminal stage's revision shows no state, so an act chained on it reads
  what the agent observed before. reproduced on the laboratory: the
  submit after a form-only look is refused as stale, with the file the
  person took out.

- the ide a person uses opens the agent port (2026-09-23, task-86): with
  `PLEV_AGENT=1` in a debug build the `ide` window serves its workbench to
  agents as app `ide` (`ide::agent`, a `wire::Host` over the `Workbench`
  and the frame's visible bindings), the same dispatcher a click goes
  through. the shell serves the port when it wakes and again after every
  git result, so a stage or a commit that ends on the worker sends its
  last stage then, and it draws only when an act ran. proved in a real
  window: an agent staged two files and wrote the message, a person
  unstaged one with real clicks, the agent's submit was refused as stale
  naming the person, the toast on screen and the node the agent reads said
  the same words, and the agent committed what the person left staged.
  the engine gains `Binding::visible_rect` and
  `Compositor::visible_bindings`, which the showcase's port now reads too.

- a recorded session answers why (2026-09-23, task-64 phase a, adr
  the-inspector-reads-a-session-never-runs-it): the new crate `inspect`
  and its cli `plev-inspect <session.monster>` read a session and answer,
  for any request or control, the questions of the thesis: which control,
  what state, who observed what and when, which request with which
  precondition, what moved in what it read between the observation and
  the request and which receipt ran then (a correlation in time, said as
  such), the verdict with its delta, the result. it never runs anything.
  it opened all nine sessions of the two checkpoint a rounds.

- a regression says which control changed before it counts pixels
  (2026-09-23, task-67, adr regression-by-meaning-before-pixels):
  `cargo run -p regress -- check` stages every
  baselined page, compares its meaning (`Snapshot::diff`), then a stable
  hash per layer and the bindings, and only then the pixels in the region
  of a layer that moved, naming the controls there. nothing changed:
  157 baselines in about 0.6 s, no frame drawn, no GPU opened. `bless`
  accepts a change and prints what it accepts. a CI job runs the check
  on macOS.

- the ide and urnaui sleep when nothing changes (2026-09-23, task-82
  phase b, adr a-tick-asks-for-a-deadline): the ide's workbench, the
  showcase's Git page and urnaui answer their tick with a `Wake`, and the
  shells wait for the next deadline instead of drawing every frame. a
  focused ide commit field went from every frame (720 in 6 s) to 11; a
  toast at rest is a deadline. the idle ide drew forever in an ordinary
  repository (`git status` took the optional index lock, the file watcher
  saw it and refreshed): every git process now runs with
  `GIT_OPTIONAL_LOCKS=0` and the watcher ignores `.git/index.lock`, and
  the idle ide draws 0 frames and runs 0 git processes. each shell
  catches time up as the loop wakes, so an animation started after a
  long idle runs from its start instead of jumping to its end.

- an agent chains its acts without observing only to learn the revision
  (2026-09-23, task-87, adr a-terminal-stage-carries-the-revision): a
  terminal stage (`completed`, `rejected`, `failed`) carries `revision`,
  the revision the app is at when it goes out, held in the port's ring
  so `--since` and `--from-revision` take it. in the ide laboratory the
  scripted agent went from 24 to 17 messages and from 91.5 to 45.9 KB
  with a person's intervention (16 to 12 messages, 54.5 to 28.8 KB
  without); the projection costs about 0.16 ms per terminal stage in
  debug. `plev --help` also says that `--keys` cuts state only.

- the commit a receipt announces is already on screen (2026-09-23,
  task-85): when the ide's commit receipt completes, the next observation
  shows the commit on top of `stacks` and its files out of the staged
  list, before the refresh reads git; `stacks.commit[<sha>]` carries
  `files` on the ten newest commits and on the last one the ide made. a
  commit's files have one definition everywhere (paths changed against
  the first parent, a rename at both paths), read in process with gix
  and cached by sha: `log(100)` went from about 3 to 23 ms cold and stays
  about 3 ms warm; a whole ide observation of this repository grew 3.8%.

- a delta says what was born, and a refusal says who acted (2026-09-23,
  task-83, adr a-delta-carries-the-node-a-refusal-names-who-acted): a
  `node_added` carries the whole node, served by the agent port as a
  snapshot serves a node, so an agent that sees a commit appear in the
  ide's `stacks` knows what it is in one trip (427 B instead of 172 B
  plus a second observation of 402 B); a stale refusal carries `acted`,
  the actions of other actors that ran on the state the refused actor
  saw, and the ide's toast says it ("you unstaged src/session.rs"); the
  stale rule is in `plev act --help`. sessions recorded before still
  open: the reader puts the node the event's own snapshot holds in place
  of the bare id.

- the ide never shows a status older than what it already did
  (2026-09-23, task-84): a refresh read before a stage, an unstage, a
  discard or an ignore used to land over it, and for a moment
  `changes:staged` said a file was out of the index when it was in, or
  the reverse. the session now drops a read asked before a mutation the
  view already shows; the refresh the mutation's answer sends replaces
  it. the scripted agent of the ide laboratory no longer waits for the
  index to hold still: without that wait, on the old code, all ten runs
  with a person's intervention went wrong.

- a caret at rest costs two frames a second, not sixty (2026-09-23,
  task-82, adr a-tick-asks-for-a-deadline): text widgets answer their tick
  with a `Wake` (a frame now, and when the next is due) instead of "keep
  drawing", the engine's `AnimationTick` carries the real time since the
  last tick beside the clamped one, and the showcase shell sleeps until
  the caret's next flip. a focused, untouched field went from 120 of 120
  measured turns drawn to 4. the ide adopts it in a second phase.

- a frame as svg, and a page's meaning as json (2026-09-23, task-66b, adr
  svg-export-reads-the-scene): `showcase --render <page> --svg f.svg`
  writes the same frame the png shows as vectors, text as glyph outlines
  from the one shaping path, with every approximation (gradients,
  shadows, clips, images and backdrop blur left out) listed on stderr and
  to `--svg-report`; `--semantics f.json` writes the page's semantic
  projection in the lean form `plev observe` serves, `--source` keeping
  each node's source. `--out` is no longer required; one output is.

- what an agent receives is lean (2026-09-23, task-57b, adr
  lean-semantic-form-and-one-redaction): the sem serde form leaves a field
  at its default out and reads it back when absent, so a whole
  observation of the showcase's todo went from 6419 to 2039 bytes with
  three items (26 KB instead of 87 with 103). where a node was projected
  (`source`) now travels only when asked, `plev observe --source`; a
  recording keeps it as before. the rule that keeps a secret inside the
  process lives once, in `sem::redact`, for the port and the recorder
  alike.

- an agent operates a plev app from outside (2026-09-23, task-61, adr
  agents-act-through-the-semantic-contract): the new `wire` crate speaks a
  small json-lines protocol over a local socket, and `plev observe` /
  `plev act` use it. an agent sees the semantic snapshot (scoped, or as a
  delta since a revision it holds) and acts on what it saw; if the state
  moved, the action comes back refused with the delta. it goes through the
  same dispatcher as a click, never touches the backend, and the port is
  off unless `PLEV_AGENT=1` in a debug build.

- the Select panel scrolls (2026-09-23, task-81): a long list is reachable
  by wheel and by keyboard methods, and drawing, clicking and binding read
  one window of rows.

- the GPU's own time per frame (2026-09-23, task-73 phase 1b):
  `RenderConfig::gpu_timestamps` turns on timestamp queries around the
  layer and composite passes when measuring, and the measure example
  reports GPU work next to submit latency. on an M4 a frame's GPU work is
  0.2 to 1.2 ms, still 4 to 12 times its CPU; a dirty layer's pass costs
  the same with 3 draws or 22, and the composite runs on every drawn
  frame. off by default, the device asks for no feature.

- a session saves to a `.monster` file (2026-09-23, task-63 phase c, adr
  session-recording-replay-not-reexecution): `Recorder::persist` and
  `export` write a session file (magic `MONS`, next to the `MON0`
  animations) whose semantic track is JSON lines and whose frames are
  monster chunks byte for byte; `Session::open` reads it back strictly; a
  panic hook can save the last window. a 20 s window of the showcase todo
  is 76 KB. `monster_player` refuses a session with a clear message.

- a person can decline a confirmation (2026-09-23, task-58b): closing the
  confirmation modal in the ide now ends the agent's request as declined
  instead of leaving it waiting.

- accessibility is a projection of the contract (2026-09-23, task-60, adr
  accessibility-projects-the-semantic-snapshot): the screen reader tree is
  built from the same semantic snapshot agents and tests read, with stable
  node ids; a screen reader's click, value or step goes through the app's
  dispatcher like any other actor. the showcase attaches the bridge and
  its todo page is the first with a tree. the old tree, guessed from hit
  regions without labels, is gone.

- the git client proves the thesis (2026-09-23, task-62 phases a and b,
  adr commit-checks-the-index-it-observed): a person and an agent use the
  ide at the same time. an agent's commit decided on a staged set that
  changed is refused with the delta ("B left the staged files"), at the
  dispatcher and again at the real index right before `git commit`; a
  valid one ends with the real sha and file list. every click, key and
  menu entry goes through one dispatcher; refusals show as toasts; an
  agent's discard waits for a person to confirm. tested in real
  repositories against the real git worker.

- a session records itself, in memory (2026-09-23, task-63 phases a and
  b): the new `rec` crate keeps a bounded ring of what happened, as data:
  who observed what, the semantic state (whole or as a diff), every stage
  of every receipt with its request, input without the typed characters,
  and the frames on screen raised into the monster IR and chunked through
  discover. reading a session never creates a request; secrets fail
  closed. recording costs about 12 us a frame on the showcase todo.
  `monster::raise` is the mirror of `lower`. persisting to a `.monster`
  session file is the next step.

- the frame measures itself (2026-09-23, task-73 phase 1): `engine::perf::stages`
  counts and times each stage of a frame (semantic projection, scene
  build, text measure hits and misses, shaping, binds, hit tests, layer
  hashing, geometry, text resolve, upload bytes, encode, gpu wait), off by
  default at the cost of one relaxed load per probe;
  `cargo run --release -p measure` runs nine real
  interactions through the window's frame path and prints the tables. the
  first finding: on an M4 the frame is bound by the GPU (0.45 to 2.2 ms of
  wait against 30 to 400 us of CPU), which reshapes the incremental work
  ahead ([docs/log/steps/task-73-measurements.md](tasks/task-73-incremental-dependency-system/phase-1-measurements.md)).

- every interactive comps widget speaks the semantic contract (2026-09-23,
  task-59 phase b): chip, icon button, submit button, switch, slider,
  select, tabs, nav link, breadcrumb, split pane, virtual list, table,
  tree, context menu, modal and toast take `.sem(id)`, bind the rect they
  draw and hit-test with, and project their role, state and actions
  (children as keys under their id). every action reads the state it
  changes, so an actor never overwrites a value it did not see. fixed on
  the way: a full `Select` panel picked an invisible option from a 4px
  band at its foot.

- headless is first class (2026-09-23, task-66, adr
  headless-render-shares-the-frame-path): any frame renders without a
  window through the window's own path. `GpuContext::new_headless` builds
  every pipeline with the same code as the windowed constructor,
  `window::draw_frame` draws into any srgb view (`present_frame` is now
  acquire, draw, present, unchanged for every app), and `engine::headless`
  reads the texture back to png. `cargo run -p uigallery -- --render <page>
  --out f.png` and `monster_player <file> --frame <t> --out f.png` use it;
  the text probe's private offscreen pipeline is gone. 78 pages, two
  themes, two viewports: byte-identical across runs.

- the showcase todo runs on the semantic contract (2026-09-23, task-59
  phase a, adr semantic-binding-beside-the-scene): the compositor keeps a
  per-frame table of which capability sits where (`bind`, `hit`), beside
  the scene and outside every layer hash; `TextField`, `Checkbox` and
  `Button` take a `SemId` and project their role, state and actions; the
  todo page projects itself, owns a dispatcher, and every click, the Enter
  key and the filters reach the model only through it, as `Actor::Human`
  requests. a test drives the same page by clicks and by test requests
  and gets the same state; an open modal makes the page inert.

- actions carry the state their actor observed (2026-09-23, task-58, adr
  actions-carry-observed-state): `sem` gains `Request::observe`, a pure
  `check` with a fixed order (not found, unknown action, not authorized,
  stale, inert, disabled, invalid param, needs confirmation, accepted) and
  one `Dispatcher` for every actor. a request goes stale only when a value
  its action reads moved, and the rejection names the delta ("removed:
  B"); a hover or a cursor never invalidates it. execution runs only after
  acceptance or a person's confirmation (at the screen or through a screen
  reader), may end later with the id it received, and can be refused again
  at the point of truth. every step lands in a `Receipt` that sinks see as
  it happens. `cargo run -p commit` plays the thesis
  scenario.

- the semantic contract, crate `sem` (2026-09-23, task-57, adr
  semantic-contract-as-pure-projection): every control can now be
  described as data independent of its pixels: a persistent `SemId`
  chosen by the app (`commit.submit`, `changes.file[src/lib.rs]`), a
  `Role`, a label, state under keys, the actions it accepts with the state
  each one reads and who may request it, and its constraints. the app
  projects it on demand from live state (`Semantics::project`), the
  builder reports every violation at once, a `Snapshot` has a stable
  `Revision` that means the same across processes, and `Snapshot::diff`
  names what changed ("removed: lexer.rs"). secret fields keep only a
  length. pure: no gpu, no window, compiles for wasm. nothing in the
  engine or the apps uses it yet; task-58 adds preconditions and the
  dispatcher on top.

- [task-57: semantic contract (crate `sem`)](tasks/task-57-semantic-contract/README.md)
- [task-57b: lean JSON wire form and a single redaction](tasks/task-57b-lean-wire-form-and-one-redaction/README.md)
- [task-58: action with precondition, dispatch and receipts](tasks/task-58-actions-preconditions-receipts/README.md)
- [task-58b: declining a confirmation](tasks/task-58b-decline-a-confirmation/README.md)
- [task-59: semantic binding with the render and one operational path](tasks/task-59-semantic-binding-and-one-path/README.md)
- [task-60: accessibility as a projection of the contract](tasks/task-60-accessibility-projection/README.md)
- [task-61: agent surface (local socket, observe, act)](tasks/task-61-agent-surface/README.md)
- [task-62: proof in the ide: stale-state rejection and human-agent handoff](tasks/task-62-ide-human-agent-proof/README.md)
- [task-66: headless as a first-class citizen](tasks/task-66-headless-first-class/README.md)
- [task-66b: SVG export and `--semantics` in headless mode](tasks/task-66b-svg-export/README.md)
- [task-67: structural regression before pixels](tasks/task-67-structural-regression/README.md)
- [task-68: plev IR and verifier](tasks/task-68-plev-ir-and-verifier/README.md)
- [task-73: incremental dependency system](tasks/task-73-incremental-dependency-system/README.md)
- [task-81: scrolling panel for `Select`](tasks/task-81-select-scrolling-panel/README.md)
- [task-82: the caret cannot request 60 frames per second](tasks/task-82-caret-blink-render-on-demand/README.md)
- [task-83: the delta carries the node that was just added, and the refusal names who acted](tasks/task-83-delta-carries-node-refusal-names-actor/README.md)
- [task-84: an old refresh does not undo what the ide has already accepted](tasks/task-84-ide-refresh-never-goes-back/README.md)
- [task-85: the commit that the receipt announces is already in the projection, with the files](tasks/task-85-the-commit-a-receipt-announces-is-projected/README.md)
- [task-86: the ide people use opens the agent port](tasks/task-86-the-ide-opens-the-agent-port/README.md)
- [task-87: a terminal stage carries the revision the app was left in](tasks/task-87-a-terminal-stage-carries-the-revision/README.md)
- [task-89: an act relies only on what was shown to the agent](tasks/task-89-an-act-rests-only-on-what-was-shown/README.md)
- [task-90: a refusal names who acted since the agent looked](tasks/task-90-a-refusal-names-who-acted-since-the-look/README.md)
- ADR 0038: [accessibility projects the semantic snapshot](adr/interface/0038-accessibility-projects-the-semantic-snapshot.md)
- ADR 0037: [session recording: replay, not re-execution](adr/recording/0037-session-recording-replay-not-reexecution.md)
- ADR 0046: [the inspector reads a session, it never runs it](adr/recording/0046-the-inspector-reads-a-session-never-runs-it.md)
- ADR 0034: [headless render shares the frame path](adr/rendering/0034-headless-render-shares-the-frame-path.md)
- ADR 0041: [the svg export reads the scene, not the pixels](adr/rendering/0041-svg-export-reads-the-scene.md)
- ADR 0042: [a tick asks for a deadline, not for every frame](adr/rendering/0042-a-tick-asks-for-a-deadline.md)
- ADR 0032: [semantic contract as a pure projection of app state](adr/semantics/0032-semantic-contract-as-pure-projection.md)
- ADR 0033: [actions carry the state their actor observed](adr/semantics/0033-actions-carry-observed-state.md)
- ADR 0035: [semantic binding beside the scene, and the click as a request](adr/semantics/0035-semantic-binding-beside-the-scene.md)
- ADR 0036: [a commit checks the index it observed](adr/semantics/0036-commit-checks-the-index-it-observed.md)
- ADR 0039: [agents act through the semantic contract](adr/semantics/0039-agents-act-through-the-semantic-contract.md)
- ADR 0040: [the semantic form is lean, and one rule redacts a secret](adr/semantics/0040-lean-semantic-form-and-one-redaction.md)
- ADR 0043: [a delta carries the node it adds, and a stale refusal names who acted](adr/semantics/0043-a-delta-carries-the-node-a-refusal-names-who-acted.md)
- ADR 0044: [a terminal stage carries the revision the app is at](adr/semantics/0044-a-terminal-stage-carries-the-revision.md)
- ADR 0047: [an act rests on what its actor was shown](adr/semantics/0047-an-act-rests-on-what-its-actor-was-shown.md)
- ADR 0045: [regression by meaning before pixels](adr/workflows/0045-regression-by-meaning-before-pixels.md)
- ADR 0085: [integration into main is a pull request merged by merge commit](adr/workflows/0085-integration-by-merge-commit.md)
- experiment: [LLM agents commit through the IDE contract: stale refusal and handoff, five rounds](experiments/agent-proof-checkpoint-a/README.md)
- experiment: [caret blink as a deadline: frames drawn with a focused, idle field](experiments/caret-render-on-demand/README.md)
- experiment: [what each stage of a frame costs in nine scenarios](experiments/frame-cost-by-stage/README.md)
- program: [program: from GPU UI engine to generative interaction runtime](research/generative-runtime-program.md)
- record: [autonomous cycle journal](research/autonomous-cycle-journal.md)
- thesis: [plev: thesis of the generative interaction runtime](research/generative-runtime-thesis.md)

## 2026-09-22

- hot reload on by default (2026-09-22, adr
  hot-reload-on-by-default-in-gpu-context): the shader watcher moved from
  the engine's `window::App` into `GpuContext`, wakes the loop with
  `request_redraw`, is polled by `present_frame` (`effects` is now `&mut`
  there) and marks every layer dirty on a swap; every debug desktop build
  of every app has it. fixes on the way: `shader_dir` pointed at a
  directory that did not exist, `reload_shader` used the raw surface
  format instead of the srgb view format, the narrate root did not match
  `file!()`. `cfg(hot_reload)` from build.rs replaces the feature cfg;
  release, wasm and mobile are unchanged. literal hot reload
  (`hot_reload/literals.rs`, the rsx-hotreload idea without the macro):
  a saved `.rs` whose only change is strings or hex colors is applied to
  the open window, state intact, at `TextNodeKey`, `TextMeasurer`,
  `push_to_layer` and the clear color; a code change rebuilds
  (`cargo build -p $CARGO_PKG_NAME --bin`) and re-execs the running
  binary. `cargo run -p uigallery` alone is the dev loop.

- [task-56: hot reload](tasks/task-56-hot-reload/README.md)
- ADR 0031: [hot reload on by default: shaders and literals in place, code by rebuild](adr/workflows/0031-hot-reload-on-by-default-in-gpu-context.md)
- learning: [hot reload session: what worked and what bit us](learnings/hot-reload.md)

## 2026-09-21

- showcase, one page per component (2026-09-21, adr
  showcase-one-page-per-component): the sidebar is a folding tree of
  three groups (Foundations, Components alphabetical, Apps Demo) over
  the page table `view::pages::PAGES`; a page is its own pane, an alias
  of a neighbour's example with a "still missing" note, or a placeholder
  slot for a widget comps does not have yet; Forms, Chrome, Extras,
  Overlays and Lists split into one module per widget; `comps::nav::
  Sidebar` gains `NavGroup`; the launch argument is the page title.
  docs/adr/ became docs/adr/ with README.md as its index, the changelog
  moved to [docs/log/changelog.md](CHANGELOG.md).

- core theme and the buttons poc (2026-09-21, adr
  core-theme-tailwind-classes-as-recipes): `Theme::hoff_core` is the
  typescript kit (nine shades, six accents, depths as shadow stacks,
  Inter -0.02em, radii 8/10/12/16/22) and the showcase default;
  `theme.button` holds one recipe per button variant and state, the css
  class lists as tokens; `comps::action` rewritten (Button, IconButton,
  SubmitButton over a shared press model); the showcase Buttons page is
  the kit's ButtonsPage. two global fixes: `rect_sdf.wgsl` mixes in
  premultiplied linear (the dark fringe on every rounded corner is
  gone) and blur-0 shadows are anti-aliased.

- editor and git in the showcase (2026-09-21, adr
  embed-an-app-view-by-frame-origin): two app sections filling the area
  under the header; the ide is a lib + bin with `GitSession`;
  `WorkspaceView::set_frame`, `Sidebar.origin`; editor wheel snaps to
  lines; nav link hover is text-only; the sidebar band fades where it
  still has links.

- [task-53: editor and git in the showcase](tasks/task-53-editor-and-git-in-the-showcase/README.md)
- [task-54: core theme, buttons poc](tasks/task-54-core-theme-buttons-poc/README.md)
- [task-55: showcase, one page per component](tasks/task-55-showcase-one-page-per-component/README.md)
- ADR 0028: [core theme: the tailwind class lists as button recipes](adr/rendering/0028-core-theme-tailwind-classes-as-recipes.md)
- ADR 0029: [embed an app view by giving it a frame origin](adr/rendering/0029-embed-an-app-view-by-frame-origin.md)
- ADR 0030: [showcase: one page per component, the sidebar as a folding tree](adr/structure/0030-showcase-one-page-per-component.md)
- learning: [core buttons session: what worked and what bit us](learnings/core-buttons.md)
- learning: [showcase pages session: what worked and what bit us](learnings/showcase-pages.md)

## 2026-09-19

- learning: [comps session: what worked and what bit us](learnings/comps.md)

## 2026-09-18

- comps, the design system crate (2026-09-18, adr
  comps-one-design-system-crate): `engine::ui`, `text_input`, `overlay`,
  `charts`, `GraphView` and the editor view moved out of the engine into
  rust/engine/src/widgets (git mv), organized by category (core, recipe, icons,
  action, form, nav, content, feedback, overlay, charts, graph, editor,
  shell); the legacy `Ui` builder and `UiTheme` are gone. the theme grew
  the scales the widgets were improvising (shape by role, control sizes,
  container sizes, durations, breakpoints + sidebar mode, shadow stacks
  by elevation, `text_active`/`text_default`/`tabs`/wash alphas,
  `headline`/`mono`/`readout`, `Theme::hoff_light`); every widget reads
  them, none restates a number; widgets that lay out from tokens take
  `&Theme` on `handle_event`. new widgets: TextField (glass field over the
  editing core), Badge, Avatar, Separator, Panel, Stat, CodeBlock,
  Skeleton, Table (columns by weight, drop by priority), NavLink, Sidebar
  (full / rail / drawer by breakpoint, pinned footer links), PanelHeader,
  Breadcrumb (collapses), AppShell (sidebar + header + content rect +
  menu button + safe area). modal and toast fit a phone width. the ide
  lost its theme and components copies (2 160 lines) and draws from
  comps over hoff / hoff light (status.rs is its only token); the
  showcase is framed by AppShell, gained the Chrome section and its App
  section's keyboard finally reaches the field (it sat behind an
  allow(dead_code)); urnaui's Field is a TextField alias; the todo and
  text_input demos were absorbed by the showcase sections, the editor
  demo moved to comps, the other demo palettes read Theme::default().
  `TextMeasurer::elide_path` came from the ide (and now drops several
  middle segments, the old helper dropped one). matrix test: every widget
  under 13 themes at phone and desktop widths. inventory and migration
  record: [docs/ref/component-catalog.md](ref/architecture/component-catalog.md). workspace tests 855 -> 1414.

- urnaui, the .urna explorer (2026-09-18): nestui renamed to urnaui end to
  end (crate, modules, types, apps/urnadbapp/web, tool/tasks/webtarget.sh urnadbapp, gate, ci)
  against hoffresearch/urna v0.4.0; the portable reader accepts both
  magics so pre-rename .nest corpora open. the poc's real bugs fixed
  against the 38k-card mtg corpus (530 MB): the file was read into ram
  twice (now one memmap2 map, sections decoded through UrnaView over it),
  results carried no text (the worker ships ChunksLoaded right behind
  Opened), results waited for the next mouse move (worker wakes the loop,
  adr worker-results-wake-the-event-loop), the subprocess bridges hung on
  a full pipe (shared draining helper), the chunks tab showed 0x03 spans
  while cite/search report the 0x16 overlay (now one span_label), blobs
  and spaces were not parsed from inspect. urna pillars surfaced: search
  over a named multimodal space (`space: <name>` in the select, registry
  embedder with the space's dim/model_hash gate), explain panel with the
  selected hit's citation + text + decoded frame, chunk filter by text
  terms / id / urna:// citation, validate, media card with hash-verified
  export, frame previews straight out of the inlined av1 via ffmpeg
  subfile (adr urnaui-media-frames-in-place-via-ffmpeg-subfile), Browse…
  native dialog (rfd). tests 57 -> 80 incl. a media corpus built with the
  urna writer (blob + overlay + space); wasm check green.

- arc unified (2026-09-18): docs/{arc.yaml, arc.md, arc.mmd} became one
  file, docs/ATLAS.toml: the machine map as tables, the human reference
  under [prose], the frame-flow mermaid as a literal string under
  [diagram]. the sync guard, AGENTS.md, the conventions, the pr template
  and the README point at it. content unchanged except nest/nestui ->
  urna/urnaui, which is the rename in flight on this branch.

- ADR 0025: [urnaui: decode media frames in place with ffmpeg's subfile protocol](adr/importing/0025-urnaui-media-frames-in-place-via-ffmpeg-subfile.md)
- ADR 0026: [background results wake the event loop; child pipes drain from the start](adr/rendering/0026-worker-results-wake-the-event-loop.md)
- ADR 0027: [comps: one design system crate, every value a token](adr/structure/0027-comps-one-design-system-crate.md)
- reference: [ui component catalog](ref/architecture/component-catalog.md)

## 2026-08-31

- glyph raster hardened (2026-08-31): atlas cache key is cosmic-text's
  CacheKey verbatim; fresh swash context per rasterization; one-texel
  zeroed gutter on all four sides of every slot; quad UVs in texels,
  normalized in text.wgsl against textureDimensions (mid-frame atlas grows
  keep emitted quads valid); atlas grows to max before evicting; every
  slot reuse marks atlas_disturbed and re-resolves all layers next frame;
  set_raster_scale is #[must_use]; LRU capacity drops return their
  rectangles; empty glyphs carry alloc_id None; headless pixel validation
  via engine::text::probe (incl. multi-frame render_frames). see
  [docs/adr/rendering/0024-glyph-raster-identity-and-atlas-isolation.md](adr/rendering/0024-glyph-raster-identity-and-atlas-isolation.md)

- jetbrains mono removed (2026-08-31): inclusive sans is the single
  embedded family; ~550kb less embedded font data, monospace pinned to it
  as well since no mono face ships.

- ADR 0024: [glyph raster identity and atlas isolation](adr/rendering/0024-glyph-raster-identity-and-atlas-isolation.md)

## 2026-08-30

- ADR 0023: [official app pattern: state in structs, retained widgets, explicit invalidation](adr/structure/0023-official-app-pattern.md)

## 2026-08

- engine wave (2026-08): inclusive sans replaces rubik+inter as the one
  embedded family (SIL OFL, five upright weights 300-700, sans family
  pinned in fontdb, ~2.1mb lighter); body letter-spacing zeroed (the
  0.025em token was calibrated for rubik). HiDPI text fixed at the root:
  glyphs rasterize at physical scale (TextSystem::set_raster_scale,
  hooked centrally in resolve_layer_text, cache purged on change), no
  more stretched bitmaps on retina. register_embedded_fonts now returns
  the registered ids and emit_glyphs warns once per non-embedded font
  actually shaped (caught a CJK/math demo string and a text "play"
  glyph, both fixed). ADR updates: embed-every-font-weight-in-use,
  adr-004-hidpi-projection.

## 2026-07-13

- ADR 0022: [one knowledge tree, and a minimal root](adr/structure/0022-one-knowledge-tree-and-a-minimal-root.md)

## 2026-06-12

- research: [mon experiment](research/mon-experiment-notes.md)

## 2026-06-10

- learning: [why the apps bypassed the engine, and what that costs](learnings/apps-bypass-the-engine.md)
- learning: [the two gamma bugs: one assumption, two opposite failures](learnings/the-two-gamma-bugs.md)
- reference: [hoff visual tokens (measured)](ref/architecture/hoff-visual-tokens.md)

## 2026-06 to 2026-08, entries without a day

- widgets for real apps: chip, empty state, indeterminate spinner,
  split pane, icon button; engine::charts promoted from the showcase
  (model + draw + meter/hbars) and engine::graph + GraphView (force
  layout, pan/zoom canvas) with the same handle_event/render contract
  as every widget. TextMeasurer::truncate_to_width (grapheme-aware),
  engine::clipboard re-export, actions::ShortcutMap for the simple
  keymap case. ADR: official-app-pattern.

- nestui (crates/nestui): .nest vector-db explorer on plev, 6 tabs
  (open/overview/search/chunks/graph/stats), native backend via
  nest-format/nest-runtime path deps + worker thread, portable wasm
  reader (nestread) for the web target.

- showcase 12 -> 14 tabs (typography, effects), scrollable sidebar.
  ci matrix: macos + ubuntu + wasm (.github/workflows/ci.yml);
  tool/tasks/fullcheck.sh skips the wasm leg with a warning when the rust-std wasm
  target is absent (Homebrew rust), CI stays the full verification.

- one knowledge tree: kdb/ renamed to docs/ and the arc trio moved into
  it (docs/, from doc/arc/); doc/ was retired (its changelog copy
  was the stale duplicate of this file). the mon working note left the
  root for [docs/ref/mon-experiment-notes.md](research/mon-experiment-notes.md). web/ owns
  the web target end to end (Trunk.toml, index.html, dist output) and
  script/ owns the commands (gate, web). the arc sync guard, the
  instruction source (.contracts/.agents/AGENTS.md, the only
  instruction file; the root carries none), the pr template, typos.toml
  and the crate docstrings follow the new paths; the svg crate and the
  svg2monster cli entered the arc trio. the root gained LICENSE (mit),
  rust-toolchain.toml (stable + wasm32 pinned) and .gitattributes, and
  lost .env.example and the Makefile (tool/tasks/fullcheck.sh runs the four-part
  gate; the other targets were one cargo call each). study-clone
  references in docs now name the repository and the inner path, never
  a machine path. ADR: one-knowledge-tree-and-a-minimal-root.

- prime number creatures (crates/prime_creatures): the Entropic Life XVI canvas
  demo ported to a native plev crate, desktop and wasm. pure, tested sim core
  (seeded rng, prime coherence matrix 250x250 in four modes, grid-local steering
  physics with kuramoto sync); faithful render through the layer encoder, motion
  trails by position history, cyan bond links via paths, glow halos, breathing
  cores, logical-pixel world, fixed-timestep loop, left-mouse brush. ADR:
  motion-trails-by-position-history. study clone under refs/prime-number-creatures.

- workspace organized to tiers: engine at root, libraries and apps in
  crates (git, ide, lot, monster, narrate, narrate-macro, parser, rope,
  showcase), demos in examples. crate renames (editor_core to rope,
  git_backend to git, basic-ide to ide, prs to parser, anm to monster,
  narrate_macro to narrate-macro). shaders moved into src/gpu/shaders.
  cargo to workspace.package + workspace.dependencies + workspace.lints +
  tuned profiles; every crate publish = false with a description.

- kdb consolidated to adr + how-to. the brain .lua graph and the framework
  catalogs were retired (drifted, parallel source of truth); the monster
  spec moved to kdb/adr/monster-format-v0.md; conventions moved to
  doc/.conventions/conventions.lua; AGENTS.md points to both. four new ADRs
  (monster format, import-by-conversion, transpiler droplist, workspace).

- qa pass: clippy --all-targets -D warnings clean (uninlined format args
  made idiomatic), naming residue fixed (basicIDE to plev ide), README in
  english added, arc trio refreshed. 1274 tests green.

- monster bridge: lottie retired at the door. lot::cnv samples a lottie once,
  dedups tessellated payloads into the asset table (exact quantized bytes:
  a static shape is one asset and zero delta bytes) and encodes .monster;
  asset_path defines the Path payload wire (uniform color, twips vertices,
  u16 indices, deterministic chunk split); stage size travels in the
  description track (stage WxH). examples: lot2monster (converter cli),
  monster_player (decodes and plays .monster, zero lottie linked).
  measured on the 5 corpus files: cards 0.36x and explosion 0.74x of the
  json (format wins on discrete motion); girl 6.5x, snake 42x, money 53x
  (60fps full-body morphs pay v0's sampled-geometry cost; v1 lever:
  morph tracks, the swf DefineMorphShape lesson)

- monster: encoder mode B (discover) + optimizer passes + full delta ops
  decodable; bench vs json/gzip/webm on 4 fixtures; mode B e2e gate
  (max deviation 0.0375 px / 0.0035 per channel); 124 crate tests

- parser transpiler poc: react tsx+sass card and gpui separator emitted as
  plev builder source; goldens byte-identical to the corpus copies;
  honest droplist (38 entries with file:line and reason, count frozen in
  test); emitted code compiles and renders (examples/parser_card; known
  defect: body text run does not wrap yet)

- workspace clippy clean again (7 parser lints); fmt clean

- brain knowledge base: kdb/brain-Mythos-e-bre/ (vision, anim format poc,
  transpiler poc, semantics/a11y, research notes, org plan)

- agents.md as the single agent instruction source; doc/arc trio (md,
  yaml, mmd); doc/.conventions

- path: open sub-paths auto-finished before tessellation (lyon abort
  fixed at the root); 2 regression tests

- 11 plev-era demos ported as official examples (crate rename, msaa field,
  srgb view, linear clears); makepad_charts verified alive

- test surface: 1022 passing, 0 failed

## 2026-05-11

- [task-48: ide, native git client in plev](tasks/task-48-ide-git-client/README.md)

## 2026-04-08

- [task-47: git, backend for plev apps](tasks/task-47-git-backend/README.md)

## 2026-04-01

- ADR 0020: [git backend: gix for reads, git CLI for mutations, a worker thread for the UI](adr/structure/0020-git-backend-gix-reads-cli-mutations.md)

## 2026-03-11

- experiment: [benchmark results, plev v0.2](experiments/benchmarks-v02/README.md)

## 2026-03-10

- [task-46: rope, text editing core](tasks/task-46-rope-editing-core/README.md)

## 2026-03-04

- ADR 0021: [the text editing core follows helix: document is rope plus selections plus history](adr/interface/0021-text-editing-core-follows-helix.md)

## 2026-02-16

- [task-51: workspace restructure and naming rebrand](tasks/task-51-workspace-restructure/README.md)

## 2025-11-14

- [task-50: multiplatform showcase as a lib (desktop/web/android/ios)](tasks/task-50-showcase-multiplatform/README.md)

## 2025-10-01

- [task-49: prime number creatures, port of entropic life](tasks/task-49-prime-creatures/README.md)

## 2025-09-24

- ADR 0014: [motion trails come from position history, not framebuffer accumulation](adr/rendering/0014-motion-trails-by-position-history.md)

## 2025-08-18

- ADR 0013: [engine at root, libraries and apps in crates, demos in examples](adr/structure/0013-workspace-engine-at-root-libs-in-crates-demos-in-examples.md)

## 2025-07-16

- [task-45: parser, transpiler poc (react/gpui to builder)](tasks/task-45-parser-transpiler/README.md)

## 2025-07-08

- ADR 0012: [the transpiler reports every unmapped construct, never drops silently](adr/importing/0012-transpiler-reports-every-unmapped-construct.md)

## 2025-06-17

- [task-44: lot, lottie importer and bridge to .monster](tasks/task-44-lot-lottie-importer/README.md)

## 2025-06-10

- ADR 0011: [import foreign formats by conversion, never by embedding a foreign runtime](adr/importing/0011-import-foreign-formats-by-conversion-not-embedding.md)

## 2025-05-21

- [task-43: monster, binary animation codec v0](tasks/task-43-monster-codec/README.md)

## 2025-05-14

- ADR 0010: [the monster animation format: keyframes plus discovered deltas](adr/importing/0010-binary-animation-format-with-discovered-deltas.md)

## 2025-03-26

- reference: [monster format v0 specification](ref/architecture/monster-format-v0.md)

## 2025-02-19

### session record: responsiveness, real measurement, web target, touch

cross-referenced from the session transcript, `git log` (95 commits at
time of writing) and the system docs (docs/plano-tecnico.md as the stable
architecture contract, docs/status.md as living state). format: what was
tried and failed, then the path that worked, with the commit that fixed
it. why-level rationale lives in kdb/adr/; operational detail in
kdb/how-to/.

#### trigger

user report: layout quality far from the reference. labels larger than
their shapes with text spilling out, elements not distributing across the
window, and zero testing on browser, iOS and android.

#### episode: text spilling out of shapes

- failed path: shapes sized by a per-character heuristic
  (chars * size * 0.58) while the rasterizer shaped with rubik 600.
  earlier in the project the same class shipped as chars * 0.6 and as
  letter-spacing drawn but not measured. repeated band-aid attempts
  (padding tweaks per widget) changed symptoms, not the class
- working path: one TextStyle per run, shared by measurement and drawing;
  heuristic deleted at its single definition point, fixing twelve call
  sites at once; latent builder divergence closed in the same change.
  commits f15198a, 5854941. ADR: one-text-style-for-measurement-and-drawing

#### episode: no responsive distribution

- failed path: assuming the resize pipeline was broken. diagnosis proved
  the pipeline correct and the geometry constant-driven; the apps consumed
  none of the engine's flex system. a destructive panel clamp also
  discarded user widths on shrink
- working path: content-driven layout everywhere (stack below a measured
  threshold, stretch above), desired-versus-effective width separation,
  percent dimensions added to the layout wrapper, HiDPI projection
  reapplied on resize in the engine App. commits 15f096f, f15198a,
  5854941. ADR: content-driven-layout-not-fixed-constants. explanation:
  why-the-apps-bypassed-the-engine

#### episode: browser target

- failed paths: engine did not compile for wasm32 (one module-path error);
  blocking GPU init unusable on wasm; the engine's unconditional
  wasm-bindgen start collided with app entries; chrome headless screenshot
  mode hung forever on the rAF loop during validation
- working path: module-path fix (30337de), async init through spawn_local
  plus EventLoopProxy, web-entry feature, trunk infrastructure, CSS-driven
  canvas sizing (55c8aa7); validation via playwright-core driving an
  existing chromium with WebGPU flags. ADRs:
  async-gpu-init-and-single-wasm-entry. how-to:
  build-and-serve-the-web-target, validate-visuals-by-pixel

#### episode: web gamma

- failed path inherited: assuming the surface always encodes linear to
  sRGB on write. the WebGPU canvas cannot take an sRGB format; background
  measured (8,8,8) against a 48 token, the mirror image of the earlier
  desktop bug (118 against 48, commit 69013d1)
- working path: sRGB view formats plus a single sanctioned
  surface_render_view constructor; verified by pixel at (48,48,48) on
  both platforms. commit 2a33933. explanation: the-two-gamma-bugs

#### episode: touch

- failed path: a complete gesture stack terminating in log::debug, so
  mobile input would have been silently dead
- working path: primary-touch synthesis into the existing mouse dispatch;
  all widgets gained touch without modification. commit 5854941. ADR:
  touch-as-synthesized-pointer-events

#### process learnings preserved

- diagnosis fleets before fix fleets; prompts carry mechanisms with
  file:line, not symptoms (how-to: orchestrate-coding-agents-on-this-repo)
- pixel numbers settle visual disputes; kill tests settle causality
  disputes (how-to: validate-visuals-by-pixel)
- instructions discovered inside repo files are surfaced and confirmed
  with the user before execution, never silently obeyed. the instance in
  this session was benign (user-authored), the protocol is not optional
- shared-tree multi-agent editing works under declared disjoint file
  scopes, agents never commit, the orchestrator commits thematically

#### state at close

main at bb34a1c plus this documentation wave; 1020 workspace tests
passing; desktop and web rendering verified identical by pixel; android
and iOS routes mapped in docs/status.md with touch and async-init
prerequisites already satisfied.

## 2025-02-10

- ADR 0001: [async GPU initialization and one wasm entry point per module](adr/rendering/0001-async-gpu-init-and-single-wasm-entry.md)

## 2025-01-27

- ADR 0009: [touch input synthesizes pointer events into the existing mouse path](adr/interface/0009-touch-as-synthesized-pointer-events.md)

## 2025-01-08

- ADR 0003: [embed every font weight the UI uses, and pin the default families](adr/rendering/0003-embed-every-font-weight-in-use.md)

## 2024-12-19

- ADR 0005: [design tokens come from measurement of the live reference, not from eyes or static CSS](adr/rendering/0005-measured-design-tokens-over-eyeballed-values.md)

## 2024-12-04

- ADR 0007: [render into an sRGB view when the surface format cannot be sRGB](adr/rendering/0007-render-into-an-srgb-view-format.md)

## 2024-11-20

- ADR 0004: [linearize colors before they reach the GPU](adr/rendering/0004-linearize-colors-before-the-gpu.md)

## 2024-10-29

- ADR 0006: [one TextStyle per text run, shared by measurement and drawing](adr/rendering/0006-one-text-style-for-measurement-and-drawing.md)

## 2024-10-07

- ADR 0008: [render on demand makes invalidation a correctness contract](adr/rendering/0008-render-on-demand-requires-explicit-invalidation.md)

## 2024-09-18

- ADR 0002: [container geometry derives from available space, never from constants](adr/rendering/0002-content-driven-layout-not-fixed-constants.md)

## 2024-07-09

- reference: [resolving 107 clippy warnings to zero](ref/architecture/clippy-zero-warnings.md)

## 2024-06-17

- ADR 0016: [adr-004: adotamos projecao ortografica em logical pixels para apps hidpi](adr/rendering/0016-hidpi-projection.md)

## 2024-05-29

- [task-52: modularization by single responsibility principle](tasks/task-52-srp-modularization/README.md)

## 2024-05-23

- reference: [SRP modularization with a 300-line limit](ref/architecture/srp-modularization.md)

## 2024-05-21

- ADR 0015: [adr-003: modularizacao por single responsibility principle](adr/structure/0015-srp-modularization.md)

## 2024-04-14

- ADR 0017: [adotamos pleverror como tipo de erro unificado e eliminamos debt de lint](adr/workflows/0017-error-handling-lint-cleanup.md)

## 2024-02-18

- ADR 0018: [adotamos runtime parser + override map para hot reload de plev_narrate! DSL](adr/workflows/0018-narrate-hot-reload.md)

## 2024-02-06

- ADR 0019: [adotamos shader hot reload via notify file watcher sobre dylib reloading](adr/workflows/0019-shader-hot-reload.md)

## 2024-01-22

- reference: [hot reload design - plev](ref/architecture/hot-reload-design.md)

## 2023-12-14

- [task-41: license + preparation for public release](tasks/task-41-license-and-public-release/README.md)

## 2023-11-09

- [task-38: event batching](tasks/task-38-event-batching/README.md)

## 2023-10-02

- [task-37: animation enhancements](tasks/task-37-animation-enhancements/README.md)

## 2023-08-28

- [task-36: signal system hardening](tasks/task-36-signal-hardening/README.md)

## 2023-07-23

- [task-35: fix spring<t>, analytical solver](tasks/task-35-spring-analytical-solver/README.md)

## 2023-07-11

- [task-33: WASM plugin architecture](tasks/task-33-wasm-plugins/README.md)

## 2023-07-05

- research: [WASM plugin architecture - research doc](research/wasm-plugin-architecture.md)

## 2023-06-25

- [task-32: text system upgrade assessment](tasks/task-32-text-upgrade-assessment/README.md)

## 2023-06-20

- research: [parley vs cosmic-text - factual comparison for plev](research/parley-vs-cosmic-text.md)

## 2023-06-12

- [task-31: vector path rendering (lyon integration)](tasks/task-31-vector-paths/README.md)

## 2023-05-02

- [task-30: accessibility (accesskit)](tasks/task-30-accessibility/README.md)

## 2023-04-12

- [task-34: exploration and extraction of ideas from the reference repos](tasks/task-34-exploration-extraction/README.md)
- research: [task-34 extraction: phase e (WASM runtime) + phase f (competitors)](research/extraction-fase-ef.md)
- research: [pattern extraction: parley, lyon, glam-rs](research/pattern-extraction-parley-lyon-glam.md)
- research: [pattern extraction: tui apps (yazi, television, bottom)](research/pattern-extraction-tui-apps.md)

## 2023-04-06

- research: [extracted patterns, 56 reference repos](research/extracted-patterns.md)

## 2023-03-22

- [task-29: app demo real (proof of life)](tasks/task-29-app-demo-real/README.md)

## 2023-01-25

- [task-28: editable text](tasks/task-28-editable-text/README.md)

## 2022-12-21

- [task-27: animation system](tasks/task-27-animation-system/README.md)

## 2022-12-10

- research: [technical report: plev vs makepad](research/plev-vs-makepad-report.md)

## 2022-11-19

- research: [makepad visual feature inventory vs plev gap analysis](research/makepad-gap-analysis.md)

## 2022-10-27

- reference: [touch & gesture recognition design](ref/architecture/touch-gesture-design.md)

## 2022-10-21

- research: [competitors by tier](research/competitors-by-tier.md)

## 2022-06-15

- [task-ref: research briefing (50+ repos)](tasks/task-ref-research-briefing/README.md)
- research: [reference analysis: accessibility (accesskit)](research/accessibility.md)
- research: [reference analysis: animation and motion](research/animation-motion.md)
- research: [reference analysis: charts and visualization](research/charts-visualization.md)
- research: [competitive positioning, plev](research/competitive-positioning.md)
- research: [competitor analysis, gui/rendering frameworks](research/competitors.md)
- research: [reference analysis: emulators and WASM runtimes](research/emulators-wasm-runtimes.md)
- research: [integration candidates, ranking](research/integration-candidates.md)
- research: [linebender ecosystem, analysis for plev](research/linebender-ecosystem.md)
- research: [reference analysis: math, physics and geometry](research/math-physics-geometry.md)
- research: [technology radar, plev](research/technology-radar.md)
- research: [reference analysis: tui patterns](research/tui-patterns.md)
- research: [reference analysis: WASM tooling ecosystem](research/wasm-tooling.md)

## 2022-05-03

- [task-25: benchmarks](tasks/task-25-benchmarks/README.md)

## 2022-02-24

- reference: [animation pattern, frame-based exponential lerp](ref/architecture/animation-pattern-lerp.md)

## 2022-01-19

- reference: [android emulator deploy](ref/platforms/android-emulator-deploy.md)

## 2021-12-27

- [task-24: cleanup & readme](tasks/task-24-cleanup-readme/README.md)

## 2021-12-18

- [task-23: CI/CD](tasks/task-23-ci-cd/README.md)

## 2021-12-11

- [task-22: error messages & DX](tasks/task-22-error-messages-dx/README.md)

## 2021-12-04

- [task-21: documentation of the narrate DSL](tasks/task-21-narrate-dsl-documentation/README.md)

## 2021-11-08

- [task-17: input <-> layer hit-testing](tasks/task-17-input-layer-hit-testing/README.md)

## 2021-10-24

- [task-16: effects <-> layer integration](tasks/task-16-effects-layer-integration/README.md)

## 2021-10-10

- [task-15: pipeline DSL -> builder -> compositor](tasks/task-15-pipeline-dsl-builder-compositor/README.md)

## 2021-09-27

- [task-14: plev_narrate!, verbal DSL](tasks/task-14-plev-narrate-dsl-verbal/README.md)

## 2021-09-17

- reference: [WASM/webgpu validation - knowledge](ref/platforms/wasm-webgpu-validation.md)

## 2021-09-12

- [task-13: mobile specifics, safe areas, IME, lifecycle](tasks/task-13-mobile-specifics-safe-areas-ime-lifecycle/README.md)

## 2021-08-30

- reference: [mobile specifics - safe areas, IME, lifecycle](ref/platforms/mobile-specifics.md)

## 2021-08-29

- [task-12: ios build + lifecycle](tasks/task-12-ios-build-lifecycle/README.md)

## 2021-08-16

- [task-11: android build + lifecycle](tasks/task-11-android-build-lifecycle/README.md)

## 2021-08-15

- reference: [ios build - decisions and knowledge](ref/platforms/ios-build.md)

## 2021-08-02

- [task-10: input system, touch + gestures](tasks/task-10-input-system-touch-gestures/README.md)

## 2021-07-26

- reference: [DSL narrate, design decisions](ref/architecture/dsl-narrate-design.md)

## 2021-07-19

- [task-09: input system, keyboard + mouse](tasks/task-09-input-system-keyboard-mouse/README.md)

## 2021-07-06

- [task-08: compositing effects (blur, shadow, opacity)](tasks/task-08-compositing-effects/README.md)

## 2021-07-04

- reference: [effects architecture (task-08)](ref/architecture/effects-architecture.md)

## 2021-06-21

- [task-07: independent layer system](tasks/task-07-independent-layer-system/README.md)

## 2021-06-16

- reference: [layer system - technical decisions](ref/architecture/layer-system.md)

## 2021-06-08

- [task-06: WASM/webgpu validation in the browser](tasks/task-06-wasm-webgpu-validation-in-the-browser/README.md)

## 2021-05-27

- reference: [input system design (task-09)](ref/architecture/input-system-design.md)

## 2021-05-23

- [task-05: declarative API + builder pattern](tasks/task-05-declarative-api-builder-pattern/README.md)

## 2021-05-06

- [task-04: signal system (reactivity)](tasks/task-04-signal-system/README.md)

## 2021-05-02

- reference: [signal system design](ref/architecture/signal-system-design.md)

## 2021-04-20

- [task-03: layout engine (flexbox-like)](tasks/task-03-layout-engine/README.md)

## 2021-04-15

- reference: [layout engine - decisions and knowledge](ref/architecture/layout-engine.md)

## 2021-03-28

- [task-02: component state management](tasks/task-02-component-state-management/README.md)

## 2021-03-24

- reference: [component state design (task-02)](ref/architecture/component-design.md)

## 2021-03-10

- [task-01: view trait + viewcontext](tasks/task-01-view-trait-viewcontext/README.md)

## 2021-03-06

- reference: [view trait - design decisions](ref/architecture/view-trait-design.md)

## undated

- learning: [General learnings](learnings/general.md)
- program: [plev — Technical Implementation Plan](research/technical-plan.md)

## summary written when the changelog began

- responsiveness wave: content-driven layout in showcase, real text
  measurement in ide (heuristic deleted), percent dims, hidpi
  resize projection, touch-to-pointer synth
- web wave: showcase runs in the browser (webgpu, trunk); async gpu init;
  web gamma fixed via srgb view formats; pixel-identical to desktop (48,48,48)
- fidelity waves: rubik embedded, measured hoff tokens (#303030), desktop
  gamma fix, glass/backdrop blur, analytic shadows
- foundation: compositor with dirty layers, render-on-demand, rope,
  git, design system (~15 widgets), actions/keymap
- history purge: 417mb of accidental build artifacts removed with
  filter-repo before first push to the private remote (plevdev)
