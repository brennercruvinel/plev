---
id: adr-0026
title: "background results wake the event loop; child pipes drain from the start"
project: plev
type: adr
status: accepted
date: 2026-09-18
date_basis: header
recorded: 2026-09-18
topics: [urna, performance, rendering]
relations: [learning-subprocesses-and-pipes, task-99, learning-cycle-0026]
source: docs/adr/rendering/0026-worker-results-wake-the-event-loop.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# background results wake the event loop; child pipes drain from the start

Decides that the worker wakes the event loop through a wake callback that sends a user event, and that every child process drains stdout and stderr from the spawn onward, through the single subprocess helper. The reason: a result that arrived seconds after the last input stayed parked in the channel until the next mouse movement, and a child that writes more than the pipe buffer blocks on write until the timeout. The result is that the result appears the moment it exists, and a stuck child costs only the timeout and a typed error.

## context

plev apps render on demand: winit sleeps until an input event, and the
app polls its background work from `about_to_wait`. that is enough while
every result is caused by a click the user is still moving the mouse
after. it stopped being enough in urnaui, where a result can land seconds
after the last input (an offline embed, a 38k-row exact scan, an ffmpeg
decode): the event arrived on the channel and sat there until the next
mouse move.

the same wave found a second silent hang: the embedder and ffmpeg bridges
spawned a child with piped stdout, polled `try_wait`, and only read the
pipe after exit. a child writing more than the OS pipe buffer (64 KiB on
macOS; one png frame is ~90 KiB) blocks on `write`, the poll never sees
an exit, and both sides wait forever until the timeout kills the child.

## decision

- the worker owns a wake callback (`UrnaWorker::set_wake`) installed by the
  platform shell at startup: a boxed `Fn` that sends
  `UserEvent::WorkerWoke` through the winit `EventLoopProxy`. every event
  the worker queues is followed by one wake; the shell's `user_event`
  handler drains the channel and invalidates. the view stays winit-free
  (it sees a closure, never a proxy); the web inline worker accepts the
  callback and ignores it, since its results are queued before `send`
  returns.
- one subprocess helper (`model/subprocess.rs`) for every child the
  worker runs: stdout and stderr are drained on their own threads from
  the moment of spawn, the parent polls `try_wait` against a deadline, a
  timeout kills and reaps. callers map `RunError` into their own typed
  errors and never touch `Command` themselves.

## consequences

- a result renders the moment it exists, with the cursor still. this is
  the contract any plev app with a worker thread should copy: poll from
  `about_to_wait` for the input-driven case, wake through the proxy for
  the rest.
- the pipe-buffer hang is impossible by construction; a wedged child
  costs its timeout and a typed error, never a frozen worker.
- the helper has its own tests (200 KiB through the pipe, stderr on a
  non-zero exit, a kill at the deadline, a missing program).

## avoid

- do not poll a background channel only from `about_to_wait`; do not
  hand the view a proxy or any winit type to work around it.
- do not spawn a child with `Stdio::piped()` and read after `try_wait`;
  go through the helper.

## learnings

- [Why waiting for a process can block reading its output](subprocesses-and-pipes.md)
- [what the autonomous cycle taught: background results wake the event loop; child pipes drain from the start](learned-worker-results-wake-the-event-loop.md)

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0025: urnaui: decode media frames in place with ffmpeg's subfile protocol](../../Importing/0025-urnaui-media-frames-in-place-via-ffmpeg-subfile/0025-urnaui-media-frames-in-place-via-ffmpeg-subfile.md)
- [ADR 0029: embed an app view by giving it a frame origin](../0029-embed-an-app-view-by-frame-origin/0029-embed-an-app-view-by-frame-origin.md)
- [ADR 0039: agents act through the semantic contract](../../Semantics/0039-agents-act-through-the-semantic-contract/0039-agents-act-through-the-semantic-contract.md)
- [what the autonomous cycle taught: background results wake the event loop; child pipes drain from the start](learned-worker-results-wake-the-event-loop.md)
- [Why waiting for a process can block reading its output](subprocesses-and-pipes.md)
- [task-60: accessibility as a projection of the contract](../../../Tasks/task-60-accessibility-projection/task-60-accessibility-projection.md)
- [task-61: agent surface (local socket, observe, act)](../../../Tasks/task-61-agent-surface/task-61-agent-surface.md)
- [task-78: AI-native applications](../../../Tasks/task-78-ai-native-apps/task-78-ai-native-apps.md)
- [task-99: explorer integration and platform fixes](../../../Tasks/task-99-integration-and-portability/task-99-integration-and-portability.md)
- [how to code against the plev engine](../../../Research/Workflows/code-against-the-plev-engine.md)
<!-- /related -->
