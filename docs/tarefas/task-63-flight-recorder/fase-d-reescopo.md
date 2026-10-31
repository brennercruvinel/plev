---
id: task-63/fase-d-reescopo
title: "task-63 fase d re-escopada: o ide grava a sessão"
project: plev
type: secao
parent: task-63
status: referencia
date: 2026-09-23
date_basis: texto
topics: [gravacao, agentes]
relations: [adr-0037, task-64, task-88]
source: docs/log/steps/task-63-flight-recorder.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# fase d re-escopada (2026-09-23, decisão do orquestrador)

Registra a mudança de escopo da fase d decidida pelo orquestrador em 2026-09-23. A retenção de pixels de imagem e a ligação no shell do showcase saem desta task, e a fase d passa a ser o ide gravando a sessão que o `plev-inspect` lê, o que permite o teste que só o Brenner pode fazer: usar o ide enquanto um agente age.

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
