---
id: task-82
title: "task-82: the caret cannot request 60 frames per second"
project: plev
type: task
status: done
date: 2026-09-23
date_basis: header
recorded: 2026-09-23
topics: [rendering, performance, input]
relations: [task-62, task-73, experiment-caret-render-on-demand]
evidence: [d97a375, a6fec8d]
source: docs/log/steps/task-82-caret-blink-render-on-demand.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-82: the caret cannot request 60 frames per second

Replaces the caret's `tick -> bool` with a deadline, the `Wake`, so that a focused and idle text field stops requesting 60 frames per second: the shell sleeps until the blink's next turn. Phase a integrated the mechanism into `main` at `d97a375`, and the measurement of scenario g fell from 120 of 120 turns drawn to 4 of 120; phase b extended the same `Wake` to the ide, to the Git page of the showcase and to urnaui, and closed at `a6fec8d`. The render-on-demand rule of the goal, with no visible change and no frame, now applies to the caret.

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

## Phases

- [phase a: the caret blink becomes a deadline](phase-a.md): the `Wake` and the `Blink` clock, the shell in `WaitUntil`, the `since_last` of `AnimationTick` and the measurement of scenario g.
- [phase b: ide, Git page and urnaui only draw when something changes](phase-b.md): the `Wake` in the ide, in the Git page and in urnaui, the toast as a deadline, git without optional locks, and the measurement in a real window.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0042: a tick asks for a deadline, not for every frame](../../adr/Rendering/0042-a-tick-asks-for-a-deadline/0042-a-tick-asks-for-a-deadline.md)
- [what the autonomous cycle taught: a tick asks for a deadline, not for every frame](../../adr/Rendering/0042-a-tick-asks-for-a-deadline/learned-a-tick-asks-for-a-deadline.md)
- [task-62: proof in the ide: stale-state rejection and human-agent handoff](../task-62-ide-human-agent-proof/task-62-ide-human-agent-proof.md)
- [task-66b: SVG export and `--semantics` in headless mode](../task-66b-svg-export/task-66b-svg-export.md)
- [task-67: structural regression before pixels](../task-67-structural-regression/task-67-structural-regression.md)
- [task-73: incremental dependency system](../task-73-incremental-dependency-system/task-73-incremental-dependency-system.md)
- [task-86: the ide people use opens the agent port](../task-86-the-ide-opens-the-agent-port/task-86-the-ide-opens-the-agent-port.md)
- [task-82 phase a: the caret blink becomes a deadline](phase-a.md)
- [task-82 phase b: ide, Git page and urnaui draw only when something changes](phase-b.md)
- [caret blink as a deadline: frames drawn with a focused, idle field](../../Research/Exploring/caret-render-on-demand.md)
- [program: from GPU UI engine to generative interaction runtime](../../Research/Synthesis/generative-runtime-program.md)
- [how to code against the plev engine](../../Research/Workflows/code-against-the-plev-engine.md)
- [autonomous cycle journal](../../Research/Workflows/autonomous-cycle-journal.md)
<!-- /related -->
