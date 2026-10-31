---
id: arquitetura-component-design
title: "component state design (task-02)"
project: plev
type: referencia
status: referencia
date: 2021-03-24
date_basis: cabecalho
topics: [components]
relations: []
source: docs/ref/component-design.md
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# component state design (task-02)

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
