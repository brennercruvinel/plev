---
project: phi
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: task-tracking
---

# task-15: pipeline DSL -> builder -> compositor, done

## objetivo
`phi_narrate!` gera código que roda end-to-end: produz scenenodes renderizados na tela.

## dependências
- task-05 (builder API)
- task-14 (phi_narrate! DSL)
- task-01 (view trait)

## checklist
- [x] bridge phi_narrate -> builder real (remover stubs, re-exportar φ::builder)
- [x] adicionar `φ` como dependência em `crates/phi_narrate/Cargo.toml`
- [x] element implementa conversão para vec<scenenode> (view trait, flatten recursivo)
- [x] intof32 trait para aceitar int e float nos métodos do builder
- [x] intoradius trait para rounded() aceitar presets nomeados
- [x] intoview for &str/string, habilita .child("text") pattern do DSL
- [x] text content merge: text("").child("hello") seta conteúdo
- [x] métodos faltantes: centered, px/py, bold/italic, child_if, children_each, etc.
- [x] exemplo end-to-end: `examples/narrate_demo.rs`
- [x] 207 testes passando (141 φ + 12 integration + 54 macro)
- [x] exemplo compila e renderiza visualmente

## arquivos modificados
- `crates/phi_narrate/src/lib.rs` (stubs -> real bridge)
- `crates/phi_narrate/Cargo.toml` (add φ dependency)
- `src/builder.rs` (intof32, intoradius, missing methods, text merge)
- `Cargo.toml` (phi_narrate dev-dependency)
- `examples/narrate_demo.rs` (novo)
