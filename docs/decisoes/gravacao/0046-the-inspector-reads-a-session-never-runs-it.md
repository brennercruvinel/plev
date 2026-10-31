---
id: adr-0046
title: "the inspector reads a session, it never runs it"
project: plev
type: decisao
status: aceito
date: 2026-09-23
date_basis: cabecalho
topics: [gravacao, semantica, agentes]
relations: [task-62, task-63, task-64, task-83, task-87, task-90, task-91, programa-runtime, estado-codigo-2026-09, manual-code-against-the-plev-engine]
source: docs/adr/recording/0046-the-inspector-reads-a-session-never-runs-it.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# the inspector reads a session, it never runs it

Registra a decisão de que o inspetor de sessões lê uma `rec::Session` e nunca a executa: o crate `rust/replay` oferece o modelo (linha do tempo, `why`, `control` e `summary`) e o binário `plev-inspect`, sem dispatcher, app ou gravador por perto. O resultado foi medido nas nove sessões de checkpoint da task-62, que abrem com saída 0 e mantêm o mesmo sha256 depois da leitura, e a janela de `apps/inspector` mostra o mesmo modelo sem recalcular respostas. Importa porque quem abre o primeiro `.monster` de uma delegação consegue ver o que precisa para confiar nele, e quem mexeu num valor aparece como correlação no tempo, nunca como escrita declarada.

## context

the question at the core of the program (checkpoint c) is whether someone
who opens the first `.monster` of a real delegation can find what they need
to trust it. since task-63 the recording holds all of it: who observed what
and when (`Kind::Observed`), the state after every step (`Kind::Snapshot`,
`Kind::Changes`), and every stage of every receipt with its time and its
whole request (`Kind::Stage`). nothing read it back as answers.

`acted` (task-83) answers "who moved what I read" only in part: the
receipts of other actors checked on the exact revision the author saw. a
change from outside the app, an act checked on a later revision, and every
refusal recorded before task-83 name nobody there. and the contract
declares what an action reads (`reads`), never what it writes: no field
anywhere says which receipt moved a value. updated on 2026-09-23
(task-90): `acted` names the chain from the first receipt that ran on the
revision the author saw, so an act checked on a later revision is named
when a chain started there; a change no receipt made still names nobody.
updated on 2026-09-24 (task-91): for an act of the port, `acted` names
every request taken after the look it rested on, the agent's own
included; the recording does not keep the look yet.

task-64 phase a asks for the inspector's model without a ui, and a command
line over it.

## decision

### a crate that only reads

`rust/replay`: the lib `inspect` and the bin `plev-inspect`, depending on
`rec` and `sem` (and serde, serde_json for the json form). `Inspector`
borrows a `rec::Session` and holds nothing else: no app, nothing that
dispatches, no recorder. its sources name none of those types, and a test
reads them to check; another records a session through a real dispatcher,
inspects everything it holds at every instant, and finds the app, the
dispatcher and the recording as they were.

### the model

- `timeline()`: every event, one entry, its time from the start of the
  session, summarized; tallies by kind and by actor (a person's input counts
  for `human`, as the recorder records it).
- `why(origin, id)`, the causal chain of one request:
  - the basis: the last `Observed` by the request's actor whose revision is
    `request.observed.revision`, before the request's first recorded stage.
    without one it says "not recorded" and why: no observation by that actor
    (a person acts on the frame on screen, and the recorder keeps only what
    the app reports; or the window starts after it), or only at other
    revisions (the revision came another way, such as the last stage of an
    earlier act, task-87). it then finds the last event that left the
    recorded state at that revision (`State::revision`), which starts the
    stretch instead; when there is none it says so and shows what the actor
    saw beside what the state held right before the request.
  - the request: actor, target (the node as the state held it right before),
    action, parameter, and each read with the value the actor saw. when the
    basis is a scoped observation, each read says whether its node was
    inside the scope: the revision is always the whole snapshot's, and the
    agent port rebuilds a value outside the scope from the whole snapshot,
    so the actor may act on a value that observation never served it.
  - the stretch between the basis and the request: every change of each read,
    with the receipts that ran then (below), and a person's input in it.
  - the stages with their times, the verdict, the delta and `acted` of a
    stale refusal, and how it ended: the outcome (sha, files), the refusal
    (at the check or at the point of truth), the failure, or still open.
- `control(id, t)`: the node the state holds at `t` (role, label, state,
  actions, the source it was projected from beside the build's git sha) and
  every request that touched it up to `t` (`Session::receipts_for`), each
  with its `why` as the window up to `t` tells it.
- `summary()`: the build, the span, the actors, every request with how it
  ended, and every refusal with its `why`.

`text` renders each answer in plain english; the causal chain has one
labelled line per question of the thesis (control, state, initiated by,
action, observed, precondition, before, stages, verdict, what changed,
acted, result). every model type is `Serialize`.

### the state by event, not by time

`Session::state_at(t)` reads by time, and the recorder writes a request's
stages and the projection it leaves in the same millisecond. read by time,
a request's own effect would sit before it. the inspector reads the state
after the first `n` events, with the session's own reader over that stretch
(`Session::from_events` from the governing keyframe, then `state_at`): the
public read side, `rec` unchanged.

### who moved a value: a correlation in time

for a change of a read recorded at event `j`, the window runs from after the
last event before `j` that set the state (a snapshot or changes) up to `j`.
the receipts named are those, other than the request explained, with a stage
`executing`, `completed` or `failed` in that window and whose last stage is
not a refusal (refused at the check, at a confirmation or at the point of
truth, it ran nothing). none: the change came from outside the app, with no
request. one: that receipt. two or more: ambiguous, all shown. the actor's
own acts count and are marked as the same actor, so a change it made is
never called one from outside. every answer that names a receipt says, in
the text and in the model's doc, that it is a correlation and not a
declared write.

its limits:

- it is correlation, not causation. a receipt that ran in the window may not
  have touched the value, and a change from outside the app that falls in
  the same window as a receipt is put on the receipt.
- a receipt whose terminal stage comes after its effect already landed is in
  the window of the next change too. the ide does this (the view moves right
  after `executing`, `completed` arrives milliseconds later), so two acts in
  a row can come out ambiguous for the second one's change.
- a receipt that started before the state before a change and ended after the
  change has no stage in the window: it is not named, and the change reads as
  one from outside. the ide does not do this; an app whose effect lands long
  after the start, with projections in between, would. a test pins it.
- the resolution is the recording: stages and projections in the order the
  app reported them. an app that projects late puts several receipts in one
  window (ambiguous, and said).
- the ring cuts: an observation or the first stages of a request before the
  window are not there (`not recorded`, and `whole: false`).

### the command line

`plev-inspect <session.monster> [--timeline] [--why <id> [--origin <n>]]
[--control <id> [--at <ms>]] [--json]`. the file alone prints the summary.
`--at` is in milliseconds from the start of the session, the time every view
prints (default: the end). `--origin` picks the dispatcher when an id is in
several. text by default, `--json` for the same answers. exit 0 answered, 1
the request is not in the session, 2 bad invocation, 3 the file does not
read as a session (the typed `rec::FileError`, said as it is: an animation,
a bad magic, a truncated file).

## measured on the checkpoint sessions

the nine sessions of task-62 (`target/agents/exp/{a1,a2}`,
`exp2/{a1p,a2pa,a2pb}`, `lab-{scripted,plev,control,control2}`) all open;
the summary, the timeline, the `why` of every request, and the control of
seven ids at the start and at the end answer with exit 0, and every file has
the same sha256 after.

across them, five reads moved between an observation and a request: the
stale submit of each session with the scripted person. each change is put
on exactly one receipt, the person's unstage of `src/session.rs` (request
6), by its `executing` stage, recorded in the same millisecond as the
change or one before it. none
came out ambiguous or from outside the app. in the first round (`exp/a2`)
the refusal names nobody, since `acted` did not exist then; the recording
names the person all the same. in that session the stale submit rested on an
observation of `commit.form` alone, and the value of `changes:staged` it
carried was outside that scope.

## consequences

- the ui of phase b reads this model. it needs from `rec` what [`estado-do-codigo-2026-09.md`](../../historia/estado-do-codigo-2026-09.md)
  2.4 already names: the frame's bindings, not recorded yet, to map a point
  on a replayed frame to a control. a read of the state after an event index
  in `Session` would replace the stretch the inspector rebuilds for each
  change (fine at the size of these sessions; a long window between
  keyframes pays it per change).
- a declared write (an action saying which state it may change) would turn
  the correlation into a check; it belongs to `sem` and is not decided here.
- the inspector is the reader the export and the session-to-test harness of
  phase c build on.

## the window (phase b, 2026-09-25)

`apps/inspector` (the lib and the bin `plev-inspector <session.monster>`)
shows this model and never works an answer out again. it is its own crate,
not a bin of `inspect`: the reader keeps its edges (`rec`, `sem`, serde),
and the window adds `comps` and `engine` (winit through it), nothing from
outside the workspace.

- the panel of a control at `t` is `control(id, t)` as `text::control`
  writes it, then the requests that touched it later, or were still open at
  `t`, from `control(id, end)`, less what `t` already told whole. in the
  commit scenario the button leaves the frame with the commit, so its sha
  only exists after any instant where the button can be clicked.
- the frame is `Session::scene_at(t)`, fit into the view by one scale and
  one offset. the session keeps neither the window's size nor its scale:
  the size is the extent of the frame's nodes and bindings from the origin
  (the recorded apps draw their background there), and the scale does not
  enter the drawing (the nodes are logical). a node past the window's edge,
  whose clip the monster IR v0 does not carry, would widen it; recording
  the window's size is pending for that case.
- the recorded frame is a picture. its bindings are drawn as outlines and
  hit-tested with `Session::hit_at`, never bound in the window's
  compositor, so no recorded control answers there as a live one. the
  window draws no image, from a file or from a session read in the
  process that recorded it; where a stream image was is not shown yet (the
  replay lowers it to nothing, and `rec` does not say where it was).
- the window's own controls (the scrub, the lanes, the list of requests,
  the pick) go to the view's handlers, never through a dispatcher: the
  inspector must never create a request, so they stand outside the rule
  that a click goes through the dispatcher
  ([code-against-the-plev-engine](../../referencia/operacao/code-against-the-plev-engine.md), definition of done, 6),
  and they project nothing for accessibility yet.
- the proof is in tests: the sources name nothing that runs, records or
  builds a request, and no socket, process, port or accessibility bridge;
  the manifest depends on exactly `comps`, `engine`, `inspect`, `rec`,
  `sem`; driving every handler over every instant of the two recorded
  sessions leaves the session and the file's bytes as they were.

## avoid

- anything in `inspect` that could create a request or run an action, or a
  dependency on a crate that does.
- calling a correlation a cause, in the text or in a field name.
- reading the state by time for the stretch before a request.
- naming a refused receipt as the one that moved a value.
- a dependency of the window on anything that dispatches or sends, a
  recorded control bound in the window's compositor, or an answer in the
  window that the model does not give.
