---
project: phi
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-11
domain: wasm
---

# WASM plugin architecture - research doc

## 1. runtime options

### wasmtime
- **maintainer**: bytecode alliance (mozilla/fastly/intel)
- **WASI**: full support (preview1 + preview2)
- **performance**: cranelift JIT, ~10ns per host function call overhead
- **size**: ~5mb binary impact
- **license**: apache-2.0
- **rust API**: `wasmtime` crate, well-documented, stable
- **platform support**: linux/macos/windows. no native ios/android (no JIT on ios). no WASM-in-WASM.
- **recommendation**: best for native-only plugins

### wasmer
- **maintainer**: wasmer inc
- **WASI**: full support
- **performance**: multiple backends (cranelift, llvm, singlepass). singlepass ~15ns/call.
- **size**: ~4mb binary impact (singlepass backend)
- **license**: MIT
- **rust API**: `wasmer` crate, occasionally breaking API changes
- **platform support**: similar to wasmtime. singlepass backend works on arm64.
- **recommendation**: good alternative if wasmer API stabilizes

### extism
- **maintainer**: dylibso
- **model**: high-level plugin SDK on top of wasmtime
- **value-add**: manifest-based plugin loading, typed function calls, host function registry, multi-language guest sdks (rust, go, c, js, etc.)
- **size**: ~6mb (includes wasmtime)
- **license**: bsd-3
- **recommendation**: simplest path for "load a .wasm plugin and call functions"

## 2. interface design

### minimal plugin interface (FFI)
```
// Plugin exports:
fn φ_build_scene(width: f32, height: f32) -> *const u8  // returns serialized SceneNodes
fn φ_handle_event(event_ptr: *const u8, len: u32) -> u32
fn φ_init() -> u32

// Host exports (callable from plugin):
fn φ_push_rect(x: f32, y: f32, w: f32, h: f32, r: f32, g: f32, b: f32, a: f32)
fn φ_push_text(text_ptr: *const u8, text_len: u32, x: f32, y: f32, size: f32)
fn φ_log(msg_ptr: *const u8, msg_len: u32)
```

### communication model
- **preferred**: host function calls (plugin calls `φ_push_rect` etc.)
  - pro: zero serialization, minimal copies
  - con: wider API surface, harder to version
- **alternative**: shared linear memory with serialized scenenodes
  - pro: narrow interface (one function), easy versioning
  - con: serialization overhead (~1us per 1000 nodes with bincode)

## 3. viability assessment

### feasible
- native desktop plugins via wasmtime/extism: mature, fast, well-documented
- overhead: 10-15ns per host call is negligible (a scene with 100 host calls = 1.5us)
- binary size: 5-6mb for the runtime is acceptable for desktop apps

### problematic
- **ios**: no JIT allowed. AOT compilation possible but complex. wasmtime has `cranelift` AOT path.
- **android**: works but adds significant APK size
- **WASM-in-WASM**: not practical. the browser already is the WASM runtime. nested WASM (wasmer-js) exists but is slow and fragile.

### not recommended now
- plugin hot-reload: requires file watching + runtime swap, separate task
- sandboxing beyond WASM defaults: WASI capability model is sufficient

## 4. recommendation

**wait.** the plugin system is p4 priority. current value proposition:
- φ targets 6 platforms. WASM plugins only work well on 3 (macos/linux/windows).
- ios/android/browser need different solutions or AOT compilation.
- the API is still evolving (task-31/30 just landed). plugins need a stable API to target.

**when ready:**
1. use `extism` for plugin loading (highest-level, least code)
2. host function interface (`φ_push_rect/text/path`) over shared memory
3. feature-gate as `plugins = ["dep:extism"]`
4. desktop-only initially, mobile/WASM stretch goals

## 5. binary size impact estimate

| component | size (release) |
|-----------|---------------|
| φ (current) | ~8mb |
| + extism/wasmtime | ~14mb (+6mb) |
| + wasmer singlepass | ~12mb (+4mb) |

WASM target is already 2.4mb. plugin support would not apply to WASM builds.
