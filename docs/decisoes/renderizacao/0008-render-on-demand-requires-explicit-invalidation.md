---
id: adr-0008
title: "render on demand makes invalidation a correctness contract"
project: plev
type: decisao
status: aceito
date: 2024-10-07
date_basis: cabecalho
topics: [renderizacao, input, sinais]
relations: []
evidence: [83720e3]
source: docs/adr/rendering/0008-render-on-demand-requires-explicit-invalidation.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# render on demand makes invalidation a correctness contract

Fixa o contrato que vale quando o engine só renderiza sob pedido: um handler que muda algo visível precisa retornar `true` ou invalidar e pedir o redesenho. O caso que motivou a decisão foi um evento de scroll consumido, com estado alterado, handler retornando `false` e nada invalidado, o que deixou o showcase inteiro sem resposta. Com isso, a invalidação passa a ser tratada como corretude, e os testes de regressão verificam a invalidação, não só o estado mutado.

## context

the engine renders only when something requests a frame (battery and CPU
discipline borrowed from production editors). under this model an event
handler that mutates visual state but fails to signal it does not produce
a stale frame, it produces a frozen application. this was shipped: scroll
events were consumed, state changed, handlers returned false, nothing
invalidated, and the showcase appeared completely unresponsive. the defect
was initially misread as "scroll not implemented".

## consequences of the model (the contract)

- every event handler that changes anything visible must return true or
  call the invalidation path (compositor invalidate plus request_redraw).
  returning false is a statement that nothing visible changed, and the
  scheduler believes it
- animations keep frames flowing only while active
  (`is_animating || compositor.needs_render()`)
- a new interaction feature is not done when state updates; it is done
  when the state change provably schedules a frame. regression tests
  assert the boolean/invalidations, not just the mutated state

## avoid

- never debug "frozen UI" by adding redraws in a loop. find the handler
  that lied about not changing state
- never copy an event handler skeleton without carrying its invalidation
  discipline along
