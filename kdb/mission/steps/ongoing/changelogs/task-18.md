---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-03-08
domain: task-tracking
---

# changelog, task-18: android build & device test

## sessão 1 (2026-03-08)

### compilação
- `cargo check --target aarch64-linux-android --features android-game-activity`
- fix: `#[no_mangle]` -> `#[unsafe(no_mangle)]` (rust edition 2024 requirement)
- `cargo ndk -t arm64-v8a build --features android-game-activity` (full build, 27s)
- NDK 27.2.12479018 com toolchain darwin-x86_64, API level 33

### ambiente
- NDK em `/Users/aac/Dev/plev/ndk/27.2.12479018/`
- emulator + system-images/android-35/google_apis/arm64-v8a disponíveis
- platform-tools/adb disponível (v37.0.0)
- cargo-ndk v4.1.2 instalado

### pendente
- APK wrapper necessário para deploy no emulator (cargo-apk ou gradle)
- deploy e teste visual no device/emulator
- lifecycle testing (background/foreground, rotate)

### warnings (esperados)
- `mut attrs` unused em window.rs (cfg-condicional, usado em desktop/ios mas não android)
