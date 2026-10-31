---
id: task-06
title: "task-06: validação WASM/webgpu no browser"
project: plev
type: tarefa
status: concluido
date: 2021-06-08
date_basis: cabecalho
topics: [task-tracking]
relations: []
source: [docs/log/steps/task-06.md, docs/log/changes/task-06.md]
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# task-06: validação WASM/webgpu no browser

## objetivo
garantir que o plev compila para `wasm32-unknown-unknown` e roda no browser com webgpu, com rendering idêntico ao nativo.

## status: concluída

## checklist de conclusão
- [x] `cargo check --target wasm32-unknown-unknown` compila sem erros (zero warnings)
- [x] fix bug crítico: init GPU async no WASM (eventloopproxy pattern)
- [x] limits WASM: `downlevel_webgl2_defaults` -> `default` (webgpu, não webgl)
- [x] feature `Performance` adicionada ao web-sys para fps counter
- [x] `trunk build --release` compila sem erros (2.4mb bundle com wasm-opt)
- [x] `trunk serve` roda e responde http 200 em localhost:8080
- [x] build nativo continua funcionando (`cargo check` + `cargo run --bin plev-app`)
- [x] documentação: knowledge/wasm-webgpu-validation.md criado
- [x] rules.md atualizado com armadilhas WASM

## itens que requerem teste manual no browser
> estes itens dependem de abrir o browser com webgpu e verificar visualmente.
> comando: `cd ../plev-task06 && trunk serve`

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
- `Cargo.toml`, bin renomeado plev-app, feature performance no web-sys
- `index.html`, data-target-name="plev" para trunk
- `.gitignore`, dist/ adicionado

## branch
`task/TASK-06-wasm-validation` (mergeada em master)

## Registro de entrega

### 2021-06-03

#### início da task
- branch `task/TASK-06-wasm-validation` criada a partir de `master`
- worktree isolado em `../plev-task06` (worktree irmao do repo)

#### fase 1: fix bug init GPU WASM
- **bug:** `spawn_local` criava gpucontext async mas nunca armazenava de volta no app
- **fix:** eventloopproxy pattern, proxy envia `AppEvent::GpuReady` quando GPU está pronta
- `ApplicationHandler<AppEvent>` com `user_event()` handler
- removido variant `GpuState::Initializing` (desnecessário com proxy pattern)
- `App::new()` condicional: nativo sem args, WASM recebe `EventLoopProxy`

#### fase 2: limits
- `wgpu::Limits::downlevel_webgl2_defaults()` -> `wgpu::Limits::default()`
- razão: estamos usando webgpu (não webgl), `default()` é o baseline garantido pelo spec

#### mudanças em arquivos
- `src/window.rs`: appevent enum, eventloopproxy, applicationhandler<appevent>
- `src/lib.rs`: eventloop::with_user_event(), proxy para app::new()
- `src/main.rs`: mesmo pattern + cfg guard para WASM
- `src/gpu.rs`: limits default() para WASM
- `Cargo.toml`: feature "performance" no web-sys, binário renomeado para `plev-app`
- `index.html`: `data-target-name="plev"` para evitar colisão trunk

#### fase 3: compilação e build WASM
- `cargo check --target wasm32-unknown-unknown`, ok, zero warnings
- `trunk build`, ok após fix de colisão bin/lib (data-target-name)
- `trunk build --release`, bundle 2.4mb (com wasm-opt -oz)
- `trunk serve`, http 200 em http://127.0.0.1:8080/
- nativo (`cargo build`) continua compilando ok

#### descobertas
- trunk 0.21 não suporta `data-type="main"`, usar `data-target-name` explícito
- colisão de artifacts quando bin e lib têm mesmo nome, renomear bin para `plev-app`
- worktree necessário quando múltiplos agentes modificam mesmo diretório
