---
project: phi
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-03-08
domain: task-tracking
---

# task-19: ios simulator test

## objetivo
build e teste visual no ios simulator.

## dependências
- task-12 (ios build code)

## checklist
- [x] `cargo check --target aarch64-apple-ios-sim` compila (zero warnings)
- [ ] `cargo build --target aarch64-apple-ios-sim` compila (requer xcode com ios SDK)
- [ ] deploy no simulator via ios-sim.sh
- [ ] verificar: quad rendering, text, metal pipeline
- [ ] testar: safe areas, orientação, lifecycle

## bloqueio
- apenas command line tools instalado, uikit framework não encontrado no link
- requer xcode.app completo para obter ios SDK
