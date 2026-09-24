---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-23
domain: ide
task: task-84
depends: [task-62]
branch: task/84-refresh-order
origin: agente roteirizado do checkpoint a (task-62 fase c)
---

# task-84: um refresh velho não desfaz o que o ide já aceitou

## por que

o agente roteirizado do laboratório da task-62 achou uma corrida no ide:
um `Refresh` que responde a uma mutação anterior pode trazer um status
lido antes de uma mutação posterior e tirar do `changes:staged` um arquivo
que o ide já tinha posto lá. o roteirizado contorna esperando o index
parar; um agente de verdade, ou uma pessoa, vê por um instante um estado
que não é nem o do git nem o que o ide aceitou, e pode decidir em cima
dele. é um defeito do ide, não do contrato, e é o tipo de defeito que o
contrato promete não esconder: o snapshot semântico tem que dizer a
verdade.

## estado no código (main @ c5fdf36)

- `crates/ide/src/session.rs:129-148`: toda mutação que volta do worker
  (`Staged`, `Unstaged`, `Discarded`, `Ignored`, `Committed`,
  `CommitRejected`) manda um `Refresh` novo.
- `crates/ide/src/session.rs:91-95`: todo `GitEvent::Status(Ok)` chega e
  substitui a lista inteira (`set_files`), sem saber se foi lido antes ou
  depois das mutações que a view já mostra.
- o worker (`crates/git/src/client.rs`) roda os comandos em ordem, um de
  cada vez. sequência que produz o defeito: stage A; volta `Staged A`; o
  ide pede o refresh R1; o worker começa R1; o ide aceita stage B (view
  otimista: B staged) e manda o comando; R1 volta com B fora do index e a
  view perde B até o refresh seguinte.

## escopo

a view nunca volta para um status mais velho que uma mutação que ela já
mandou. o mecanismo é do agente, com teste que falha antes da correção.

## aceite

teste determinístico que reproduz a sequência acima e falha na `main`;
com a correção, `changes:staged` nunca perde um arquivo que o ide aceitou
e cujo comando o git ainda não recusou; mudanças de fora do app (o
watcher) continuam chegando; gate verde.
