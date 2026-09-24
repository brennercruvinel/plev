---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-23
domain: agent-surface
task: task-89
depends: [task-61, task-87]
branch: task/89-rests-on-what-was-shown
origin: o inspector de sessão (task-64 fase a) lendo a sessão do A2 da primeira rodada do checkpoint a
---

# task-89: um ato se apoia só no que foi mostrado ao agente

## por que

o ADR agents-act-through-the-semantic-contract diz que `from_revision`
reconstrói os valores observados de um ato "so the stale check compares
what the agent saw, not a later frame". não é o que acontece com
observação por escopo. o servidor guarda o snapshot inteiro de cada
revisão servida e o `act --from-revision r` monta o `observed` com
`Request::observe` sobre o snapshot inteiro de `r`, inclusive estado fora
do escopo que o agente pediu. o inspector de sessão mostrou isso na
sessão do A2 da primeira rodada: o submit se apoiou numa observação só de
`commit.form`, e o `changes:staged` que ele "observou" nunca foi servido
naquela observação.

o cenário que atravessa a proteção:

1. o agente observa `changes` (A, B e C staged);
2. uma pessoa tira C do index;
3. o agente observa só `commit.form` e recebe a revisão nova;
4. o agente pede `submit --from-revision` com essa revisão: o request diz
   ter visto `changes:staged = {A, B}`, a checagem passa, o ponto de
   verdade confere `{A, B}` contra o index `{A, B}`, e o commit sai com
   dois arquivos. o agente decidiu achando que eram três.

é o erro que o contrato existe para impedir, entrando por uma observação
com escopo. é invariante quebrada: a garantia está escrita e o código não
a cumpre.

## estado no código (main)

- `crates/wire/src/server.rs`, `act`: `(None, Some(revision))` pega o
  snapshot inteiro do anel e chama `Request::observe` sobre ele.
- `observe` serve o recorte (`scoped`, `keys`) mas guarda no anel o
  snapshot inteiro (`self.ring.serve(live)`), sem registrar o que foi
  mostrado a quem.
- desde a task-87, um estágio terminal também serve uma revisão (sem
  mostrar estado nenhum).

## decisão (orquestrador, modo autônomo)

a resposta conservadora é evidente: um request nunca diz ter visto um
valor que não foi mostrado àquela conexão. regra: um ato de uma conexão
com `from_revision r` se apoia no que aquela conexão tinha recebido quando
recebeu `r`: para cada estado que a ação lê, o valor vem da observação
mais recente daquela conexão, até `r`, cujo recorte (escopo e `keys`)
incluía aquele estado. um estado que a conexão nunca recebeu num recorte
é recusado antes do dispatch, com um erro tipado que diz quais `reads`
faltam (o agente observa e decide de novo). uma revisão de estágio
terminal marca o ponto no tempo, mas não mostra estado. o formato das
mensagens não muda.

consequência aceita: um agente que encadeia atos sem observar o que o
próximo ato lê (ex.: o `submit` depois de pôr arquivos no index) precisa
de uma observação antes, exatamente o que o princípio pede ("agir sobre o
que viu"). o laboratório roteirizado e os agentes LLM das duas rodadas já
observavam antes de cada submit.

## aceite

o cenário acima vira teste e o `submit` é recusado (stale ou o erro de
"não observado"); os testes da task-87 (encadear em revisão de estágio)
continuam verdes onde o ato lê o que foi observado; o laboratório
roteirizado sai 0 com MATCH nos dois cenários; o ADR diz a regra; gate
verde.
