---
id: task-96
title: "task-96: a generated surface in the ide"
project: plev
type: task
status: done
date: 2026-09-25
date_basis: header
recorded: 2026-09-25
topics: [generated-surfaces, ir, agents, llm]
relations: [task-63, task-64, task-68, task-69, task-70, task-80, task-85, task-93, task-97, experiment-five-demos]
evidence: [73a318a7, 6bacaa3d, f53667c0, 73704549, f8f39298, e315e11e, 2e6c9e9a, 07650665, 9dadd352, 855b1a4d, 3df1297f, 205e90b3, 1283cc5b, e71222cf, 3959d958, 4be4bb2f, e42bd571, a427d2b8]
source: docs/log/steps/task-96-ide-generated-surface.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-96: a generated surface in the ide

The task puts a generated surface (`SurfaceHost`) in the ide window, beside the fixed views, to review three commits. The intent "review these three commits" generates, through the rules, a verified surface that the person operates, and each act passes through the workbench's single dispatcher. The result is task-80's demo 5 in a real application, measured against the fixed panel: the surface shows the files better and the diff worse. Why it matters: it is the test of the unified-contract hypothesis in an application that already exists, not only in the video demo. Phase b fixes the title of the diff card and the column of the intent line; the proof with the real Claude and demo 5 with a window are left to the orchestrator.

## objetivo

a demo 5 da task-80 (UI generativa numa aplicação real) pede uma
intenção num app que já existe gerando uma superfície verificada que a
pessoa opera. hoje as superfícies geradas vivem só nas páginas Surface e
Silences do showcase: nem o ide nem o urnaui têm `SurfaceHost` (sonda da
task-80 fase 1, item (a)). esta task põe uma no ide.

## ligação com a tese

seção 28 (a hipótese é o contrato unificado, provada em aplicação real,
não só na demo de vídeo); goal 10 do programa; a demo 5 da task-80.

## estado atual no código

- `rust/screen` (`SurfaceHost`, `Surface::route`) e `rust/offers`
  (regras e `Llm`) servem a Silences e a página Surface do showcase.
- o `uir` já verifica IR no domínio do ide contra o registro que a
  projeção do ide dá: `rust/plevir/tests/fixtures/idestudio-review-commit.ir.json`,
  `ide-commit-form.ir.json`, `ide-workbench.snapshot.json`, escrito por
  `apps/idestudio/tests/plevir_fixture.rs`.
- `apps/idestudio/Cargo.toml` não depende de `surface`, `propose` nem `uir`.

## escopo

1. um `SurfaceHost` na janela do ide ao lado das vistas fixas; a linha de
   intenção (campo, proponente, propor, fechar) projetada como na
   Silences.
2. o registro de capacidades tirado do `Workbench::snapshot`; os atos da
   superfície roteados ao dispatcher único do workbench
   (`Surface::route`), com os receipts e a gravação do ide como estão.
3. as regras propondo "review these commits" sobre `stacks`; o Claude
   pelo `propose::Llm` fora da thread do app (a prova com o Claude de
   verdade é do orquestrador).
4. o laboratório grava a demo 5 sem janela, pelo roteiro e o critério
   escritos na task-80.

## fora de escopo

o urnaui; transformar as vistas fixas do ide (elas se exprimem em IR,
pela decisão da task-70 fase 1); qualquer chamada real ao Claude por um
agente.

## dependências

68, 69, 70, 63d e 64 integradas.

## superfície sob responsabilidade

`apps/idestudio/src/**` (a janela e a projeção), `apps/idestudio/Cargo.toml`
(três dependências de workspace: `surface`, `propose`, `uir`; nenhuma
externa), testes do ide, e a demo 5 no arquivo da task-80.

## critérios de aceitação

- "review these three commits" gera, pelas regras, uma superfície
  verificada que mostra os três commits com os arquivos e o diff de cada
  um, e que a pessoa opera.
- cada ato da superfície passa pelo dispatcher do workbench, com o
  receipt de quem pediu; nada da superfície fica depois de fechar.
- a demo 5 medida contra o painel de stacks fixo mais o diff, pelo
  critério escrito na task-80.
- `./tool/tasks/fullcheck.sh` verde.

## o que a derruba

a superfície pior que o painel fixo para revisar três commits; um ato
fora do dispatcher; a superfície deixando rastro.

## pare e reporte se

- o ide precisar de dependência externa nova;
- a projeção do ide não der o registro que a superfície precisa sem
  mudar tipo do `sem`.

## Phases

1. [phase a (2026-09-25): the generated surface in the ide, with probe, handoff and closing](phase-a-surface.md): the reading probe before the code, the handoff with the commits of branch `task/96a-ide-surface`, and the closing that records the phase as integrated.
2. [phase b (2026-09-25): the diff card title and the surface-only column](phase-b-title-and-column.md): the probe of the fix, the diff card with the label "diff", the intent line in a strip below the fixed views, demo 5 recorded again, and the handoff of the phase.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0049: the plev IR is typed, references only declared capabilities, and runs only once verified](../../adr/Synthesis/0049-plev-ir-typed-and-verified/0049-plev-ir-typed-and-verified.md)
- [ADR 0070: generative runtime results](../../adr/Synthesis/0070-generative-runtime-results/0070-generative-runtime-results.md)
- [what the autonomous cycle taught: generative runtime results](../../adr/Synthesis/0070-generative-runtime-results/learned-generative-runtime-results.md)
- [task-63: session flight recorder](../task-63-flight-recorder/task-63-flight-recorder.md)
- [task-64: session inspector, bug report and session for testing](../task-64-session-inspector-and-tests/task-64-session-inspector-and-tests.md)
- [task-68: plev IR and verifier](../task-68-plev-ir-and-verifier/task-68-plev-ir-and-verifier.md)
- [task-69: ephemeral and generated surfaces](../task-69-ephemeral-generative-surfaces/task-69-ephemeral-generative-surfaces.md)
- [task-70: adaptive transformations](../task-70-adaptive-transformations/task-70-adaptive-transformations.md)
- [task-80: proof of qualitatively different software](../task-80-proof-qualitatively-different-software/task-80-proof-qualitatively-different-software.md)
- [task-85: the commit that the receipt announces is already in the projection, with the files](../task-85-the-commit-a-receipt-announces-is-projected/task-85-the-commit-a-receipt-announces-is-projected.md)
- [task-93: the person in the lab answers](../task-93-the-lab-person-answers/task-93-the-lab-person-answers.md)
- [task-97: code and diff in the IR vocabulary](../task-97-code-and-diff-in-the-ir/task-97-code-and-diff-in-the-ir.md)
- [task-80, demo 5: generative UI in a real application](../task-80-proof-qualitatively-different-software/demo-5-generative-ui.md)
- [task-80, phase 1: probe, five-demo method, demos 3 and 4 and handoff](../task-80-proof-qualitatively-different-software/phase-1-probe-and-demos.md)
- [task-96, phase a: the generated surface in the ide (probe, handoff and close-out)](phase-a-surface.md)
- [task-96, phase b: the diff card title and the column shown only with a surface](phase-b-title-and-column.md)
- [five demos of qualitatively different software, each with its method written before it ran](../../Research/Exploring/five-demos.md)
- [program: from GPU UI engine to generative interaction runtime](../../Research/Synthesis/generative-runtime-program.md)
- [operating mode: autonomous orchestrator](../../Research/Workflows/autonomy.md)
- [autonomous cycle journal](../../Research/Workflows/autonomous-cycle-journal.md)
<!-- /related -->
