---
id: task-82/phase-a
title: "task-82 phase a: the caret blink becomes a deadline"
project: plev
type: section
status: reference
parent: task-82
date: 2026-09-23
date_basis: text
topics: [rendering, performance, input]
relations: [adr-0042, task-66b, task-73]
source: docs/log/steps/task-82-caret-blink-render-on-demand.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
evidence: [d97a375]
---

# phase a: the caret blink becomes a deadline

Records the first phase: the caret blink becomes a deadline (`comps::Wake`, with the `Blink` clock that `TextInput`, `TextField` and `EditorView` share), the showcase shell sleeps in `WaitUntil`, and `AnimationTick` gains `since_last`. The measurement of scenario g dropped from 120 to 4 drawn turns out of 120, that is, 2 frames per second with the field focused and idle, and the gate closed with 1985 tests passing. The integration into `main` is commit `d97a375`.

## fase a integrada (2026-09-23)

integrada na `main` em `d97a375` (oito commits, rebaseados pelo agente
sobre a 62c e a 66b e pelo orquestrador sobre um commit de docs, sem
conflito), validada pelo orquestrador: diff inteiro lido (o `Wake` e o
`Blink` no comps, `since_last` aditivo no engine, o shell do showcase em
`WaitUntil` com o retorno a `Wait` em `ResumeTimeReached`), gate verde no
diretório da task sobre a base `26d7e3d` (1985 passed, 0 failed, 6
ignored; a `main` tinha 1969); o rebase final só cruzou docs. ADR:
[`0042-a-tick-asks-for-a-deadline.md`](../../adr/rendering/0042-a-tick-asks-for-a-deadline.md).

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

`cargo run -p uigallery Todo` com `RUST_LOG=showcase=debug`, campo
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
