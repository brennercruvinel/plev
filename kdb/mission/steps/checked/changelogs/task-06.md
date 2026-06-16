---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-08
domain: changelog
---

# changelog, task-06: validação WASM/webgpu

## 2026-03-08

### início da task
- branch `task/TASK-06-wasm-validation` criada a partir de `master`
- worktree isolado em `/Users/aac/Dev/plev-task06`

### fase 1: fix bug init GPU WASM
- **bug:** `spawn_local` criava gpucontext async mas nunca armazenava de volta no app
- **fix:** eventloopproxy pattern, proxy envia `AppEvent::GpuReady` quando GPU está pronta
- `ApplicationHandler<AppEvent>` com `user_event()` handler
- removido variant `GpuState::Initializing` (desnecessário com proxy pattern)
- `App::new()` condicional: nativo sem args, WASM recebe `EventLoopProxy`

### fase 2: limits
- `wgpu::Limits::downlevel_webgl2_defaults()` -> `wgpu::Limits::default()`
- razão: estamos usando webgpu (não webgl), `default()` é o baseline garantido pelo spec

### mudanças em arquivos
- `src/window.rs`: appevent enum, eventloopproxy, applicationhandler<appevent>
- `src/lib.rs`: eventloop::with_user_event(), proxy para app::new()
- `src/main.rs`: mesmo pattern + cfg guard para WASM
- `src/gpu.rs`: limits default() para WASM
- `Cargo.toml`: feature "performance" no web-sys, binário renomeado para `plev-app`
- `index.html`: `data-target-name="plev"` para evitar colisão trunk

### fase 3: compilação e build WASM
- `cargo check --target wasm32-unknown-unknown`, ok, zero warnings
- `trunk build`, ok após fix de colisão bin/lib (data-target-name)
- `trunk build --release`, bundle 2.4mb (com wasm-opt -oz)
- `trunk serve`, http 200 em http://127.0.0.1:8080/
- nativo (`cargo build`) continua compilando ok

### descobertas
- trunk 0.21 não suporta `data-type="main"`, usar `data-target-name` explícito
- colisão de artifacts quando bin e lib têm mesmo nome, renomear bin para `plev-app`
- worktree necessário quando múltiplos agentes modificam mesmo diretório
