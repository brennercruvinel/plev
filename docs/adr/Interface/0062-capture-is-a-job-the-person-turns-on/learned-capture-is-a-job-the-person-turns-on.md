---
id: learning-cycle-0062
title: "what the autonomous cycle taught: capture is a job the person turns on"
project: plev
type: learning
status: reference
date: 2026-09
date_basis: text
recorded: 2026-09-23
topics: [platform, rendering, semantics, agents]
relations: [adr-0062, task-76, task-79]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: capture is a job the person turns on

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0062](0062-capture-is-a-job-the-person-turns-on.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-24, task-76 item 10

- **um frame por imagem enchia o atlas em 16 segundos:** o depósito de
  imagens do engine só crescia e guardava os pixels de cada envio, o que
  serve a ícones e fotos e quebra com vídeo. a imagem de fluxo (um lugar
  fixo, os pixels trocados no lugar, reenviados uma vez por atlas) é o
  caminho de qualquer fonte ao vivo: câmera, vídeo, canvas de terceiros.
- **o ffmpeg não sabe de permissão:** quem decide é o macOS, pelo
  processo que lançou (o terminal ou o `.app`). a recusa chega como texto
  de erro genérico do AVFoundation, então a classificação em "permissão
  negada" só se confirma na máquina de uma pessoa.

## 2026-09-25, teste de privacidade procura tudo, não amostra

- **o teste da câmera da fase 1 da task-79 procurava 16 das 200 cores da
  câmera falsa:** passava porque os frames daquela rodada caíam nessas
  cores; na fase 2 os frames tinham outras, e o teste seguia verde sem
  provar nada. o agente da fase 2 achou e trocou por todas. regra para
  briefing e para a revisão: um teste de que algo não atravessa procura o
  conjunto inteiro do que não pode atravessar (todas as cores, a string
  inteira, todo byte do fio), nunca uma amostra.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0062: capture is a job the person turns on](0062-capture-is-a-job-the-person-turns-on.md)
- [task-76: platform integration](../../../Tasks/task-76-platform-integration/task-76-platform-integration.md)
- [task-79: scene over socket](../../../Tasks/task-79-scene-over-socket/task-79-scene-over-socket.md)
<!-- /related -->
