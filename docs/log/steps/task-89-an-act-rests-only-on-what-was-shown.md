---
project: plev
audience: [ai-agents, contributors]
status: done
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

- `rust/socket/src/server.rs`, `act`: `(None, Some(revision))` pega o
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

## handoff (agente, 2026-09-23)

a primeira leitura do briefing pedia o histórico por conexão. o agente
parou e reportou: o `plev` abre uma conexão por comando
(`rust/socket/src/bin/plev.rs`, `connect`), então por conexão todo
`plev act --from-revision` viraria `unknown_revision`, inclusive no
`rust/socket/tests/cli.rs` e no braço A do laboratório. o orquestrador
decidiu pela opção A: a chave é o ator declarado no `hello`.

### a regra como ficou

- um ato com `from_revision r` se apoia no que foi mostrado ao ator dele
  (o nome do `hello`, em qualquer conexão; o `close` não apaga), e em
  nada além disso. `r` tem que ter sido dado àquele ator e estar no anel,
  senão `unknown_revision`. a declaração da ação e os `reads` vêm do
  snapshot de `r`, como antes. o valor de cada `read` vem da última parte
  mostrada, até a última vez que o ator recebeu `r`, que mostrou aquele
  `read`, lido no anel. a `Observation` continua com a revisão `r`.
- o que cada coisa mostra: um snapshot mostra os nós sob o `scope` (todos
  sem ele) com o estado sob as `keys`; um delta mostra só o que traz (um
  estado que mudou, o estado sob as `keys` de um nó adicionado ou
  removido); um estágio terminal dá a revisão e não mostra estado. um
  `observe --since <revisão de estágio>` vazio não mostra nada.
- um `read` nunca mostrado, ou cuja última exibição saiu do anel, recusa
  o ato antes do dispatch com `not_observed {reads}` (refs, nunca valor,
  na ordem da ação). uma exibição mais antiga que o anel ainda guarda não
  entra no lugar: não é o que o agente viu por último. um snapshot com
  escopo cuja revisão saiu do anel já não sabe quais nós tinha, então um
  `read` que ele pode ter mostrado também conta como não mostrado
  (conservador: recusa, nunca admite).
- limites: as últimas `ring.cap()` revisões por ator (16 por padrão), no
  máximo `MAX_ACTORS` = 16 atores, sai o servido há mais tempo; o
  esquecido conta como nunca mostrado.
- `observed` explícito continua como estava: é a afirmação do agente.
- `plev act` sem `--from-revision`: a olhada no escopo do alvo conta como
  mostrada ao nome; o que o ato lê fora do alvo vem das observações
  anteriores daquele nome ou é `not_observed`. `plev` sai 3, como no
  `unknown_revision` (recusa do protocolo antes do dispatch, mesmo
  remédio).

### o teste do cenário, antes e depois

`rust/socket/tests/server.rs`,
`an_act_rests_on_what_was_shown_not_on_the_whole_revision`. o toy ganhou
`todo.clear`, que lê `todo.counter:done` (estado de outro nó, como o
`commit.submit` lê `changes:staged`). o agente observa só o counter
(item 3 done), a pessoa marca o item 1, o agente observa só `todo.clear`
e age com a revisão nova.

- antes (base, commit 1 com `#[ignore]`): o ato montava o observado do
  snapshot inteiro da revisão.

  ```text
  assertion `left == right` failed: what the agent was shown, not what the revision holds
    left: [(todo.counter:done, Some(Set({"1", "3"})))]
   right: [(todo.counter:done, Some(Set({"3"})))]
  ```

  com `{1, 3}` observado e `{1, 3}` vivo, o stale passava e o clear
  apagava duas linhas.
- depois: recusado como stale, com o delta (`done` de `{3}` para
  `{1, 3}`, `added: ["1"]`), nada apagado. stale e não `not_observed`
  porque o counter foi mostrado ao agente, com o valor de antes da
  mudança. sem nunca ter observado o counter, é `not_observed`
  (`a_read_never_shown_is_not_observed`).

### testes

- `rust/socket/tests/shown.rs` (12, novo): observação que cobre o `read`
  depois da mudança (aceito); `read` nunca mostrado (`not_observed`, nada
  despachado, aceito depois de observar); `--keys` que corta a chave;
  encadear numa revisão de estágio (aceito com o `read` observado antes,
  `not_observed` sem); delta vazio desde a revisão de estágio não mostra
  nada (stale); última exibição fora do anel (`not_observed`, a mais
  antiga no anel não entra); limite de `cap` revisões por ator; revisão
  de outro ator ou de antes do `hello` (`unknown_revision`); um nome em
  duas conexões é um ator; o 17º ator derruba o mais antigo, que é
  recusado; estágio que termina depois dá a revisão ao ator; `observed`
  explícito inalterado.
- `rust/socket/tests/server.rs`: o cenário (sai do `#[ignore]` no commit
  da correção); `a_row_the_agents_own_act_moved_is_stale_until_it_observes_it`
  (o próprio ato deixa o seguinte stale com `acted` vazio, o agente
  observa, o ato é aceito). mudaram, como a task aceitou:
  `a_completed_act_carries_the_revision_the_next_one_rests_on` (o t3
  encadeia no item 3, que nenhum ato anterior mexeu, em vez do item 1) e
  `stages_reach_only_the_connection_that_asked` (o A observa antes; A e B
  já tinham nomes diferentes, `claude` e `gemini`).
- `rust/socket/tests/cli.rs`:
  `an_act_that_reads_a_node_its_name_never_observed_is_the_error_exit`
  (`plev act todo.clear clear --name fresh` sai 3 com `not_observed`; a
  observação de outro nome não conta; depois de `plev observe --scope
  todo.counter --name fresh`, o mesmo comando completa).
- `rust/socket/tests/codec.rs`: a forma de `not_observed` fixada e no
  roundtrip de toda resposta. `plev`: teste do `--help`.

### o laboratório

`ide-lab --arm scripted`, debug, cinco vezes cada cenário, depois do
rebase na main com a task-86: 10 de 10 com exit 0 e MATCH, nenhum erro de
protocolo. o roteirizado não precisou de observação a mais: ele observa o
workspace inteiro antes de cada decisão, e cada ato encadeado lê o estado
do próprio arquivo, que nenhum ato dele mexeu.

| cenário | mensagens in | mensagens out | bytes out | stale |
| --- | --- | --- | --- | --- |
| sem pessoa | 12 | 30 | 28776 a 28796 | 0 |
| a pessoa tira `src/session.rs` | 17 | 39 | 45861 a 45865 | 1 |

são as mensagens da task-87 (12 e 17).

### validação

- `./tool/tasks/fullcheck.sh` verde depois do rebase na main (`1f16133`), com
  `df -h /` em 19 GB livres. workspace depois: 2136 passed, 11 ignored.
  antes (a main): 2120 passed, 11 ignored, contado como o total de depois
  menos os 16 testes que a branch adiciona (`git diff main`: 16 `#[test]`
  novos, nenhum removido, nenhum `#[ignore]` novo). perna urnaui: 82
  passed, 1 ignored.
- `wire`: 60 antes, 76 depois. `ide` e `showcase` verdes (também no gate).
- `apps/idestudio/tests/agent_port.rs` (task-86), depois do rebase: os dois
  testes verdes, e a regra não muda nada neles. o encadeamento lê o que o
  agente viu na observação inteira e ninguém mexeu, e a recuperação
  observa de novo antes do `submit`.

### decisões

- a ordem dos commits mudou um pouco em relação ao briefing: o erro novo
  entrou antes do histórico, para cada commit passar no clippy sem código
  morto; o histórico, o observado montado dele e o uso do erro entraram
  juntos no commit da correção.
- `Shown` num módulo próprio (`rust/socket/src/shown.rs`), com os valores
  lidos no anel: o histórico guarda escopo, `keys` e os refs que um delta
  trouxe, nunca valor.

### limitações e riscos

- `acted` nomeia quem agiu na revisão que o ato nomeia
  (`Observation::revision` = `r`). quando um `read` vem de uma observação
  anterior e alguém o mexeu antes de `r`, `changes` diz o que mudou e
  `acted` não nomeia ninguém por isso (o teste do delta vazio mostra).
  isso toca a task-90 (a recusa nomeia quem agiu desde que o agente
  olhou): a regra dela ainda se apoia na revisão do `Observation`.
- dois processos com o mesmo nome são um ator. o nome padrão do CLI é
  `plev`, então agentes distintos precisam de `--name` ou
  `PLEV_AGENT_NAME` (escrito no ADR e no `--help`).
- o custo por ato (voltar no histórico por `read`) não foi medido; é
  limitado por `cap` revisões por `read`.

### linha de ADR proposta (docs/adr/README.md)

```text
| 2026-09-23 | [an-act-rests-on-what-its-actor-was-shown](an-act-rests-on-what-its-actor-was-shown.md) | accepted | um ato com `from_revision` se apoia só no que foi mostrado ao ator (o nome do `hello`, em qualquer conexão: dois processos com o mesmo nome são um ator): a revisão tem que ter sido dada a ele, a ação vem dela, e o valor de cada `read` vem da última observação até ela que o mostrou (escopo e `keys`; um delta mostra só o que traz; a revisão de um estágio não mostra estado); um `read` nunca mostrado é recusado antes do dispatch com `not_observed {reads}` (aditivo no v1, `plev` sai 3); `observed` explícito continua sendo a afirmação do agente; limitado como o anel (16 revisões por ator, 16 atores); corrige a promessa do anel no ADR do agent port, furo achado pelo inspector na sessão do A2 (task-89) |
```

### commits

- `4d94364` o cenário como teste, ignorado até a correção
- `69b3191` `not_observed`, o erro do protocolo
- `114151d` a correção: um ato se apoia no que foi mostrado ao ator
- `ea6176d` os testes
- `a125a06` docs: doc do `Act`, `plev act --help`, ADR novo e correções
  datadas no ADR do agent port
- este: o handoff

## fechamento (orquestrador, 2026-09-23)

integrada na `main` em `79b8c93` (seis commits sobre `1f16133`).

o agente parou uma vez, antes de implementar. a regra "por conexão" do
briefing fechava o furo e também recusava todo `plev act
--from-revision`, porque cada `plev` abre uma conexão nova. decisão do
orquestrador: por ator, com o nome do `hello` (journal, ciclo 2).

validação do orquestrador:
- diff inteiro lido commit a commit (`shown.rs`, `server.rs::rest`, o
  registro no `observe` e no `forward`, o erro, os docs);
- gate verde no diretório da task: 2218 passaram, 0 falharam, 12
  ignorados, com a perna urnaui (a main tinha 2202; 16 testes novos).

reprodução do orquestrador no laboratório (`ide-lab --arm plev`, a
pessoa tirando `src/session.rs` logo depois de uma observação), com o
`plev` desta branch:
- um nome que nunca observou: `plev act commit.submit submit --name
  fresh` sai 3 com `not_observed` e as reads `changes:staged` e
  `commit.message:text`;
- o cenário do A2: o agente observa `changes` e vê os três arquivos, a
  pessoa tira um, o agente observa só `commit.form` e submete com a
  revisão nova. o request leva `changes:staged` com os três, que é o que
  ele viu, e é recusado stale com `removed: ["src/session.rs"]`. antes
  desta task, o valor saía da revisão inteira (dois arquivos) e o commit
  saía com dois;
- de quebra, uma observação com `--keys staged,files` não mostra o
  `unstaged` que o `stage` lê, e o stage saiu `not_observed`. a regra
  vale também para `keys`.

o roteirizado: `--arm scripted` sem e com a pessoa, exit 0 e MATCH nos
dois.

fica para a task-91: nessa cena o `acted` sai vazio. a recusa usa a
revisão `r` do ato (a do formulário, depois do unstage) e não a da
observação de onde veio o valor. o agente propôs `Shown::values`
devolver a revisão de onde saiu cada `read`, que é a âncora da task-91.
