---
project: phi
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: task-tracking
---

# task-06: validação WASM/webgpu no browser, done

## objetivo
garantir que o φ compila para `wasm32-unknown-unknown` e roda no browser com webgpu, com rendering idêntico ao nativo.

## status: concluída

## checklist de conclusão
- [x] `cargo check --target wasm32-unknown-unknown` compila sem erros (zero warnings)
- [x] fix bug crítico: init GPU async no WASM (eventloopproxy pattern)
- [x] limits WASM: `downlevel_webgl2_defaults` -> `default` (webgpu, não webgl)
- [x] feature `Performance` adicionada ao web-sys para fps counter
- [x] `trunk build --release` compila sem erros (2.4mb bundle com wasm-opt)
- [x] `trunk serve` roda e responde http 200 em localhost:8080
- [x] build nativo continua funcionando (`cargo check` + `cargo run --bin φ-app`)
- [x] documentação: knowledge/wasm-webgpu-validation.md criado
- [x] rules.md atualizado com armadilhas WASM

## itens que requerem teste manual no browser
> estes itens dependem de abrir o browser com webgpu e verificar visualmente.
> comando: `cd /Users/aac/Dev/φ-task06 && trunk serve`

- rendering visual no chrome/firefox/safari
- text rendering via atlas
- dirty tracking via console logs
- fps measurement
- font embedded via include_bytes!

## arquivos modificados
- `src/window.rs`, eventloopproxy pattern, appevent, applicationhandler<appevent>
- `src/lib.rs`, eventloop::with_user_event(), proxy para app::new()
- `src/main.rs`, cfg guard para WASM, eventloop com appevent
- `src/gpu.rs`, limits::default() para WASM (era downlevel_webgl2_defaults)
- `Cargo.toml`, bin renomeado φ-app, feature performance no web-sys
- `index.html`, data-target-name="φ" para trunk
- `.gitignore`, dist/ adicionado

## branch
`task/TASK-06-wasm-validation` (mergeada em master)
