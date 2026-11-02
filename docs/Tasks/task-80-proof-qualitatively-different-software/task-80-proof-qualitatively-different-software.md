---
id: task-80
title: "task-80: proof of qualitatively different software"
project: plev
type: task
status: in-progress
date: 2026-09-25
date_basis: header
recorded: 2026-09-23
topics: [generated-surfaces, agents, recording, research]
relations: [adr-0050, adr-0070, task-60, task-62, task-63, task-64, task-68, task-69, task-70, task-84, task-86, task-87, task-89, task-92, task-93, task-94, task-96, task-97, generative-runtime-thesis, experiment-five-demos]
evidence: [e44e4234, c8a6c3a9, f008c897, 46f272d3, b26dc988, ae5be30e, 677b1ad9]
source: docs/log/steps/task-80-proof-qualitatively-different-software.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-80: proof of qualitatively different software

Task-80 gathers five demonstrations that the model produces qualitatively different software, and records where they stand as of 2026-09-25. Phase 1 was delivered to the orchestrator on that date, with the code probe, the method for the five demos written before running, demo 3 (a real bug from task-84 recorded without a window, read at the moment by plev-inspect and turned into a regression test) and demo 4 (the ephemeral interface measured by the recording); demo 5 was done by task-96. Demos 1 and 2 have partial proofs, and the core question and the outside person remain open, so the task continues in progress. It matters because the program itself defines that, if the demos do not sustain the direction, it is reassessed with the data, not defended.

## objetivo

provar que o modelo produz software qualitativamente diferente, não só
mais rápido: interfaces efêmeras, handoff humano-agente, stale-state
rejection, inspectability e UI generativa, em aplicações reais, com
método, com números onde números cabem e com a pergunta do núcleo
respondida por quem usa.

## ligação com a tese

goal 10 do programa. tese seções 28 (não afirmar primeiro; a hipótese é o
contrato unificado; a frase "interfaces que humanos e agentes operam, e que
qualquer pessoa pode inspecionar depois"; a pergunta do núcleo), 33 (o que
o plev pode se tornar), e o experimento do arquivo de concepção.

## estado atual no código

depende das entregas: 62 (ide com rejeição e handoff, experimento com
métricas), 63 e 64 (sessão gravada e inspecionada, bug report, sessão para
teste), 69 (revisão de silêncios e comparação geradas), 70 (transformação
adaptativa e capacidade composta).

## escopo

cinco demonstrações, cada uma com roteiro, gravação (`.monster`) e
resultado escrito:

1. stale-state rejection: o experimento do ide completo (task-62, fase c),
   com as métricas (sucesso, tempo, dados e tokens para o agente,
   recuperação de erro, custo de gravação) e as comparações contra
   screenshot e contra a alternativa estruturada existente.
2. handoff humano-agente: o agente prepara um commit, o brenner assume,
   muda parte, devolve, o agente termina; o receipt de cada passo diz quem
   fez.
3. inspectability: um bug real (ou provocado) enviado como trecho de
   sessão, aberto no inspector no instante exato, com as perguntas da tese
   respondidas na tela, e virando teste de regressão que pega a quebra.
4. interface efêmera: a revisão de silêncios nasce da intenção, vira
   comparação, some; nunca existiu como tela.
5. UI generativa em aplicação real: além da demo de vídeo, uma intenção num
   app que já existe (no urnaui, "compara esses dois resultados de busca";
   ou no ide, "revisa esses três commits") gera uma superfície verificada
   que o usuário opera.

método:

- cada demo tem uma alternativa de controle (tela fixa bem feita; agente
  por screenshot; agente por snapshot de accessibility de terceiros) e o
  critério de comparação escrito antes.
- a pergunta do núcleo, com o brenner: ao abrir o primeiro `.monster` de uma
  delegação real, o que ele consegue descobrir, e isso basta para confiar
  na delegação? a resposta é escrita com as palavras dele.

critérios de parada, herdados do programa: se a rejeição e os receipts não
mudarem como a delegação é feita, ou se as superfícies geradas forem piores
que as fixas nas mesmas tarefas, a direção é reavaliada com os dados, não
defendida.

## fora de escopo

marketing, texto de blog (o destino blog passa pela skill de escrita, em
outra sessão), benchmarks isolados sem demonstração.

## dependências

62, 64, 69 e 70 integradas.

## superfície sob responsabilidade

roteiros e resultados em `docs/log/`, gravações de sessão (fora
do git se forem grandes), páginas Apps Demo do showcase para as demos que
couberem lá, ADR de resultado [`0070-generative-runtime-results.md`](../../adr/Synthesis/0070-generative-runtime-results/0070-generative-runtime-results.md).

## interfaces que consome

tudo do programa, sem alterar.

## interfaces que pode alterar

nenhuma. defeito encontrado vira task nova ou volta para a task dona.

## critérios de aceitação

- as cinco demonstrações executadas, gravadas, com resultado escrito.
- a comparação de cada uma contra a alternativa de controle, com o
  critério definido antes.
- a pergunta do núcleo respondida.
- a decisão do checkpoint c registrada no programa.

## testes esperados

as demos que viram teste (a 3 obrigatoriamente) entram no gate.

## riscos e perguntas abertas

- viés de quem constrói avaliando o que construiu: sempre que possível,
  uma pessoa de fora usa as demos e o relato dela entra no resultado.

## commits esperados

um por demonstração com roteiro e resultado, `task-80:` no início, e o ADR
de resultado.

## pare e reporte se

- uma demo exigir funcionalidade que nenhuma task entregou (vira task, não
  atalho).

## Phases

1. [phase 1: probe, method for the five demos, demos 3 and 4, and handoff](phase-1-probe-and-demos.md): the code probe, the method written before running, what had already been proven, demos 3 and 4, and the handoff of 2026-09-25.
2. [demo 5: generative UI in a real application](demo-5-generative-ui.md): the surface generated in the ide, built by task-96 and measured with the IR `code` by task-97 and by phase b of task-96.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0032: semantic contract as a pure projection of app state](../../adr/Semantics/0032-semantic-contract-as-pure-projection/0032-semantic-contract-as-pure-projection.md)
- [ADR 0049: the plev IR is typed, references only declared capabilities, and runs only once verified](../../adr/Synthesis/0049-plev-ir-typed-and-verified/0049-plev-ir-typed-and-verified.md)
- [ADR 0050: sessions become regression tests](../../adr/Recording/0050-sessions-become-regression-tests/0050-sessions-become-regression-tests.md)
- [ADR 0051: a surface materializes only from verified IR, projects from the IR and asks through the app's dispatcher](../../adr/Synthesis/0051-materialized-surfaces-from-verified-ir/0051-materialized-surfaces-from-verified-ir.md)
- [ADR 0070: generative runtime results](../../adr/Synthesis/0070-generative-runtime-results/0070-generative-runtime-results.md)
- [what the autonomous cycle taught: sessions become regression tests](../../adr/Recording/0050-sessions-become-regression-tests/learned-sessions-become-regression-tests.md)
- [what the autonomous cycle taught: generative runtime results](../../adr/Synthesis/0070-generative-runtime-results/learned-generative-runtime-results.md)
- [task-60: accessibility as a projection of the contract](../task-60-accessibility-projection/task-60-accessibility-projection.md)
- [task-62: proof in the ide: stale-state rejection and human-agent handoff](../task-62-ide-human-agent-proof/task-62-ide-human-agent-proof.md)
- [task-63: session flight recorder](../task-63-flight-recorder/task-63-flight-recorder.md)
- [task-64: session inspector, bug report and session for testing](../task-64-session-inspector-and-tests/task-64-session-inspector-and-tests.md)
- [task-68: plev IR and verifier](../task-68-plev-ir-and-verifier/task-68-plev-ir-and-verifier.md)
- [task-69: ephemeral and generated surfaces](../task-69-ephemeral-generative-surfaces/task-69-ephemeral-generative-surfaces.md)
- [task-70: adaptive transformations](../task-70-adaptive-transformations/task-70-adaptive-transformations.md)
- [task-84: an old refresh does not undo what the ide has already accepted](../task-84-ide-refresh-never-goes-back/task-84-ide-refresh-never-goes-back.md)
- [task-86: the ide people use opens the agent port](../task-86-the-ide-opens-the-agent-port/task-86-the-ide-opens-the-agent-port.md)
- [task-87: a terminal stage carries the revision the app was left in](../task-87-a-terminal-stage-carries-the-revision/task-87-a-terminal-stage-carries-the-revision.md)
- [task-89: an act relies only on what was shown to the agent](../task-89-an-act-rests-only-on-what-was-shown/task-89-an-act-rests-only-on-what-was-shown.md)
- [task-92: the agent asks the person](../task-92-the-agent-asks-the-person/task-92-the-agent-asks-the-person.md)
- [task-93: the person in the lab answers](../task-93-the-lab-person-answers/task-93-the-lab-person-answers.md)
- [task-94: the refusal that names a person points to the question](../task-94-the-refusal-points-at-the-question/task-94-the-refusal-points-at-the-question.md)
- [task-96: a generated surface in the ide](../task-96-ide-generated-surface/task-96-ide-generated-surface.md)
- [task-97: code and diff in the IR vocabulary](../task-97-code-and-diff-in-the-ir/task-97-code-and-diff-in-the-ir.md)
- [task-69, checkpoint b: second round](../task-69-ephemeral-generative-surfaces/checkpoint-b-round-2.md)
- [task-80, demo 5: generative UI in a real application](demo-5-generative-ui.md)
- [task-80, phase 1: probe, five-demo method, demos 3 and 4 and handoff](phase-1-probe-and-demos.md)
- [task-96, phase a: the generated surface in the ide (probe, handoff and close-out)](../task-96-ide-generated-surface/phase-a-surface.md)
- [task-96, phase b: the diff card title and the column shown only with a surface](../task-96-ide-generated-surface/phase-b-title-and-column.md)
- [five demos of qualitatively different software, each with its method written before it ran](../../Research/Exploring/five-demos.md)
- [plev: thesis of the generative interaction runtime](../../Research/Synthesis/generative-runtime-thesis.md)
- [program: from GPU UI engine to generative interaction runtime](../../Research/Synthesis/generative-runtime-program.md)
- [autonomous cycle journal](../../Research/Workflows/autonomous-cycle-journal.md)
<!-- /related -->
