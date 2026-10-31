---
id: task-77/phase-1b-blur
title: "phase 1b handoff (task-77b): blur follows the scale"
project: plev
type: section
parent: task-77
status: reference
date: 2026-09-25
date_basis: header
recorded: 2026-09-23
topics: [determinism, rendering, effects]
relations: [adr-0064]
evidence: [7127f65a, f6c688b5, adbf948b]
source: docs/log/steps/task-77-visual-determinism.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase 1b handoff (task-77b): blur follows the scale

Records phase 1b of task-77: the fix for blur that did not follow the scale, so that glass and the Blur and Shadow layer effects reach the same distance in logical px at every scale. Three options were measured; the chosen one was one sample per physical texel, with area weights. Scale 1 stayed byte for byte identical to the baseline, and the page with glass cost 1.55x what it did before at 2x, below the stop threshold. The tests and the cost table are on this page.

branch `task/77b-blur-scale`, base `3a30113e` (o merge da fase 1). mesma
máquina da fase 1 (apple M4, metal). o wgpu, o naga e o wgpu-hal citados
são os do `Cargo.lock`: wgpu 28.0.0, naga 28.0.0, wgpu-hal 28.0.1.

## a sonda

feita antes da correção, sobre a base. os números saem de
`rust/engine/tests/blur_scale.rs`, dois probes ignorados que imprimem
tabelas (`cargo test -p engine --test blur_scale -- --ignored --nocapture
--test-threads=1`). as três variantes medidas foram protótipos
descartáveis no mesmo binário, escolhidos por variável de ambiente, fora
de qualquer commit; a escala 1 passa pelo mesmo caminho nas três.

### (a) os caminhos de blur e de sombra

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

### (b) as opções, com números

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

### (c) onde o sigma é declarado, e quem compensa a escala

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

## commits

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

## a opção escolhida

a opção 1, uma amostra por texel físico, pelos números da sonda: a mesma
precisão contra 1x das outras duas (delta máximo abaixo de 1 na borda,
média de 0,9 a 1,5 no detalhe sob vidro, contra 5,4 a 14,0 antes), a luz
de um detalhe fino guardada em toda fase da grade (a opção 2 perde e
triplica uma linha de um px físico a 3x), sigma 0 continua sem blur em
toda escala, e o custo fica abaixo do limite de parada (2x a 2x). a 1b
teria a mesma precisão por menos, mas põe a fração bilinear dentro do
blur; fica para quando o custo a 3x pedir.

## a correção, arquivo por arquivo

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

## os testes

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

## a escala 1, byte a byte

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

## o custo antes e depois

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

## a captura da fase 1 depois da correção

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

## gate e regress

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

## decisões

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

## o que fica aberto

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
