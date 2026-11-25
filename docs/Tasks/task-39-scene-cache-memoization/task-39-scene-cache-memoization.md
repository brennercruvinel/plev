---
id: task-39
title: "task-39: scene cache, memoization + dirty flag bubbling"
project: plev
type: task
status: superseded
date: 2026-09-25
date_basis: header
recorded: 2026-03-13
topics: [rendering, performance]
relations: [task-73, research-extracted-patterns]
source: docs/log/steps/task-39-scene-cache-memoization.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-39: scene cache, memoization + dirty flag bubbling

Records the proposal to optimize component rendering with memoization by `PartialEq` (b7) and with dirty flags propagated to the parent (b8, `merge_up`), to skip `render()` when the props do not change. The task has status `superseded`: [task-73](../task-73-incremental-dependency-system/task-73-incremental-dependency-system.md) absorbed it and carried the memoization options and the separate layout and paint flags into the phase 2 ADR, compared with the phase 1 numbers. The per-component cache (`cached_nodes`, `needs_render`) already existed and remains in the engine; what is still open is a decision for 73.

## objetivo
implementar otimizacoes de rendering por componente: memoizacao via partialeq (b7) e dirty flag bubbling (b8) para pular render() de componentes cujas props nao mudaram.

## contexto
- scene cache per-component ja implementado (task-xxx, component.rs): `cached_nodes`, `needs_render`, `invalidate()`
- o que falta: mecanismo para o proprio componente detectar que nao precisa re-render sem chamar invalidate() manualmente
- patterns: b7 (xilem memoize), b8 (masonry merge_up)

## dependencias
- nenhuma task bloqueadora

## checklist

### b7: component memoization via partialeq
- [ ] adicionar metodo `needs_render_for_props<P: PartialEq>(&self, new_props: &P, prev_props: &P) -> bool` como helper
- [ ] ou: adicionar variante `Component::memoized(inner, prev_props)` que compara automaticamente
- [ ] testes: componente com props identicas nao chama render() no segundo frame

### b8: dirty flag bubbling (merge_up)
- [ ] definir flags granulares: `needs_layout`, `needs_paint` separados de `needs_render` (current)
- [ ] `merge_up(child: &ComponentState) -> ()` propaga flags do filho para o pai
- [ ] considerar impacto no modelo atual (sem arvore de componentes explicita no plev)
- [ ] testes: mudar cor (paint only) nao dispara relayout

## notas tecnicas
- pattern b7 de xilem usa `core::mem::take(&mut view_state.dirty) || prev.data != self.data`
- pattern b8 de masonry: flags separados `request_layout`, `request_paint`, `needs_layout`, `needs_paint`
- plev nao tem arvore de componentes retida, implementar merge_up pode requerer mudar o modelo ou limitar ao subset de flags paint vs layout
- consultar [`extracted-patterns.md`](../../Research/Ecosystem/extracted-patterns.md) secoes b7 e b8 antes de implementar

## fechamento (orquestrador, 2026-09-25)

absorvida pela task-73 (`absorbs: [task-39]` no arquivo dela): a
memoização por `PartialEq` e as flags de layout e paint separadas são
opções que o ADR da fase 2 da 73 compara com os números da fase 1. o
`cached_nodes` e o `needs_render` que esta task citava continuam no
engine; o que falta é decisão da 73, não trabalho desta.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [task-73: incremental dependency system](../task-73-incremental-dependency-system/task-73-incremental-dependency-system.md)
- [extracted patterns, 56 reference repos](../../Research/Ecosystem/extracted-patterns.md)
- [program: from GPU UI engine to generative interaction runtime](../../Research/Synthesis/generative-runtime-program.md)
<!-- /related -->
