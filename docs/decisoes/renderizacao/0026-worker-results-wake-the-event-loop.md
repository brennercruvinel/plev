---
id: adr-0026
title: "background results wake the event loop; child pipes drain from the start"
project: plev
type: decisao
status: aceito
date: 2026-09-18
date_basis: cabecalho
topics: [urna, desempenho, renderizacao]
relations: []
source: docs/adr/rendering/0026-worker-results-wake-the-event-loop.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# background results wake the event loop; child pipes drain from the start

Decide que o worker acorda o event loop por um callback de wake que envia um evento de usuário, e que todo processo filho drena stdout e stderr desde o spawn, pelo helper único de subprocesso. O motivo: um resultado que chega segundos após a última entrada ficava parado no canal até o próximo movimento do mouse, e um filho que escreve mais que o buffer do pipe bloqueia no write até o timeout. O resultado é que o resultado aparece no momento em que existe, e um filho travado custa só o timeout e um erro tipado.

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
