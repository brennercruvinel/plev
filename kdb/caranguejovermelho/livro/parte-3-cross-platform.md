---
title: um codebase, varios mundos
parte: 3
status: stub
rastros: []
---

# parte 3, um codebase, varios mundos

espinha de secoes. so a estrutura, sem corpo. um codebase, renderizacao
identica em cada target, e os asteriscos honestos do que ainda nao roda.

## 3.1 macos/metal, o alvo nativo

### winit 0.30 e o app runner
### metal via wgpu, o caminho mais maduro
### responsiveness-multiplatform-and-fidelity

## 3.2 web/webgpu, init async e a entrada wasm unica

### canvas wasm e o run_web do showcase
### async-gpu-init-and-single-wasm-entry
### a entrada unica, web-entry vs android-entry

## 3.3 wasm validation, limits, build guards

### wasm-webgpu-validation, limits default vs webgl2
### build guards por cfg
### o build release de 2.4mb

## 3.4 android, GameActivity e cargo-ndk

### GameActivity host, libshowcase.so
### cargo-ndk, jniLibs, gradle apk
### android_main e a feature android-entry

## 3.5 ios, o shell objc, metal e fontes embedded

### ios-build, shell objc chamando showcase_ios_main
### fontes embedded, lifecycle handlers
### staticlib, xcodegen, simulador

## 3.6 mobile specifics, safe areas, IME, scale factor

### mobile-specifics, safe area insets por cfg
### a maquina de estados do IME
### scale factor e a heuristica de teclado

## 3.7 hidpi, a projecao em displays retina

### adr-004-hidpi-projection
### layout em pixel logico, set_projection

## 3.8 os asteriscos honestos, o que ainda nao roda

### android-emulator-deploy, GPU host obrigatorio
### linux/vulkan e windows/d3d12 pendentes
### o que e parcial e por que (responsiveness-multiplatform-and-fidelity)
