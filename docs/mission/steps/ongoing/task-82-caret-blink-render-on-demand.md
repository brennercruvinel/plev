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

feito, em três commits que compilam sozinhos:

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

medição (release, uma vez, 120 turnos a 60 Hz = 2 s, viewport 1200x800
a escala 2, theme core):

| cenário | antes (task-73) | depois |
| --- | --- | --- |
| g. campo focado, nada digitado | 120 de 120 turnos | 4 de 120 turnos (2 frames por segundo: as viradas em 0,53 s, 1,06 s, 1,59 s e 2,12 s) |
| a. digitar no campo | 120 de 120 | 120 de 120 (cada tecla desenha) |
| f. página parada, deixada em paz | 0 frames | 0 frames |

testes: comps 349 + 3 antes, 363 + 3 depois (11 do `Wake`/`Blink`, 2 do
`TextInput`, 1 do `EditorView`, todos com dt injetado); showcase 124
(122 passam, 2 ignorados) + 7 antes e depois (um teste do Input page
reescrito, contagem igual).

### parado antes do shell: o `FrameClock` trava o dt em 0,1 s

problema. `FrameClock::tick` (`crates/engine/src/animation/mod.rs`)
devolve `dt.min(0.1)`. o shell do showcase alimenta `view.tick(tick.dt)`
com isso. se o shell dormir em `ControlFlow::WaitUntil(agora + 0,53 s)`
e acordar, o tick recebe 0,1 s, não 0,53: o `Blink` avança 0,1, devolve
`next = 0,43`, o shell dorme de novo, recebe 0,1 de novo. o caret vira
depois de seis acordadas (período de 1,7 s em vez de 0,53) e desenha
seis frames por virada. o mecanismo do comps está certo; o que chega
nele é que mente sobre o tempo. o clamp não protege estabilidade: a
`Spring` é analítica (exp, cos, sen; estável para qualquer dt) e o
`Tween` satura no fim. ele só encurta o tempo que os timers veem depois
de um stall.

opções.

1. o engine expõe o tempo real: `AnimationTick` ganha um campo (por
   exemplo `since_last: f32`, sem clamp) ou `FrameClock::tick` deixa de
   travar. o shell passa o tempo real para `wake`. impacto: um arquivo
   do engine; adicionar campo quebra os literais `AnimationTick { .. }`
   em `crates/engine/src/perf/tests.rs` e `crates/monster/src/tests/`
   (dono do engine ajusta); tirar o clamp muda o que ide, urnaui e os
   exemplos do engine veem depois de um stall (as animações saltam para
   o fim em vez de tocar 0,1 s por frame; com a spring analítica é o
   destino certo, e é cosmético).
2. o shell do showcase guarda o próprio `Instant` e calcula o dt sem
   clamp (o `comps --example editor` já faz isso). impacto: nenhum no
   engine, mas um relógio paralelo num app, que o ide e o urnaui
   copiariam na fase b. contra o "avoid building parallel
   implementations of engine capabilities in apps" do AGENTS.md.
3. o shell deriva o dt real de `tick.elapsed` (diferença entre frames).
   impacto: `elapsed` é f32 desde o início; depois de 8 h o ulp é 3 ms
   e o dt de 16 ms fica quantizado. não serve como mecanismo.

recomendação: a opção 1, na forma de um campo novo (`since_last`), com
o `dt` travado mantido para quem já lê. o shell do showcase passa
`since_last` para `wake`. a tradução no shell, pronta para entrar num
commit pequeno depois da decisão (`crates/showcase/src/app.rs`, o
`RedrawRequested` e dois handlers):

- `RedrawRequested`: `let wake = self.view.wake(dt_real)`; render;
  depois `match wake.next`: `Some(ZERO)` -> `compositor.invalidate()` +
  `request_redraw()` e `ControlFlow::Wait` (como hoje para animação);
  `Some(d)` -> `set_control_flow(WaitUntil(Instant::now() + d))`;
  `None` -> `ControlFlow::Wait`. `wake.now` não pede nada a mais nesse
  ponto: o frame em curso já carrega a virada.
- `new_events(StartCause::ResumeTimeReached { .. })`: `set_control_flow
  (Wait)` (um `WaitUntil` no passado dispara a cada iteração) e
  `self.invalidate()`, que pede o frame cujo tick vira o caret.
- entrada do usuário e `UserEvent` continuam chamando `invalidate()`;
  um `WaitUntil` pendente sobrevive a eles e o próximo redraw o refaz.
  o `PerfHud` não existe no shell do showcase (o contrato
  `hud-implies-continuous-frames` é do `engine::window`, intocado).

decisão necessária: qual opção, e quem edita `animation/mod.rs`.

### estado da branch enquanto o shell espera

o shell do showcase ainda chama `self.view.tick(tick.dt)` e pede frame
quando ele devolve verdadeiro. com o `tick` do view agora querendo
dizer "em movimento", na janela o caret do Todo desenha na virada e o
loop dorme até a próxima entrada: o caret fica parado entre teclas (o
Editor page já era assim). é o estado de espera, não o de aceite; o
commit do shell fecha.

### dívidas e pendências

- ide (`crates/ide/src/views/commit_form.rs`, `crates/ide/src/main.rs`):
  fase b, depois da 62c. `TextField::tick` e o `bool` de
  `CommitForm::tick` somem nessa fase.
- urnaui (`crates/urnaui/src/view/field.rs` e o `app.rs`): não compila
  sem `../urna`; adota o `Wake` na mesma fase b, sem verificação local.
- o `EditorView` não tem foco: o blink dele está sempre pendente, então
  o Editor page vai pedir uma virada a cada 0,53 s enquanto estiver
  visível, mesmo sem ninguém editando. um estado de foco no editor
  (blur -> `Wake::idle()`) é task própria.
- o caret do dock (`crates/showcase/src/model/dock.rs`, `blink_t`) é
  do modelo da demo e continua pedindo frame contínuo enquanto o painel
  está aberto. fora do escopo desta task.
- `showcase::view::Wake` (o callback do worker do git,
  `apps/git.rs`) tem o mesmo nome que `comps::Wake`. os módulos que
  usam o do comps importam direto e não colidem, mas dois `Wake` no
  mesmo crate confundem; um rename do alias do git (`Waker`) é
  cosmético e ficou de fora.
