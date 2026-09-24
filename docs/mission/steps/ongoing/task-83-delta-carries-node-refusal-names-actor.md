---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-23
domain: semantic-contract
task: task-83
depends: [task-57b, task-61, task-62]
branch: task/83-delta-and-refusal
origin: checkpoint a (task-62 fase c), decisão do brenner "correções primeiro"
---

# task-83: o delta traz o nó que nasceu, e a recusa diz quem agiu

## por que

os dois agentes LLM do checkpoint a operaram o ide só pelo contrato e
apontaram, sem que isso impedisse a tarefa (`task-62`, seção "o que o
experimento mostrou"):

- `node_added` num delta traz só o id: o agente que vê o commit nascer em
  `stacks` precisa de uma segunda observação para saber o que nasceu;
- a recusa por estado velho diz o que mudou, mas não quem mudou: o agente
  não sabe se foi uma pessoa, outro agente ou algo de fora do app;
- a regra "stale é por `reads`, e a revisão só rastreia" foi deduzida pelo
  agente, não lida em nenhum lugar que ele alcança.

o brenner escolheu corrigir isso antes da onda 5.

## estado no código (main @ c5fdf36)

- `crates/sem/src/diff.rs:23`: `Change::NodeAdded(SemId)`.
- `crates/rec/src/ring.rs:47-53`: o recorder trata `NodeAdded`,
  `ParentChanged` e `Reordered` como estruturais e grava um snapshot
  inteiro; `crates/rec/src/state.rs:83` recusa `node_added` dentro de
  `Kind::Changes`.
- `crates/wire/src/server.rs:329-339`: o delta passa o `Change` como veio
  do `diff`, filtrando só `StateChanged` por `keys`; o snapshot tira
  `source` quando não pedido (`:349`).
- `crates/sem/src/dispatch.rs`: o dispatcher guarda os receipts abertos e
  os últimos 256 terminados, cada um com `validated_against` (a revisão
  viva em que foi checado); ninguém usa isso para dizer quem agiu.
- `crates/ide/src/workbench/notice.rs:84-87`: a recusa stale vira "Refused
  agent's submit: <o que mudou>", sem autor da mudança.

## escopo

1. `Change::NodeAdded` carrega o nó inteiro; o wire aplica ao nó o mesmo
   filtro de `keys` e a mesma retirada de `source` do snapshot.
2. `Rejection::Stale` ganha a lista das ações de outros atores que rodaram
   sobre o estado que o autor da ação recusada viu (regra exata no
   briefing); o dispatcher preenche num lugar só.
3. a recusa do ide nomeia quem agiu.
4. a regra de stale fica escrita onde o agente lê (`plev --help`, doc do
   protocolo) e no ADR.

## fora do escopo

aplicar `NodeAdded` no recorder sem snapshot inteiro (a posição entre
irmãos não vem no `Change`), `files` no nó do commit (task-85), a corrida
do `Refresh` (task-84), eco do `set_value`.

## aceite

testes novos no sem, no wire e no ide; o delta de um item novo no todo traz
o nó; a recusa do laboratório (`ide-lab --arm scripted --human
unstage-after-observe:src/session.rs`) nomeia a pessoa que tirou o arquivo
do index; gate verde; ADR novo.
