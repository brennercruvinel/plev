---
project: plev
audience: [ai-agents, contributors]
status: pending
last-updated: 2026-09-24
domain: agent-port
task: task-94
depends: [task-92, task-93]
branch: task/94-refusal-points-at-question
origin: quarta rodada da task-62 (nenhum agente sem dica achou o `--ask`)
severity: P2 (completude do handoff; nenhum comportamento inválido)
---

# task-94: a recusa que nomeia uma pessoa aponta para a pergunta

## por que

na quarta rodada da task-62, com a pessoa tirando um arquivo logo antes
do submit, os quatro agentes sem dica nunca perguntaram: dois pararam
dizendo que não havia canal, e dois passaram por cima dela. os dois que
receberam uma frase sobre o `--ask` perguntaram na primeira recusa, e a
resposta da pessoa decidiu. nenhum dos seis abriu o `plev act --help`. o
que o agente vê na recusa é o JSON do estágio, e nada ali aponta para a
pergunta.

## escopo

- o `plev act`, quando o estágio terminal é uma recusa stale cujo
  `acted` tem uma pessoa (`human` ou `assistive`) e o ato não tinha
  `--ask`, escreve no stderr uma linha que diz quem agiu e como perguntar
  antes de agir por cima (`--ask "<por quê>"`, o limite de 280);
- nenhuma linha depois de um `declined` (a pessoa já respondeu; não se
  sugere perguntar de novo), nem quando o `acted` só tem agentes, nem
  quando o ato já perguntava;
- o stdout (o JSON) e o protocolo v1 não mudam.

## aceite

testes do `plev` para os quatro casos (pessoa no `acted`, só agentes,
`declined`, ato que já perguntava); o stdout igual byte a byte; a linha
no stderr pela porta real, num teste do ide com a pessoa; gate verde.
depois, a quinta rodada da task-62 mede se a linha basta.
