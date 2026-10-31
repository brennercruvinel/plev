---
id: architecture-component-design
title: "component state design (task-02)"
project: plev
type: reference
status: reference
date: 2021-03-24
date_basis: header
topics: [design-system, signals]
relations: [task-02, task-04]
source: docs/ref/component-design.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# component state design (task-02)

Records the task-02 decision to separate component state from the `View` trait: `View::render` takes `&self` and keeps no state, while `Lifecycle` and `Component` handle the persistent state and the hooks `on_mount`, `on_update` and `on_unmount`. The result is a split by disjoint fields of the component itself, which does without `RefCell` and keeps the runtime cost at zero, with `on_unmount` firing only after a mount. The design matters because it defines the boundary between declaration and state before the integration with signals (task-04).

## decisão: lifecycle trait separada de view

`View::render` usa `&self` (stateless). `Component::render` usa `&mut self` (lifecycle hooks mutam estado). são dois caminhos distintos:

- **view**: declaração pura, sem estado -> `&self`
- **component<l: lifecycle>**: wrapper com estado persistente -> `&mut self`

component não implementa view. forçar isso exigiria `RefCell`, violando as constraints de zero runtime overhead.

## borrow checker: campos disjuntos

`self.inner.on_mount(&mut self.state)` compila porque `inner` e `state` são campos disjuntos, borrow imutável em `inner` + mutável em `state`. sem lifetimes, sem refcell.

## estado acessível via acessores, não via viewcontext

o design original sugeria estado via viewcontext. o design final é mais simples:
- `Lifecycle::render(&self, state: &Self::State, cx: &mut ViewContext)`, recebe estado diretamente
- `Component::state()` / `Component::state_mut()`, para acesso externo

## drop seguro

`Drop` chama `on_unmount` apenas se `mounted == true`. se o componente nunca foi renderizado, `on_unmount` nunca dispara.

## compatibilidade com task-04 (signals)

- `on_update` pode ser gated por signal dirty flags
- `state_mut()` permite signals escreverem no estado externamente
- nenhuma mudança na trait lifecycle será necessária
