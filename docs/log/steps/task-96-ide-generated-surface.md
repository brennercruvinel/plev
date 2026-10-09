---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-09-25
domain: generative-runtime
task: task-96
wave: 8
depends: [task-68, task-69, task-70, task-63, task-64]
branch: task/96a-ide-surface
---

# task-96: uma superfície gerada no ide

## objetivo

a demo 5 da task-80 (UI generativa numa aplicação real) pede uma
intenção num app que já existe gerando uma superfície verificada que a
pessoa opera. hoje as superfícies geradas vivem só nas páginas Surface e
Silences do showcase: nem o ide nem o urnaui têm `SurfaceHost` (sonda da
task-80 fase 1, item (a)). esta task põe uma no ide.

## ligação com a tese

seção 28 (a hipótese é o contrato unificado, provada em aplicação real,
não só na demo de vídeo); goal 10 do programa; a demo 5 da task-80.

## estado atual no código

- `rust/screen` (`SurfaceHost`, `Surface::route`) e `rust/offers`
  (regras e `Llm`) servem a Silences e a página Surface do showcase.
- o `uir` já verifica IR no domínio do ide contra o registro que a
  projeção do ide dá: `rust/plevir/tests/fixtures/idestudio-review-commit.ir.json`,
  `ide-commit-form.ir.json`, `ide-workbench.snapshot.json`, escrito por
  `apps/idestudio/tests/plevir_fixture.rs`.
- `apps/idestudio/Cargo.toml` não depende de `surface`, `propose` nem `uir`.

## escopo

1. um `SurfaceHost` na janela do ide ao lado das vistas fixas; a linha de
   intenção (campo, proponente, propor, fechar) projetada como na
   Silences.
2. o registro de capacidades tirado do `Workbench::snapshot`; os atos da
   superfície roteados ao dispatcher único do workbench
   (`Surface::route`), com os receipts e a gravação do ide como estão.
3. as regras propondo "review these commits" sobre `stacks`; o Claude
   pelo `propose::Llm` fora da thread do app (a prova com o Claude de
   verdade é do orquestrador).
4. o laboratório grava a demo 5 sem janela, pelo roteiro e o critério
   escritos na task-80.

## fora de escopo

o urnaui; transformar as vistas fixas do ide (elas se exprimem em IR,
pela decisão da task-70 fase 1); qualquer chamada real ao Claude por um
agente.

## dependências

68, 69, 70, 63d e 64 integradas.

## superfície sob responsabilidade

`apps/idestudio/src/**` (a janela e a projeção), `apps/idestudio/Cargo.toml`
(três dependências de workspace: `surface`, `propose`, `uir`; nenhuma
externa), testes do ide, e a demo 5 no arquivo da task-80.

## critérios de aceitação

- "review these three commits" gera, pelas regras, uma superfície
  verificada que mostra os três commits com os arquivos e o diff de cada
  um, e que a pessoa opera.
- cada ato da superfície passa pelo dispatcher do workbench, com o
  receipt de quem pediu; nada da superfície fica depois de fechar.
- a demo 5 medida contra o painel de stacks fixo mais o diff, pelo
  critério escrito na task-80.
- `./tool/tasks/fullcheck.sh` verde.

## o que a derruba

a superfície pior que o painel fixo para revisar três commits; um ato
fora do dispatcher; a superfície deixando rastro.

## pare e reporte se

- o ide precisar de dependência externa nova;
- a projeção do ide não der o registro que a superfície precisa sem
  mudar tipo do `sem`.

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
- `docs/log/steps/task-96-ide-generated-surface.md` (este),
  a seção da demo 5 em `docs/log/steps/task-80-proof-qualitatively-different-software.md`.

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
- `cargo run --offline -p showcase --example regress -- check`, com o
  `main.rs` tocado: 176 baselines, 176 iguais, nenhum frame desenhado,
  nenhuma GPU aberta (em `9dadd352`, `3df1297f` e `1283cc5b`).
- `cargo test -p ide --test lab_review` cinco vezes seguidas, e
  `--test claude` três: todas verdes. o braço da demo rodado à mão
  (`cargo run -p ide --bin ide-lab -- --arm review --out <scratch>`), a
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
- a demo 5 com janela, pelo brenner (`cargo run -p ide`, com
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

## fase b (2026-09-25): o título do cartão do diff e a coluna só com superfície

a validação da task-97 abriu as imagens da demo 5 e viu dois defeitos
que os testes não pegam, porque contam linhas e atos: o cartão do diff,
à direita da revisão gerada, tem como título o id cru do commit
(`stacks.commit[847f583aab69146fe62a5936…`); e sem superfície aberta a
coluna da linha de intenção ocupa 368 px da janela e aperta as vistas
fixas. branch `task/96b-ide-surface-polish`, base `main` @ `8704ba3a`.

### sonda (antes de qualquer código)

só leitura, e uma corrida do laboratório na base para as imagens de
antes (`ide-lab --arm review --out <scratchpad>/demo5b/before`, sem
janela, os mesmos números da task-97: 14, 7 e 18 linhas no painel fixo;
14, 7 e 14 na superfície). toda afirmação sobre código tem arquivo e
linha.

#### (a) de onde vem o título do cartão do diff

- não é o rótulo do nó nem o materializador: é um texto da região que as
  regras põem primeiro e no tipo de título. a revisão faz da região que
  as ações dos itens leem fora deles um cartão
  (`rust/offers/src/review/mod.rs:458-479`, `regions`, e `:433-445`,
  `region`): os textos dela primeiro, depois os números, as listas por
  último. o primeiro texto ganha `Base2sm`, o tipo do que nomeia
  (`review.rs:304-312`). a região é `diff`, e o primeiro estado dela é
  `target`, o id do nó cujo diff aparece
  (`apps/idestudio/src/capability/project.rs:152-157`): num commit,
  `stacks.commit[<sha>]`. é o estado que o `show_diff` de cada commit lê
  (`project.rs:204`, `:231`), a palavra do dispatcher para "qual item",
  não a de uma pessoa.
- o rótulo do cartão é o da região, `"diff"` (`review.rs:440`), mas um
  `card` não desenha o rótulo (`rust/screen/src/draw.rs:291-297`, só o
  painel); um `text` desenha o valor ligado, ou o rótulo quando não liga
  nada (`draw.rs:71-76`, `:317-329`).
- a palavra certa que a projeção tem, sem inventar estado: a mensagem do
  commit que o diff mostra existe, como o rótulo e o `summary` do nó que
  `target` nomeia (`project.rs:217-227`), mas num outro nó. o IR liga um
  estado por uma referência fixa (`uir::Bind`,
  `rust/plevir/src/ir.rs:112`), e nenhum componente segue um valor até o
  rótulo de outro nó (`text` mostra o `text` ligado,
  `rust/plevir/src/vocabulary.rs:332`). um texto pode mostrar "o summary de
  `stacks.commit[847f…]`", não "o summary do commit que `diff:target`
  nomeia agora". a mensagem que muda com o diff pediria um estado novo na
  região (a projeção muda, e o golden
  `rust/plevir/tests/fixtures/idestudio-workbench.snapshot.json` com ela) ou o
  `surface` seguindo a referência: os dois fora desta fase. escrever na
  proposta a mensagem que `target` nomeia na hora também não serve: o
  diff segue o `target` e o rótulo fixo não, e depois do próximo "show
  diff" o título mentiria. o painel fixo mostra a mensagem no cabeçalho
  (`apps/idestudio/src/interfaces/panels/diff.rs:84-92`), que não é projetado.
- então a palavra é o rótulo da região, `"diff"` (`project.rs:153`), o
  que o cartão mostra. o que muda, só em `review.rs`: um texto da região
  que as ações dos itens leem, cujo valor é o id de um nó do app (ou
  vazio), é o ponteiro do dispatcher e não aparece; uma região sem texto
  próprio abre com o rótulo dela num texto no tipo de título, primeiro no
  cartão. esse texto não liga nada, então o `Namer` não o guarda entre
  duas propostas (`rust/offers/src/ids.rs:98-113`: guarda os nós
  ligados e os containers com filhos); a revisão procura na superfície
  aberta o texto sem ligação que abre o cartão da mesma região e mantém o
  id dele. o ato da pessoa não muda: o request é observado no snapshot
  do app (`rust/screen/src/surface.rs:386`), e um estado que a
  superfície não mostra é lido como o app o guarda
  (`rust/screen/src/project.rs:26-28`).

#### (b) onde a coluna nasce e o desenho mínimo

- a coluna nasce ligada ao bloco: `host_surfaces_over`
  (`apps/idestudio/src/generative/mod.rs:59-64`) e o `lay_out`
  (`:103-112`) dão ao workspace a janela menos `column_w`
  (`apps/idestudio/src/generative/row.rs:61-70`): a largura de um card
  (`size.card_w`, 368 px no hoff, `rust/engine/src/styling/theme/scales.rs:291`)
  sem superfície, metade da janela com uma. `Workbench::resize`
  (`generated.rs:93-99`) refaz o mesmo a cada tamanho. a coluna vai de
  cima a baixo, a linha em cima e a superfície embaixo (`row.rs:73-82`).
  a janela liga o bloco sempre (`apps/idestudio/src/main.rs:565`).
- o que isso custa sem superfície: o workspace tem 912 de 1280 px. o
  `apply_panel_widths` (`apps/idestudio/src/interfaces/panels/layout/mod.rs:380-392`)
  guarda as larguras de changes e de diff e dá o resto a stacks, então é
  a coluna de stacks que perde os 368 px, e a mensagem de cada commit sai
  cortada (`apps/idestudio/src/interfaces/panels/stacks.rs:272-276`). na
  imagem de antes `demo5b/before/fixed.png`: "readme: how…", "login
  rememb…", "session timeo…".
- o desenho mínimo: a linha numa faixa ao longo de baixo da janela, na
  largura inteira, e a coluna só com superfície aberta. sem superfície o
  workspace recebe a largura inteira da janela, a de antes da task-96, e
  só a altura da faixa a menos; com superfície, a coluna à direita, em
  cima da faixa, metade da janela, como hoje. na faixa: o campo (um card
  de largura, no máximo), o proponente, propor e fechar numa linha, e a
  mensagem ao lado deles enquanto a largura dá, embaixo deles quando não
  dá. tudo pelo `resize` do workspace; nenhum arquivo de
  `apps/idestudio/src/interfaces/panels/**` muda.
- por que embaixo e não em cima: o header do workspace desenha em y = 0
  (`apps/idestudio/src/interfaces/panels/layout/render.rs:28-40`), e deslocar a origem
  é o `set_frame` (`views/workspace/mod.rs:252-257`), que liga o modo
  embutido e tira o header: mudaria uma vista fixa. dentro do header
  também não: ele centra o repositório e a branch
  (`apps/idestudio/src/interfaces/panels/header.rs:91-100`), e a linha passaria por cima.
- o que a faixa embaixo pede: o proponente é o `Select` do comps, que
  abre o painel embaixo do retângulo que recebe
  (`rust/engine/src/widgets/form/select/panel.rs:73-86`), e embaixo da faixa
  não há janela. o comps está fora desta fase; a linha dá ao `Select`,
  para o painel, o retângulo do controle levantado pela altura do painel
  e pelas duas folgas (o `dropdown_rect` dá as duas), então o painel abre
  para cima, por cima do workspace, com o controle desenhado e clicado
  onde está. um `Select` que abre para cima quando falta espaço embaixo
  é o jeito limpo, no comps (fica como P3).
- o que não muda: os mesmos widgets, os mesmos ids, a mesma projeção
  (`apps/idestudio/src/generative/mod.rs:255-313`); só os retângulos se movem.
  a pessoa chega à linha pelo clique no campo, um agente pela porta
  (`set_value` e `propose` em `ide_surface.*`, `Workbench::dispatch`),
  o teclado como hoje (`apps/idestudio/src/interfaces/window/keyboard.rs:70-95`: com o campo
  focado, as teclas são dele, Enter propõe, Escape solta).
- o preço: a altura da faixa, uns 60 px de 800, tirada das vistas fixas,
  que rolam na vertical; o diff mais longo da demo (18 linhas) cabe
  inteiro nas duas alturas. as imagens de antes e depois de cada caso
  (sem superfície, com superfície) ficam em `demo5b/`, citadas no
  handoff desta fase.

### handoff da fase b (2026-09-25)

```text
Goal: task-96 fase b, o título do cartão do diff e a coluna só com superfície
Status: pronto para o orquestrador, na branch task/96b-ide-surface-polish
```

#### commits

- `e71222cf` task-96: the probe of phase b, the diff card's title and the
  column only with a surface
- `3959d958` task-96: the diff card says what it shows, "diff", never the
  id of the commit it shows
- `4be4bb2f` task-96: the line in a strip under the fixed views, and a
  column only while a surface is open
- `e42bd571` task-96: the laboratory says how many of the three commits'
  messages the stacks panel cuts (a gravação da demo nomeia este build)
- `a427d2b8` task-96: the line's test finds each repository's own commit
  card (o primeiro gate pegou: dois repositórios, dois shas; o teste só
  passava quando os dois commits caíam no mesmo segundo)
- o commit deste handoff (esta seção e a demo 5 na task-80)

um commit a mais que os quatro do briefing: a medida nova do laboratório
foi para um commit próprio antes da gravação, para a gravação nomear um
build com ela.

#### a sonda

a seção "sonda" acima, (a) e (b), escrita antes do código.

#### o título

- `rust/offers/src/review/mod.rs`: um texto da região que as ações dos
  itens leem, cujo valor é o id de um nó do app ou vazio (`diff:target`),
  não aparece; uma região sem texto próprio abre com o rótulo dela
  (`"diff"`), no tipo de título (`Base2sm`), sem ligar nada. o `Namer`
  não guarda um nó sem ligação e sem filhos entre duas propostas, então a
  revisão procura na superfície aberta o texto sem ligação que abre o
  cartão da mesma região e mantém o id (`proposed.diff.heading`); um nó
  assim o `Namer` nunca entrega, então o id não sai duas vezes.
- por que "diff" e não a mensagem do commit: a sonda (a). a mensagem
  pediria um estado novo na região ou o `surface` seguindo a referência.
- o botão "show diff" de cada commit na superfície passou a ler
  `diff:target` como o app o guarda, porque a superfície não mostra mais
  esse estado (`rust/screen/src/project.rs:26-28`); o stale é o mesmo
  (`apps/idestudio/tests/review_acts.rs` passa sem mudança).
- testes: `rust/offers/src/review/tests.rs`
  (`the_region_says_what_it_shows_never_the_id_of_the_commit_it_shows`,
  novo; o da revisão de três commits confere o título e que nada liga
  `diff:target`; o de propor de novo confere que o id do título fica em
  duas propostas seguidas) e `apps/idestudio/tests/review.rs`
  (`the_diff_card_says_diff_never_the_id_of_the_commit_it_shows`, novo:
  para cada um dos três diffs aberto pela superfície, o cartão abre com
  "diff" e nenhum nó da superfície tem o id do commit no nome ou no
  estado).

#### a coluna

- a linha numa faixa ao longo de baixo da janela, na largura inteira
  (`apps/idestudio/src/generative/row.rs`, `place`, `row`): o campo (um card de
  largura, no máximo), o proponente, propor e fechar numa linha e a
  mensagem ao lado deles enquanto a largura deixa a ela o mínimo de um
  card; a mensagem embaixo deles quando não; os três embaixo do campo
  numa janela estreita. a faixa tem 60 px numa janela de 1280 por 800.
- sem superfície: o workspace tem a janela inteira de largura e a altura
  menos a faixa. com superfície: a coluna à direita, do topo até a faixa,
  metade da janela como antes. fechar devolve a largura. tudo pelo
  `resize` do workspace (`apps/idestudio/src/generative/mod.rs`,
  `lay_out`); nenhum arquivo de `apps/idestudio/src/interfaces/panels/**` mudou.
- o proponente abre o painel para cima: a linha dá ao `Select` do comps,
  para o painel, o controle levantado pela altura do painel e pelas duas
  folgas (`row.rs`, `lifted`); o controle é desenhado e clicado onde está.
- a linha continua a mesma para a pessoa, o agente e o teclado: os mesmos
  widgets, ids e projeção; só os retângulos mudaram
  (`apps/idestudio/src/generative/column.rs`).
- testes: `apps/idestudio/tests/line.rs` (novo, seis testes): sem superfície,
  as vistas fixas no mesmo x e com a mesma largura de um workbench sem a
  linha, e a linha inteira na faixa; a superfície abre na coluna e fechar
  devolve a largura; a pessoa chega à linha pelo clique e pelas teclas
  (Enter propõe, Escape solta); o painel do proponente abre acima do
  controle, dentro da janela, e a opção é escolhida pelo clique; a linha
  quebra na faixa em três larguras (1280, 800, 480) e o workspace fica com
  a largura inteira; um agente pela porta vê a linha na faixa (os
  retângulos da observação) e escreve, propõe e fecha por ela.
  `apps/idestudio/tests/surface.rs` mudou as asserções da coluna para a
  faixa.

#### a demo 5 de novo

gravada sem janela no build `e42bd571`, lida pelo `plev-inspect`; o
resultado inteiro na task-80, seção "de novo, com a linha embaixo
(task-96, fase b)". em resumo: nenhum número do critério mudou de lado
(atos, arquivos, linhas do diff à vista e cortadas na largura iguais; o
tempo varia de corrida a corrida). o que mudou: no painel fixo as três
mensagens dos commits agora aparecem inteiras (0 de 3 cortadas, eram 3 de
3, medido pelo dado novo do laboratório sobre o layout de antes), e o
cartão do diff na superfície abre com "diff". o teste
`apps/agenteval/tests/review.rs` segura as 0 mensagens cortadas.

#### as imagens

na scratchpad da sessão, `demo5b/`:

- antes (base `8704ba3a`): `before/fixed.png` (sem superfície: a coluna
  de 368 px, as mensagens cortadas), `before/surface.png` e
  `before/surface-diff.png` (com superfície: o título
  `stacks.commit[847f…`), `before/closed.png`.
- depois (build `e42bd571`): `after/fixed.png` (sem superfície: a
  largura inteira, as mensagens inteiras, a linha na faixa),
  `after/surface.png` e `after/surface-diff.png` (com superfície: a coluna
  do topo até a faixa, o cartão abrindo com "diff"), `after/closed.png`.
- `proposer/proposer-open.png`: o painel do proponente aberto para cima
  (uma corrida descartável do laboratório com um clique no proponente,
  fora do git).
- `before-measure/report.json`: a medida nova do laboratório sobre o
  layout de antes (3 de 3 mensagens cortadas).

#### validação

com o ambiente do briefing em todo comando cargo (`CARGO_TARGET_DIR` em
`target/agents/one`, `CARGO_INCREMENTAL=0`, debug `line-tables-only`, o
`PLEV_CLAUDE` no stub que só sai com erro) e `--offline`
(`CARGO_NET_OFFLINE=true` no gate):

- `./tool/tasks/fullcheck.sh` em `a427d2b8`: verde (exit 0, 05:55 a 05:58), com a
  perna urnaui e o check wasm com `-D warnings`. workspace 2977 passam, 0
  falham, 26 ignorados (165 suítes); urnaui 85 passam, 1 ignorado (6
  suítes). total 3062 e 27, contra 3054 e 27 na base: os 8 testes novos.
  `df -h /`: 28 GB livres antes e depois.
- um gate anterior, em `e42bd571`, caiu em `apps/idestudio/tests/line.rs`
  (o teste que o `a427d2b8` corrige); o que rodou antes dele passava.
- `cargo run --offline -p showcase --example regress -- check`, com o
  `main.rs` tocado: 176 baselines, 176 iguais, nenhum frame desenhado,
  nenhuma GPU aberta.
- `cargo test -p ide --test line` cinco vezes seguidas: verde.
- nenhuma janela aberta (o binário `ide` não rodou), nenhum clique ou
  tecla do sistema (os cliques são pelo `Workbench::click`, no meio do
  retângulo que o frame ligou), nenhuma chamada ao `claude -p`, nenhuma
  dependência nova, `Cargo.toml` e `Cargo.lock` iguais.

#### decisões

- a faixa embaixo, não em cima: em cima pediria tirar o header do
  workspace (o modo embutido), uma vista fixa.
- a linha fica na faixa também com a superfície aberta: ela não pula de
  lugar quando uma superfície abre ou fecha.
- "diff" e não a mensagem do commit: sem estado novo, o título não
  consegue seguir o diff.
- o painel do proponente para cima por um retângulo levantado, sem mudar
  o comps.
- o id do título guardado em `review.rs`, porque `ids.rs` está fora da
  lista de arquivos.

#### limitações e o que fica aberto

- o painel para cima é um arranjo em cima do `Select` do comps; um
  `Select` que abre para cima quando falta espaço embaixo é o jeito limpo
  (comps, P3).
- a faixa cresce quando a mensagem pede mais linhas do que ela tem (uma
  recusa longa numa janela estreita): o workspace perde esses pixels e os
  controles sobem junto. em 1280 de largura, uma mensagem de até duas
  linhas cabe na altura do campo; três linhas somam uns 4 px.
- sem coluna depois de fechar, o que resta da superfície some por cima
  do workspace (a área acima da faixa) durante o fade.
- as vistas fixas perdem 60 px de altura com a linha ligada.
- a mensagem do commit no título do cartão do diff pede um estado por
  região (junto do P3 dos três diffs de uma vez).
- seguem com o orquestrador e o brenner: a prova com o Claude de verdade
  e a demo 5 com janela.
