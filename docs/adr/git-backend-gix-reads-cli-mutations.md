---
type: adr
status: accepted
tags: [git, backend, gix, threading, apps]
date: 2026-04-01
---

# git backend: gix for reads, git CLI for mutations, a worker thread for the UI

## context

plev apps (the native git client in `crates/ide`) need git: history, status,
diffs, staging, commits. two failure modes to avoid. first, doing git work on
the UI thread blocks the frame loop, a status or diff on a large repo stalls
rendering. second, assembling the whole porcelain surface (status, unified
diff, stage/unstage/discard/commit) out of gix plumbing crates by hand is a
large, fragile surface to own.

## decision

split the work by what each tool makes easy, and isolate all of it behind a
crate with no UI dependency.

- gix owns the read side it does well: open and discover the repo, walk
  history, list refs.
- the git CLI owns worktree status, diffs and every mutation (stage, unstage,
  discard, commit). its porcelain v2 and unified diff formats are explicitly
  stable for tooling. this is the same pragmatic route Zed takes.
- `GitRepo` is the synchronous API. the UI never calls it directly. it talks
  to `GitClient`, which runs a `GitRepo` on a worker thread, takes commands
  over an mpsc channel, and returns results through a caller-provided callback
  (typically forwarded into a `winit::EventLoopProxy`). the UI thread never
  blocks.
- the crate carries no UI types and is tested against real temporary
  repositories (`tests/real_repo.rs`).

## consequences

- the frame loop never stalls on git. a slow status on a huge repo is a
  worker-thread cost, not a dropped frame.
- the porcelain parsing surface (porcelain v2, unified diff) is owned and
  tested, but it is text parsing against a documented stable format, not a
  hand-built plumbing pipeline.
- git must be on PATH. a pure-gix build would drop that dependency but would
  mean owning the mutation and porcelain surface by hand; deferred until gix
  ships a stable porcelain API.
- `GitCommand::Refresh` emits Status, Log and Branches in one pass, used after
  mutations and by file watchers, so the UI reconciles optimistic updates
  against one consistent snapshot.

## avoid

- never call `GitRepo` from the UI thread. the whole point of `GitClient` is
  that the frame loop stays free.
- do not grow a parallel async runtime here. one worker thread plus a channel
  is the contract.
- do not replace the CLI mutations with hand-assembled gix plumbing until
  there is a stable porcelain API; that hand-built version is exactly the
  fragile surface this split avoids.
