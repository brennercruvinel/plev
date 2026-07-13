---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2021-03-06
domain: changelog
---

# changelog, task-01: view trait + viewcontext

## 2021-03-06

### decisões de design
- `ViewContext` sem referência ao compositor, views retornam `Vec<SceneNode>`, caller empurra pro compositor
  - razão: testabilidade (sem GPU nos testes), composabilidade, sem lifetimes complexos
  - alocação de vec é negligível vs trabalho de GPU
- `ViewContext` com `&mut` na signature do trait para extensão futura (layout state)
- views concretas: `RectView` e `TextView` como structs com campos públicos
- examples (`hello.rs`, `text_demo.rs`) não refatorados, continuam usando compositor diretamente

### arquivos criados
- `src/view.rs`, trait view, viewcontext, rectview, textview, testes unitários

### arquivos modificados
- `src/lib.rs`, adicionado `pub mod view`
- `src/window.rs`, refatorado render() para usar views
