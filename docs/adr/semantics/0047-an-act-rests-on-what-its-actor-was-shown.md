---
type: adr
status: accepted
tags: [wire, agents, protocol, revision, ring, stale, cli, security, generative-runtime]
date: 2026-09-23
---

# an act rests on what its actor was shown

## context

`docs/ADR/agents-act-through-the-semantic-contract.md` promised that
`from_revision` rebuilds the observed values of an act "so the stale check
compares what the agent saw, not a later frame". the code did not keep it.
the port held the whole snapshot of every revision it served and built the
act's `observed` with `Request::observe` over the whole snapshot of the
revision named, out of the scope and the keys the agent had asked for too.
since task-87 a terminal stage also serves a revision, which shows no
state at all, and an act on it read the whole snapshot the same way.

the session inspector (task-64 phase a) found it reading the session of A2
in the first round of checkpoint a: the submit rested on an observation of
`commit.form` only, and the `changes:staged` it "observed" was never served
in it. the case the contract exists to stop gets through that door: the
agent sees A, B and C staged, a person unstages C, the agent observes only
the form and submits on the new revision; the request says it saw {A, B},
the check passes, the point of truth agrees, and two files are committed
by an agent that decided on three.

## decision

an act on `from_revision r` rests on what its actor was shown, and on
nothing else.

- the actor is the name declared in `hello`, the identity receipts,
  `acted` (task-83) and `Host::observed` already use. two connections
  under one name, at once or one after the other, are one actor: the
  `plev` command line opens one connection per command, and an agent
  observes in one and acts in the next. `Server::close` forgets the
  connection, not what its actor was shown.
- the port remembers, per actor, the revisions it gave it and what came
  with each (`Shown`, `crates/wire/src/shown.rs`): a snapshot shows the
  nodes under its `scope` (every node without one) with the state under
  its `keys`; a delta shows only what it carries, a state that changed and
  the state under its `keys` of a node added or removed, so a state of its
  cut that did not move is not shown again; a terminal stage gives a
  revision and shows no state. the values stay in the ring, which already
  holds the whole snapshot of each revision served.
- `r` must have been given to the actor and still be held, or the act is
  `unknown_revision`, as before. the action and its `reads` come from the
  snapshot of `r`, as before.
- the value of each read comes from the last part, up to the last time
  the actor was given `r`, that showed it, read in the snapshot of that
  part's revision. a read no part showed, or whose last showing left the
  ring, refuses the act before the dispatcher with a new protocol error,
  `not_observed {reads}`: the refs, never a value, in the action's order.
  an older showing the ring still holds is not used in place of a newer
  one that left it: it is not what the agent saw last. a scoped snapshot
  whose revision left the ring can no longer say which nodes it held, so
  a read it may have shown counts as not shown too.
- the request's `Observation::revision` stays `r`, to trace it. it
  decides nothing, as before.
- `observed` given explicitly is the agent's claim and is taken as given:
  the dispatcher checks it against the live state like any act, and the
  port does not ask whether it was shown.
- bounded like the ring: the last `ring.cap()` revisions given to each
  actor (16 by default), and at most `MAX_ACTORS` (16) actors, the one
  served longest ago forgotten first. whatever was forgotten counts as
  never shown: its acts are refused (`unknown_revision`, `not_observed`),
  never admitted.
- `plev act` without `--from-revision` looks at the target first (an
  observation scoped to it) and acts on that revision. that look counts as
  shown to the name: the agent chose to act on the target as it is now.
  what the act reads outside the target rests on the earlier observations
  under that name, or is `not_observed`. `plev` exits 3 on it, as on
  `unknown_revision`: a refusal of the protocol before the dispatcher,
  and the same remedy, observe and decide again.
- protocol v1 keeps its version. `not_observed` is additive, as the
  revision of task-87 was; its form is pinned in
  `crates/wire/tests/codec.rs`. the rule is in the doc of `Act`
  (`crates/wire/src/proto.rs`) and in `plev act --help`.

## the name is the agent

two processes under one name are one actor for the contract: their
receipts, the `acted` of a refusal and what they were shown are one. the
default name of the command line is `plev`, so distinct agents pass
`--name` or `PLEV_AGENT_NAME`, or they rest on each other's observations.
the name is declared, not authenticated. the boundary stays the one the
port already has: a local socket of the same user, off unless opted in,
and opened by `from_env` only in a debug build
(`docs/ADR/agents-act-through-the-semantic-contract.md`, security
posture).

## consequences

- an agent that chains acts on stage revisions keeps doing it when each
  act reads state none of its own acts moved since it observed it. an act
  that reads what the agent's own act just moved, with no observation in
  between, is stale: it rests on what the agent saw before its own act.
  the refusal's `acted` is empty then, since the only actor on that state
  was the agent itself; `changes` says what moved. that is the
  consequence the task accepted: an agent acts on what it saw, and
  observes what it has not seen.
- `acted` names who acted on the revision the act names. when a read
  rests on an older observation and someone moved it before that
  revision, `changes` says what moved and `acted` names nobody for it.
- updated on 2026-09-24 (task-91): `Shown` keeps, with each revision
  given, the dispatcher's mark at that instant, and an act rests on the
  oldest observation its reads come from (with `observed` given, on the
  last giving of its revision); `acted` names what ran after that look,
  the agent's own acts included, so the two consequences above about an
  `acted` that names nobody hold only when there is no look
  (`docs/ADR/a-delta-carries-the-node-a-refusal-names-who-acted.md`).
- the scripted agent of the laboratory needed no extra observation: it
  observes the whole workspace before each decision, and each act it
  chains reads the state of its own file, which no act of its own moved.
  five runs of each scenario (`ide-lab --arm scripted`, debug build) all
  exit 0 with MATCH, with the messages of task-87: 12 in and 28.8 KB out
  with nobody intervening, 17 in and 45.9 KB out with the person of the
  script, who makes the first submit stale.
- a revision given to another actor is `unknown_revision` for this one.
  an observation served before `hello` belongs to no actor and counts for
  none.
- the memory is bounded by the caps above; the port keeps a scope, keys
  and the refs a delta carried per revision given, never a value. the
  cost at an act is one walk back over at most `cap` revisions per read,
  with a lookup in the ring and, for a scoped snapshot, a walk up the
  node's parents. it is not measured here.
- `docs/ADR/a-terminal-stage-carries-the-revision.md` says the revision a
  stage carries serves `from_revision`: it still does, and it shows no
  state, so an act on it reads what the agent was shown before it.

## avoid

- building an act's observed values from a snapshot the agent was not
  shown: the whole snapshot of a revision, a stage's revision, a delta's
  base.
- keying what was shown by connection: the command line opens one per
  command.
- admitting what was forgotten: past the caps, an act is refused.
- a value in `not_observed`: it names refs.
- deciding stale by the revision; it still only traces.
