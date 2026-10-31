---
id: architecture-view-trait-design
title: "view trait - design decisions"
project: plev
type: reference
status: reference
date: 2021-03-06
date_basis: reconstructed
recorded: 2026-03-08
topics: [rendering, performance]
relations: []
source: docs/ref/view-trait-design.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# view trait - design decisions

Records the design decisions of the `View` trait: the signature `render(&self, cx)`, which returns `Vec<SceneNode>`, a `ViewContext` with no reference to the compositor, and the concrete views `RectView` and `TextView`. The result is views as pure declarations, testable without a GPU, with one `Vec` allocation per render that the page considers negligible. The design matters because the compositor remains the only destination for `SceneNode`s, with no change to `push()`.

## assinatura
```rust
pub trait View {
    fn render(&self, cx: &mut ViewContext) -> Vec<SceneNode>;
}
```

## viewcontext sem referência ao compositor
viewcontext contém apenas informações do viewport (width, height). não guarda referência ao compositor.

**razões:**
- **testabilidade:** testes unitários de views não precisam de compositor, GPU, ou qualquer infraestrutura wgpu
- **composabilidade:** views podem compor o output de outros views concatenando vecs
- **simplicidade:** sem lifetime parameters no viewcontext
- **desacoplamento:** views produzem scenenodes puros, o caller decide o que fazer com eles

**trade-off aceito:** cada `render()` aloca um `Vec<SceneNode>`. isso é negligível porque:
- são alocações CPU temporárias, não GPU
- o hot path de performance é o dirty tracking via fxhasher (que permanece intacto)
- views típicas produzem 1-5 nodes cada

## `&mut ViewContext` na signature
embora viewcontext hoje seja read-only, a signature usa `&mut` para permitir extensão futura (ex: layout state acumulado, cursor de posição).

## views concretas
- `RectView`: wraps campos de scenenode::rect
- `TextView`: wraps campos de texto, cria textnodekey internamente

## integração com window.rs
```rust
let mut cx = ViewContext { width: w, height: h };
for view in &views {
    for node in view.render(&mut cx) {
        self.compositor.push(node);
    }
}
```
o compositor continua recebendo scenenodes via `push()` - nenhuma alteração necessária no compositor.
