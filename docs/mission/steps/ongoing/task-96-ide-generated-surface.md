---
project: plev
audience: [ai-agents, contributors]
status: ongoing
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

- `crates/surface` (`SurfaceHost`, `Surface::route`) e `crates/propose`
  (regras e `Llm`) servem a Silences e a página Surface do showcase.
- o `uir` já verifica IR no domínio do ide contra o registro que a
  projeção do ide dá: `crates/uir/fixtures/ide-review-commit.ir.json`,
  `ide-commit-form.ir.json`, `ide-workbench.snapshot.json`, escrito por
  `crates/ide/tests/uir_fixture.rs`.
- `crates/ide/Cargo.toml` não depende de `surface`, `propose` nem `uir`.

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

`crates/ide/src/**` (a janela e a projeção), `crates/ide/Cargo.toml`
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
- `./script/gate` verde.

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

- `stacks` (list) tem só `branch` (`crates/ide/src/workbench/project.rs:166-168`).
  cada commit é um nó filho `stacks.commit[<sha>]` (list item) com
  `summary`, `author` e a ação `show_diff`, que lê `diff:target`
  (`:186-196`). não há um `List` com os shas dos commits: os commits são
  os filhos de `stacks`, na ordem da projeção, a pilha ativa primeiro
  (`crates/ide/src/adapters.rs:63`, `sort_by_key(|s| !s.is_active)`).
- os arquivos de cada commit (`files`, task-85) vão só para os
  `FILES_PROJECTED` (10) mais novos da pilha ativa e para o último commit
  que o workbench fez (`project.rs:46`, `:176`, `:193-195`). três commits
  do topo têm `files`.
- o diff: um só, a região `diff` com `target` (o id cujo diff aparece),
  `lines` (as primeiras `DIFF_LINES` = 200, com `+`, `-`, espaço ou o
  cabeçalho do hunk, que traz `path: @@ ...`) e `count`
  (`project.rs:40`, `:112-133`). um commit mostra o seu diff assim: o
  `show_diff` num `stacks.commit[<sha>]` vira `Op::ShowCommitDiff`
  (`crates/ide/src/workbench/execute.rs:58-59`), o `execute` põe o
  `target` e esvazia as linhas na hora (`execute.rs:123-142`,
  `crates/ide/src/views/diff_view.rs:84-94`) e pede o diff ao worker
  (`crates/ide/src/session.rs:194-201`); as linhas chegam no
  `GitEvent::DiffCommit` (`session.rs:298`). o receipt do `show_diff`
  termina `completed` no `execute`, antes das linhas.
- o que falta no registro para a superfície mostrar o diff de cada
  commit: nada, se o diff é um por vez, como no painel fixo. a superfície
  liga uma tabela a `diff:lines` e um texto a `diff:target`, e o botão de
  cada commit liga o `show_diff` dele: a pessoa abre cada diff pela
  superfície, que é o roteiro da demo 5 ("a pessoa abre cada diff pela
  superfície"). os três diffs ao mesmo tempo pediriam um estado por
  commit (um `diff` em cada `stacks.commit[<sha>]`), uma leitura do git
  por commit e mudariam o golden `crates/uir/fixtures/ide-workbench.snapshot.json`,
  que `crates/ide/tests/uir_fixture.rs:31-66` confere a cada corrida: fica
  de fora, anotado como P3. nenhum tipo do `sem` muda. a projeção fixa do
  ide não muda em nada: o bloco gerado só entra quando o host liga ele
  (item b), e o golden continua igual.

### (b) onde a linha e o host moram na janela do ide

- a janela desenha só o workbench: `renderer.rs:25-27` abre o frame e
  chama `Workbench::render` (`crates/ide/src/workbench/mod.rs:234`), que
  desenha o `WorkspaceView` na janela inteira (`crates/ide/src/views/workspace/render.rs:8-128`).
  a janela dá o tamanho ao workspace por `workspace_mut().resize`
  (`crates/ide/src/main.rs:266`, `:317`), roteia o clique por
  `Workbench::click` (`main.rs:387-392`, `mod.rs:242`), a roda por
  `workspace_mut().scroll` (`main.rs:424`) e as teclas pelo `keys.rs`
  (o campo do commit pega os caracteres, `keys.rs:26-37`). o ritmo é o do
  `Wake` (`pace.rs`).
- o `WorkspaceView` já se deixa enquadrar sem mudar: `resize(vw, vh)`
  (`crates/ide/src/views/workspace/mod.rs:334`) refaz as larguras dos
  painéis a partir das desejadas; o `set_frame` (`:252`) também, mas liga
  o modo embutido, que tira o header (`render.rs:30`). o header desenha
  em y = 0 e só com `origin` zero faz sentido fora do modo embutido.
- o mínimo sem mexer nas vistas fixas: o workbench guarda o tamanho da
  janela e, com o bloco gerado ligado, dá ao workspace a janela menos uma
  faixa embaixo (a linha de intenção: campo, proponente, propor, fechar,
  e a mensagem) e, com uma superfície aberta, menos uma coluna à
  direita, onde o host desenha a superfície (a metade da janela, como a
  Silences, `crates/showcase/src/view/apps/silences/generated.rs:202-231`),
  recortada à coluna. tudo pelo `resize` público do workspace; nenhum
  arquivo de `crates/ide/src/views/**` muda. o shell passa a chamar
  `Workbench::resize` e `Workbench::scroll` e manda ao workbench as teclas
  enquanto a linha ou um campo da superfície tem o teclado.

### (c) as regras

- o `propose::Rules` lê a intenção por uma gramática pequena
  (`crates/propose/src/words.rs:204`) e liga por estrutura
  (`crates/propose/src/rules.rs:448-520`): uma palavra que não é da
  gramática nem do app é `NoMatch` com as palavras do app (`:466`),
  nenhuma palavra lida é `NotUnderstood` (`:452`), uma parte pedida que o
  app não tem é `Nothing` (`:473`). a Silences propõe pelas regras dentro
  do `execute` (`crates/showcase/src/view/apps/silences/generated/page.rs:263-281`)
  e abre ou transforma no lugar (`page.rs:202-228`); o todo, o mesmo pela
  página Surface.
- a parte "lista" pede um `List` ou `Set` cujos valores são as chaves
  dos filhos do nó (`crates/propose/src/list.rs:88-106`, `:108-160`). no
  ide isso é `changes:files`; `stacks` não tem esse estado (item a), então
  as regras de hoje não veem os commits como lista, e "review these three
  commits" é recusado: "review" e "three" não são palavras do app
  (`NoMatch`).
- o que muda, só por adição, em `crates/propose/src/review.rs` (novo): a
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
  (`crates/surface/src/surface.rs:381-398`), e entrega ao `Dispatch` do
  app. no ide o `Dispatch` é o workbench: `Workbench::dispatch`
  (`mod.rs:150-160`), o dispatcher único, com o receipt da pessoa. a
  recusa stale é a do `sem` (`crates/sem/src/check.rs:174-193`), igual
  para qualquer ato.
- um agente pela porta: o `Served` do ide projeta `Workbench::snapshot` e
  despacha em `Workbench::dispatch` (`crates/ide/src/agent.rs:85-105`).
  com o bloco ligado, o snapshot leva a superfície (o host projeta depois
  da projeção do app, `crates/surface/src/host.rs:557`), e o
  `Workbench::dispatch` passa o request pelo `SurfaceHost::route`
  (`host.rs:565`, `surface.rs:537-565`) antes do dispatcher: um ato num nó
  da superfície vira a capacidade do app, com a observação renomeada, e
  segue o mesmo caminho.
- a gravação do ide (`crates/ide/src/record.rs`): o sink no dispatcher
  (`:126`), o estado por `Workbench::snapshot` (`:132`), os frames com as
  ligações (`:141`). com o bloco ligado a superfície entra no estado e nas
  ligações sem mudar o `record.rs`. fechar é `SurfaceHost::close`
  (`host.rs:469`): sai da projeção, do roteamento e do hit na hora.

### (e) o laboratório

- a pessoa roteirizada (task-93) responde o modal com um clique no meio
  do botão como ele foi desenhado no último frame, pelo
  `Workbench::click`, e o clique é gravado como input
  (`crates/ide/src/bin/lab/answer.rs:161-198`). o laboratório grava estado
  e frame a cada mudança (`crates/ide/src/bin/lab/lab.rs:286`) com o
  renderer headless (`crates/ide/src/bin/lab/bench.rs:100-116`), sem
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
  o programa (`crates/showcase/src/view/apps/surface_page/asking.rs:143`),
  uma thread com o `ClaudeCli` e o `propose::Llm` pelo `propose_verified`
  (`asking.rs:255-306`); o `propose` responde `Execution::Started` e o
  settle termina o request. o seletor começa no Claude quando ele existe
  (`crates/showcase/src/view/apps/surface_page/proposing.rs:279-283`).
- o ide entra pelo mesmo caminho (`claude-model` como dependência, pelo
  `ClaudeCli`, a thread e o `Llm`), com o seletor começando em regras.
- a privacidade: o prompt leva todos os nós de `ask.app` com os valores
  (`crates/propose/src/prompt.rs:192-230`, cada valor cortado em 60
  caracteres, `:15`). o snapshot inteiro do ide levaria os arquivos
  mudados da árvore de trabalho, as mensagens de todos os commits, o
  autor, a mensagem do commit em edição e o diff de um arquivo fora dos
  commits. então o ide dá ao `Llm` um snapshot recortado: só os commits
  que as regras leem da intenção (a mensagem e os arquivos), e a região
  do diff só quando ela mostra um deles. testado com o `fake-claude` do
  repositório (`crates/claude-model/tests/fake-claude`, que guarda o
  prompt em `prompt.N`), nunca com o `claude -p`.

### dependências e arquivos

`crates/ide/Cargo.toml` ganha `surface`, `propose`, `uir` e `claude-model`
(o Claude entra pelo caminho do showcase), todos do workspace. no
`Cargo.lock` muda só a lista `dependencies` do pacote `ide`.
