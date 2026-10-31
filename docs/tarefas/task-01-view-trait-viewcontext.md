---
id: task-01
title: "task-01: view trait + viewcontext"
project: plev
type: tarefa
status: concluido
date: 2021-03-10
date_basis: cabecalho
topics: [renderizacao, processo]
relations: []
source: [docs/log/steps/task-01.md, docs/log/changes/task-01.md]
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# task-01: view trait + viewcontext

Registra a primeira etapa das views: a trait `View`, que encapsula a produção de `SceneNode`s, e o `ViewContext`, com as informações de viewport. Em `window.rs`, a cena passou a vir das views em vez de ser empurrada direto para o compositor, com `RectView` e `TextView` como primeiras views concretas. O resultado foi validado por 4 testes unitários e pelo `text_demo`, que roda igual ao anterior. A abstração é a base da API declarativa das tasks seguintes.

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

## Registro de entrega

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
