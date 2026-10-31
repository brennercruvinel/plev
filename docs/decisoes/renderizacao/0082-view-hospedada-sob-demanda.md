---
id: adr-0082
title: "Uma view hospedada usa a janela e o relógio da engine"
project: plev
type: decisao
status: aceito
date: 2026-10-27
date_basis: git
topics: [grafos, plataforma]
relations: [task-100]
evidence: [da4be63a, 4aa9706b]
source: reconstrucao
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# Uma view hospedada usa a janela e o relógio da engine

## Contexto

Um demo de um único canvas precisava de eventos, ticks e composição,
mas não de uma implementação própria de toda a janela.

## Decisão

`App::with_view` recebe um `Box<dyn HostedView>` (`rust/engine/src/runtime/window/mod.rs:117`).
A trait separa `handle_event`, `tick` e `render` (`runtime/window/view.rs:15` a `:24`).
A engine entrega eventos em pixels lógicos, hospeda a view no frame e pede frames
enquanto há movimento (`runtime/window/lifecycle.rs:183` a `:190`). O relógio da
view trata o repouso: ao parar de se mover, o relógio para, e a retomada não avança
a simulação pelo tempo inteiro em que a janela ficou sem desenhar (`lifecycle.rs:184`
a `:186`; `4aa9706b`).

## Consequências

O demo compartilha o caminho de composição e inicialização da engine. A
documentação da trait diz que uigallery e idestudio mantêm as próprias shells
(`runtime/window/view.rs:6` e `:7`). A trait de canvas não representa
automaticamente todo o contrato semântico de um app.

## Alternativas

Uma shell nova por demo duplicaria ciclo de vida e conversão de eventos.
Forçar os apps existentes a adotar a trait ampliaria o escopo da entrega.

## Evidências

- [`da4be63a`](https://github.com/brennercruvinel/plev/commit/da4be63ab386c4a7218add9e320ef6e60d842a1a)
- [`4aa9706b`](https://github.com/brennercruvinel/plev/commit/4aa9706bdfdd63f2eff7d9b9b59cf6dda4ab225b)
