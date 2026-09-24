---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-23
domain: ide
task: task-85
depends: [task-84]
branch: task/85-commit-projected
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

## mecanismo (2026-09-23)

### os arquivos de cada commit, lidos pelo log

contexto: `git::Commit` não tinha arquivos, e `GitRepo::log` percorria o
histórico com gix sem abrir árvore nenhuma. o receipt já trazia os
arquivos do commit, lidos por `git show --name-only` logo depois do `git
commit`.

decisão:

- `Commit.files: Vec<String>` (`crates/git/src/types.rs`): os caminhos
  cujo entry muda entre a árvore do primeiro pai e a do commit; commit
  raiz contra a árvore vazia; merge contra o primeiro pai. entrada de
  árvore fica de fora; symlink e submódulo entram. a ordem é a de byte do
  caminho inteiro, que é a ordem em que o git lista (a regra da barra
  final das árvores dá o mesmo resultado).
- a leitura é o diff de árvores do gix (`gix::diff::tree` com o
  `Recorder`, `TreeDiff` em `crates/git/src/repo.rs`), sem feature nova e
  sem processo `git`.
- cache por sha dentro do `GitRepo` (`FilesCache`, num `RefCell`, então
  `log` continua `&self`): commit é imutável, a entrada nunca envelhece.
  cada consulta carimba a entrada, e no fim de cada `log` o cache fica
  com os commits pedidos por último, quatro vezes o `limit`. um log que
  andou um commit faz um diff de árvore.
- renomeação conta os dois caminhos: o antigo, que sai da árvore, e o
  novo, que entra. por quê: é o que as duas árvores dizem, sem heurística
  de similaridade e sem depender de `diff.renames`. a detecção de
  renomeação do gix mora atrás da feature `blob-diff` (puxa attributes,
  filter, pathspec, worktree, imara-diff), e ligá-la muda um
  `Cargo.toml`, fora dos arquivos desta task. parear só a renomeação
  exata (o mesmo blob dos dois lados) não resolve: aqui a renomeação vem
  quase sempre com edição. no maior dos dez commits com renomeação entre
  os últimos 100, sem renomeação são 177 caminhos, com `-M100%` 122 e com
  o `-M` padrão 116.

conferido neste repositório, nos 415 commits do HEAD da branch: os
`files` do gix são iguais a `git diff-tree -r --no-renames` contra o
primeiro pai em todos, 44 merges incluídos. contra `git show --name-only`,
de onde vem o receipt, 83 divergem; dos últimos 100, 12: 10 por
renomeação e 2 merges, em que o `git show` usa o diff combinado e não
lista nada.

### o palpite do commit

contexto: o `Committed(Ok)` completava o receipt (`truth.rs`) e a sessão
só logava e pedia o refresh. até o refresh chegar, `changes:staged`
seguia com os arquivos commitados e `stacks` sem o commit.

decisão: ao aplicar o `Committed(Ok)`, `Workbench::settle`
(`crates/ide/src/workbench/truth.rs`, `committed`) chama
`GitSession::show_commit` (`crates/ide/src/session.rs`) antes de
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

numa renomeação o `status` e o receipt usam o caminho novo, então sai a
linha staged do caminho novo; o antigo não aparece na lista.

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
  nenhum arquivo commitado em `changes:staged` e `stacks.commit[<sha>]`
  no topo, com os `files` do receipt (até o refresh; ver o item
  seguinte).
- depois do refresh, `files` é o que o log lê. num commit sem
  renomeação é igual ao receipt. com renomeação, ganha o caminho antigo.
  num commit de merge feito pelo ide (concluir um merge), o receipt é o
  diff combinado do `git show` e o log é o diff contra o primeiro pai.
- o painel de diff não é tocado: se mostrava um arquivo que o commit
  levou, segue com as linhas de antes, como já acontecia depois do
  refresh (task-84, "o diff mostrado não é relido").

## medição: `GitRepo::log(100)` neste repositório

`crates/git/tests/log_cost.rs`, ignorado por padrão, build de debug (as
dependências com `opt-level = 2`):
`cargo test -p git --test log_cost -- --ignored --nocapture`. frio é um
`GitRepo` novo a cada corrida, com o cache vazio (o cache de página do
sistema já está quente); com cache é a segunda chamada no mesmo
`GitRepo`, o que todo refresh depois do primeiro paga. mediana de 21
corridas, três rodadas alternando antes e depois, 415 commits no HEAD.

| | frio | com cache |
| --- | --- | --- |
| antes (`cf0abd3`, sem arquivos) | 1,25 a 1,49 ms | 1,22 a 1,37 ms |
| depois (task-85, com arquivos) | 12,8 a 13,3 ms | 1,16 a 1,27 ms |

o frio sobe uns 11,5 ms (o teto do briefing era 250 ms); com cache fica
igual, dentro do ruído (teto de 5 ms). os 100 commits carregam 2127
caminhos, uns 81 KB de texto, que agora vão no snapshot de quem observa
o workspace inteiro.

## laboratório

`ide-lab --arm scripted`, build de debug, cinco corridas por cenário,
sobre `f12fd30`:

| cenário | corrida | exit | recusas stale | delta | verdade | mensagens | confirmação pelo nó |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `--human none` | 1 a 5 | 0, 0, 0, 0, 0 | 0 | | MATCH | 14 | 5 de 5: `stacks.commit[<sha do receipt>]` com os três arquivos do receipt, 2 commits projetados |
| `--human unstage-after-observe:src/session.rs` | 1 a 5 | 0, 0, 0, 0, 0 | 1 | `removed: [src/session.rs]` | MATCH, recuperou | 22 | 5 de 5, idem |

o mesmo agente com o palpite desligado (a chamada de `show_commit`
trocada só para o build, depois restaurada), dez corridas: todas com exit
0 e MATCH, porque o exit olha só sessão x git, e nenhuma confirma. o
relatório diz "observed stacks: 2 nodes" (só o commit inicial) e a nota
"stacks.commit[...] does not hold the receipt's files [...]: None".

## handoff (2026-09-23)

goal: task-85, o commit que o receipt anuncia já está na projeção, com os
arquivos.

status: pronto para a validação do orquestrador, na branch
`task/85-commit-projected`, base `cf0abd3`.

implementado:

- `git::Commit.files`, lido por gix no `log`, com cache por sha no
  `GitRepo` (seção mecanismo).
- o estado `files` em todo `stacks.commit[<sha>]`, via
  `CommitEntry.files`; a tabela da projeção da task-62 ganhou a linha.
- o palpite do commit no mesmo passo em que o receipt conclui, antes
  dele (`GitSession::show_commit`, chamado de `truth.rs`).
- o agente roteirizado confirma o próprio commit pelo `files` do nó
  `stacks.commit[<sha do receipt>]`, na observação logo depois do
  receipt, sem esperar; os dois testes do laboratório exigem essa
  confirmação.

arquivos principais: `crates/git/src/{repo.rs,types.rs}`,
`crates/git/tests/{real_repo.rs,log_cost.rs}`,
`crates/ide/src/{session.rs,adapters.rs}`,
`crates/ide/src/workbench/{truth.rs,project.rs,mod.rs,tests.rs}`,
`crates/ide/src/views/multi_stack_view.rs` (o modelo:
`CommitEntry.files`), `crates/ide/src/views/workspace/tests.rs` (o
construtor), `crates/ide/src/bin/lab/scripted.rs`,
`crates/ide/tests/{human_agent.rs,lab.rs}`.

interfaces: `git::Commit` ganhou `files: Vec<String>` (campo público;
quem constrói `Commit` passa `files`); `CommitEntry` ganhou `files`;
`GitSession::show_commit` (`pub(crate)`) é nova; `adapters::unix_now`
passou a `pub(crate)`. `GitEvent`, `GitCommand`, `CommitIf`, `NewCommit`,
a regra de leituras velhas e os ids, roles, ações e reads da projeção não
mudaram; só entrou o estado `files`.

testes:

- git: `log_lists_the_files_each_commit_changes_in_gits_order` (normal e
  raiz, iguais a `git diff-tree`),
  `the_files_of_a_merge_are_what_it_brings_to_its_first_parent`,
  `a_moved_file_counts_at_both_paths` (e o receipt com só o caminho
  novo), `a_second_log_reads_no_tree` (as árvores somem do banco de
  objetos depois do primeiro log: um `GitRepo` novo falha, o mesmo
  devolve o log igual), e o limite do cache (teste de unidade em
  `repo.rs`).
- ide: `every_commit_in_the_stacks_exposes_the_files_it_changes`,
  `the_commit_a_receipt_announces_is_projected_before_any_refresh` (aplica
  só o `Committed` e olha: o nó existe no topo com os `files` do receipt,
  nada dele em `changes:staged`, o caminho com mudança no worktree fica
  unstaged; depois do refresh, o mesmo vindo do git),
  `no_answer_after_the_commit_takes_it_back` (um refresh do watcher
  pedido antes do submit e outro depois; a invariante vale depois de cada
  resposta), `a_path_unstaged_after_the_commit_left_is_left_for_the_refresh`.
- sem o palpite, os três últimos falham em `Rig::value` ("projected"): o
  nó `stacks.commit[<sha>]` não existe depois do `Committed`. os cinco
  testes de repositório e os três de unidade da task-84 seguem verdes sem
  mudança.

validação executada (target `target/agents/task-85`, o ambiente do
briefing):

- `cargo test -p git`: 33 passed, 1 ignored (antes 28).
- `cargo test -p ide`: 76 passed (antes 72: 48 lib, 5 bin, 3 lab bin, 18
  human_agent, 2 lab).
- `cargo test -p showcase`: 135 passed, 2 ignored (igual).
- `./script/gate`: verde. `cargo test --workspace` 2002 passed, 0 failed,
  7 ignored (a base tinha 1993, 0, 6; mais nove testes e a medição
  ignorada). a perna urnaui rodou (o symlink `../urna` do worktree): 80
  passed, 1 ignored, clippy, fmt e wasm verdes; o urnaui não depende de
  `git` nem de `ide`, então a contagem dele não muda com esta task.

limitações:

- renomeação e merge: depois do refresh, `files` de um commit com
  renomeação traz o caminho antigo, que o receipt não traz; num merge
  feito pelo ide o receipt é o diff combinado. a invariante vale na
  observação seguinte ao receipt quando o refresh ainda não chegou, e
  depois dele só em commit sem renomeação e sem merge.
- `files` não tem limite na projeção: neste repositório são uns 81 KB a
  mais no snapshot inteiro do ide. `diff:lines` tem limite e `count`;
  `files` não, como o briefing pediu.
- o limite do cache é por chamada (quatro vezes o `limit` do `log` em
  curso): um `GitCommand::Log` com `limit` pequeno encolhe o cache. o ide
  só pede `LOG_LIMIT`.
- o `summary`, o autor e a hora do palpite são palpites (seção
  mecanismo); o refresh os troca.

o que ficou de fora:

- detectar renomeação no log como o `git show`: ligar a feature
  `blob-diff` do gix em `crates/git/Cargo.toml` (ou no `Cargo.toml` da
  raiz) e usar `Tree::changes` com o rastreio de renomeação da
  configuração. o custo não foi medido. a outra saída, o receipt ler os
  arquivos pelo mesmo diff de árvores, muda o receipt e o laboratório,
  que compara com `git show --name-only`.
- o adr commit-checks-the-index-it-observed lista `stacks.commit[<sha>]`
  com `summary`, `author`, `show_diff`: falta `files`, e uma linha sobre
  o palpite do commit (`docs/ADR/**` é proibido nesta task).
- o diff mostrado depois de um commit (achado da task-84).

dependências pendentes: nenhuma.

riscos de integração: quem constrói `git::Commit` ou `CommitEntry` numa
branch paralela precisa passar `files`. no laboratório só o
`scripted.rs` mudou: a contagem de commits do `stacks` virou a função
`confirm`; nenhum match tocado. os testes novos do ide estão no fim de
`human_agent.rs`.

commits:

- `834738a` task-85: the log carries the files of each commit, read once
  per commit
- `c948482` task-85: every commit in the stacks projects its files
- `563db5e` task-85: the commit a receipt announces is shown before the
  receipt ends
- `f12fd30` task-85: the scripted agent confirms its commit by the node's
  files
- `421583d` task-85: the log measurement also weighs the files it carries
- o commit desta seção: mecanismo, medição, laboratório e handoff
