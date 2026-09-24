---
project: plev
audience: [ai-agents, contributors]
status: pending
last-updated: 2026-09-23
domain: generative-runtime
task: task-72
wave: trilha
depends: []
waits-for: [task-63]
branch: task/72-asset-compiler
---

# task-72: UI asset compiler

## objetivo

levar a filosofia do monster (formato externo complexo, compilado uma vez
em build time para uma representação própria otimizada, runtime pequeno)
para os outros assets de UI: SVG, fontes, ícones, animações, imagens,
layouts estáticos, shaders e design assets. e gravar interação real do app
como asset `.monster`.

## ligação com a tese

seção 11 (a filosofia do monster), seção 24 (UI asset compiler; gravar
interação real como asset: o app gera o motion, não o designer). as
anotações: "o monster é pequeno hoje, mas precisa de mais atenção".

## estado atual no código

- `lot` (lottie para monster) e `svg` (svg estático para monster, via usvg
  e lyon) já seguem a filosofia; o player não carrega runtime estrangeiro.
- ícones lucide são tesselados em runtime e cacheados
  (`crates/comps/src/icons/mod.rs:127`, `tessellate_icon`).
- fontes: ttf inteiros embutidos (`crates/engine/src/text/fonts.rs:38-52`),
  raster de glifo em runtime no atlas, determinístico.
- shaders WGSL carregados e validados em runtime (hot reload lê do disco em
  debug).
- imagens decodificadas em runtime (`image` com png e jpeg).
- o container do monster é superfície da task-63 enquanto ela roda.

## escopo

1. inventário medido: o que é parseado, tesselado, decodificado ou validado
   em runtime que poderia ir para build time, com o custo atual de cada um
   (startup, memória, tamanho de binário). nada é movido sem esse número.
2. candidatos, em ordem de ganho medido:
   - ícones: tesselação em build time para os tamanhos usados pelos tokens
     (`theme.control` icon sizes), guardados como `asset_path` do monster;
   - shaders: validação naga em build time (erro de shader vira erro de
     build, não de runtime), sem mudar o hot reload de debug;
   - fontes: subset só para fontes de conteúdo fixo (codicons); fontes de
     texto do usuário continuam inteiras (cobertura de glifo, task-74);
   - imagens: pré-decodificação ou formato de textura quando o ganho
     compensar o tamanho;
   - layouts: só para telas estáticas (splash); o contrato de layout
     derivado do espaço disponível continua valendo para o resto;
   - design assets: tokens gerados a partir de fonte de design só se a
     fonte puder ser nomeada em arquivo versionado (o kit privado não é
     citado em arquivo rastreado, regra do AGENTS.md).
3. ferramenta: um passo de build (build.rs por crate ou um CLI de assets,
   decisão no ADR) com saída determinística e cache por hash.
4. gravar interação real: com a trilha visual da task-63, gravar uma
   interação (spring, fade, gesto), passar pelo discover e pelo optimize do
   monster e exportar como asset tocável pelo player.

## fora de escopo

mudar o formato do monster antes da task-63 fechar o ADR de sessão.

## dependências

nenhuma para o inventário e para ícones e shaders. o item 4 e qualquer
mudança de container esperam a task-63.

## superfície sob responsabilidade

`crates/lot`, `crates/svg`, o passo de build novo, `crates/comps/src/icons`
(só a origem dos dados tesselados), proposta de ADR
`docs/ADR/ui-asset-compiler.md`. `crates/monster/src/container.rs` só
depois da 63.

## interfaces que consome

`monster` (codec, discover, optimize, `asset_path`), a trilha visual (63).

## interfaces que pode alterar

a forma como ícones e shaders chegam ao runtime, sem mudar a API pública de
comps.

## critérios de aceitação

- inventário com números antes e depois de cada item movido.
- ícones: pixels iguais antes e depois (headless, 66) em todos os tamanhos
  de token.
- shaders: um erro de sintaxe em WGSL quebra o build com mensagem clara.
- `./script/gate` verde; build release e wasm continuam.

## testes esperados

determinismo do passo de build (duas execuções, mesmo byte); igualdade de
pixel dos ícones; teste do erro de shader.

## riscos e perguntas abertas

- build time maior; medir e registrar.
- wasm: o passo de build roda no host, a saída entra no binário wasm.

## commits esperados

1. `task-72: inventário medido de trabalho em runtime`
2. `task-72: ícones tesselados em build time`
3. `task-72: validação de WGSL em build time`
4. `task-72: <próximo item pelo ganho medido>`
5. `task-72: interação gravada exportada como asset` (depois da 63)

## pare e reporte se

- o ganho medido de um item não justificar a complexidade;
- mover um asset para build time exigir mudar contrato de layout ou de
  texto.

## fase 1: a sonda e o inventário medido

medido em 2026-09-24 nesta máquina (apple silicon, metal), no perfil
debug: o workspace em opt-level 0 e as dependências em 2
(`[profile.dev.package."*"]` no Cargo.toml da raiz). nenhum
`cargo build --release` nesta task; o número de release fica para o
brenner. os comandos que reproduzem cada número:

```text
cargo test -p engine --test asset_cost -- --ignored --nocapture
    # fases do renderer headless, shaders, fontes, imagens
cargo test -p comps --lib icons::tests::the_tessellation_cost -- --ignored --nocapture
    # ícones nos tamanhos de token
showcase --render Theme --out f.png
    # o binário de debug; primeiro frame headless, 11 execuções
```

quais ícones o showcase pede (nome, tamanho, acerto ou falta de cache,
tempo) veio de um `eprintln!` temporário em `icon_path`, ligado por
variável de ambiente, rodado em `showcase --render Theme` e em
`regress check` (os 163 baselines) e desfeito antes do commit.

### o primeiro frame headless do showcase

`showcase --render Theme --out f.png` (a página de abertura,
`PAGES[0]`, tema core, 1200x800, escala 1), do início ao fim do
processo: mediana de 79,1 ms em 11 execuções (mínimo 77,8 ms). pico de
memória residente 58 MB (`/usr/bin/time -l`). o binário de debug tem
55.575.032 bytes.

| item | o que roda até o primeiro frame | custo (debug) | parte dos 79 ms |
| --- | --- | ---: | ---: |
| fontes | dois font systems: o do `TextMeasurer` (thread-local, na montagem da página) e o do `TextSystem` (no `Renderer::new`). cada um varre as 805 faces do sistema (`FontSystem::new` chama `load_system_fonts`, cosmic-text-0.18.2/src/font/system.rs:428) e registra as 9 embutidas | 18 ms o primeiro, 5 ms o segundo; o registro das 9 embutidas, 28 us | ~29% |
| device | o primeiro device metal do processo | 26 ms | ~33% |
| shaders | 7 pipelines do `GpuContext` e 4 do `EffectProcessor`: parse e validação naga, MSL e compilação metal | 2,7 ms, dos quais o naga (parse e validação dos 9) 0,7 ms | ~3% (naga ~1%) |
| ícones | 30 pedidos, 9 tesselações (8 das 9 num tamanho de token) | 0,17 ms | 0,2% |
| imagens | nenhuma: o showcase não embute imagem | 0 | 0 |

o resto (montagem e layout da página, shaping, raster de glifo,
desenho, readback, PNG, início do processo) não é asset e fica fora do
inventário.

onde o shader paga: o `wgpu-core` faz o parse e a validação naga no
`create_shader_module` (wgpu-core-28.0.1/src/device/resource.rs:2205 e
:2275). no metal, o `create_shader_module` do hal só guarda o módulo
(wgpu-hal-28.0.1/src/metal/device.rs:1039); o MSL é escrito e compilado
na criação do pipeline (`load_shader`, :121, :191, :216).

### memória e binário

| item | no binário (debug) | no heap |
| --- | --- | --- |
| fontes | 1.744.532 bytes, as 9 faces (3,1% do binário) | uma cópia por font system (`.to_vec()` em crates/engine/src/text/fonts.rs:38-52; o fontdb guarda o `Vec` em `Source::Binary`, fontdb-0.23.0/src/lib.rs:195-196): 2 x 1,66 MiB |
| shaders | 21.107 bytes, os 9 que o engine carrega. o `3Dscene.wgsl` de src/gpu/shaders (1.594 bytes) nenhum código carrega; o exemplo scene3d tem a própria cópia | o módulo naga é transitório |
| ícones | ~3,5 KB de texto de path e de pontos, 42 ícones (crates/comps/src/icons/data.rs) | o cache de geometria: 17,6 KB na página de abertura (505 vértices x 24 B, 1.383 índices x 4 B); 188 KB depois da galeria inteira (5.347 vértices, 14.973 índices) |
| imagens | 0 no showcase. o engine tem um PNG (`assets/logo-plev.png`, 17.921 bytes) que só o exemplo `visual` usa | decodificado, 4.665.600 bytes RGBA em 0,87 ms |

### ícones

- os tamanhos dos tokens: `theme.control.icons` = [16, 20, 24]
  (`ControlTokens::hoff`, crates/engine/src/theme/scales.rs:186), o
  mesmo em todos os temas (core.rs:76, hoff_core.rs:126, hoff.rs:225);
  os botões pedem 16 e 20 (button.rs:331-385). a tabela tem 42 ícones.
- o que o showcase pede de fato: na página de abertura, 9 pares (nome,
  tamanho). na galeria inteira (os 163 baselines do regress), 83 pares
  em 7 tamanhos: 12 px (1), 14 (1), 16 (13), 18 (1), 20 (23), 22 (42),
  24 (2). 38 dos 83 estão num tamanho de token; os 45 de fora (12, 14,
  18, 22) continuariam tesselados em runtime de qualquer jeito.
- o que o `tessellate_icon` (crates/comps/src/icons/mod.rs:127) usa: o
  parser de `d` do próprio comps (icons/svg.rs, o arco pelo apêndice
  B.2.4 da SVG 1.1), o `PathBuilder` do engine
  (crates/engine/src/path/builder.rs) e o `stroke_round` (builder.rs:190,
  tessellation.rs:94), que é o `StrokeTessellator` do lyon_tessellation
  1.0.20 (tessellation.rs:3-7 e :118; o pacote `lyon_tessellation` no
  Cargo.lock) com cap e join redondos e a tolerância
  `default_tolerance()` (crates/engine/src/path/mod.rs:14-26: 0,1,
  trocada por `RenderConfig::path_tolerance` em
  crates/engine/src/gpu/context.rs:256). o ícone não passa pelo `svg`
  (usvg) nem pelo `lot`.
- o custo: mediana de 7 a 16 us por tesselação, conforme a carga da
  máquina (duas execuções da sonda); os 42 ícones nos 3 tamanhos somam
  de 0,9 a 1,7 ms. um acerto de cache (a cópia com deslocamento e cor,
  que todo pedido paga, pré-tesselado ou não) custa de 1,7 a 2,5 us para
  o `settings` a 20 px (248 vértices). a tabela pré-tesselada dos 42 x 3
  pesaria 107.544 bytes no binário (posição em dois `f32`, índice
  `u16`), contra os ~3,5 KB de fonte de hoje.
- as duas formas de levar os ícones para antes do runtime:
  - passo de CLI, que cabe sem mudar o lock: um gerador dentro do comps
    chama o mesmo `tessellate_icon` e grava um arquivo versionado
    (binário com `include_bytes!`, o hash da fonte, da tolerância e dos
    tamanhos no cabeçalho); um teste do gate gera de novo em memória e
    compara os bytes. o comps já depende do engine, que tem o lyon.
  - `build.rs` no comps, que não cabe: um build script só enxerga
    `[build-dependencies]`, e o comps não tem nenhuma (o pacote `comps`
    no Cargo.lock: engine, env_logger, pollster, rope, rustc-hash, sem,
    unicode-segmentation, web-time, wgpu, winit). tesselar ali pede o
    lyon como dependência de build (aresta nova) e uma segunda cópia do
    parser e do tesselador, que moram no próprio comps.
- o limite para mover um item: o ganho medido tem que ser de pelo menos
  5% do tempo até o primeiro frame headless do showcase (4 ms dos 79)
  ou de um frame a 60 Hz (16,7 ms) somado na sessão inteira, e o
  binário não pode crescer mais do que o runtime economiza em memória.
  os ícones dão 0,17 ms na abertura (0,2%) e 0,97 ms na galeria
  inteira; a tabela cresceria o binário em ~105 KB para tirar no máximo
  1,7 ms, e o runtime continuaria com o tesselador para os tamanhos
  fora do token e para outra tolerância. não justifica: os ícones ficam
  em runtime nesta fase.

### shaders

- 10 arquivos `.wgsl` em crates/engine/src/gpu/shaders (22.701 bytes) e
  1 no exemplo scene3d. o naga no CPU, melhor de 20: o parse de 31 a
  148 us por shader, 707 us os 10; a validação de 2 a 16 us, 66 us os
  10.
- o `wgpu::naga` já está no engine: o wgpu 28 re-exporta o naga do
  wgpu-core sob `cfg(wgpu_core)` (wgpu-28.0.0/src/lib.rs:166-171), que o
  build.rs do wgpu liga em todo alvo nativo (wgpu-28.0.0/build.rs:3 e
  :34). no wasm32 o re-export só existe com as features `naga-ir`,
  `spirv` ou `glsl` (lib.rs:172-177, build.rs:46), então a checagem roda
  no host. o parse de WGSL vem da feature `wgsl`, default do wgpu
  (`wgsl = ["wgpu-core?/wgsl"]`, wgpu-28.0.0/Cargo.toml:124, e
  `wgsl = ["naga/wgsl-in"]`, wgpu-core-28.0.1/Cargo.toml:102).
  crates/engine/src/gpu/tests.rs já usa `wgpu::naga`.
- um teste do gate que valida todo `.wgsl`: possível sem aresta nova (o
  engine tem o wgpu como dependência normal, e o teste de integração a
  enxerga).
- um `build.rs` que falha o build: não sem aresta nova. o build.rs do
  engine não tem `[build-dependencies]`; usar o naga ali pede `naga` ou
  `wgpu` como dependência de build. o `naga` não está na lista do
  `engine` no Cargo.lock (chega pelo `wgpu-core`), então é aresta nova;
  o `wgpu` está na lista, mas o briefing proíbe `[build-dependencies]`
  de crate que já existe no lock. fica o teste.
- o que o teste não muda: o custo em runtime. o `wgpu-core` faz o parse
  e a validação de novo a cada `create_shader_module`; tirar isso do
  runtime pediria entregar o módulo já traduzido (IR do naga, ou MSL e
  SPIR-V em passthrough), e 0,7 ms (~1% do primeiro frame) não
  justifica. o ganho da checagem é o erro, que sai do primeiro run e vai
  para o gate com arquivo, linha e coluna.

### fontes e imagens (só o número; nada muda nesta fase)

- fontes: o custo é a varredura das faces do sistema, duas vezes por
  processo (18 ms e 5 ms, ~29% do primeiro frame), não as faces
  embutidas (28 us de registro). recomendação para a task-74, que é a
  decisão do brenner: um só banco de faces para o `TextSystem` e o
  `TextMeasurer` (uma varredura em vez de duas), ou a varredura do
  sistema adiada até o primeiro fallback; e `Source::Binary` sobre
  `&'static [u8]` em vez do `.to_vec()` (o fontdb aceita qualquer
  `AsRef<[u8]>`, fontdb-0.23.0/src/lib.rs:866), que tira 1,66 MiB do
  heap por font system. o subset do codicons (123.192 bytes) tiraria no
  máximo isso do binário.
- imagens: o showcase não embute imagem; a decodificação em runtime é
  só de conteúdo do usuário (arquivo solto, miniatura do ffmpeg). a
  recomendação é não mover: pré-decodificar o único PNG do engine
  custaria 4,6 MB de binário para tirar 0,87 ms de um exemplo.
