---
id: learning-cycle-0054
title: "what the autonomous cycle taught: a lost GPU device is recovered in place, and the next frame draws the same"
project: plev
type: learning
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [rendering, gpu, platform]
relations: [adr-0054, task-76]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: a lost GPU device is recovered in place, and the next frame draws the same

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0054](0054-gpu-device-loss-recovers-in-place.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-24, task-76 fase 1

- **o device perdido era um buraco silencioso:** o wgpu engole todo erro
  `DeviceLost` antes do handler fatal, então um `draw` num device morto
  "termina" sem pânico e o compositor marca as layers como limpas em cima
  de nada; só o readback estourava. `destroy` sozinho não avisa: o
  callback chega no primeiro `poll`. a recuperação no lugar refaz o que
  mora no device e o frame seguinte é byte a byte o anterior. a surface
  sobrevive (é do `Instance`) e só precisa ser reconfigurada. fonte:
  `task-76-platform-integration-done.md`, "handoff da fase 1".

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0054: a lost GPU device is recovered in place, and the next frame draws the same](0054-gpu-device-loss-recovers-in-place.md)
- [task-76: platform integration](../../../Tasks/task-76-platform-integration/task-76-platform-integration.md)
<!-- /related -->
