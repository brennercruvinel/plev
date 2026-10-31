---
id: task-84
title: "task-84: an old refresh does not undo what the ide has already accepted"
project: plev
type: task
status: done
date: 2026-09-23
date_basis: header
recorded: 2026-09-23
topics: [git, semantics]
relations: [task-62, task-83, task-85, adr-0036]
evidence: [4896451]
source: docs/log/steps/task-84-ide-refresh-never-goes-back.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-84: an old refresh does not undo what the ide has already accepted

Prevents a `Refresh` that responds to an earlier mutation from returning a status older than what the ide has already accepted. The `GitSession` counts the reads owed and discards the response to a read requested before a mutation the view already shows. Four tests on a real repository reproduced the race and failed on the base branch; with the fix, the fourteen `human_agent` tests pass in 20 consecutive rounds. Integrated into `main` at `4896451`; the commit receipt, which completes before the projection shows the commit, was left for task-85.

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

- `apps/idestudio/src/repository/git.rs:129-148`: toda mutação que volta do worker
  (`Staged`, `Unstaged`, `Discarded`, `Ignored`, `Committed`,
  `CommitRejected`) manda um `Refresh` novo.
- `apps/idestudio/src/repository/git.rs:91-95`: todo `GitEvent::Status(Ok)` chega e
  substitui a lista inteira (`set_files`), sem saber se foi lido antes ou
  depois das mutações que a view já mostra.
- o worker (`rust/gitdrv/src/client.rs`) roda os comandos em ordem, um de
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

## mecanismo (2026-09-23)

contexto: o worker do `rust/gitdrv` roda um comando por vez, na ordem em
que chegaram, e responde na mesma ordem (`rust/gitdrv/src/client.rs:125-130`).
a view mostra cada mutação assim que ela sai para o worker, e toda
resposta de mutação manda um `Refresh`. um status pedido antes de uma
mutação é lido antes dela, e a resposta chega depois de a view já
mostrar a mutação.

decisão: a sessão conta as leituras que pediu e, quando uma mutação sai,
marca como velhas todas as leituras ainda devidas do que ela muda; a
resposta velha é descartada. nada muda no worker nem na API do
`rust/gitdrv`.

- `apps/idestudio/src/repository/git.rs:39-70` (`Owed`): para cada leitura (status,
  log, branches), quantas respostas o worker ainda deve (`owed`) e quantas
  das mais antigas foram lidas antes de uma mutação (`stale`). `asked`
  soma uma devida, `outdated` faz `stale = owed`, `fresh` consome a
  resposta que chegou e diz se ela vale. resposta que ninguém pediu pela
  sessão vale, como antes.
- `apps/idestudio/src/repository/git.rs:77-120` (`Reads`): `sent` classifica o
  comando que sai. `Refresh` pede as três leituras; `Status`, `Log` e
  `Branches` pedem a sua; `Stage`, `Unstage`, `Discard` e `Ignore`
  envelhecem o status devido; `CommitIf` envelhece os três (o commit muda
  o index, o log e, no primeiro commit, cria a branch). o `match` não tem
  curinga: um comando novo em `GitCommand` obriga a dizer o que ele lê ou
  muda.
- `apps/idestudio/src/repository/git.rs:168-170`: todo comando passa por
  `GitSession::send`, que conta antes de mandar (`refresh`, `diff_file`,
  `diff_commit` e o `execute` do workbench já passavam por ali).
- `apps/idestudio/src/repository/git.rs:198-202`: `apply` descarta a resposta velha
  antes de tocar a view. o `settle` do workbench (`truth.rs:39`) roda
  antes e não olha status, log nem branches, então os receipts não mudam.

razão, e por que é a menor correta: toda mutação termina num `Refresh`
mandado quando a resposta dela chega (`session.rs:242-262`), e esse
refresh entra na fila depois de todas as mutações mandadas até ali. o
último refresh sempre vale e a view converge no git; um palpite que o git
recusou continua desfeito por ele. reaplicar as mutações pendentes sobre
um status velho exigiria uma segunda cópia das atualizações otimistas,
que hoje vivem só no `execute` (`set_staged`, `drop_path`), e saber quais
o git já fez. um número de sequência ecoado pelo worker exigiria mudar
`GitCommand` e `GitEvent` e o showcase junto. a contagem cabe na sessão e
se apoia na mesma ordem em que o casamento dos receipts já se apoia
(adr commit-checks-the-index-it-observed, consequências).

consequências:

- uma mudança de fora (watcher, `git` no terminal) lida por um refresh
  que ficou velho chega com o refresh seguinte, que já está na fila (o da
  mutação que envelheceu o primeiro). sem mutação pendente, o refresh do
  watcher vale como antes. numa rajada de mutações, a mudança de fora
  aparece no refresh da última.
- log e branches seguem a regra só para o commit. hoje nada no painel de
  stacks é otimista, então um log velho não tem o que desfazer: a regra
  só adia um log lido antes de um commit que o ide mandou até o refresh
  desse commit. ela garante que o painel nunca fique mais velho que o
  commit que o ide mandou, inclusive quando o painel ganhar um palpite.
- `GitSession::refresh`, `GitSession::send` e `Workbench::refresh` pedem
  `&mut self` (a contagem é estado da sessão); todo chamador já tinha
  `&mut`. `GitEvent`, `GitCommand`, `CommitIf` e a projeção semântica não
  mudam.

## handoff (2026-09-23)

goal: task-84, um refresh velho não desfaz o que o ide já aceitou.

status: pronto para a validação do orquestrador, na branch
`task/84-refresh-order`, base `dbc4a3f`.

implementado:

- a contagem de leituras devidas no `GitSession` (seção mecanismo).
- cinco testes em repositório real com o worker de verdade
  (`apps/idestudio/tests/human_agent.rs:466-611`), sem sleep: a ordem é a da
  fila do worker, e cada teste aplica os eventos um tipo por vez e olha a
  projeção no meio. quatro reproduzem a corrida (stage, unstage da
  pessoa, discard da pessoa, status pedido pelo watcher) e falham na
  base; o quinto segura o que não pode mudar (um stage que o git recusa
  é desfeito pelo refresh da resposta).
- três testes de unidade da regra (`apps/idestudio/src/repository/git.rs:276-335`):
  stage envelhece só o status, commit envelhece os três, leitura pedida
  depois da última mutação e leitura que ninguém pediu valem.
- o agente roteirizado do laboratório não espera mais o index "parar"
  (`apps/agenteval/src/arms/scripted.rs`, só o contorno: o `settle`, as
  duas constantes e as duas chamadas).

arquivos principais: `apps/idestudio/src/repository/git.rs`,
`apps/idestudio/src/capability/{mod.rs,execute.rs}` (só `&mut`),
`apps/idestudio/tests/human_agent.rs`, `apps/agenteval/src/arms/scripted.rs`.

interfaces alteradas: `GitSession::refresh(&mut self)`,
`GitSession::send(&mut self, GitCommand) -> bool`,
`Workbench::refresh(&mut self)`. nenhuma adicionada.

o teste falhando na base (commit `378d5fc`, com `--include-ignored`; dez
rodadas seguidas, as mesmas quatro falhas):

```text
a_status_read_before_a_stage_never_takes_the_file_back_out
  assertion failed: a status read before stage B does not take B out
  left: Set({"A"})  right: Set({"A", "B"})
a_status_read_before_an_unstage_never_puts_the_file_back
  assertion failed: a status read before the person's unstage does not put B back
  left: Set({"B", "C"})  right: Set({"C"})
a_status_read_before_a_discard_never_brings_the_file_back
  assertion failed: a status read before the discard does not bring B back
  left: List(["A", "B"])  right: List(["A"])
a_change_from_outside_lands_with_the_refresh_after_the_last_mutation
  assertion failed: the watcher's status, read before stage A, does not take A out
  left: Set({"B", "C"})  right: Set({"A", "C"})
test result: FAILED. 10 passed; 4 failed; 0 ignored
```

com a correção: 14 passed, 20 rodadas seguidas.

as dez corridas do laboratório (`ide-lab --arm scripted`, build de debug,
sem o contorno, em `1a4594b`):

| cenário | corrida | exit | recusas stale | delta | verdade | mensagens |
| --- | --- | --- | --- | --- | --- | --- |
| `--human none` | 1 a 5 | 0, 0, 0, 0, 0 | 0 | | MATCH | 14 |
| `--human unstage-after-observe:src/session.rs` | 1 a 5 | 0, 0, 0, 0, 0 | 1 | `removed: [src/session.rs]` | MATCH, recuperou | 22 |

o mesmo agente sem o contorno sobre a sessão da base (os três arquivos de
`378d5fc` postos de volta só para o build, depois restaurados), dez
corridas por cenário: `none` 10 de 10 com exit 0; com a pessoa, 10 de 10
falham (nove exit 1, uma exit 2). o relatório mostra as duas coisas que a
task temia: um status lido antes de um stage tira
`tests/login_test.rs` da view, o primeiro submit é decidido sobre isso e
recusado no index à toa ("the index staged [tests/login_test.rs] and not
[]"); depois da pessoa, um status lido antes do unstage dela põe
`src/session.rs` de volta, o agente não o põe de novo no index e commita
dois arquivos (`files_are_the_task: false`). o contorno estava lá por
causa desta corrida.

validação executada (com o ambiente do briefing, target
`target/agents/task-84`):

- `cargo test -p gitdrv`: 28 passed.
- `cargo test -p idestudio`: 72 passed (48 lib, 5 bin, 3 lab bin, 14
  human_agent, 2 lab).
- `cargo test -p uigallery`: 135 passed, 2 ignored.
- `./tool/tasks/fullcheck.sh`: verde, 1993 passed, 0 failed, 6 ignored (a `main` tinha
  1985, 0, 6; mais oito testes novos). a perna urnaui pulou por não haver
  `../urna` a partir do worktree, como na contagem da base.

outras corridas encontradas (descritas, não corrigidas):

- o receipt do commit fica à frente da projeção. o `Committed` completa o
  receipt com sha e arquivos (`apps/idestudio/src/capability/truth.rs:103-122`),
  mas `changes:staged` segue com os arquivos commitados e `stacks` sem o
  commit até chegar o refresh que o `Committed` manda
  (`apps/idestudio/src/repository/git.rs:251-256`). não é leitura velha por cima de
  um palpite (o commit não tem palpite na view), então não cabe nesta
  correção. aparece no laboratório: nas dez corridas o roteirizado
  observa `stacks` logo depois do submit concluído
  (`apps/agenteval/src/arms/scripted.rs:118-121`) e vê só o commit inicial
  ("stacks: 1 commit(s) projected"). um agente que confere o próprio
  commit pelo `stacks` na hora não o acha (o A1 da fase c achou porque
  levou segundos). caminhos: um palpite do commit na view quando o
  `Committed` chega (tirar do staged os arquivos do commit, pôr o commit
  no topo do stack), ou completar o receipt só quando o refresh seguinte
  ao commit for aplicado (mexe em `truth.rs`, fora do escopo).
- o diff mostrado não é relido. o `Refresh` lê status, log e branches
  (`rust/gitdrv/src/client.rs:200-204`); o diff de um arquivo (HEAD contra
  worktree, `rust/gitdrv/src/repo.rs:113-127`) fica com as linhas de antes
  de um commit ou de uma edição de fora até alguém pedir o diff de novo,
  e `diff:lines` projeta essas linhas. não é corrida de ordem, é leitura
  que ninguém refaz. discard e ignore já apagam o diff do arquivo e
  esquecem a resposta em voo (`apps/idestudio/src/capability/execute.rs:103-112`).

limitações:

- a contagem assume que todo comando passa por `GitSession::send`.
  `GitSession::client()` ainda expõe o `GitClient`, e um comando mandado
  direto por ele escaparia da conta; hoje ninguém faz, e o doc do
  `client()` diz isso.
- com mais de um worker, ou com reordenação, a contagem quebra junto com
  o casamento dos receipts; um tag ecoado por `GitCommand` e `GitEvent`
  resolveria os dois.

o que ficou de fora: as duas corridas acima; uma linha no adr
commit-checks-the-index-it-observed dizendo que as leituras também se
apoiam na ordem do worker (`docs/adr/**` é proibido nesta task, fica para
a integração).

dependências pendentes: nenhuma.

riscos de integração: a task-83 muda padrões de match em
`apps/agenteval/src/**`; aqui o `scripted.rs` só perdeu o `settle`
(36 linhas saíram, 8 entraram, nenhum match tocado). um conflito, se
houver, é mecânico.

commits:

- `378d5fc` task-84: the race, reproduced: a refresh read before a
  mutation lands over it
- `5aa3619` task-84: a read taken before a mutation the view shows is
  dropped
- `1a4594b` task-84: the scripted agent stops waiting for the index to
  hold still
- o commit desta seção: doc, mecanismo e handoff

## fechamento (orquestrador, 2026-09-23)

integrada na `main` em `4896451` (quatro commits, rebaseados sobre a
`main` sem conflito; a `main` tinha andado só em docs e na correção do
wasm `c07a7f0`). validação do orquestrador:

- diff inteiro lido, commit por commit. o `Reads`/`Owed` da sessão está
  certo pela ordem FIFO do worker: as respostas `stale` são exatamente as
  mais antigas das devidas; toda mutação termina num refresh posterior,
  então descartar nunca deixa a view sem a leitura seguinte. o `match` de
  `Reads::sent` não tem curinga: comando novo obriga a decidir.
- os testes de `apps/idestudio/tests/human_agent.rs` dirigem a ordem pelos
  eventos do worker real (`Rig::until`), sem sleep.
- gate verde no diretório da task sobre a base rebaseada: 2073 passaram,
  0 falharam, 7 ignorados, agora com a perna urnaui rodando no worktree
  (symlink `.claude/worktrees/urna`; antes ela pulava em todo worktree) e
  as pernas wasm com `-D warnings` como o CI.
- reproduzido pelo orquestrador: `ide-lab --arm scripted`, três corridas
  sem intervenção (exit 0, zero recusas, MATCH) e três com
  `unstage-after-observe:src/session.rs` (exit 0, uma recusa stale,
  recuperação, MATCH).
- o ADR commit-checks-the-index-it-observed ganhou a consequência das
  leituras na ordem do worker.

achados do agente que viraram trabalho: o receipt do commit conclui antes
de a projeção mostrar o commit (entrou na task-85, junto com `files` no nó
do commit); o diff mostrado nunca é relido depois de um commit ou de uma
edição de fora (registrado em `truth.md`, limite, sem task: não é
corrida de ordem e nenhum agente dependeu dele).
