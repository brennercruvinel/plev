---
id: task-01
title: "task-01: view trait + viewcontext"
project: plev
type: task
status: done
date: 2021-03-10
date_basis: header
topics: [rendering, process]
relations: []
source: [docs/log/steps/task-01.md, docs/log/changes/task-01.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-01: view trait + viewcontext

Records the first step of the views: the `View` trait, which encapsulates the production of `SceneNode`s, and the `ViewContext`, with the viewport information. In `window.rs`, the scene now comes from the views instead of being pushed straight to the compositor, with `RectView` and `TextView` as the first concrete views. The result was validated by 4 unit tests and by `text_demo`, which runs the same as before. The abstraction is the basis of the declarative API of the following tasks.

## objetivo
criar a abstração base `View` que encapsula a produção de `SceneNode`s. o usuário do plev deve declarar views que produzem cena, sem tocar no compositor diretamente.

## contexto
hoje o `window.rs` empurra `SceneNode::Rect` e `SceneNode::Text` diretamente no compositor. a camada de view abstrai isso, cada view recebe um `ViewContext` e retorna scenenodes. isso é o passo fundamental para a API declarativa futura.

## dependências
- nenhuma (primeiro passo da nova fase)

## checklist de conclusão
- [x] trait `View` definido com método `render(&self, cx: &mut ViewContext) -> Vec<SceneNode>`
- [x] `ViewContext` struct com viewport info (width, height), sem ref ao compositor por testabilidade
- [x] duas views concretas: `RectView` e `TextView`
- [x] `window.rs` refatorado para usar views em vez de empurrar scenenodes diretamente
- [x] `cargo build` passa sem warnings relevantes
- [x] `cargo run --example text_demo` funciona identicamente ao antes
- [x] 4 testes unitários: rect_view, text_view, custom_view_composes, dyn_view_dispatch

## armadilhas
- não over-engineer: view é trait simples, não framework de widgets
- não introduzir alocações por frame, views devem poder reusar buffers
- manter compatibilidade com o dirty tracking existente do compositor
- o borrow checker pattern do textsystem::resolve() não pode ser afetado

## Delivery record

### 2021-03-06

#### decisões de design
- `ViewContext` sem referência ao compositor, views retornam `Vec<SceneNode>`, caller empurra pro compositor
  - razão: testabilidade (sem GPU nos testes), composabilidade, sem lifetimes complexos
  - alocação de vec é negligível vs trabalho de GPU
- `ViewContext` com `&mut` na signature do trait para extensão futura (layout state)
- views concretas: `RectView` e `TextView` como structs com campos públicos
- examples (`hello.rs`, `text_demo.rs`) não refatorados, continuam usando compositor diretamente

#### arquivos criados
- `src/view.rs`, trait view, viewcontext, rectview, textview, testes unitários

#### arquivos modificados
- `src/lib.rs`, adicionado `pub mod view`
- `src/window.rs`, refatorado render() para usar views
