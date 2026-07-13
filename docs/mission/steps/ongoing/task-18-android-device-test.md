---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2025-11-26
domain: task-tracking
---

# task-18: android build & device test

## objetivo
compilar, gerar APK e testar em device/emulator android.

## dependências
- task-11 (android build code)
- NDK instalado

## checklist
- [x] `cargo check --target aarch64-linux-android --features android-game-activity` compila
- [x] `cargo ndk -t arm64-v8a build --features android-game-activity` compila (.so)
- [x] fix: `#[unsafe(no_mangle)]` para rust edition 2024
- [ ] gerar APK wrapper (cargo-apk ou gradle project)
- [ ] deploy no device/emulator
- [ ] verificar: quad rendering, text, input touch, gestures
- [ ] testar lifecycle: background/foreground, rotate, memory warning

## notas
- NDK: `ndk/27.2.12479018/` (raiz do repo, gitignored)
- cargo-ndk v4.1.2 instalado
- emulator + system-images arm64 API 35 disponíveis
- APK wrapper é bloqueio para deploy
