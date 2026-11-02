---
id: adr-0036
title: "a commit checks the index it observed"
project: plev
type: adr
status: accepted
date: 2026-09-23
date_basis: header
recorded: 2026-09-23
topics: [semantics, git, agents]
relations: [adr-0033, adr-0035, task-58, task-58b, task-59, task-60, task-61, task-69, task-84, task-85, learning-cycle-0036]
source: docs/adr/semantics/0036-commit-checks-the-index-it-observed.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# a commit checks the index it observed

Decides that a commit from the ide checks the index that a person or an agent observed before running `git commit`: the request carries the set of files seen as staged (`expect_staged`), and the git worker compares that set with the real `status`, refusing with `CommitRejected` when they differ. A single `Workbench` brings together the views, the git session and the `Dispatcher`, and each capability has a single handler. The receipt ends at the point of truth; the window that remains between reading the index and `git commit` is outside the contract, as the decision itself records.

## context

the ide committed whatever the index held when the git worker ran: a
click on Commit pushed `UiRequest::Commit { message }`, the session sent
`GitCommand::Commit`, the worker ran `git commit`. nothing compared the
index with what the person, or an agent, saw when they decided. the view
was optimistic (a stage flipped the row at once, a refresh reconciled
later), so for a moment the screen and the index could disagree, and a
terminal, an editor, the file watcher, the person and an agent all move
the index.

task-58 gave the rule ([`0033-actions-carry-observed-state.md`](../0033-actions-carry-observed-state/0033-actions-carry-observed-state.md)): a
request carries the values its actor observed for the state the action
reads, the dispatcher checks them against the live projection, and when
the truth lives outside the ui process the point of truth checks them
again right before the effect. task-59 gave the binding table and the
click as a request ([`0035-semantic-binding-beside-the-scene.md`](../0035-semantic-binding-beside-the-scene/0035-semantic-binding-beside-the-scene.md)). this
record applies both to the git client.

## decision

### one workbench

`apps/idestudio/src/capability` is the git workspace as a capability. the
`Workbench` owns the views (`WorkspaceView`), the git side (`GitSession`)
and one `sem::Dispatcher`; it implements `Semantics` (`project.rs`) and
`Operable` (`execute.rs`, one handler per capability). the ide binary and
the showcase Git page hold a `Workbench`; neither wires the pieces itself.

the model: `changes` (`files`, `staged`, `selected`),
`changes.file[<path>]` (`status`, `staged`, `unstaged`; `stage`,
`unstage`, `discard`, `ignore` reading the file's `staged` and
`unstaged`, `show_diff` reading `diff:target`), `diff` (`target`, the
first 200 `lines`, `count`), `commit.form` (`open`; `open`, `cancel`
reading `open` and the message it clears), `commit.message` (`text`,
required; `set_value`), `commit.submit` (`submit` reading
`changes:staged` and `commit.message:text`, nothing else), `stacks`
(`branch`) with `stacks.commit[<sha>]` (`summary`, `author`,
`show_diff`), and `notices` (the toasts). a path with staged and
unstaged changes is two rows in the list and one node. the list cursor
is projected as `changes:selected`: navigation, like focus, moved by the
person outside `execute` and read by no action, so it moves the revision
and never makes a commit stale.

### the person asks, execute acts

a click or a key runs the view first (hit test, cursor, menus, focus,
typing). what acts on a capability is queued as a `UiRequest` naming it
without ids; the workbench observes the workspace at that instant and
dispatches it as `Actor::Human`. the optimistic updates (a row staged, a
row removed, the form opened or cleared) happen in `execute`, after the
dispatcher accepted the request. the keymap still names input actions
(`commit::Submit`, `file::Stage`); the ones that act on a capability go
through the workbench, in one place. a keymap entry naming a capability
directly (`commit.submit/submit`) was left out: most bindings act on the
row under the cursor, which is not a fixed id.

### the second check, at the index

`rust/gitdrv` replaces `GitCommand::Commit` with `GitCommand::CommitIf {
message, expect_staged }`. the worker reads the real index through
`status` (the same source the list shows), compares its staged set with
`expect_staged`, and only then runs `git commit`; when they differ it
answers `GitEvent::CommitRejected { added, removed }` and commits
nothing. `GitEvent::Committed` carries `NewCommit { sha, files }`: the
paths the commit changes against its first parent (a root commit against
the empty tree), a rename at both paths, read in process by the same
function the log uses (`git diff-tree -r --no-renames --root
--diff-merges=first-parent` by the CLI; task-85, which replaced `git show
--name-only`). the workbench shows the commit (its files out of the
staged rows, the commit on top of the stack) in the step that completes
the receipt, before it; `stacks.commit[<sha>]` carries `files` on the ten
newest commits and on the last one the workbench made. `expect_staged` is the
`changes:staged` value the actor observed, which the dispatcher found
still projected.

### the receipt ends at the point of truth

everything `execute` sends to the worker returns `Execution::Started`;
the workbench keeps the `RequestId` it received with what the request
waits for. the worker runs and answers commands in the order they were
sent, so an answer belongs to the oldest request waiting for its kind and
path. `Committed` completes the receipt with `commit` and `files`; an
error fails it; `CommitRejected` ends it with `reject_at_truth` and a
`Rejection::Stale` holding one `Change::StateChanged` on `changes:staged`
(`before` the expected set, `after` the set the index held). the commit
form keeps its message until the commit exists, so a commit refused at
the index can be asked again as it is.

### what the person sees

a refusal (at the dispatcher or at the index) and a failure show as a
comps toast that says what changed or why ("Refused test-agent's submit:
B left the staged files"). a request that waits for a person
(`discard` is destructive with `Confirm::ForAgents`) opens a comps modal
naming who asks for what; its confirm calls `Dispatcher::confirm` as
`Actor::Human`, which checks again before running.

## reason

- an agent's commit rests on the staged set and the message it saw, not
  on whatever the index holds when the worker gets to it.
- the dispatcher's check catches what the ui knows (a person unstaged a
  file); the worker's check catches what it cannot (a terminal staged
  one). either alone leaves a hole.
- reading the staged set from `status` on both sides keeps the two
  checks comparing the same thing (renames count as their new path).
- one handler per capability, reached by the person, an agent and a test
  alike, gives one record of who asked for what.

## consequences

- residual race: between the worker's read of the index and `git
  commit`, another process can still move it; git holds `index.lock`
  only inside its own process. that window is outside this contract.
  closing it means committing the tree that was checked (`git
  write-tree`, compare, `git commit-tree`, `git update-ref` with the old
  HEAD), which skips the hooks, signing and reflog message `git commit`
  gives; a later decision can take that trade.
- the correlation of answers to requests relies on one worker answering
  in order. a pool of workers, or reordering, would need an opaque tag
  echoed by `GitCommand` and `GitEvent`. the reads rely on the same order
  (task-84): the session counts the status, log and branches reads it
  asked for, and a read asked before a mutation the view already shows is
  dropped when it arrives, since the refresh that mutation's answer sends
  is queued after it (`apps/idestudio/src/repository/git.rs`, `Reads`). without it a
  status read before a stage landed over the stage and the projection
  showed, for a moment, a state that was neither git's nor the one the
  ide had accepted.
- when this was decided, `sem` had no way to decline or expire a request
  awaiting confirmation: when the person closed the modal without
  confirming, the receipt stayed awaiting and the workbench stopped asking
  and said so. task-58b added the decline (see the update at the end of
  this section).
- the confirmation modal and the context menu are drawn with comps
  widgets and not projected: the menu's items are the file's
  capabilities, and confirming is the dispatcher's `confirm`, not a
  capability. confirming through assistive technology needs a projected
  confirmation (task-60, task-69).
- an open overlay does not make the workspace inert for other actors, as
  the todo page does: an agent keeps acting while the person reads a
  menu or a modal, and the checks keep it safe.
- task-61 carries the same requests and receipts over the agent socket;
  the phase c experiment drives this workbench from outside the process.
- updated on 2026-09-23: `Dispatcher::decline` exists since task-58b
  (`rust/accord/src/dispatch.rs`). closing the modal without confirming
  (cancel, escape, a click outside it) declines as `Actor::Human`
  (`apps/idestudio/src/capability/person.rs`): the receipt ends rejected with
  `Rejection::Declined`, nothing runs, and the next request waiting for a
  person is asked. expiring a request awaiting confirmation still has no
  api.

## avoid

- a commit sent without the staged set it was decided on.
- deciding stale by the revision, or reading the cursor in `submit`.
- a view that changes git state or projected state outside `execute`.
- clearing the commit form before the commit exists.
- updated on 2026-09-23: projected state does change outside `execute`
  today, and the rule above stands for what a view decides on its own.
  the git worker's answers land when they arrive: the refresh's status
  replaces `changes:staged` with what the index holds (`GitSession::apply`,
  `apps/idestudio/src/repository/git.rs`), which is how a change made outside the ui
  reaches the projection, and since task-84 a status read before a
  mutation the view shows is dropped. typing changes `commit.message:text`
  (`apps/idestudio/src/main.rs`), editing that
  [`0035-semantic-binding-beside-the-scene.md`](../0035-semantic-binding-beside-the-scene/0035-semantic-binding-beside-the-scene.md) exempts. navigation moves
  `changes:selected`, accepted under "one workbench" above.

## learnings

- [what the autonomous cycle taught: a commit checks the index it observed](learned-commit-checks-the-index-it-observed.md)

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0032: semantic contract as a pure projection of app state](../0032-semantic-contract-as-pure-projection/0032-semantic-contract-as-pure-projection.md)
- [ADR 0033: actions carry the state their actor observed](../0033-actions-carry-observed-state/0033-actions-carry-observed-state.md)
- [ADR 0035: semantic binding beside the scene, and the click as a request](../0035-semantic-binding-beside-the-scene/0035-semantic-binding-beside-the-scene.md)
- [what the autonomous cycle taught: a commit checks the index it observed](learned-commit-checks-the-index-it-observed.md)
- [task-58: action with precondition, dispatch and receipts](../../../Tasks/task-58-actions-preconditions-receipts/task-58-actions-preconditions-receipts.md)
- [task-58b: declining a confirmation](../../../Tasks/task-58b-decline-a-confirmation/task-58b-decline-a-confirmation.md)
- [task-59: semantic binding with the render and one operational path](../../../Tasks/task-59-semantic-binding-and-one-path/task-59-semantic-binding-and-one-path.md)
- [task-60: accessibility as a projection of the contract](../../../Tasks/task-60-accessibility-projection/task-60-accessibility-projection.md)
- [task-61: agent surface (local socket, observe, act)](../../../Tasks/task-61-agent-surface/task-61-agent-surface.md)
- [task-62: proof in the ide: stale-state rejection and human-agent handoff](../../../Tasks/task-62-ide-human-agent-proof/task-62-ide-human-agent-proof.md)
- [task-69: ephemeral and generated surfaces](../../../Tasks/task-69-ephemeral-generative-surfaces/task-69-ephemeral-generative-surfaces.md)
- [task-84: an old refresh does not undo what the ide has already accepted](../../../Tasks/task-84-ide-refresh-never-goes-back/task-84-ide-refresh-never-goes-back.md)
- [task-85: the commit that the receipt announces is already in the projection, with the files](../../../Tasks/task-85-the-commit-a-receipt-announces-is-projected/task-85-the-commit-a-receipt-announces-is-projected.md)
- [task-62 phases a and b: close-out (2026-09-23)](../../../Tasks/task-62-ide-human-agent-proof/phase-ab-closing.md)
- [LLM agents commit through the IDE contract: stale refusal and handoff, five rounds](../../../Research/Exploring/agent-proof-checkpoint-a.md)
<!-- /related -->
