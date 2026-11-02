---
id: adr-0050
title: "sessions become regression tests"
project: plev
type: adr
status: accepted
date: 2026-09-24
date_basis: header
recorded: 2026-09-24
topics: [recording, process, semantics]
relations: [adr-0037, adr-0046, task-63, task-64, generative-runtime-thesis, learning-cycle-0050]
source: docs/adr/recording/0050-sessions-become-regression-tests.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# sessions become regression tests

Decides that a recorded session becomes a regression test through controlled re-execution, and only in tests: the harness re-runs a segment against a fixture that the app provides, in an isolated environment (temporary directory, injected clock, no network), and compares the verdicts and the marked values. The harness lives in `apps/idestudio/tests/rerun/`, outside `rec` and `inspect`, so that reading still executes nothing. Measured on the `a2ppa` session (184 KB, 65 events, 12 frames), the six marks pass in green in 0.4 s, and removing `changes:staged` from the submit's `reads` by hand, without a commit, makes the test fail; it matters because it turns a bug report into a test without making reading an execution.

## context

the thesis (section 12) asks two things of a recorded session: the bug
report stops being a video, and a session becomes a test. task-63 records
a session as data (`rec`, a `.monster` whose magic is `MONS`) and promises
replay only: reading a session never creates a request and never runs an
action. task-64 phase a reads it (`inspect`, `plev-inspect`) under the same
promise, and a test checks that the inspector's sources name nothing that
runs.

a test of a session has to run something: the requests of an excerpt,
against the app, to see whether the verdicts and the values come out as
recorded. that is re-execution, the thing the recording ADR keeps out of
replay. task-64 phase c asks for it, under control: an isolated
environment (a temp dir, an injected clock, no network), a fixture the app
provides, nothing that touches real state, and replay in the inspector
still running nothing.

## decision

### re-execution under control, in the tests only

the harness reruns an excerpt against a fixture and compares. it lives in
`apps/idestudio/tests/rerun/`, beside the only app fixture that exists today,
and nowhere in a library:

- not in `rec`: the crate runs nothing, and its `Session` holds no app
  and takes none. the recording ADR avoids "any api that takes a
  dispatcher or an app on the read side", and a harness beside that read
  side would take exactly that.
- not in `inspect`: the inspector reads; its sources are checked for every
  type and call that runs an action, and its dependencies are fixed.
- not in a crate of its own yet: a crate is named in `docs/ATLAS.toml`
  and the readme (the arc sync guard), and one fixture does not make a
  reuse. the second app with a fixture (the showcase's todo page) is the
  moment to move it to a crate, test-only, depending on `rec` and `sem`.

### the fixture: what an app provides

```rust
pub trait Fixture {
    fn dispatch(&mut self, req: Request, t: Millis) -> RequestId;
    fn confirm(&mut self, id: RequestId, by: Actor, t: Millis) -> Result<(), String>;
    fn decline(&mut self, id: RequestId, by: Actor, t: Millis) -> Result<(), String>;
    fn settle(&mut self) -> Result<(), String>;
    fn project(&self) -> Result<Snapshot, String>;
    fn receipt(&self, id: RequestId) -> Option<Receipt>;
}

pub fn rerun<F: Fixture>(
    excerpt: &Session,
    marks: &[Mark],
    start: impl FnOnce() -> Result<F, String>,
) -> Result<Report, String>;
```

- `start` builds the app in the state the excerpt starts from, in an
  isolated environment. it is a closure, so a test can hand a fixture it
  keeps (`&mut F` is a fixture too) and look at it after.
- `dispatch` takes a request through the app's one dispatcher, the path
  every actor takes. `t` is the recorded time, for an app that stamps with
  the caller's clock.
- `settle` lets everything asynchronous finish: every execution that left
  ends, every read it asked for lands.
- `project` and `receipt` are how the harness reads the app. nothing else
  of the app is visible to it.

### the rerun

in the order the excerpt recorded them:

- every act is rerun. a request's first stage (`requested`) is a dispatch
  of the request exactly as recorded: actor, target, action, parameter,
  and the values the actor observed, which are its preconditions. a
  person's `confirmed` and `declined` are the same answer to the rerun's
  request. every other stage is the app's answer and is produced by the
  rerun, never replayed.
- before each act the app settles, unless the recording had an execution
  still running at that point (its last stage before the act was
  `executing`): then the act goes in with that execution in flight, as it
  did. the a2ppa session has one such overlap: the person's unstage was
  executing when the agent's submit was refused.
- a request the excerpt holds without its first stage (it started before
  the cut) cannot be rerun, and the rerun is refused before the fixture is
  built.
- at the end, each request's verdict (the stage after `requested`) and
  terminal stage are compared with the recorded ones: the same stage, for
  a refusal the same kind, and for a stale refusal the same reads changed.
  a request still open when the excerpt ends has no terminal stage to
  compare.
- never compared: times (the rerun's clock is not the recording's),
  revisions (they hash every projected value, sha-bearing ids included),
  `acted` (it rests on the revision or on the look, and neither carries
  over), and the values an outcome holds (a commit's sha). a mark checks
  an outcome value when one matters.

the result is a `Report`: one line per request (the same, or the verdict
and the terminal stage recorded and rerun, the one that differs marked)
and one per mark (held, or what was found). a divergence is in the report,
and the test fails with it as its message.

### the marks

the developer marks values in a text file beside the excerpt. only what is
marked is checked, never the whole state:

```text
# a comment
at 0             changes:staged = {"README.md", "src/login.rs"}
after request 7  stale = {"changes:staged"}
after request 7  changes:staged = {"src/login.rs", "tests/login_test.rs"}
after request 9  outcome.files = ["src/login.rs", "src/session.rs", "tests/login_test.rs"]
```

- the anchor: `after request <id>` (with `origin <n>` when the id is in
  several dispatchers), once that request is over in the recording; or
  `at <ms>` / `at +<s>s`, from the start of the excerpt as `plev-inspect`
  prints and takes it, once every act recorded up to then was rerun. the
  value is read right before the next act the recording holds, or at the
  end, settled unless the recording had an execution running there.
- what: a state ref of the projection; `outcome.<key>`, a value the
  anchored request completed with; `stale`, the reads its stale refusal
  names, as a set.
- the value: `true`, `false`, an integer, a number, a json string, a list
  `["a"]`, a set `{"a"}`, or `absent`.

a line that does not read is an error with its number; a mark on a request
the excerpt does not hold refuses the rerun.

### the ide's fixture

the laboratory's own start: `ide::lab_repo::Repo::build` (moved from the
`ide-lab` bin to the lib, unchanged, so the test and the laboratory build
the same repository) in a temp dir, a real `git::GitClient` on it, the
`Workbench` connected and loaded. the worker only runs `git` in that
directory. it settles by asking a refresh and applying the worker's events
until every refresh owed is answered: the worker answers in the order it
was asked, a refresh ends with its `Branches`, and each mutation it
answers asks one more.

the workbench stamps its receipts on its own `Instant` and takes no clock;
changing that is outside this phase. nothing the harness compares reads
it, and the toasts' timers move only through `Workbench::wake`, which the
fixture never calls. git stamps its commits with the wall clock, so a
rerun's sha differs from the recorded one, which is why outcome values are
compared only when marked.

### the export

`plev-inspect <session> --export <t0> <t1> <out>` writes the stretch (ms
from the start, as `--at`) as a session of its own through
`Session::excerpt`: from the full snapshot and the chunk of frames that
govern t0, up to t1, only the assets its frames use. it goes alone, never
over the file it reads, and exits 4 when the write fails. the excerpt of a
test is made with it.

## measured

- the probe (in the task file): the nine requests of the third round's
  a2ppa session, rerun on a fresh laboratory repository, give the same
  verdicts and the same stages; so do a1pp, a2ppb and a2ppc.
- `apps/idestudio/tests/fixtures/sessions/a2ppa.monster` (184 KB, 65 events,
  12 frames) with six marks reruns green in 0.4 s.
- with `changes:staged` taken out of the submit's `reads` (by hand, never
  committed), the same test fails on request 7: recorded rejected stale on
  `changes:staged`, rerun accepted. its terminal stage stays the same,
  refused at the point of truth: the worker's `CommitIf` checks the real
  index. the toy fixture keeps a divergent verdict and its message in the
  gate.

## what stays fragile

- ids that carry a run's own values: a commit's `stacks.commit[<sha>]`, a
  toast's `notices[<n>]`. a request aimed at one reruns as `not_found`.
  none of the laboratory's sessions has one before its commit.
- the start. the fixture must start where the excerpt does; the harness
  refuses a request the excerpt cut, but cannot check the state it starts
  from. an excerpt cut in the middle of a session needs a fixture that
  builds that middle.
- `settle` rests on what the fixture knows of the app's asynchronous work
  (for the ide, that every mutation answered asks a refresh). an app that
  changes that makes `settle` wait until its timeout, and the rerun fails
  saying so.
- a parameter aimed at a secret field is recorded as its length and reruns
  as that.
- a session with several dispatchers reruns through the fixture's one.
- the rerun reads what the app projects, never what it draws: frames are
  not compared.
- the proof that a regression is caught was made by hand on the ide,
  since the rule lives in the workbench; in the gate, the toy fixture
  keeps it.
- git reads the user's global configuration (the repository's local one
  sets the identity and turns signing off); it writes nothing outside the
  temp dir.

## consequences

- the showcase's todo page gets its fixture when it records sessions: its
  shell does not depend on `rec` yet. the fixture is a dispatcher over the
  page with the harness's clock; the harness moves to a crate then.
- phase b (the inspector's ui) marks values on screen and writes this
  marks file, and exports through the same `Session::excerpt`.
- recording the revision served with the terminal stage, and the look
  the agent port knows, would let `acted` be compared too.

## avoid

- a harness, a fixture or anything that dispatches in `rec` or `inspect`.
- comparing the whole state, a time, a revision or a sha by default.
- a fixture that reaches past its temp dir, or the network.
- replaying the app's answers instead of producing them.

## learnings

- [what the autonomous cycle taught: sessions become regression tests](learned-sessions-become-regression-tests.md)

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0037: session recording: replay, not re-execution](../0037-session-recording-replay-not-reexecution/0037-session-recording-replay-not-reexecution.md)
- [ADR 0046: the inspector reads a session, it never runs it](../0046-the-inspector-reads-a-session-never-runs-it/0046-the-inspector-reads-a-session-never-runs-it.md)
- [what the autonomous cycle taught: sessions become regression tests](learned-sessions-become-regression-tests.md)
- [task-63: session flight recorder](../../../Tasks/task-63-flight-recorder/task-63-flight-recorder.md)
- [task-64: session inspector, bug report and session for testing](../../../Tasks/task-64-session-inspector-and-tests/task-64-session-inspector-and-tests.md)
- [task-80: proof of qualitatively different software](../../../Tasks/task-80-proof-qualitatively-different-software/task-80-proof-qualitatively-different-software.md)
- [task-64 phase c: session to test, and export](../../../Tasks/task-64-session-inspector-and-tests/phase-c-session-to-test.md)
- [task-80, phase 1: probe, five-demo method, demos 3 and 4 and handoff](../../../Tasks/task-80-proof-qualitatively-different-software/phase-1-probe-and-demos.md)
- [plev: thesis of the generative interaction runtime](../../../Research/Synthesis/generative-runtime-thesis.md)
- [program: from GPU UI engine to generative interaction runtime](../../../Research/Synthesis/generative-runtime-program.md)
<!-- /related -->
