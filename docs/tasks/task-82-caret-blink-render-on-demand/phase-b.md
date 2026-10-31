---
id: task-82/phase-b
title: "task-82 phase b: ide, Git page and urnaui draw only when something changes"
project: plev
type: section
status: reference
parent: task-82
date: 2026-09-23
date_basis: text
topics: [rendering, performance, input]
relations: [task-85, task-87]
source: docs/log/steps/task-82-caret-blink-render-on-demand.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
evidence: [a6fec8d, 890f454, 1e85275, 0d59815, fccb310, 5647f57, 026f9ea, 473c3de, a11a1c0, d3201ef, 31bf7b4, 8a61778]
---

# phase b: ide, Git page and urnaui draw only when something changes

Records the second phase: the `Wake` reaches the ide, the showcase's Git page and urnaui, the toast becomes a deadline, and git runs without optional locks, so that the idle ide stops firing `git status` in a loop. In a real window, the commit field focused in the ide went from 720 frames in 6 s to 11, one per blink transition, and the idle, unfocused ide went from 34 frames and 17 `git` processes to zero. The merge into `main` is `a6fec8d`, with the green gate (2128 tests passing, 12 ignored).

## fase b aberta (orquestrador, 2026-09-23)

a auditoria de verdade (`truth.md`, seções 4g e 5.1)
mostrou que a promessa do README ("no change, no frame") está quebrada no
app da prova da tese: o binário `ide` desenha todo frame com o campo de
commit focado ou com um toast na tela, a página Git do showcase volta a 60
fps pelo mesmo caminho, e o worker do git faz o showcase desenhar mesmo
com outra página visível. é invariante quebrada, primeira na ordem do
ciclo autônomo. escopo da fase b: o `Wake` no ide (cadeia e shell), na
página Git, no urnaui, o toast como prazo e o frame avulso do worker no
showcase; medição antes e depois numa janela real. fora: o blink sem foco
do `EditorView` e o caret do dock (demo), que ficam na lista de dívidas.
branch `task/82b-wake-everywhere`.


## fase b (2026-09-23, agente, branch `task/82b-wake-everywhere`)

feito, em dez commits de código que compilam sozinhos, sobre a `main`
em `64e10a1`:

- comps: `ToastManager::wake(dt) -> Wake` no lugar do `tick -> bool`.
  enquanto uma mola de entrada ou saída anda, `Wake::continuous()`; com
  todos parados, `Wake::after` do tempo até o primeiro toast visível
  sumir (`duration - age`); sem toast, `idle`. o frame que deixa de
  desenhar um toast é `now`. uma duração infinita ou NaN não dá prazo,
  e um `dt` que não é tempo conta como zero. o tick conta todo o tempo
  que recebe: quem aplica uma mudança alcança o tempo antes.
- ide: `CommitForm::wake`, `wake` nos três painéis (o fade do scrollbar
  por `Wake::moving`), `WorkspaceView::wake` junta painéis, formulário e
  toasts, `Workbench::wake` repassa. o shell (`apps/idestudio/src/main.rs`)
  é o do showcase: `since_last`, `pace` com `WaitUntil`/`Wait`,
  `new_events` com `ResumeTimeReached` e a linha de debug por frame.
  `Workbench::tick -> bool` fica, com o sentido "ainda em movimento",
  porque o settle dos testes do workbench (`workbench/tests.rs`, fora
  desta fase) lê.
- showcase: `GitPage::wake` é o pedido do workbench; os eventos do
  worker entram por `GitPage::apply_events`, que o host chama.
  `ShowcaseView::wake` ticka a página Git em qualquer página e só junta
  o pedido dela com a página na tela; `ShowcaseView::apply_git` aplica
  os eventos e diz se a mudança está na tela, e `UserEvent::Wake` usa
  isso. os toasts do próprio gallery também respondem com o `Wake`
  deles.
- urnaui: `Field::wake` e o `wake` das telas Open, Search, Chunks, Graph
  e Stats, do `Explorer` e do `UrnauiView`; o shell com o mesmo `pace` e
  `new_events` (`web_time::Instant`, o build wasm passa).
- `TextField::tick` e `ToastManager::tick` saíram quando o último
  chamador saiu. nenhum `tick -> bool` novo.
- git: todo processo `git` do `GitRepo` nasce num helper só
  (`GitRepo::command`), com `GIT_OPTIONAL_LOCKS=0` no ambiente: leitura
  não escreve nada no `.git`, mutação continua com os locks dela.
- ide: o watcher não dispara refresh para evento cujo único caminho é
  `.git/index.lock` (`is_change`), com o motivo no doc.
- os três shells alcançam o tempo no começo de cada volta do loop:
  `new_events` chama `catch_up`, que ticka a view com o `since_last`
  real antes de qualquer evento daquela volta (entrada, resultado do
  worker, pedido da bridge ou do agente, e o dreno do urnaui no
  `about_to_wait`). o tick de alcance não desenha; se mudou algo visível
  (um prazo que venceu na hora), pede o frame, e o `now` dele vai junto
  no `wake` do frame (`caught_up`), para a linha de debug marcar a
  virada. no showcase o `pace`, o `catch_up` e o `frame_wake` foram para
  `app/pace.rs` (o `app.rs` passou das 639 linhas), e `App::with_view`
  monta o shell sem os argumentos de lançamento, para os testes.
- a regra do "fresh" (o `Toast` e o `TextInput` contavam zero no tick
  depois de uma mudança) entrou nesta fase e saiu nesta fase: o alcance
  nos shells cobre todo widget, e nenhum caso ficou só com ela (nada
  empurra toast nem reinicia caret dentro de um tick; a página Git
  aplicava eventos do worker dentro do tick e deixou de aplicar). o
  `TextInput` voltou ao código e aos testes da fase a. quem dirige
  widgets sem shell (testes, headless, o settle do rec, o exemplo
  `measure`) alcança o tempo por conta própria antes de uma mudança.

### fase b: medição

método: build de debug (`cargo build -p idestudio -p uigallery`), numa janela
real. "antes" é a base `83967a0` com uma linha de debug por frame
posta no `main.rs` do ide (ele não tinha) e uma linha `git worker woke`
no `user_event` do showcase, compilada no mesmo target dir e copiada;
"depois" é esta branch. o app abre num repositório de rascunho (um
commit, um arquivo modificado, um não rastreado). as teclas são
CGEvent postados no pid do app depois de ativá-lo; os frames são as
linhas de log por frame, carimbadas na chegada pelo harness; os
processos git, as linhas que um `git` de contagem no PATH escreve antes
de chamar o git de verdade. o harness espera 4 s sem entrada de hardware
antes de abrir e marca a rodada se alguém mexer no mouse ou teclado
durante a janela (`CGEventSource.secondsSinceLastEventType`). em (d) um
`git` de mentira no PATH dorme 2 s e chama o git de verdade, para os
resultados chegarem com a janela parada. a taxa de display variou entre
rodadas (120 ou 240 por segundo), então os números "antes" contínuos são
a taxa do display, não do app.

rodada 2, o código final, sem `GIT_OPTIONAL_LOCKS` no ambiente, todas as
rodadas limpas:

| cenário | antes | depois |
| --- | --- | --- |
| ide parado e sem foco, 6 s | 34 frames, 17 processos `git status` | 0 frames, 0 processos |
| a. ide, campo de commit focado (tecla `c`), 6 s parado | 720 frames, todo frame; 18 processos | 11 frames, um por virada; 0 processos |
| b. ide, toast de recusa (`c`, Enter vazio, Esc), 6 s a partir do Esc | 653 frames, todo frame até o toast sair; 17 processos | 381 frames: a mola de entrada (~1,7 s), nada nos ~2,3 s de repouso, o frame do prazo, a mola de saída (~1,7 s), depois nada; 0 processos |
| c. showcase na página Git, campo focado (`c`), 6 s parado | 720 frames, todo frame | 11 frames, um por virada |
| d. showcase na página Button, worker do git acordando 3 vezes, 8 s desde a abertura | 2 frames sem mudança pelas 3 acordadas, 4 com os 2 da abertura | 0 frames pelas acordadas, 2 no total (os da abertura) |

rodada 1, antes do alcance e do conserto do lock, com
`GIT_OPTIONAL_LOCKS=0` para isolar o caret e o toast do loop do watcher
(as rodadas "antes" com o campo ou o toast foram marcadas por entrada
de hardware, e já desenhavam todo frame): a 1431 -> 11, b 653 -> 381,
c 1423 -> 11, d 3 frames sem mudança -> 0. o ide parado sem a variável
desenhava 36 frames em 6 s antes e 37 depois (o loop, que a rodada 1
não tocava).

conferências a mais, com o código final: no ide, cinco teclas depois do
`c` desenham um frame cada, e cada frame de tecla pede a próxima virada
em ~0,529 s (o meio-período recomeça na tecla, menos os milissegundos
até o frame); as viradas do campo focado saem com `now true` e
`since_last` 0,000 (o alcance fez a virada e o frame a mostra); o urnaui
parado desenha 0 frames em 6 s depois dos dois da abertura; com a página
Git na tela, 3 acordadas do worker desenham 2 frames.

testes com relógio injetado, todos na ordem do shell (alcança o tempo
parado, aplica o evento, desenha o frame alguns milissegundos depois;
depois um turno de 60 Hz só é desenhado se o pedido anterior era
contínuo ou o prazo venceu, e recebe o tempo desde o último turno
desenhado):

- git (`rust/gitdrv/tests/real_repo.rs`): com o stat do index velho, o
  `status` deixa o `.git/index` como achou (tempo e bytes), e um
  `git status` puro, o controle, reescreve. sem a variável o teste
  falha.
- ide, watcher (`apps/idestudio/src/repository/watcher.rs`): o predicado em eventos
  montados, e o watcher de verdade num diretório temporário, onde mexer
  no lock não dispara e escrever na worktree dispara. sem o filtro o
  segundo falha.
- ide, shell (`apps/idestudio/src/main.rs`): a recusa depois de 10 s parado
  vive os 4 s inteiros; o caret focado depois de 10 s parado começa
  sólido e vira 0,53 s depois da tecla, 11 viradas em 6 s.
- showcase, shell (`apps/uigallery/src/shell/window/pace/tests.rs`): o toast
  empurrado depois de 10 s parado vive a duração inteira e entra do
  começo; o caret focado na página Git depois de 10 s parado começa
  sólido, 11 viradas, a primeira no turno 32; um alcance que não muda
  nada não pede frame.
- showcase, `Switch` (`apps/uigallery/src/views/compositor/render.rs`): virado
  depois de 3 s parado, o knob anima (mais de 10 frames) com o alcance,
  e sem o alcance chega ao fim no primeiro frame.
- showcase, página Git (`apps/uigallery/src/views/apps/git/desktop/tests.rs`):
  o campo focado pede 11 frames, todos viradas; o toast de recusa anda,
  descansa e sai no prazo; eventos do worker esperam o host aplicar; o
  gallery pede o caret do git só com a página na tela; eventos do worker
  são aplicados com a página escondida e só pedem frame com ela na tela.
- ide, workspace (`apps/idestudio/src/interfaces/panels/layout/tests/wake.rs`): o
  campo focado pede 11 frames em 6 s, todos viradas; o formulário
  escondido não pede nada; o toast pede o instante em que sai e nada
  depois; toast e campo juntos pedem o menor prazo.
- comps (`rust/engine/src/widgets/tests/wake.rs`): o toast anda, descansa num
  prazo e sai; o tick conta todo o tempo que recebe (um push avançado
  10 s de uma vez já está saindo, alcançado antes vive a duração
  inteira); o clique anima a saída; sem toast ou com duração infinita,
  `idle`.
- urnaui: a busca focada pede 11 frames, todos viradas; o toast do view
  anda, descansa e sai no prazo.

### fase b: handoff

```text
Goal: task-82 fase b, o ide, a página Git do showcase e o urnaui só
      desenham quando algo muda
Status: pronto para validação do orquestrador

Implementado:
- comps: ToastManager::wake (toast como prazo); as pontes
  TextField::tick e ToastManager::tick removidas
- ide: CommitForm, painéis, WorkspaceView e Workbench com wake; shell
  com since_last, WaitUntil/Wait, ResumeTimeReached e o alcance
- showcase: GitPage::wake e apply_events, ShowcaseView::apply_git,
  UserEvent::Wake só desenha com a página Git na tela; o alcance
- urnaui: os oito tick -> bool viraram wake -> Wake; shell com
  WaitUntil e o alcance
- git: GIT_OPTIONAL_LOCKS=0 em todo processo, num helper só
- ide: o watcher ignora .git/index.lock sozinho

Arquivos principais:
- rust/engine/src/widgets/feedback/toast.rs, rust/engine/src/widgets/form/text_field.rs
- rust/gitdrv/src/repo.rs, rust/gitdrv/tests/real_repo.rs
- apps/idestudio/src/main.rs, apps/idestudio/src/repository/watcher.rs,
  apps/idestudio/src/interfaces/panels/{commit_form,diff_view,multi_stack_view,
  unassigned_view}.rs, apps/idestudio/src/interfaces/panels/layout/mod.rs,
  apps/idestudio/src/capability/mod.rs (só tick/wake)
- apps/uigallery/src/shell/mod.rs, apps/uigallery/src/shell/window/pace/mod.rs (novo),
  apps/uigallery/src/views/compositor/render.rs,
  apps/uigallery/src/views/apps/git/{desktop,stub}.rs e o doc de
  apps/uigallery/src/views/apps/git/mod.rs
- apps/urnadbapp/src/app.rs, apps/urnadbapp/src/view/{mod,field,open,
  search,chunks,graph,stats,explorer}.rs

Interfaces adicionadas/alteradas:
- adicionadas: ToastManager::wake, CommitForm::wake, {Unassigned,
  MultiStack,Diff}View::wake, WorkspaceView::wake, Workbench::wake,
  GitPage::{wake, apply_events}, ShowcaseView::apply_git, o wake das
  views do urnaui; nos shells, catch_up (e no ide e no showcase advance
  e frame_wake, que os testes chamam)
- removidas: ToastManager::tick, TextField::tick, CommitForm::tick,
  WorkspaceView::tick, o tick dos três painéis, GitPage::tick e os oito
  tick do urnaui
- mudou de sentido: Workbench::tick agora é "ainda em movimento" (now ou
  contínuo); GitPage::wake não aplica mais os eventos do worker
- Wake, Blink e TextInput: sem mudança no fim da fase

Testes:
- 26 novos (comps 4, ide 8, showcase 11, git 1, urnaui 2), com relógio
  injetado onde há tempo; os de git e do watcher falham sem o conserto

Validação executada:
- cargo test -p gitdrv, -p engine, -p idestudio, -p uigallery verdes
- ./tool/tasks/fullcheck.sh verde sobre a main em 64e10a1, com a perna urnaui
  rodando (../urna presente)
- contagem: base (main 64e10a1) workspace 2022 passed, 0 failed, 11
  ignored (74 "test result"); urnaui 80 passed, 1 ignored. branch:
  workspace 2046 passed, 0 failed, 11 ignored (74); urnaui 82 passed,
  1 ignored. por crate: comps 366 -> 370, ide 80 -> 88, showcase
  135 -> 146 (3 ignorados nos dois), git 33 -> 34 (2 ignorados),
  urnaui 80 -> 82
- medição em janela real antes e depois, rodada 2 sem
  GIT_OPTIONAL_LOCKS (tabela acima)

Decisões:
- o alcance mora no new_events de cada shell, um gancho só em vez de
  espalhado por window_event, user_event e about_to_wait: toda volta do
  loop começa com o tempo em dia, e tudo que chega naquela volta cai
  depois dele
- a regra do "fresh" saiu: dois mecanismos para um problema; o alcance
  cobre todo widget
- a página Git não aplica eventos no tick: o host aplica depois do
  alcance
- o pacing do showcase foi para app/pace.rs pelo limite de 639 linhas
- o doc de apps/uigallery/src/views/apps/git/mod.rs dizia que os eventos
  eram drenados no próximo tick; mudei as três linhas

Limitações:
- a mola do toast leva ~1,7 s em cada sentido para repousar: a medição b
  caiu de 653 para 381, não mais
- a bridge de accessibility, o EditorView sem foco e o caret do dock
  continuam como na fase a

Dependências pendentes:
- nenhuma para integrar

Riscos de integração:
- o teste do watcher de verdade depende do tempo do FSEvents (drena os
  eventos repetidos da criação do diretório e espera 1,2 s de silêncio);
  passou em seis rodadas seguidas e no gate, mas é o único teste com
  tempo de sistema de arquivos que entrou
- o exemplo tool/bench/measure/src modela o shell sem o
  alcance; os números dele não mudam (são da fase a, antes do "fresh"),
  mas o modelo não repete mais o shell à risca (arquivo fora desta fase)
- o alcance roda em new_events também no wasm e no mobile: compilado
  (check wasm com -D warnings), não rodado num browser nem num aparelho
- apps/idestudio/src/capability/mod.rs: só tick/wake mudaram; o rebase sobre
  a task-85 e a task-87 foi limpo

Commits:
- 890f454 task-82b: comps: the toasts answer with a Wake (o "fresh" do toast saiu em 8a61778)
- 1e85275 task-82b: comps: the caret's half-period restarts at the key, not at the last frame (revertido em 8a61778)
- 0d59815 task-82b: ide: the workbench answers with a Wake and the shell sleeps until it is due
- fccb310 task-82b: showcase: the Git page asks with the workbench's Wake, and the git worker draws only on it
- 5647f57 task-82b: urnaui: the eight ticks answer with a Wake and the shell sleeps until it is due
- 026f9ea task-82b: comps: the bool bridges go, no caller is left
- 473c3de task-82b: measurement, debts and handoff (rodada 1)
- a11a1c0 task-82b: git: every git process runs without optional locks
- d3201ef task-82b: ide: the file watcher does not refresh for .git/index.lock alone
- 31bf7b4 task-82b: the shells catch time up before anything from outside the frame lands
- 8a61778 task-82b: comps: the toast and the text input lose their fresh flag, the shells' catch-up covers them
- o commit deste texto: rodada 2, dívidas, ADR e handoff
```

## fechamento da fase b (orquestrador, 2026-09-23)

integrada na `main` em `a6fec8d` (onze commits). validação: diff inteiro
lido nas duas rodadas; devolvida uma vez com dois achados do próprio
agente, da mesma responsabilidade: o laço do `index.lock` (o ide parado
nunca parava de desenhar num repositório comum) e as molas que pulavam o
intervalo antes de um evento (a flag `fresh` por widget virou o tick de
alcance nos shells, e a flag saiu). gate verde no diretório da task, com
a perna urnaui: 2128 passaram, 0 falharam, 12 ignorados. reproduzido pelo
orquestrador: o ide desta branch num repositório novo com o index velho,
sem `GIT_OPTIONAL_LOCKS` no ambiente, desenhou 2 frames na subida e
depois 0 frames e 0 processos `git` em uns 8,7 s parado (um `git status`
na subida, contado por um shim no PATH). fechamento: README e arc.toml
sem as exceções que a fase b fechou, o comentário do watcher com o número
certo (um `git status` por volta), changelog, programa, truth.

dívidas que ficam, sem task ainda (nenhuma quebra uma promessa com
usuário real hoje): o blink do `EditorView` sem foco; o caret do dock da
demo; a ponte de acessibilidade pedindo frame sem mudança visível; a mola
do toast levando ~1,7 s para repousar abaixo de um pixel; o exemplo
`measure` modelando o shell sem o tick de alcance.
