---
id: task-64
title: "task-64: session inspector, bug report and session for testing"
project: plev
type: task
status: done
date: 2026-09-25
date_basis: header
recorded: 2026-09-23
topics: [recording, semantics, agents]
relations: [adr-0050, task-63, task-65, task-66, task-83]
evidence: [e6560a2, d611fa4]
source: docs/log/steps/task-64-session-inspector-and-tests.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-64: session inspector, bug report and session for testing

Records the task that turned a recorded session into a tool for reading and testing: an inspector that answers the thesis questions about a control or a request (`plev-inspect` in phase a, the `plev-inspector` window in phase b), a bug report exported from an excerpt (`--export`), and a regression test generated from the session, run in the gate with an isolated harness (phase c). It was closed on 2026-09-25, with the acceptance criteria checked item by item. It matters because the inspector only reads the recording and never runs it; controlled re-execution stays restricted to the test, in an isolated environment.

## objetivo

abrir uma sessão gravada, navegar no tempo com scrub, escolher um controle
num instante e ver o estado semântico, a ação, o ator, a precondição, o
delta e o resultado; exportar um trecho como bug report; e transformar um
trecho escolhido em teste de regressão.

## ligação com a tese

seções 11 (open session, scrub timeline, select control, inspect semantic
state, action, actor, result), 12 (bug report deixa de ser mídia; sessão
vira teste), 13 (no replay, selecionar um controle leva ao código daquela
versão), 28 (a pergunta do núcleo: o que alguém precisa descobrir no
primeiro `.monster` para confiar na delegação).

## estado atual no código

- `monster_player` (`demo/player/src`) toca
  animação com seek e scrub pelo player determinístico de
  `rust/motion/src/play/mod.rs`.
- depois de 63: `Session::open`, estado semântico em t, receipts por
  `SemId`, trilha visual.
- depois de 66: render headless de frame.

## escopo

### inspector

- um app ou uma página do showcase (decisão no briefing; recomendação:
  crate de app próprio, porque abre arquivo e vai crescer) que abre uma
  sessão.
- timeline com os eventos marcados por tipo e por ator; scrub com seek
  O(1) no keyframe mais próximo.
- o frame do instante, com as ligações desenhadas por cima (contorno do
  controle, `SemId` no hover).
- clique num controle no instante t: painel com o nó (role, label, state),
  os receipts que tocam esse `SemId` até t, e para cada receipt a cadeia
  causal: observação (quem, qual revisão), request (ator, `observed`),
  veredito (com o delta quando stale), execução, resultado.
- as perguntas da tese respondidas na tela: qual controle era esse, qual
  era o estado, qual foi a sequência anterior, qual ação ocorreu, quem
  iniciou, qual era a precondição, o que mudou, qual foi o resultado.
- link para o código quando a task-65 estiver integrada: o `source` do nó
  mais a identidade do build no cabeçalho da sessão; se a árvore de
  trabalho diverge daquele sha, mostrar o arquivo da versão gravada
  (`git show <sha>:<arquivo>`), não o atual.

### bug report

exportar `[t0, t1]` a partir do inspector (usa o export da task-63).

### sessão para teste

- o desenvolvedor escolhe, no inspector, os estados e resultados que
  viram asserções.
- a saída é um arquivo de dados (o trecho com as asserções marcadas) e um
  harness genérico que roda o trecho contra uma fixture do app: aplica as
  requests na ordem, confere os vereditos e os valores marcados.
- isso é reexecução controlada, e o documento diz isso: o harness roda num
  ambiente isolado (tempdir, relógio injetado, sem rede), a fixture é
  fornecida pelo app, e nada toca estado externo real. replay no
  inspector continua sem executar nada.

## fora de escopo

reexecução de efeitos externos reais; edição de sessão; compartilhamento
remoto.

## dependências

63 e 66 integradas; 65 para o link de código.

## superfície sob responsabilidade

o crate ou página do inspector; o harness de teste de sessão (onde o ADR
de 63 colocou o leitor); proposta de ADR
[`0050-sessions-become-regression-tests.md`](../../adr/Recording/0050-sessions-become-regression-tests/0050-sessions-become-regression-tests.md).

## interfaces que consome

`Session` (63), render headless (66), `Node.source` (57) e o resolvedor de
código (65), comps para a UI.

## interfaces que pode alterar

as próprias.

## critérios de aceitação

- a sessão do cenário do commit (gravada em 63) aberta no inspector: scrub
  até antes do commit, clique no botão, painel mostra o estado que o agente
  observou, o request, a rejeição com "removed: B", depois o aceite e o sha.
- export de trecho reabre com o mesmo conteúdo.
- um teste gerado a partir da sessão roda verde no gate e falha quando a
  regra de precondição é quebrada de propósito (prova que o teste pega
  regressão).
- inspector não cria request em nenhum caminho (teste).
- `./tool/tasks/fullcheck.sh` verde.

## testes esperados

testes do modelo do inspector sem UI (estado em t, cadeia causal por
`SemId`), testes de viewport estreito e largo da UI, teste do harness com
uma fixture de brinquedo.

## riscos e perguntas abertas

- asserção gerada acoplada demais (todo o estado) fica frágil: o default
  é só o que o desenvolvedor marcou.
- a fixture do app é trabalho do app; o harness só define a interface.

## commits esperados

1. `task-64: modelo do inspector (estado em t, cadeia causal)`
2. `task-64: UI com timeline, frame e ligações`
3. `task-64: painel do controle e perguntas da tese`
4. `task-64: export de trecho`
5. `task-64: sessão para teste com harness isolado`

## pare e reporte se

- a geração de teste exigir reexecução que toque estado externo;
- o formato de sessão não der acesso O(1) que o scrub precisa.

## fases (orquestrador, 2026-09-23)

a task é grande (modelo, UI, export, sessão para teste) e foi dividida:

- **fase a, agora:** o modelo do inspector sem UI e uma CLI
  (`plev-inspect`) que responde às perguntas da tese sobre uma sessão
  gravada: o que era o controle, qual era o estado, quem observou o quê e
  quando, qual request, qual veredito (com o delta e o `acted`), o que
  aconteceu entre a observação e o request (as mudanças no que a ação lê
  e os receipts de outros atores que rodaram nesse intervalo, pela
  gravação), e o resultado. responde por tempo a pergunta "quem mudou o
  que eu li", que o `acted` da task-83 responde só em parte. funciona já
  nas sessões dos dois checkpoints a (`target/agents/exp/`,
  `target/agents/exp2/`).
- **fase b:** a UI (timeline, frame com as ligações por cima, painel do
  controle). pede gravar as ligações por frame no `rec`, que hoje não
  grava (`truth.md`, 2.4).
- **fase c:** export de trecho como bug report e sessão para teste com o
  harness isolado.

## Phases

- [phase a: inspector model and `plev-inspect` CLI](phase-a-model.md): the model without an interface, the causal chain of each request, the control at the instant and the CLI, with the handoff and the integration (2026-09-23).
- [phase b: inspector UI](phase-b-ui.md): the probe of where the UI lives and what it reads, the script for opening it, the `plev-inspector` window and the handoff (2026-09-25).
- [phase c: session for testing and export](phase-c-session-to-test.md): the probe that replays the recorded requests, the `rerun` harness, the generated test for a2ppa, `--export` and the closing of the phase (2026-09-24).
- [closing of the task](closing.md): the final check of the acceptance criterion, item by item (2026-09-25).

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0037: session recording: replay, not re-execution](../../adr/Recording/0037-session-recording-replay-not-reexecution/0037-session-recording-replay-not-reexecution.md)
- [ADR 0039: agents act through the semantic contract](../../adr/Semantics/0039-agents-act-through-the-semantic-contract/0039-agents-act-through-the-semantic-contract.md)
- [ADR 0046: the inspector reads a session, it never runs it](../../adr/Recording/0046-the-inspector-reads-a-session-never-runs-it/0046-the-inspector-reads-a-session-never-runs-it.md)
- [ADR 0047: an act rests on what its actor was shown](../../adr/Semantics/0047-an-act-rests-on-what-its-actor-was-shown/0047-an-act-rests-on-what-its-actor-was-shown.md)
- [ADR 0050: sessions become regression tests](../../adr/Recording/0050-sessions-become-regression-tests/0050-sessions-become-regression-tests.md)
- [ADR 0067: adaptive transformations are small verified operations that preserve identity](../../adr/Synthesis/0067-adaptive-transforms-preserve-identity/0067-adaptive-transforms-preserve-identity.md)
- [ADR 0070: generative runtime results](../../adr/Synthesis/0070-generative-runtime-results/0070-generative-runtime-results.md)
- [what the autonomous cycle taught: a commit checks the index it observed](../../adr/Semantics/0036-commit-checks-the-index-it-observed/learned-commit-checks-the-index-it-observed.md)
- [what the autonomous cycle taught: the inspector reads a session, it never runs it](../../adr/Recording/0046-the-inspector-reads-a-session-never-runs-it/learned-the-inspector-reads-a-session-never-runs-it.md)
- [what the autonomous cycle taught: sessions become regression tests](../../adr/Recording/0050-sessions-become-regression-tests/learned-sessions-become-regression-tests.md)
- [task-62: proof in the ide: stale-state rejection and human-agent handoff](../task-62-ide-human-agent-proof/task-62-ide-human-agent-proof.md)
- [task-63: session flight recorder](../task-63-flight-recorder/task-63-flight-recorder.md)
- [task-65: pixel to code and development inspector](../task-65-pixel-to-source/task-65-pixel-to-source.md)
- [task-66: headless as a first-class citizen](../task-66-headless-first-class/task-66-headless-first-class.md)
- [task-67: structural regression before pixels](../task-67-structural-regression/task-67-structural-regression.md)
- [task-80: proof of qualitatively different software](../task-80-proof-qualitatively-different-software/task-80-proof-qualitatively-different-software.md)
- [task-83: the delta carries the node that was just added, and the refusal names who acted](../task-83-delta-carries-node-refusal-names-actor/task-83-delta-carries-node-refusal-names-actor.md)
- [task-86: the ide people use opens the agent port](../task-86-the-ide-opens-the-agent-port/task-86-the-ide-opens-the-agent-port.md)
- [task-96: a generated surface in the ide](../task-96-ide-generated-surface/task-96-ide-generated-surface.md)
- [task-62 phase c, first round: design, protocol and result (2026-09-23)](../task-62-ide-human-agent-proof/phase-c-round-1.md)
- [task-63 phase d: the ide records the session the inspector reads](../task-63-flight-recorder/phase-d.md)
- [task-63 phase d re-scoped: the ide records the session](../task-63-flight-recorder/phase-d-rescope.md)
- [task-64 closing: acceptance criterion checked](closing.md)
- [task-64 phase a: inspector model and the plev-inspect CLI](phase-a-model.md)
- [task-64 phase b: inspector UI](phase-b-ui.md)
- [task-64 phase c: session to test, and export](phase-c-session-to-test.md)
- [task-80, phase 1: probe, five-demo method, demos 3 and 4 and handoff](../task-80-proof-qualitatively-different-software/phase-1-probe-and-demos.md)
- [plev: thesis of the generative interaction runtime](../../Research/Synthesis/generative-runtime-thesis.md)
- [program: from GPU UI engine to generative interaction runtime](../../Research/Synthesis/generative-runtime-program.md)
- [autonomous cycle journal](../../Research/Workflows/autonomous-cycle-journal.md)
<!-- /related -->
