---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-06-16
domain: changelog
---

# task-50 changelog: showcase multiplataforma como lib

## lib + entries
- [x] showcase como lib (crate-type lib + cdylib + staticlib)
- [x] entry desktop (run), web (run_web), android (android_main), ios (showcase_ios_main)
- [x] features android-entry / web-entry off por padrao (sem clash de simbolo)

## android
- [x] host GameActivity (MainActivity.kt) carrega libshowcase.so
- [x] cargo-ndk -> jniLibs (arm64-v8a, x86_64)
- [x] gradle kts -> app-debug.apk
- [x] build_android.sh

## ios
- [x] shell objc (main.m) chama showcase_ios_main
- [x] xcodegen gera xcodeproj de project.yml
- [x] build_ios.sh (staticlib + xcodegen + xcodebuild)
- [x] run_ios.sh (boot simulador, install, launch, screenshot)

## validacao
- [x] rodando no simulador ios e no emulador android, mesma cena que desktop/web
- [ ] device fisico (task-18 segue parcial)
