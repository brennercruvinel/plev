---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-23
domain: agent-surface
task: task-87
depends: [task-61, task-83]
branch: task/87-stage-revision
origin: checkpoint a, segunda rodada (os três agentes pediram)
---

# task-87: um estágio terminal traz a revisão em que o app ficou

## por que

na segunda rodada do checkpoint a, os três agentes LLM disseram a mesma
coisa: o `completed` de um ato não traz a revisão nova, então para
encadear o próximo ato o agente precisa de uma observação só para saber
a revisão. é o maior custo de idas e voltas que sobrou, e idas e voltas
são onde o plev perde para o `git` direto (H4). um agente que age
`--from-revision` precisa de uma revisão servida; hoje a única fonte é o
`observe`.

## estado no código (main @ 5be83c5)

- `crates/wire/src/proto.rs:197-203`: `StageReply { request, stage, at,
  asked }`, sem revisão.
- `crates/wire/src/server.rs:225-261`: `Server::forward` manda os
  estágios que o sink ouviu, sem acesso ao `Host`; chamado em `:219` (fim
  do `handle`) e em `crates/wire/src/port.rs:296` (fim do `drain`).
- `crates/wire/src/ring.rs:52`: `Ring::serve(snapshot) -> Revision`
  guarda um snapshot servido; `since` e `from_revision` só aceitam
  revisão que esteja no anel.

## escopo

um estágio terminal (`completed`, `rejected`, `failed`) servido a um
agente leva `revision`: a revisão da projeção inteira do app no instante
em que o estágio sai, guardada no anel como toda revisão servida. aditivo
no protocolo v1. mais a frase do `--keys` no `plev --help` (ele corta só
mudanças de estado; outras mudanças passam), que o controle da segunda
rodada estranhou.

## fora do escopo

por que um controle está desabilitado; o que `constrained` restringe;
conexão persistente do `plev`.

## aceite

teste no wire: o `revision` de um `completed` é aceito por `observe
--since` e por `act --from-revision`; o laboratório roteirizado usa a
revisão do `completed` onde hoje observa só para saber a revisão, com as
mensagens antes e depois no arquivo da task; gate verde.
