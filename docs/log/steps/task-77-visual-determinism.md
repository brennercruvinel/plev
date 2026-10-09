---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-09-25
domain: generative-runtime
task: task-77
wave: trilha
depends: [task-66]
consumes: [task-18, task-19, task-20]
branch: task/77-determinism
---

# task-77: determinismo visual

## objetivo

mesma UI, mesmo layout, mesmas fontes, mesmos pixels em macOS, Windows,
Linux, browser e mobile, na medida em que isso serve a previsibilidade,
teste e reprodutibilidade. e, onde a igualdade não acontece, o plev sabe
dizer se a diferença foi pedida pelo produto ou apareceu por divergência
silenciosa entre plataformas.

## ligação com a tese

seção 23. o mantra das anotações: "one pixel-identical frame on every
target", sabendo que fonts, drivers, GPU, DPI, text rasterization e color
management tornam isso infernal. e a correção da mentoria: determinismo
serve previsibilidade, não vira dogma.

## estado atual no código

- contratos de cor: srgb decodificado uma vez ao entrar na GPU, codificado
  uma vez na escrita da surface (`surface_render_view`, `to_linear_array`);
  mix de cor em espaço linear pré-multiplicado nos shaders.
- raster de glifo determinístico, só fontes embutidas (ADR
  `glyph-raster-identity-and-atlas-isolation`,
  `rust/engine/src/drawing/text/fonts.rs`).
- web declarado igual ao desktop; a validação visual do wasm está em
  andamento (task-20); android e ios em andamento (tasks 18 e 19); linux e
  windows pendentes (`docs/ATLAS.toml`, targets).
- depois da 66: render headless no desktop.

## escopo

1. harness de captura por alvo, sempre da mesma página, tema, viewport e
   escala: desktop pelo headless (66); web por navegador com WebGPU
   controlado por automação sobre o build do trunk; iOS pelo simulador
   (`tool/tasks/mobileapp.sh ios-run` já tira screenshot); Android pelo emulador.
2. comparação com classificação: igual em bit; diferente por decisão do
   produto (safe area, escala, chrome de plataforma) declarada numa tabela
   versionada; diferente por divergência silenciosa (o que precisa de
   causa).
3. investigação das divergências silenciosas por classe: antialiasing de
   borda em SDF (derivadas e `fwidth` por driver), dithering de gradiente,
   color management (espaço de cor da layer Metal, displays P3), escala
   fracionária de DPI, raster de texto.
4. ADR definindo o que "idêntico" quer dizer por classe: bit a bit onde é
   possível, diferença limitada e medida (delta de cor por pixel, número de
   pixels) onde não é, e a lista do que é decisão de produto.
5. o CI roda a comparação nos alvos que o CI consegue rodar e registra os
   outros como manuais.

## fora de escopo

bring-up de linux e windows (tarefa própria quando o alvo for priorizado);
fontes de sistema (decisão da task-74).

## dependências

66 integrada. consome os resultados das tasks 18, 19 e 20 sem duplicá-las.

## superfície sob responsabilidade

o harness de captura e comparação (exemplo ou binário), scripts de
captura em `script/`, correções de shader quando a causa for encontrada
(com aprovação, porque shader é superfície compartilhada), ADR
`docs/adr/workflows/0064-what-identical-means.md`.

## interfaces que consome

headless (66), os scripts de mobile e web existentes.

## interfaces que pode alterar

shaders e caminho de cor só com a causa demonstrada e aprovação.

## critérios de aceitação

- relatório de divergência entre desktop e web e entre desktop e iOS
  simulador para todas as páginas do showcase, com cada diferença
  classificada.
- toda divergência silenciosa encontrada tem causa escrita ou está
  registrada como aberta.
- ADR com a definição de idêntico.
- `./tool/tasks/fullcheck.sh` verde.

## testes esperados

o comparador (classificação com imagens sintéticas), e a comparação real
como job.

## riscos e perguntas abertas

- driver de GPU de CI (software) diferente do de desenvolvimento.
- automação de navegador com WebGPU em CI pode não estar disponível;
  nesse caso a captura web fica manual e registrada.

## commits esperados

1. `task-77: comparador com classificação`
2. `task-77: captura por alvo (desktop, web, iOS, Android)`
3. `task-77: relatório e tabela de diferenças pedidas`
4. `task-77: <causa>` por divergência resolvida
5. `task-77: ADR do que idêntico quer dizer`

## pare e reporte se

- a correção de uma divergência exigir mudar o contrato de cor;
- a causa estiver no driver e não houver contorno no plev.

## fase 1 (task-77a): a sonda

feita antes do código, no worktree da branch `task/77a-determinism`, base
`5207f9b8`. máquina: apple M4 (metal 4), macOS 26.6.2, dois displays (o
built-in liquid retina XDR, backing scale 2, e um DELL AW2518HF, backing
scale 1). toda afirmação sobre biblioteca vem da fonte da versão do
`Cargo.lock` (wgpu 28.0.0, wgpu-core 28.0.1, wgpu-hal 28.0.1, wgpu-types
28.0.0, winit 0.30.13), com arquivo e linha.

### (a) ferramentas, alvo por alvo

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

### (b) o desktop consigo mesmo

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

### (c) a janela contra o headless

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
  (`docs/ref/validate-visuals-by-pixel.md`, "color profiles in
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

### (d) os shaders

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

## handoff da fase 1b (task-77b): o blur segue a escala

branch `task/77b-blur-scale`, base `3a30113e` (o merge da fase 1). mesma
máquina da fase 1 (apple M4, metal). o wgpu, o naga e o wgpu-hal citados
são os do `Cargo.lock`: wgpu 28.0.0, naga 28.0.0, wgpu-hal 28.0.1.

### a sonda

feita antes da correção, sobre a base. os números saem de
`rust/engine/tests/blur_scale.rs`, dois probes ignorados que imprimem
tabelas (`cargo test -p engine --test blur_scale -- --ignored --nocapture
--test-threads=1`). as três variantes medidas foram protótipos
descartáveis no mesmo binário, escolhidos por variável de ambiente, fora
de qualquer commit; a escala 1 passa pelo mesmo caminho nas três.

#### (a) os caminhos de blur e de sombra

| caminho | de onde vem o sigma | onde vira kernel | unidade antes |
|---|---|---|---|
| backdrop blur (vidro) | `SceneNode::BackdropBlur.sigma` (`compositor/scene.rs:77`), copiado para `DrawCommand::BackdropBlur` (`compositor/layer/geometry.rs:410-451`, `compositor/sequence.rs:44`) | `window/render_passes.rs:382-410` suspende a pass da layer; `resolve_blurred_backdrop` (171-263) compõe tudo abaixo numa textura do tamanho físico e chama `apply_blur` (228-239) | texel físico |
| efeito de layer `Blur` | `LayerEffect::Blur { sigma }` (`compositor/layer/mod.rs:24-27`) | `WindowHost::apply_layer_effects` (`render_passes.rs:439-537`, o `Blur` em 482-493) chama `apply_blur` | texel físico |
| efeito de layer `Shadow` | `LayerEffect::Shadow { sigma, color }` | `render_passes.rs:494-506` chama `apply_shadow` (`effects/apply.rs:154-227`): a silhueta pelo `shadow.wgsl`, que não tem sigma, e depois `apply_blur(sigma)` (207-226) | texel físico |
| sombra analítica | `SceneNode::Shadow.blur_radius`, sigma = raio / 2 (`compositor/vertex.rs:243-245`), no vértice com `half_size` e `local` em px lógicos (`geometry.rs:275-350`) | `shadow_analytic.wgsl:99-116`: `erf` e gaussiana sobre `in.local`, a posição lógica interpolada por fragmento | px lógico, já segue a escala |

`apply_blur` antes (`effects/apply.rs:71-73`): `gaussian_weights(sigma)`
(`effects/types.rs:52-77`, centro e ±1 a ±6 normalizados nos 13) com
`texel_size = 1 / tamanho físico`; o `blur.wgsl` (`fs_main`, 47-63) soma as
13 amostras em passos de um texel físico. nenhum shader de sombra tem o
defeito: o `shadow.wgsl` não desfoca, e o `shadow_analytic.wgsl` já está
em px lógicos.

dois achados de caminho:

- os efeitos de layer só rodam no frame do `App` do engine
  (`window/render.rs:13-19` e 165). o `draw_frame`, que o headless e todo
  app de `present_frame` usam, compõe sem efeitos (`window/frame.rs:209-217`
  passa `&[]`): o `LayerEffect::Blur` da cena do `headless_determinism`
  nunca é desenhado, e fora dos testes ninguém chama `set_layer_effects`.
  o gate e a captura só exercitam o blur pelo vidro; os efeitos de layer
  passam pelo mesmo `apply_blur` e ganham teste no nível do
  `EffectProcessor` (`effects/tests.rs`).
- o `effects::LayerEffect` de `effects/types.rs` (`Blur`, `DropShadow`,
  `Opacity`) não tem uso; os caminhos usam o `compositor::LayerEffect`.
  fica anotado, fora do escopo.

#### (b) as opções, com números

- opção 1, as amostras crescem com a escala: o kernel de 1x espalhado
  pelos texels físicos por área. a amostra k de 1x vale pelo px lógico
  [k - 0,5, k + 0,5]; a amostra física j pega de cada uma a parte que cai
  em [(j - 0,5) / s, (j + 0,5) / s]. mesmos pesos, mesmo corte (6 px
  lógicos e meio de cada lado), soma 1, contínuo em s. amostras por
  passada: 2n + 1 com n = ceil(6,5 s + 0,5) - 1, então 17 a 1.25, 21 a 1.5,
  27 a 2 e 39 a 3. o uniform de hoje tem 16 floats (13 pesos): a 2x
  caberiam os 14 pesos, a 3x já são 20; o caminho novo tem um uniform seu,
  de 64 (até uma escala de 9,7), e o de hoje não muda.
- opção 1b, variante da 1: os mesmos pesos, dois texels por amostra
  bilinear (15 amostras a 2x, 21 a 3x).
- opção 2, alvo na resolução lógica: reduz a W / s por H / s com uma
  amostra bilinear por pixel, desfoca com os 13 taps de hoje, reamostra ao
  tamanho físico (duas passadas a mais, com o pipeline de composite que o
  `EffectProcessor` já tem).

diferença contra 1x, borda branca e preta em x = 48 lógico (cai na borda
de um pixel físico em toda escala, então nada fica meio coberto e o que
difere de 1x é o blur). método: cada pixel físico da linha do meio lê a
linha de 1x no seu centro em px lógicos, por interpolação linear em luz
linear entre os dois centros de pixel lógico em volta; o delta é a maior
diferença, 0 a 255. a rampa é a de 10% a 90% em px lógicos, a métrica do
teste ignorado (a 1x ela conta pixels inteiros: 9, 9 e 10 para sigma 4, 8
e 50).

| sigma | escala | rampa: antes / 1 / 2 | delta máximo: antes / 1 / 2 |
|---|---|---|---|
| 4 | 1.25 | 7,2 / 9,6 / 9,6 | 63,7 / 0,8 / 0,4 |
| 4 | 1.5 | 6,0 / 8,7 / 8,7 | 88,2 / 0,5 / 0,5 |
| 4 | 2 | 4,5 / 9,0 / 9,0 | 116,6 / 0,6 / 0,6 |
| 8 | 1.25 | 7,2 / 9,6 / 9,6 | 82,8 / 0,6 / 0,5 |
| 8 | 1.5 | 6,0 / 10,0 / 10,0 | 107,3 / 0,6 / 0,5 |
| 8 | 2 | 4,5 / 10,0 / 10,0 | 131,7 / 0,9 / 0,5 |
| 50 | 1.25 | 8,0 / 10,4 / 10,4 | 88,7 / 0,6 / 0,5 |
| 50 | 1.5 | 6,7 / 10,7 / 10,7 | 113,2 / 0,6 / 0,5 |
| 50 | 2 | 5,0 / 10,0 / 10,0 | 136,8 / 0,6 / 0,5 |

a 1b dá os números da 1 nesta tabela. com a borda em x = 50 (62,5 px
físicos a 1.25, um pixel meio coberto) a opção 1 dá delta máximo de 12 a
17 num único pixel a 1.25, na cauda onde o corte de 1x cai (a linha de 1x
vai de 53 a 0 num pixel); a média fica em 0,15.

uma linha de um px físico de largura, em cada fase da grade dentro de um
px lógico, sob sigma 8: a luz dela depois do blur, somada em linear
(1 = o blur guarda a luz da linha), e o pico da linha, por fase. o pico
esperado é o de 1x (83) com a luz dividida pela escala: 74, 67, 58, 47.

| escala | antes: luz, pico | opção 1 e 1b: luz, pico | opção 2: luz, pico |
|---|---|---|---|
| 1.25 | 1,00 / 1,00; 83 / 83 | 1,00 / 1,00; 74 / 74 | 1,08 / 0,94; 77 / 72 |
| 1.5 | 1,00 / 1,00; 83 / 83 | 1,00 / 1,00; 67 / 67 | 1,12 / 0,75; 72 / 58 |
| 2 | 1,00 / 1,00; 83 / 83 | 1,00 / 1,00; 58 / 58 | 1,00 / 1,00; 58 / 58 |
| 3 | 1,00 / 1,00 / 1,00; 83 | 0,99 nas três; 47 nas três | 0,00 / 3,01 / 0,00; 0 / 83 / 0 |

a amostra bilinear única da opção 2 é uma média de 2x2 exata só a 2x; a
1.25 e a 1.5 ela pesa os texels de forma desigual, e a 3x cai no centro
de um texel e ignora os outros oito. o que passa por baixo do vidro
pisca quando rola um px físico.

detalhe sob o vidro (uma borda em x fracionário, listras de 1 px lógico,
imagem, círculo, texto), sigma 8: o frame da escala levado à grade de 1x
por área (cada px lógico é a média, em luz linear, dos pixels físicos que
ele cobre, pesada pela área) contra o frame de 1x, dentro do vidro. média e
máximo da diferença de canal, 0 a 255. o piso é a mesma cena sem vidro:
o raster do conteúdo já difere entre escalas.

| escala | sem vidro | antes | opção 1 | opção 1b | opção 2 |
|---|---|---|---|---|---|
| 1.25 | 3,23 (127) | 5,37 (44) | 1,04 (13) | 1,03 (13) | 1,17 (12) |
| 1.5 | 2,32 (167) | 8,75 (86) | 0,87 (13) | 0,87 (13) | 1,09 (13) |
| 2 | 3,63 (190) | 14,00 (110) | 1,49 (19) | 1,49 (19) | 1,52 (18) |

o custo: uma página de 1280x800 lógico (24 cards com sombra analítica,
borda e texto, uma imagem) e um vidro do tamanho de um diálogo (480x300,
sigma 8, numa layer acima), toda layer redesenhada a cada frame. o span de
GPU pelos timestamps (do início da primeira pass ao fim do composite),
p50 de 240 frames depois de 30 de aquecimento, mediana de 5 execuções que
alternam página sem vidro e com vidro; três rodadas, as quatro variantes
intercaladas em cada rodada. sem a espera da GPU entre frames: com ela a
GPU ociosa baixa o clock e o p50 do mesmo caso variava 400 us entre
execuções. o backdrop compõe e desfoca a superfície inteira, então o
custo não depende do tamanho do vidro. em us, rodada 1 / 2 / 3:

| escala | sem vidro | antes | opção 1 | opção 1b | opção 2 |
|---|---|---|---|---|---|
| 1 | 402 / 402 / 430 | 844 / 901 / 901 | 843 / 902 / 901 | 843 / 902 / 902 | 846 / 903 / 900 |
| 1.25 | 623 / 653 / 666 | 1314 / 1402 / 1404 | 1546 / 1652 / 1650 | 1311 / 1401 / 1400 | 1210 / 1275 / 1268 |
| 1.5 | 893 / 953 / 955 | 1890 / 2011 / 2021 | 2489 / 2651 / 2650 | 2065 / 2203 / 2189 | 1601 / 1655 / 1653 |
| 2 | 1593 / 1635 / 1687 | 3559 / 3546 / 3711 | 5044 / 5292 / 5375 | 4948 / 4544 / 4269 | 2590 / 2606 / 2608 |

contra o antes da mesma rodada: a opção 1 custa 1,18x a 1.25, 1,31 a
1,32x a 1.5 e 1,42 a 1,49x a 2 (o limite de parada era 2x a 2x); a 1b, 1,00x,
1,08 a 1,10x e 1,15 a 1,39x; a opção 2, 0,90 a 0,92x, 0,82 a 0,85x e 0,70 a
0,74x. a 1x as quatro são o mesmo caminho e o mesmo custo.

o que muda no código:

- opção 1: `effects/types.rs` (os pesos por área, um uniform de 64 floats),
  `effects/apply.rs` (a escala no `EffectContext`; o pipeline e o uniform
  de cada passada), `effects/pipelines.rs` e `effects/processor.rs` (um
  pipeline a mais, `fs_scaled`), `blur.wgsl` (a entrada `fs_scaled` abaixo
  do `fs_main`, que não muda), `window/render_passes.rs` (a escala nos três
  `EffectContext`).
- opção 1b: o mesmo, com um segundo array de deslocamentos no uniform; a
  fração bilinear entra no blur.
- opção 2: `effects/apply.rs` (duas passadas e duas texturas do pool por
  blur, com o composite do processor) e `window/render_passes.rs`; o
  `blur.wgsl` não muda.

a escala 1 não muda em nenhuma: o `EffectContext` leva
`GpuContext::clip_scale()` por eixo, e um eixo de escala exatamente 1 (a
janela e o headless a 1, ou um app sem projeção lógica, cuja cena já está
em px físicos) usa o pipeline de hoje com o uniform de hoje, byte a byte.
o `fs_main` também não muda por o módulo ganhar outra entrada: o wgpu-hal
passa cada estágio por `process_overrides`
(`wgpu-hal-28.0.1/src/metal/device.rs:132-137`), que, num módulo de mais de
uma entrada, guarda só a entrada pedida e compacta o resto
(`naga-28.0.0/src/back/pipeline_constants.rs:64-87`); o MSL que o Metal
recebe para o `fs_main` do blur, lido do log de debug do wgpu-hal
(`device.rs:199-204`) com o `monster_player`, tem os mesmos 3339 bytes
antes e depois, e as opções de compilação não dependem do módulo
(`device.rs:206-216`: só a versão da linguagem e `preserve_invariance`). um frame de vidro e sombras a 1x gravado na base
(`rust/engine/tests/fixtures/blur_at_scale_1.png`, 160x96, 10 KB) é o
teste de byte a byte.

recomendação: a opção 1, com uma amostra por texel.

- a precisão contra 1x é a mesma das outras: delta máximo abaixo de 1 na
  borda, média de 0,9 a 1,5 no detalhe sob vidro (antes: 5,4 a 14,0).
- é a única, com a 1b, que guarda a luz de um detalhe fino em toda fase
  da grade; a opção 2 pisca o que rola sob o vidro (0 / 3,01 / 0 a 3x,
  1,12 / 0,75 a 1.5x), e um sigma 0 vira um blur bilinear de ida e volta.
- o custo cabe: 1,42 a 1,49x a 2x na página com vidro, abaixo do limite de
  2x; nada a 1x.
- a 1b é mais barata (1,15 a 1,39x a 2x) com a mesma precisão aqui, mas
  põe a fração bilinear de subtexel dentro de todo blur fora da escala 1, e
  essa fração é uma das fontes de diferença entre GPUs que o ADR lista. a
  opção 1 continua amostrando no centro do texel, como o blur de hoje. a
  1b fica como o passo seguinte se o custo a 3x num telefone pedir.

#### (c) onde o sigma é declarado, e quem compensa a escala

- tokens: `EffectTokens.blur_sigma` (`theme/tokens.rs:327`): 8 no dark e
  no light (`theme/core.rs:111`; o light herda do dark em 145), 8 no core
  (`theme/hoff_core.rs:173`), 50 no hoff (`theme/hoff.rs:197`). o
  `shadow_sigma` (4 em `core.rs:109` e `hoff_core.rs:171`, 12 em
  `hoff.rs:194`) alimenta sombra analítica, que já está em px lógicos.
- quem usa o `blur_sigma`: o modal (`comps/src/feedback/modal.rs:245-252`),
  a página de efeitos (`showcase/src/view/foundations/effects.rs:263-270`),
  o dock (`showcase/src/view/apps/dock.rs:208-214`), o builder
  (`Element::backdrop_blur`, `builder/element/builders.rs:228`, emitido em
  `builder/emit.rs:143-153`) e o parser (o `backdrop_blur` do CSS vira
  `theme.effects.blur_sigma`, `parser/src/css_map.rs:51-54`).
- ninguém compensa: todos passam o token cru, em px lógicos, sem
  multiplicar nem dividir por escala. a correção fica inteira no engine, e
  nenhum tema ou widget muda.

### commits

- `7127f65a` task-77: the blur-scale probe, its measures and the frame of before
- `f6c688b5` task-77: the blur follows the scale
- `adbf948b` task-77: the tests of the blur at every scale
- (este) task-77: the table, the ADR line and the phase 1b handoff

```text
Goal: task-77 fase 1b, o vidro e os efeitos de layer Blur e Shadow com o
      mesmo alcance e o mesmo perfil em px lógicos em toda escala
Status: concluída; a escala 1 igual byte a byte, o teste ignorado verde,
        o custo a 2x em 1,55x o de antes numa página com vidro
```

### a opção escolhida

a opção 1, uma amostra por texel físico, pelos números da sonda: a mesma
precisão contra 1x das outras duas (delta máximo abaixo de 1 na borda,
média de 0,9 a 1,5 no detalhe sob vidro, contra 5,4 a 14,0 antes), a luz
de um detalhe fino guardada em toda fase da grade (a opção 2 perde e
triplica uma linha de um px físico a 3x), sigma 0 continua sem blur em
toda escala, e o custo fica abaixo do limite de parada (2x a 2x). a 1b
teria a mesma precisão por menos, mas põe a fração bilinear dentro do
blur; fica para quando o custo a 3x pedir.

### a correção, arquivo por arquivo

- `rust/engine/src/drawing/effects/parameters.rs`: `scaled_weights(sigma, scale)`, os
  pesos de `gaussian_weights(sigma)` espalhados por área sobre os texels
  físicos (mesmo corte, soma 1; sigma 0 ou menos é uma amostra), e o
  uniform do caminho novo, `ScaledBlurUniforms`, com 64 pesos
  (`SCALED_WEIGHTS`). o `BlurUniforms` de 16 floats não muda.
- `rust/engine/src/drawing/effects/apply.rs`: `EffectContext.scale`, px físicos
  por px lógico, por eixo. `blur_pass` escolhe por eixo: escala
  exatamente 1 usa o `blur_pipeline` com o `BlurUniforms` de antes (os
  mesmos bytes); outra usa o `scaled_blur_pipeline` com `scaled_weights`.
  `apply_shadow` repassa a escala ao blur da silhueta.
- `rust/engine/src/drawing/effects/pipelines.rs` e `processor.rs`: o pipeline do
  `fs_scaled` (o construtor de pipeline de tela cheia ganhou o nome da
  entrada de fragmento), e o hot reload do `blur.wgsl` troca os dois.
- `rust/engine/src/drawing/gpu/shaders/blur.wgsl`: a entrada `fs_scaled` e o seu
  uniform, abaixo do `fs_main`, que não mudou.
- `rust/engine/src/runtime/window/render_passes.rs`: `scale: gpu.clip_scale()`
  nos três `EffectContext` (o vidro em `resolve_blurred_backdrop`, o
  `Blur` e o `Shadow` em `apply_layer_effects`). são três linhas no
  compositor que chama o efeito; nenhum outro arquivo do engine mudou, e
  nenhum tema, widget, shader de sombra ou o contrato de cor.

### os testes

- `rust/engine/tests/headless_determinism.rs`:
  `a_backdrop_blur_reaches_as_far_in_logical_px_at_every_scale` sem
  `#[ignore]`, verde.
- `rust/engine/tests/blur_scale.rs`:
  - `the_glass_at_scale_1_is_the_glass_of_before_byte_for_byte`: um frame
    de 160x96 a 1x, vidro em sigma 3, 8 e 50 sobre uma borda fracionária,
    listras, imagem, círculo e texto, com sombra analítica de fora e de
    dentro, contra `rust/engine/tests/fixtures/blur_at_scale_1.png`. o
    png foi gravado na base, com `PLEV_RECORD_BLUR_FIXTURE=1`, antes de
    qualquer mudança no engine, e entrou no commit 1: o teste é verde
    antes e depois da correção. bytes dependem da GPU e do compilador, então
    ele compara no adapter que gravou (apple M4, metal) e em outro pula
    dizendo qual é.
  - `a_backdrop_blur_has_the_profile_of_1x_at_the_fractional_scales`: a
    1.25 e a 1.5, sigma 4, 8 e 50, a rampa a até 1 px lógico da de 1x e a
    borda a até 2 da linha de 1x reamostrada.
  - `a_line_thinner_than_a_logical_px_keeps_its_light_under_the_glass`: a
    1.25, 1.5 e 3, a luz de uma linha de um px físico em 1 ± 0,02 em cada
    fase, e o mesmo pico (± 1) em todas. o blur de antes também passa
    nesse (ele não depende da fase, só do tamanho físico); o teste segura
    a escolha contra um alvo reduzido.
  - os dois probes, ignorados.
- `rust/engine/src/drawing/effects/tests.rs`: `scaled_weights` a 1 dá os pesos
  de 1x bit a bit; a 0.5, 1.25, 1.5, 2 e 3 guarda o corte (3, 8, 10, 13 e 19
  amostras por lado) e soma 1; sigma 0 é uma amostra; o uniform limita a 63
  por lado. e `the_layer_blur_and_shadow_reach_as_far_in_logical_px_at_every_scale`:
  o `EffectProcessor` na GPU, `apply_blur` e `apply_shadow` sobre uma borda
  opaca, a 1.25, 1.5, 2 e 3 contra 1x. os efeitos de layer ficam testados
  nesse nível porque o frame do `App` (`WindowHost::render`) limpa a cena
  antes de desenhar e o `draw_frame` não aplica efeitos: as três linhas
  de `apply_layer_effects` que passam a escala não têm teste que as
  alcance.

com o blur de antes (a escala forçada a 1 num build de conferência,
descartado), falham: o teste de alcance ("1.5x: the blur spreads 6
logical px, 9 at 1x"), o de perfil fracionário ("sigma 4 at 1.25x spreads
7.2 logical px, 9 at 1x") e o dos efeitos de layer ("blur at 1.25x spreads
6.4 logical px, 8 at 1x").

### a escala 1, byte a byte

- o caminho: um eixo de escala exatamente 1 usa o mesmo pipeline, o mesmo
  uniform com os mesmos bytes e as mesmas passadas de antes. o MSL do
  `fs_main` do blur que o Metal recebe tem os mesmos 3339 bytes antes e
  depois (a sonda diz como foi lido).
- o frame de vidro e sombras: `the_glass_at_scale_1_is_the_glass_of_before_byte_for_byte`,
  verde.
- a galeria inteira: `regress capture --against` a captura feita na base
  antes da correção (`target/determinism/desktop-before`): os 328 frames
  a 1x (82 páginas nos 4 temas) iguais em bit, inclusive as 5 páginas com
  vidro. os `frame.png` dos 163 baselines são de escala 1, então nenhum
  fica velho.

### o custo antes e depois

o probe de custo com o binário de teste da base e o da correção, lado a
lado, em seis rodadas intercaladas (a página de 1280x800 com um vidro de
sigma 8, toda layer redesenhada, span de GPU p50 em us, mediana de 5
execuções por caso). a tabela dá a mediana das seis rodadas; a última
coluna, a razão rodada a rodada. esta GPU alterna entre dois estados de
clock entre processos (844 ou 900 us a 1x no mesmo binário), e a razão
dentro de um mesmo estado é a mais estável: 1,18 a 1.25, 1,32 a 1.5.

| escala | sem vidro | com vidro, antes | com vidro, depois | depois / antes |
|---|---|---|---|---|
| 1 | 416 | 901 | 893 | 0,99 (0,94 a 1,04) |
| 1.25 | 644 | 1403 | 1650 | 1,18 (1,10 a 1,25) |
| 1.5 | 896 | 2011 | 2645 | 1,32 (1,24 a 1,40) |
| 2 | 1605 | 3549 | 5487 | 1,55 (1,31 a 1,71) |

só a parte do vidro (com menos sem vidro): 737 para 985 us a 1.25, 1059
para 1717 a 1.5, 1911 para 3813 a 2. a 2x o blur em si dobra (27 amostras
por passada em vez de 13), e a página com vidro fica em 1,55x, abaixo do
limite de 2x. a 1x nada muda.

### a captura da fase 1 depois da correção

| execução | frames | contextos | processos | tempo |
|---|---|---|---|---|
| `capture` (escreve os pngs) | 1312 | 1312 iguais | não pedido | 64,4 s |
| `capture --out target/determinism/desktop-b --against target/determinism/desktop --reverse --no-png` | 1312 | 1312 iguais | 1312 iguais | 38,8 s |
| `capture --out target/determinism/desktop-vs-before --against target/determinism/desktop-before --no-png` | 1312 | 1312 iguais | 1252 iguais, 60 caladas | |

o desktop continua determinístico consigo mesmo: zero pedidas, zero
caladas, zero de outro tamanho entre contextos e entre processos. as 60
caladas da última linha são o que a correção mudou, contra os pngs da
base: 5 páginas, 4 temas, 3 escalas (1.25, 1.5, 2), e nenhuma a 1x. são as
páginas com vidro: Effects (o painel de backdrop blur; a 2x até 66208 px,
delta máximo de 76 a 157 por tema) e Bubble, Message, Message Scroller e
Dock (o vidro do dock de mensagens, sobre fundo quase liso: até 7371 px,
delta máximo de 1 a 5).

### gate e regress

- antes (a base, o log do gate da fase 1 sobre a mesma árvore): workspace
  2687 passaram, 18 ignorados; urnaui 85 passaram, 1 ignorado; total 2772.
- depois: workspace 2696 passaram, 19 ignorados; urnaui 85 e 1; total
  2781. os 9 a mais: o teste de alcance que deixou de ser ignorado, 3 em
  `blur_scale.rs` e 5 em `effects/tests.rs`; os ignorados são 18 menos o
  de alcance mais os 2 probes. `./tool/tasks/fullcheck.sh` verde com as pernas wasm e
  urnaui (log em `target/agents/gate-77b.txt`).
- `regress check`: 163 baselines, 163 inalterados, nenhum frame, nenhuma
  GPU; a saída é a da fase 1 fora os tempos.
- o `cargo check --target wasm32-unknown-unknown -p uigallery` do gate é
  verde. iOS e android não compilam aqui (os alvos não estão instalados,
  como na fase 1); a mudança não tem código por plataforma.

### decisões

- os pesos por área, e não a gaussiana amostrada na escala: reproduzem os
  pesos de 1x por px lógico, são contínuos na escala (a última amostra
  cresce de zero) e dão os pesos de 1x exatos a 1. o que a área soma de
  largura é uma variância de 1/16 de px lógico² a 2x (1/12 no limite),
  nada perto das 16 de sigma 4.
- a escala por eixo, de `GpuContext::clip_scale()`: sem projeção lógica ela
  é 1 e o sigma fica em px físicos, que ali são as unidades da cena.
- o frame de antes como fixture de 10 KB, gravado na base e comparado só
  no adapter que o gravou.
- a tabela ganhou a frase na razão da linha `size` do desktop, porque o
  blur não é uma área e a tabela não tinha entrada sobre ele para tirar.
  no ADR mudaram a linha "across scales" e a consequência que esperava a
  correção.

### o que fica aberto

- o custo a 3x num telefone: 39 amostras por passada e 9 vezes os pixels de
  1x. não medido aqui. se pesar, a 1b corta para 21 amostras com a mesma
  precisão neste M4, trazendo a fração bilinear para o blur.
- as três linhas de `apply_layer_effects` que passam a escala aos efeitos
  de layer não têm teste que as alcance; o frame do `App` do engine limpa
  a cena antes de desenhar. os efeitos em si estão testados no
  `EffectProcessor`.
- o `effects::LayerEffect` sem uso em `effects/types.rs`.
- o fixture de escala 1 só compara no apple M4 com metal; outra máquina
  precisa gravar o seu, ou fica no limite entre GPUs do ADR.
- iOS e android sem compilar nesta máquina.

## fechamento (orquestrador, 2026-09-25)

fechada pelo critério de aceite, item por item. as ferramentas da web e
do iOS foram instaladas com o ok do brenner (tasks 20 e 19).

- relatório de todas as páginas: `tool/tasks/screencap.sh web --all` e
  `tool/tasks/screencap.sh ios --all` desenham as 82 páginas do desktop
  (`regress capture`), abrem cada página no alvo (web: a query da URL,
  `?page=…&theme=…`, que o build web agora lê por
  `engine::window::launch_args`; iOS: os argumentos de lançamento) e
  classificam cada frame contra o do desktop, uma linha por página em
  `target/determinism/<run>/report.txt`. tema core.
  - web, 1280x800 a 1x, WebGPU sobre Metal: 79 iguais em todo byte, 3
    caladas.
  - iOS, iPhone 17 Pro, 402x874 a 3x: 78 com diferença só na barra de
    status (pedida), 4 caladas.
- cada calada com causa:
  - Surface e Silences (web e iOS): o seletor de quem propõe mostra
    "claude" no desktop, que tem o programa, e "rules" onde não há Claude.
    decisão do app, não do renderer.
  - Git (web e iOS): a página diz que o workspace git pede o CLI e uma
    thread, só no desktop. decisão do app.
  - Editor (iOS): 7x60 px físicos, o cursor: ele pisca e a captura cai em
    outra fase do piscar. tempo, não renderer.
  - Table e Data Table (web, na primeira rodada): defeito, corrigido. o id
    em hex da tabela era calculado em `usize`, que tem 32 bits no wasm32 e
    estourava; agora em u64 (`view/components/table.rs`), e as duas
    páginas ficaram iguais em todo byte.
- ADR: `what-identical-means`, com a adição da fase 2; a tabela
  `requested.json` confirma as linhas medidas (web `size`; iOS
  `status-bar`, `home-indicator`, `size`; android `navigation-bar`,
  `size`).
- o android (task-18) ficou a no máximo 2/255 do desktop, na GPU por
  software do emulador.

pendências:
- a janela contra o headless nas páginas e o `colorspace` num display P3
  (fase 2 original); o job de CI no mac quando o CI voltar.
- android `status-bar` segue `to_confirm`: a captura não mostrou
  diferença ali.
- as capturas foram em um tema e uma escala por alvo; os outros temas e
  escalas não foram varridos página a página.
