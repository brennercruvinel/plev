---
id: task-96/phase-a-surface
title: "task-96, phase a: the generated surface in the ide (probe, handoff and close-out)"
project: plev
type: section
parent: task-96
status: reference
date: 2026-09-25
date_basis: text
recorded: 2026-09-25
topics: [generated-surfaces, ir, agents, llm]
relations: [task-69, task-80, task-85, task-93]
evidence: [73a318a7, 6bacaa3d, f53667c0, 73704549, f8f39298, e315e11e, 2e6c9e9a, 07650665, 9dadd352, 855b1a4d, 3df1297f, 205e90b3, 1283cc5b]
source: docs/log/steps/task-96-ide-generated-surface.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase a (2026-09-25): probe, handoff and close-out

Phase a is the task's first pass: a read-only probe, done before any code, establishes what the ide's projection gives for reviewing commits and where the intent line and the host can live in the window. The handoff records the implementation on the branch `task/96a-ide-surface`, with the intent line, the host, the registry and the diff, the rules, the acts through the dispatcher, the lab and Claude. The orchestrator's close-out marks the phase as integrated; demo 5, recorded in the lab, came out mixed against the fixed panel: better on files, worse on the diff.

## sonda (2026-09-25, antes de qualquer código)

base: `main` @ `7d503841`, branch `task/96a-ide-surface`. só leitura:
nenhum teste novo rodou. toda afirmação sobre código tem arquivo e linha.

### (a) o que a projeção do ide dá para revisar commits

- `stacks` (list) tem só `branch` (`apps/idestudio/src/capability/project.rs:166-168`).
  cada commit é um nó filho `stacks.commit[<sha>]` (list item) com
  `summary`, `author` e a ação `show_diff`, que lê `diff:target`
  (`:186-196`). não há um `List` com os shas dos commits: os commits são
  os filhos de `stacks`, na ordem da projeção, a pilha ativa primeiro
  (`apps/idestudio/src/repository/adapters.rs:63`, `sort_by_key(|s| !s.is_active)`).
- os arquivos de cada commit (`files`, task-85) vão só para os
  `FILES_PROJECTED` (10) mais novos da pilha ativa e para o último commit
  que o workbench fez (`project.rs:46`, `:176`, `:193-195`). três commits
  do topo têm `files`.
- o diff: um só, a região `diff` com `target` (o id cujo diff aparece),
  `lines` (as primeiras `DIFF_LINES` = 200, com `+`, `-`, espaço ou o
  cabeçalho do hunk, que traz `path: @@ ...`) e `count`
  (`project.rs:40`, `:112-133`). um commit mostra o seu diff assim: o
  `show_diff` num `stacks.commit[<sha>]` vira `Op::ShowCommitDiff`
  (`apps/idestudio/src/capability/execute.rs:58-59`), o `execute` põe o
  `target` e esvazia as linhas na hora (`execute.rs:123-142`,
  `apps/idestudio/src/interfaces/panels/diff.rs:84-94`) e pede o diff ao worker
  (`apps/idestudio/src/repository/git.rs:194-201`); as linhas chegam no
  `GitEvent::DiffCommit` (`session.rs:298`). o receipt do `show_diff`
  termina `completed` no `execute`, antes das linhas.
- o que falta no registro para a superfície mostrar o diff de cada
  commit: nada, se o diff é um por vez, como no painel fixo. a superfície
  liga uma tabela a `diff:lines` e um texto a `diff:target`, e o botão de
  cada commit liga o `show_diff` dele: a pessoa abre cada diff pela
  superfície, que é o roteiro da demo 5 ("a pessoa abre cada diff pela
  superfície"). os três diffs ao mesmo tempo pediriam um estado por
  commit (um `diff` em cada `stacks.commit[<sha>]`), uma leitura do git
  por commit e mudariam o golden `rust/plevir/tests/fixtures/idestudio-workbench.snapshot.json`,
  que `apps/idestudio/tests/plevir_fixture.rs:31-66` confere a cada corrida: fica
  de fora, anotado como P3. nenhum tipo do `sem` muda. a projeção fixa do
  ide não muda em nada: o bloco gerado só entra quando o host liga ele
  (item b), e o golden continua igual.

### (b) onde a linha e o host moram na janela do ide

- a janela desenha só o workbench: `renderer.rs:25-27` abre o frame e
  chama `Workbench::render` (`apps/idestudio/src/capability/mod.rs:234`), que
  desenha o `WorkspaceView` na janela inteira (`apps/idestudio/src/interfaces/panels/layout/render.rs:8-128`).
  a janela dá o tamanho ao workspace por `workspace_mut().resize`
  (`apps/idestudio/src/main.rs:266`, `:317`), roteia o clique por
  `Workbench::click` (`main.rs:387-392`, `mod.rs:242`), a roda por
  `workspace_mut().scroll` (`main.rs:424`) e as teclas pelo `keys.rs`
  (o campo do commit pega os caracteres, `keys.rs:26-37`). o ritmo é o do
  `Wake` (`pace.rs`).
- o `WorkspaceView` já se deixa enquadrar sem mudar: `resize(vw, vh)`
  (`apps/idestudio/src/interfaces/panels/layout/mod.rs:334`) refaz as larguras dos
  painéis a partir das desejadas; o `set_frame` (`:252`) também, mas liga
  o modo embutido, que tira o header (`render.rs:30`). o header desenha
  em y = 0 e só com `origin` zero faz sentido fora do modo embutido.
- o mínimo sem mexer nas vistas fixas: o workbench guarda o tamanho da
  janela e, com o bloco gerado ligado, dá ao workspace a janela menos uma
  faixa embaixo (a linha de intenção: campo, proponente, propor, fechar,
  e a mensagem) e, com uma superfície aberta, menos uma coluna à
  direita, onde o host desenha a superfície (a metade da janela, como a
  Silences, `apps/uigallery/src/views/apps/silences/generative/mod.rs:202-231`),
  recortada à coluna. tudo pelo `resize` público do workspace; nenhum
  arquivo de `apps/idestudio/src/interfaces/panels/**` muda. o shell passa a chamar
  `Workbench::resize` e `Workbench::scroll` e manda ao workbench as teclas
  enquanto a linha ou um campo da superfície tem o teclado.

### (c) as regras

- o `propose::Rules` lê a intenção por uma gramática pequena
  (`rust/offers/src/words.rs:204`) e liga por estrutura
  (`rust/offers/src/rules.rs:448-520`): uma palavra que não é da
  gramática nem do app é `NoMatch` com as palavras do app (`:466`),
  nenhuma palavra lida é `NotUnderstood` (`:452`), uma parte pedida que o
  app não tem é `Nothing` (`:473`). a Silences propõe pelas regras dentro
  do `execute` (`apps/uigallery/src/views/apps/silences/generative/page.rs:263-281`)
  e abre ou transforma no lugar (`page.rs:202-228`); o todo, o mesmo pela
  página Surface.
- a parte "lista" pede um `List` ou `Set` cujos valores são as chaves
  dos filhos do nó (`rust/offers/src/list.rs:88-106`, `:108-160`). no
  ide isso é `changes:files`; `stacks` não tem esse estado (item a), então
  as regras de hoje não veem os commits como lista, e "review these three
  commits" é recusado: "review" e "three" não são palavras do app
  (`NoMatch`).
- o que muda, só por adição, em `rust/offers/src/review/mod.rs` (novo): a
  palavra de revisão ("review", "revisar", "revise", "revisa",
  "revisão") abre um caminho próprio das regras, antes da gramática de
  hoje, que não muda (uma intenção sem essa palavra segue igual, e antes
  toda intenção com ela era `NoMatch`). ele acha, por estrutura, os tipos
  de item do app: os filhos de um nó com a mesma chave
  (`stacks.commit[<sha>]`, `changes.file[<path>]`), nomeados pelo nome do
  item e pelo rótulo do pai; a palavra que sobra escolhe o tipo
  ("commits"); um número ("three", "três", "3") escolhe os N primeiros na
  ordem da projeção (sem número, todos). não é um repeat: são "estes
  três", os nós dos três fixos na superfície. cada item mostra os seus
  textos, os seus `List` como tabela (os arquivos), os seus números, e um
  botão por ação sem parâmetro que o autor pode pedir e não destrutiva
  (`show_diff`); a região que essas ações leem fora do item (`diff`, por
  `diff:target`) vem ao lado, com os textos, os números e as listas dela
  (as linhas do diff, numa tabela com a altura limitada pelo token
  `dropdown_max_h`). os itens à esquerda e a região à direita, num
  `split`. recusas: palavra que não nomeia tipo nenhum (`NoMatch`, com as
  palavras dos tipos), nenhum tipo nomeado ou mais de um sem palavra,
  menos itens do que o número pedido (`Nothing`, com o que o app tem).
  nada no `uir` nem no `surface` muda.

### (d) os atos

- um ato da superfície: o widget monta o request como a pessoa, observado
  no snapshot que a superfície mostra, para a capacidade que o IR ligou
  (`rust/screen/src/surface.rs:381-398`), e entrega ao `Dispatch` do
  app. no ide o `Dispatch` é o workbench: `Workbench::dispatch`
  (`mod.rs:150-160`), o dispatcher único, com o receipt da pessoa. a
  recusa stale é a do `sem` (`rust/accord/src/decision.rs:174-193`), igual
  para qualquer ato.
- um agente pela porta: o `Served` do ide projeta `Workbench::snapshot` e
  despacha em `Workbench::dispatch` (`apps/idestudio/src/capability/agent.rs:85-105`).
  com o bloco ligado, o snapshot leva a superfície (o host projeta depois
  da projeção do app, `rust/screen/src/host/mod.rs:557`), e o
  `Workbench::dispatch` passa o request pelo `SurfaceHost::route`
  (`host.rs:565`, `surface.rs:537-565`) antes do dispatcher: um ato num nó
  da superfície vira a capacidade do app, com a observação renomeada, e
  segue o mesmo caminho.
- a gravação do ide (`apps/idestudio/src/interfaces/window/record/mod.rs`): o sink no dispatcher
  (`:126`), o estado por `Workbench::snapshot` (`:132`), os frames com as
  ligações (`:141`). com o bloco ligado a superfície entra no estado e nas
  ligações sem mudar o `record.rs`. fechar é `SurfaceHost::close`
  (`host.rs:469`): sai da projeção, do roteamento e do hit na hora.

### (e) o laboratório

- a pessoa roteirizada (task-93) responde o modal com um clique no meio
  do botão como ele foi desenhado no último frame, pelo
  `Workbench::click`, e o clique é gravado como input
  (`apps/agenteval/src/answer.rs:161-198`). o laboratório grava estado
  e frame a cada mudança (`apps/agenteval/src/arms/plev.rs:286`) com o
  renderer headless (`apps/agenteval/src/subject.rs:100-116`), sem
  janela.
- a demo 5 entra como um braço novo, `--arm review`: sem porta e sem
  agente, sobre um repositório do laboratório com três commits de
  revisão (`src/lab_repo.rs`, por adição), com o bloco gerado ligado. a
  pessoa acha cada controle pelo id no último frame (o retângulo ligado)
  e clica no meio dele pelo `Workbench::click`, como o `answer.rs`: nenhum
  evento do sistema, nenhuma janela, nenhuma coordenada às cegas. o
  controle primeiro: o painel fixo na janela inteira, a pessoa clica nos
  três commits de `stacks`; depois a superfície: escreve a intenção,
  propõe, abre os três diffs pela superfície e fecha. gravado, salvo e
  lido pelo `plev-inspect`; um teste no gate roda o braço.

### (f) o Claude

- o showcase pergunta ao Claude fora da thread: `Claude::find` sem rodar
  o programa (`apps/uigallery/src/views/apps/surface/asking.rs:143`),
  uma thread com o `ClaudeCli` e o `propose::Llm` pelo `propose_verified`
  (`asking.rs:255-306`); o `propose` responde `Execution::Started` e o
  settle termina o request. o seletor começa no Claude quando ele existe
  (`apps/uigallery/src/views/apps/surface/offering/mod.rs:279-283`).
- o ide entra pelo mesmo caminho (`claude-model` como dependência, pelo
  `ClaudeCli`, a thread e o `Llm`), com o seletor começando em regras.
- a privacidade: o prompt leva todos os nós de `ask.app` com os valores
  (`rust/offers/src/prompt.rs:192-230`, cada valor cortado em 60
  caracteres, `:15`). o snapshot inteiro do ide levaria os arquivos
  mudados da árvore de trabalho, as mensagens de todos os commits, o
  autor, a mensagem do commit em edição e o diff de um arquivo fora dos
  commits. então o ide dá ao `Llm` um snapshot recortado: só os commits
  que as regras leem da intenção (a mensagem e os arquivos), e a região
  do diff só quando ela mostra um deles. testado com o `fake-claude` do
  repositório (`rust/claude/tests/fakes/claude`, que guarda o
  prompt em `prompt.N`), nunca com o `claude -p`.

### dependências e arquivos

`apps/idestudio/Cargo.toml` ganha `surface`, `propose`, `uir` e `claude-model`
(o Claude entra pelo caminho do showcase), todos do workspace. no
`Cargo.lock` muda só a lista `dependencies` do pacote `ide`.

### adendo à sonda (depois da primeira gravação da demo)

o primeiro frame da revisão, no laboratório, mostrou duas coisas que a
sonda não viu: o terceiro commit cortado embaixo da coluna, e as linhas
do diff passando da tabela que as limita, por cima do que vinha depois.
a primeira era do layout das regras (corrigido em `review.rs`: a mensagem
e o botão numa linha só, a região num card com as linhas por último). a
segunda era do `surface`: a lista virtual da tabela desenha a sobra além
dos limites dela, e quem hospeda recorta; o `draw.rs` da superfície não
recortava, porque até aqui nenhuma tabela gerada tinha menos altura que
as linhas. a adição mínima: uma tabela limitada em altura (`size.max`)
com mais linhas do que cabem é recortada ao retângulo dela
(`rust/screen/src/draw.rs`, com teste na mesma fonte); uma tabela da
altura das linhas desenha como sempre, e os 176 baselines não mudam.

## handoff (2026-09-25)

```text
Goal: task-96, uma superfície gerada no ide e a demo 5 da task-80
Status: pronto para o orquestrador, na branch task/96a-ide-surface
```

### commits

- `73a318a7` task-96: the probe, and the task file in ongoing
- `6bacaa3d` task-96: the intent line and the host of generated surfaces in the ide
- `f53667c0` task-96: the registry of a review, and each commit's diff through it
- `73704549` task-96: the rules review items: "review these three commits"
- `f8f39298` task-96: every act on the review through the workbench's one dispatcher
- `e315e11e` task-96: the review fits one screen, and a bounded table keeps its rows
- `2e6c9e9a` task-96: the demo 5 in the laboratory, recorded with no window
- `07650665` task-96: the demo 5 result, written in task-80
- `9dadd352` task-96: the Claude proposes for the line, by the showcase's path
- `855b1a4d` task-96: the review test of propose, formatted (o `e315e11e`
  deixou uma asserção sem `cargo fmt`; o gate pegou; commit à parte para
  não reescrever a branch, porque a gravação da demo nomeia `2e6c9e9a`)
- `3df1297f` task-96: a move off the column lets go of what it hovered
- `205e90b3` task-96: the handoff
- `1283cc5b` task-96: the review's tests of propose, inside propose's src
  (os testes das regras saem de `rust/offers/tests/review.rs` para
  `rust/offers/src/review/tests.rs`: o briefing deixa a task mexer só em
  `rust/offers/src/**`)
- o commit desta última versão do handoff

### a linha e o host

- moram no workbench, ligados por `Workbench::host_surfaces` (a janela do
  ide liga; o showcase, os testes antigos e os braços antigos do
  laboratório não): `apps/idestudio/src/generative/` (a linha, o host, onde
  eles ficam e como desenham, o Claude e o recorte dele) e
  `apps/idestudio/src/capability/{generated,column,asking}.rs` (o lado do
  workbench: o que um request pede da linha, o roteamento, o layout, o
  ponteiro e as teclas, o Claude).
- uma coluna à direita da janela: a largura de um card sem superfície,
  metade da janela com uma; a linha em cima (campo, proponente, propor,
  fechar, a mensagem, o cancelar enquanto o Claude responde), a superfície
  embaixo, recortada à coluna. o workspace recebe a janela menos a coluna
  pelo `resize` dele; nenhum arquivo de `apps/idestudio/src/interfaces/panels/**` mudou.
- a projeção: o workspace, depois a superfície e a linha (`ide_surface`).
  `Workbench::app_snapshot` é o workspace sozinho: o registro de toda
  proposta e o que a superfície mostra. o golden
  `rust/plevir/tests/fixtures/idestudio-workbench.snapshot.json` continua igual.
- o shell (`main.rs`, `keys.rs`): o tamanho, a roda e o hover vão para o
  workbench; as teclas vão para a linha ou para um campo da superfície
  enquanto um deles tem o teclado.
- fechar: `SurfaceHost::close`; fora da projeção, do roteamento e do hit
  na hora; o workspace volta à largura; no host nada fica (teste).

### o registro e o diff

nada mudou na projeção: cada commit já tem `summary`, `author`, `files`
(os 10 mais novos) e `show_diff`, que lê `diff:target`; a região `diff`
mostra o diff de um commit por vez. a superfície abre cada diff pelo
`show_diff` do commit, como o painel fixo. os três diffs ao mesmo tempo
ficam como P3 (um estado de diff por commit).

### as regras

`rust/offers/src/review/mod.rs` (novo), um caminho próprio das regras
aberto por uma palavra de revisão (en e pt), antes da gramática de hoje,
que não muda. acha os tipos de item por estrutura, escolhe o tipo pela
palavra, o número pelo "three"/"três"/"3", e faz um card por item (a
mensagem e o botão numa linha, o autor, os arquivos em tabela) e a
região que as ações dele leem ao lado (o diff, com as linhas numa tabela
limitada por `dropdown_max_h`), num split. recusa com o motivo: palavra
que não nomeia item (`NoMatch`), nada a revisar, nenhuma palavra dizendo
quais itens, menos itens do que o número (`ProposeError::Review`, variante
nova). em `propose` também: três funções privadas viraram `pub(crate)`
(`words::is`, `list::last_key`, `parts::allows`) e o `Rules::propose`
ganhou o desvio de três linhas.

### os atos

- o clique da pessoa na superfície: o widget monta o request dela para a
  capacidade (`show_diff` em `stacks.commit[<sha>]`), observado no estado
  do app que a superfície mostrava, e o `Workbench::dispatch` roda; o
  receipt é `human` (teste).
- um agente pela porta observa a superfície e aperta o mesmo botão pelo
  nó dela; o `Workbench::dispatch` passa pelo `SurfaceHost::route` antes
  do dispatcher e o request chega como a capacidade, com o nome do agente
  (teste).
- stale: o agente olhou, a pessoa abriu outro diff pela superfície, o ato
  do agente volta `stale` com o delta de `diff:target` e o clique da
  pessoa em `acted` (teste).
- a gravação do ide não mudou: o estado gravado leva a superfície e a
  linha, os frames levam as ligações dela.

### a demo 5 e o resultado contra o painel fixo

`ide-lab --arm review` (novo braço), gravado sem janela no build
`2e6c9e9a`, lido pelo `plev-inspect`; o resultado inteiro na task-80,
seção "demo 5, UI generativa em aplicação real (feita pela task-96)". em
resumo: a superfície mostra os arquivos dos três commits de uma vez (3
contra no máximo 1 no painel fixo, que nunca nomeia o arquivo de um
commit de um arquivo só), cada ato dela é um request da pessoa pelo
dispatcher, e nada fica depois de fechar. ela custa mais atos (5 cliques
e 26 teclas contra 3 cliques) e mostra o diff pior (5 linhas por vez numa
tabela, sem cor e sem número, contra o diff inteiro). pelo critério
escrito antes, a demo cai no diff e se sustenta nos arquivos; o que falta
ao diff é um componente do vocabulário do IR, não o ide.

### o Claude

o mesmo caminho do showcase: `claude-model` (o `ClaudeCli`), uma thread
com o `propose::Llm` e o `propose_verified`, `Execution::Started`, o
wake do workbench termina o request (`completed` com a superfície, as
tentativas, os segundos e o custo, ou `failed`), e o cancelar termina
como cancelado por quem pediu. o seletor oferece o Claude onde o programa
existe e começa em regras. o Claude recebe um snapshot só dele: os
commits que a revisão das regras nomeia na intenção, com a mensagem, os
arquivos e o `show_diff`, e a região do diff só enquanto ela mostra um
deles; nunca a árvore de trabalho, outro commit, autor, branch, a
mensagem em edição, a linha ou a superfície aberta. uma intenção que não
nomeia commit não é enviada. testado só com o `fake-claude` do
repositório (o prompt dele conferido palavra a palavra); nenhuma chamada
ao `claude -p`. o prompt do `propose` mostra cada valor nos primeiros 60
caracteres, então o Claude vê só o começo do diff.

### o diff do lock

só a lista `dependencies` do pacote `ide`; nenhum pacote novo, nenhuma
versão nova:

```diff
@@ -2391,6 +2391,7 @@
 name = "ide"
 version = "0.1.0"
 dependencies = [
+ "claude-model",
  "comps",
  "engine",
  "env_logger",
@@ -2399,10 +2400,13 @@ dependencies = [
  "log",
  "notify",
  "pollster",
+ "propose",
  "rec",
  "sem",
  "serde_json",
+ "surface",
  "tempfile",
+ "uir",
  "wgpu",
  "winit",
  "wire",
```

### arquivos

- `apps/idestudio/Cargo.toml` (quatro dependências do workspace), `Cargo.lock`
  (a lista do `ide`).
- `apps/idestudio/src/generative/{mod,row,claude,scope}.rs` (novos),
  `apps/idestudio/src/capability/{generated,column,asking}.rs` (novos),
  `apps/idestudio/src/capability/{mod,project,execute}.rs`, `apps/idestudio/src/lib.rs`,
  `apps/idestudio/src/main.rs`, `apps/idestudio/src/interfaces/window/keyboard.rs`,
  `apps/idestudio/src/repository/eval_repo.rs` (o repositório da revisão),
  `apps/agenteval/src/{review.rs (novo),args.rs,bench.rs,main.rs}`.
- testes: `apps/idestudio/tests/{surface,review,review_acts,lab_review,claude}.rs`
  (novos), `apps/idestudio/tests/support/mod.rs` (`Rig::review`).
- `rust/offers/src/review/mod.rs` e `rust/offers/src/review/tests.rs`
  (novos), `rust/offers/src/{rules,lib,error,words,list,parts}.rs`.
- `rust/screen/src/draw.rs` (o recorte da tabela limitada, com teste).
- [`task-96-ide-generated-surface.md`](task-96-ide-generated-surface.md) (este),
  a seção da demo 5 em [`task-80-proof-qualitatively-different-software.md`](../task-80-proof-qualitatively-different-software/task-80-proof-qualitatively-different-software.md).

nenhum arquivo de `rust/accord`, `wire`, `engine`, `comps`, `claude-model`,
`showcase`, `uir`, nem das vistas fixas do ide; nenhum baseline.

### interfaces

- `ide::workbench::Workbench`: `host_surfaces`, `host_surfaces_with`,
  `hosts_surfaces`, `surfaces`, `surface_open`, `app_snapshot`, `resize`,
  `scroll`, `hover`, `has_keyboard`, `type_text`, `edit_key`, `enter`,
  `escape` (novos); `dispatch` passa pelo roteamento da superfície antes
  do dispatcher; `click`, `render` e `wake` cuidam da coluna quando ela
  está ligada.
- `ide::lab_repo`: `Repo::build_review`, `review_in`, `REVIEWED`, `FIRST`,
  `UNCOMMITTED`.
- `propose::ProposeError::Review` (variante nova). nenhuma interface do
  `sem`, do `wire`, do `uir` ou do `surface` mudou.

### validação

com o ambiente do briefing em todo comando cargo (`CARGO_TARGET_DIR` em
`target/agents/one`, `CARGO_INCREMENTAL=0`, debug `line-tables-only`, o
`PLEV_CLAUDE` no stub que só sai com erro) e `--offline`:

- `./tool/tasks/fullcheck.sh` em `1283cc5b`: verde (exit 0, 04:32 a 04:35), com a
  perna urnaui e o check wasm com `-D warnings`. workspace 2951 passam, 0
  falham, 26 ignorados (162 suítes); urnaui 85 passam, 1 ignorado (6
  suítes). total 3036 passam e 27 ignorados, contra 3012 e 27 na base: os
  24 testes novos. o mesmo total em `3df1297f`, antes de os testes das
  regras mudarem de lugar (163 suítes). `df -h /`: 14 GB livres antes do
  primeiro gate, 12 GB depois do último.
- um gate anterior, em `9dadd352`, caiu no `cargo fmt --check` por uma
  asserção de `rust/offers/tests/review.rs` (o `855b1a4d` corrige); os
  testes e o clippy já passavam nele.
- `cargo run --offline -p regress -- check`, com o
  `main.rs` tocado: 176 baselines, 176 iguais, nenhum frame desenhado,
  nenhuma GPU aberta (em `9dadd352`, `3df1297f` e `1283cc5b`).
- `cargo test -p idestudio --test lab_review` cinco vezes seguidas, e
  `--test claude` três: todas verdes. o braço da demo rodado à mão
  (`cargo run -p agenteval -- --arm review --out <scratch>`), a
  sessão lida pelo `plev-inspect`.
- nenhuma janela aberta (o binário `ide` não rodou), nenhum clique ou
  tecla por coordenada de tela do sistema (os cliques do laboratório e dos
  testes são pelo `Workbench::click`, no meio do retângulo que o frame
  ligou ao id), nenhuma chamada ao `claude -p` (só o `fake-claude`),
  nenhum pacote novo no lock.

### decisões

- a linha e o host dentro do workbench, ligados por opção: o dispatcher é
  do workbench, e só assim a porta, a gravação e o clique da pessoa
  passam pelo mesmo caminho sem tocar no shell de cada host.
- a coluna à direita, não uma faixa embaixo: o seletor abre para baixo, e
  a coluna é onde a superfície cresce.
- a revisão fixa os três commits na superfície (cards, não repeat): "estes
  três" são estes três.
- o Claude só para uma revisão que as regras leem: é o jeito de saber
  quais commits o prompt pode levar.
- o laboratório oferece só as regras (`host_surfaces_with(None)`), para a
  linha ser a mesma em toda máquina.

### limitações

- o diff na superfície: 5 linhas por vez, sem cor e sem número (o
  vocabulário não tem componente de código; a tabela é a única lista de
  textos, com linhas de 48 px e um token de altura). as linhas longas
  quebram dentro da célula.
- um diff por vez, como no painel fixo.
- a linha de intenção do ide é uma segunda cópia da linha da Silences
  (`apps/uigallery/src/views/apps/surface/offering/mod.rs`); o showcase
  estava fora desta task, então a extração para um lugar comum fica para
  depois.
- a observação da pessoa na superfície não aparece na gravação: o request
  dela é observado no snapshot do app sozinho, e o gravador guarda o do
  workbench inteiro (o achado da task-69 fase 4c, um passo mais longe).
- o Claude vê só os primeiros 60 caracteres de cada valor (o `propose`
  corta assim para todo app), então só o começo do diff.

### o que fica aberto

- a prova com o Claude de verdade (o orquestrador).
- a demo 5 com janela, pelo brenner (`cargo run -p idestudio`, com
  `PLEV_RECORD=1`).
- P3: um estado de diff por commit (os três diffs de uma vez); um
  componente de código ou diff no vocabulário (widget no comps e ADR); a
  linha de intenção num lugar comum ao showcase e ao ide.

## fechamento (2026-09-25, orquestrador)

integrada. os critérios cumpridos pelas regras: a revisão de três
commits gerada e verificada, operada pelo dispatcher do workbench, sem
rastro ao fechar, e a demo 5 medida contra o painel fixo (misto: melhor
nos arquivos, pior no diff). fica com o orquestrador e o brenner: a
prova com o Claude de verdade e a demo 5 com janela. o que faltou para o
diff (um componente de diff no vocabulário do IR) vira task nova.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [task-69: ephemeral and generated surfaces](../task-69-ephemeral-generative-surfaces/task-69-ephemeral-generative-surfaces.md)
- [task-80: proof of qualitatively different software](../task-80-proof-qualitatively-different-software/task-80-proof-qualitatively-different-software.md)
- [task-85: the commit that the receipt announces is already in the projection, with the files](../task-85-the-commit-a-receipt-announces-is-projected/task-85-the-commit-a-receipt-announces-is-projected.md)
- [task-93: the person in the lab answers](../task-93-the-lab-person-answers/task-93-the-lab-person-answers.md)
- [task-96: a generated surface in the ide](task-96-ide-generated-surface.md)
- [five demos of qualitatively different software, each with its method written before it ran](../../Research/Exploring/five-demos.md)
<!-- /related -->
