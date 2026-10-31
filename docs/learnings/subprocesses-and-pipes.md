---
id: learning-subprocesses-and-pipes
title: "Why waiting for a process can block reading its output"
project: plev
type: learning
status: reference
date: 2026-10-16
date_basis: git
recorded: 2026-10-31
topics: [process, urna]
relations: [task-99, adr-0026]
evidence: [05fe993c, 701d2993]
source: reconstruction
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# Why waiting for a process can block reading its output

Records why reading a subprocess's output only after it ends can deadlock both
sides: with the pipe buffer full, the child waits for the parent to consume it,
and the parent waits for the child to finish. The `.urna` explorer's executor
avoids the deadlock by draining stdout and stderr on its own threads, and a test
with output larger than the pipe buffer guards the case. The same helper has
served the embedder since `701d2993`.

Consider a child that writes a large vector to stdout. The pipe buffer
fills up, and the child waits for the parent to consume the bytes. If the
parent is in `wait`, waiting for the child to finish before starting to
read, neither one moves forward.

The explorer's executor takes stdout and stderr from the `Child` and
hands each pipe to a thread. The process owner waits with a timeout while
these threads drain the data. After it finishes, it collects the outputs and
classifies the result. The test `captures_stdout_larger_than_a_pipe_buffer`
exercises exactly an output larger than the buffer.

Ownership makes the distribution of the handles explicit: `take()` removes
the pipe from the `Child`, the thread comes to own it, and its `JoinHandle`
lets it collect the byte vector. This resolves the deadlock from late reading;
it does not replace a policy for byte limits or for descendant processes.

Source: [apps/urnadbapp/src/backend/workers/subprocess.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/apps/urnadbapp/src/backend/workers/subprocess.rs).

## Evidence

- [`701d2993`](https://github.com/brennercruvinel/plev/commit/701d2993266b090acdfc13f641d88df2d57c74e6)
