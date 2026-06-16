---
project: plev
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-08
domain: build-ios
---

# ios build - decisões e conhecimento

## font loading no ios
- `fontdb` (usado por cosmic-text) no ios tenta scan de diretórios linux-style e não encontra fontes do sistema
- solução: embedded font via `include_bytes!` - mesmo padrão usado no WASM
- cfg: `cfg(any(target_arch = "wasm32", target_os = "ios"))` unifica WASM e ios no mesmo branch
- desktop (macos, linux, windows) mantém `FontSystem::new()` que escaneia fontes do sistema

## GPU no ios
- `Backends::PRIMARY` já seleciona metal no ios (mesmo backend que macos)
- `Limits::default()` funciona no ios (metal suporta todos os limites default)
- `pollster::block_on` funciona no ios para init async do GPU
- ios não destrói surface metal no background (diferente do android/vulkan)
- não precisa de drop/recreate de surface no lifecycle suspend/resume

## window no ios
- `with_inner_size` é irrelevante (ios sempre fullscreen)
- `WindowAttributesExtIOS::with_valid_orientations` controla orientações suportadas
- `winit::platform::ios::ValidOrientations::LandscapeAndPortrait` é o default
- `winit::platform::ios::ScreenEdge` controla system gesture edges
- `winit::platform::ios::WindowExtIOS` tem `set_prefers_home_indicator_hidden`, `set_prefers_status_bar_hidden`

## lifecycle no ios (winit 0.30)
- `applicationDidBecomeActive` -> `resumed()` - window pode ser criada aqui
- `applicationWillResignActive` -> `suspended()` - app vai para background
- `applicationDidReceiveMemoryWarning` -> `memory_warning()` - liberar memória
- `applicationWillTerminate` -> `LoopExiting` event
- `run_app` nunca retorna no ios - comportamento esperado

## build
- target simulator: `aarch64-apple-ios-sim` (apple silicon)
- target device: `aarch64-apple-ios`
- `staticlib` no `crate-type` gera `.a` necessário para linking no xcode
- simulator: script bash + `xcrun simctl install/launch`
- device: requer xcode project com code signing
- binary size debug: ~50mb+ (wgpu+cosmic-text). release com lto reduz significativamente.

## info.plist obrigatório
- `UILaunchStoryboardName` (vazio, mas obrigatório para resolução nativa)
- `LSRequiresIPhoneOS: true`
- `UISupportedInterfaceOrientations` para declarar orientações
- `UIRequiresFullScreen: true` para evitar split-screen no ipad
