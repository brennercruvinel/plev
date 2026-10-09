---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-09-23
domain: engine
task: task-82
depends: [task-62]
branch: task/82-caret-wake
integrated-phase-a: d97a375
integrated-phase-b: a6fec8d
---

# task-82: o caret não pode pedir 60 frames por segundo

## objetivo

um campo de texto focado e parado desenha 60 frames por segundo para um
caret que muda de visibilidade umas duas vezes por segundo. a task-73 mediu
(cenário g: 120 de 120 turnos desenhados com o campo focado e nada
digitado). isso fere o contrato de render on demand: sem mudança visível,
sem frame.

## estado atual no código

- `TextField::tick` (`rust/engine/src/widgets/form/text_field.rs`) devolve `true`
  em todo frame enquanto está focado; o `TextInput` guarda a fase do blink.
- as páginas repassam isso: `TodoPage::tick` devolve
  `animating || self.input.is_focused()`
  (`apps/uigallery/src/views/apps/todo/mod.rs`), e o formulário de commit do ide
  faz o mesmo (`apps/idestudio/src/interfaces/panels/commit_form.rs`).
- com `ControlFlow::Wait`, um tick que devolve `false` deixa o loop dormir,
  e ninguém acorda para virar o caret: por isso o atalho de pedir frame
  sempre.

## escopo

- o blink vira um prazo: o tick diz se precisa de frame agora (a
  visibilidade virou) e quando precisa do próximo (o instante da próxima
  virada). o shell de cada app usa `ControlFlow::WaitUntil` com o menor
  prazo pendente, em vez de pedir frame contínuo.
- `TextField`, `TextInput`, o `EditorView` e os ticks das páginas e apps
  que repassam o caret (showcase, ide, urnaui) passam pelo mesmo mecanismo;
  um lugar só decide o próximo prazo.
- o `PerfHud` continua implicando frames contínuos (contrato
  `hud-implies-continuous-frames`).

## aceite

cenário g da `showcase --example measure` cai para 2 ou 3 frames por
segundo desenhados com o campo focado e parado; digitar continua
desenhando cada tecla; o caret continua piscando (teste com relógio
injetado conferindo as viradas); gate verde.

## fase a integrada (2026-09-23)

integrada na `main` em `d97a375` (oito commits, rebaseados pelo agente
sobre a 62c e a 66b e pelo orquestrador sobre um commit de docs, sem
conflito), validada pelo orquestrador: diff inteiro lido (o `Wake` e o
`Blink` no comps, `since_last` aditivo no engine, o shell do showcase em
`WaitUntil` com o retorno a `Wait` em `ResumeTimeReached`), gate verde no
diretório da task sobre a base `26d7e3d` (1985 passed, 0 failed, 6
ignored; a `main` tinha 1969); o rebase final só cruzou docs. ADR:
`docs/adr/rendering/0042-a-tick-asks-for-a-deadline.md`.

decisão do orquestrador registrada: opção 1 do agente (`since_last` no
`AnimationTick`, `dt` travado mantido); os dois literais em
`engine/src/window/mod.rs` e os testes do perf e do monster ganharam o
campo. aceitos: `Wake::after` no lugar de `in_`; `tick -> bool` preso onde
o ide, o exemplo do comps, o rec e o headless ainda leem.

a task fica em `ongoing/` pela fase b: o ide (`commit_form.rs`, `main.rs`)
e o urnaui adotam o `Wake` e o shell deles passa `since_last`, agora que
a 62c saiu da superfície do ide. dívidas herdadas: `EditorView` sem foco
pisca enquanto visível; caret do dock contínuo com o painel aberto; o
worker do git e a bridge de accessibility pedem frames avulsos.

## revisão do orquestrador (2026-09-23, antes do briefing)

- mecanismo: `TextInput::tick(dt)` (`rust/engine/src/widgets/form/text_input/component.rs:61-70`)
  acumula `cursor_blink_timer` e vira a visibilidade a cada
  `CURSOR_BLINK_INTERVAL` (0,53 s); `TextField::tick`
  (`rust/engine/src/widgets/form/text_field.rs:205`) devolve `true` enquanto
  focado; `EditorView` tem o mesmo relógio (`rust/engine/src/widgets/editor/view.rs:52,180`).
  `TodoPage::tick` devolve `animating || self.input.is_focused()`
  (`apps/uigallery/src/views/apps/todo/mod.rs:311-314`), e o shell do showcase
  pede frame sempre que o tick devolve `true`
  (`apps/uigallery/src/shell/mod.rs:394-417`), com `ControlFlow::Wait`.
- decisão: o tick passa a devolver um prazo, não um booleano de "continue":
  um tipo do comps (nome sugerido `Wake`) que diz se um frame é preciso
  agora e, opcionalmente, quando o próximo é preciso (`Option<Duration>`);
  a composição de vários widgets é o menor prazo. o shell traduz em
  `ControlFlow::WaitUntil(instante)` e só pede frame quando o prazo vence
  ou uma entrada chega. o `PerfHud` continua implicando frames contínuos.
- sequência por ownership: o ide (`apps/idestudio/**`) é da task-62 fase c em
  andamento; esta task NÃO toca o ide. o `commit_form.rs` e o shell do ide
  adotam o prazo numa fase b, depois da 62c integrar. o urnaui não compila
  sem `../urna`; fica anotado como pendente.

## estado (2026-09-23, agente, branch `task/82-caret-wake`)

feito, em seis commits que compilam sozinhos:

- `comps::Wake` (`rust/engine/src/widgets/wake.rs`, exportado na raiz e no
  prelude): `now: bool` (este tick mudou algo visível) e
  `next: Option<Duration>` (quando o próximo tick é preciso;
  `Some(ZERO)` é movimento contínuo, `None` é nada pendente).
  construtores `idle`, `now`, `after(d)`, `continuous`, `moving(bool)`
  (o `bool` legado elevado); `merge` (qualquer `now`, o menor `next`);
  `pending` (`now || next.is_some()`) e `is_continuous`. o construtor
  do prazo chama `after` e não `in_`: `in` é keyword e o escape lê mal.
- `comps::wake::Blink`, o relógio do caret que os três widgets
  dividem: fase dentro do meio-período, `advance(dt, interval) -> Wake`,
  `reset`. a conta roda em f64 (um stall longo mantém o resto exato);
  um número par de viradas num dt só não muda nada visível (`now`
  falso); `dt` ou `interval` que não é tempo (negativo, NaN, infinito)
  é ignorado em vez de dar panic ou laço infinito. `interval` não
  positivo quer dizer sem blink: caret sólido, nada pendente.
- `TextInput::wake`, `TextField::wake`, `EditorView::wake` devolvem o
  prazo. `focus` e `reset_blink` reiniciam o meio-período. o `bool`
  fica onde um consumidor que esta task não toca lê: `TextField::tick`
  é `pending()` (verdadeiro enquanto focado) para o shell do ide até a
  fase b; `EditorView::tick` é `now` (virou) para o `comps --example
  editor`, que tem o próprio timer.
- showcase: `TodoPage::wake`, `EditorPage::wake`, `InputPage::wake`,
  `ShowcaseView::page_wake` e `ShowcaseView::wake` compõem os filhos
  (movimento contínuo dos outros widgets entra por `Wake::moving`).
  `ShowcaseView::tick -> bool` fica para os laços de settle que leem
  bool (o `render_page` headless, o `rec/tests/session/measure.rs`):
  "ainda em movimento" (`now || is_continuous()`); um caret parado
  entre viradas está assentado. antes ele devolvia verdadeiro para
  sempre com o campo focado.
- o harness do `measure` modela o shell que ele substitui: um prazo
  desconta um `DT` fixo por turno e desenha no turno em que vence; o
  tick de um turno desenhado recebe o tempo desde o último turno
  desenhado. o cabeçalho do relatório diz isso no lugar de "fixed tick".
- engine: `AnimationTick::since_last`, os segundos reais desde o tick
  anterior, sem clamp (0,0 no primeiro tick); `dt` continua travado em
  0,1 s para quem anima. os literais `AnimationTick { .. }` que pararam
  de compilar ganharam o campo: `engine/src/perf/tests.rs`,
  `monster/src/tests/{play.rs, play_ops.rs}` e os dois ticks zerados de
  `engine/src/window/mod.rs` (fora da lista autorizada, mas era isso ou
  o engine não compilava; duas linhas). teste em
  `engine/src/animation/tests_clock.rs`: dois ticks com 120 ms de sleep
  entre eles dão `since_last` acima do sleep e `dt` no máximo 0,1.
- o shell do showcase (`apps/uigallery/src/shell/mod.rs`): `RedrawRequested`
  chama `view.wake(tick.since_last)`, desenha, e `pace` traduz o `Wake`:
  `Some(ZERO)` invalida e pede o próximo frame (como antes para
  animação), `Some(d)` põe `ControlFlow::WaitUntil(Instant::now() + d)`,
  `None` põe `Wait`. `now` não pede nada a mais nesse ponto: a cena é
  construída depois do tick, então o frame em curso já carrega a virada
  (confirmado na janela: um frame por virada, abaixo).
  `new_events(StartCause::ResumeTimeReached)` volta o flow para `Wait`
  (um `WaitUntil` no passado dispara a cada iteração se o redraw não
  vier, janela ocluída) e chama `invalidate()`, que pede o frame cujo
  tick vira o caret. uma linha `log::debug!` por frame registra o
  intervalo e o `Wake`, para contar frames numa janela real.
- `showcase::view::Wake`, o callback do worker do git, virou `GitWake`
  (`apps/git.rs`, `git/{stub,desktop}.rs`, `view/mod.rs`); `comps::Wake`
  é o único `Wake` no view.

medição (release, uma vez, 120 turnos a 60 Hz = 2 s, viewport 1200x800
a escala 2, theme core):

| cenário | antes (task-73) | depois |
| --- | --- | --- |
| g. campo focado, nada digitado | 120 de 120 turnos | 4 de 120 turnos (2 frames por segundo: as viradas em 0,53 s, 1,06 s, 1,59 s e 2,12 s) |
| a. digitar no campo | 120 de 120 | 120 de 120 (cada tecla desenha) |
| f. página parada, deixada em paz | 0 frames | 0 frames |

testes (gate final, `cargo test --workspace`): comps 349 + 3 antes,
363 + 3 depois (11 do `Wake`/`Blink`, 2 do `TextInput`, 1 do
`EditorView`, todos com dt injetado); showcase 124 (122 passam, 2
ignorados) + 7 antes e depois (um teste do Input page reescrito,
contagem igual); engine 755 antes, 757 depois (os dois do
`since_last`); monster 119, rec 5, ide 45 iguais. gate verde quatro
vezes: duas sobre a base `19aeef7` (69 `test result: ok`), uma depois
do rebase sobre a `main` em `3d6bdf3` (a 62c) e a última sobre a `main`
em `26d7e3d` (a 66b; 72 `test result: ok`, 0 falhas, engine 771 e
showcase 129 pelos testes que a 66b trouxe; a perna do urnaui pulou sem
`../urna`). os dois rebases foram mecânicos: a 62c não toca arquivo meu
e só chama o `tick` interno do ide; a 66b tirou o módulo `headless` do
fim de `showcase/src/app.rs` (meu diff é o loop, no meio) e escreveu
noutra seção do how-to.

### a decisão do FrameClock (tomada: opção 1)

problema: `FrameClock::tick` devolve `dt.min(0.1)`. com o loop dormindo
em `WaitUntil(agora + 0,53 s)`, o tick acordaria com 0,1 s, o `Blink`
pediria 0,43, e o caret viraria só depois de seis acordadas (período de
1,7 s, seis frames por virada). o clamp não protege estabilidade (a
`Spring` é analítica, o `Tween` satura); ele só encurta o tempo que os
timers veem depois de um stall.

opções consideradas: (1) o engine expõe o tempo real num campo novo,
`dt` travado mantido; (2) o shell guarda o próprio `Instant` (relógio
paralelo num app, que ide e urnaui copiariam); (3) dt real por
diferença de `tick.elapsed` (f32; depois de 8 h o ulp é 3 ms).

decisão do orquestrador (2026-09-23): opção 1, campo `since_last`. o
shell passa `since_last` para `wake`.

### na janela real

`cargo run -p showcase Todo` com `RUST_LOG=showcase=debug`, campo
focado por um clique real (CGEvent) no centro do campo achado pela
árvore de acessibilidade, depois 6 s sem tocar em nada:

- antes do clique, com a página parada, os frames que apareceram tinham
  todos `wake now false next None`: pedidos de fora (o worker do git via
  `UserEvent::Wake`, a bridge de acessibilidade via `UserEvent::Access`
  quando a árvore é consultada), nenhum do tick. é o comportamento
  anterior a esta task e fica como está.
- depois do clique, 11 frames em 6 s, todos `now true`, com `since_last`
  entre 0,529 e 0,531 s e `next` em torno de 528 ms: um frame por
  virada, 1,9 por segundo. nenhum frame extra entre viradas. antes desta
  task o mesmo estado desenhava 60 por segundo.

o `WaitUntil` não segura os outros acordadores. winit 0.30
(`event_loop.rs`, `ControlFlow::WaitUntil`): "When the current loop
iteration finishes, suspend the thread until either another event
arrives or the given time is reached." um `request_redraw` do worker do
git, da bridge de acessibilidade ou do agent port chega como evento e
encerra a espera; o `RedrawRequested` seguinte refaz o prazo a partir
do `Wake` daquele frame. o shell do showcase não tem hot reload próprio
(o `check_narrate_reload` é do `engine::window`, intocado); o `PerfHud`
também é só do `engine::window`, então o contrato
`hud-implies-continuous-frames` não muda.

### dívidas e pendências

atualizada no fim da fase b (2026-09-23, branch `task/82b-wake-everywhere`).
o que a fase a deixou e o que aconteceu com cada item:

- ide (`apps/idestudio/src/interfaces/panels/commit_form.rs`, `apps/idestudio/src/main.rs`):
  fechou na fase b. a cadeia `Workbench` -> `WorkspaceView` -> painéis,
  formulário e toasts responde com `Wake`, e o shell do ide passa
  `since_last` e traduz o `Wake` como o showcase. `TextField::tick` e o
  `bool` de `CommitForm::tick` saíram. medido abaixo: 11 frames em 6 s
  com o campo focado, eram 1431.
- urnaui (`apps/urnadbapp/src/view/field.rs` e o `app.rs`): fechou na
  fase b. os oito `tick -> bool` viraram `wake -> Wake` e o shell faz o
  mesmo `WaitUntil`. com o `../urna` presente o gate compila e testa a
  perna do urnaui (82 testes, eram 80); numa janela real parada, 0
  frames em 6 s depois dos dois da abertura.
- com a página parada e sem foco, o worker do git pedia frames avulsos no
  showcase: fechou na fase b. `UserEvent::Wake` aplica os eventos no
  workspace embutido qualquer que seja a página e só pede frame com a
  página Git na tela (3 acordadas, 0 frames, eram 3).
- a bridge de acessibilidade continua pedindo frame avulso:
  `UserEvent::Access` invalida sempre (`apps/uigallery/src/shell/mod.rs`). fica.
- o `EditorView` não tem foco: o blink dele está sempre pendente, então
  o Editor page pede uma virada a cada 0,53 s enquanto está visível,
  mesmo sem ninguém editando. um estado de foco no editor (blur ->
  `Wake::idle()`) é task própria. fica. (a primeira virada depois de
  uma edição já conta a partir da tecla: os shells alcançam o tempo
  antes de aplicar a tecla, descrito abaixo.)
- o caret do dock (`apps/uigallery/src/domain/dock.rs`, `blink_t`) é
  do modelo da demo e continua pedindo frame contínuo enquanto o painel
  está aberto. fica.
- a linha `log::debug!` por frame fica nos três shells (showcase, ide,
  urnaui), com o mesmo texto: só aparece com `RUST_LOG=<crate>=debug` e é
  o contador da medição. o showcase ganhou uma segunda linha,
  `git worker woke: frame <bool>`, pelo mesmo motivo.

achados da auditoria que entraram na fase b e fecharam:

- toast do ide pedindo frame contínuo até sair (`|| !toasts.is_empty()`):
  o `ToastManager` responde com `Wake` (movimento enquanto a mola anda,
  prazo no instante em que o primeiro toast visível some, `idle` sem
  toast). o `ToastManager::tick` saiu.
- a página Git do showcase elevava o `bool` do `Workbench::tick` a
  `Wake::moving`: agora pega `GitPage::wake` e o gallery só junta o pedido
  com a página na tela.

achados novos da fase b. os dois primeiros o orquestrador trouxe para
dentro da fase (e fecharam); o terceiro fica:

- o ide parado nunca parava de desenhar num repositório comum: o
  `git status` do worker pega o `.git/index.lock` opcional, o watcher
  (`apps/idestudio/src/repository/watcher.rs`) via a escrita dentro do `.git`, disparava
  `Refresh`, que rodava `git status` de novo. eram dois frames a cada
  ~0,34 s e um processo `git status` por volta (34 frames e 17 processos
  em 6 s, medidos com um `git` de contagem no PATH; log e branches vêm
  do gix, sem processo), para sempre. fechou: o crate `git` roda todo
  processo com `GIT_OPTIONAL_LOCKS=0` num lugar só (`GitRepo::command`),
  e o watcher ignora evento cujo único caminho é `.git/index.lock`
  (defesa contra outro git na máquina). parado: 0 frames e 0 processos
  em 6 s.
- molas que começam num evento absorviam o intervalo antes dele: o shell
  passa `since_last`, e uma mudança aplicada entre dois frames recebia o
  intervalo inteiro no frame seguinte (um `Switch` virado depois de 3 s
  parado chegava ao fim no primeiro frame; o `Scrollbar` não aparecia no
  primeiro wheel depois de um tempo parado; um toast de recusa depois de
  10 s nascia vencido). fechou: os três shells alcançam o tempo no
  começo de cada volta do loop (`new_events`), antes de qualquer
  entrada, resultado do worker ou pedido do agente e da bridge; o tick de
  alcance não desenha, só pede o frame se mudou algo visível. a regra do
  "fresh" que o `Toast` e o `TextInput` tinham ganhado saiu (redundante,
  abaixo).
- as molas do toast levam ~1,7 s cada para chegar ao `rest_threshold`
  de 0,001 do `Spring` do engine: da vida de ~5,7 s de um toast, ~3,4 s
  ainda desenham todo frame, a maior parte com movimento abaixo de um
  pixel: é quase todo o custo dos 381 frames da medição b. é do engine
  (o limiar de repouso) ou da task-73 (frame só com mudança visível).
  fica.

## fase b aberta (orquestrador, 2026-09-23)

a auditoria de verdade (`docs/log/truth.md`, seções 4g e 5.1)
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

método: build de debug (`cargo build -p ide -p showcase`), numa janela
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
- cargo test -p git, -p comps, -p ide, -p showcase verdes
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
