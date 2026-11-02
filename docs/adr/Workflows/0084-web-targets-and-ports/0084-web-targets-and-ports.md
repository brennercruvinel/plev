---
id: adr-0084
title: "each web target has one command and a port range"
project: plev
type: adr
status: accepted
date: 2026-10-27
date_basis: git
recorded: 2026-10-31
topics: [web-wasm, platform]
relations: [task-102]
evidence: [aa51e7d3, 5de80aad]
source: reconstruction
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# each web target has one command and a port range

## Context

A server left open held the fixed port and blocked another run: the failure
appeared after the build, with "Address already in use" (`5de80aad`). The
script knew only the gallery and the explorer; `graphs` and `primes` needed a
`cd` into the target's folder (`aa51e7d3`).

## Decision

`tool/tasks/webtarget.sh` takes the target name (`uigallery`, `urnadbapp`,
`graphs` or `primes`, lines 19 to 27) and forwards the mode and the arguments
to the matching Trunk. `serve` tries the target's starting port and the next 50
ports (lines 49 to 56); an explicit `--port` takes precedence (line 47).
`build` does not look for ports (line 61).

## Consequences

The command reports the chosen URL (lines 57 and 58). The check uses `lsof` or
`nc` (lines 32 to 43); if neither is available, Trunk reports that the port may
be in use. The probe does not reserve the port: another process can still take
it before the server opens the socket.

## Alternatives

Always failing on the fixed port would require finding and stopping another
server even when a neighboring port was free.

## Evidence

- [`aa51e7d3`](https://github.com/brennercruvinel/plev/commit/aa51e7d33788a679259251c4803eec0d2c9194c3)
- [`5de80aad`](https://github.com/brennercruvinel/plev/commit/5de80aad9378495c53dd053a17fefbe6a0d0c72e)

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [task-102: reusable builds and web commands](../../../Tasks/task-102-builds-and-web-commands/task-102-builds-and-web-commands.md)
<!-- /related -->
