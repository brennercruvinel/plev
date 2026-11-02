---
id: task-79
title: "task-79: scene over socket"
project: plev
type: task
status: done
date: 2026-09-25
date_basis: header
recorded: 2026-09-23
topics: [agents, recording, performance]
relations: [adr-0065, task-59, task-61, task-63, task-66, task-77]
source: docs/log/steps/task-79-scene-over-socket.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-79: scene over socket

Task 79 takes a window's scene over a socket: the app draws, and a viewer on the desktop mirrors the frame with the bindings on top and returns the clicks as `act` through the semantic contract. Phase 1 (task-79a) delivered the scene channel in version 1 of the protocol, the per-layer codec that marks layers dirty in `rec`, the mirror in-process and across two processes with a frame hash equal to the app's on the measured pages, and the bandwidth per page. Phase 2 (task-79b) brought the click back as an agent act and the secret mask, without changing the protocol; the camera's stream image never crosses the channel, only its size and position. The scene remains restricted to localhost: going beyond localhost, with pairing and encrypted transport, remains a pending item.

## objetivo

a UI roda num lugar e desenha noutro. o app no android manda a cena para o
desktop, que renderiza com o inspector por cima; ou o contrário, lógica
num servidor e cliente fino renderizando (server-driven UI no nível de
cena, não de JSON de widget). o urnaui explorando um corpus remoto sem
baixar nada é o caso óbvio.

## ligação com a tese

seção 27 (cena por socket), seção 2 (a cena é um vetor de nós puros,
hasheado por layer, gravável com delta).

## estado atual no código

- `SceneNode` é dado puro, `Clone + Debug + PartialEq`, sem serde
  (`rust/engine/src/drawing/compositor/scene.rs:6-7`).
- layers com hash: só as sujas precisariam viajar.
- o monster já sabe codificar cena com delta descoberto (modo B).
- depois da 61: transporte local e protocolo versionado para o agente.
  depois da 66: render sem janela.

## escopo

1. um transporte só: cena e semântica viajam pelo mesmo transporte da
   task-61, em canais separados, com o protocolo versionado. nenhum segundo
   protocolo paralelo.
2. codificação por frame: só as layers sujas, pelo hash; texto como
   `TextStyle` mais string (o receptor rasteriza com as mesmas fontes
   embutidas, determinismo da 77); imagem enviada uma vez por hash de
   conteúdo. avaliar reusar o codec de cena do monster em vez de serde.
3. visualizador no desktop que recebe e renderiza, com as ligações da
   task-59 por cima (inspector).
4. fase 2: input de volta (o visualizador manda eventos ou, melhor,
   requests semânticas pelo canal do agente).
5. rede: fora do local só com pareamento explícito e nunca execução
   remota; o ADR decide se esta task sai do localhost.

## fora de escopo

streaming de vídeo de pixels; multiusuário.

## dependências

61 e 66 integradas; 63 se o codec de cena do monster for o escolhido.

## superfície sob responsabilidade

o canal de cena no crate de protocolo (com aprovação, porque o protocolo é
da 61), o visualizador (exemplo ou app), serde ou codec para `SceneNode`
(mudança no engine, com aprovação), ADR
[`0065-scene-travels-as-data.md`](../../adr/Recording/0065-scene-travels-as-data/0065-scene-travels-as-data.md).

## interfaces que consome

protocolo e transporte (61), headless e renderer (66), ligações (59),
codec do monster (63).

## interfaces que pode alterar

`SceneNode` por derive de serialização, com aprovação.

## critérios de aceitação

- o showcase rodando num processo (ou no simulador iOS) espelhado num
  visualizador desktop com pixels iguais (hash de frame renderizado nos
  dois lados).
- banda medida por página, com e sem mudança.
- `./tool/tasks/fullcheck.sh` verde.

## testes esperados

roundtrip da codificação de cena, delta só com layers sujas, espelho em
processo com dois compositores.

## riscos e perguntas abertas

- segurança: cena pode conter dado do usuário; nada sai da máquina sem
  pareamento explícito.

## commits esperados

1. `task-79: serialização de cena por layer suja`
2. `task-79: canal de cena no transporte da 61`
3. `task-79: visualizador com ligações por cima`
4. `task-79: input de volta como request semântica`

## pare e reporte se

- o protocolo da 61 não comportar um segundo canal sem versão nova;
- `SceneNode` precisar mudar além de derive.

## Phases

1. [phase 1 (task-79a): the probe](phase-79a-probe.md): the measurement of the protocol, the codec, the images and the bindings; the stop point on the pixels; the channel, the mirror and the privacy review.
2. [phase 2 (task-79b): the probe](phase-79b-probe.md): the measurement of the click and of redaction; the click returned as `act`; the secret mask; the tests and the handoff.
3. [closing (orchestrator, 2026-09-25)](closing.md): the closing against the acceptance criterion and the pending items.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0039: agents act through the semantic contract](../../adr/Semantics/0039-agents-act-through-the-semantic-contract/0039-agents-act-through-the-semantic-contract.md)
- [ADR 0051: a surface materializes only from verified IR, projects from the IR and asks through the app's dispatcher](../../adr/Synthesis/0051-materialized-surfaces-from-verified-ir/0051-materialized-surfaces-from-verified-ir.md)
- [ADR 0065: the scene travels as data](../../adr/Recording/0065-scene-travels-as-data/0065-scene-travels-as-data.md)
- [what the autonomous cycle taught: the silences app is the fixed screen the generated one is compared against](../../adr/Structure/0053-the-silences-app-is-the-fixed-screen/learned-the-silences-app-is-the-fixed-screen.md)
- [what the autonomous cycle taught: capture is a job the person turns on](../../adr/Interface/0062-capture-is-a-job-the-person-turns-on/learned-capture-is-a-job-the-person-turns-on.md)
- [what the autonomous cycle taught: the scene travels as data](../../adr/Recording/0065-scene-travels-as-data/learned-scene-travels-as-data.md)
- [task-59: semantic binding with the render and one operational path](../task-59-semantic-binding-and-one-path/task-59-semantic-binding-and-one-path.md)
- [task-61: agent surface (local socket, observe, act)](../task-61-agent-surface/task-61-agent-surface.md)
- [task-63: session flight recorder](../task-63-flight-recorder/task-63-flight-recorder.md)
- [task-66: headless as a first-class citizen](../task-66-headless-first-class/task-66-headless-first-class.md)
- [task-77: visual determinism](../task-77-visual-determinism/task-77-visual-determinism.md)
- [closure (orchestrator, 2026-09-25)](closing.md)
- [phase 1 (task-79a): the probe](phase-79a-probe.md)
- [phase 2 (task-79b): the probe](phase-79b-probe.md)
- [plev: thesis of the generative interaction runtime](../../Research/Synthesis/generative-runtime-thesis.md)
- [program: from GPU UI engine to generative interaction runtime](../../Research/Synthesis/generative-runtime-program.md)
- [autonomous cycle journal](../../Research/Workflows/autonomous-cycle-journal.md)
<!-- /related -->
