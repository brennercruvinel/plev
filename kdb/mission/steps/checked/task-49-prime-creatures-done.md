---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-06-14
domain: task-tracking
---

# task-49: prime number creatures, port do entropic life

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
- adr [motion-trails-by-position-history](../../../adr/motion-trails-by-position-history.md)
- commits c749d12 (port), d2f1060 (merge feat/prime-number-creatures)

## fora de escopo
- acumulacao real de framebuffer (textura de feedback + blit), net-new gpu work, adiado ate o trail valer o encanamento
