---
id: adr-0070
title: "generative runtime results"
project: plev
type: adr
status: proposed
date: 2026-09-25
date_basis: header
recorded: 2026-09-25
topics: [agents, generated-surfaces, semantics, recording, research]
relations: [adr-0051, task-62, task-63, task-64, task-69, task-70, task-80, task-84, task-96, experiment-five-demos, learning-generative-runtime, learning-cycle-0070]
evidence: [c8a6c3a9, 6500f2dc]
source: docs/adr/generated/0070-generative-runtime-results.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# generative runtime results

Records the method of the program's five proof demos, written and committed before any new demo ran, and the results of demos 1, 3 and 4: rejection of stale state, inspection of a real run and an ephemeral interface. Demo 2 has proof only with one scripted person, two control arms of demo 1 did not run, and demo 5 was referred to task-96 as of this record's date. It matters because the program's direction can only be revisited with the data if the criteria and the stopping rule were written beforehand.

## context

goal 10 of the program asks for proof that the model makes software
qualitatively different, not only faster: ephemeral interfaces, human and
agent handoff, stale-state rejection, inspectability and generative ui in
real applications (task-80, thesis sections 28 and 33). the thesis itself
says the differentiation is a hypothesis to demonstrate, not a fact
(section 28): agents operating without pixels exist elsewhere (the
Playwright MCP reads accessibility snapshots), so what has to be shown is
the unified contract: the control, its identity, meaning, actions,
accessibility, automation and record belong to one system.

tasks 62, 63, 64, 69 and 70 are integrated, and much was measured on the
way (the ide experiment in five rounds, checkpoint b). what nobody had done
is put it under one method, and run the two demos that need neither
brenner nor the real Claude.

## decision

### the method, written before running

each of the five demos has, written and committed before any new demo
runs (`c8a6c3a9`, in the task file, "o método das cinco demos"): a script,
a control alternative, a comparison criterion, and what brings the demo
down. the result is written as it comes, negative included. the program's
stop rule holds over all five: if rejection and receipts do not change how
delegation is done, or if generated surfaces are worse than fixed ones on
the same tasks, the direction is revisited with the data.

a criterion is measured from the recording when the recording can say it
(the state event by event, the bindings frame by frame, the receipts),
never by eye where a number exists.

### what is proven, and where

- **demo 1, stale-state rejection**, by task-62's five rounds: an act on a
  state that changed since the agent looked is refused before any effect,
  with the delta naming the file, in every run with a person; the agent
  recovers; the receipt matches git. against git on the command line the
  equivalent intervention produced a wrong commit with no warning, and git
  won on bytes (1,459 B in 20 calls against 22,040 B in 30 messages, first
  round). the screenshot arm measured volume only (an observation is 9.9
  times smaller than the frame's PNG), not capability.
- **demo 3, inspectability**, in this task: a real bug of the history, the
  task-84 race (a status read before a stage lands after it and takes the
  file out of `changes:staged`; fixed by `6500f2dc`), was put back on a
  throwaway branch and recorded by the laboratory with no window and no
  model. the excerpt, opened in `plev-inspect` at the instant, answers the
  eight questions of thesis section 12 from the file alone, where the
  frames alone answer at most three and the two lapses (29 and 24 ms) are
  shorter than a 30 fps video frame. the excerpt (43,916 B) became a test
  in the gate (`apps/idestudio/tests/rerun/inflight.rs`) that fails with the
  race back (three marks diverge) and passes without it, ten of ten.
- **demo 4, the ephemeral interface**, in this task: the review of the
  silences born from an intent, a person deciding a cut on it, turned into
  the comparison in place and closed, recorded with no window by a test in
  the gate (`apps/uigallery/tests/ephemeral.rs`). from the recording: the
  surface is in the state only between the propose and the close and
  leaves no node or binding after; its 40 nodes come from the materializer
  (`rust/screen/src/project.rs`), none from a file of the app's screen;
  the person's click on it is her request on the app's cut; at 1200 by 800
  it shows the five pieces of each cut where the fixed screen beside it
  shows none and the fixed screen alone shows one cut of two; it puts the
  original beside the result where the fixed screen shows one frame.
  checkpoint b (task-69) had already found the generated review no longer
  worse than the fixed one.

### re-running with the reads in flight

the session-to-test harness (task-64 phase c) settles the app before each
act, which drains every refresh owed: no read is ever in flight when the
next act leaves, so no race between a worker's answer and an act can show
in a rerun (the at-rest rerun passes with the task-84 race back). the
test fixture `InFlight` settles only the executions and leaves the reads
the app asked for in flight, the latest point a live window can be at
when the next act comes, which the recording allows. the worker answers
in order, so it is deterministic. the harness, the `Fixture` trait and the
marks are unchanged; the fixture is test code beside the ide's.

## consequences

- demo 3 and demo 4 are tests: the race of task-84 and the ephemeral
  property of a generated surface are guarded in the gate by recordings,
  not by hand-written scenarios.
- a bug report recorded with the bug carries the bug's verdicts (the
  task-84 excerpt's submit was accepted at the check and refused at the
  point of truth), and the harness always compares verdicts with the
  recording: the test's excerpt was cut before the submit. marking an
  expected verdict, as marks already mark a value, would let a whole bug
  report become a test (P3).
- the point of truth held under the race: with the view lying, the
  `CommitIf` refused a two-file commit. the second check is what made the
  race cost a refusal instead of a wrong commit.
- on the Silences, "I can't compare these" is refused by the rules (the
  app declares no list of numbers; task-70); the comparison that exists
  is of views ("compare the original with the cuts"). a list of durations
  declared by the app would let it become a chart (P3).
- the text of an intent field is projected state and not secret, so what
  a person types there reaches the recording through the state, not the
  input: by design of the recording, noted for whoever decides whether an
  intent field should be secret.

## alternatives

- git na linha de comando, como braço de controle da demo 1: a mesma intervenção produziu um commit errado sem aviso. Em bytes, git ficou com 1.459 B em 20 chamadas, contra 22.040 B em 30 mensagens, na primeira rodada.
- captura de tela, como braço de controle da demo 1: só o volume foi medido (uma observação é 9,9 vezes menor que o PNG do quadro), não a capacidade. Esse braço ainda não rodou.
- tela fixa ao lado, como controle da demo 4: a tela fixa ao lado não mostra nenhum dos cinco pedaços de cada corte, e a tela fixa sozinha mostra um corte de dois. A superfície gerada mostra os cinco pedaços de cada corte.
- superfície fixa nas mesmas tarefas, critério da regra de parada: se a superfície gerada for pior, a direção volta a ser revista. O checkpoint b da task-69 já tinha achado a revisão gerada não pior que a fixa.

## what remains, with its owner

- **demo 1, the two control arms never run**: an agent by screenshot
  (needs a pixel-input path in the laboratory, "click at x, y" resolved by
  the headless frame's hit test: new work) and an agent by a third-party
  accessibility snapshot (the AccessKit tree read from outside the
  window). the orchestrator, with the real Claude.
- **demo 2, the handoff with brenner**: the agent prepares a commit,
  brenner takes over in the ide's window, changes part, hands back, the
  agent finishes, each receipt naming who did each step. proven so far
  only with a scripted person (task-62 rounds 4 and 5). brenner, with the
  orchestrator (window and real Claude).
- **the nucleus question (checkpoint c)**: on opening the first `.monster`
  of a real delegation, what brenner can find out, and whether it is
  enough to trust it, in his words. brenner.
- **demo 5, generative ui in an app that exists**: neither the ide nor
  urnaui has a `SurfaceHost`; proposed as task-96 (a generated surface in
  the ide, "review these three commits", beside the stacks panel, acts
  through the workbench's one dispatcher). the orchestrator opens it.
- **the real Claude** on demo 4's surfaces (review, comparison, "I can't
  compare these"), and a person from outside on demos 2 and 4. the
  orchestrator.

## avoid

- claiming a demo from a run that was not scripted, controlled and
  judged by a criterion written before it.
- judging a surface by eye where the recording measures it.
- a session test whose fixture settles everything: it cannot see a race.

## learnings

- [Generative runtime session: what worked and what bit us](generative-runtime.md)
- [what the autonomous cycle taught: generative runtime results](learned-generative-runtime-results.md)

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0051: a surface materializes only from verified IR, projects from the IR and asks through the app's dispatcher](../0051-materialized-surfaces-from-verified-ir/0051-materialized-surfaces-from-verified-ir.md)
- [what the autonomous cycle taught: generative runtime results](learned-generative-runtime-results.md)
- [Generative runtime session: what worked and what bit us](generative-runtime.md)
- [task-62: proof in the ide: stale-state rejection and human-agent handoff](../../../Tasks/task-62-ide-human-agent-proof/task-62-ide-human-agent-proof.md)
- [task-63: session flight recorder](../../../Tasks/task-63-flight-recorder/task-63-flight-recorder.md)
- [task-64: session inspector, bug report and session for testing](../../../Tasks/task-64-session-inspector-and-tests/task-64-session-inspector-and-tests.md)
- [task-69: ephemeral and generated surfaces](../../../Tasks/task-69-ephemeral-generative-surfaces/task-69-ephemeral-generative-surfaces.md)
- [task-70: adaptive transformations](../../../Tasks/task-70-adaptive-transformations/task-70-adaptive-transformations.md)
- [task-80: proof of qualitatively different software](../../../Tasks/task-80-proof-qualitatively-different-software/task-80-proof-qualitatively-different-software.md)
- [task-84: an old refresh does not undo what the ide has already accepted](../../../Tasks/task-84-ide-refresh-never-goes-back/task-84-ide-refresh-never-goes-back.md)
- [task-96: a generated surface in the ide](../../../Tasks/task-96-ide-generated-surface/task-96-ide-generated-surface.md)
- [task-80, phase 1: probe, five-demo method, demos 3 and 4 and handoff](../../../Tasks/task-80-proof-qualitatively-different-software/phase-1-probe-and-demos.md)
- [five demos of qualitatively different software, each with its method written before it ran](../../../Research/Exploring/five-demos.md)
<!-- /related -->
