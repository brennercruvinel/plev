---
id: task-62
title: "task-62: proof in the ide: stale-state rejection and human-agent handoff"
project: plev
type: task
status: done
date: 2026-09-23
date_basis: header
recorded: 2026-09-23
topics: [semantics, agents, recording, git]
relations: [task-57, task-58, task-59, task-60, task-61, task-63, task-64, task-70, task-85, adr-0035, adr-0036, experiment-agent-proof-checkpoint-a]
evidence: [8978602]
source: docs/log/steps/task-62-ide-human-agent-proof.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-62: proof in the ide: stale-state rejection and human-agent handoff

Proves, in a real git client in the ide, that a human and an agent operate the same software through the semantic contract. The agent's action on a state that has gone stale is refused with the delta that names what changed, and the valid action returns the sha and the files of the commit, checked against git. Phases a and b are integrated in `8978602`; phase c ran in five rounds with LLM agents and the `ide-lab` lab, and the central hypothesis held in all of them. It matters because it separates what the contract already guarantees (no wrong commit, refusal before the effect) from what still costs too much (bytes and back-and-forth against `git` on the command line).

## objetivo

provar num app real, o git client, que humano e agente operam o mesmo
software ao mesmo tempo com segurança: a ação do agente sobre um estado
que envelheceu é recusada com o delta, a ação válida é aceita e devolve um
resultado verificável (o sha e os arquivos do commit), e a condução passa
de um para o outro sem que ninguém aja sobre um mundo que já mudou.

## ligação com a tese

seções 8 (o exemplo do git client), 9 (receipts), 10 (handoff), 11 e 12
(o experimento termina no `.monster`). do arquivo de concepção, o
"experimento do plev" inteiro está na fase c.

## estado atual no código

- o caminho do commit: `apps/idestudio/src/interfaces/panels/commit_form.rs`
  (`hit_test_click` devolve `CommitFormAction::Commit`) ->
  `apps/idestudio/src/interfaces/panels/layout/input.rs:47` -> `submit_commit`
  (`:259-267`) empurra `UiRequest::Commit { message }`
  (`apps/idestudio/src/interfaces/panels/layout/mod.rs:115-123`) -> `GitSession`
  traduz para `GitCommand::Commit` (`apps/idestudio/src/repository/git.rs:76`) ->
  worker -> `rust/gitdrv/src/repo.rs:187` roda `git commit --message` e
  `git rev-parse HEAD`.
- "arquivos selecionados" no ide é o index: `FileEntry.staged`
  (`apps/idestudio/src/interfaces/panels/unassigned.rs:51-55`). stage e unstage vão
  por `UiRequest::Stage`/`Unstage`.
- `UnassignedView.selected_idx` é a linha em destaque (cursor). não entra
  no commit. é o exemplo perfeito de estado visual que não pode invalidar
  a ação.
- a view é otimista e reconcilia depois de cada mutação com um refresh
  completo (`apps/idestudio/src/repository/git.rs:118-138`): por um instante a view
  pode divergir do index real. por isso a precondição é conferida em dois
  pontos (task-58): no dispatcher, contra a projeção, e no worker, contra
  o index real, logo antes do `git commit`.
- o showcase embute o workspace do ide na página Git
  (`apps/uigallery/src/views/apps/git*`).
- existe teste de repositório real com tempfile em
  `rust/gitdrv/tests/real_repo.rs`.

## revisão do orquestrador (2026-09-23, antes do briefing)

- o que já existe na `main` e muda esta task: o `sem` completo (contrato,
  dispatcher, `Operable::execute(id, req)`, `reject_at_truth`), a ligação
  no compositor, e todo widget do comps com `.sem(id)` e `project`
  (`rust/engine/src/widgets`, adr semantic-binding-beside-the-scene). a regra "toda
  ação lê o estado que ela muda" vale: `stage`, `unstage` e `discard` leem
  o `staged` do arquivo; `set_value` da mensagem lê `text`.
- onde mora o `Operable`: um tipo só no lib do ide (nome sugerido
  `Workbench`) que junta `WorkspaceView`, `GitSession` e um `Dispatcher`.
  o binário do ide (`apps/idestudio/src/main.rs`, hoje `App { workspace, git,
  .. }`) e a página Git do showcase (`apps/uigallery/src/views/apps/git/desktop/mod.rs`)
  passam a segurar esse tipo, sem duplicar a ligação. a execução
  assíncrona guarda o `RequestId` que `execute` recebeu, e o
  `GitSession::apply` fecha o receipt (`complete` ou `reject_at_truth`)
  quando o `GitEvent` volta.
- esta rodada faz as fases a e b. a fase c (o experimento com agente
  externo e recorder) espera as tasks 61 e 63.

## modelo semântico do ide (proposta; o agente refina e o orquestrador aprova)

| id | role | state | ações (reads) |
|---|---|---|---|
| `changes` | List | `staged: Set`, `files: List` | |
| `changes.file[<path>]` | ListItem | `status: Text`, `staged: Bool` | `stage`, `unstage` (reads o próprio `staged`), `discard` (destrutiva, `Confirm::ForAgents`), `show_diff` |
| `diff` | Region | `target: Text`, `lines: List` (limitado) | |
| `commit.form` | Group | `open: Bool` | `open`, `cancel` |
| `commit.message` | TextInput | `text: Text` | `set_value` (constraints `required`, `editable`) |
| `commit.submit` | Button | `enabled` pelo form aberto, mensagem e staged não vazios | `submit` (reads `changes:staged`, `commit.message:text`) |
| `stacks.commit[<sha>]` | ListItem | `summary: Text`, `author: Text`; `files: List` nos `FILES_PROJECTED` (10) commits mais recentes do stack ativo e no último commit feito pelo ide (os caminhos que o commit muda contra o primeiro pai, renomeação nos dois caminhos, na ordem do git; task-85) | `show_diff` |

`selected_idx` pode aparecer como seleção da lista, nunca em `reads` de
`submit`.

## escopo

### fase a: o ide no contrato, em processo

1. `WorkspaceView` (ou o app do ide) implementa `Semantics` com o modelo
   acima e `Operable` com um handler por capacidade.
2. o clique em Commit, a tecla (`commit::Submit` do keymap) e o stage e
   unstage passam a ser `Request` com `Actor::Human` pelo dispatcher. o
   handler é um só.
3. `rust/gitdrv` ganha o commit condicional:
   `GitCommand::CommitIf { message, expect_staged: BTreeSet<String> }`. o
   worker lê o status real, compara o conjunto staged com o esperado e só
   então commita; se diferir, devolve a rejeição com o delta real
   (`GitEvent::CommitRejected { added, removed }`). o `GitEvent::Committed`
   passa a carregar também a lista de arquivos do commit (`git show
   --name-only`). mudança de API do `rust/gitdrv`, aprovada aqui pelo
   programa porque o ide é o único consumidor.
4. o receipt: `Accepted` no dispatcher, `Executing` quando o comando sai
   para o worker, `Completed { commit, files }` ou `Rejected(Stale)` no
   ponto de verdade quando o evento volta.

### fase b: handoff e rejeição visíveis

1. um agente em processo (`Actor::Agent { name: "test-agent" }`) nos
   testes, e o receipt aparecendo na UI: um toast ou uma linha de status
   com "recusado: B saiu da seleção" (comps, sem widget novo se `Toast`
   serve).
2. confirmação humana para `discard` pedido por agente (UI mínima com
   `Modal` de comps).
3. o shell do ide abre o `AgentPort` da task-61 quando ela estiver
   integrada. (2026-09-23: não abre. o binário `ide` não usa `wire`
   (`apps/idestudio/src/main.rs`); quem serve o `Workbench` pela porta é o
   `ide-lab` da fase c (`apps/agenteval/src/arms/plev.rs`). a porta no
   shell de janela do ide continua por fazer.)

### fase c: o experimento do plev (depende de 61 e 63)

um repositório de teste com várias alterações; a tarefa diz quais arquivos
entram no commit. o agente:

1. descobre os controles pela interface exposta pelo plev;
2. lê o diff pela mesma interface;
3. não usa screenshot;
4. não acessa o git diretamente;
5. seleciona os arquivos certos;
6. preenche a mensagem;
7. aciona o commit pelo caminho compartilhado da interface.

em uma das execuções, o humano altera a seleção depois que o agente
observou e antes que a ação dele seja executada; o contrato percebe que a
observação envelheceu e impede a ação. depois: abrir o `.monster` (63 e
64), navegar pela sessão, verificar o estado observado, a ação pedida, a
origem, o resultado, e comparar com o commit realmente criado.

métricas: sucesso, tempo, volume de dados e tokens enviados ao agente,
recuperação de erro, custo da gravação na engine. comparação contra
screenshot e contra uma alternativa estruturada existente (snapshot de
acessibilidade no estilo do Playwright MCP sobre um git client web
equivalente). se não existir comparação justa, a limitação é escrita no
resultado em vez de uma comparação fraca.

este é o checkpoint a do programa.

## fora de escopo

accessibility do ide (task-60 liga depois desta integração), gravação
(63), inspector (64), ações compostas (70).

## dependências

57, 58, 59 integradas para as fases a e b. 61 e 63 integradas para a fase
c.

## superfície sob responsabilidade

- `apps/idestudio/**`
- `rust/gitdrv/**`
- `apps/uigallery/src/views/apps/git*` (só se a mudança de API do ide
  exigir; mínimo)
- proposta de ADR [`0036-commit-checks-the-index-it-observed.md`](../../adr/Semantics/0036-commit-checks-the-index-it-observed/0036-commit-checks-the-index-it-observed.md)

proibido: `rust/accord`, `rust/engine/src/control/accessibility`,
`rust/engine/src/runtime/window`, o crate de protocolo da task-61.

## interfaces que consome

`Semantics`, `Operable`, `Dispatcher`, `Request`, `Receipt` (57, 58);
`SemId` opcional e helpers de projeção de comps (59); `AgentPort` (61,
fase c); API de gravação (63, fase c).

## interfaces que pode alterar

`rust/gitdrv` (`GitCommand`, `GitEvent`, `Repo::commit` ou um irmão
condicional), `apps/idestudio` inteiro.

## critérios de aceitação

testes em repositório git real num tempdir:

1. o agente observa `{A,B,C}` e "fix login"; o humano faz unstage de B
   pelo dispatcher; o submit do agente volta `Rejected(Stale)` com
   `removed: [B]`; `git log` não muda.
2. sem mudança no meio: aceito; o receipt termina com o sha igual a
   `git rev-parse HEAD` e `files` igual aos arquivos do commit (desde a task-85, `git diff-tree -r --no-renames --root --diff-merges=first-parent HEAD`; antes, `git show --name-only HEAD`).
3. mover o cursor da lista entre observe e submit: aceito.
4. o index muda por fora (o teste roda `git add` direto) entre a validação
   no dispatcher e a execução no worker: rejeitado no ponto de verdade,
   nenhum commit.
5. handoff: o agente faz stage e escreve a mensagem; o humano edita a
   mensagem; o submit do agente volta stale com o delta da mensagem; o
   agente observa de novo e o submit é aceito; os receipts guardam quem fez
   cada passo.
6. `discard` pedido por agente espera confirmação; humano confirma; o
   arquivo é descartado.

e ainda: o clique real e o `Actor::Test` produzem o mesmo resultado; a
página Git do showcase continua funcionando; `./tool/tasks/fullcheck.sh` verde.

## riscos e perguntas abertas

- corrida residual: um processo externo pode mudar o index entre a
  checagem do worker e o `git commit`. o git segura `index.lock` só dentro
  do próprio processo. fica documentado como fora do contrato no ADR.
- diff grande na observação do agente: `diff.lines` limitado e observação
  por escopo; tamanho medido na fase c.

## commits esperados

1. `task-62: projeção semântica do workspace do ide`
2. `task-62: commit condicional no rust/gitdrv com o delta real`
3. `task-62: ide implementa Operable; clique e tecla pelo dispatcher`
4. `task-62: receipt do commit até o sha e os arquivos`
5. `task-62: testes em repositório real (stale, aceito, cursor, ponto de verdade, handoff)`
6. `task-62: rejeição e confirmação visíveis na UI`
7. `task-62: AgentPort no shell do ide` (fase c)
8. `task-62: roteiro e resultado do experimento` (fase c)

## pare e reporte se

- o modelo semântico precisar de role, value ou campo que `sem` não tem;
- o `WorkspaceView` não conseguir ser `Operable` sem reestruturar o ide
  além do escopo;
- a mudança no `rust/gitdrv` afetar outro consumidor.

## Phases

1. [phases a and b closed](phase-ab-closing.md): integration in `8978602`, with the conditional commit, the receipt and the six acceptance criteria.
2. [phase c, first round](phase-c-round-1.md): design, protocol and result of checkpoint a, with hypotheses H1 to H5.
3. [phase c, second round](phase-c-round-2.md): the same experiment with the fixes from tasks 83, 84 and 85.
4. [phase c, third round](phase-c-round-3.md): the rule from task-89 and the person removing the file before the submit.
5. [phase c, fourth round](phase-c-round-4.md): the handoff, with and without the sentence about `--ask`.
6. [phase c, fifth round](phase-c-round-5.md): the refusal that points to `--ask`, without the sentence in the briefing.
