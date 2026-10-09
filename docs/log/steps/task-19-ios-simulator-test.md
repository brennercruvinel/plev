---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-09-25
domain: task-tracking
---

# task-19: ios simulator test

## objetivo
build e teste visual no ios simulator.

## dependências
- task-12 (ios build code)

## checklist
- [x] `cargo check --target aarch64-apple-ios-sim` compila (zero warnings)
- [x] `cargo build --target aarch64-apple-ios-sim` compila (requer xcode com ios SDK)
- [x] deploy no simulator (`ios/showcase/build_ios.sh` e `simctl install`/`launch`, pelo `script/capture-ios`)
- [x] verificar: quad rendering, text, metal pipeline
- [x] testar: safe areas, orientação, lifecycle (orientação não testada, ver o fechamento)

## bloqueio
- apenas command line tools instalado, uikit framework não encontrado no link
- requer xcode.app completo para obter ios SDK

## fechamento (orquestrador, 2026-09-25)

o bloqueio antigo (só command line tools) não existe mais: o Xcode está
instalado, com os runtimes iOS 26.5 e 27.0. `xcodegen` e o target
`aarch64-apple-ios-sim` foram instalados com o ok do brenner.

- build e deploy: `script/capture-ios` (task-77) compila o staticlib, gera
  o projeto, compila o app e instala no simulador "iPhone 17 Pro"
  (402x874 a 3x), sem abrir o Simulator.app.
- rendering: o frame do simulador é igual ao do desktop de mesma página,
  tema, viewport e escala em todo pixel, fora a faixa da barra de status
  (diferença declarada em `crates/showcase/fixtures/determinism/requested.json`).
  dois defeitos antes disso:
  - o `capture-ios` lia o tamanho da tela de `mainScreenWidth` no
    `profile.plist`, que os perfis do Xcode 26 não têm mais; agora cai no
    `capabilities.plist` (`ScreenDimensionsCapability`).
  - a janela nascia com o tamanho fixo do desktop (1200x800): no iOS o
    winit faz desse tamanho o frame da UIWindow
    (`winit-0.30.13/src/platform_impl/ios/window.rs:511`), e a galeria saía
    larga, cortada à direita e com uma faixa preta embaixo. o tamanho fixo
    ficou só no desktop.
- lifecycle: com o app aberto, abrir o Ajustes e voltar pelo `simctl
  launch` retoma o mesmo processo (mesmo pid), e o frame continua igual
  ao do desktop fora da barra de status.
- safe areas: testadas; a galeria não recua para a área segura, e a barra
  de status e a ilha dinâmica ficam por cima do título.

pendências:
- a galeria não usa a área segura no iOS (o host já sabe o recuo:
  `SafeAreaInsets::from_window`).
- orientação: o `simctl` não gira o aparelho; girar pede a janela do
  Simulator.app (Cmd+seta), fora do que o orquestrador faz na máquina.
