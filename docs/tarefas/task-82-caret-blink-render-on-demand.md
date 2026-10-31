---
id: task-82
title: "task-82: o caret não pode pedir 60 frames por segundo"
project: plev
type: tarefa
status: concluido
date: 2026-09-23
date_basis: cabecalho
topics: [renderizacao, desempenho, input]
relations: [task-62, task-73]
evidence: [d97a375, a6fec8d]
source: docs/log/steps/task-82-caret-blink-render-on-demand.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# task-82: o caret não pode pedir 60 frames por segundo

Troca o `tick -> bool` do caret por um prazo, o `Wake`, para que um campo de texto focado e parado deixe de pedir 60 frames por segundo: o shell dorme até a próxima virada do blink. A fase a integrou o mecanismo na `main` em `d97a375`, e a medição do cenário g caiu de 120 de 120 turnos desenhados para 4 de 120; a fase b estendeu o mesmo `Wake` ao ide, à página Git do showcase e ao urnaui, e fechou em `a6fec8d`. A regra de render on demand do objetivo, sem mudança visível e sem frame, passa a valer para o caret.

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

## Fases

- [fase a: o blink do caret vira prazo](task-82-caret-blink-render-on-demand/fase-a.md): o `Wake` e o relógio `Blink`, o shell em `WaitUntil`, o `since_last` do `AnimationTick` e a medição do cenário g.
- [fase b: ide, página Git e urnaui só desenham com mudança](task-82-caret-blink-render-on-demand/fase-b.md): o `Wake` no ide, na página Git e no urnaui, o toast como prazo, o git sem locks opcionais e a medição numa janela real.
