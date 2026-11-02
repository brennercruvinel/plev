---
id: task-77/phase-1a-probe
title: "phase 1 (task-77a): the probe"
project: plev
type: section
parent: task-77
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [determinism, rendering, platform]
relations: [adr-0034, adr-0064, operations-validate-visuals-by-pixel]
evidence: [2dd2bc0d, c779fca6, 65d45235, 8af26af5, 35a47b26, a69cd464]
source: docs/log/steps/task-77-visual-determinism.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase 1 (task-77a): the probe

Records phase 1 of task-77: the probe of the targets (web, iOS, android and desktop), the desktop capture against itself and the divergences this machine reproduces, with the phase's handoff. The result is that the desktop came out bit-identical with itself, across 1312 frames and two processes, and that blur does not follow the scale; the cause was found and the fix was left for phase 1b. It matters because it defines the classes of difference that the comparator uses.

feita antes do código, no worktree da branch `task/77a-determinism`, base
`5207f9b8`. máquina: apple M4 (metal 4), macOS 26.6.2, dois displays (o
built-in liquid retina XDR, backing scale 2, e um DELL AW2518HF, backing
scale 1). toda afirmação sobre biblioteca vem da fonte da versão do
`Cargo.lock` (wgpu 28.0.0, wgpu-core 28.0.1, wgpu-hal 28.0.1, wgpu-types
28.0.0, winit 0.30.13), com arquivo e linha.

## (a) ferramentas, alvo por alvo

| alvo | o que existe | o que falta | o que o brenner roda (com rede) | tamanho aproximado |
|---|---|---|---|---|
| web | python playwright 1.62 (`/opt/homebrew/bin/playwright`), chrome-headless-shell 151.0.7922.34 em `~/Library/Caches/ms-playwright/chromium_headless_shell-1234`, node 26.7; alvo `wasm32-unknown-unknown` | `trunk`, `wasm-bindgen` (CLI), `wasm-opt`; Chrome.app (só o Safari.app está em /Applications) | `brew install trunk` e `cargo install --locked wasm-bindgen-cli --version 0.2.123` (a versão do lock); `brew install binaryen` só se o build release pedir o wasm-opt | trunk uns 10 MB; wasm-bindgen-cli uns 20 MB (compila alguns minutos); binaryen uns 40 MB |
| iOS | xcode 27.0 (27A266a), runtimes iOS 26.5 e 27.0, iPhone 17 Pro no 26.5 (`D2B562EB-DEFA-4B56-9AC9-FD6B82614ED2`, desligado), 1206x2622 a 3x (402x874 lógico, do `profile.plist` do device type) | o alvo rust `aarch64-apple-ios-sim`; o `xcodegen`, que o `tool/tasks/mobileapp.sh ios` chama e o orquestrador não listou | `rustup target add aarch64-apple-ios-sim` e `brew install xcodegen` | o rust-std do alvo, algumas dezenas de MB; xcodegen uns 10 MB |
| android | nada: sem SDK em `~/Library/Android/sdk`, sem `emulator`, `adb`, `sdkmanager`, sem JDK (`java` é o stub do macOS, "Unable to locate a Java Runtime"), sem `cargo-ndk`, sem NDK na raiz, sem alvos rust de android | tudo | `brew install --cask temurin@17 android-commandlinetools`; `sdkmanager "platform-tools" "emulator" "platforms;android-35" "system-images;android-35;google_apis;arm64-v8a" "ndk;27.2.12479018"`; `avdmanager create avd -n plev35 -k "system-images;android-35;google_apis;arm64-v8a"`; `cargo install cargo-ndk`; `rustup target add aarch64-linux-android x86_64-linux-android` | uns 4 GB no total (imagem do sistema uns 1,5 GB, NDK uns 1,5 GB, emulador uns 300 MB); o disco tem uns 15 GB livres |

achado da sonda (a): a web está mais perto do que parecia. o
chrome-headless-shell que já está instalado expõe WebGPU numa página
segura (`file://`), sem janela: com `--enable-unsafe-webgpu
--use-angle=metal` o adapter é `apple / metal-3` (a GPU desta máquina,
pelo dawn); sem `--use-angle=metal` é `google / swiftshader`, software. o
formato preferido do canvas é `bgra8unorm` nos dois. o script da web fixa
as flags e registra o adapter, senão a captura mede o swiftshader achando
que mede a GPU.

## (b) o desktop consigo mesmo

`rust/engine/tests/headless_determinism.rs` (novo): uma cena com cada
primitivo do frame path (rect, rounded rect com borda, gradiente com
stop transparente, sombra externa e interna, path preenchido e com
traço, imagem com raio, texto em três tamanhos dentro de um clip, uma
layer de vidro com backdrop blur e opacidade, uma layer com efeito de
blur), em coordenadas lógicas fracionárias de propósito. resultado neste
M4:

- dois contextos de GPU no mesmo processo dão os mesmos bytes em 1, 1.25,
  1.5 e 2;
- um renderer redimensionado por 2, 1.5, 1.25, 1, 1.5, 2 dá, em cada
  escala, os bytes de um renderer novo.

dois processos e todas as páginas da galeria: `regress capture`
(seção "a captura do desktop", mais abaixo). as 82 páginas nos quatro temas nas quatro escalas, 1312
frames, deram 1312 iguais em bit entre dois contextos, nos dois processos,
e 1312 iguais em bit entre os dois processos, o segundo desenhando na
ordem inversa (outro histórico de atlas e de resize). o desktop é
determinístico consigo mesmo nesta máquina: não houve parada.

## (c) a janela contra o headless

- formato. no metal a surface oferece `Bgra8Unorm`, `Bgra8UnormSrgb`,
  `Rgba16Float` e, quando dá, `Rgb10a2Unorm`
  (`wgpu-hal-28.0.1/src/metal/adapter.rs:361-372`); o wgpu-core põe os
  sRGB na frente (`wgpu-core-28.0.1/src/device/global.rs:56`); o engine
  pega o primeiro sRGB (`rust/engine/src/drawing/gpu/context.rs`,
  `surface_configuration`): `Bgra8UnormSrgb`. `add_srgb_suffix` nele é a
  identidade (`wgpu-types-28.0.0/src/texture/format.rs:1466`), então
  `view_formats` fica vazio. o headless desenha em `Rgba8UnormSrgb`. as
  layers seguem `surface_format()`: BGRA na janela, RGBA no headless. a
  diferença é a ordem dos canais; o encode linear para sRGB na escrita é o
  mesmo. evidência já registrada: `monster_player --capture` (a textura da
  surface lida antes do present) e `--frame --out` (headless) dão o mesmo
  sha256 (ADR headless-render-shares-the-frame-path). para as páginas da
  galeria isso não foi medido: pede janela, fica para a fase 2.
- a layer metal. `configure` (`wgpu-hal-28.0.1/src/metal/surface.rs:140-190`)
  define `opaque` pelo alpha mode (161-162; o engine pega o primeiro,
  `Opaque`), `pixelFormat` (168), `framebufferOnly` (169), EDR só para
  `Rgba16Float` (172-174), o número de drawables, o tamanho e o display
  sync. não define `colorspace`: a palavra não aparece em
  `wgpu-hal-28.0.1/src/metal/`. o winit 0.30.13 também não define espaço
  de cor na janela nem na view (só `setWantsLayer(true)`,
  `src/platform_impl/macos/window_delegate.rs:652`; `NSDeviceRGBColorSpace`
  só no cursor, `cursor.rs:45`). o default de `CAMetalLayer.colorspace`,
  medido nesta máquina com um `CAMetalLayer()` sem janela (swift): `nil`.
- na web o wgpu configura o canvas com formato, uso, alpha mode e view
  formats e não toca no espaço de cor (`wgpu-28.0.0/src/backend/webgpu.rs:3904-3942`),
  que fica no default do `GPUCanvasConfiguration`.
- o que a pessoa vê. os bytes da janela são os do baseline (sRGB de 8 bits),
  mas o que chega ao display passa pelo window server depois do present,
  fora do plev: a layer vai sem espaço de cor (nil), e o display built-in
  é P3 com perfil próprio. o manual de validação registra uma medida que
  cabe em "o window server trata o conteúdo como sRGB e converte para o
  perfil do display": numa captura de janela, #303030 leu (42,42,42) cru e
  (48,48,48) depois de converter do perfil embutido para sRGB
  ([`validate-visuals-by-pixel.md`](../../Research/Workflows/validate-visuals-by-pixel.md), "color profiles in
  captures"). cinza neutro só testa a curva de transferência, não o gamut.
  se o window server tratasse nil como "já está no espaço do display", um
  vermelho sRGB puro sairia vermelho P3, mais saturado. isso fica aberto:
  falta uma captura de janela com as primárias sRGB puras no display P3 e
  no DELL, convertida pelo perfil embutido, ou ler o `colorspace` da layer
  viva. a correção, se for preciso, é definir o espaço de cor da layer no
  engine (contrato de cor): não se muda nesta fase.
- a escala. no macOS o fator vem de `backingScaleFactor`
  (`winit-0.30.13/src/platform_impl/macos/view.rs:842-843`,
  `window_delegate.rs:1147-1148`), que é 1 ou 2 (2 no built-in, 1 no
  DELL). as resoluções "parece com" do macOS são o window server
  reamostrando um backing de 2x depois dos bytes: outra transformação que
  a pessoa vê e o baseline não. a janela calcula o lógico como físico
  dividido pela escala (`window/events.rs`, `logical_projection_size`), o
  headless o físico como lógico vezes a escala arredondado
  (`Viewport::physical`); para 1280x800 nas quatro escalas os dois batem.

## (d) os shaders

- antialiasing de borda. `rect_sdf.wgsl`, `image.wgsl`, `backdrop.wgsl` e
  o modo inset de `shadow_analytic.wgsl` fazem `1 - smoothstep(-0.5, 0.5,
  d)` sobre a distância do SDF em unidades lógicas (`p = uv * half_size`,
  com `half_size` em px lógicos, `compositor/layer/geometry.rs`,
  `emit_sdf_rect`). não há `fwidth`, `dpdx`, `dpdy` nem dithering em shader
  nenhum. a borda tem largura fixa de 1 px lógico, que são `escala` px
  físicos: 1 px a 1x, 2 px a 2x, 1,25 a 1.25x. por cima disso as layers
  desenham com MSAA 4x (`RenderConfig::default`) e resolvem numa textura
  sRGB de 8 bits.
- gradientes. `rect_sdf.wgsl` mistura dois stops em linear
  pré-multiplicado, `t` pela projeção do ponto na direção, sem dithering:
  a quantização é a do encode sRGB de 8 bits na escrita.
- derivadas implícitas. todo `textureSample` (texto, imagem, composite,
  blur, backdrop) usa sampler `Linear` em mag e min e textura de um mip só
  (`text/atlas.rs:41`, `gpu/image.rs:484`, `gpu/context.rs:419`,
  `effects/processor.rs:24`): a escolha de LOD por derivada não muda o
  resultado. os quads de glifo caem em px físicos inteiros
  (`text/atlas.rs`, `emit_glyphs`), então o texto amostra no centro do
  texel.
- o que pode variar por driver ou GPU, a medir na fase 2 contra este
  baseline: a precisão de `pow` (`srgb_to_linear` em quad, rect_sdf, text,
  shadow, shadow_analytic), de `exp`, `sqrt` e da `erf` polinomial da
  sombra analítica; a interpolação dos varyings; o encode sRGB e o blend na
  escrita em alvo de 8 bits; o resolve do MSAA num formato sRGB (em linear
  ou no espaço codificado); os pesos do filtro bilinear (a fração de
  subtexel) no composite e no blur. nada disso usa derivada. o esperado é
  delta de 1 ou 2 em borda, gradiente, sombra e vidro, e zero em cor chapada
  e no texto.
- achado que esta máquina reproduz: o blur não segue a escala. o
  `sigma` do backdrop blur e dos efeitos de layer `Blur` e `Shadow` passa
  inalterado da cena (px lógicos) até `EffectProcessor::apply_blur`
  (`window/render_passes.rs:228-238` e `482-504`), que o aplica em texels
  físicos (`effects/apply.rs:71-73`, `texel_size = 1 / largura física`,
  `gaussian_weights(sigma)` sem a escala). medido com uma cena
  descartável (borda branca e preta sob um backdrop blur de sigma 4): a
  rampa de 10% a 90% tem 9 px físicos em toda escala, 9 px lógicos a 1x, 6
  a 1.5x, 4,5 a 2x. o mesmo vidro fica mais nítido a 2x do que a 1x, e
  mais ainda no iPhone a 3x. a sombra analítica não tem isso (o sigma e a
  posição estão em px lógicos no shader). a correção é no engine, fora
  desta fase: vai no relatório de parada no fim.

## fase 1: a captura do desktop

`cargo run -p regress -- capture` desenha cada página
da galeria nos quatro temas que a tecla T percorre (core, hoff, dark,
light) nas escalas 1, 2, 1.25 e 1.5, a 1280x800 lógico, duas vezes, por
dois contextos de GPU do mesmo processo, e classifica o segundo frame
contra o primeiro com as linhas do desktop da tabela de diferenças
pedidas. `--against <dir>` classifica também contra o png do mesmo frame
escrito por outro processo; `--reverse` desenha na ordem inversa. os pngs,
`report.txt` (uma linha por página e tema, uma coluna por escala) e
`report.json` vão para `target/determinism/desktop`, fora do git. nenhum
baseline novo.

medido neste M4 (build de debug do exemplo, dependências em opt-level 2):

| execução | frames | contextos | processos | tempo |
|---|---|---|---|---|
| `capture` (escreve os pngs) | 1312 | 1312 iguais | não pedido | 68,5 s |
| `capture --out target/determinism/desktop-b --against target/determinism/desktop --reverse --no-png` | 1312 | 1312 iguais | 1312 iguais | 43,2 s |

zero pedidas, zero caladas, zero de tamanho diferente. os 1312 pngs
ocupam 183 MB.

## fase 1: as divergências que esta máquina reproduz

o desktop contra ele mesmo não diverge (1312 de 1312 iguais em bit, dois
contextos e dois processos). o que a sonda achou, com causa ou aberto:

1. o blur não segue a escala (causa achada; correção proibida nesta fase).
   `SceneNode::BackdropBlur.sigma` e os efeitos de layer `Blur` e `Shadow`
   chegam a `EffectProcessor::apply_blur` sem a escala
   (`compositor/layer/geometry.rs`, `DrawCommand::BackdropBlur`;
   `window/render_passes.rs:228-238` e `482-504`), e `apply_blur` aplica o
   kernel em texels físicos (`effects/apply.rs:71-73`: `texel_size = 1 /
   largura física`, `gaussian_weights(sigma)`). medido: a rampa de 10% a
   90% de uma borda branca e preta sob um backdrop blur de sigma 4 tem 9 px
   físicos em toda escala, 9 px lógicos a 1x, 6 a 1.5x, 4,5 a 2x. o mesmo
   vidro fica duas vezes mais nítido a 2x do que a 1x, três vezes no
   iPhone. a sombra analítica e o feather do SDF seguem a escala (estão em
   px lógicos no shader). reproduz com
   `cargo test -p engine --test headless_determinism -- --ignored`: o teste
   `a_backdrop_blur_reaches_as_far_in_logical_px_at_every_scale` falha hoje
   ("1.5x: the blur spreads 6 logical px, 9 at 1x") e é o que a correção
   tem que deixar verde. a correção é multiplicar o sigma pela escala do
   contexto (`clip_scale`) antes de `gaussian_weights`, e o kernel de 13
   taps corta em 6 texels, então a 2x um sigma acima de uns 3 lógicos
   pede mais taps ou um downsample: mexe no engine e no `blur.wgsl`, fora
   desta fase. vai no relatório de parada.
2. o espaço de cor da layer metal (aberto). o wgpu 28 e o winit 0.30.13
   nunca definem `colorspace` na `CAMetalLayer` (nil, medido), e a web
   deixa o canvas no default. os bytes não mudam com isso; o que muda é o
   que o window server mostra num display P3. falta para achar: uma
   captura de janela (`screencapture -l<id>`) das primárias sRGB puras no
   built-in e no DELL, convertida pelo perfil embutido, ou o `colorspace`
   lido da layer viva. pede janela, fase 2. se a resposta for "nil é
   tratado como o espaço do display", a correção é definir sRGB na layer:
   engine e contrato de cor, com aprovação.
3. a janela contra o headless nas páginas da galeria (aberto, esperado
   igual). a evidência é do monster player (sha256 igual) e da mesma
   função `draw_frame`; as páginas não foram capturadas por janela nesta
   fase (nenhuma janela real). falta: `monster_player --capture` tem o
   caminho (ler a textura da surface antes do present); o showcase não
   tem. fase 2.
4. entre GPUs e drivers (aberto, não reproduzível aqui: uma GPU só).
   candidatos pela leitura dos shaders, sem derivada em nenhum: precisão
   de `pow`, `exp`, `sqrt` e da `erf` polinomial; encode sRGB e blend em
   alvo de 8 bits; resolve do MSAA em formato sRGB; pesos do bilinear. a
   web no dawn sobre o mesmo M4 (flag `--use-angle=metal`) e o simulador
   de iOS (metal do host) são as primeiras comparações; o swiftshader do
   chromium sem a flag é o primeiro adapter de software.

## handoff da fase 1 (task-77a): o comparador, a captura do desktop e as divergências desta máquina

branch `task/77a-determinism`, base `5207f9b8`. commits, em ordem:

- `2dd2bc0d` task-77: the probe, the desktop against itself
- `c779fca6` task-77: the comparator classifies two frames
- `65d45235` task-77: the table of requested differences
- `8af26af5` task-77: the desktop captured against itself, and its report
- `35a47b26` task-77: the blur does not follow the scale (the cause, not the fix)
- `a69cd464` task-77: the capture scripts for web, iOS and android
- (este) task-77: ADR and the phase 1 handoff

```text
Goal: task-77 fase 1, determinismo visual (comparador, captura do desktop,
      divergências que esta máquina reproduz, scripts dos outros alvos)
Status: concluída; web, iOS e android com scripts prontos e sem captura
        (faltam ferramentas); uma correção de engine espera aprovação
```

### a sonda

colada acima ("fase 1 (task-77a): a sonda"). em uma linha por item: (a) a
web tem playwright e um chromium headless com WebGPU no metal, faltam
trunk e wasm-bindgen; o iOS tem xcode 27 e o simulador, faltam o alvo
`aarch64-apple-ios-sim` e o xcodegen; o android não tem nada. (b) o
desktop é igual em bit consigo mesmo: dois contextos, dois processos, 1,
2, 1.25 e 1.5. (c) a janela escreve os mesmos bytes sRGB que o headless
(BGRA contra RGBA), mas a layer metal vai sem `colorspace` (nil) e o que
a pessoa vê depende do window server e do display. (d) nenhum shader usa
derivada nem dithering; a borda é um feather fixo de 1 px lógico; o que
pode variar entre GPUs é precisão de função e de blend.

### o comparador e as classes

`tool/bench/regress/src/classify.rs`. dois frames da mesma
página, tema, viewport e escala:

- `Equal`: todo byte igual;
- `Requested { reasons }`: diferem, e todo pixel diferente cai numa área
  que uma decisão declara (faixa numa borda ou retângulo, em px lógicos,
  mapeados por centro de pixel como o `pixels.rs` faz com um controle);
  cada razão com o nome e os pixels que explica;
- `Silent { pixels, max_delta, mean_delta, bbox, explained }`: medido só
  sobre os pixels que nenhuma decisão explica; delta de um pixel é o maior
  dos quatro canais (0 a 255); bbox em px físicos;
- `Resized { base, other, reason }`: tamanhos diferentes não se comparam
  por pixel; a razão quando a tabela declara uma decisão de tamanho.

linha de comando: `regress compare <base.png> <other.png> [--scale <s>]
[--target <desktop|web|ios|android>] [--table <file>]`, saída 0 igual ou
pedido, 1 calado ou de outro tamanho, 2 pedido malformado, 3 erro. testes
com imagens sintéticas para cada classe, o mapeamento de área nas quatro
bordas a 1.25, e a leitura de dois pngs em disco.

### a tabela

`tool/bench/regress/requested.json`, formato
`determinism.requested.v1`. JSON e não TOML: o exemplo já lê serde_json, e
um leitor de TOML seria dependência nova. uma linha por decisão: alvo,
nome kebab-case, área (`edge`, `rect` ou `size`), `known` ou
`to_confirm`, razão numa linha. a leitura recusa alvo desconhecido, nome
repetido no alvo, razão em duas linhas, área vazia, duas linhas de
tamanho num alvo.

| alvo | nome | área | status |
|---|---|---|---|
| desktop | size | tamanho | known |
| web | size | tamanho | to_confirm |
| ios | status-bar | faixa no topo, 62 pt | to_confirm |
| ios | home-indicator | faixa embaixo, 34 pt | to_confirm |
| ios | size | tamanho (402x874 a 3) | to_confirm |
| android | status-bar | faixa no topo, 24 dp | to_confirm |
| android | navigation-bar | faixa embaixo, 48 dp | to_confirm |
| android | size | tamanho (densidade / 160) | to_confirm |

### o relatório do desktop em números

`regress capture`, apple M4 (metal), 1280x800 lógico: 82 páginas x 4
temas (core, hoff, dark, light) x 4 escalas (1, 2, 1.25, 1.5) = 1312
frames.

- dois contextos de GPU do mesmo processo: 1312 iguais, 0 pedidas, 0
  caladas, 0 de outro tamanho (68,5 s, escrevendo os pngs, 183 MB);
- segundo processo, ordem inversa, contra os pngs do primeiro: 1312
  iguais entre contextos e 1312 iguais entre processos (43,2 s).

reproduzir: `cargo run -p regress -- capture`, depois
`... capture --out target/determinism/desktop-b --against
target/determinism/desktop --reverse --no-png`. o relatório fica em
`target/determinism/<dir>/report.txt` e `report.json`.

### as divergências

detalhe na seção "as divergências que esta máquina reproduz" acima.

1. o blur não segue a escala: causa achada (sigma em texels físicos em
   `EffectProcessor::apply_blur`), medida (9, 6 e 4,5 px lógicos de
   espalhamento a 1x, 1.5x e 2x para sigma 4), correção proibida nesta
   fase; teste ignorado que a correção tem que deixar verde.
2. o `colorspace` da layer metal: aberto, falta captura de janela num
   display P3.
3. a janela contra o headless nas páginas da galeria: aberto, esperado
   igual, falta um caminho de captura de surface no showcase.
4. entre GPUs e drivers: aberto, uma GPU só nesta máquina.

### relatório de parada: o blur e a escala

- problema: o backdrop blur e os efeitos de layer `Blur` e `Shadow`
  aplicam o sigma em texels físicos; o mesmo vidro espalha 9 px lógicos a
  1x, 6 a 1.5x, 4,5 a 2x, e ficaria em 3 no iPhone a 3x. uma tela igual
  em px lógicos fica diferente por escala, sem ninguém ter pedido.
- opções: (a) multiplicar o sigma pela escala do contexto
  (`clip_scale`) antes de `gaussian_weights`, e dar ao `blur.wgsl` taps
  que alcancem uns 3 sigma (hoje 13 taps cortam em 6 texels, 1,5 sigma a
  1x para sigma 4); (b) desfocar num alvo reduzido pela escala e
  reamostrar, o que mantém os 13 taps; (c) aceitar e declarar como decisão
  de produto na tabela (não recomendo: é uma diferença que ninguém
  escolheu).
- impacto: (a) e (b) mexem no engine (`effects/apply.rs`,
  `window/render_passes.rs`) e no shader, mudam os pixels do vidro em toda
  escala que não é 1 e, em (a), também a 1x se o kernel crescer; o
  `regress check` não muda (os hashes são da cena), as imagens de baseline
  do todo não têm backdrop blur (conferir no bless); (b) custa um
  downsample e um upsample por backdrop. (c) não custa nada e deixa a
  divergência.
- recomendação: (a), numa task de engine com aprovação, com o teste
  `a_backdrop_blur_reaches_as_far_in_logical_px_at_every_scale` como
  critério; o custo de mais taps medido com `perf::stages`.
- decisão necessária: aprovar a mudança de engine e shader, e em que
  task ela entra.

### os scripts e o que o brenner instala

`tool/tasks/screencap.sh web`, `tool/tasks/screencap.sh ios`, `tool/tasks/screencap.sh android`, com
`tool/tasks/screencap.sh`. cada um checa as ferramentas primeiro e para
com o comando que instala o que falta (rodados nesta máquina: os três
param ali, código 2, sem compilar nada). depois compilam pelo caminho das
tasks 18, 19 e 20 (`tool/tasks/mobileapp.sh android`, `tool/tasks/mobileapp.sh ios`,
`tool/tasks/webtarget.sh build --release`), capturam um frame, desenham o frame do
desktop da mesma página, tema, viewport e escala com `regress capture`, e
classificam com `regress compare --target`. a cadeia desktop mais compare
foi exercitada com um png do próprio desktop a 402x874 a 3: igual deu 0;
a faixa do status bar mexida deu `requested (status-bar)`; um pixel a
mais no meio deu `silent 1 px`, código 1.

- web: `brew install trunk`; `cargo install --locked wasm-bindgen-cli
  --version 0.2.123`; `brew install binaryen` se o build release pedir o
  wasm-opt. uns 30 a 70 MB. o chromium headless e o playwright já estão
  na máquina.
- iOS: `rustup target add aarch64-apple-ios-sim`; `brew install xcodegen`.
  algumas dezenas de MB.
- android: `brew install --cask temurin@17 android-commandlinetools`;
  `sdkmanager "platform-tools" "emulator" "platforms;android-35"
  "system-images;android-35;google_apis;arm64-v8a" "ndk;27.2.12479018"`;
  `avdmanager create avd -n plev35 -k
  "system-images;android-35;google_apis;arm64-v8a"`; `cargo install
  cargo-ndk`; `rustup target add aarch64-linux-android
  x86_64-linux-android`. uns 4 GB; o `build_android.sh` lê `ANDROID_HOME`
  e `JAVA_HOME` (default `~/Library/Android/sdk` e `~/jdk17`), então exportar
  os caminhos do cask. o primeiro gradle baixa a distribuição.

### gate e regress

- antes (base `5207f9b8`): workspace 2672 passaram, 17 ignorados; urnaui
  85 passaram, 1 ignorado; total 2757.
- depois: workspace 2687 passaram, 18 ignorados; urnaui 85 e 1; total
  2772. os 15 novos: 13 no exemplo `regress` (31 para 44) e 2 em
  `rust/engine/tests/headless_determinism.rs`; o ignorado novo é o do
  blur. `./tool/tasks/fullcheck.sh` verde com as pernas wasm e urnaui.
- `regress check`: 163 baselines, 163 inalterados, nenhum frame, nenhuma
  GPU; a saída é a de antes fora os tempos.
- `cargo check --target wasm32-unknown-unknown -p uigallery --example
  regress` verde. iOS e android não compilam aqui (alvos ausentes); o
  código novo fica todo atrás do mesmo `cfg` desktop do exemplo, e o `src`
  do showcase não mudou.

### a linha do ADR para o índice

```text
| 2026-09-24 | [what-identical-means](what-identical-means.md) | proposed | idêntico por classe de comparação: o desktop contra ele mesmo em todo byte (1312 de 1312 frames, 82 páginas, 4 temas, escalas 1, 2, 1.25 e 1.5, dois contextos e dois processos no M4); outro alvo na mesma GPU em todo byte fora das áreas declaradas; entre GPUs delta de canal medido, proposto em no máximo 2, zero esperado dentro de cor chapada e glifo; entre escalas não se compara por pixel; o veredito é `equal`, `requested`, `silent` (pixels, delta máximo e médio, retângulo) ou `resized`; as decisões de produto numa tabela versionada (`tool/bench/regress/requested.json`), web, iOS e android `to_confirm`; idêntico para nos bytes da surface, color management do sistema fica fora; `regress compare` e `regress capture` (task-77 fase 1) |
```

### decisões

- a tabela em JSON (sem dependência nova).
- os quatro temas são os que a tecla T percorre (core, hoff, dark, light).
- a captura escreve em `target/determinism/`, nada no git; nenhum baseline
  novo.
- o segundo processo desenha na ordem inversa, para que a comparação entre
  processos também cubra outro histórico de atlas e de resize.
- um delta dentro do limite nunca vira `equal`: o comparador diz `silent`
  com os números, e o limite decide um job.

### limitações e riscos

- nenhuma captura real de web, iOS e android; as linhas desses alvos na
  tabela são `to_confirm` e os números das faixas (62, 34, 24, 48) são os
  do comportamento de cada plataforma, a medir na primeira captura.
- a web e o android só escolhem página e tema por tecla: as dez primeiras
  páginas e os quatro temas. qualquer página exige um canal de lançamento
  (query na URL, extra do intent), que é mudança no `src` do showcase.
- o screenshot do chromium headless e o do simulador podem vir com perfil
  de cor; o script converte para sRGB com `sips` quando o perfil não é
  sRGB, e diz qual achou. sem medida ainda.
- o `target/agents/one` é compartilhado entre checkouts: numa rodada de
  teste dos scripts, um `REPO_ROOT` errado (corrigido com `BASH_SOURCE`)
  fez o cargo compilar o `regress` do checkout principal nesse diretório, e
  o `cargo run` seguinte deste worktree executou aquele binário (o
  fingerprint local estava fresco e o cargo não recopiou o executável).
  resolvido com `touch` e novo build; vale para qualquer agente que
  compartilhe o diretório.

### o que fica para a fase 2

- as capturas reais: web (depois de trunk e wasm-bindgen; o chromium e o
  playwright já estão), iOS (depois do alvo e do xcodegen), android
  (depois do SDK). cada uma confirma ou corrige as linhas `to_confirm` da
  tabela na mesma mudança e mede o primeiro delta entre GPUs (dawn sobre
  metal, simulador sobre a GPU do host, emulador).
- um canal de lançamento de página e tema na web e no android.
- a janela contra o headless nas páginas (ler a textura da surface antes
  do present, como o `monster_player --capture`), e o `colorspace` da
  layer medido num display P3 com as primárias sRGB puras.
- a correção do blur, se aprovada, com o teste ignorado como critério.
- o job de CI no mac com `regress capture` exigindo `equal` em todo frame,
  quando o CI voltar.
