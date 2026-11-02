---
id: task-75
title: "task-75: production-ready platform accessibility"
project: plev
type: task
status: in-progress
date: 2026-09-25
date_basis: header
recorded: 2026-09-23
topics: [accessibility, platform, input, semantics]
relations: [adr-0068, task-18, task-19, task-57, task-58, task-59, task-60, task-70, task-74, learning-generative-runtime]
evidence: [d811876f, f3e9f914, 3badee06, 917cdd2a, 2a19ad66]
source: docs/log/steps/task-75-platform-accessibility.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# task-75: production-ready platform accessibility

Records the task that makes the accessibility designed by task-60 work with each platform's screen readers and with full keyboard support on the component pages. Phase 1 showed that only macOS runs with VoiceOver today, and the decision record for this task is [ADR 0068](../../adr/Interface/0068-accessibility-is-production-ready-per-platform/0068-accessibility-is-production-ready-per-platform.md). Phases 2a to 2d fixed the accessibility tree of the app and component pages, projected the 30 component pages with projectable widgets, and brought keyboard support to 29 of the 30 pages, with the gate green in each phase according to the handoffs. Still open are the walkthrough with the real screen reader, which Brenner runs, the decision on the 102 contrast pairs below AA, and the pending keyboard contract items, so the task remains in progress.

## objetivo

a accessibility que a task-60 projeta do contrato passa a funcionar bem
com os leitores de tela reais de cada plataforma, com navegação de teclado
completa e com as semânticas de foco e ação que um produto grande exige
para poder adotar o plev.

## ligação com a tese

seção 21 (esse ponto sozinho mata frameworks alternativos; estar presente
e estar production-ready são coisas diferentes; VoiceOver, NVDA, TalkBack,
keyboard navigation, focus semantics e action semantics são cidadãos do
runtime).

## estado atual no código

- depois da 60: árvore AccessKit a partir do snapshot, `NodeId` estável,
  ações do leitor pelo dispatcher, ordem de foco semântica, ponte no
  showcase, verificado com VoiceOver no macOS.
- AccessKit 0.24 com `accesskit_winit` 0.32. alvos do plev: macOS e web
  shipando; android e ios em andamento (tasks 18 e 19); linux e windows
  pendentes.
- web: AccessKit não tem adapter para navegador.
- recipe de focus ring em `rust/engine/src/widgets/recipe.rs`; reduced motion e
  contraste wcag declarados como requisitos nativos no AGENTS.md.

## escopo

1. matriz por plataforma, com o suporte real do AccessKit verificado (não
   presumido): VoiceOver no macOS e no iOS, TalkBack no Android, NVDA e
   Narrator no Windows, Orca no Linux (AT-SPI). onde o adapter não existir
   ou não servir, o registro diz isso.
2. web: decisão por ADR entre um espelho DOM oculto com ARIA gerado a
   partir do snapshot e nenhuma accessibility no web por enquanto; se o
   espelho for o caminho, ele é outra projeção do mesmo snapshot, nunca uma
   segunda fonte.
3. teclado: Tab e Shift-Tab pela ordem semântica, setas em listas, menus,
   tabs e árvores (padrões WAI-ARIA), trap de foco em modal e retorno do
   foco ao fechar, atalhos anunciados.
4. live regions: toast e status anunciados sem roubar foco.
5. reduced motion (a preferência do sistema desliga transições e springs)
   e contraste (verificação dos pares de token de texto e fundo contra
   wcag AA nos temas).
6. roteiros manuais por plataforma, executados e com resultado registrado
   neste arquivo ou num anexo.

## fora de escopo

accessibility de texto rico (74, fase 9); bring-up de linux e windows
como alvo (arc: pendente; esta task só entra lá quando o alvo existir).

## dependências

60 integrada; por plataforma, o alvo precisa rodar.

## superfície sob responsabilidade

`rust/engine/src/control/accessibility/**`, os shells dos apps na parte da
ponte, `rust/engine/src/widgets` na parte de teclado e foco dos widgets, testes de
contraste em `rust/engine/src/styling/theme` (só testes), ADRs por decisão.

## interfaces que consome

a ponte e o construtor de árvore (60), `Snapshot` (57), dispatcher (58).

## interfaces que pode alterar

`rust/engine/src/control/accessibility` por inteiro; widgets de comps na parte de
teclado com aprovação.

## critérios de aceitação

- para cada plataforma com alvo rodando: roteiro executado com o leitor
  real sobre o showcase (todo, formulários, modal, tabs, lista) e o ide,
  resultado registrado.
- testes automáticos de árvore para toda página do showcase.
- teste de contraste dos tokens de texto nos temas.
- teclado completo nas páginas de componente, com teste.
- `./tool/tasks/fullcheck.sh` verde.

## testes esperados

asserções de árvore por página, navegação de teclado simulada, contraste,
reduced motion desligando animação.

## riscos e perguntas abertas

- suporte do AccessKit por plataforma muda por versão; atualizar a versão é
  decisão com aprovação.
- teste manual não roda em CI; o registro é a evidência.

## commits esperados

um grupo por plataforma e por item (teclado, live regions, reduced motion,
contraste), com `task-75:` no início.

## pare e reporte se

- o web exigir o espelho DOM (decisão por ADR);
- um leitor real se comportar diferente do que a árvore promete.

## Phases

1. [phase 1: the probe and the manual walkthroughs](phase-1-probe-and-scripts.md): the AccessKit matrix per platform from the locked source, the page tree, live regions, contrast, reduced motion and the VoiceOver walkthrough.
2. [phase 2a: app pages that already project](phase-2a-app-pages.md): unreachable actions turned into controls inside the item, projected focus, status with a message, toast, modal and menu in the view's projection.
3. [phase 2b: component pages](phase-2b-component-pages.md): the `Component` trait, the projection of the 30 pages with projectable widgets, the 46 pages without projection with the reason for each, and the 60 baselines.
4. [phase 2c: keyboard on the component pages](phase-2c-keyboard.md): Tab in projection order, arrows per widget, Enter and Space as the person's request, modal with focus trap and focus return.
5. [phase 2d: lists and tables with keyboard](phase-2d-lists-and-tables.md): lists and tables with one focusable row at a time, the keyboard on 29 of the 30 pages, and the fix that returned tabs and tree to phase 2c.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0038: accessibility projects the semantic snapshot](../../adr/Interface/0038-accessibility-projects-the-semantic-snapshot/0038-accessibility-projects-the-semantic-snapshot.md)
- [ADR 0051: a surface materializes only from verified IR, projects from the IR and asks through the app's dispatcher](../../adr/Synthesis/0051-materialized-surfaces-from-verified-ir/0051-materialized-surfaces-from-verified-ir.md)
- [ADR 0067: adaptive transformations are small verified operations that preserve identity](../../adr/Synthesis/0067-adaptive-transforms-preserve-identity/0067-adaptive-transforms-preserve-identity.md)
- [ADR 0068: accessibility is production-ready per platform](../../adr/Interface/0068-accessibility-is-production-ready-per-platform/0068-accessibility-is-production-ready-per-platform.md)
- [ADR 0076: the keyboard walks the projection](../../adr/Interface/0076-the-keyboard-walks-the-projection/0076-the-keyboard-walks-the-projection.md)
- [ADR 0086: the adaptive foundation reads preferences once into the theme](../../adr/Interface/0086-the-adaptive-foundation-reads-preferences-once-into-the-theme/0086-the-adaptive-foundation-reads-preferences-once-into-the-theme.md)
- [what the autonomous cycle taught: accessibility is production-ready per platform](../../adr/Interface/0068-accessibility-is-production-ready-per-platform/learned-accessibility-is-production-ready-per-platform.md)
- [what the autonomous cycle taught about the way of working](../../Research/Workflows/autonomous-cycle-learnings.md)
- [Generative runtime session: what worked and what bit us](../../adr/Synthesis/0070-generative-runtime-results/generative-runtime.md)
- [task-104: the 35 empty or borrowed gallery tabs become kit components](../task-104-gallery-components-from-the-kit/task-104-gallery-components-from-the-kit.md)
- [task-18: android build & device test](../task-18-android-device-test/task-18-android-device-test.md)
- [task-19: ios simulator test](../task-19-ios-simulator-test/task-19-ios-simulator-test.md)
- [task-42: plev event dispatch + gitbutler UI port](../task-42-plev-event-dispatch/task-42-plev-event-dispatch.md)
- [task-57: semantic contract (crate `sem`)](../task-57-semantic-contract/task-57-semantic-contract.md)
- [task-58: action with precondition, dispatch and receipts](../task-58-actions-preconditions-receipts/task-58-actions-preconditions-receipts.md)
- [task-59: semantic binding with the render and one operational path](../task-59-semantic-binding-and-one-path/task-59-semantic-binding-and-one-path.md)
- [task-60: accessibility as a projection of the contract](../task-60-accessibility-projection/task-60-accessibility-projection.md)
- [task-70: adaptive transformations](../task-70-adaptive-transformations/task-70-adaptive-transformations.md)
- [task-74: text and editing](../task-74-text-and-editing/task-74-text-and-editing.md)
- [task-86: the ide people use opens the agent port](../task-86-the-ide-opens-the-agent-port/task-86-the-ide-opens-the-agent-port.md)
- [task-97: code and diff in the IR vocabulary](../task-97-code-and-diff-in-the-ir/task-97-code-and-diff-in-the-ir.md)
- [task-69, checkpoint b: the corrections](../task-69-ephemeral-generative-surfaces/checkpoint-b-fixes.md)
- [task-69, phase 2: transformation by SemId](../task-69-ephemeral-generative-surfaces/phase-2.md)
- [task-70: task closure, by the orchestrator](../task-70-adaptive-transformations/closing.md)
- [task-70: phase 4, accessibility as a transformation](../task-70-adaptive-transformations/phase-4-accessibility.md)
- [task-75 phase 1: the probe and the manual test scripts](phase-1-probe-and-scripts.md)
- [task-75 phase 2a: app pages that already project](phase-2a-app-pages.md)
- [task-75 phase 2b: component pages](phase-2b-component-pages.md)
- [task-75 phase 2c: keyboard on the component pages](phase-2c-keyboard.md)
- [task-75 phase 2d: lists and tables with keyboard](phase-2d-lists-and-tables.md)
- [plev: thesis of the generative interaction runtime](../../Research/Synthesis/generative-runtime-thesis.md)
- [program: from GPU UI engine to generative interaction runtime](../../Research/Synthesis/generative-runtime-program.md)
- [operating mode: autonomous orchestrator](../../Research/Workflows/autonomy.md)
- [how to build and serve the web target](../../Research/Workflows/build-and-serve-the-web-target.md)
- [autonomous cycle journal](../../Research/Workflows/autonomous-cycle-journal.md)
<!-- /related -->
