---
id: task-18
title: "task-18: android build & device test"
project: plev
type: task
status: done
date: 2026-09-25
date_basis: header
recorded: 2026-09-25
topics: [mobile, platform, input]
relations: [task-11, task-50, task-77]
source: [docs/log/steps/task-18-android-device-test.md, docs/log/changes/task-18.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-18: android build & device test

Records the build of plev for Android and the gallery test on the emulator: the library is built with cargo-ndk, the APK with gradle and the installation with `tool/tasks/screencap.sh android`. The emulator frame differs from desktop by at most 2/255 per channel, and touch, rotation and return from the background work. Left open are the activity the system recreates right after boot and dragging controls by touch. It matters because it shows the same gallery scene running on Android, as it already ran on desktop and web.

## objetivo
compilar, gerar APK e testar em device/emulator android.

## dependências
- task-11 (android build code)
- NDK instalado

## checklist
- [x] `cargo check --target aarch64-linux-android --features android-game-activity` compila
- [x] `cargo ndk -t arm64-v8a build --features android-game-activity` compila (.so)
- [x] fix: `#[unsafe(no_mangle)]` para rust edition 2024
- [x] gerar APK wrapper (cargo-apk ou gradle project)
- [x] deploy no device/emulator
- [x] verificar: quad rendering, text, input touch, gestures
- [x] testar lifecycle: background/foreground, rotate, memory warning

## notas
- NDK: `ndk/27.2.12479018/` (raiz do repo, gitignored)
- cargo-ndk v4.1.2 instalado
- emulator + system-images arm64 API 35 disponíveis
- APK wrapper é bloqueio para deploy

## fechamento (orquestrador, 2026-09-25)

instalados com o ok do brenner: JDK 17 (`openjdk@17`, `~/jdk17`), SDK
com platform-tools, emulador, android-35, NDK 27.2.12479018 e a imagem
arm64 google_apis; o AVD `plev35` é um Pixel 7 (1080x2400, 420 dpi). o
projeto gradle (`apps/uigallery/android/`) e o `build_android.sh` já existiam.

- APK e deploy: `tool/tasks/screencap.sh android` (task-77) compila o cdylib pelo
  cargo-ndk, monta o APK pelo gradle, sobe o emulador sem janela e
  instala.
- rendering e texto: o frame do emulador difere do desktop de mesma
  página, tema, viewport e escala em no máximo 2/255 por canal (média 1),
  fora a barra de navegação: o Vulkan do emulador é o llvmpipe, software.
- toque e gestos: a galeria não tratava `WindowEvent::Touch`, só mouse.
  agora (`apps/uigallery/src/shell/system/touch.rs`) o primeiro dedo é o ponteiro:
  um toque é um clique onde o dedo encostou (pressão e soltura no
  levantar, porque um card de paleta age na pressão), e um arrasto além
  de 8 px lógicos rola a página e não clica nada. no emulador, tocar no
  card "hoff" aplica o tema; arrastar rola sem trocar o tema.
- lifecycle:
  - segundo plano e volta: morria. ao voltar da tela inicial a superfície
    velha era configurada de novo e o wgpu recusava ("Surface does not
    support the adapter's queue family"), matando a thread de desenho.
    agora, no android, dormir larga a superfície (`Windows::suspend`) e
    acordar cria a janela e a superfície de novo no device mantido
    (`Windows::resume`); o `recreate_surface` do engine usa a instância do
    device em vez de uma nova. mesmo processo, e o frame de volta é igual
    ao de antes de sair.
  - rotação (`settings put system user_rotation`): a paisagem troca o
    layout, a volta ao retrato também, sem recriar a activity.
  - aviso de memória (`am send-trim-memory ... RUNNING_CRITICAL`): o app
    segue de pé e o frame não muda.
- o `capture-android` apertava T duas vezes quando não devia: o `seq` do
  macOS conta para trás (`seq 1 0` dá "1 0").

pendências:
- uma activity recriada pelo sistema (na primeira troca de cores do
  sistema logo depois do boot) não desenha mais: a velha é destruída e a
  nova nunca começa. o `capture-android` espera 20 s depois do boot.
- arrastar um controle (um slider) por toque ainda não existe: o arrasto
  rola a página.
- o toque no iOS usa o mesmo caminho e não foi testado: o `simctl` não
  toca na tela.

## Delivery record

### sessão 1 (2021-11-17)

#### compilação
- `cargo check --target aarch64-linux-android --features android-game-activity`
- fix: `#[no_mangle]` -> `#[unsafe(no_mangle)]` (rust edition 2024 requirement)
- `cargo ndk -t arm64-v8a build --features android-game-activity` (full build, 27s)
- NDK 27.2.12479018 com toolchain darwin-x86_64, API level 33

#### ambiente
- NDK em `ndk/27.2.12479018/` (raiz do repo, gitignored)
- emulator + system-images/android-35/google_apis/arm64-v8a disponíveis
- platform-tools/adb disponível (v37.0.0)
- cargo-ndk v4.1.2 instalado

#### pendente
- APK wrapper necessário para deploy no emulator (cargo-apk ou gradle)
- deploy e teste visual no device/emulator
- lifecycle testing (background/foreground, rotate)

#### warnings (esperados)
- `mut attrs` unused em window.rs (cfg-condicional, usado em desktop/ios mas não android)

### sessão 2 (2025-11-26)

#### avanço via task-50 (showcase multiplataforma)
- [x] APK wrapper resolvido: gradle kts + cargo-ndk geram jniLibs (arm64-v8a, x86_64) e o app-debug.apk
- [x] host GameActivity (MainActivity.kt) carrega libshowcase.so e entrega ao android_main
- [x] showcase rodando no emulador android (mesma cena que desktop e web)

#### pendente
- deploy e teste em device físico real
- lifecycle testing completo (background/foreground, rotate)

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0060: a covered window does not draw, and wakes with a new frame](../../adr/Rendering/0060-a-covered-window-does-not-draw-and-wakes-new/0060-a-covered-window-does-not-draw-and-wakes-new.md)
- [task-11: android build + lifecycle](../task-11-android-build-lifecycle/task-11-android-build-lifecycle.md)
- [task-50: multiplatform showcase as a lib (desktop/web/android/ios)](../task-50-showcase-multiplatform/task-50-showcase-multiplatform.md)
- [task-75: production-ready platform accessibility](../task-75-platform-accessibility/task-75-platform-accessibility.md)
- [task-77: visual determinism](../task-77-visual-determinism/task-77-visual-determinism.md)
- [wrap-up (orchestrator, 2026-09-25)](../task-77-visual-determinism/closing.md)
- [program: from GPU UI engine to generative interaction runtime](../../Research/Synthesis/generative-runtime-program.md)
- [autonomous cycle journal](../../Research/Workflows/autonomous-cycle-journal.md)
<!-- /related -->
