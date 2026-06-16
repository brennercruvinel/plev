---
project: plev
audience: [ai-agents, contributors]
status: pending
last-updated: 2026-03-13
domain: task-tracking
---

# task-39: scene cache, memoization + dirty flag bubbling

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
- consultar `mission/knowledge/extracted-patterns.md` secoes b7 e b8 antes de implementar
