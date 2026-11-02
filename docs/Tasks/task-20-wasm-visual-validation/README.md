---
id: task-20
title: "task-20: WASM visual validation"
project: plev
type: task
status: done
date: 2026-09-25
date_basis: header
recorded: 2026-09-25
topics: [web-wasm, gpu, performance]
relations: [task-06, task-77]
source: [docs/log/steps/task-20-wasm-visual-validation.md, docs/log/changes/task-20.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-20: WASM visual validation

Records the validation of the WASM target with WebGPU in the browser: the release WASM takes up 2,4 MB, and the capture runs in headless Chromium with WebGPU over Metal. The measured captures match the desktop captures of the same page byte for byte, and scrolling Typography and Effects stays at 8,3 ms between frames (p50), at 120 Hz. It matters because it proves parity between web and native, an item that was on the checklist, and records three defects that existed only on the web and were fixed.

## objetivo
validar rendering WASM/webgpu no browser.

## dependências
- task-06 (WASM validation code)

## checklist
- [x] `cargo check --target wasm32-unknown-unknown` compila
- [x] `trunk build` funciona (dev)
- [x] `trunk build --release`, bundle size: **2.4mb** (wasm-opt z)
- [x] `trunk serve` + visual validation no chrome com webgpu
- [x] rendering idêntico ao native
- [x] showcase scene renderiza corretamente
- [x] FPS/performance check

## fechamento (orquestrador, 2026-09-25)

fechada pelo checklist, item por item. o `trunk` e o `wasm-bindgen-cli
0.2.123` (a versão do `Cargo.lock`) foram instalados com o ok do brenner.

- validação visual no chrome com WebGPU: `tool/tasks/screencap.sh web` (task-77)
  serve o `web/dist` do `tool/tasks/webtarget.sh build --release` e abre no chromium
  headless do playwright com WebGPU no Metal (adapter `apple / metal-3`,
  não o swiftshader).
- idêntico ao native: 12 capturas iguais em todo byte ao frame do desktop
  da mesma página, tema, viewport e escala: as dez páginas que o teclado
  alcança, nos quatro temas, em 1x, 2x e 3x (390x844 a 3x no Avatar).
- a cena renderiza: renderizava só a cor de fundo. três defeitos, todos
  só da web:
  - o `trunk` recusava o build: o cdylib (o `.so` do android) e o bin
    `showcase` têm o mesmo nome, e o trunk escolhe o artefato pelo nome
    (`trunk-0.21.14/src/pipelines/rust/mod.rs:825-857`). a web ganhou um
    bin próprio, `showcase-web` (`apps/uigallery/src/bin/web.rs`), e o
    pacote um `default-run = "showcase"`.
  - o app morria ao abrir: o worker do Silences pedia
    `std::env::temp_dir()`, que entra em pânico no wasm32.
  - a tela vazia: a GPU chega depois de o canvas ter tamanho, e o surface
    configurado em 0x0 não acompanhava (`WindowHost::attach_gpu` agora
    redimensiona quando o tamanho difere; no desktop são iguais e nada
    muda). e a passada de composição do backdrop blur usava o formato cru
    do surface (`Bgra8Unorm`) em vez do formato de render (a view sRGB), e
    a WebGPU descartava todo frame da página Effects.
  - na captura, o `device_scale_factor` do playwright sozinho não muda o
    `devicePixelContentBoxSize` que o winit lê, e a 2x o app recebia o
    tamanho CSS como físico. o `capture-web` passa também
    `--force-device-scale-factor`.
- FPS: rolando Typography e Effects por 2 s, a 1x e a 2x, o intervalo
  entre frames do navegador fica em 8,3 ms (p50) e até 9,4 ms (p95), a
  120 Hz, sem nenhuma long task (acima de 50 ms).

pendências: as páginas depois da décima não têm canal de lançamento na web
(`capture-web` diz isso); o `trunk serve` em modo watch não foi usado, a
captura serve o `dist` por `python3 -m http.server`.

## Delivery record

### sessão 1 (2021-12-01)

#### build
- `cargo check --target wasm32-unknown-unknown`
- `trunk build` (dev, 0.09s cached)
- `trunk build --release` (wasm-opt z, 30.5s)

#### bundle size
- WASM bundle: **2.4mb** (release, wasm-opt z)
- total dist: 2.5mb

#### pendente
- `trunk serve` visual validation no chrome com webgpu
- screenshot comparativo native vs WASM
- FPS counter / performance check
- showcase scene rendering verification

#### warnings (esperados)
- `mut gpu` unused (cfg-condicional, usado no native mas não WASM)
- `touch_input` field never read (será usado quando input integration completa)

### sessão 2 (2026-01-13)

#### avanço
- [x] trunk serve com showcase rodando no browser (webgpu)
- [x] web bate o desktop por amostra de pixel (fundo 48,48,48), via render em view sRGB
- [x] prime creatures também roda na web

#### pendente
- screenshot diff formal native vs web em critical paths com baseline versionado
