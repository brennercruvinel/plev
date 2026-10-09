---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-09-23
domain: ide
task: task-85
depends: [task-84]
branch: task/85-commit-projected
integrated: 476bbb4
origin: checkpoint a (agentes LLM) e validação da task-84
---

# task-85: o commit que o receipt anuncia já está na projeção, com os arquivos

## por que

duas coisas que agentes reais esbarraram, no mesmo nó:

- o nó `stacks.commit[sha]` não expõe os arquivos do commit como estado
  (apontado pelos agentes LLM do checkpoint a, que confirmaram o próprio
  commit por `show_diff` ou pelo receipt);
- o receipt do `submit` chega a `completed { commit, files }` antes de a
  projeção mostrar o commit: `changes:staged` ainda lista os arquivos
  commitados e `stacks` ainda não tem o commit até o refresh chegar
  (achado pelo agente da task-84: nas dez corridas do roteirizado, a
  observação logo depois do submit concluído viu só o commit inicial). um
  agente que confere o próprio commit na hora não o encontra; um que
  decide em cima dessa observação decide sobre um estado que contradiz o
  receipt que ele acabou de receber.

## invariante

depois que o receipt de um commit chega a `completed`, a próxima
observação não contradiz o receipt: os arquivos commitados não estão em
`changes:staged`, e `stacks.commit[<sha>]` existe com `files` igual ao
`files` do receipt. o refresh que vem depois reconcilia com o git sem
desfazer isso (regra da task-84).

## escopo

1. `files` como estado de todo `stacks.commit[sha]`: os caminhos que o
   commit muda contra o primeiro pai (commit raiz: contra a árvore vazia),
   lidos pelo worker junto com o log, com cache por sha (commit é imutável).
2. ao aplicar `Committed(Ok(NewCommit { sha, files }))`, a view mostra o
   commit na hora (palpite otimista, como as outras mutações), no mesmo
   passo em que o receipt conclui.

## aceite

teste de git (log com arquivos: commit normal, raiz, merge pelo primeiro
pai), teste do ide para a invariante sem nenhum refresh aplicado entre o
receipt e a observação, custo do log medido antes e depois, laboratório
roteirizado confirmando o commit pelo `files` do nó, gate verde.

## mecanismo (2026-09-23, revisto depois da primeira validação)

### uma definição dos arquivos de um commit

contexto: `git::Commit` não tinha arquivos, e `GitRepo::log` percorria o
histórico com gix sem abrir árvore nenhuma. o receipt trazia os arquivos
do commit novo, lidos por `git show --name-only`: renomeação só no caminho
novo, merge em diff combinado. a primeira entrega desta task leu os do log
por diff de árvores, com renomeação nos dois caminhos, e o nó e o receipt
discordavam depois do refresh num commit com renomeação. o orquestrador
decidiu por uma definição só, a do diff de árvores.

decisão:

- os arquivos de um commit são os caminhos cujo entry muda entre a árvore
  do primeiro pai e a do commit. o commit raiz é comparado com a árvore
  vazia, o merge com o primeiro pai, e a renomeação conta os dois caminhos
  (o antigo sai, o novo entra), sem heurística de similaridade e sem
  depender de `diff.renames`. entrada de árvore fica de fora; symlink e
  submódulo entram. a ordem é a de byte do caminho inteiro, que é a ordem
  em que o git lista. pela cli, é o que lista
  `git diff-tree --no-commit-id --name-only -r --no-renames --root --diff-merges=first-parent <sha>`.
- uma função só em `rust/gitdrv/src/repo.rs`: `GitRepo::files_of` (privada),
  sobre o `TreeDiff` (`gix::diff::tree` com o `Recorder`, sem feature nova
  do gix e sem processo `git`). `commit_files(sha)` (qualquer revisão que
  nomeie um commit), `commit` e `commit_if` (o `NewCommit` do receipt) e
  `log` passam por ela. o `git show --name-only` saiu.
- cache por sha dentro do `GitRepo` (`CommitFiles`, cache e buffers num
  `RefCell`, então tudo continua `&self`): commit é imutável, a entrada
  nunca envelhece. cada consulta carimba a entrada, e no fim de cada `log`
  o cache fica com os commits pedidos por último, quatro vezes o `limit`.
  o commit que o ide faz entra no cache pelo receipt, e o refresh seguinte
  não relê a árvore dele.
- as testemunhas independentes leem pela cli, com aquele `git diff-tree`:
  `Repo::head_files` do laboratório (o lado git da comparação sessão x
  git, mudança autorizada pelo orquestrador; só essa função mudou nos
  arquivos do laboratório além do `scripted.rs`) e `Rig::head_files` dos
  testes do ide.

conferido neste repositório (`rust/gitdrv/tests/log_cost.rs`, ignorado): nos
440 commits do HEAD, 47 merges, o `files` do log e o `commit_files` lido
sozinho, como o receipt lê, são iguais ao `git diff-tree` acima.

o que muda para quem lê o receipt: numa renomeação, o receipt traz os dois
caminhos (antes, só o novo); num merge feito pelo ide, os caminhos que ele
traz para o primeiro pai (antes, o diff combinado, em geral vazio). o
`status`, e com ele `changes:staged` e o `expect_staged` que o `CommitIf`
confere, continua vendo uma renomeação no index pelo caminho novo (adr
commit-checks-the-index-it-observed). então, num commit com renomeação, o
`files` do receipt tem um caminho a mais que o staged set que o agente
mandou: o antigo. a dupla checagem não mudou.

### o palpite do commit

contexto: o `Committed(Ok)` completava o receipt (`truth.rs`) e a sessão
só logava e pedia o refresh. até o refresh chegar, `changes:staged`
seguia com os arquivos commitados e `stacks` sem o commit.

decisão: ao aplicar o `Committed(Ok)`, `Workbench::settle`
(`apps/idestudio/src/capability/truth.rs`, `committed`) chama
`GitSession::show_commit` (`apps/idestudio/src/repository/git.rs`) antes de
`dispatcher.complete`, no mesmo `Workbench::apply`. a chamada fica em
`truth.rs` porque é ali que existe a mensagem do `CommitIf`
(`Inflight::Commit`); guardar uma segunda cópia dela na sessão duplicaria
o que o workbench já guarda.

como o palpite decide cada arquivo do receipt:

| o que a view mostra do caminho | depois do palpite |
| --- | --- |
| só a linha staged | o caminho sai da lista |
| linha staged e linha unstaged (mudou de novo no worktree depois do add) | a linha staged sai; a unstaged fica, com o status dela |
| nenhuma linha staged (a pessoa tirou do index depois que o commit saiu, ou o caminho não está na lista) | fica como estava; o refresh decide |

numa renomeação o receipt traz os dois caminhos: sai a linha staged do
caminho novo, e o antigo não está na lista (o `status` já o juntou na
renomeação).

o commit entra no topo do log de onde as stacks saem (`log_cache`,
reconstruído com as branches que já estavam lá): o `sha` do receipt, a
primeira linha da mensagem sem espaço nas pontas como `summary`, o autor
do commit que era o HEAD até ali, a hora de agora e os `files` do
receipt. o log palpitado é cortado em `LOG_LIMIT`. o que o palpite não
sabe e o refresh traz: um primeiro parágrafo de várias linhas, que o git
junta com espaço no `summary`; o autor de verdade, quando a identidade
configurada não é a do HEAD anterior; e, num repositório sem commit, a
branch: o primeiro commit aparece no stack "(detached HEAD)", com autor
vazio e `stacks:branch` vazio, até o refresh.

razão, e a regra da task-84: toda leitura devida quando o `CommitIf` saiu
já é marcada velha por ele e descartada; toda leitura pedida depois dele
é lida depois do commit, porque o worker responde em ordem. então nenhuma
resposta desfaz o palpite, e o refresh que o `Committed` manda substitui
o log palpitado pelo que o git leu. `Reads`, `Owed`, `GitEvent` e
`GitCommand` não mudaram, e o receipt conclui no mesmo momento de antes.

consequências:

- a observação seguinte ao receipt concluído não contradiz o receipt:
  nenhum arquivo commitado em `changes:staged`, e `stacks.commit[<sha>]`
  no topo, com os `files` do receipt. depois do refresh, o mesmo, agora
  lido do git, também com renomeação e em merge (uma definição só).
- o painel de diff não é tocado: se mostrava um arquivo que o commit
  levou, segue com as linhas de antes, como já acontecia depois do
  refresh (task-84, "o diff mostrado não é relido").

### o que a projeção carrega

decisão: `files` vai só nos `FILES_PROJECTED` (10) commits mais recentes
do stack ativo (`apps/idestudio/src/capability/project.rs`) e no último commit
que o workbench fez (`GitSession::made`, gravado por `show_commit`), onde
quer que ele esteja no stack. os outros commits mantêm `summary`, `author`
e `show_diff`, sem `files`. os stacks das outras branches não têm commits
projetados (o log segue só o HEAD).

razão: com `files` em todo commit, uma observação inteira do ide sobre
este repositório tinha três vezes o tamanho da base (tabela abaixo). um
agente que confere o próprio commit precisa de um; dez dão o que entrou
logo antes por uns 1,6 KB a mais (3,8%) numa observação inteira daqui; 20
custariam uns 3,6 KB (8%); todos os 100, uns 88 KB (203%). o peso é o
tamanho dos commits que caem na janela, não o N: cada caminho custa uns
40 bytes, então um commit de 259 caminhos (o maior dos últimos 100 daqui)
pesaria sozinho uns 10 KB se estivesse entre os dez.

## medições

### `GitRepo::log(100)` neste repositório

`rust/gitdrv/tests/log_cost.rs`, ignorado por padrão, build de debug (as
dependências com `opt-level = 2`):
`cargo test -p git --test log_cost -- --ignored --nocapture --test-threads=1`.
frio é um `GitRepo` novo a cada corrida, com o cache vazio (o cache de
página do sistema já está quente); com cache é a segunda chamada no mesmo
`GitRepo`, o que todo refresh depois do primeiro paga. mediana de 21
corridas, três rodadas alternando antes e depois.

| sessão | | frio | com cache |
| --- | --- | --- | --- |
| primeira entrega (415 commits no HEAD) | antes (`cf0abd3`, sem arquivos) | 1,25 a 1,49 ms | 1,22 a 1,37 ms |
| | depois | 12,8 a 13,3 ms | 1,16 a 1,27 ms |
| depois da correção (a main com a task-83, máquina mais carregada) | antes (main, sem arquivos) | 2,67 a 5,70 ms | 2,55 a 5,36 ms |
| | depois (uma definição, `files_of`) | 22,3 a 23,7 ms | 2,89 a 3,11 ms |

o frio sobe de 11 a 20 ms (o teto do briefing era 250 ms); com cache fica
no ruído da base (teto de 5 ms a mais). na segunda sessão a própria base
dobrou (outros agentes compilando), e `files_of` lê de novo o objeto do
commit que o log já tinha lido, porque recebe só o id; as duas coisas
entram no aumento, e a medição não separa quanto cada uma pesa. os 100
commits do log carregam 1756 caminhos, uns 65 KB, que ficam no processo:
a projeção leva só os de dez.

### o que uma observação pesa

a mesma forma do braço screenshot do laboratório: a linha que o servidor
escreve para uma observação (`Reply::Snapshot` pelo codec do wire, sem
`source`, id de um caractere, mais o fim de linha).

neste repositório, `apps/idestudio/tests/observe_cost.rs` (ignorado; um clone
limpo do HEAD, para o que está sem commit no worktree não pesar; 100
commits no stack, 107 nós):

| | observação inteira | `--scope stacks` |
| --- | --- | --- |
| base (main, sem `files`), HEAD `02f789e` | 43.388 B | 42.253 B |
| `FILES_PROJECTED = 10`, HEAD `02f789e` | 45.020 B | 43.885 B |

o mesmo teste com outros N, sobre o HEAD anterior (`bf648fc`), inteira:
N = 1, 43.507 B; 3, 43.773 B; 5, 43.935 B; 10, 44.864 B; 20, 46.944 B; 100
(`files` em todo commit, a primeira entrega), 131.893 B.

no repositório do laboratório, os passos do agente roteirizado (cada
observação diz o que pesa; corridas abaixo):

| | base (main) | depois |
| --- | --- | --- |
| inteira, antes do submit (1 commit) | 6.390 B sem a pessoa, 6.579 B com | 6.481 B, 6.670 B |
| `--scope stacks`, logo depois do commit | 506 B: só o commit inicial, o novo não está lá | 998 B: os dois, com `files` |
| inteira, logo depois do commit | 6.386 B, 6.575 B: os três arquivos ainda staged, sem o commit | 4.385 B, 4.574 B: os arquivos saíram de `changes`, o commit entrou |

no laboratório o stack tem no máximo dois commits, então o N não muda
nada ali; o que muda é que a observação logo depois do commit ficou menor
e diz o que o receipt diz.

## laboratório

`ide-lab --arm scripted`, build de debug, cinco corridas por cenário,
sobre a branch depois da correção (antes do commit desta seção):

| cenário | corrida | exit | recusas stale | delta | verdade | mensagens | confirmação pelo nó |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `--human none` | 1 a 5 | 0, 0, 0, 0, 0 | 0 | | MATCH | 15 | 5 de 5: `stacks.commit[<sha do receipt>]` com os três arquivos do receipt, 2 commits projetados |
| `--human unstage-after-observe:src/session.rs` | 1 a 5 | 0, 0, 0, 0, 0 | 1 | `removed: [src/session.rs]` | MATCH, recuperou | 23 | 5 de 5, idem |

`truth.head_files` agora vem do `git diff-tree` da definição; nas dez
corridas é `[src/login.rs, src/session.rs, tests/login_test.rs]`, igual aos
`files` do receipt. uma mensagem a mais que na primeira entrega: a
observação inteira depois da confirmação.

com a biblioteca da main (sem arquivos no log e sem palpite) e este
laboratório, uma corrida por cenário: exit 0 e MATCH, porque o exit olha
só sessão x git, e nenhuma confirma ("stacks.commit[...] does not hold
the receipt's files [...]: None"). na primeira entrega, as mesmas dez
corridas com o palpite desligado deram 0 de 10.

## handoff (2026-09-23, depois da correção)

goal: task-85, o commit que o receipt anuncia já está na projeção, com os
arquivos.

status: pronto para a validação do orquestrador, na branch
`task/85-commit-projected`, rebaseada sobre a main `558b061`.

implementado:

- uma definição dos arquivos de um commit (diff de árvores contra o
  primeiro pai, renomeação nos dois caminhos, raiz contra a árvore vazia),
  numa função só do `rust/gitdrv`, para o receipt (`commit`, `commit_if`,
  `commit_files`) e para o log, com cache por sha; as testemunhas do
  laboratório e dos testes leem a mesma definição pelo `git diff-tree`.
- o estado `files` nos `FILES_PROJECTED` (10) commits mais recentes do
  stack ativo e no último commit que o ide fez; a tabela da projeção da
  task-62 diz isso.
- o palpite do commit no mesmo passo em que o receipt conclui, antes
  dele (`GitSession::show_commit`, chamado de `truth.rs`).
- o agente roteirizado confirma o próprio commit pelo `files` do nó
  `stacks.commit[<sha do receipt>]`, na observação logo depois do
  receipt, sem esperar, e diz quanto pesa cada observação; os dois testes
  do laboratório exigem a confirmação.

arquivos principais: `rust/gitdrv/src/{repo.rs,types.rs}`,
`rust/gitdrv/tests/{real_repo.rs,log_cost.rs}`,
`apps/idestudio/src/{session.rs,adapters.rs}`,
`apps/idestudio/src/capability/{truth.rs,project.rs,mod.rs,tests.rs}`,
`apps/idestudio/src/interfaces/panels/stacks.rs` (`CommitEntry.files`),
`apps/idestudio/src/interfaces/panels/layout/tests/mod.rs` (o construtor),
`apps/agenteval/src/{scripted.rs,repo.rs}` (`repo.rs`: só
`head_files`),
`apps/idestudio/tests/{human_agent.rs,lab.rs,observe_cost.rs,support/mod.rs}`.

interfaces:

- `git::Commit` ganhou `files: Vec<String>` (quem constrói `Commit` passa
  `files`).
- `GitRepo::commit_files(sha)`: mesma assinatura, outra definição (a de
  `Commit::files`), lida por gix, aceita qualquer revisão que nomeie um
  commit. o `NewCommit.files` do receipt segue a mesma definição.
- `CommitEntry` ganhou `files`; `ide::workbench::FILES_PROJECTED` é
  pública; `GitSession::show_commit` e `GitSession::made` são
  `pub(crate)`; `adapters::unix_now` passou a `pub(crate)`.
- não mudaram: `GitEvent`, `GitCommand`, `CommitIf` e a dupla checagem,
  a regra de leituras velhas, e os ids, roles, ações e reads da projeção.

testes:

- git: `log_lists_the_files_each_commit_changes_in_gits_order` (normal e
  raiz, iguais ao `git diff-tree` e ao `commit_files`),
  `the_files_of_a_merge_are_what_it_brings_to_its_first_parent`,
  `a_moved_file_counts_at_both_paths_in_the_commit_and_in_the_log` (um
  `git mv`, que o index vê como uma renomeação no caminho novo, commitado
  pelo `commit_if`: os dois caminhos no `NewCommit` e no log),
  `a_second_log_reads_no_tree` (as árvores somem do banco de objetos
  depois do primeiro log: um `GitRepo` novo falha, o mesmo devolve o log
  igual), o limite do cache (unidade, em `repo.rs`). ignorados: o custo
  do log e a checagem dos commits deste repositório.
- ide: `only_the_newest_commits_of_the_checked_out_stack_carry_their_files`
  (unidade), `every_commit_in_the_stacks_exposes_the_files_it_changes`,
  `the_commit_a_receipt_announces_is_projected_before_any_refresh`,
  `no_answer_after_the_commit_takes_it_back`,
  `a_path_unstaged_after_the_commit_left_is_left_for_the_refresh`,
  `a_commit_with_a_rename_holds_both_paths_in_the_receipt_and_the_node`
  (o agente põe no index um README apagado e o mesmo arquivo em docs/,
  que o index junta numa renomeação; receipt, nó antes do refresh, nó
  depois e `git diff-tree` têm os dois caminhos),
  `the_last_commit_the_ide_made_keeps_its_files_past_the_newest` (falha
  sem o `made`). ignorado: o peso de uma observação deste repositório.
- sem o palpite, os quatro testes do palpite falham em `Rig::value`
  ("projected"). os testes da task-84 seguem verdes sem mudança.

validação executada (target `target/agents/task-85`, o ambiente do
briefing), sobre a main `558b061`:

- `cargo test -p git`: 33 passed, 2 ignored (a main: 28, 0).
- `cargo test -p ide`: 80 passed, 1 ignored (a main: 73, 0; 50 lib, 5
  bin, 3 lab bin, 20 human_agent, 2 lab).
- `cargo test -p showcase`: 135 passed, 3 ignored (igual à main).
- `./tool/tasks/fullcheck.sh`: verde. `cargo test --workspace` 2016 passed, 0
  failed, 11 ignored (a main `558b061`: 2004, 0, 8; mais doze testes e as
  três medições ignoradas). a perna urnaui rodou, no subdiretório próprio
  do target (a correção `1bf6606`): 80 passed, 1 ignored, clippy, fmt e
  wasm verdes.

limitações:

- numa renomeação o receipt traz um caminho a mais que o staged set que o
  agente mandou (o antigo); um agente que compare os dois precisa saber a
  definição. o adr commit-checks-the-index-it-observed ainda diz que o
  receipt lista os arquivos como o `git show --name-only`.
- a janela de dez não tem teto em bytes: um commit enorme entre os dez
  pesa em toda observação inteira enquanto estiver lá.
- o limite do cache é por chamada (quatro vezes o `limit` do `log` em
  curso): um `GitCommand::Log` com `limit` pequeno encolhe o cache. o ide
  só pede `LOG_LIMIT`.
- o `summary`, o autor e a hora do palpite são palpites (seção
  mecanismo); o refresh os troca.

o que ficou de fora:

- o diff mostrado depois de um commit (achado da task-84).
- os textos da task-62 que ainda citam o `git show --name-only` como a
  verdade do laboratório (a tabela de exit code, a descrição da verdade na
  fase c): este arquivo só podia mexer na tabela da projeção de lá.

linha proposta para o adr commit-checks-the-index-it-observed (seção "the
second check, at the index", no lugar de "the files as `git show
--name-only` lists them"; e no modelo, `stacks.commit[<sha>]`):

```text
`GitEvent::Committed` carries `NewCommit { sha, files }`: the paths the
commit changes against its first parent (a root commit against the empty
tree), a rename at both paths, read in process by the same function the
log uses (`git diff-tree -r --no-renames --root
--diff-merges=first-parent` by the CLI). The workbench shows the commit
(its files out of the staged rows, the commit on top of the stack) in
the step that completes the receipt, before it; `stacks.commit[<sha>]`
carries `files` on the ten newest commits and on the last one the
workbench made.
```

dependências pendentes: nenhuma.

riscos de integração: quem constrói `git::Commit` ou `CommitEntry` numa
branch paralela precisa passar `files`. quem compara o `files` de um
receipt com `git show --name-only` vê diferença em renomeação e merge.
no laboratório mudaram só o `scripted.rs` (confirmação, peso de cada
observação, uma observação inteira depois do commit) e o `head_files` do
`repo.rs`.

commits:

- `90f7a12` task-85: the log carries the files of each commit, read once
  per commit
- `f5486a1` task-85: every commit in the stacks projects its files
- `cc9a4fa` task-85: the commit a receipt announces is shown before the
  receipt ends
- `3aafd4a` task-85: the scripted agent confirms its commit by the node's
  files
- `e811f39` task-85: the log measurement also weighs the files it carries
- `40ec683` task-85: the task file gets the mechanism, the measurement and
  the handoff
- `9faeae0` task-85: one definition of a commit's files, for the receipt
  and the log
- `b809f96` task-85: the witnesses read a commit's files as git diff-tree
  lists them
- `da6b251` task-85: only the newest commits carry their files in the
  projection
- `05a67cd` task-85: what an observation weighs, in the lab and on this
  repository
- o commit desta seção: mecanismo, medições, laboratório e handoff
  revistos

## fechamento (orquestrador, 2026-09-23)

integrada na `main` em `476bbb4`. validação do orquestrador: diff inteiro
lido nas duas rodadas; devolvida uma vez com duas correções (uma
definição só dos arquivos de um commit, porque receipt e nó discordavam
depois do refresh numa renomeação; e `files` só nos commits mais
recentes, porque em todos os 100 a observação inteira deste repositório
triplicava). gate verde no diretório da task: 2096 passaram, 0 falharam,
12 ignorados (três medições novas ignoradas), com a perna urnaui.
laboratório reproduzido pelo orquestrador: MATCH sem e com a intervenção,
o roteirizado confirmando pelo `files` do nó (exigido por
`apps/agenteval/tests/experiment.rs`). fechamento: ADR commit-checks (a definição e
o palpite do commit), textos da task-62 que citavam `git show
--name-only`, changelog, programa, truth, learnings.
