---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-24
domain: experiment
task: task-93
depends: [task-88, task-92]
branch: task/93-lab-person-answers
origin: handoff da task-92 (o laboratório não tem pessoa que responde ao modal)
severity: P2 (habilita o experimento do handoff; nada que existe fica inválido)
---

# task-93: a pessoa do laboratório responde

## por que

a task-92 deu ao agente um jeito de perguntar à pessoa antes de agir
(`plev act --ask`), pela confirmação que o ide já mostra num modal. o
laboratório (`ide-lab`) tem uma pessoa roteirizada que tira um arquivo do
index. ela não responde a modal nenhum, então um agente que pergunta
fica esperando até o timeout. o próximo experimento com LLM (o agente
pergunta em vez de passar por cima ou parar?) precisa dessa pessoa.

## escopo

- um cenário novo da pessoa, que responde a toda confirmação pedida por
  um agente: permitir, recusar, ou não responder (o padrão de hoje),
  depois de um atraso configurável que o relatório diz;
- o relatório conta as perguntas (requests com `ask`), as respostas, o
  tempo até a resposta e o que o agente fez depois de cada uma;
- a gravação da sessão tem a resposta como input da pessoa;
- o roteirizado ganha um modo que pergunta antes de recolocar o arquivo,
  para o teste do gate.

## aceite

testes do laboratório: o roteirizado que pergunta, com a pessoa
permitindo (commit com os três arquivos) e recusando (sem commit, o
agente para); o relatório com as perguntas e as respostas; gate verde.
