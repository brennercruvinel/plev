---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-23
domain: generative-runtime
task: task-58b
depends: [task-58]
branch: task/58b-sem-decline
---

# task-58b: recusar uma confirmação

## objetivo

um request que espera confirmação hoje só sai desse estado se uma pessoa
confirmar. fechar o modal sem confirmar (task-62) deixa o receipt
aguardando para sempre. o contrato ganha a recusa: uma pessoa, na tela ou
por tecnologia assistiva, recusa a confirmação, e o receipt termina
rejeitado com o motivo.

## escopo

- `Rejection::Declined { by: Actor }` e `Dispatcher::decline(id, by, now)`,
  com as mesmas regras de `confirm`: só `AwaitingConfirmation`, só
  `Actor::Human` ou `Actor::Assistive` (`ConfirmByMachine` para agente e
  teste), receipt inalterado no erro.
- a transição `AwaitingConfirmation -> Rejected` já é legal na tabela de
  estágios; nada muda nela.
- a forma JSON de `Rejection` ganha a variante; os goldens do `sem` são
  atualizados.
- o ide passa a chamar `decline` quando a pessoa fecha o modal sem
  confirmar (`crates/ide/src/workbench`), e o toast diz que foi recusado.

## aceite

testes no `sem` (pessoa recusa, agente não pode recusar, recusa em quem não
espera é `IllegalTransition`, sinks veem o estágio); teste no ide (fechar o
modal do `discard` de um agente termina o receipt como `Declined` e o
arquivo não é descartado); gate verde.
