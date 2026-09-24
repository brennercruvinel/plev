---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-23
domain: experiment
task: task-88
depends: [task-87, task-89]
branch: task/88-lab-rests-on
origin: fechamento da task-87 e a regra da task-89
---

# task-88: o laboratório mede no que o agente se apoiou

## por que

o laboratório da task-62 (`ide-lab`) mede a checagem de stale com uma
pessoa roteirizada. duas premissas dele envelheceram:

1. a medição é válida quando um `submit` depois da intervenção se apoiou
   numa revisão servida por observação antes dela (`story.rs`,
   `stale_measurement`). desde a task-87 um agente encadeia na revisão de
   um estágio terminal, que não é observação. desde a task-89 o valor de
   cada `read` vem da última observação que o mostrou ao agente, não da
   revisão do ato. a revisão do ato deixou de dizer em que o agente se
   apoiou; o valor que o request carrega diz.
2. a pessoa age logo depois de uma observação servida
   (`LabHost::observed`, `intervene`). um agente que olha uma vez e
   encadeia o resto nos estágios nunca dispara a pessoa, e a rodada sai
   sem intervenção.

e a task-89 criou uma recusa nova antes do dispatch (`not_observed`), que
o relatório não conta.

sem isso, o próximo experimento com LLM (conferir que agentes se
recuperam de `not_observed` e da regra da 89) mede errado.

## escopo

- a medição de stale pelo significado: válida quando um `submit` depois
  da intervenção carrega em `observed` o `changes:staged` de antes da
  intervenção (o mundo velho), qualquer que seja a revisão;
- a pessoa age também logo depois de um estágio terminal entregue ao
  agente, quando a tarefa está pronta;
- o relatório conta `not_observed` e `unknown_revision` como recusas do
  protocolo, com as reads;
- `lab.rs` dividido por responsabilidade abaixo do limite (sai da lista
  congelada do guard).

## aceite

testes do laboratório para os dois jeitos de agente (observa antes do
submit; encadeia nos estágios sem observar) com e sem a pessoa; medição
válida nos dois com a pessoa; o roteirizado com MATCH; gate verde.
