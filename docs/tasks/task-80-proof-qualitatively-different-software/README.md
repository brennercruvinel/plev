---
id: task-80
title: "task-80: proof of qualitatively different software"
project: plev
type: task
status: in-progress
date: 2026-09-25
date_basis: header
topics: [generated-surfaces, agents, recording, research]
relations: [adr-0050, adr-0070, task-60, task-62, task-63, task-64, task-68, task-69, task-70, task-84, task-86, task-87, task-89, task-92, task-93, task-94, task-96, task-97, generative-runtime-thesis]
source: docs/log/steps/task-80-proof-qualitatively-different-software.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
evidence: [e44e4234, c8a6c3a9, f008c897, 46f272d3, b26dc988, ae5be30e, 677b1ad9]
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
couberem lá, ADR de resultado [`0070-generative-runtime-results.md`](../../adr/generated/0070-generative-runtime-results.md).

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
