---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-09-25
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
projeto gradle (`android/`) e o `build_android.sh` já existiam.

- APK e deploy: `script/capture-android` (task-77) compila o cdylib pelo
  cargo-ndk, monta o APK pelo gradle, sobe o emulador sem janela e
  instala.
- rendering e texto: o frame do emulador difere do desktop de mesma
  página, tema, viewport e escala em no máximo 2/255 por canal (média 1),
  fora a barra de navegação: o Vulkan do emulador é o llvmpipe, software.
- toque e gestos: a galeria não tratava `WindowEvent::Touch`, só mouse.
  agora (`crates/showcase/src/app/touch.rs`) o primeiro dedo é o ponteiro:
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
