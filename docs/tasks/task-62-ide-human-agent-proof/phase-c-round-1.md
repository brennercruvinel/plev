---
id: task-62/phase-c-round-1
title: "task-62 phase c, first round: design, protocol and result (2026-09-23)"
project: plev
type: section
parent: task-62
status: reference
date: 2026-09-23
date_basis: text
recorded: 2026-09-23
topics: [agents, semantics, recording]
relations: [task-64, task-65, task-67, task-85, task-88, task-89, task-93]
source: docs/log/steps/task-62-ide-human-agent-proof.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase c: experiment design (orchestrator review, 2026-09-23)

Design, protocol and result of checkpoint a of task-62. The design was written before running and fixed the hypotheses and what would refute them: an LLM agent and a scripted one operate the ide only through the socket, against a real repository, and the result is compared with `git` on the command line. H1, H2, H3 and H5 were supported; H4 was refuted on cost, since the contract gains in safety and explanation, but not in bytes or in round trips.

a fase c é o checkpoint a do programa. o desenho abaixo é escrito antes
de rodar, com as hipóteses e o que as derruba, para que o resultado possa
ser negativo. a tese que ele testa: um agente de fora opera o ide só pelo
contrato, uma ação sobre estado que envelheceu é recusada com o delta, o
agente se recupera, e a sessão gravada explica e confere com o git real.

## o que já existe e o que falta

- existe, na `main`: o `Workbench` (`apps/idestudio/src/capability/mod.rs`)
  com `snapshot()`, `dispatch(req)`, `add_sink`, `apply(GitEvent)`,
  `render(compositor)`; o commit conferido em dois pontos
  (`truth.rs`, `GitCommand::CommitIf`); o rig de repositório real
  (`apps/idestudio/tests/support/mod.rs`: `Rig::new(files, staged)`, `until`,
  `git(args)`, `head`, `head_files`); o `AgentPort` com `open_with(Config,
  wake)` e `drain(&mut dyn Host)` (`rust/socket/src/port.rs`), o `Host`
  com `project`, `bounds`, `dispatch`, `add_sink`
  (`rust/socket/src/server.rs:33`); o `plev observe`/`plev act` (que sem
  `--from-revision` observa o alvo antes e age sobre o estado de agora);
  o `Recorder` com `sink()`, `observed(t, actor, scope, rev)`,
  `snapshot(t, &snap)`, `input(t, kind, at, target)`, `frame(t,
  &compositor)`, `persist(path)`, `stats()` e a leitura `Session::open`,
  `state_at`, `receipts_for`; o `headless::Renderer::draw(&mut
  compositor, clear)` pelo mesmo caminho de frame da janela.
- falta: o ide não depende de `wire` nem de `rec` (`apps/idestudio/Cargo.toml`);
  ninguém chama `Recorder::observed` fora dos testes do `rec`; o `Host`
  não avisa o app de uma observação servida; o `AgentPort` não conta
  bytes nem mensagens; o relógio do `Workbench` (`now()`, `mod.rs:315`) é
  privado, e os eventos do `rec` precisam do mesmo relógio dos receipts.

## o laboratório: `ide-lab`

um binário headless no crate do ide (`apps/agenteval/src/`, entrada
em `main.rs`), sem janela, que é o app sob experimento e o experimentador
ao mesmo tempo:

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

## os braços

- **A0, agente roteirizado pelo socket, sem LLM.** uma thread com o
  `wire::Client` faz o ciclo canônico: observa, põe os arquivos certos no
  index, escreve a mensagem, observa de novo, sofre a intervenção, tenta
  `submit` com `from_revision`, recebe `stale`, observa com `since`,
  tenta de novo, conclui, observa `stacks`. é o teste do laboratório e da
  intervenção antes de gastar token, roda no gate
  (`apps/agenteval/tests/experiment.rs`), e mede o custo puro do protocolo (latência
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

## hipóteses (escritas antes de rodar) e o que as derruba

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

## quem faz o quê

um agente constrói o laboratório, o braço A0 (teste no gate), o
construtor do braço B (repo, hook, shim) e o braço C, e escreve neste
arquivo como rodar cada braço; o orquestrador roda os braços A e B com
agentes novos, extrai tokens e comandos das transcrições, e escreve o
resultado. o agente do laboratório nunca é o agente do experimento.

## superfície da fase c

- `apps/idestudio/**` (o bin em `src/bin/lab/`, `Cargo.toml` com `wire` e
  `rec`, `tests/lab.rs`, `Workbench::now` público se preciso).
- `rust/socket/src/{server.rs,port.rs,lib.rs}`, só aditivo: um método
  `Host::observed(&mut self, actor: &Actor, scope: Option<&SemId>,
  revision: Revision)` com implementação padrão vazia, chamado pelo
  servidor depois de servir uma observação; contadores de tráfego
  (`Traffic`) lidos pelo app. os testes do `wire` continuam verdes.
- este arquivo (protocolo e como rodar).
- proibido: `rust/accord`, `rust/record` (consumir só; o que faltar é
  parada e reporte), `rust/engine`, `rust/engine/src/widgets`, `rust/gitdrv`,
  `apps/uigallery`.

## como rodar a fase c

o laboratório é o binário `ide-lab` (`apps/agenteval/src/`, unix
only; em outro alvo imprime e sai com 3). ele monta o repositório do
experimento num diretório temporário, sobe o `Workbench` sem janela com
o worker de git de verdade, abre o `AgentPort`, liga o `Recorder` no
relógio do workbench (`Workbench::now`), roda o humano roteirizado e, no
fim, reabre a sessão gravada e confere com o git real.

ambiente, em todo comando:

```sh
export CARGO_TARGET_DIR=<um target dir por agente>
export CARGO_INCREMENTAL=0
```

o repositório é sempre o mesmo, em `$TMPDIR`: um commit inicial e seis
alterações, três da tarefa (`src/login.rs` no index, `src/session.rs`
fora, `tests/login_test.rs` novo e fora) e três que não são
(`README.md` no index, `notes.txt` novo, `Cargo.toml` modificado). a
tarefa: "commit only the login fix: src/login.rs, src/session.rs,
tests/login_test.rs, message: fix login session timeout". nos braços
`plev` e `scripted` o caminho do repositório nunca é impresso; no braço
`git` ele é impresso, porque o agente precisa dele.

### braço A0, o agente roteirizado (controle, sem token)

```sh
cargo run -p agenteval -- --arm scripted \
    --human unstage-after-observe:src/session.rs --out <dir>
cargo run -p agenteval -- --arm scripted --human none --out <dir>
```

é o mesmo que `apps/agenteval/tests/experiment.rs` roda no gate. mede o protocolo
puro: bytes, idas e voltas, custo do contrato e da gravação, sem o ruído
do llm.

### braço A, o agente llm pelo contrato

o laboratório sobe e espera; o agente é outro processo.

```sh
cargo run -p agenteval -- --arm plev \
    --human unstage-after-observe:src/session.rs --out <dir> --app ide
```

ele imprime a tarefa, o nome do app e o caminho do socket, e fica no
loop até o primeiro `commit.submit` concluído (mais `--grace`, 1 s por
padrão) ou até `--timeout` (600 s por padrão). o agente recebe só a
tarefa e dois comandos, com o binário já compilado:

```sh
cargo build -p socket --bin plev
plev observe --app ide [--scope <id>] [--keys <k,k>] [--since <rev>]
plev act <target> <action> --from-revision <rev> [--param <json>]
```

a regra do braço A: agir sempre com `--from-revision` da observação em
que a decisão se apoiou. sem ela o `plev act` observa o alvo antes e age
sobre o estado de agora, o que nunca fica stale e anula a medição
daquela corrida; o relatório marca isso em `agent.stale_measurement_valid`
e diz por quê. proibido ao agente: `git`, ler arquivo, screenshot,
qualquer comando fora do `plev`.

para a corrida de controle, o mesmo comando com `--human none`.

### braço B, o git na linha de comando

```sh
cargo run -p agenteval -- --arm git \
    --human unstage-after-observe:src/session.rs --out <dir>
```

imprime o caminho do repositório e a linha `export PATH=<dir>/bin:$PATH`
que o orquestrador dá ao agente, instala o shim de `git` e, com
`--human`, o hook `pre-commit` que uma vez só roda
`git reset -q HEAD -- <path>` no instante do commit. o agente usa o git
normalmente (`status --porcelain`, `add`, `reset`, `commit`, `log`,
`show`); o laboratório observa o `.git` até um HEAD novo e então fecha.
sem `--human`, o commit bate com a tarefa e o exit é 0; com o hook, o
commit gravado é menor do que a tarefa, o git não avisa ninguém, e o
laboratório sai 1 dizendo isso. este braço não tem workbench, porta nem
gravação: o relatório diz "receipts: none; session: none".

### braço C, o volume de um screenshot

```sh
cargo run -p agenteval -- --arm screenshot --out <dir>
```

desenha o ide em 1280x800 na escala 1 e na 2 (um renderer, redimensionado
entre as duas), escreve `frame@1.png` e `frame@2.png` e imprime os bytes
do PNG e do RGBA cru de cada um ao lado dos bytes de uma observação
inteira e de uma com `--scope changes`, serializadas como o servidor
escreve. é comparação de volume de dados, não de capacidade, e a tabela
diz isso. sem GPU na máquina, ele avisa e segue sem os frames.

### o que fica em `out/`

| arquivo | conteúdo |
| --- | --- |
| `report.json` | o relatório inteiro: `traffic`, `agent` (observações, atos, forma de cada ato, recusas, recuperação, `stale_measurement_valid`), `intervention`, `stale` (com o HEAD no instante de cada recusa), `cost.contract`, `cost.recording`, `session`, `truth`, `notes`, `exit` |
| `session.monster` | a sessão gravada (magic `MONS`), reaberta pelo próprio laboratório com `Session::open`; `monster_player` recusa com a mensagem de sessão, sem panic |
| `frame@1.png`, `frame@2.png` | só no braço `screenshot` |
| `bin/git`, `git-log.jsonl`, `hooks/pre-commit`, `hook-fired` | só no braço `git`: o shim, uma linha json por invocação (hora, argumentos, bytes de stdout, exit), o hook e o marcador que o faz disparar uma vez só |

### exit codes

| code | significado |
| --- | --- |
| 0 | o commit existe, é exatamente o conjunto da tarefa e, quando há sessão, o receipt concluído bate com `git rev-parse HEAD` e com os arquivos do commit pela cli (`git diff-tree --no-renames` desde a task-85; `git show --name-only` antes) (MATCH) |
| 1 | um commit existe e difere: conjunto errado, ou a sessão discorda do git (MISMATCH) |
| 2 | não houve commit |
| 3 | erro do laboratório (argumento, repositório, porta, gravação) |

### o que ler no relatório antes de concluir qualquer coisa

- `agent.stale_measurement_valid`: falso quer dizer que a recusa por
  estado velho não foi exercida naquela corrida, mesmo que o humano
  tenha agido. o texto em `agent.stale_measurement` diz o motivo.
- `agent.acts[].form`: `from_revision` quando o ato se apoia numa
  revisão que esta porta serviu, `observed` quando não.
- `cost.recording.gpu`: falso quer dizer que a máquina não tem adapter e
  a trilha visual está vazia; a semântica continua completa.
- `notes`: projeção que não validou, frame não gravado, agente
  roteirizado que parou no meio.

os números medidos aqui são de build debug. release muda o custo do
contrato e da gravação, não os bytes.

nota (2026-09-23, task-88): o protocolo de medição mudou em três pontos,
depois das tasks 87 e 89.

- a pessoa tem dois cenários. `unstage-after-observe:<path>` é o de
  sempre, o das rodadas 1 e 2: ela age logo depois de uma observação
  servida, quando a tarefa está pronta, o que modela a pessoa que mexe
  logo depois de o agente olhar. `unstage-before-submit:<path>` é novo:
  ela age logo antes de o `submit` do agente chegar ao dispatcher,
  também quando a tarefa está pronta, o que modela a pessoa entre a
  decisão do agente e o `submit`, como o hook do braço `git` no instante
  do commit. esse exerce a checagem apoie-se o agente numa observação ou
  numa revisão que um estágio terminal carregou.
- `agent.stale_measurement_valid` é pelo valor: válida quando um
  `submit` que chegou ao dispatcher depois da pessoa carrega em
  `observed` o `changes:staged` de antes dela (o valor visto) e o app
  tinha outro naquele dispatch (o valor vivo). `agent.stale_measured`
  traz os dois, e o texto diz por que é válida ou não. a medição por
  revisão saiu do veredito: um ato encadeado num estágio terminal se
  apoia numa revisão que nenhuma observação serviu, e a revisão é hash
  do que o app projeta, então o mundo que o agente repõe tem a revisão
  que tinha quando a pessoa agiu. `form`, `revision_served_index`,
  `observations_between` e `fresh_before` ficam como informação. um ato
  sem `--from-revision` também é medido pelo que carregou, em vez de
  anular a corrida.
- `agent.protocol_refusals` conta os atos recusados antes do dispatcher
  (`before_dispatch`, em todo braço). o tipo de cada um
  (`not_observed` com as reads, `unknown_revision`) só o lado do agente
  vê: o roteirizado guarda cada um; no braço `plev`, o tipo está na
  transcrição do agente.

`--arm scripted --scripted chain` é o roteirizado que encadeia o
primeiro `submit` na revisão do estágio do `set_value`, sem observar.
com a regra da task-89 esse `submit` é recusado stale pela própria
escrita da mensagem, com `acted` vazio: um estágio terminal não mostra
estado. com `unstage-after-observe` a pessoa chega tarde para ele (age
na observação da recuperação) e a medição sai inválida, dizendo isso;
com `unstage-before-submit` ela sai válida.

nota (2026-09-24, task-93): a pessoa também responde ao modal.

- `--answer none|allow|decline` e `--answer-after <ms>` (padrão `none` e
  0): a mesma pessoa roteirizada responde a todo request de agente que o
  dispatcher deixa esperando uma pessoa (a pergunta de um agente, `plev
  act --ask`, ou uma ação que pede confirmação, como o `discard`),
  `<ms>` depois de ele começar a esperar, com um clique no botão do modal
  (Allow ou Cancel): o caminho de uma pessoa na tela, pelo dispatcher
  como `Actor::Human`, gravado como input dela. combina com `--human`:
  ela ainda tira o arquivo, e agora também responde. `none` é o de
  antes: o request espera até o fim da corrida.
- o relatório ganha `agent.questions` (a pergunta, a resposta,
  `answer_ms`, o modal, como terminou, o próximo ato do agente e, numa
  palavra, o que ele fez depois: `then`), `agent.questions_asked`,
  `person_answers`, `run.no_commit_because` e `agent.over_the_person`
  (atos sem pergunta que decidem sobre o que a pessoa mexeu, depois de uma
  recusa que a nomeou).
- `--scripted ask` é o roteirizado que, recusado stale com a pessoa em
  `acted`, pergunta antes de recolocar o arquivo. com `allow` commita os
  três (exit 0); com `decline` para sem commit (exit 2), e
  `run.no_commit_because` diz que foi ela.
- o experimento do handoff com LLM roda `--arm plev --human
  unstage-before-submit:src/session.rs` com `--answer allow` e com
  `--answer decline`; o protocolo está no handoff da task-93.

## resultado da fase c (checkpoint a, 2026-09-23)

rodado pelo orquestrador com o `ide-lab` da `main` em `d26c9b1`, build de
debug, num M4. os agentes LLM são agentes novos, sem contexto do
repositório, com o briefing da seção anterior; a transcrição de cada um
foi auditada por script (turnos, tokens por tipo, todo comando de shell,
comandos fora do permitido). relatórios, sessões e logs em
`target/agents/exp/{a0,a1,a2,b1,b2,b3,c}/`.

### as corridas

| corrida | braço | intervenção | resultado | exit |
| --- | --- | --- | --- | --- |
| A0 | roteirizado pelo socket, sem LLM | sim | 1 recusa stale nomeando `src/session.rs`, HEAD igual na recusa, recuperação, MATCH sessão x git | 0 |
| A0 | roteirizado | não | 0 recusas, MATCH | 0 |
| A2 | LLM pelo contrato | sim | 1 recusa stale válida, recuperação, receipt igual ao git, três arquivos | 0 |
| A1 | LLM pelo contrato | não | 0 recusas, três arquivos, confirmado pela interface (`stacks.commit[sha]` e `show_diff`) | 0 |
| B2 | LLM pelo `git` | hook no commit (graça 1 s) | commit com 2 arquivos, sem aviso do git; o agente viu "2 files changed" e tentou `show --name-only`, mas o laboratório já tinha derrubado o repositório | 1 |
| B3 | LLM pelo `git` | hook no commit (graça 180 s) | commit `cfe1156` com 2 arquivos; o agente viu "2 files changed", investigou, re-adicionou e fez `commit --amend` para `e6116c6` com os três | 0 |
| B1 | LLM pelo `git` | não | três arquivos, confirmado por `show --name-only` | 0 |
| C | volume de screenshot, sem LLM | | PNG 1280x800 = 62.939 B; observação inteira = 6.359 B (14 nós); `--scope changes` = 4.975 B; RGBA cru 4 MB | |

### as hipóteses

- **H1, suficiência do contrato: sustentada.** nas duas corridas A o
  agente concluiu com exatamente os três arquivos e a mensagem certa, sem
  nenhum comando fora do `plev` (auditoria: A2 18 comandos, A1 12; os
  únicos "fora" foram um `sleep 2` e uma variável de shell na frente do
  próprio `plev`, declarados pelo agente). ele descobriu sozinho que o
  form precisa ser aberto antes da mensagem (`enabled: false`), usou
  escopo e delta sem ser ensinado, e no A1 confirmou o commit lendo o
  nó novo em `stacks` e o diff dele.
- **H2, stale-state rejection: sustentada.** no A2 o `submit` apoiado na
  revisão servida antes da intervenção voltou `rejected` `stale` com o
  delta exato (`changes:staged` perdeu `src/session.rs`), o HEAD não
  mudou, o agente leu o delta, observou de novo, repôs o arquivo e o
  segundo `submit` concluiu com sha e arquivos iguais aos do git
  (`stale_measurement_valid: true`). todos os oito atos carregaram
  `from_revision`.
- **H3, gravação: sustentada.** a sessão do A2 tem 67 eventos e 15
  frames em 181.586 bytes (152.594 semânticos, 28.764 visuais); o
  laboratório reabriu o arquivo e leu de volta o receipt concluído, igual
  ao `git rev-parse HEAD` e ao `git show --name-only`; o teste do
  roteirizado prova `state_at` antes e depois da intervenção com o
  arquivo saindo de `changes:staged`.
- **H4, comparação: o plev ganha em segurança e explicação, e não ganha
  em bytes nem em idas e voltas.** no `git`, a intervenção equivalente
  produziu um commit diferente da tarefa sem nenhum aviso (B2 e B3, exit
  0 do `git commit`). o agente diligente percebeu pelo "2 files changed",
  mas só conseguiu saber o que faltava investigando depois, e consertou
  reescrevendo o commit (`--amend`), o que só funciona porque o commit
  era local; num branch compartilhado seria um segundo commit ou um
  force push. no plev o commit errado nunca existiu: a recusa veio antes
  do efeito, nomeando o arquivo. em volume, o `git` é muito mais enxuto:
  1.459 bytes de saída em 20 invocações (B3) contra 22.040 bytes em 30
  mensagens (A2); e o contrato custa mais idas e voltas (o `plev` abre
  uma conexão por comando, 30 no A2). contra screenshot, uma observação
  inteira é 9,9 vezes menor que um PNG a escala 1, e isso é volume de
  dados, não capacidade: nenhum agente por screenshot foi rodado.
- **H5, validade: nenhuma corrida anulada.** os dois agentes A usaram
  `--from-revision` em todo ato.

### os números

| | A2 (plev, intervenção) | A1 (plev, controle) | B3 (git, intervenção) | B1 (git, controle) |
| --- | --- | --- | --- | --- |
| turnos do modelo | 15 | 13 | 11 | 7 |
| tokens de saída | 10.510 | 7.747 | 8.857 | 3.875 |
| contexto novo (cache criado) | 43.054 | 40.999 | 33.998 | 28.689 |
| contexto relido (cache lido) | 748.597 | 629.534 | 469.273 | 273.973 |
| comandos de shell | 18 | 12 | 19 | 8 |
| bytes enviados ao agente | 22.040 | 21.639 | 1.459 | (456 no B2) |
| mensagens / invocações | 30 in, 52 out | 28 in, 49 out | 20 | 8 |
| observações (snapshot / delta) | 2 / 5 | 4 / 3 | | |
| atos | 8 | 7 | | |
| recusas stale | 1 | 0 | (hook silencioso) | |
| primeira mensagem ao commit | 109 s | 66 s | ~57 s ao primeiro commit, +57 s até o amend | ~40 s |
| tempo do agente (transcrição) | 3 min 15 s | 2 min 31 s | 2 min 40 s | 1 min 12 s |
| sessão gravada | 181.586 B, 67 eventos, 15 frames | 162.817 B, 61 eventos, 15 frames | nenhuma | nenhuma |

custo do contrato (lado do servidor, debug): 254,8 us por
`bench.snapshot()` e 326,1 us por `bench.dispatch()` no A2, 2,6 ms por
mensagem servida dentro de `drain` (0,06 % dos 149 s da corrida); no A1
454 us por mensagem. custo da gravação: 138 us por `rec.snapshot`, 289 us
por `rec.frame`, 1,2 ms por draw de GPU do frame do ide. no roteirizado
sem LLM (A0) a corrida inteira levou 1.313 ms com 27 mensagens: o tempo
do experimento é o modelo pensando, não o contrato.

### o que o experimento mostrou além das hipóteses

- **falha de protocolo, minha:** o laboratório fechava 1 s depois do
  commit e derrubava a porta e o repositório. no A2 o agente não pôde
  confirmar pelo `stacks` (três `observe` com exit 3), e no B2 o agente do
  `git` não pôde rodar o `show --name-only` que teria revelado o commit
  de dois arquivos. corrigi para 180 s de graça nas corridas A1, B1 e B3;
  os números de A2 e B2 valem, a verificação final deles não.
- **ergonomia do contrato, levantada pelos agentes:** o delta de
  `node_added` traz só o id (obriga uma segunda observação); o nó do
  commit em `stacks` não expõe a lista de arquivos como estado; o aviso
  em `notices` diz que a recusa aconteceu mas não quem agiu (o ator do
  request 6); a regra "a checagem de stale é por `reads`, não por
  revisão" foi deduzida, não lida; o `set_value` conclui sem eco do
  valor. tudo isso é candidato à task-64 (inspector) e a um ajuste do
  protocolo, e nenhum impediu a tarefa.
- **uma corrida no ide, achada pelo roteirizado:** um `Refresh` que
  responde a uma mutação anterior pode carregar um status lido antes de
  uma mutação posterior e tirar do `changes:staged` um arquivo já
  staged; o roteirizado espera o index parar. é defeito do ide, não do
  contrato, e vai virar task.
- **o `plev` abre uma conexão por comando** (30 no A2). um agente que
  mantivesse a conexão gastaria menos idas e voltas; é decisão de
  ferramenta, não de contrato.

### decisão pedida (checkpoint a)

a hipótese central sobreviveu ao experimento que podia derrubá-la, e a
comparação honesta diz onde ela não ganha. recomendação do orquestrador:
seguir para a onda 5 (task-64 inspector, task-65 pixel para código,
task-67 regressão estrutural) levando as três correções pequenas que os
agentes apontaram (nó inteiro no `node_added`, `files` no nó do commit,
ator na recusa), e a corrida do `Refresh` no ide como task própria. o que
não entra ainda: IR, superfícies geradas, adaptativo, como pedido.
