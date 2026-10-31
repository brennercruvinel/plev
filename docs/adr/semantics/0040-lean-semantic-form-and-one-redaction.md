---
id: adr-0040
title: "the semantic form is lean, and one rule redacts a secret"
project: plev
type: adr
status: accepted
date: 2026-09-23
date_basis: header
recorded: 2026-09-23
topics: [semantics, agents, formats]
relations: [adr-0039, task-61, task-65, task-83]
source: docs/adr/semantics/0040-lean-semantic-form-and-one-redaction.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# the semantic form is lean, and one rule redacts a secret

Decides that the serde form of the `rust/accord` crate (then `sem`) writes no field with a default value, and that the rule keeping a secret inside the process exists only once, in the `redactor` module of that crate. Measured over the whole showcase, a full observation dropped from 6419 to 2039 bytes with three items, and from 87417 to 26237 with 103, without any revision changing. `source` stays in the form when the node has an origin, but it leaves the agent's response only when the agent asks for it.

## context

task-61 measured what an agent receives when it observes the showcase's
todo page: 6419 bytes for three items. about 28 percent of that was
`source`, the file, line and column of the call that projected each
node, and about 40 percent more was fields written at their default
(`"description": null`, `"enabled": true`, a `constraints` object with
seven default fields, a `policy` naming four actors who may all ask).
none of it carries meaning to an agent that acts by id; all of it costs
tokens on every observation ([`0039-agents-act-through-the-semantic-contract.md`](0039-agents-act-through-the-semantic-contract.md),
consequences).

the same task left the rule that keeps a secret inside the process
written twice, once in `rust/socket/src/redact.rs` and once in
`rust/record/src/redact.rs`, independently. the two already differed in
one point: the port redacted a stale delta only on the nodes of the
request, the recorder on any node its last snapshot marked secret. a
security rule with two copies diverges; this one had.

## decision

the serde form of `sem` is lean. a field at its default is not written
(`skip_serializing_if`), and a field left out reads as its default
(`#[serde(default)]`), field by field on `Node`, `ActionDecl`, `Policy`,
`Constraints`, the snapshot's `focused` and a request's `param`. `id`,
`role` and `label` are always written. the revision is computed from the
tree in memory and never from the text, so no form changes a revision.
the form before this decision, every default spelled out and
`"source": null`, still reads, and is kept as a test fixture; the
session file format (`MONS`) does not change version, its semantic track
being JSON lines of this form.

`source` stays in the form whenever a node has one, because the recorder
needs it to go from a replay back to the code (task-65). it leaves the
agent port's `snapshot` reply unless the agent asks: `observe {"source":
true}` on the protocol, `plev observe --source` on the command line. an
agent acts by id, and the place in the code that projected a control is
not part of what the control means. a `delta` never carried `source`: a
`Change` names ids and values only. (updated on 2026-09-23: since
task-83 a `node_added` carries the whole node, and the port serves it as
a snapshot serves a node, `source` only when asked; adr
a-delta-carries-the-node-a-refusal-names-who-acted.)

the redaction rule lives once, in `sem::redact`, because `sem` owns what
`secret` means and is the one crate the port and the recorder share.
`REDACTED` is the text that replaces a free text about a secret target;
`is_secret(state, id)` is true when the snapshot marks the node secret or
does not hold it; `length(v)` is what the snapshot keeps of a secret
value; `Secrets::of(request, state)` captures, from a snapshot at one
instant, the nodes that matter to one request and redacts its request
and every stage of its receipt the same way. `Secrets::holds` is the
union of the two rules that existed: the request's own nodes (target and
observed) that are secret or unknown, plus every node the snapshot marks
secret. it is at least as closed as either copy, identical to both in
every case the tests reach, and fails closed with no snapshot.

## consequences

- measured on the showcase's todo, bytes of the line the server writes:
  a whole observation went from 6419 to 2039 bytes with three items and
  from 87417 to 26237 with 103; one item by scope from 929 to 361; a
  delta after one toggle stays at 624, since it never carried what left.
  with `source` asked for, 2939 bytes for three items.
- the `wire` and `rec` crates hold no redaction code; both call
  `sem::redact`. `wire::REDACTED` is gone; its one reader now reads
  `sem::redact::REDACTED`.
- `Secrets::of` walks the snapshot's nodes once per capture (the port
  captures at dispatch, the recorder per stage) to collect the marked
  ones; the cost is linear in the snapshot and negligible at the sizes an
  app projects. a projection with thousands of secret fields would pay
  it; none exists.
- a JSON producer outside this repository that wrote the old form keeps
  working; one that reads must accept absent fields as defaults. the
  pinned forms in `rust/accord/tests/encoding.rs` say exactly what is written.
- the proto's `SnapshotReply.focused` is still written as `null` when
  there is no focus (15 bytes, outside the sem form); a v2 of the
  protocol may drop it.
