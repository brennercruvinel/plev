---
project: phi
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-08
domain: build-wasm
---

# WASM/webgpu validation - knowledge

## eventloopproxy pattern (GPU init async no WASM)

### problema
no WASM, `GpuContext::new()` é async e roda via `wasm_bindgen_futures::spawn_local`. o closure precisa ser `'static`, então `self` não pode ser movido para dentro dele. o código original criava o gpucontext mas o descartava - o estado ficava preso em `Initializing` para sempre.

### solução
usar `EventLoopProxy<AppEvent>` para enviar o resultado de volta ao event loop:

1. `EventLoop::<AppEvent>::with_user_event().build()` cria event loop com tipo custom
2. `event_loop.create_proxy()` retorna `EventLoopProxy<AppEvent>` (send + sync)
3. `spawn_local` clona o proxy e envia `AppEvent::GpuReady { gpu, text_system }` quando async completa
4. `ApplicationHandler<AppEvent>::user_event()` recebe o evento e transiciona estado

### API winit 0.30
- `EventLoop::<T>::with_user_event()` retorna `EventLoopBuilder<T>`
- `EventLoopBuilder::build()` retorna `Result<EventLoop<T>, EventLoopError>`
- `EventLoopProxy::send_event(event: T)` retorna `Result<(), EventLoopClosed<T>>`
- `ApplicationHandler<T>` trait com `user_event(&mut self, &ActiveEventLoop, T)` method

## limits: webgpu vs webgl

- `Limits::downlevel_webgl2_defaults()` - limita `max_texture_dimension_2d` a 2048, inadequado para atlas
- `Limits::default()` - baseline garantido pelo spec webgpu, suporta texturas até 8192x8192
- decisão: usar `Limits::default()` pois estamos targeting webgpu (`Backends::BROWSER_WEBGPU`), não webgl

## build WASM

- `main.rs` precisa de `#[cfg(not(target_arch = "wasm32"))]` guard (usa `env_logger`, não disponível no WASM)
- entry point WASM é `wasm_main()` em `lib.rs` (anotado com `#[wasm_bindgen(start)]`)
- `App::new()` tem assinaturas diferentes: nativo sem args, WASM recebe `EventLoopProxy<AppEvent>`
- `EventLoopProxy` import é condicional (`#[cfg(target_arch = "wasm32")]`)

## fps measurement (WASM)
- `web-sys` feature `Performance` necessária
- `web_sys::window().unwrap().performance().unwrap().now()` retorna milliseconds (f64)
