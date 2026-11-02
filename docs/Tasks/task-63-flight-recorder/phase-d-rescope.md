---
id: task-63/phase-d-rescope
title: "task-63 phase d re-scoped: the ide records the session"
project: plev
type: section
parent: task-63
status: reference
date: 2026-09-23
date_basis: text
recorded: 2026-09-23
topics: [recording, agents]
relations: [adr-0037, task-64, task-88]
source: docs/log/steps/task-63-flight-recorder.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase d re-scoped (2026-09-23, orchestrator decision)

Records the scope change of phase d, decided by the orchestrator on 2026-09-23. Pixel retention for images and the wiring into the showcase shell leave this task, and phase d becomes the ide recording the session that `plev-inspect` reads, which enables the test that only Brenner can do: using the ide while an agent acts.

a fase d original (retenção de pixels de imagem ligada pelo gravador e a
ligação no shell do showcase, com atalho de debug e panic hook) sai desta
task: o ide não desenha imagem, e o shell do showcase fica para depois.
os dois continuam descritos na fase c abaixo e nas consequências do ADR
`session-recording-replay-not-reexecution`, sem dono agora.

a fase d passa a ser o ide gravando a sessão que o inspector lê (branch
`task/63d-ide-records`, base `main` em `2ecaa8d`):

1. `rec`: cada frame gravado leva onde cada id ligado aparece
   (`Compositor::visible_bindings`), só quando muda em relação ao frame
   anterior, aditivo no formato `MONS` (as sessões antigas continuam
   abrindo), e a `Session` responde onde cada id estava num instante. a
   fase b da task-64 precisa disso para ligar um ponto do frame a um
   controle.
2. `ide`: com a gravação ligada num build de debug, o shell grava no
   relógio do workbench o estado, os receipts, quem observou o quê pela
   porta e os frames, e salva o trecho retido por atalho, no panic e ao
   fechar a janela. sem a variável nada é criado.
3. o laboratório não muda (a task-88 mexe nele).
4. a janela pode subir sem roubar o foco, por variável, só no macOS.

o teste que só o brenner pode fazer (ele usando o ide enquanto um agente
age) precisa deixar uma sessão que o `plev-inspect` lê; é isso que esta
fase entrega.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0037: session recording: replay, not re-execution](../../adr/Recording/0037-session-recording-replay-not-reexecution/0037-session-recording-replay-not-reexecution.md)
- [task-63: session flight recorder](task-63-flight-recorder.md)
- [task-64: session inspector, bug report and session for testing](../task-64-session-inspector-and-tests/task-64-session-inspector-and-tests.md)
- [task-88: the lab measures what the agent rested on](../task-88-the-lab-measures-what-the-agent-rested-on/task-88-the-lab-measures-what-the-agent-rested-on.md)
<!-- /related -->
