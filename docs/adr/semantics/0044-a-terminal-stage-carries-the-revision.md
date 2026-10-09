---
type: adr
status: accepted
tags: [wire, agents, protocol, revision, ring, receipts, stages, generative-runtime]
date: 2026-09-23
---

# a terminal stage carries the revision the app is at

## context

in the second round of checkpoint a (task-62, "resultado da segunda
rodada"), the three LLM agents said the same thing: the `completed` of an
act does not say at which revision the app is now, so to chain one act on
the next an agent observes only to learn that revision. it was the
largest cost in round trips left, and round trips are where plev loses to
a direct `git` (H4). the scripted agent of the laboratory
(`apps/agenteval/src/arms/scripted.rs`) did the same.

the app already knew the answer. an observation serves a revision by
projecting the app (`Host::project`) and holding the whole snapshot in
the ring (`Ring::serve`), and `since` and `from_revision` take only a
revision the ring holds. what was missing was a stage that went through
that path when it ended.

## decision

`StageReply` gains `revision: Option<Revision>`, written only when set
(`skip_serializing_if`) and read as absent when missing (`default`).

- what it means: the revision the app is at when the terminal stage goes
  out; it includes what other actors changed until then; it serves
  `since` and `from_revision` (on the command line, `--since` and
  `--from-revision`). the same words are in the doc of `StageReply` and
  in `plev --help`.
- where: on a terminal stage (`completed`, `rejected`, `failed`), and on
  no other. absent when the app projects nothing at that instant.
- how: `Server::forward(&mut self, host: &mut dyn Host)` projects the
  app through the host and serves the snapshot into the ring, the path
  `observe` takes (`host.project()`, `self.ring.serve(live)`). once per
  call, and only when a terminal stage of a request this port issued
  goes out: nothing runs on the app's thread between two stages of one
  call, so they all leave it at the same revision. both call sites pass
  the host they already had, the end of `Server::handle` and the end of
  `AgentPort::drain`; an app that drains the port changes nothing.
- it decides nothing. an act resting on it is checked on its reads like
  any act (`docs/adr/semantics/0033-actions-carry-observed-state.md`), and a stale
  refusal still says what moved and who acted.
- it is not an observation. the agent is given the name of a state, not
  the state: `Traffic::observes_snapshot` and `observes_delta` do not
  count it, and `Host::observed` is not called for it, so a recorder that
  writes what an agent observed (`rec::Kind::Observed`) does not write
  it.
- a secret stays inside: the revision is a hash of the projection, where
  a secret field is already its length, and a stage aimed at a secret is
  redacted as before.
- protocol v1 keeps its version: the field is additive, a client that
  does not know it ignores it, and a stage written before it reads as one
  with none.

## consequences

- a terminal stage costs one projection of the app on its thread, and a
  hash of it. measured in the laboratory (`ide-lab --arm scripted`, the
  ide's workbench, debug build, five runs of each scenario): a median of
  158 us with nobody intervening and 172 us with the person of the
  script, 306 us at most, against the 2 ms the task set.
- the scripted agent of the laboratory chains the acts it has nothing to
  decide between on the revision the last stage carries. with nobody
  intervening it sends 12 messages instead of 16 and receives 28.8 KB
  instead of 54.5 KB; with the person of the script, 17 instead of 24 and
  45.9 KB instead of 91.5 KB; the stale refusal, the recovery and the
  commit are the same. the table is in
  `docs/log/steps/task-87-a-terminal-stage-carries-the-revision.md`.
- the ring is shared by every connection of a port and bounded (16 by
  default). a revision served in a stage takes a place in it as an
  observation does, so a run of acts now pushes out old revisions that
  before only observations pushed out: an agent that holds a revision it
  observed and then lets 16 other revisions be served (its own acts or
  another agent's) gets `unknown_revision` and observes again. a stage
  that ends at a revision the ring already holds only makes it the
  newest.
- the revision is exactly what an observation at that instant would
  serve, no more. where an app's view is still settling when the stage
  goes out, the revision is of the view as it stands. in the ide
  (`apps/idestudio/src/capability`), a `show_diff` completes before the
  worker sends its lines (no action reads them; `since` that revision
  brings them), and a stage or unstage that failed at git leaves the
  view's guess until the refresh the failure sends lands (a commit
  resting on it is refused at the index, the point of truth). the
  showcase's todo runs every action on the spot, and the effect of a
  completed act is always in its revision.
- a replay or a report that tells which revision an act rested on by
  looking at the observations served (the laboratory's `story::acts`,
  its `form` and `served_index`) does not see a revision a stage
  carried: it has to hear the stages too.

## avoid

- a second path to serve a revision: a terminal stage goes through
  `host.project()` and `Ring::serve`, as `observe` does.
- one projection per stage when several end together.
- deciding stale by this revision, or anything else by it.
- counting it as an observation, or reporting it to `Host::observed`.
