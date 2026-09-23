---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-23
domain: engine
task: task-82
depends: [task-62]
branch: task/82-caret-wake
---

# task-82: o caret não pode pedir 60 frames por segundo

## objetivo

um campo de texto focado e parado desenha 60 frames por segundo para um
caret que muda de visibilidade umas duas vezes por segundo. a task-73 mediu
(cenário g: 120 de 120 turnos desenhados com o campo focado e nada
digitado). isso fere o contrato de render on demand: sem mudança visível,
sem frame.

## estado atual no código

- `TextField::tick` (`crates/comps/src/form/text_field.rs`) devolve `true`
  em todo frame enquanto está focado; o `TextInput` guarda a fase do blink.
- as páginas repassam isso: `TodoPage::tick` devolve
  `animating || self.input.is_focused()`
  (`crates/showcase/src/view/apps/app.rs`), e o formulário de commit do ide
  faz o mesmo (`crates/ide/src/views/commit_form.rs`).
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

## revisão do orquestrador (2026-09-23, antes do briefing)

- mecanismo: `TextInput::tick(dt)` (`crates/comps/src/form/text_input/component.rs:61-70`)
  acumula `cursor_blink_timer` e vira a visibilidade a cada
  `CURSOR_BLINK_INTERVAL` (0,53 s); `TextField::tick`
  (`crates/comps/src/form/text_field.rs:205`) devolve `true` enquanto
  focado; `EditorView` tem o mesmo relógio (`crates/comps/src/editor/view.rs:52,180`).
  `TodoPage::tick` devolve `animating || self.input.is_focused()`
  (`crates/showcase/src/view/apps/app.rs:311-314`), e o shell do showcase
  pede frame sempre que o tick devolve `true`
  (`crates/showcase/src/app.rs:394-417`), com `ControlFlow::Wait`.
- decisão: o tick passa a devolver um prazo, não um booleano de "continue":
  um tipo do comps (nome sugerido `Wake`) que diz se um frame é preciso
  agora e, opcionalmente, quando o próximo é preciso (`Option<Duration>`);
  a composição de vários widgets é o menor prazo. o shell traduz em
  `ControlFlow::WaitUntil(instante)` e só pede frame quando o prazo vence
  ou uma entrada chega. o `PerfHud` continua implicando frames contínuos.
- sequência por ownership: o ide (`crates/ide/**`) é da task-62 fase c em
  andamento; esta task NÃO toca o ide. o `commit_form.rs` e o shell do ide
  adotam o prazo numa fase b, depois da 62c integrar. o urnaui não compila
  sem `../urna`; fica anotado como pendente.

## estado (2026-09-23, agente, branch `task/82-caret-wake`)

feito, em seis commits que compilam sozinhos:

- `comps::Wake` (`crates/comps/src/wake.rs`, exportado na raiz e no
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
- o shell do showcase (`crates/showcase/src/app.rs`): `RedrawRequested`
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

- ide (`crates/ide/src/views/commit_form.rs`, `crates/ide/src/main.rs`):
  fase b, depois da 62c. `TextField::tick` e o `bool` de
  `CommitForm::tick` somem nessa fase; o shell do ide passa
  `since_last` e traduz o `Wake` como o showcase.
- urnaui (`crates/urnaui/src/view/field.rs` e o `app.rs`): não compila
  sem `../urna`; adota o `Wake` na mesma fase b, sem verificação local.
- o `EditorView` não tem foco: o blink dele está sempre pendente, então
  o Editor page pede uma virada a cada 0,53 s enquanto está visível,
  mesmo sem ninguém editando (era 60 por segundo só enquanto alguém
  digitava, e parado entre teclas). um estado de foco no editor (blur
  -> `Wake::idle()`) é task própria.
- o caret do dock (`crates/showcase/src/model/dock.rs`, `blink_t`) é
  do modelo da demo e continua pedindo frame contínuo enquanto o painel
  está aberto. fora do escopo desta task.
- com a página parada e sem foco, o worker do git e a bridge de
  acessibilidade ainda pedem frames avulsos (`next None`); não são do
  tick e não eram desta task, mas são frames sem mudança visível.
- a linha `log::debug!` por frame no shell fica: só aparece com
  `RUST_LOG=showcase=debug` e é o contador que a verificação usou.
