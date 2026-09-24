---
type: adr
status: accepted
tags: [wire, agents, protocol, socket, sem, dispatcher, one-path, security, cli, showcase, generative-runtime]
date: 2026-09-23
---

# agents act through the semantic contract

## context

nothing reached a plev app from outside its process. an agent that wanted
to operate one had the screen: a screenshot to read and a coordinate to
click, the representation the thesis says an agent should not need
(section 25), with no way to know that what it saw still held when its
click landed (section 8).

the contract that makes the screen unnecessary already existed. every
control has an identity, a role, a state and the actions it accepts, as a
projection of the app (`docs/ADR/semantic-contract-as-pure-projection.md`);
every action goes through one `Dispatcher` that checks the state its actor
observed (`docs/ADR/actions-carry-observed-state.md`); the compositor knows
where each id sits in the frame on screen
(`docs/ADR/semantic-binding-beside-the-scene.md`); and the showcase shell
already routes one actor that is not the page's own input, a screen
reader, through the page's hooks and an `EventLoopProxy`
(`docs/ADR/accessibility-projects-the-semantic-snapshot.md`). what was
missing was a door: a channel from a process outside, carrying the
contract and nothing else.

## decision

a new crate, `crates/wire`. its protocol types, codec, ring and server
core are pure (they compile for wasm); the socket (`AgentPort`), the client
and the `plev` binary are unix only.

### protocol v1

JSON lines. every message is one object on one line with `"v": 1`, an
`"id"` the client picks (a string) and a `"type"`. every reply carries the
id of the message it answers; an error about a line whose id could not be
read has a null id. snapshots, changes, requests, observations and stages
travel in the serde form `sem` pins (`crates/sem/tests/wire.rs`), so the
protocol adds an envelope and nothing else.

- `hello {name}` names the actor of the connection, `Actor::Agent {
  name }`, and gets `welcome {app, actor}`. an `act` before it is refused.
- `observe {scope?, keys?, since?, bounds?, source?}` gets `snapshot
  {revision, nodes, focused, bounds?}` or `delta {base, revision,
  changes, bounds?}`.
  `scope` keeps one node and its descendants (`Snapshot::subtree`); `keys`
  keeps only those state keys (a node keeps its other fields, a delta
  loses the changes of other keys); `since` answers the `Snapshot::diff`
  from a revision the app served, when it still holds it, and the whole
  snapshot otherwise; `bounds` adds the visible rect of each bound node of
  the frame on screen, and is off by default, since an agent acts by id.
  `source` keeps each node's `source` in a snapshot, off by default too
  (task-57b, see the consequences). `revision` is always the revision of
  the app's whole snapshot at that instant, also for a scoped
  observation: it is what `since` and `from_revision` take.
- `act {target, action, param?, observed}` or `act {target, action,
  param?, from_revision}`, exactly one of the two. the reply is one `stage
  {request, stage, at}` message per stage of the receipt, in order, under
  the act's id, until a terminal one (rejected, completed, failed); a stage
  that ends later (an execution that starts now and ends on a worker)
  comes later, on the same connection. the first, `requested`, also
  carries `asked`: the request as the dispatcher took it, with the
  observed values the app rebuilt.
- `confirm {request}` and `decline {request}` exist so the protocol is
  complete, and are always refused on a socket: only a person answers a
  confirmation, at the screen or through assistive technology, and the
  actor of a connection is always an agent. an agent never confirms its
  own action.
- typed errors, `{"type":"error","error":<kind>,...}`: `bad_version {got}`
  (checked before the shape, so a later version is never a
  `bad_message`), `bad_message {reason}` (not JSON, not an object, no
  string id, an unknown type, a field that does not parse, an act with
  both or neither of `observed` and `from_revision`, an action the served
  revision does not offer), `unknown_revision {revision}`, `not_enabled
  {reason}` (a confirmation by an agent, an act before hello, nothing
  projected, no dispatcher). a refused action is not one of these: it is
  a `stage` with `rejected`, from the dispatcher. added by task-89,
  still v1: `not_observed {reads}`, an act on `from_revision` that reads
  state its actor was never shown (see the ring below).

### the ring of served revisions

the port keeps the whole snapshots it served, the last N (16 by default,
configurable), keyed by revision. `since` rebuilds a delta from one;
`from_revision` rebuilds the observed values of an act from one with
`Request::observe`, so the stale check compares what the agent saw, not a
later frame. a revision out of the ring answers `unknown_revision` and the
agent observes again. serving a held revision again makes it the newest.

corrected on 2026-09-23 (task-89): the code did not keep the promise
above. it rebuilt the observed values from the whole snapshot of the
revision, out of the scope and the keys the agent observed too, so an act
could say it saw state that was never shown to it. the session inspector
(task-64 phase a) found it in the session of A2 of checkpoint a: a submit
resting on an observation of `commit.form` only. the rule now: the port
remembers, per actor (the name of `hello`, over every connection under
it), the revisions it gave it and what each one showed; an act on
`from_revision r` needs `r` given to its actor, takes the action from
`r`, and takes the value of each read from the last observation up to `r`
that showed it to that actor. a read never shown is refused before the
dispatcher with `not_observed`. a terminal stage's revision shows no
state, and a delta shows only what it carries. the rule, its bounds and
the name as the agent: `docs/ADR/an-act-rests-on-what-its-actor-was-shown.md`.

### one path, through the host

the port reaches the app only through `Host`: `project` (the live
projection of what the app means now), `bounds` (the frame's bindings),
`dispatch` (a `Request` through the app's one dispatcher, stamped with
the dispatcher's own clock, the one a click is stamped with) and
`add_sink` (a `ReceiptSink` on that dispatcher). an act is a
`sem::Request` with `Actor::Agent`, through the same `dispatch` a person's
click goes through; the policy (`Actors.agent`, `Confirm::ForAgents`), the
stale check, inert and disabled apply as they apply to anyone. the sink
hears every stage of every agent request and the port forwards only the
ones it issued, to the connection that asked; a person's receipts are
never forwarded. `drain(host)` takes no clock: the port stamps nothing.

### threads

one thread accepts; per connection one thread reads, decodes and queues,
one writes. they never touch the app: a message goes into a bounded queue
and the thread wakes the loop through the closure the shell passed (its
`EventLoopProxy`, the pattern of
`docs/ADR/worker-results-wake-the-event-loop.md`); wakes coalesce until
the next drain. `drain` runs on the app's thread and does every
projection and every dispatch there (the ide's `Workbench` is not
`Send`). the app never waits on a socket: a client that does not read its
replies is cut off (its queue of 1024 lines is full, or one write waits
10 s), one drain serves at most 64 messages and wakes the loop again for
the rest, a line holds at most 1 MiB (past it the connection ends, after
its error), a port holds at most 16 connections. a line that does not
decode answers its error from the reader's thread and the connection
goes on. when a client stops sending, or breaks the line limit, the
replies already queued for it still go out and its writer ends the
connection after the last one; the app's thread never shuts it down
under the writer, which dropped them now and then (a closed client or an
over-long line got the end without its last replies).

### security posture

- off by default. `AgentPort::from_env` opens only when `PLEV_AGENT=1`
  and the build has debug assertions; a release build opens a port only
  when the app opts in and calls `AgentPort::open` itself. the showcase
  opens it on desktop only, never in wasm or on mobile. a test proves no
  socket and no directory exist without the variable (a child process of
  the test binary reads the real environment).
- local only: a unix socket at `$TMPDIR/plev/<app>-<pid>.sock`. the
  directory is made 0700 (a symlink, a file or a directory this user
  cannot chmod is refused), the socket is 0600, and it is removed when
  the port drops. no network, no remote pairing (a remote scene is
  task-79).
- no arbitrary access: there is no message to run code, read a file or
  call a backend. an agent can observe what the app projects and ask for
  the actions the projection declares, under the policy the app gave
  them, and nothing else.
- secrets never leave the process. a snapshot already holds only the
  length of a `secret` field; a stage message reduces everything aimed at
  a secret node the same way (a parameter and an observed value to their
  length, an outcome's values to their length, a stale delta to lengths
  without items, an error or a refusal reason to a fixed text). which
  nodes are secret is read from the live snapshot at dispatch, and a node
  it does not hold counts as secret.
- the actor's name is declared, not authenticated. any process of the
  same user can connect and call itself anything; the boundary is the
  user account, as for any file in `$TMPDIR`. agent authentication and
  finer permissions are future work. since task-89 the name also keys
  what the port showed: two processes under one name are one actor for
  the contract (receipts, `acted`, what was shown). the default name of
  `plev` is `plev`, so distinct agents pass `--name` or
  `PLEV_AGENT_NAME`.

### the `plev` binary

`plev observe [--scope <id>] [--keys <k,k>] [--since <rev>] [--bounds]
[--source]` and `plev act <target> <action> [--from-revision <rev>] [--param <json>]`,
with `--app <name or socket path>` (without it, the one live app in
`$TMPDIR/plev`; a socket nobody answers on does not count), `--name`,
`--timeout`, `--pretty`. every message the app answers is printed as it
arrives, one JSON line each. the exit code says how it ended: 0 observed
or completed, 1 rejected, 2 failed, 3 error (the protocol, the
connection, the arguments). `act` without `--from-revision` observes the
target first and acts on that revision, the state as it is now, so the
check still compares what was seen. `--param` takes a sem value or a
plain JSON string, bool, number or list of strings.

updated on 2026-09-23 (task-89): that look at the target counts as shown
to the name, and what the act reads outside the target rests on the
earlier observations under that name, or is `not_observed`, which exits
3 like `unknown_revision`.

### the showcase

the desktop shell opens the port beside the accessibility bridge (both in
`Access`, the window's outside actors). the port wakes the loop with
`UserEvent::Agent`; the shell drains it through
`ShowcaseView::serve_agent` and draws only when an act was dispatched,
since an observation changes nothing. the view serves the port through
the hooks a screen reader already goes through: `semantics()` projects
the active page, `act` (renamed from `assist`, since it serves any actor
that is not the page's own input) runs a request through the page's
dispatcher with the page inert while an overlay is up, and `add_sink`
puts the port's sink on that dispatcher. the bounds are the visible part
of each binding of the frame on screen. only the todo page projects and
dispatches today.

## measured

apple m4, rustc 1.98.1, the showcase's todo page served over a real unix
socket with no window (`agent_port_cost` in
`crates/showcase/src/view/dispatch.rs`, release, best of 50 acts, three
runs), and the windowed showcase in a debug build driven by `plev` and by
a client holding one connection.

| what | 3 items (11 nodes) | 103 items (111 nodes) |
| --- | --- | --- |
| whole observation | 6419 B | 87417 B |
| the same without `source` | 4599 B | 65097 B |
| without `source` and default fields | 2532 B | 38245 B |
| one row (`--scope todo.item[1]`) | 929 B | 929 B |
| whole, keys cut to `completed` | 6281 B | 87277 B |
| delta after one toggle (`since`) | 624 B | 632 B |
| act round trip, in process, release | 32 to 35 us | 175 to 178 us |
| act round trip, in process, debug | 146 us | 1072 us |

against the windowed debug showcase: a hello round trip of 0.2 to 0.7 ms;
an act to its last stage in 1.0 ms at best, 8 ms at the median, since a
message waits behind the frames the row's strike animation draws in a
debug build; a whole `plev act` (process start, connect, hello, the
observation of the target, the act) in 23 ms at best, 25 ms at the median.
the two lean rows are not a form the protocol has: they weigh the same
reply with each node's `source` removed, then also every field at its
default (null, false, empty, the default policy).

JSON lines stay. a whole page costs about 1600 tokens with 3 items and
20000 with 103, and the protocol is built so an agent rarely asks for a
whole page: a row is under 1 KB, a delta after a change under 1 KB. keys
cut little, since the bulk of a node is its actions, constraints and
source, not its state.

updated on 2026-09-23: the table and the token counts above measure the
verbose form this task shipped, and stay as its record. the lean form of
task-57b replaced it (see the consequences): a whole observation is now
2039 B with 3 items and 26237 B with 103, one row 361 B, a delta after
one toggle still 624 B; the new numbers are in
`docs/ADR/lean-semantic-form-and-one-redaction.md`, consequences.

## reason

- the agent works on the structure the engine uses (thesis, section 2),
  not on pixels: identity, state, actions and their preconditions are the
  whole interface, and a stale world is a typed rejection with the delta,
  not a misplaced click.
- one path keeps one record: an agent's request is a receipt in the same
  dispatcher, next to the person's clicks and the screen reader's
  requests, with its actor named.
- a scoped, incremental observation spends little data and few tokens,
  and a revision in the ring lets the agent act on exactly what it saw
  without sending it back.
- the app's thread owns the app: threads that only decode and queue
  cannot race it, block it or crash it.
- JSON because an agent reads it; the cost is measured above and a
  leaner v2 is a decision with data.

## consequences

- the ide's `Workbench` is served as the host by the `ide-lab` binary
  (task-62 phase c, `crates/ide/src/bin/lab/lab.rs`), not by the ide's
  own shell as planned here. its executions end on the git worker, so the
  lab drains the port also after the worker's wake, for the late stages
  to go out. updated on 2026-09-23 (task-86): the `ide` binary's window
  opens the port too (`PLEV_AGENT=1`, debug) through `ide::agent`, a
  `Host` over the `Workbench` and the frame's bindings; the shell drains
  after the port's wake and after every git result it applies, so late
  stages go out, and draws only when an act ran. the visible part of a
  binding is `Binding::visible_rect` in the engine, read by both hosts.
  proved in a real window with a person's clicks and an agent at once
  (`docs/mission/steps/checked/task-86-the-ide-opens-the-agent-port-done.md`).
  updated on 2026-09-24 (task-92): a stage of an agent's request that
  ends outside a drain (a person's answer to a confirmation, the second
  check refusing, a worker's answer) wakes the port through its sink, so
  every host that drains on the port's wake sends it; before, it waited
  for the agent's next message and was lost when a `plev act` gave up
  (P0). an agent may also ask a person to confirm an act first
  (`Act.ask`, `plev act --ask`, adr actions-carry-observed-state).
- a host with more than one dispatcher gets one sink; the showcase has
  one today (the todo page). request ids are per dispatcher, so a host
  must route the port to one.
- the serde form was verbose for a node at rest (`source` about 28
  percent of a whole observation, fields at their default about 40 more).
  task-57b made the sem form lean and `source` opt-in on the port
  (`observe {"source": true}`, `plev observe --source`), still protocol
  v1: `docs/ADR/lean-semantic-form-and-one-redaction.md`.
- the redaction rule of a secret was written twice, in `rec` and in
  `wire`, and had diverged; it lives once now, in `sem::redact` (same
  ADR).
- task-83, still protocol v1: a delta's `node_added` carries the node,
  served as a snapshot's, a stale refusal names in `acted` who ran
  something on the state the agent saw, and the stale rule is in `plev
  --help` and the doc of `Act`:
  `docs/ADR/a-delta-carries-the-node-a-refusal-names-who-acted.md`.
- task-87, still protocol v1: a terminal stage (`completed`, `rejected`,
  `failed`) carries `revision`, the revision the app is at when that
  stage goes out. it includes what other actors changed until then, and
  it serves `since` and `from_revision` (`--since`, `--from-revision`),
  so the next act needs no observation only to learn it. `forward`
  projects through the `Host` and serves it into the ring as `observe`
  does, once per call; it decides nothing and is not an observation:
  `docs/ADR/a-terminal-stage-carries-the-revision.md`.
- task-89, still protocol v1: an act on `from_revision` rests on what its
  actor was shown, never on the whole snapshot of the revision; a read
  never shown is `not_observed`:
  `docs/ADR/an-act-rests-on-what-its-actor-was-shown.md`.
- a process killed by a signal leaves its socket file behind. `plev`
  ignores a socket nobody answers on, and the next process with that pid
  replaces it; nothing else sweeps them.
- an app that spawns children can leak the port's descriptors into
  them. on macOS std sets `FD_CLOEXEC` on a socket after creating it
  (there is no `SOCK_CLOEXEC`), so a child spawned from another thread in
  between (the ide and the showcase's Git page run `git` on their worker)
  inherits the listener, or a connection the port just accepted, for as
  long as it lives. while the app lives nothing shows: the child never
  accepts, and the app's accepting thread takes every connection. when
  the port drops, the socket file goes and nobody can reach the leaked
  listener. but when the app ends without dropping the port (a signal, an
  abort) while such a child lives, the file stays and the child's copy
  still listens: a connect succeeds, so `find` counts the socket as live
  (it returns it alone, or reports `Ambiguous` next to another instance
  of the app), the client's hello is queued and never answered, and
  `Client` returns `Timeout` (`plev` exits 3 after its 30 s default). a
  leaked connection keeps that client from seeing the end of the
  connection until the child exits; it waits the same way. once the child
  exits the socket is dead again: a connect is refused and `find` ignores
  it. not fixed here: a `find` that asks for a hello answered within a
  short timeout would tell such a socket from a live app, and spawning
  children with every other descriptor closed would stop the leak. the
  wire tests that spawn processes live in their own binaries, since the
  same leak made a dropped listener answer in a test.
- `confirm` and `decline` never reach the dispatcher from a socket: the
  refusal is decided by the connection's actor, which is always an agent.
  a person who wants to answer from outside the window is future work,
  with authentication.

## avoid

- a second way to run an action from outside: a message that reaches the
  effect without `dispatch`, or that runs code, reads a file or calls a
  backend.
- projecting or dispatching on a socket thread.
- the app's thread waiting on a socket.
- a secret, or anything that quotes one, in a message.
- an observation that invalidates a frame: only an act that ran may
  change what is drawn.
- a revision used to decide stale: the ring serves the observed values,
  the dispatcher compares the reads.
