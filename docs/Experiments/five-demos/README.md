---
id: experiment-five-demos
title: "five demos of qualitatively different software, each with its method written before it ran"
project: plev
type: experiment
status: reference
date: 2026-09-25
date_basis: text
recorded: 2026-10-31
topics: [generated-surfaces, agents, recording, ir]
relations: [task-80, task-80/phase-1-probe-and-demos, task-80/demo-5-generative-ui, task-96, task-96/phase-a-surface, task-96/phase-b-title-and-column, task-97, adr-0070]
evidence: [c8a6c3a9, 46f272d3, b26dc988, ae5be30e, 677b1ad9, 07650665, 9c96bde4]
source: reconstruction
source_revision: c2e3f8cd
revised: 2026-10-31
review: reviewed
---

# five demos of qualitatively different software, each with its method written before it ran

Task-80 asked whether the model produces software that is qualitatively different, not only faster, and set five demos to test it: stale-state rejection, human-agent handoff, inspectability, an ephemeral interface and generative UI in a real application. The method for all five (a script, a control, a comparison criterion and what would bring each one down) was committed before the new runs (`c8a6c3a9`), together with the program's stop rule. As of 2026-09-25, demos 3 and 4 ran as tests in the gate, demo 1 had proof from task-62 while its two control arms had not run, demo 2 had proof only with a scripted person, and demo 5, built by task-96, came out mixed against the fixed panel: better on the files of the three commits, worse on actions and on the diff.

## Question

Task-80 states the goal as "provar que o modelo produz software qualitativamente diferente, não só mais rápido: interfaces efêmeras, handoff humano-agente, stale-state rejection, inspectability e UI generativa, em aplicações reais, com método, com números onde números cabem e com a pergunta do núcleo respondida por quem usa" (task-80 README, "objetivo"). In English, a translation made for this page: to prove that the model produces qualitatively different software, not only faster, with method, with numbers where numbers fit, and with the core question answered by the person who uses it.

The five demos (task-80 README, "escopo"):

1. stale-state rejection: the full ide experiment of task-62, phase c, with its metrics and its comparisons against a screenshot and against the existing structured alternative.
2. human-agent handoff: the agent prepares a commit, the person takes it over, changes part, hands it back, and the agent finishes; each receipt says who did the step.
3. inspectability: a real or provoked bug, sent as a session excerpt, opened in the inspector at the exact instant, with the thesis questions answered on screen, and turned into a regression test that catches the break.
4. ephemeral interface: the silences review is born from an intent, becomes a comparison and disappears; it never existed as a screen.
5. generative UI in a real application: an intent in an app that already exists ("revisa esses três commits" in the ide) generates a verified surface that the user operates.

The stop rule, which covers all five: "se a rejeição e os receipts não mudarem como a delegação é feita, ou se as superfícies geradas forem piores que as fixas nas mesmas tarefas, a direção é reavaliada com os dados, não defendida" (task-80 README, "critérios de parada").

## Setup

Method. Each demo had, written and committed before any new demo ran, a script, a control alternative, a comparison criterion and what brings the demo down. The commit `c8a6c3a9` is the proof that the method came first: no new session was recorded before it (task-80 phase 1, "o método das cinco demos"). Phase 1 started from `main` @ `d1b4938d` on branch `task/80a-proof` (task-80 phase 1, "fase 1: sonda").

What the sources record of each run:

| demo | run | build or branch | environment as recorded |
|---|---|---|---|
| 1 | task-62 lab rounds, five of them; the first is checkpoint a | first round on `main` @ `d26c9b1`; other rounds: not recorded | one model, `claude-fable-5-1`, in the task-62 runs; recording costs in a debug build |
| 2 | task-62 rounds four and five, with a scripted person | not recorded | the same task-62 runs as demo 1 |
| 3 | `ide-lab --arm scripted --human none`, no window, no model | branch `task/80a-bug84` (the race put back, commit `84161012`); test run on `task/80a-proof` (`46f272d3`) | no window, no model |
| 4 | test `apps/uigallery/tests/ephemeral.rs`, 1200 by 800, seeded cuts, synthetic frames | the inspector header says "built from b26dc988" | no window, no ffmpeg, no Claude |
| 5 | `ide-lab --arm review`, scripted person, rules only | `2e6c9e9a` (`task/96a-ide-surface`); `10f66729` (`task/97a-code-in-ir`); `e42bd571` (`task/96b-ide-surface-polish`) | no window, no Claude; the Claude path was tested only with the fake model |

- Machine: not recorded for any of the five demo runs.
- Build profile of the demo runs: not recorded. The gates of tasks 96 and 97 ran in debug with `line-tables-only` (task-96 phase a and task-97, "validação").
- Phase 1 runs opened no window, synthesized no click or key, and made no call to `claude` (task-80 phase 1, "validação").
- Units follow the sources: `B` is bytes, decimals use a comma and thousands use a dot, and numbers are copied as written.

## Results

### Demo 1, stale-state rejection: proven by task-62, two control arms not run

Source: [task-80 phase 1](../../Tasks/task-80-proof-qualitatively-different-software/phase-1-probe-and-demos.md), section "o que já estava provado", which reports the task-62 rounds.

- First round (checkpoint a, `main` @ `d26c9b1`): H1 to H5 held. In the A2 arm (the LLM through the contract, with the person), the `submit` based on the review taken before the person's change came back `stale` with the exact delta: `changes:staged` lost `src/session.rs`. HEAD did not move; the agent observed, re-staged and committed the three files, and the receipt matched git.
- The git arm on the command line (B2, B3): the equivalent intervention, a hook on the commit, produced a two-file commit with no warning. B3 fixed it only after investigating, by rewriting with `--amend`.
- Volume, first round: git used 1.459 B in 20 invocations (B3); the A2 arm used 22.040 B in 30 messages. One observation is 9,9 times smaller than the frame's PNG at scale 1 (6.359 B against 62.939 B). The source reads this as volume, not capability.
- Second round (after tasks 83, 84 and 85): R1 to R3 held, and the `acted` named the person. R4 was mixed: the control was 11% cheaper in bytes, and the runs with the person were 14 to 16% more expensive.
- Third round (the rule of task-89): safety held in all four runs, with no wrong commit. One agent (A2''b) stopped without a commit because it had no way to ask the person, which became task-92.
- Fourth and fifth rounds (the handoff): safety held in all eight runs. Without a hint, nobody asked. With the phrase in the briefing, or the stderr line of task-94, the four runs that could ask did ask, and the person's answer decided the commit.
- Recording cost, first round, debug build: 138 us per `rec.snapshot` and 289 us per `rec.frame`. The A2 session was 181.586 B, with 67 events and 15 frames.
- Screenshot arm (C): "comparação de volume, não de capacidade". The source records "nenhum agente por screenshot foi rodado"; ADR 0070 says "Esse braço ainda não rodou." Source: [ADR 0070](../../adr/Generative/0070-generative-runtime-results/0070-generative-runtime-results.md).

### Demo 2, human-agent handoff: proven only with a scripted person

Source: [task-80 phase 1](../../Tasks/task-80-proof-qualitatively-different-software/phase-1-probe-and-demos.md), section "o que já estava provado".

- Each refusal named the person in `acted` (request 6, `unstage` of `src/session.rs`) in the six runs of the fourth round and in the two of the fifth.
- The `--ask` reached the person through the modal, and the person's answer decided in the four runs where there was a question (A4c1, A4c2, A5a, A5b). No act came after a `declined`.
- Not covered: the scope's handoff in a real window, with brenner taking over, changing part and handing back; a person from outside; the core question.
- These sources record no result for a control arm of demo 2.

### Demo 3, inspectability: the task-84 race read at the instant, and its test

Source: [task-80 phase 1](../../Tasks/task-80-proof-qualitatively-different-software/phase-1-probe-and-demos.md), sections "demo 3, inspectability (feita nesta fase)" and "resultado contra o critério"; [ADR 0070](../../adr/Generative/0070-generative-runtime-results/0070-generative-runtime-results.md).

- The bug is the `Refresh` race of task-84: a status read taken before a stage lands after it and takes a file out of `changes:staged`. The correction is `6500f2dc`. The race was put back on the throwaway branch `task/80a-bug84` (commit `84161012`, never merged).
- Session, recorded with no window and no model (`ide-lab --arm scripted --human none --out <scratch>/bug84-none --app ide80 --timeout 60`): exit 0, session and git MATCH, the three files committed. But the first `submit` (request 6) came back `stale`, with "changes: added [tests/login_test.rs]" and `acted` empty. The session is 81.862 bytes, 47 events and 10 frames.
- Excerpts from `plev-inspect --export`: the bug report (`--export 0 120`) is 71.580 bytes, 37 events and 9 frames. The test excerpt (`--export 0 92`) is 43.916 bytes, 25 events and 7 frames.
- The eight thesis questions were answered from the excerpt alone, checked against the code. The frames alone answer at most three ("3 das 8"). The two lapses lasted 29 ms (+0.048 to +0.077 s) and 24 ms (+0.077 to +0.101 s), shorter than one frame at 30 fps (33 ms).
- The test is `apps/idestudio/tests/rerun/inflight.rs`, `the_bug84_excerpt_keeps_every_staged_file_with_the_reads_in_flight`. With the race back it fails: three marks diverge (lines 23, 24 and 25). On `task/80a-proof` it passes ten times out of ten. The control, a rerun at rest, passes on both branches. The four hand-written task-84 tests fail as they did on their own base.
- Cost of the test: 26 lines of marks (six marks), plus the `InFlight` fixture, which serves any lab session. The hand-written task-84 tests are 146 lines of Rust (`apps/idestudio/tests/human_agent.rs:466-611`) and cover four scenarios; the excerpt covers the stage scenario, "o do stage, duas vezes" in the source's count. Phase 1 gives the hand-written range as `:491-611` in its alternatives section and `:466-611` in K4; both are in the same page.
- Privacy: the session has no person input (`--human none`). The bug report has event types `observed` 3, `snapshot` 3, `changes` 7 and `stage` 24, with one actor, `agent:scripted`, and `inputs: 0`.
- Source's verdict: "nada derrubou a demo."

### Demo 4, ephemeral interface: the review born from an intent, and its comparison

Source: [task-80 phase 1](../../Tasks/task-80-proof-qualitatively-different-software/phase-1-probe-and-demos.md), sections "demo 4, interface efêmera (feita nesta fase)", "a gravação", "a comparação contra a tela fixa"; [ADR 0070](../../adr/Generative/0070-generative-runtime-results/0070-generative-runtime-results.md).

The recording (`showcase-ephemeral-test.monster`, built from `b26dc988`) is 153.658 bytes (events 71.146, chunks 52.539, bindings 8.923), with 59 events and 61 frames. It has nine requests: `agent:demo80` made eight, and `human` made one, with two inputs.

| criterion | generated surface | fixed screen beside it | source |
|---|---|---|---|
| K1, the task reaches the app | the click on `proposed.silences.cut[1].apply_cut` is the person's request 4, `apply_cut` on `silences.cut[1]`, completed | `silences.cut[1]:decision = "apply"` at the end | receipts and final state |
| K2, cut 1 in view at 1200x800 | 5 of 5 (frame, band, decision, apply, reject) | beside the generated one: 0 of 5 (the line at y = 968, below 800). Alone, before the surface: 5 of 5 (the whole line at y = 684) | bindings of the frame |
| K2, cut 2 in view at 1200x800 | 5 of 5 | beside: 0 of 5 (y = 1080). Alone: 0 of 5 (the line at y = 752, 8 of 68 px in view) | bindings of the frame |
| K3, the original beside the result | 2 frames in view, side by side (`proposed.silences.video.original` and `.result`, 180 by 140 each) | 1 frame (`silences.video`, 380 by 214); the result needs another act | bindings after the comparison |
| K4, ephemeral | nodes in the state only from +0.043 s (just after the propose, at +0.041 s) to +0.117 s (the close, at +0.116 s); no binding outside that span; nothing in the last frame | in the state from start to end | state event by event, bindings frame by frame |
| K5, cost | review in 0 ms by the rules (39 nodes); comparison in 0 ms (33 nodes); the surface lived 75 ms in this run | not applicable | receipts |
| K6, refusal | "I can't compare these" ends `failed` with its reason; the surface's nodes and states are the same before and after request 6 | not applicable | state before and after request 6 |

- Node count disagrees inside the source. The propose receipt (request 3) says `nodes = 39`, and K5 says "39 nós". The privacy check of phase 1 says "os 40 nós da superfície carregam `source`", and ADR 0070 says "its 40 nodes come from the materializer".
- Source's verdict: "nada derrubou a demo. a gerada não foi pior em K2 nem em K3, e em K2 foi melhor que a fixa sozinha na janela padrão (a fixa mostra o segundo corte só 8 px)."
- Earlier checkpoint b of task-69, as phase 1 records it: the first round of the generated review came out worse (the cut line broken by the rules; with Claude, the task last, sliders without labels and raw numbers). After the corrections, the second round was no longer worse; the state was one, and deciding on either screen left both equal.
- Privacy, checked by the test over 2.521 nodes read event by event: the person's input is two `pointer_down` and `pointer_up` events with the control and the position; no node is `secret`; no camera is opened; the accessibility preferences strip did not open, so none of its nodes is in the recording. The intent text is recorded as the field's state, a point the source notes for whoever decides whether an intent field should be secret.

### Demo 5, generative UI in an application that exists: mixed, and it falls on the diff

Sources: [task-80 demo 5](../../Tasks/task-80-proof-qualitatively-different-software/demo-5-generative-ui.md) (the three runs), [task-96 phase a](../../Tasks/task-96-ide-generated-surface/phase-a-surface.md) and [task-96 phase b](../../Tasks/task-96-ide-generated-surface/phase-b-title-and-column.md), [task-97](../../Tasks/task-97-code-and-diff-in-the-ir/README.md).

The same scripted person ran the same task, "review these three commits", three times. The fixed panel is the stacks panel with the diff panel. Recordings: 128.824 B, 84 events, 19 frames (first run); 138.262 B, 84 events, 19 frames (second run); 136.414 B, 84 events, 19 frames (third run).

| criterion | first run, `2e6c9e9a` | second run, `10f66729` | third run, `e42bd571` |
|---|---|---|---|
| acts to see the files and the diff of the three | surface: 5 clicks (the field, propose, three "show diff") and 26 keystrokes; closing is one more. Fixed: 3 clicks | surface: 5 clicks and 26 keystrokes; closing one more. Fixed: 3 clicks | same as the second run |
| time, workbench clock (machine, not person) | surface 73 ms from the first act to the third diff; proposal by the rules 0 ms (24 nodes). Fixed 47 ms | surface 94 ms; proposal 0 ms (24 nodes). Fixed 64 ms | surface 70 ms; fixed 44 ms |
| files of the three in view at once | surface 3 of 3; fixed 1 of 3 at most | surface 3 of 3; fixed 1 of 3 at most | same |
| diff lines in view without scrolling, the three commits (surface, then fixed) | 5 of 14, 5 of 7, 5 of 18 / 14 of 14, 7 of 7, 18 of 18 | 14 of 14, 7 of 7, 14 of 18 / 14, 7, 18 | same as the second run |
| diff lines cut by the width (surface, then fixed) | not measured | 0, 0 and 3 / 0, 0 and 4 | 0, 0 and 3 / 0, 0 and 4 |
| the three commits in view | surface: 3 whole cards. Fixed: 3 cards, the message cut | surface: 3 whole cards. Fixed: 3 cards, the message cut | surface unchanged. Fixed: 3 cards, the whole message (0 of 3 cut; 3 of 3 cut on the layout before phase b) |
| title of the diff card in the surface | not recorded | the commit id, `stacks.commit[847f583a…` | "diff" |
| acts in the workbench's dispatcher, with the person's receipt | 5 `human` requests in the surface, completed; fixed: 3 `human` requests, completed | same; `show_diff` arrives as the app's capability | 8 `human` requests in the whole run, all completed, 0 refusals |
| nothing left after close | 0 nodes, 0 bindings, 0 surfaces; state from +0.086 s (after the propose) to before the close (+0.176 s) | 0 nodes, 0 bindings, 0 surfaces; state from +0.114 s to before the close (+0.227 s) | same |

- Source's verdict on the first run: "na tarefa inteira ('ver os arquivos e o diff dos três'), a superfície não foi melhor que o painel fixo, e no diff foi pior". By the method, "a demo 5 cai no diff e se sustenta nos arquivos".
- Second run: "pelo critério escrito antes, a demo continua caindo no diff, agora por 4 linhas de um dos três commits e pelo número de linha, e continua de pé nos arquivos".
- Third run: "nada do resultado da task-97 mudou de lado".
- Why the surface lost on the diff: the `code` is limited by `dropdown_max_h` (320 px in hoff), which shows about 14 lines of 19,6 px. The projection carries each line's mark, not the old and new line numbers (task-97, "onde ela perde, e por quê").
- The task-97 closing recorded two visual defects as debt (P2): the diff card titled with the raw commit id, and the surface column taking about 370 px even with no surface open. Task-96 phase b fixed both and measured again; the base before the change measured 71 ms (surface) and 43 ms (fixed).
- Failed or negative results recorded: the surface lost on actions (5 clicks and 26 keystrokes against 3 clicks) and on the diff; the fixed panel was faster on the clock in each run (47, 64 and 44 ms against 73, 94 and 70 ms).

## Limits

- Demo 1: the screenshot-agent and accessibility-snapshot arms did not run. All runs used one model. Runs per arm were two to four ("duas corridas por braço não medem taxa"). Costs come from a debug build. The person was always scripted (task-80 phase 1, "o que ainda não cobre").
- Demo 2: proven only with a scripted person. The handoff in a real window, the outside person and the core question depend on brenner and on a person from outside (task-80 phase 1, "o que falta, e com quem").
- Demo 3: covers the stage scenario of the race. The task-64 harness's settle hides any race between a worker's answer and the next act (P3). A recorded verdict cannot be marked as expected, so a whole bug report cannot become a test yet (P3, in phase 1 and ADR 0070).
- Demo 4: rules only, with Claude outside these runs. Cuts come from a seed, and frames are synthetic. "À vista" counts presence in the frame, not how well a person decides; the generated frame is smaller than the fixed one.
- Demo 5: rules only, with a scripted person, so the time is machine time and varies from run to run. The diffs are small enough to fit the fixed panel. The demo with a window, run by brenner, is still open. The Claude path was tested only with the fake model.
- Machine and build profile of the demo runs: not recorded (see Setup).
- Two source disagreements are shown above: the node count of demo 4 (39 against 40) and the line range of the hand-written task-84 tests (`:491-611` against `:466-611`).
- The task-80 record keeps the task in progress: demos 1 and 2 have partial proofs, and the core question and the outside person remain open (task-80 README, opening paragraph).

## Records

- [task-80 record](../../Tasks/task-80-proof-qualitatively-different-software/README.md): the goal, the five demos, the method, the stop rule and the phase list.
- [task-80 phase 1](../../Tasks/task-80-proof-qualitatively-different-software/phase-1-probe-and-demos.md): the probe, the method, what was already proven, demos 3 and 4 with their recordings, and the handoff.
- [task-80 demo 5](../../Tasks/task-80-proof-qualitatively-different-software/demo-5-generative-ui.md): demo 5 run three times against the fixed panel, with the criterion, the results and the limits.
- [task-96 record](../../Tasks/task-96-ide-generated-surface/README.md): the generated surface in the ide, built for demo 5.
- [task-96 phase a](../../Tasks/task-96-ide-generated-surface/phase-a-surface.md): the probe, the implementation handoff and the first demo 5 run.
- [task-96 phase b](../../Tasks/task-96-ide-generated-surface/phase-b-title-and-column.md): the diff card title, the column shown only with a surface, and the third demo 5 run.
- [task-97 record](../../Tasks/task-97-code-and-diff-in-the-ir/README.md): the `code` component in the IR, and the second demo 5 run with it.
- [ADR 0070, generative runtime results](../../adr/Generative/0070-generative-runtime-results/0070-generative-runtime-results.md): the method, the results of demos 1, 3 and 4, and what remains with its owner.
