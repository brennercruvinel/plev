---
project: plev
audience: [ai-agents, contributors]
status: reference
last-updated: 2021-03-06
domain: rendering
---

# view trait - decisões de design

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
