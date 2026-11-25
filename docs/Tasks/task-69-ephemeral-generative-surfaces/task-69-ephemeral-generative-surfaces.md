---
id: task-69
title: "task-69: ephemeral and generated surfaces"
project: plev
type: task
status: done
date: 2026-09-24
date_basis: header
recorded: 2026-09-23
topics: [generated-surfaces, ir, agents]
relations: [adr-0023, adr-0051, task-58, task-59, task-63, task-68, task-70, task-71, experiment-generated-surface-checkpoint-b]
source: docs/log/steps/task-69-ephemeral-generative-surfaces.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-69: ephemeral and generated surfaces

Records the task that made interface surfaces be born from an intent: materialized from verified IR, operated by a human and an agent through the same contract, transformed by `SemId`, and dismissed when the task ends. The final result is in checkpoint b: the generated silences screen became usable in the default window, and the second round recorded a tie with different advantages, with no defeat for the generative approach.

## decisões do orquestrador (2026-09-24)

- **ordem das fases:** 1 (materializar) agora, depois 2 (ciclo de vida e
  transformação), 3 (proponentes) e 4 (a demo do vídeo). a fase 0 (spike
  em `refs/`) não roda. a pergunta de UX dela é respondida de verdade no
  checkpoint b, com a superfície gerada de verdade.
- **o host mora num crate novo, `surface`.** ele depende de `uir`, `sem`,
  `engine` e `comps`. o `comps` continua biblioteca de widgets e não
  depende do IR.
- **a projeção sai do IR, não do `project` dos widgets.** 9 dos 21
  componentes do vocabulário não têm `project` no comps (achado da
  task-68). a superfície é semântica por construção: role por
  `uir::role_of`, label, estado lido das ligações no snapshot vivo do app,
  e as ações que o IR ligou.
- **a fase 1 prova com as fixtures da task-68**, em testes sem janela e
  numa página do showcase. a demo do vídeo continua na fase 4, como o
  plano diz. o urnaui já usa ffmpeg em subprocesso, então o domínio do
  vídeo tem base no código. a fixture da revisão de silêncios nasce na
  fase 4, com as capacidades de verdade.
- **o provedor do LLM (fase 3) é decisão do brenner**, como o arquivo
  diz. a fase 3 para ali e pergunta.

## objetivo

uma superfície de interface nasce de uma intenção, é materializada a
partir de IR verificado, é operada por humano e agente pelo mesmo
contrato, se transforma sem perder identidade e desaparece quando a
tarefa termina. nunca foi codificada como tela.

## ligação com a tese

seções 1 (o ciclo invertido), 4 (a interface deixa de ser fixa: o editor
de vídeo, os silêncios, a comparação, o espectro de fixa até gerada sob
demanda no mesmo runtime), 5 (LLM como interface semântica, plev como
interface perceptual), 15 (restrições), 18 (GPU-first para superfícies que
existem por poucos segundos), 33 (uma interface pode existir por vinte
segundos e nunca ter sido tela). goal 1 e goal 8 do programa. checkpoint b.

## estado atual no código

- não existe host de superfície materializada; o padrão oficial é estado
  em structs com widgets retained ([`0023-official-app-pattern.md`](../../adr/Structure/0023-official-app-pattern/0023-official-app-pattern.md)).
- `rust/engine/src/compose/layout` (taffy) resolve layout a partir de árvore;
  `rust/engine/src/widgets` tem os widgets; `rust/engine/src/widgets/overlay` tem spring para
  entrada e saída.
- o urnaui já decodifica frames de mídia com ffmpeg em subprocesso
  (`apps/urnadbapp`, ADR `urnaui-media-frames-in-place-via-ffmpeg-subfile`).
  ffmpeg tem `silencedetect`.
- depois de 68: `VerifiedIr`, `Capabilities`.

## escopo

### fase 0: spike descartável em `refs/` (pode rodar desde a onda 4)

a revisão de silêncios montada à mão, com capacidades falsas, só para
testar a hipótese de UX cedo: uma superfície que aparece para uma tarefa é
melhor que a tela fixa? nada é integrado. entrega: o relatório do que foi
aprendido e o que muda nas tasks 68 e 69. segue a regra de `refs/` do
AGENTS.md.

### fase 1: materializador

- `SurfaceHost` (nome e lugar decididos no ADR: comps ou crate próprio):
  guarda as superfícies ativas; cada uma é um `VerifiedIr` mais as
  instâncias de widget retained indexadas por `SemId`.
- layout a partir do `Layout` do IR pelo `LayoutEngine` do engine (sem
  segundo motor de layout); tokens resolvidos contra o `Theme` atual.
- render pelos widgets do comps, com `SemId` e ligação (59).
- projeção semântica automática: a superfície é semântica por
  construção, porque o IR já carrega identidade, role, label e as ligações
  de estado e ação.
- eventos dos widgets viram `Request` pelo dispatcher (58, 59). a
  superfície nunca executa nada sozinha.

### fase 2: ciclo de vida e transformação

- criar, materializar, operar, transformar, dispensar.
- transformar = receber um novo `VerifiedIr` e reconciliar por `SemId`: o
  mesmo `SemId` mantém a instância do widget e o estado dela (um slider em
  arraste continua em arraste quando muda de painel); `SemId` novo nasce;
  `SemId` que sumiu sai com transição.
- transições de entrada, saída e movimento com `theme.duration` e o
  spring do overlay; reduced motion respeitado.

### fase 3: proponentes

- trait `Proposer`: recebe intenção (texto), `Capabilities`, o
  `Snapshot` atual e a superfície atual (se houver); devolve IR.
- proponente determinístico por regras, para teste e para demo sem rede.
- proponente LLM atrás do mesmo trait: o prompt leva o registro de
  capacidades e o schema; a resposta passa pelo verifier; os erros do
  verifier voltam para o modelo num número limitado de tentativas; IR
  inválido nunca chega ao runtime. o provedor (modelo local ou API) é
  decisão do brenner nesta task, registrada no ADR.
- o modelo propõe superfície; ação continua indo pelo dispatcher com o
  ator certo (humano ou agente), com precondição e receipt.

### fase 4: a demo do editor de vídeo

- capacidades: `detect_silence` (ffmpeg `silencedetect` real sobre um
  arquivo), `preview_segment`, `change_threshold`, `apply_cut`,
  `reject_cut`.
- "corta os silêncios desse vídeo e depois quero revisar os cortes" ->
  superfície de revisão (vídeo, contagem, previews, slider de intensidade,
  aplicar todos, revisar individualmente).
- "quero comparar a versão original com os cortes" -> a superfície se
  transforma em comparação lado a lado com as diferenças na timeline,
  mantendo identidade do que continua.
- terminada a comparação, a composição some.
- lugar: página do grupo Apps Demo do showcase primeiro; app próprio
  depois, se crescer.

### padrão de app

ADR atualizando `official-app-pattern`: superfície materializada a partir
de IR vira o segundo padrão suportado, ao lado de estado em structs com
widgets retained, com as regras de quando usar cada um.

## fora de escopo

transformações adaptativas sobre telas fixas e capacidades compostas
(70); importers (71).

## dependências

fase 0: nenhuma. fases 1 a 4: 68, 59 e 58 integradas.

## superfície sob responsabilidade

o host de superfícies (lugar no ADR), a página demo no showcase
(`apps/uigallery/src/views/apps/` com arquivo novo), o proponente,
`refs/` para a fase 0, propostas de ADR
[`0051-materialized-surfaces-from-verified-ir.md`](../../adr/Synthesis/0051-materialized-surfaces-from-verified-ir/0051-materialized-surfaces-from-verified-ir.md) e a atualização de
`official-app-pattern`.

## interfaces que consome

`VerifiedIr`, `Capabilities` (68), `Dispatcher`, `Request` (58), `SemId`
opcional e helpers dos widgets, `Compositor::bind` (59), `LayoutEngine`,
`Theme`.

## interfaces que pode alterar

as próprias. mudança em widget de comps só com aprovação.

## critérios de aceitação

- teste: materializar a fixture da revisão de silêncios produz um snapshot
  com os mesmos `SemId`, roles e ações do IR.
- teste: transformar revisão em comparação preserva a instância e o estado
  de todo `SemId` que continua.
- teste: IR inválido do proponente nunca materializa; os erros voltam.
- ação disparada numa superfície gerada tem receipt com o ator certo.
- medir o tempo de intenção até primeiro frame e de IR verificado até
  primeiro frame; registrar.
- demo gravada (task-63 se integrada) e roteiro no handoff.
- checkpoint b: a superfície gerada comparada com uma tela fixa bem
  desenhada para a mesma tarefa, com o brenner, resultado escrito.
- `./tool/tasks/fullcheck.sh` verde.

## testes esperados

materialização, reconciliação por `SemId`, dispensa, proponente
determinístico, loop de verificação do proponente LLM com um cliente falso,
viewports estreito e largo da demo.

## riscos e perguntas abertas

- latência de modelo: a superfície pode chegar tarde para a tarefa; a
  medição decide se o proponente precisa de cache, de modelo local ou de
  proposta parcial.
- usabilidade: a hipótese de que superfície gerada é melhor pode falhar.
  é para isso que existem a fase 0 e o checkpoint b.

## commits esperados

1. `task-69: spike da revisão de silêncios em refs` (relatório; o código
   fica em refs/)
2. `task-69: host de superfícies e materialização a partir de VerifiedIr`
3. `task-69: reconciliação por SemId e transições`
4. `task-69: proponente determinístico`
5. `task-69: proponente LLM com loop de verificação`
6. `task-69: capacidades do editor de vídeo com silencedetect`
7. `task-69: demo de revisão e comparação no showcase`

## pare e reporte se

- o layout do IR não mapear no `LayoutEngine` sem motor novo;
- a reconciliação exigir estado de widget que o comps não expõe;
- a escolha de provedor de LLM (sempre decisão do brenner).

## Phases

1. [phase 1: surface materializer](phase-1.md): the `surface` host materializes from `VerifiedIr`; the Surface page shows the hand-written whole and the materialized one.
2. [phase 2: transformation by SemId](phase-2.md): reconciliation by `SemId`, transitions and dismissal; the text, the caret and the drag survive when the widget is the same.
3. [phase 3a: proposers without a provider](phase-3a.md): the `Proposer` trait, the rule-based proposer and the model loop behind `Model`, proven with a fake model.
4. [phase 3b-i: Claude as the proposer's model](phase-3b-i.md): Claude through `claude -p` as a process, in the `claude-model` crate; the example's real runs.
5. [phase 3b-ii: the page asks Claude for the screen](phase-3b-ii.md): choice of proposer, request off the app thread, visible waiting and cancellation.
6. [phase 4a: the silences app and the fixed screen](phase-4a.md): pure model, `silencedetect` parser, ffmpeg worker and the review's fixed screen.
7. [phase 4b: the generated silences screen](phase-4b.md): the generated screen with real frames through the `Frames` trait, the comparison and the wrap-up.
8. [phase 4c: recording and the recorded demo](phase-4c.md): the showcase records the session the way the ide does; the video editor demo lives in two `.monster` files.
9. [checkpoint b, first round](checkpoint-b-round-1.md): the orchestrator's comparison between the fixed screen and the generated one; both versions come out worse to use.
10. [checkpoint b, the corrections](checkpoint-b-fixes.md): line, slider, number and prompt fixed; the regression check now stages the generated screen.
11. [checkpoint b, second round](checkpoint-b-round-2.md): the same comparison with the corrected binary; the generated one stopped being worse.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0023: official app pattern: state in structs, retained widgets, explicit invalidation](../../adr/Structure/0023-official-app-pattern/0023-official-app-pattern.md)
- [ADR 0033: actions carry the state their actor observed](../../adr/Semantics/0033-actions-carry-observed-state/0033-actions-carry-observed-state.md)
- [ADR 0036: a commit checks the index it observed](../../adr/Semantics/0036-commit-checks-the-index-it-observed/0036-commit-checks-the-index-it-observed.md)
- [ADR 0037: session recording: replay, not re-execution](../../adr/Recording/0037-session-recording-replay-not-reexecution/0037-session-recording-replay-not-reexecution.md)
- [ADR 0049: the plev IR is typed, references only declared capabilities, and runs only once verified](../../adr/Synthesis/0049-plev-ir-typed-and-verified/0049-plev-ir-typed-and-verified.md)
- [ADR 0051: a surface materializes only from verified IR, projects from the IR and asks through the app's dispatcher](../../adr/Synthesis/0051-materialized-surfaces-from-verified-ir/0051-materialized-surfaces-from-verified-ir.md)
- [ADR 0052: proposers propose, the runtime verifies](../../adr/Synthesis/0052-proposers-propose-the-runtime-verifies/0052-proposers-propose-the-runtime-verifies.md)
- [ADR 0053: the silences app is the fixed screen the generated one is compared against](../../adr/Structure/0053-the-silences-app-is-the-fixed-screen/0053-the-silences-app-is-the-fixed-screen.md)
- [ADR 0067: adaptive transformations are small verified operations that preserve identity](../../adr/Synthesis/0067-adaptive-transforms-preserve-identity/0067-adaptive-transforms-preserve-identity.md)
- [ADR 0069: importers target plev IR, with a report in four categories](../../adr/Importing/0069-importers-target-plev-ir/0069-importers-target-plev-ir.md)
- [ADR 0070: generative runtime results](../../adr/Synthesis/0070-generative-runtime-results/0070-generative-runtime-results.md)
- [what the autonomous cycle taught: a surface materializes only from verified IR, projects from the IR and asks through the app's dispatcher](../../adr/Synthesis/0051-materialized-surfaces-from-verified-ir/learned-materialized-surfaces-from-verified-ir.md)
- [what the autonomous cycle taught: proposers propose, the runtime verifies](../../adr/Synthesis/0052-proposers-propose-the-runtime-verifies/learned-proposers-propose-the-runtime-verifies.md)
- [what the autonomous cycle taught: the silences app is the fixed screen the generated one is compared against](../../adr/Structure/0053-the-silences-app-is-the-fixed-screen/learned-the-silences-app-is-the-fixed-screen.md)
- [what the autonomous cycle taught: adaptive transformations are small verified operations that preserve identity](../../adr/Synthesis/0067-adaptive-transforms-preserve-identity/learned-adaptive-transforms-preserve-identity.md)
- [task-58: action with precondition, dispatch and receipts](../task-58-actions-preconditions-receipts/task-58-actions-preconditions-receipts.md)
- [task-59: semantic binding with the render and one operational path](../task-59-semantic-binding-and-one-path/task-59-semantic-binding-and-one-path.md)
- [task-63: session flight recorder](../task-63-flight-recorder/task-63-flight-recorder.md)
- [task-68: plev IR and verifier](../task-68-plev-ir-and-verifier/task-68-plev-ir-and-verifier.md)
- [task-70: adaptive transformations](../task-70-adaptive-transformations/task-70-adaptive-transformations.md)
- [task-71: plev as compiler target (importers to IR)](../task-71-compiler-target/task-71-compiler-target.md)
- [task-73: incremental dependency system](../task-73-incremental-dependency-system/task-73-incremental-dependency-system.md)
- [task-80: proof of qualitatively different software](../task-80-proof-qualitatively-different-software/task-80-proof-qualitatively-different-software.md)
- [task-95: the todo projects its items as a list](../task-95-the-todo-projects-its-items-as-a-list/task-95-the-todo-projects-its-items-as-a-list.md)
- [task-96: a generated surface in the ide](../task-96-ide-generated-surface/task-96-ide-generated-surface.md)
- [task-97: code and diff in the IR vocabulary](../task-97-code-and-diff-in-the-ir/task-97-code-and-diff-in-the-ir.md)
- [task-64 phase c: session to test, and export](../task-64-session-inspector-and-tests/phase-c-session-to-test.md)
- [task-68 implementation phase: probe, handoff and close-out](../task-68-plev-ir-and-verifier/phase-implementation.md)
- [task-69, checkpoint b: the corrections](checkpoint-b-fixes.md)
- [task-69, checkpoint b: first round](checkpoint-b-round-1.md)
- [task-69, checkpoint b: second round](checkpoint-b-round-2.md)
- [task-69, phase 1: surface materializer](phase-1.md)
- [task-69, phase 2: transformation by SemId](phase-2.md)
- [task-69, phase 3a: proposers without a provider](phase-3a.md)
- [task-69, phase 3b-i: Claude as the proposer's model](phase-3b-i.md)
- [task-69, phase 3b-ii: the page asks Claude for the screen](phase-3b-ii.md)
- [task-69, phase 4a: the silences app and the fixed screen](phase-4a.md)
- [task-69, phase 4b: the generated screen of the silences](phase-4b.md)
- [task-69, phase 4c: recording and the recorded demo](phase-4c.md)
- [task-70: phase 1, the operations, the inverse, the verification and the host](../task-70-adaptive-transformations/phase-1-operations.md)
- [task-70: phase 2, the transformation proposer](../task-70-adaptive-transformations/phase-2-proposer.md)
- [task-71 phase 1: probe and handoff of the plev IR emitter](../task-71-compiler-target/phase-1-ir-emitter.md)
- [task-80, demo 5: generative UI in a real application](../task-80-proof-qualitatively-different-software/demo-5-generative-ui.md)
- [task-80, phase 1: probe, five-demo method, demos 3 and 4 and handoff](../task-80-proof-qualitatively-different-software/phase-1-probe-and-demos.md)
- [task-96, phase a: the generated surface in the ide (probe, handoff and close-out)](../task-96-ide-generated-surface/phase-a-surface.md)
- [generated silences screen against the fixed screen, checkpoint b of task-69](../../Research/Exploring/generated-surface-checkpoint-b.md)
- [plev: thesis of the generative interaction runtime](../../Research/Synthesis/generative-runtime-thesis.md)
- [program: from GPU UI engine to generative interaction runtime](../../Research/Synthesis/generative-runtime-program.md)
- [how to code against the plev engine](../../Research/Workflows/code-against-the-plev-engine.md)
- [autonomous cycle journal](../../Research/Workflows/autonomous-cycle-journal.md)
<!-- /related -->
