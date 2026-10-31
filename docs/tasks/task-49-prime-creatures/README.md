---
id: task-49
title: "task-49: prime number creatures, port of entropic life"
project: plev
type: task
status: done
date: 2025-10-01
date_basis: reconstructed
recorded: 2026-06-14
topics: [rendering, animation]
relations: [adr-0014, task-27, task-31]
evidence: [c749d12, d2f1060]
source: [docs/log/steps/task-49-prime-creatures.md, docs/log/changes/task-49.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-49: prime number creatures, port of entropic life

Records the task that ported Entropic Life XVI to the `prime` crate, with a pure, tested sim core and rendering by the engine, on desktop and web. The result is proof that the engine can handle an emergent swarm of particles, not only widgets; the trails are built from position history. The trail is shorter and coarser than the source's and is not pixel-identical, as ADR 0014 records.

## objetivo
portar o canvas demo Entropic Life XVI para um crate plev nativo (desktop e web), com sim core puro e testado, e render fiel pela engine. e a prova de que a engine aguenta um swarm de particulas emergente, nao so ui de widgets.

## dependencias
- task-31 (vector paths, para bonds e cores como sdf/path)
- task-27 (animation tick, fixed timestep)
- engine (compositor, layer, render on demand)

## contexto
o demo de referencia desenha motion trails de graca: cada frame pinta um rect translucido sobre o canvas inteiro em vez de limpar, e o frame anterior some por baixo. a engine e o oposto por design (render on demand, cada layer limpa por frame, sem primitiva de feedback). esse choque virou um ADR proprio.

## o que foi entregue
- crate `prime`, lib (sim core puro, sem gpu) + bin (app windowed). desktop e web.
- `sim/`: rng seeded, geracao de primos, matriz de coerencia de primos 250x250 em 4 modos, params compartilhados, estado de particula, physics (steering grid-local + sincronia kuramoto), grid espacial.
- `color.rs`: mapeamento primo -> cor de particula (HSL -> sRGB).
- `scene.rs`: construcao da cena.
- render fiel pela engine: motion trails por position history (ring buffer por particula), bonds ciano via paths, glow halos, breathing cores, mundo em logical pixels, loop de fixed-timestep, brush no botao esquerdo do mouse.

## numeros honestos
- 12 arquivos .rs, ~1611 LOC, 24 testes (sim core deterministico, seeded).
- study clone em `ref/prime-number-creatures`.
- o trail e mais curto e grosseiro que a acumulacao suave do canvas; le como movimento, nao e pixel-identico a fonte (ver ADR).

## referencias
- adr [motion-trails-by-position-history](../../adr/rendering/0014-motion-trails-by-position-history.md)
- commits c749d12 (port), d2f1060 (merge feat/prime-number-creatures)

## fora de escopo
- acumulacao real de framebuffer (textura de feedback + blit), net-new gpu work, adiado ate o trail valer o encanamento

## Delivery record

### sim core (puro, sem gpu)
- [x] rng seeded deterministico
- [x] geracao de primos
- [x] matriz de coerencia de primos 250x250 em 4 modos
- [x] physics: steering grid-local + sincronia kuramoto
- [x] grid espacial, params compartilhados, estado de particula

### render (engine)
- [x] motion trails por position history (ring buffer por particula)
- [x] bonds ciano via paths
- [x] glow halos, breathing cores
- [x] mapeamento primo -> cor (HSL -> sRGB)
- [x] mundo em logical pixels, loop fixed-timestep
- [x] brush no botao esquerdo

### plataforma
- [x] desktop e web (wasm)
- [x] 24 testes (sim core deterministico)
- [x] study clone em ref/prime-number-creatures
