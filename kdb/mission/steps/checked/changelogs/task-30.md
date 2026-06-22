---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: changelog
---

# task-30 changelog

## novos arquivos
- `src/accessibility.rs`, accessibilitystate, focusgraph, id mapping

## modificados
- `src/window.rs`, a11y adapter, handlers, tree updates
- `src/input/mod.rs`, hit_regions(), set_focused_view()
- `src/lib.rs`, conditional pub mod accessibility
- `Cargo.toml`, accesskit + accesskit_winit features

## testes adicionados
- 8 em accessibility.rs (id mapping, tree gen, focus graph)
