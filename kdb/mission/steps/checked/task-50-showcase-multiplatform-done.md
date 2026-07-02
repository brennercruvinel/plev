---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-06-16
domain: task-tracking
---

# task-50: showcase multiplataforma como lib (desktop/web/android/ios)

## objetivo
rodar o showcase (a galeria do design system) nativamente em desktop, web, android e ios a partir do mesmo codigo rust, empacotado como lib, sem nenhuma ui de kotlin ou swift. cada pixel desenhado na gpu pela engine.

## dependencias
- task-11 (android build + lifecycle)
- task-12 (ios build + lifecycle)
- task-06 (wasm/webgpu)
- task-18, task-19 (device e simulator test)

## contexto
a engine ja compilava nos quatro targets, mas faltava um app real multiplataforma saindo de um codigo unico. o showcase virou esse caso: uma lib com um app shell que expoe um entry por plataforma.

## o que foi entregue
- showcase como lib (crate-type lib + cdylib + staticlib). entries por plataforma: `run` (desktop), `run_web` (wasm), `android_main` (android), `showcase_ios_main` (ios).
- android: host GameActivity (`MainActivity.kt`) carrega `libshowcase.so` e entrega para o `android_main` rust; gradle kts + cargo-ndk geram jniLibs (arm64-v8a, x86_64) e o apk. `build_android.sh`.
- ios: shell objc fino (`main.m`) chama `showcase_ios_main`; xcodegen gera o xcodeproj a partir de `project.yml`; `build_ios.sh` (cargo staticlib + xcodegen + xcodebuild) e `run_ios.sh` (boot simulador, install, launch, screenshot).
- features `android-entry` e `web-entry` off por padrao, para apps downstream definirem o proprio entry sem clash de simbolo (a engine guarda o seu `android_main` atras de `android-entry`).

## numeros honestos
- apk buildando (`app-debug.apk`), showcase rodando no simulador ios e no emulador android, mesma cena que desktop e web.
- device fisico ainda nao validado (task-18 segue parcial). ios e o elo mais imaturo da stack (ver notes do experimento mon).

## referencias
- commits cb6643d (showcase como lib), a2c846d (feature android-entry), 75861fe (android apk), 7428245 (ios xcodegen no simulador), 33ef42c (arc com estrutura mobile)

## fora de escopo
- teste em device fisico
- paridade de maturidade do ios com o desktop
