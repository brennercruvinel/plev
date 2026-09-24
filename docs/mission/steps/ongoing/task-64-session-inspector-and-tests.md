---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-23
domain: generative-runtime
task: task-64
wave: 5
depends: [task-63, task-66]
depends-optional: [task-65]
branch: task/64-session-inspector
---

# task-64: inspector de sessão, bug report e sessão para teste

## objetivo

abrir uma sessão gravada, navegar no tempo com scrub, escolher um controle
num instante e ver o estado semântico, a ação, o ator, a precondição, o
delta e o resultado; exportar um trecho como bug report; e transformar um
trecho escolhido em teste de regressão.

## ligação com a tese

seções 11 (open session, scrub timeline, select control, inspect semantic
state, action, actor, result), 12 (bug report deixa de ser mídia; sessão
vira teste), 13 (no replay, selecionar um controle leva ao código daquela
versão), 28 (a pergunta do núcleo: o que alguém precisa descobrir no
primeiro `.monster` para confiar na delegação).

## estado atual no código

- `monster_player` (`crates/engine/examples/monster_player`) toca
  animação com seek e scrub pelo player determinístico de
  `crates/monster/src/play.rs`.
- depois de 63: `Session::open`, estado semântico em t, receipts por
  `SemId`, trilha visual.
- depois de 66: render headless de frame.

## escopo

### inspector

- um app ou uma página do showcase (decisão no briefing; recomendação:
  crate de app próprio, porque abre arquivo e vai crescer) que abre uma
  sessão.
- timeline com os eventos marcados por tipo e por ator; scrub com seek
  O(1) no keyframe mais próximo.
- o frame do instante, com as ligações desenhadas por cima (contorno do
  controle, `SemId` no hover).
- clique num controle no instante t: painel com o nó (role, label, state),
  os receipts que tocam esse `SemId` até t, e para cada receipt a cadeia
  causal: observação (quem, qual revisão), request (ator, `observed`),
  veredito (com o delta quando stale), execução, resultado.
- as perguntas da tese respondidas na tela: qual controle era esse, qual
  era o estado, qual foi a sequência anterior, qual ação ocorreu, quem
  iniciou, qual era a precondição, o que mudou, qual foi o resultado.
- link para o código quando a task-65 estiver integrada: o `source` do nó
  mais a identidade do build no cabeçalho da sessão; se a árvore de
  trabalho diverge daquele sha, mostrar o arquivo da versão gravada
  (`git show <sha>:<arquivo>`), não o atual.

### bug report

exportar `[t0, t1]` a partir do inspector (usa o export da task-63).

### sessão para teste

- o desenvolvedor escolhe, no inspector, os estados e resultados que
  viram asserções.
- a saída é um arquivo de dados (o trecho com as asserções marcadas) e um
  harness genérico que roda o trecho contra uma fixture do app: aplica as
  requests na ordem, confere os vereditos e os valores marcados.
- isso é reexecução controlada, e o documento diz isso: o harness roda num
  ambiente isolado (tempdir, relógio injetado, sem rede), a fixture é
  fornecida pelo app, e nada toca estado externo real. replay no
  inspector continua sem executar nada.

## fora de escopo

reexecução de efeitos externos reais; edição de sessão; compartilhamento
remoto.

## dependências

63 e 66 integradas; 65 para o link de código.

## superfície sob responsabilidade

o crate ou página do inspector; o harness de teste de sessão (onde o ADR
de 63 colocou o leitor); proposta de ADR
`docs/ADR/sessions-become-regression-tests.md`.

## interfaces que consome

`Session` (63), render headless (66), `Node.source` (57) e o resolvedor de
código (65), comps para a UI.

## interfaces que pode alterar

as próprias.

## critérios de aceitação

- a sessão do cenário do commit (gravada em 63) aberta no inspector: scrub
  até antes do commit, clique no botão, painel mostra o estado que o agente
  observou, o request, a rejeição com "removed: B", depois o aceite e o sha.
- export de trecho reabre com o mesmo conteúdo.
- um teste gerado a partir da sessão roda verde no gate e falha quando a
  regra de precondição é quebrada de propósito (prova que o teste pega
  regressão).
- inspector não cria request em nenhum caminho (teste).
- `./script/gate` verde.

## testes esperados

testes do modelo do inspector sem UI (estado em t, cadeia causal por
`SemId`), testes de viewport estreito e largo da UI, teste do harness com
uma fixture de brinquedo.

## riscos e perguntas abertas

- asserção gerada acoplada demais (todo o estado) fica frágil: o default
  é só o que o desenvolvedor marcou.
- a fixture do app é trabalho do app; o harness só define a interface.

## commits esperados

1. `task-64: modelo do inspector (estado em t, cadeia causal)`
2. `task-64: UI com timeline, frame e ligações`
3. `task-64: painel do controle e perguntas da tese`
4. `task-64: export de trecho`
5. `task-64: sessão para teste com harness isolado`

## pare e reporte se

- a geração de teste exigir reexecução que toque estado externo;
- o formato de sessão não der acesso O(1) que o scrub precisa.

## fases (orquestrador, 2026-09-23)

a task é grande (modelo, UI, export, sessão para teste) e foi dividida:

- **fase a, agora:** o modelo do inspector sem UI e uma CLI
  (`plev-inspect`) que responde às perguntas da tese sobre uma sessão
  gravada: o que era o controle, qual era o estado, quem observou o quê e
  quando, qual request, qual veredito (com o delta e o `acted`), o que
  aconteceu entre a observação e o request (as mudanças no que a ação lê
  e os receipts de outros atores que rodaram nesse intervalo, pela
  gravação), e o resultado. responde por tempo a pergunta "quem mudou o
  que eu li", que o `acted` da task-83 responde só em parte. funciona já
  nas sessões dos dois checkpoints a (`target/agents/exp/`,
  `target/agents/exp2/`).
- **fase b:** a UI (timeline, frame com as ligações por cima, painel do
  controle). pede gravar as ligações por frame no `rec`, que hoje não
  grava (`state/truth.md`, 2.4).
- **fase c:** export de trecho como bug report e sessão para teste com o
  harness isolado.

## fase a: handoff (2026-09-23)

```text
Goal: task-64 fase a, o modelo do inspector de sessão e a CLI plev-inspect
Status: pronto para revisão (branch task/64a-inspector-model, rebaseada na main @ e6b1039)
```

### implementado

- crate novo `crates/inspect` (lib `inspect`, bin `plev-inspect`),
  dependendo só de `rec`, `sem`, `serde` e `serde_json`. `Inspector`
  empresta uma `rec::Session` e não guarda mais nada: nenhum app, nada que
  despacha, nenhum recorder.
- `timeline()`: um item por evento, com o tempo desde o começo da sessão e
  o evento resumido; contagem por tipo e por ator (o input de uma pessoa
  conta para `human`, como o `rec` grava).
- `why(origin, id)`, a cadeia causal de um request:
  (a) a observação em que ele se apoiou: o último `Observed` do mesmo ator
  com a revisão igual a `request.observed.revision` antes do primeiro
  estágio gravado; sem ela, "não gravada" e por quê (nenhuma observação
  desse ator: uma pessoa age sobre o frame da tela, e o recorder só guarda
  o que o app reporta; ou só em outras revisões: a revisão veio de outro
  jeito, como o último estágio de um ato anterior), mais o último instante
  em que o estado gravado teve aquela revisão, que passa a abrir o trecho;
  (b) o request: ator, alvo (o nó como o estado o tinha logo antes), ação,
  parâmetro, e cada `read` com o valor que o ator viu, e se o nó estava
  dentro do escopo da observação;
  (c) os estágios com tempo, o veredito, o delta e o `acted` da recusa
  stale;
  (d) entre a observação e o request: cada mudança de cada `read`, lida
  no estado depois de cada evento, com os receipts que rodaram naquela
  janela, e o input de uma pessoa no trecho;
  (e) como terminou: outcome (sha, arquivos), recusa (no check ou no ponto
  de verdade), falha, ou aberto.
- `control(id, t)`: o nó no estado gravado em t (role, label, estado,
  ações, o `source` de onde foi projetado ao lado do sha do build) e os
  requests que tocam o id até t (`Session::receipts_for`), cada um com o
  seu `why` como a janela até t conta.
- `summary()`: build, span, atores, todo request com como terminou, toda
  recusa com o seu `why`.
- `text`: cada resposta em inglês direto; a cadeia causal tem uma linha
  rotulada por pergunta da tese (control, state, initiated by, action,
  observed, precondition, before, stages, verdict, what changed, acted,
  result). todo tipo do modelo é `Serialize`.
- `plev-inspect <session.monster> [--timeline] [--why <id> [--origin <n>]]
  [--control <id> [--at <ms>]] [--json]`: o arquivo sozinho dá o resumo;
  `--at` em ms desde o começo da sessão, o mesmo tempo que as views
  imprimem; parse sem dependência, `--help`, exit 0 respondido, 1 request
  que a sessão não tem, 2 invocação errada, 3 arquivo que não lê como
  sessão (o `rec::FileError` dito como é).
- ADR `docs/ADR/the-inspector-reads-a-session-never-runs-it.md`; entrada
  `[crates.inspect]` no `arc.toml`; `inspect` na lista de crates do README
  e uma linha de comando na seção de comandos.

### o critério de correlação

para uma mudança de um `read` gravada no evento j, a janela vai de logo
depois do último evento antes de j que mudou o estado (snapshot ou
changes) até j. entram os receipts, fora o explicado, com um estágio
`executing`, `completed` ou `failed` nessa janela e cujo último estágio não
é uma recusa (recusado no check, na confirmação ou no ponto de verdade, não
rodou nada). nenhum: veio de fora do app, sem request. um: aquele. dois ou
mais: ambíguo, todos mostrados. os atos do próprio ator entram, marcados
como o mesmo ator, para que uma mudança dele nunca vire "de fora do app".
o texto e o doc do modelo dizem sempre que é correlação, não declaração.

duas decisões locais, registradas no ADR: o estado do trecho é lido depois
de cada evento, não por tempo (o recorder grava os estágios de um request e
a projeção que ele deixa no mesmo milissegundo; por tempo, o efeito do
próprio request ficaria antes dele), com o leitor público da sessão
(`Session::from_events` a partir do keyframe, depois `state_at`); e os atos
do mesmo ator contam, o que é mais largo que "de outro ator" do briefing,
pelo motivo acima.

limites (no ADR): correlação não é causa; um receipt cujo estágio terminal
chega depois do efeito (o ide: a view muda logo depois de `executing`, o
`completed` chega ms depois) cai também na janela da mudança seguinte, e
dois atos seguidos podem sair ambíguos; um receipt que começa antes do
estado anterior à mudança e termina depois dela não tem estágio na janela e
não é nomeado (a mudança lê como de fora do app; o ide não faz isso, um
teste fixa o comportamento); uma mudança de fora do app que cai na mesma
janela de um receipt vai para o receipt; o anel corta observação e
primeiros estágios (`not recorded`, `whole: false`).

### saídas de aceite

rodadas de `/Users/nn/Dev/hoff/plevdev` com o binário do target da task.

segunda rodada, o primeiro e o segundo submit:

```text
$ plev-inspect target/agents/exp2/a2pa/session.monster --why 7
request 7 (origin 0): submit on commit.submit by agent:claude-a2pa, at +118.472s
  control       commit.submit: button "Commit", enabled
  state         none of its own
  initiated by  agent:claude-a2pa
  action        submit, no parameter
  observed      +111.731s, the whole app, at revision f6763a3f27895037
                (the last observation by agent:claude-a2pa at the revision the request carries)
  precondition  what the action reads, as agent:claude-a2pa saw it:
                  changes:staged = {src/login.rs, src/session.rs, tests/login_test.rs}
                  commit.message:text = "fix login session timeout"
  before        from the observation (+111.731s) to the request (+118.472s):
                  +111.744s changes:staged: removed src/session.rs
                            by human, request 6: unstage on changes.file[src/session.rs] (executing at +111.744s)
                  commit.message:text did not move
                  +111.744s input: pointer_down on changes.file[src/session.rs]
  stages        +118.472s requested
                +118.472s rejected: stale
  verdict       rejected at +118.472s: stale, a state the action reads changed since the observation
  what changed  changes:staged: removed src/session.rs
  acted         human, request 6: unstage on changes.file[src/session.rs]
  result        nothing ran: the refusal came before any effect
  note          "by" is a correlation in time, not a declared write: the receipts that ran
                (executing, completed or failed) between the recorded state before a change and
                the change. the contract declares what an action reads, never what it writes.
exit 0

$ plev-inspect target/agents/exp2/a2pa/session.monster --why 9
request 9 (origin 0): submit on commit.submit by agent:claude-a2pa, at +161.018s
  control       commit.submit: button "Commit", enabled
  state         none of its own
  initiated by  agent:claude-a2pa
  action        submit, no parameter
  observed      +153.450s, the whole app, at revision 3f65366b35e09e9b
                (the last observation by agent:claude-a2pa at the revision the request carries)
  precondition  what the action reads, as agent:claude-a2pa saw it:
                  changes:staged = {src/login.rs, src/session.rs, tests/login_test.rs}
                  commit.message:text = "fix login session timeout"
  before        from the observation (+153.450s) to the request (+161.018s):
                  changes:staged did not move
                  commit.message:text did not move
  stages        +161.018s requested
                +161.018s accepted
                +161.018s executing
                +161.062s completed: commit = "784572b0497cea7fbe04dc05c60a3925538da8d8", files = [src/login.rs, src/session.rs, tests/login_test.rs]
  verdict       accepted at +161.018s
  result        completed at +161.062s: commit = "784572b0497cea7fbe04dc05c60a3925538da8d8", files = [src/login.rs, src/session.rs, tests/login_test.rs]
exit 0
```

primeira rodada, gravada antes do `acted` (a recusa não nomeia ninguém; a
gravação nomeia a pessoa):

```text
$ plev-inspect target/agents/exp/a2/session.monster --why 7
request 7 (origin 0): submit on commit.submit by agent:claude-a2, at +103.620s
  control       commit.submit: button "Commit", enabled
  state         none of its own
  initiated by  agent:claude-a2
  action        submit, no parameter
  observed      +97.599s, the subtree commit.form, at revision b92c0e4bec9b5227
                (the last observation by agent:claude-a2 at the revision the request carries)
  precondition  what the action reads, as agent:claude-a2 saw it:
                  changes:staged = {src/login.rs, src/session.rs, tests/login_test.rs}, outside the scope observed: that observation did not serve it
                  commit.message:text = "fix login session timeout"
  before        from the observation (+97.599s) to the request (+103.620s):
                  +97.627s changes:staged: removed src/session.rs
                            by human, request 6: unstage on changes.file[src/session.rs] (executing at +97.627s)
                  commit.message:text did not move
                  +97.627s input: pointer_down on changes.file[src/session.rs]
  stages        +103.620s requested
                +103.620s rejected: stale
  verdict       rejected at +103.620s: stale, a state the action reads changed since the observation
  what changed  changes:staged: removed src/session.rs
  acted         the refusal names nobody
                (acted names only other actors' receipts checked on the exact revision the
                author saw; a change from outside the app never shows there, nor does anyone
                in a refusal recorded before acted existed. before says what the recording
                shows)
  result        nothing ran: the refusal came before any effect
  note          "by" is a correlation in time, not a declared write: the receipts that ran
                (executing, completed or failed) between the recorded state before a change and
                the change. the contract declares what an action reads, never what it writes.
exit 0

$ plev-inspect target/agents/exp/a2/session.monster --why 9
request 9 (origin 0): submit on commit.submit by agent:claude-a2, at +147.621s
  control       commit.submit: button "Commit", enabled
  state         none of its own
  initiated by  agent:claude-a2
  action        submit, no parameter
  observed      +139.849s, the whole app, at revision 2361f15ea7773d8f
                (the last observation by agent:claude-a2 at the revision the request carries)
  precondition  what the action reads, as agent:claude-a2 saw it:
                  changes:staged = {src/login.rs, src/session.rs, tests/login_test.rs}
                  commit.message:text = "fix login session timeout"
  before        from the observation (+139.849s) to the request (+147.621s):
                  changes:staged did not move
                  commit.message:text did not move
  stages        +147.621s requested
                +147.621s accepted
                +147.621s executing
                +147.712s completed: commit = "7c21633954cbe8506305cb9bd112e46b5bad3620", files = [src/login.rs, src/session.rs, tests/login_test.rs]
  verdict       accepted at +147.621s
  result        completed at +147.712s: commit = "7c21633954cbe8506305cb9bd112e46b5bad3620", files = [src/login.rs, src/session.rs, tests/login_test.rs]
exit 0
```

as nove sessões listadas (as sete do briefing mais `lab-control` e
`lab-control2`, que estavam na lista de caminhos), sem argumento:

```text
$ plev-inspect target/agents/exp/a1/session.monster
session   rec 0.1.0, built from d26c9b1
span      +0.000s to +149.588s, 61 events, 15 frames
actors    agent:claude-a1: 7 observations, 7 requests, 28 stages
requests  7
  +53.702s   request 1  agent:claude-a1  unstage on changes.file[README.md]                                    completed
  +53.718s   request 2  agent:claude-a1  stage on changes.file[src/session.rs]                                 completed
  +53.741s   request 3  agent:claude-a1  stage on changes.file[tests/login_test.rs]                            completed
  +75.977s   request 4  agent:claude-a1  open on commit.form                                                   completed
  +88.439s   request 5  agent:claude-a1  set_value on commit.message                                           completed
  +103.728s  request 6  agent:claude-a1  submit on commit.submit                                               completed: commit = "6eb55b76560fc2fd4fb5752692d594382b10d291", files = [src/login.rs, src/session.rs, tests/login_test.rs]
  +141.582s  request 7  agent:claude-a1  show_diff on stacks.commit[6eb55b76560fc2fd4fb5752692d594382b10d291]  completed
refusals  0
exit 0

$ plev-inspect target/agents/exp/a2/session.monster
session   rec 0.1.0, built from d26c9b1
span      +0.000s to +147.743s, 67 events, 15 frames
actors    agent:claude-a2: 7 observations, 8 requests, 30 stages
          human: 1 request, 4 stages, 1 input
requests  9
  +54.170s   request 1  agent:claude-a2  unstage on changes.file[README.md]          completed
  +55.579s   request 2  agent:claude-a2  stage on changes.file[src/session.rs]       completed
  +56.857s   request 3  agent:claude-a2  stage on changes.file[tests/login_test.rs]  completed
  +75.379s   request 4  agent:claude-a2  open on commit.form                         completed
  +91.844s   request 5  agent:claude-a2  set_value on commit.message                 completed
  +97.627s   request 6  human            unstage on changes.file[src/session.rs]     completed
  +103.620s  request 7  agent:claude-a2  submit on commit.submit                     rejected: stale
  +130.749s  request 8  agent:claude-a2  stage on changes.file[src/session.rs]       completed
  +147.621s  request 9  agent:claude-a2  submit on commit.submit                     completed: commit = "7c21633954cbe8506305cb9bd112e46b5bad3620", files = [src/login.rs, src/session.rs, tests/login_test.rs]
refusals  1
  request 7, submit on commit.submit by agent:claude-a2, rejected at +103.620s: stale
    changes:staged: removed src/session.rs
    changes:staged moved at +97.627s by human, request 6: unstage on changes.file[src/session.rs] (executing at +97.627s)
exit 0

$ plev-inspect target/agents/exp2/a1p/session.monster
session   rec 0.1.0, built from 8942f0d
span      +0.000s to +124.719s, 49 events, 10 frames
actors    agent:claude-a1p: 6 observations, 6 requests, 24 stages
requests  6
  +51.610s   request 1  agent:claude-a1p  unstage on changes.file[README.md]          completed
  +70.769s   request 2  agent:claude-a1p  stage on changes.file[src/session.rs]       completed
  +72.037s   request 3  agent:claude-a1p  stage on changes.file[tests/login_test.rs]  completed
  +90.896s   request 4  agent:claude-a1p  open on commit.form                         completed
  +104.050s  request 5  agent:claude-a1p  set_value on commit.message                 completed
  +115.738s  request 6  agent:claude-a1p  submit on commit.submit                     completed: commit = "9b61c97512239562251823b0e9f55d1e3e525987", files = [src/login.rs, src/session.rs, tests/login_test.rs]
refusals  0
exit 0

$ plev-inspect target/agents/exp2/a2pa/session.monster
session   rec 0.1.0, built from 8942f0d
span      +0.000s to +176.614s, 67 events, 14 frames
actors    agent:claude-a2pa: 7 observations, 8 requests, 30 stages
          human: 1 request, 4 stages, 1 input
requests  9
  +61.312s   request 1  agent:claude-a2pa  unstage on changes.file[README.md]          completed
  +62.702s   request 2  agent:claude-a2pa  stage on changes.file[src/session.rs]       completed
  +64.088s   request 3  agent:claude-a2pa  stage on changes.file[tests/login_test.rs]  completed
  +85.068s   request 4  agent:claude-a2pa  open on commit.form                         completed
  +103.021s  request 5  agent:claude-a2pa  set_value on commit.message                 completed
  +111.744s  request 6  human              unstage on changes.file[src/session.rs]     completed
  +118.472s  request 7  agent:claude-a2pa  submit on commit.submit                     rejected: stale
  +146.143s  request 8  agent:claude-a2pa  stage on changes.file[src/session.rs]       completed
  +161.018s  request 9  agent:claude-a2pa  submit on commit.submit                     completed: commit = "784572b0497cea7fbe04dc05c60a3925538da8d8", files = [src/login.rs, src/session.rs, tests/login_test.rs]
refusals  1
  request 7, submit on commit.submit by agent:claude-a2pa, rejected at +118.472s: stale
    changes:staged: removed src/session.rs
    changes:staged moved at +111.744s by human, request 6: unstage on changes.file[src/session.rs] (executing at +111.744s)
exit 0

$ plev-inspect target/agents/exp2/a2pb/session.monster
session   rec 0.1.0, built from 8942f0d
span      +0.000s to +175.184s, 66 events, 14 frames
actors    agent:claude-a2pb: 7 observations, 8 requests, 30 stages
          human: 1 request, 4 stages, 1 input
requests  9
  +79.990s   request 1  agent:claude-a2pb  unstage on changes.file[README.md]          completed
  +80.008s   request 2  agent:claude-a2pb  stage on changes.file[src/session.rs]       completed
  +80.033s   request 3  agent:claude-a2pb  stage on changes.file[tests/login_test.rs]  completed
  +97.699s   request 4  agent:claude-a2pb  open on commit.form                         completed
  +109.293s  request 5  agent:claude-a2pb  set_value on commit.message                 completed
  +121.898s  request 6  human              unstage on changes.file[src/session.rs]     completed
  +128.568s  request 7  agent:claude-a2pb  submit on commit.submit                     rejected: stale
  +151.682s  request 8  agent:claude-a2pb  stage on changes.file[src/session.rs]       completed
  +165.399s  request 9  agent:claude-a2pb  submit on commit.submit                     completed: commit = "925898ce96d17ebd6a88321485db26d9be39c21a", files = [src/login.rs, src/session.rs, tests/login_test.rs]
refusals  1
  request 7, submit on commit.submit by agent:claude-a2pb, rejected at +128.568s: stale
    changes:staged: removed src/session.rs
    changes:staged moved at +121.898s by human, request 6: unstage on changes.file[src/session.rs] (executing at +121.898s)
exit 0

$ plev-inspect target/agents/lab-scripted/session.monster
session   rec 0.1.0, built from 81a9945
span      +0.000s to +0.325s, 71 events, 16 frames
actors    agent:scripted: 19 observations, 8 requests, 30 stages
          human: 1 request, 4 stages, 1 input
requests  9
  +0.022s  request 1  agent:scripted  unstage on changes.file[README.md]          completed
  +0.039s  request 2  agent:scripted  stage on changes.file[src/session.rs]       completed
  +0.065s  request 3  agent:scripted  stage on changes.file[tests/login_test.rs]  completed
  +0.143s  request 4  agent:scripted  open on commit.form                         completed
  +0.146s  request 5  agent:scripted  set_value on commit.message                 completed
  +0.160s  request 6  human           unstage on changes.file[src/session.rs]     completed
  +0.162s  request 7  agent:scripted  submit on commit.submit                     rejected: stale
  +0.177s  request 8  agent:scripted  stage on changes.file[src/session.rs]       completed
  +0.250s  request 9  agent:scripted  submit on commit.submit                     completed: commit = "117a78ba3c9c8ed101eb3c402e0af3dd562428b0", files = [src/login.rs, src/session.rs, tests/login_test.rs]
refusals  1
  request 7, submit on commit.submit by agent:scripted, rejected at +0.162s: stale
    changes:staged: removed src/session.rs
    changes:staged moved at +0.161s by human, request 6: unstage on changes.file[src/session.rs] (executing at +0.160s)
exit 0

$ plev-inspect target/agents/lab-plev/session.monster
session   rec 0.1.0, built from 81a9945
span      +0.000s to +9.782s, 58 events, 12 frames
actors    agent:plev: 8 observations, 8 requests, 30 stages
          human: 1 request, 4 stages, 1 input
requests  9
  +6.376s  request 1  agent:plev  unstage on changes.file[README.md]          completed
  +6.439s  request 2  agent:plev  stage on changes.file[src/session.rs]       completed
  +6.479s  request 3  agent:plev  stage on changes.file[tests/login_test.rs]  completed
  +7.546s  request 4  agent:plev  open on commit.form                         completed
  +7.579s  request 5  agent:plev  set_value on commit.message                 completed
  +7.599s  request 6  human       unstage on changes.file[src/session.rs]     completed
  +7.613s  request 7  agent:plev  submit on commit.submit                     rejected: stale
  +8.657s  request 8  agent:plev  stage on changes.file[src/session.rs]       completed
  +9.707s  request 9  agent:plev  submit on commit.submit                     completed: commit = "60685b2618995f2efdf915701c4bc60320250e92", files = [src/login.rs, src/session.rs, tests/login_test.rs]
refusals  1
  request 7, submit on commit.submit by agent:plev, rejected at +7.613s: stale
    changes:staged: removed src/session.rs
    changes:staged moved at +7.600s by human, request 6: unstage on changes.file[src/session.rs] (executing at +7.599s)
exit 0

$ plev-inspect target/agents/lab-control/session.monster
session   rec 0.1.0, built from d947b19
span      +0.000s to +0.410s, 46 events, 12 frames
actors    agent:scripted: 10 observations, 6 requests, 24 stages
requests  6
  +0.050s  request 1  agent:scripted  unstage on changes.file[README.md]          completed
  +0.084s  request 2  agent:scripted  stage on changes.file[src/session.rs]       completed
  +0.141s  request 3  agent:scripted  stage on changes.file[tests/login_test.rs]  completed
  +0.242s  request 4  agent:scripted  open on commit.form                         completed
  +0.247s  request 5  agent:scripted  set_value on commit.message                 completed
  +0.253s  request 6  agent:scripted  submit on commit.submit                     completed: commit = "0222ec09e77f86969d33f03085490a7440fd7858", files = [src/login.rs, src/session.rs, tests/login_test.rs]
refusals  0
exit 0

$ plev-inspect target/agents/lab-control2/session.monster
session   rec 0.1.0, built from 5217e3d
span      +0.000s to +0.224s, 46 events, 12 frames
actors    agent:scripted: 10 observations, 6 requests, 24 stages
requests  6
  +0.018s  request 1  agent:scripted  unstage on changes.file[README.md]          completed
  +0.034s  request 2  agent:scripted  stage on changes.file[src/session.rs]       completed
  +0.059s  request 3  agent:scripted  stage on changes.file[tests/login_test.rs]  completed
  +0.137s  request 4  agent:scripted  open on commit.form                         completed
  +0.140s  request 5  agent:scripted  set_value on commit.message                 completed
  +0.143s  request 6  agent:scripted  submit on commit.submit                     completed: commit = "9f333181fb5d3175704474048ae7b509abccbce8", files = [src/login.rs, src/session.rs, tests/login_test.rs]
refusals  0
exit 0
```

além disso, em cada uma das nove: `--timeline`, `--timeline --json`,
`--why` de todo request e `--control` de sete ids (`changes`,
`commit.submit`, `commit.message`, `commit.form`,
`changes.file[src/session.rs]`, `stacks`, `notices`) no fim e em `--at 0`,
texto e json: todos com exit 0, e o sha256 de cada arquivo igual antes e
depois. nas nove, cinco `read`s se moveram entre uma observação e um
request (o submit stale de cada sessão com a pessoa roteirizada); cada
mudança caiu em exatamente um receipt, o unstage da pessoa (request 6), pelo
estágio `executing` gravado no mesmo milissegundo da mudança ou no anterior.
nenhuma saiu ambígua nem de fora do app: a correlação não deu resultado
enganoso em nenhuma sessão real. exit 1 para `--why 99`, exit 3 para arquivo
ausente, texto que não é sessão e animação `MON0` (com a mensagem do `rec`).

### testes

`crates/inspect/tests/model/` (33) e o parse da CLI (3), com sessões
montadas pelo `Recorder` sobre um git de brinquedo por um `Dispatcher` de
verdade (o padrão de `crates/rec/tests/session/`):

- timeline: um item por evento, tempo relativo, contagem por tipo e ator;
- observação encontrada (com escopo e sem) e não gravada (pessoa; revisão
  que nenhuma observação serviu; revisão que o estado nunca teve);
- stale com a mudança causada por outro ator (a pessoa, request nomeado,
  executing no mesmo ms) e por fora do app (`By::Outside`, e o `acted`
  vazio também);
- request concluído com sha e arquivos;
- correlação ambígua (pessoa e tecnologia assistiva antes de uma projeção
  só: os dois mostrados);
- um receipt recusado nunca explica mudança, inclusive recusado no ponto de
  verdade; o ato do próprio ator entra marcado;
- o efeito do próprio request no mesmo ms não aparece antes dele;
- janela que começa depois do request (`whole: false`);
- o limite documentado: receipt rodando através da janela não é nomeado;
- control antes, durante e depois; request ainda executando em t fica
  aberto; controle que o estado não tem;
- texto: cada pergunta da tese numa linha rotulada; "de fora do app" e
  "ambíguo" ditos; resumo, timeline e control;
- binário: todos os modos sobre um arquivo escrito, json com a ordem de
  campos do modelo, códigos de saída, bytes do arquivo iguais depois;
- ler não roda nada: inspecionar tudo, em todo instante, deixa o app, o
  dispatcher e a gravação como estavam (e uma sessão fria, só com os
  eventos, responde igual); os fontes de `src/` não nomeiam `Dispatcher`,
  `Operable`, `ReceiptSink`, `Execution`, `Recorder`, nem chamam
  `dispatch`, `execute`, `confirm` ou `decline`, e o `[dependencies]` é
  exatamente `rec`, `sem`, `serde`, `serde_json`.

### validação

- `./script/gate` verde na árvore rebaseada (com a perna urnaui): 2199
  passados (2117 do workspace, 82 do urnaui), 0 falhas, 12 ignorados. a
  main na task-67 registrou 2160; o `inspect` soma 36 (33 do modelo, 3 do
  parse). os outros 3 vêm de fora do crate e não investiguei (a perna
  urnaui lê o checkout irmão `../urna`, com 82 testes aqui e 80 no log da
  task-87). a contagem não cai.
- o `file_hygiene_guard` novo da main passa: o maior arquivo de produção do
  crate é `src/why.rs` (579 linhas); `text` foi dividido em `text/mod.rs` e
  `text/chain.rs` por responsabilidade.

### linha proposta para o índice de ADRs

```text
| 2026-09-23 | [the-inspector-reads-a-session-never-runs-it](the-inspector-reads-a-session-never-runs-it.md) | accepted | o crate `inspect` lê uma sessão gravada e responde as perguntas da tese sobre qualquer request ou controle, sem executar nada (nenhum app, nada que despacha); `why` acha a observação do mesmo ator na revisão do request, lê o estado depois de cada evento e põe cada mudança no que o request lê nos receipts que rodaram entre o estado anterior e a mudança, por correlação no tempo e nunca por escrita declarada (nenhum: de fora do app; dois ou mais: ambíguo, todos mostrados); CLI `plev-inspect` com resumo, timeline, why, control e json (task-64 fase a) |
```

### o que a fase b (UI) vai precisar do `rec`

- as ligações por frame (`Compositor::bindings`), que o `rec` não grava
  (`state/truth.md` 2.4): sem elas não se liga um ponto do frame
  reproduzido a um controle.
- uma leitura do estado depois de um índice de evento em `Session` (e o
  índice de evento de cada estágio no `ReceiptAt`). o modelo reconstrói o
  estado de um trecho por mudança, a partir do keyframe, com
  `Session::from_events`; nas sessões do checkpoint isso é instantâneo, mas
  um scrub por evento numa janela longa entre keyframes pagaria isso a cada
  passo.
- nada mais para o que a fase a responde: `state_at`, `receipts`,
  `receipts_for`, `keyframe_at`, `scene_at` e `frame_at` bastaram.

observação para depois da fase b (não é do `rec` sozinho): desde a task-87
o último estágio de um ato leva a revisão em que o app está, e o `plev act
--from-revision` aceita ela; o servidor não reporta isso como observação
(`Host::observed`, `crates/wire/src/server.rs`), e o `rec` não grava a
revisão no estágio. numa sessão gravada depois disso, um ato encadeado
nessa revisão aparece no inspector como "não gravada: as observações do ator
estão em outras revisões", com o instante em que o estado teve aquela
revisão. a resposta é correta, mas não diz que a revisão veio do request
anterior; gravar a revisão servida com o estágio terminal resolveria.

### limitações e riscos de integração

- a correlação é por tempo, com os limites acima; nenhuma sessão real os
  exercita hoje, e os testes fixam cada um.
- `--at` é relativo ao começo da sessão, e o `t` absoluto (relógio do
  recorder) só aparece no json.
- o crate traz o `engine` junto pelo `rec` (o `rec` depende dele); o
  `plev-inspect` compila com a pilha de render, mesmo sem usá-la.
- nenhuma interface compartilhada mudou; `rec`, `sem` e o formato `MONS`
  intactos.

### commits

- `449b748` task-64: the inspect crate and the timeline model
- `bfa30b4` task-64: why, the causal chain of a request
- `c7c715f` task-64: control, a control at an instant
- `e1d7338` task-64: plev-inspect, the inspector's command line
- este: task-64: the inspector ADR and the phase a handoff

## fase a integrada (orquestrador, 2026-09-23)

integrada na `main` em `e6560a2` (cinco commits). validação: diff lido
(o modelo só empresta a `Session`; nenhum app, nada que despacha; o
critério de correlação escrito como correlação); gate verde no diretório
da task: 2199 passaram, 0 falharam, 12 ignorados, com o guard das 639
linhas. reproduzido pelo orquestrador: `plev-inspect` na sessão do A2'a
(`--why 7`) mostra a observação, as `reads`, a mudança da pessoa no
intervalo (request 6), a recusa e o `acted`; na sessão do A2 da primeira
rodada, o resumo e o `--why 7` com a leitura fora do escopo observado.

o que a fase a achou: o submit do A2 da primeira rodada se apoiou numa
observação só de `commit.form`, e o servidor montou o `changes:staged`
observado a partir do snapshot inteiro. isso revelou que um `act
--from-revision` pode dizer ter visto estado que nunca foi mostrado à
conexão: virou a task-89 (invariante quebrada). fases b (UI) e c (export,
sessão para teste) continuam na fila; o que a b precisa do `rec` está no
handoff acima.

## fase c: sonda (2026-09-24)

antes de implementar, a pergunta da sonda: os requests de uma sessão real,
reaplicados na ordem gravada contra uma fixture nova do mesmo repositório,
dão os mesmos vereditos?

### os requests gravados

sessão `target/agents/exp3/a2ppa/session.monster` (terceira rodada, o
`ide-lab` do build `96c20a0`, agente LLM `claude-a2ppa`, pessoa
`unstage-before-submit:src/session.rs`), lida com o `plev-inspect` do
target da task (`--timeline` e `--why` de cada request). origem 0, um
dispatcher só:

```text
n  +t        ator                alvo                               ação       observed (os reads, como o ator viu)                           veredito terminal
1  +41.011s  agent:claude-a2ppa  changes.file[README.md]            unstage    staged=true, unstaged=false                                    accepted, completed
2  +42.343s  agent:claude-a2ppa  changes.file[src/session.rs]       stage      staged=false, unstaged=true                                    accepted, completed
3  +43.703s  agent:claude-a2ppa  changes.file[tests/login_test.rs]  stage      staged=false, unstaged=true                                    accepted, completed
4  +56.210s  agent:claude-a2ppa  commit.form                        open       commit.form:open=false                                         accepted, completed
5  +62.426s  agent:claude-a2ppa  commit.message                     set_value  commit.message:text=""  (param "fix login session timeout")    accepted, completed
6  +70.679s  human               changes.file[src/session.rs]       unstage    staged=true, unstaged=false                                    accepted, completed (+70.693s)
7  +70.679s  agent:claude-a2ppa  commit.submit                      submit     changes:staged={src/login.rs, src/session.rs,                  rejected: stale, changes:staged
                                                                               tests/login_test.rs}, commit.message:text="fix login           removed src/session.rs
                                                                               session timeout"
8  +95.083s  agent:claude-a2ppa  changes.file[src/session.rs]       stage      staged=false, unstaged=true                                    accepted, completed
9  +110.643s agent:claude-a2ppa  commit.submit                      submit     changes:staged={src/login.rs, src/session.rs,                  accepted, completed: commit bf680c84,
                                                                               tests/login_test.rs}, commit.message:text="fix login           files [src/login.rs, src/session.rs,
                                                                               session timeout"                                               tests/login_test.rs]
```

o request 6 é a pessoa roteirizada: o laboratório o despacha logo antes do
submit 7 chegar ao dispatcher, e o 6 ainda estava `executing` quando o 7 foi
recusado (o worker respondeu 14 ms depois).

### a fixture e como ela alcança o repositório

`Repo::build` morava no bin do laboratório (`src/bin/lab/repo.rs`), fora do
alcance de um teste. a fixture o alcança sem duplicar: o arquivo foi movido
com `git mv` para a lib, `crates/ide/src/lab_repo.rs` (`pub mod lab_repo`,
conteúdo igual), e o bin do laboratório passou a importá-lo com `use
ide::lab_repo as repo;`, então todo `crate::repo::...` do laboratório
continua como estava. a fixture é a mesma montagem do `Bench::open` do
laboratório e do `Rig` dos testes: `Repo::build()` num tempdir, um
`git::GitClient` nele, `Workbench::new(1280.0, 800.0)`, `connect`, e os
eventos do worker aplicados até o primeiro `Branches`.

### o rerun

cada request gravado, inteiro (ator, alvo, ação, parâmetro, `observed`),
passou por `Workbench::dispatch` na ordem gravada, e o worker foi drenado
entre um e outro: os eventos aplicados até o receipt terminar, depois um
`Workbench::refresh()` como barreira e os eventos aplicados até chegar cada
`Branches` devido (um por resposta de mutação, mais o da barreira; o worker
responde na ordem em que recebe). um teste descartável, fora do commit.

resultado: os nove vereditos iguais, e o caminho inteiro de estágios igual
em cada request (`requested, accepted, executing, completed` em 1 a 6, 8 e
9; `requested, rejected` em 7). a recusa do 7 com o mesmo `read` mudado e os
mesmos itens (`changes:staged`, removido `src/session.rs`); o commit do 9
com os mesmos três arquivos. a mesma sonda nas outras sessões da terceira
rodada: `a1pp` (7 requests, o submit 6 recusado stale em
`commit.message:text`), `a2ppc` (8) e `a2ppb` (7, o submit 7 recusado stale
em `changes:staged`), todos os vereditos iguais.

### o que difere sem mexer em veredito

- a revisão. o único nó diferente no estado de partida é
  `stacks.commit[<sha>]` do commit inicial (`0d3be3cc...` gravado,
  `a17675e2...` no rerun): o `git commit` do `Repo::build` carimba a hora do
  relógio, e o sha muda a cada montagem. por isso toda revisão difere
  (`0e3579d7ca054a45` gravada no início, `8790fa32701b4f56` no rerun). o
  `check` nunca compara revisão, só os `reads`.
- o `acted` da recusa do 7. gravado: a pessoa, request 6 (a porta sabia do
  olhar do agente). no rerun: ninguém, porque nenhum sink sabe do olhar e,
  sem ele, a regra parte de um receipt validado na revisão observada, que o
  rerun nunca tem. o harness não compara o `acted`; compara os `reads`
  mudados, como o briefing pede.
- a sobreposição do 6 com o 7. na gravação o 6 ainda executava quando o 7
  chegou; na sonda o worker foi drenado entre eles, e o 6 já tinha
  terminado. o veredito é o mesmo porque a view mostra o unstage assim que
  ele sai para o worker.
- os tempos: o workbench carimba os receipts no próprio relógio (`Instant`),
  e o sha do commit do 9 (`85f2f632...` no rerun) carrega a hora do git.
  nenhum dos dois é comparado.

nenhum veredito divergiu, e reaplicar não tocou nada fora do tempdir nem a
rede: o worker roda `git` no repositório temporário. a implementação segue.

## fase c: handoff (2026-09-24)

```text
Goal: task-64 fase c, um trecho de sessão gravada vira teste de regressão no gate, e a CLI exporta um trecho como bug report
Status: pronto para revisão (branch task/64c-session-to-test, rebaseada na main @ 51394a9; o briefing partiu de 41449e1)
```

### commits

- `edf6420` task-64c: the probe, a real session's requests rerun on a fresh laboratory repository
- `ed82720` task-64c: the session-to-test harness and a toy fixture
- `bb0a99f` task-64c: the ide's fixture and a test generated from a real session
- `901710c` task-64c: plev-inspect --export, a stretch of a session as a bug report
- este: task-64c: the ADR and the handoff

todos assinados. a main andou durante a fase (a task-68, o crate `uir`, e
a abertura da task-69); a branch entrou nela sem conflito e foi rebaseada
antes do pedido de integração.

### a sonda

na seção "fase c: sonda", acima: os nove requests da a2ppa, reaplicados na
ordem gravada sobre um repositório novo do laboratório, deram os mesmos
vereditos e o mesmo caminho de estágios; a1pp, a2ppb e a2ppc também. o que
difere (revisão, pelo sha do commit inicial; `acted`; a sobreposição do
unstage da pessoa com o submit; relógios) não mexe em veredito.

### o harness

onde: `crates/ide/tests/rerun/`, um binário de teste (`cargo test -p ide
--test rerun`). pelo ADR da 63, não no `rec` (o crate não executa nada, o
`Session` não segura app, e o ADR evita qualquer api que receba um
dispatcher ou um app no lado da leitura), nem no
`inspect` (só lê; o teste que confere as fontes continua verde). um crate
novo pede `arc.toml` e README, fora desta fase, e uma fixture só não faz
reuso: quando o showcase tiver a dele, o harness vira crate de teste.

- `harness.rs`: o `Fixture` e o executor `rerun`.
- `marks.rs`: o formato das marcas.
- `report.rs`: o relatório, request por request e marca por marca.
- `toy.rs`: o harness contra uma fixture de brinquedo (6 testes).
- `lab.rs`: a fixture do ide e o teste gerado da a2ppa.

a interface que o app fornece:

```rust
pub trait Fixture {
    fn dispatch(&mut self, req: Request, t: Millis) -> RequestId;
    fn confirm(&mut self, id: RequestId, by: Actor, t: Millis) -> Result<(), String>;
    fn decline(&mut self, id: RequestId, by: Actor, t: Millis) -> Result<(), String>;
    fn settle(&mut self) -> Result<(), String>;
    fn project(&self) -> Result<Snapshot, String>;
    fn receipt(&self, id: RequestId) -> Option<Receipt>;
}

pub fn rerun<F: Fixture>(
    excerpt: &Session,
    marks: &[Mark],
    start: impl FnOnce() -> Result<F, String>,
) -> Result<Report, String>;
```

`start` é o "partir do estado inicial num ambiente isolado"; `&mut F`
também é fixture, para o teste olhar a fixture depois. as regras do rerun:

- cada ato gravado é refeito na ordem: o `requested` vira um dispatch do
  request exatamente como foi gravado (ator, alvo, ação, parâmetro e os
  valores observados, que são a precondição); `confirmed` e `declined` de
  uma pessoa viram a mesma resposta. os outros estágios são a resposta do
  app, produzida pelo rerun, nunca reproduzida da gravação.
- antes de cada ato o app assenta (`settle`), a menos que a gravação
  tivesse uma execução ainda rodando naquele ponto: aí o ato entra com ela
  em voo, como entrou. isso diverge do "drenando o worker entre eles" do
  briefing de propósito, só onde a gravação mostra sobreposição: na a2ppa,
  o unstage da pessoa (6) ainda executava quando o submit (7) foi recusado,
  e o rerun reproduz isso. o teste do brinquedo confere a ordem das
  chamadas (nenhum `settle` no meio da sobreposição) e que o commit lento
  leva só o arquivo que levou na gravação; assentando no meio, o unstage
  chegaria depois do commit e ele levaria os dois.
- um request que o trecho tem sem o primeiro estágio (começou antes do
  corte) recusa o rerun antes de a fixture ser montada.
- no fim, veredito (o estágio depois de `requested`) e estágio terminal de
  cada request contra os gravados: mesmo estágio, numa recusa o mesmo tipo,
  numa recusa stale os mesmos `reads` mudados. um request aberto no fim do
  trecho não tem terminal a comparar. não compara tempo, revisão, `acted`
  nem valor de outcome (o sha do commit); uma marca confere valor de
  outcome quando importa.

a fixture do ide (`lab.rs`) é a mesma partida do laboratório
(`Bench::open`): `ide::lab_repo::Repo::build()` num tempdir, um
`git::GitClient` real nele, `Workbench::new(1280.0, 800.0)`, `connect` e
os eventos aplicados até carregar. o `settle` pede um `refresh` como
barreira e aplica os eventos do worker até chegar cada `Branches` devido
(um por refresh pedido, mais um por mutação respondida, que o
`GitSession::apply` sempre segue de um refresh). o worker só roda `git`
no tempdir; nada de rede.

o `Repo::build` saiu do bin sem duplicar: `git mv` de
`src/bin/lab/repo.rs` para `src/lab_repo.rs` (conteúdo idêntico), `pub mod
lab_repo;` no `lib.rs` (com uma linha de doc), e o bin do laboratório
troca `mod repo;` por `use ide::lab_repo as repo;`, então todo
`crate::repo::...` dele continua valendo. o teste do repositório só mudou
de alvo (do bin `ide-lab` para a lib), a contagem não muda. o `tests/lab.rs`,
que roda o `ide-lab` inteiro, continua verde.

### o formato das asserções

um arquivo de texto ao lado do trecho (`<nome>.marks`), uma marca por
linha, `#` comenta. só o que foi marcado é conferido, nunca o estado
inteiro:

```text
<âncora>  <o quê> = <valor>

âncora  after request <id> [origin <n>]   quando o request acabou na gravação
        at <ms> | at +<s>s                tempo desde o começo do trecho, como o plev-inspect imprime e o --at recebe
o quê   <id>:<chave>                      um valor da projeção do app
        outcome.<chave>                   um valor com que o request da âncora terminou (só com after request)
        stale                             os reads que a recusa stale do request nomeia, como conjunto (só com after request)
valor   true | false | 12 | 0.5 | "texto" | ["a", "b"] (lista) | {"a", "b"} (conjunto) | absent
```

a marca é lida logo antes do próximo ato que a gravação tem, ou no fim,
com o app assentado (menos no meio de uma sobreposição). linha que não lê
é erro com o número; marca num request que o trecho não tem recusa o
rerun. o `" = "` primeiro da linha separa o quê do valor.

### o teste gerado e a prova de que pega regressão

`crates/ide/tests/fixtures/sessions/a2ppa.monster` é a sessão da terceira
rodada exportada do começo até o commit e a projeção que ele deixa
(`plev-inspect target/agents/exp3/a2ppa/session.monster --export 0 110688
a2ppa.monster`; 184042 bytes, 65 eventos, 12 frames; o sha256 da sessão de
origem, `f07f34fc...`, igual antes e depois). `a2ppa.marks` tem seis
marcas: o estado de partida, a recusa stale do primeiro submit em
`changes:staged` com o staged e a mensagem depois dela, e o commit com os
três arquivos da tarefa e o staged vazio.

verde no gate:

```text
rerun of an excerpt recorded by rec 0.1.0, built from 96c20a0: 9 requests, 6 marks
  same      request 1 (origin 0, rerun as 1): unstage on changes.file[README.md] by agent:claude-a2ppa: completed
  same      request 2 (origin 0, rerun as 2): stage on changes.file[src/session.rs] by agent:claude-a2ppa: completed
  same      request 3 (origin 0, rerun as 3): stage on changes.file[tests/login_test.rs] by agent:claude-a2ppa: completed
  same      request 4 (origin 0, rerun as 4): open on commit.form by agent:claude-a2ppa: completed
  same      request 5 (origin 0, rerun as 5): set_value on commit.message by agent:claude-a2ppa: completed
  same      request 6 (origin 0, rerun as 6): unstage on changes.file[src/session.rs] by human: completed
  same      request 7 (origin 0, rerun as 7): submit on commit.submit by agent:claude-a2ppa: rejected: stale, changes:staged (removed src/session.rs)
  same      request 8 (origin 0, rerun as 8): stage on changes.file[src/session.rs] by agent:claude-a2ppa: completed
  same      request 9 (origin 0, rerun as 9): submit on commit.submit by agent:claude-a2ppa: completed
  held      line 11: at 0  changes:staged = {"README.md", "src/login.rs"}
  held      line 15: after request 7  stale = {"changes:staged"}
  held      line 16: after request 7  changes:staged = {"src/login.rs", "tests/login_test.rs"}
  held      line 17: after request 7  commit.message:text = "fix login session timeout"
  held      line 21: after request 9  outcome.files = ["src/login.rs", "src/session.rs", "tests/login_test.rs"]
  held      line 22: after request 9  changes:staged = {}
nothing diverged
```

o teste também confere, no repositório temporário, que o `HEAD` do rerun
é o commit "fix login session timeout" com os três arquivos.

a prova: `.reads(format!("{}:staged", ids.changes))` comentada em
`crates/ide/src/workbench/project.rs` (a precondição do `commit.submit`
quebrada de propósito), `cargo test -p ide --test rerun -- lab`, e o
arquivo restaurado com `git checkout` logo depois (nenhum diff no
workbench, nada disso commitado). a saída:

```text
test lab::the_a2ppa_session_reruns_as_it_was_recorded ... FAILED
---- lab::the_a2ppa_session_reruns_as_it_was_recorded stdout ----
thread 'lab::the_a2ppa_session_reruns_as_it_was_recorded' panicked at crates/ide/tests/rerun/lab.rs:171:5:
rerun of an excerpt recorded by rec 0.1.0, built from 96c20a0: 9 requests, 6 marks
  same      request 1 (origin 0, rerun as 1): unstage on changes.file[README.md] by agent:claude-a2ppa: completed
  same      request 2 (origin 0, rerun as 2): stage on changes.file[src/session.rs] by agent:claude-a2ppa: completed
  same      request 3 (origin 0, rerun as 3): stage on changes.file[tests/login_test.rs] by agent:claude-a2ppa: completed
  same      request 4 (origin 0, rerun as 4): open on commit.form by agent:claude-a2ppa: completed
  same      request 5 (origin 0, rerun as 5): set_value on commit.message by agent:claude-a2ppa: completed
  same      request 6 (origin 0, rerun as 6): unstage on changes.file[src/session.rs] by human: completed
  DIVERGED  request 7 (origin 0, rerun as 7): submit on commit.submit by agent:claude-a2ppa
            verdict   recorded  rejected: stale, changes:staged (removed src/session.rs)
                      rerun     accepted  <- differs
            terminal  recorded  rejected: stale, changes:staged (removed src/session.rs)
                      rerun     rejected: stale, changes:staged (removed src/session.rs)
  same      request 8 (origin 0, rerun as 8): stage on changes.file[src/session.rs] by agent:claude-a2ppa: completed
  same      request 9 (origin 0, rerun as 9): submit on commit.submit by agent:claude-a2ppa: completed
  held      line 11: at 0  changes:staged = {"README.md", "src/login.rs"}
  held      line 15: after request 7  stale = {"changes:staged"}
  held      line 16: after request 7  changes:staged = {"src/login.rs", "tests/login_test.rs"}
  held      line 17: after request 7  commit.message:text = "fix login session timeout"
  held      line 21: after request 9  outcome.files = ["src/login.rs", "src/session.rs", "tests/login_test.rs"]
  held      line 22: after request 9  changes:staged = {}
the rerun diverged: 1 request(s), 0 mark(s)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.38s
```

o veredito do 7 diverge (aceito no check em vez de recusado stale) e o
teste falha dizendo onde. o terminal fica igual porque o ponto de verdade
ainda segura: o `CommitIf` do worker confere o índice real e recusa com o
mesmo delta. ou seja, com a regra do app quebrada o índice continua
protegido, e o harness pega a regressão pelo veredito, não pelo
resultado. no gate, o teste do brinquedo
(`a_broken_precondition_diverges_and_the_report_says_where`) guarda o
mesmo caso de forma automática, com o texto da mensagem conferido:

```text
  DIVERGED  request 2 (origin 0, rerun as 2): submit on commit.submit by agent:toy
            verdict   recorded  rejected: stale, changes:staged (removed B)
                      rerun     accepted  <- differs
            terminal  recorded  rejected: stale, changes:staged (removed B)
                      rerun     completed  <- differs
  ...
  DIVERGED  line 4: after request 2  stale = {"changes:staged"}
            found     absent
  DIVERGED  line 5: after request 3  outcome.files = ["A", "C"]
            found     the request did not complete in the rerun: rejected: disabled
the rerun diverged: 2 request(s), 2 mark(s)
```

### o export

`plev-inspect <sessão> --export <t0> <t1> <saída>`: t0 e t1 em ms desde o
começo da sessão (o mesmo tempo do `--at`), a saída pelo
`Session::excerpt` do `rec` (do keyframe de cada trilha que governa t0 até
t1, chunks copiados como estão, só os assets que eles usam). mantive a
forma do briefing. vai sozinho (com `--timeline`, `--why`, `--control` ou
`--json` é exit 2), nunca por cima do arquivo que lê (exit 2), e uma
escrita que falha é exit 4, novo, no `--help`. a lib do `inspect` não
mudou; só o bin.

testes (`crates/inspect/tests/model/export.rs`): uma sessão com frames e
ligações de um compositor e keyframes frequentes; o trecho do meio reabre
abrindo nos keyframes das duas trilhas antes de t0, com os mesmos eventos
em [t0, t1], o mesmo estado, frame, tempo de frame e ligações a cada 10 ms,
e os mesmos requests inteiros (a recusa stale e o commit); o arquivo de
origem com os mesmos bytes; o resumo do trecho pelo próprio
`plev-inspect`. e os erros: t1 antes de t0, junto com `--why`, por cima do
arquivo lido (sem escrever nada), diretório que não existe (exit 4),
sessão que não lê (exit 3). o parse ganhou os casos no teste do bin.

### validação

- `cargo test -p inspect -p rec -p ide`: verde.
- `./script/gate` com a perna urnaui (o `worktrees/urna` aponta para o
  checkout irmão), antes e depois, no target da task:
  - antes (a main, com o `git mv` já feito, que não muda contagem): 2277
    passaram (2195 do workspace, 82 da urnaui), 0 falharam, 12 ignorados.
  - depois (os quatro commits de código, ainda sobre 41449e1): 2288
    passaram (2206 do workspace, 82 da urnaui), 0 falharam, 12 ignorados.
  - os 11 a mais: 9 do `rerun` (2 do formato das marcas, 6 do brinquedo, 1
    da a2ppa) e 2 do export.
  - rebaseada na main @ 51394a9: 2349 passaram (2267 do workspace, 82 da
    urnaui), 0 falharam, 12 ignorados, com os 9 do `rerun` e os 35 do
    modelo do `inspect` (eram 33) dentro. a main sozinha, por diferença e
    sem rodar à parte, fica em 2338.
- cada commit de código conferido sozinho (o resto guardado em stash):
  clippy `-D warnings`, fmt e os testes do crate.
- o guard das 639 linhas passa: o bin do `plev-inspect` tem 440; os
  arquivos do harness são de teste.

### arquivos fora do óbvio

- `crates/ide/Cargo.toml`: só o comentário do `[dependencies]` (dizia que
  a lib não usa o `tempfile`; agora usa, pelo `lab_repo`). o briefing
  permitia esse arquivo para dev-dependencies; nenhuma dependência mudou.
- `crates/inspect/Cargo.toml`: `engine` como dev-dependency, para o teste
  do export gravar frames com ligações de um compositor (o `rec` já
  depende do `engine`); no `Cargo.lock`, essa linha.
- nada em `sem`, `wire`, `workbench`, `views`, `rec`, no formato `MONS`, na
  lib do `inspect`, no `arc.toml`, no README ou no índice de ADRs.

### linha proposta para o índice de ADRs

```text
| 2026-09-24 | [sessions-become-regression-tests](sessions-become-regression-tests.md) | proposed | um trecho de sessão gravada vira teste de regressão: o harness (crates/ide/tests/rerun, reexecução controlada, só em teste; nunca no rec nem no inspect) refaz cada ato do trecho na ordem gravada contra uma fixture do app (dispatch, confirm, decline, settle, project, receipt), assenta antes de cada ato menos onde a gravação tinha uma execução em voo, e compara veredito e estágio terminal (numa recusa stale, os reads mudados) e as marcas do desenvolvedor num arquivo de texto ao lado; a fixture do ide sobe o repositório do laboratório num tempdir com o worker do git real; plev-inspect --export grava um trecho como bug report (task-64 fase c) |
```

### para a integração (o que é do orquestrador)

- `arc.toml`, entrada do ide: o repositório do laboratório mora na lib
  (`lab_repo`), e `crates/ide/tests/rerun` guarda o harness e as sessões
  que viraram teste.
- README: a linha do `plev-inspect` pode ganhar o `--export`.
- manual: nenhum `docs/how-to` trata do inspector; se um passar a tratar,
  o formato das marcas está no ADR.

### limitações

- relógio: o workbench carimba no próprio `Instant` e não recebe relógio;
  injetar pede mexer no workbench, proibido aqui. nada que o harness
  compara lê esse relógio, e os timers dos toasts só andam pelo
  `Workbench::wake`, que a fixture nunca chama. na fixture de brinquedo o
  relógio é o do harness.
- a prova de regressão no ide foi à mão, porque a regra mora no workbench;
  no gate, quem guarda é o brinquedo.
- o que fica frágil está no ADR: ids que carregam valor da execução
  (`stacks.commit[<sha>]`, `notices[<n>]`) viram `not_found` no rerun; o
  estado de partida da fixture tem de ser o do começo do trecho (o harness
  recusa um request cortado, mas não confere o estado); o `settle` do ide
  depende de cada mutação respondida pedir um refresh; parâmetro secreto
  volta como o comprimento; sessão com vários dispatchers passa pelo único
  da fixture; frames não são comparados; o `git` lê a configuração global
  do usuário (a local do repositório fixa identidade e desliga
  assinatura), sem escrever fora do tempdir.
- `acted` e revisão ficam de fora da comparação: a revisão muda com o sha
  do commit inicial (o `Repo::build` carimba a hora), e o olhar do agente
  não está na gravação. fixar a data do commit inicial no `Repo::build`
  deixaria o estado de partida com a mesma revisão numa gravação futura;
  não mudei porque as sessões já gravadas têm o sha antigo.

### o que falta para o showcase

- o showcase não grava sessão: não depende do `rec` (a fase d da task-63
  ligou só o ide). sem sessão da página do todo não há trecho para
  rerodar.
- a fixture da página do todo: o dispatcher da página (os cliques já
  passam pelo `sem`) com o relógio do harness, um `settle` que avance o que
  a página anima, e o `project` dela.
- com a segunda fixture, o harness sai de `crates/ide/tests/rerun` para um
  crate de teste (com `arc.toml` e README).

### o que falta para a fase b

- marcar na UI e escrever o arquivo de marcas neste formato (o `marks.rs`
  tem o `render` que escreve um valor de volta).
- exportar da UI pelo mesmo `Session::excerpt` que a CLI usa; o modelo não
  precisa de nada novo para isso.
- mostrar o relatório do rerun ao lado da cadeia causal, quando a fase b
  quiser; hoje ele é texto de teste.

## fechamento da fase c (orquestrador, 2026-09-24)

integrada na `main` em `d611fa4` (cinco commits, rebaseados sem
conflito). o ADR sessions-become-regression-tests foi aceito.

validação do orquestrador:
- diff lido (o harness, a fixture do ide, as marcas, o export, o
  `lab_repo` movido para a lib sem mudar o arquivo);
- gate verde no diretório da task: 2349 passaram, 0 falharam, 12
  ignorados, com a perna urnaui;
- o teste gerado da sessão a2ppa rodou dez vezes seguidas, verde nas dez;
- a prova de que ele pega regressão foi repetida pelo orquestrador: com
  o `changes:staged` tirado dos `reads` do `submit` em `project.rs`, o
  teste falhou com "DIVERGED request 7 ... recorded rejected: stale,
  changes:staged ... rerun accepted", e o arquivo foi restaurado.

severidade do que ficou:
- P3: o `lab_repo` é módulo público da lib do ide (código do
  laboratório no caminho de produção); o relógio do workbench não é
  injetado (nada que o harness compara lê esse relógio);
- P2: o showcase não grava sessão nem tem fixture; com a segunda fixture
  o harness vira crate próprio; a fase b (UI) marca valores e escreve o
  `.marks`.
