---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2025-09-26
domain: changelog
---

# task-49 changelog: prime number creatures, port do entropic life

## sim core (puro, sem gpu)
- [x] rng seeded deterministico
- [x] geracao de primos
- [x] matriz de coerencia de primos 250x250 em 4 modos
- [x] physics: steering grid-local + sincronia kuramoto
- [x] grid espacial, params compartilhados, estado de particula

## render (engine)
- [x] motion trails por position history (ring buffer por particula)
- [x] bonds ciano via paths
- [x] glow halos, breathing cores
- [x] mapeamento primo -> cor (HSL -> sRGB)
- [x] mundo em logical pixels, loop fixed-timestep
- [x] brush no botao esquerdo

## plataforma
- [x] desktop e web (wasm)
- [x] 24 testes (sim core deterministico)
- [x] study clone em ref/prime-number-creatures
