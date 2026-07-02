---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-03-08
domain: task-tracking
---

# changelog, task-19: ios simulator test

## sessão 1 (2026-03-08)

### compilação
- `cargo check --target aarch64-apple-ios-sim` (clean, zero warnings)
- `cargo build --target aarch64-apple-ios-sim` link falha: "framework 'uikit' not found"
- causa: apenas command line tools instalado, sem xcode.app com ios SDK
- `xcrun --sdk iphonesimulator --show-sdk-path` retorna erro

### pendente
- instalar xcode.app completo (ou obter ios SDK separadamente)
- após SDK disponível: `cargo build --target aarch64-apple-ios-sim` deve linkar
- deploy via ios-sim.sh no simulator
- teste visual: quad rendering, text, metal pipeline, safe areas

## sessão 2 (2026-06-16)

### avanço via task-50 (showcase multiplataforma)
- [x] xcodegen gera o xcodeproj de project.yml; shell objc (main.m) chama showcase_ios_main
- [x] build_ios.sh (cargo staticlib + xcodegen + xcodebuild) e run_ios.sh (boot simulador, install, launch, screenshot)
- [x] showcase rodando no simulador ios (aarch64-apple-ios-sim)

### pendente
- device físico real
- validação visual formal (screenshot diff) contra o desktop
