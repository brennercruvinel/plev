---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-23
domain: generative-runtime
task: task-62
wave: 4
depends: [task-57, task-58, task-59]
depends-phase-c: [task-61, task-63]
branch: task/62-ide-proof
---

# task-62: prova no ide: stale-state rejection e handoff humano-agente

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

- o caminho do commit: `crates/ide/src/views/commit_form.rs`
  (`hit_test_click` devolve `CommitFormAction::Commit`) ->
  `crates/ide/src/views/workspace/input.rs:47` -> `submit_commit`
  (`:259-267`) empurra `UiRequest::Commit { message }`
  (`crates/ide/src/views/workspace/mod.rs:115-123`) -> `GitSession`
  traduz para `GitCommand::Commit` (`crates/ide/src/session.rs:76`) ->
  worker -> `crates/git/src/repo.rs:187` roda `git commit --message` e
  `git rev-parse HEAD`.
- "arquivos selecionados" no ide é o index: `FileEntry.staged`
  (`crates/ide/src/views/unassigned_view.rs:51-55`). stage e unstage vão
  por `UiRequest::Stage`/`Unstage`.
- `UnassignedView.selected_idx` é a linha em destaque (cursor). não entra
  no commit. é o exemplo perfeito de estado visual que não pode invalidar
  a ação.
- a view é otimista e reconcilia depois de cada mutação com um refresh
  completo (`crates/ide/src/session.rs:118-138`): por um instante a view
  pode divergir do index real. por isso a precondição é conferida em dois
  pontos (task-58): no dispatcher, contra a projeção, e no worker, contra
  o index real, logo antes do `git commit`.
- o showcase embute o workspace do ide na página Git
  (`crates/showcase/src/view/apps/git*`).
- existe teste de repositório real com tempfile em
  `crates/git/tests/real_repo.rs`.

## fases a e b fechadas (2026-09-23)

integradas na `main` em `8978602` (sete commits), rebaseadas pelo
orquestrador sobre a 63 e a 73 sem conflito, gate verde (1809 testes). os
seis critérios de aceite passam em repositório real com o worker de
verdade, mais o clique real contra `Actor::Test` e o commit duplicado que
só commita uma vez; a suíte passou oito vezes seguidas. ADR:
`docs/ADR/commit-checks-the-index-it-observed.md`.

decisões aceitas: o nó de arquivo tem `unstaged` além de `staged` (uma
path com mudança nos dois é uma linha por lado mas um nó só), e
`stage`/`unstage`/`discard`/`ignore` leem os dois; `ignore` virou
capacidade; `changes:selected` é navegação, projetado e lido por ninguém;
`commit.submit` declarado à mão para não ter `activate` e `submit` para a
mesma coisa; o workspace não fica inerte com overlay aberto (agentes
seguem agindo, as checagens garantem); o keymap continua nomeando ações do
engine, mapeadas para capacidades num lugar só do Workbench.

limitações que seguem:

- a resposta do worker é casada com o request pela ordem (FIFO por tipo),
  correto porque há um worker só e ele responde em ordem; se um dia houver
  mais de um, o `RequestId` precisa ir e voltar no comando.
- fechar o modal sem confirmar deixa o receipt aguardando: falta recusar
  uma confirmação no `sem`. virou a task-58b.
- o modal de confirmação não é projetado, então não há confirmação por
  tecnologia assistiva ainda (entra com a 58b e a 60).
- resta a corrida entre a leitura do index pelo worker e o `git commit`
  por outro processo; o ADR descreve o caminho por write-tree/commit-tree
  que a fecharia.
- o `Workbench` não é `Send` (o dispatcher não é): a task-61 leva os
  requests do socket para a thread da UI.

falta a fase c: o experimento com agente externo pelo socket (61) e a
sessão gravada (63).

## revisão do orquestrador (2026-09-23, antes do briefing)

- o que já existe na `main` e muda esta task: o `sem` completo (contrato,
  dispatcher, `Operable::execute(id, req)`, `reject_at_truth`), a ligação
  no compositor, e todo widget do comps com `.sem(id)` e `project`
  (`crates/comps`, adr semantic-binding-beside-the-scene). a regra "toda
  ação lê o estado que ela muda" vale: `stage`, `unstage` e `discard` leem
  o `staged` do arquivo; `set_value` da mensagem lê `text`.
- onde mora o `Operable`: um tipo só no lib do ide (nome sugerido
  `Workbench`) que junta `WorkspaceView`, `GitSession` e um `Dispatcher`.
  o binário do ide (`crates/ide/src/main.rs`, hoje `App { workspace, git,
  .. }`) e a página Git do showcase (`crates/showcase/src/view/apps/git/desktop.rs`)
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
| `stacks.commit[<sha>]` | ListItem | `summary: Text`, `author: Text` | `show_diff` |

`selected_idx` pode aparecer como seleção da lista, nunca em `reads` de
`submit`.

## escopo

### fase a: o ide no contrato, em processo

1. `WorkspaceView` (ou o app do ide) implementa `Semantics` com o modelo
   acima e `Operable` com um handler por capacidade.
2. o clique em Commit, a tecla (`commit::Submit` do keymap) e o stage e
   unstage passam a ser `Request` com `Actor::Human` pelo dispatcher. o
   handler é um só.
3. `crates/git` ganha o commit condicional:
   `GitCommand::CommitIf { message, expect_staged: BTreeSet<String> }`. o
   worker lê o status real, compara o conjunto staged com o esperado e só
   então commita; se diferir, devolve a rejeição com o delta real
   (`GitEvent::CommitRejected { added, removed }`). o `GitEvent::Committed`
   passa a carregar também a lista de arquivos do commit (`git show
   --name-only`). mudança de API do `crates/git`, aprovada aqui pelo
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
   integrada.

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

- `crates/ide/**`
- `crates/git/**`
- `crates/showcase/src/view/apps/git*` (só se a mudança de API do ide
  exigir; mínimo)
- proposta de ADR `docs/ADR/commit-checks-the-index-it-observed.md`

proibido: `crates/sem`, `crates/engine/src/accessibility`,
`crates/engine/src/window`, o crate de protocolo da task-61.

## interfaces que consome

`Semantics`, `Operable`, `Dispatcher`, `Request`, `Receipt` (57, 58);
`SemId` opcional e helpers de projeção de comps (59); `AgentPort` (61,
fase c); API de gravação (63, fase c).

## interfaces que pode alterar

`crates/git` (`GitCommand`, `GitEvent`, `Repo::commit` ou um irmão
condicional), `crates/ide` inteiro.

## critérios de aceitação

testes em repositório git real num tempdir:

1. o agente observa `{A,B,C}` e "fix login"; o humano faz unstage de B
   pelo dispatcher; o submit do agente volta `Rejected(Stale)` com
   `removed: [B]`; `git log` não muda.
2. sem mudança no meio: aceito; o receipt termina com o sha igual a
   `git rev-parse HEAD` e `files` igual a `git show --name-only HEAD`.
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
página Git do showcase continua funcionando; `./script/gate` verde.

## riscos e perguntas abertas

- corrida residual: um processo externo pode mudar o index entre a
  checagem do worker e o `git commit`. o git segura `index.lock` só dentro
  do próprio processo. fica documentado como fora do contrato no ADR.
- diff grande na observação do agente: `diff.lines` limitado e observação
  por escopo; tamanho medido na fase c.

## commits esperados

1. `task-62: projeção semântica do workspace do ide`
2. `task-62: commit condicional no crates/git com o delta real`
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
- a mudança no `crates/git` afetar outro consumidor.

## fase c: desenho do experimento (revisão do orquestrador, 2026-09-23)

a fase c é o checkpoint a do programa. o desenho abaixo é escrito antes
de rodar, com as hipóteses e o que as derruba, para que o resultado possa
ser negativo. a tese que ele testa: um agente de fora opera o ide só pelo
contrato, uma ação sobre estado que envelheceu é recusada com o delta, o
agente se recupera, e a sessão gravada explica e confere com o git real.

### o que já existe e o que falta

- existe, na `main`: o `Workbench` (`crates/ide/src/workbench/mod.rs`)
  com `snapshot()`, `dispatch(req)`, `add_sink`, `apply(GitEvent)`,
  `render(compositor)`; o commit conferido em dois pontos
  (`truth.rs`, `GitCommand::CommitIf`); o rig de repositório real
  (`crates/ide/tests/support/mod.rs`: `Rig::new(files, staged)`, `until`,
  `git(args)`, `head`, `head_files`); o `AgentPort` com `open_with(Config,
  wake)` e `drain(&mut dyn Host)` (`crates/wire/src/port.rs`), o `Host`
  com `project`, `bounds`, `dispatch`, `add_sink`
  (`crates/wire/src/server.rs:33`); o `plev observe`/`plev act` (que sem
  `--from-revision` observa o alvo antes e age sobre o estado de agora);
  o `Recorder` com `sink()`, `observed(t, actor, scope, rev)`,
  `snapshot(t, &snap)`, `input(t, kind, at, target)`, `frame(t,
  &compositor)`, `persist(path)`, `stats()` e a leitura `Session::open`,
  `state_at`, `receipts_for`; o `headless::Renderer::draw(&mut
  compositor, clear)` pelo mesmo caminho de frame da janela.
- falta: o ide não depende de `wire` nem de `rec` (`crates/ide/Cargo.toml`);
  ninguém chama `Recorder::observed` fora dos testes do `rec`; o `Host`
  não avisa o app de uma observação servida; o `AgentPort` não conta
  bytes nem mensagens; o relógio do `Workbench` (`now()`, `mod.rs:315`) é
  privado, e os eventos do `rec` precisam do mesmo relógio dos receipts.

### o laboratório: `ide-lab`

um binário headless no crate do ide (`crates/ide/src/bin/lab.rs`), sem
janela, que é o app sob experimento e o experimentador ao mesmo tempo:

1. constrói um repositório real num diretório temporário (o padrão do
   rig): um commit inicial e seis arquivos alterados, parte deles já no
   index, misturando os que pertencem à tarefa e os que não pertencem. o
   caminho do repositório não é mostrado ao agente.
2. sobe o `Workbench`, conecta o `GitClient`, bombeia os eventos até o
   status, o log e as branches chegarem.
3. abre o `AgentPort` como `ide` no diretório padrão de sockets
   (`open_with`, opt-in explícito do laboratório), para o `plev --app ide`
   achar.
4. liga o `Recorder`: `add_sink` no dispatcher, `snapshot` a cada mudança
   (evento do git, ato, intervenção), `observed` a cada observação
   servida, `input` na ação do humano roteirizado, e a trilha visual com
   frames reais do ide pelo `headless::Renderer` (1280 por 800, escala 1),
   um frame por mudança, como o render on demand manda. no fim, `persist`
   em `<out>/session.monster`, e o próprio laboratório reabre o arquivo
   com `Session::open` e lê de volta os receipts.
5. o humano roteirizado (`--human <cenário>`), determinístico e com hora
   registrada:
   - `none`: sem intervenção (a corrida de controle);
   - `unstage-after-observe:<path>`: quando o agente já deixou no index
     todos os arquivos da tarefa e a mensagem está escrita (a precondição
     do `submit` passaria), na observação seguinte que o laboratório
     serve ao agente, logo depois de servi-la, o humano tira `<path>` do
     index pelo dispatcher (`Actor::Human`, `unstage`). a observação do
     agente está velha antes de ele agir. dispara uma vez só.
6. métricas, impressas como JSON e como tabela no fim: conexões,
   mensagens e bytes em cada sentido (contadores aditivos no
   `AgentPort`/`Server`), observações (snapshot e delta), atos, estágios
   enviados, recusas por `stale`, erros; tempo do primeiro `hello` ao
   estágio terminal do `submit` concluído; custo do contrato do lado do
   servidor (tempo dentro de `drain`: projeção, `check`, serialização,
   por mensagem, e como fração do tempo total); custo da gravação (tempo
   dentro de `snapshot`/`frame`/`sink`, por evento e por frame);
   `Stats` do recorder, bytes da trilha semântica e da visual, tamanho do
   `.monster`; e a verdade: `git rev-parse HEAD`, `git show --name-only
   --format= HEAD`, `git log --oneline` no fim, comparados com o
   `commit` e `files` do receipt lido da sessão. sai com 0 quando sessão
   e git conferem.
7. termina em `--until commit` (primeiro `submit` concluído, mais um
   segundo para os estágios saírem) ou em `--timeout <s>` (600 por
   padrão). sempre escreve a sessão e o relatório.

condição de validade: um ato do agente sem `--from-revision` age sobre
uma observação nova (o `plev act` observa o alvo antes) e anula a medição
de stale daquela corrida. o laboratório registra a forma de cada ato (o
`asked` do primeiro estágio) e o relatório diz quando isso aconteceu, em
vez de esconder.

### os braços

- **A0, agente roteirizado pelo socket, sem LLM.** uma thread com o
  `wire::Client` faz o ciclo canônico: observa, põe os arquivos certos no
  index, escreve a mensagem, observa de novo, sofre a intervenção, tenta
  `submit` com `from_revision`, recebe `stale`, observa com `since`,
  tenta de novo, conclui, observa `stacks`. é o teste do laboratório e da
  intervenção antes de gastar token, roda no gate
  (`crates/ide/tests/lab.rs`), e mede o custo puro do protocolo (latência
  por ida e volta, bytes) sem o ruído do LLM.
- **A, agente LLM pelo contrato.** um agente novo, sem contexto do repo,
  recebe a tarefa e só dois comandos: `plev observe` e `plev act`, com o
  binário já compilado, `--app ide`, `--name` próprio, e a regra de agir
  com `--from-revision` da observação em que se baseou. proibido: `git`,
  ler arquivo, screenshot, qualquer outro comando. a transcrição dele é a
  auditoria: cada comando fora do `plev` conta, e os tokens saem dos
  campos de uso de cada turno (entrada, cache lido, cache criado,
  saída). corridas: A1 controle (`none`), A2 intervenção, A3 intervenção
  repetida se o custo permitir (variância).
- **B, alternativa estruturada convencional: o `git` na linha de
  comando.** o mesmo modelo, agente novo, a mesma tarefa, com `git`
  (`status --porcelain`, `add`, `reset`, `commit`, `log`, `show`) num
  repositório idêntico cujo caminho ele recebe. a intervenção equivalente
  é um hook `pre-commit` instalado pelo laboratório (`--arm git`) que,
  uma vez só, tira `<path>` do index no momento do commit: o git então
  commita o conjunto que sobrou, sem avisar. um shim de `git` no PATH do
  agente registra cada comando, os bytes de saída e a hora (o volume de
  dados e as idas e voltas do braço B). o que se mede: o commit criado
  bate com a tarefa? o agente percebeu? quantos bytes, comandos, tokens e
  segundos? e o que sobrou para explicar depois (nada equivalente a um
  receipt ou a uma sessão). a assimetria fica escrita: no B a
  intervenção dispara no instante do commit, no A logo depois da
  observação; as duas simulam "um humano mexeu no index entre a leitura e
  a escrita do agente".
- **C, volume de dados de um screenshot, sem LLM.** o laboratório escreve
  um PNG do frame do ide em escala 1 e 2 e informa os bytes (PNG e RGBA
  cru), ao lado dos bytes de uma observação completa e de uma por escopo.
  é comparação de volume, não de capacidade, e o relatório diz isso.

### hipóteses (escritas antes de rodar) e o que as derruba

- **H1, suficiência do contrato.** um agente LLM novo, com a tarefa e os
  dois comandos, conclui o commit com exatamente o conjunto de arquivos
  pedido, sem nenhum comando fora do `plev`. derruba: não concluir,
  conjunto errado, ou qualquer comando fora do `plev` na transcrição.
- **H2, stale-state rejection.** na corrida com intervenção, o `submit`
  do agente apoiado na revisão anterior à intervenção volta `rejected`
  `stale` com o delta nomeando exatamente o arquivo retirado (em
  `changes:staged`), o HEAD não muda; o agente observa de novo, repõe o
  arquivo e conclui; o receipt final tem sha e arquivos iguais aos do
  git. derruba: um commit criado pelo `submit` velho; um delta que não
  nomeia o arquivo; o agente não se recuperar; receipt diferente do git.
- **H3, gravação.** o `.monster` guarda as observações, a recusa com o
  delta, o receipt concluído e o estado em cada instante: `state_at`
  antes e depois da intervenção mostra o arquivo saindo de
  `changes:staged`, e `receipts_for("commit.submit")` lido do arquivo é
  igual ao git. tamanho da sessão e custo da gravação medidos. derruba:
  estado ou receipt ausente ou diferente; arquivo que não abre.
- **H4, comparação, sem resultado favorito.** sob a intervenção
  equivalente, o braço B commita um conjunto diferente da tarefa sem ser
  avisado, a não ser que o agente confira de novo antes ou depois; bytes
  e tokens por braço lado a lado. o plev pode perder em bytes para um
  `git status --porcelain`, e isso é escrito como está.
- **H5, validade.** um ato sem `--from-revision` anula a medição de stale
  da corrida; o relatório diz.

### quem faz o quê

um agente constrói o laboratório, o braço A0 (teste no gate), o
construtor do braço B (repo, hook, shim) e o braço C, e escreve neste
arquivo como rodar cada braço; o orquestrador roda os braços A e B com
agentes novos, extrai tokens e comandos das transcrições, e escreve o
resultado. o agente do laboratório nunca é o agente do experimento.

### superfície da fase c

- `crates/ide/**` (o bin `lab.rs`, `Cargo.toml` com `wire` e `rec`,
  `tests/lab.rs`, `Workbench::now` público se preciso).
- `crates/wire/src/{server.rs,port.rs,lib.rs}`, só aditivo: um método
  `Host::observed(&mut self, actor: &Actor, scope: Option<&SemId>,
  revision: Revision)` com implementação padrão vazia, chamado pelo
  servidor depois de servir uma observação; contadores de tráfego
  (`Traffic`) lidos pelo app. os testes do `wire` continuam verdes.
- este arquivo (protocolo e como rodar).
- proibido: `crates/sem`, `crates/rec` (consumir só; o que faltar é
  parada e reporte), `crates/engine`, `crates/comps`, `crates/git`,
  `crates/showcase`.
