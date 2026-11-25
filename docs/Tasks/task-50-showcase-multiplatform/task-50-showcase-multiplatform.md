---
id: task-50
title: "task-50: multiplatform showcase as a lib (desktop/web/android/ios)"
project: plev
type: task
status: done
date: 2025-11-14
date_basis: reconstructed
recorded: 2026-06-16
topics: [platform, mobile]
relations: [task-06, task-11, task-12, task-18, task-19]
evidence: [cb6643d, a2c846d, 75861fe, 7428245, 33ef42c]
source: [docs/log/steps/task-50-showcase-multiplatform.md, docs/log/changes/task-50.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-50: multiplatform showcase as a lib (desktop/web/android/ios)

Records the task that got the showcase running as a lib on all four targets, desktop, web, Android and iOS, with the same scene and one entry point per platform. The result is an APK that compiles and the scene running on the iOS simulator and the Android emulator, with no UI code in Kotlin or Swift. Validation on a physical device has not been done, and iOS is the least mature link in the stack.

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

## Delivery record

### lib + entries
- [x] showcase como lib (crate-type lib + cdylib + staticlib)
- [x] entry desktop (run), web (run_web), android (android_main), ios (showcase_ios_main)
- [x] features android-entry / web-entry off por padrao (sem clash de simbolo)

### android
- [x] host GameActivity (MainActivity.kt) carrega libshowcase.so
- [x] cargo-ndk -> jniLibs (arm64-v8a, x86_64)
- [x] gradle kts -> app-debug.apk
- [x] build_android.sh

### ios
- [x] shell objc (main.m) chama showcase_ios_main
- [x] xcodegen gera xcodeproj de project.yml
- [x] build_ios.sh (staticlib + xcodegen + xcodebuild)
- [x] run_ios.sh (boot simulador, install, launch, screenshot)

### validacao
- [x] rodando no simulador ios e no emulador android, mesma cena que desktop/web
- [ ] device fisico (task-18 segue parcial)

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [task-06: WASM/webgpu validation in the browser](../task-06-wasm-webgpu-validation-in-the-browser/task-06-wasm-webgpu-validation-in-the-browser.md)
- [task-11: android build + lifecycle](../task-11-android-build-lifecycle/task-11-android-build-lifecycle.md)
- [task-12: ios build + lifecycle](../task-12-ios-build-lifecycle/task-12-ios-build-lifecycle.md)
- [task-18: android build & device test](../task-18-android-device-test/task-18-android-device-test.md)
- [task-19: ios simulator test](../task-19-ios-simulator-test/task-19-ios-simulator-test.md)
<!-- /related -->
