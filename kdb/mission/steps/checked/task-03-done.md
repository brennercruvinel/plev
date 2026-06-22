---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: task-tracking
---

# task-03: layout engine (flexbox-like), done

## objetivo
implementar sistema de layout que calcula posição e tamanho de views antes de gerar scenenodes. modelo inspirado em flexbox (direção, alinhamento, padding, gap).

## dependências
- task-01 (view trait, layout alimenta o viewcontext)

## checklist de conclusão
- [x] struct `LayoutStyle` com propriedades: direction, align, justify, padding, gap, size constraints
- [x] algoritmo de layout via taffy 0.9 que resolve constraints top-down (parent -> children)
- [x] integração com viewcontext, views recebem bounds calculados via computedbounds
- [x] containerview que aplica layout aos filhos
- [x] exemplo funcional: layout_demo.rs com header/sidebar/main layout
- [x] `cargo build` passa sem warnings relevantes
- [x] `cargo test` passa com 19 testes (12 layout + 7 view)
- [x] performance: layout de 1000 nodes < 1ms em release mode

## implementação
- `src/layout.rs`: layoutengine, layoutstyle, computedbounds, direction, align, justify
- `src/view.rs`: viewcontext.bounds, view.layout(), view.children(), containerview
- `src/window.rs`: two-phase rendering com collect_layout_items + walk_and_render
- `examples/layout_demo.rs`: demo de layout sem GPU
- taffy wrappado como detalhe de implementação, consumidores usam tipos plev
