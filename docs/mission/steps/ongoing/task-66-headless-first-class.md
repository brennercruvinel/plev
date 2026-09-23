---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-23
domain: generative-runtime
task: task-66
wave: 3
depends: []
depends-optional: [task-57]
branch: task/66-headless
---

# task-66: headless como primeira classe

## objetivo

qualquer interface plev pode ser compreendida e desenhada sem janela: a
estrutura semântica sem GPU nenhuma, e os pixels por render offscreen de
qualquer página, tema, viewport e escala, e de qualquer frame de
`.monster`. é a base de CI, de og-image, de relatório, do inspector de
sessão, da regressão estrutural e do determinismo.

## ligação com a tese

seção 25 (headless é primeira classe; o agente trabalha com a estrutura
sem framebuffer; a imagem continua disponível quando importa), seção 2 (a
cena é renderizável sem janela).

## estado atual no código

- `crates/engine/src/text/probe.rs` e o exemplo
  `crates/engine/examples/text_probe`: wgpu offscreen para png, só texto,
  escala configurável. existe porque o sistema às vezes nega captura de
  tela.
- `crates/engine/tests/text_raster_pixels.rs` e
  `text_drawn_extent.rs` já medem pixel em teste.
- o showcase aceita `[page] [theme] [scroll]` na linha de comando
  (`cargo run -p showcase`), com janela.
- o frame inteiro passa por `engine::window::present_frame`, que escreve na
  surface da janela.

## diagnóstico do orquestrador (2026-09-23)

- `GpuContext::new_with_config(window, config)`
  (`crates/engine/src/gpu/context.rs:77`) exige uma janela: cria a surface
  a partir dela, tira largura e altura de `window.inner_size()`, escolhe o
  formato na capacidade da surface e cria todos os pipelines nesse
  `render_format` (a variante srgb). o campo `surface` já é
  `Option<wgpu::Surface>` (`context.rs:26`), então o resto do contexto não
  depende de ter uma surface.
- `present_frame` (`crates/engine/src/window/frame.rs:29`) mistura duas
  coisas: pegar a textura da surface (`get_current_texture`, tratamento de
  `Lost`/`Outdated`/`Timeout`, `surface_render_view`) e desenhar o frame
  (resolve do compositor, texto, imagens, passes de layer, composite na
  view, submit), e no fim `output.present()`.
- `engine::text::probe` (`crates/engine/src/text/probe.rs`) é um segundo
  caminho offscreen: cria device, textura, pipeline de texto próprios e
  desenha só texto. é o que esta task substitui.

o corte, portanto:

1. um construtor headless: `GpuContext::new_headless(width, height,
   config) -> Option<Self>` (async com o mesmo padrão do `new`; `None`
   quando não há adapter, para teste pular em vez de falhar), com
   `surface: None`, formato `Rgba8UnormSrgb`, `surface_config` com o
   tamanho pedido, e a mesma construção de pipelines (extrair o corpo
   comum de `new_with_config` em uma função que recebe device, queue,
   formato e tamanho; nenhuma cópia).
2. extrair de `present_frame` uma função que desenha o frame numa
   `wgpu::TextureView` qualquer; `present_frame` passa a pegar a textura
   da surface, chamar essa função e dar `present`. comportamento com
   janela idêntico (mesmo número de draws, mesma ordem).
3. o headless cria a textura alvo, chama a mesma função e faz readback
   com o alinhamento de linha de 256 bytes.

## escopo

1. `engine::headless` (nome aprovado no briefing): cria device e textura
   offscreen, roda o mesmo caminho de resolve e passes do `present_frame`
   contra a textura (sem segundo pipeline de desenho), lê de volta para
   RGBA e grava png. mesma regra srgb: render target pela view srgb, sem
   atalho.
2. CLI do showcase: `showcase --render <page> --theme <t> --width <w>
   --height <h> --scale <s> --out file.png` e `--semantics out.json`
   (projeção sem GPU, quando a página tiver `Semantics`).
3. `monster_player --frame <t> --out file.png`.
4. SVG: exportação vetorial dos nós que têm equivalente honesto (rect,
   rounded rect, path, gradiente linear); texto como glifos em path ou
   `<text>` com a fonte embutida (decisão no ADR); sombra, blur e backdrop
   aproximados e reportados, como o importador de svg já faz ao contrário.
   PDF fica registrado como próximo passo.
5. o `text_probe` passa a usar o módulo novo (sem duas implementações de
   offscreen).

## fora de escopo

harness de regressão (67), comparação entre alvos (77), inspector (64).

## dependências

nenhuma para o render. a saída semântica usa 57 se já integrada.

## superfície sob responsabilidade

`crates/engine/src/headless/**` (novo), `crates/engine/src/text/probe.rs`,
`crates/engine/examples/{text_probe,monster_player}`, o entry point do
showcase (`crates/showcase/src/main.rs` e o parse de argumentos em
`crates/showcase/src/app.rs`, só a parte de CLI), proposta de ADR
`docs/ADR/headless-render-shares-the-frame-path.md`.

proibido na onda 3: `crates/engine/src/compositor` (task-59),
`crates/comps`, `crates/showcase/src/view/apps/app*` (task-59).

## interfaces que consome

`present_frame` e os passes de `crates/engine/src/window`, `GpuContext`,
`TextSystem`, `Compositor`.

## interfaces que pode alterar

pode extrair do caminho de present a parte que escreve na surface para
aceitar outro alvo, sem mudar comportamento com janela. qualquer mudança
de assinatura pública em `window` precisa de aprovação.

## critérios de aceitação

- todas as páginas do showcase, nos dois temas principais, em phone e
  desktop, renderizam headless.
- duas execuções na mesma máquina dão png byte a byte iguais.
- amostras de pixel conhecidas batem (fundo do tema dark medido como no
  resto do repo).
- frame de monster renderizado headless igual ao frame do player com
  janela (hash).
- `--semantics` do todo (depois de 59) igual à projeção em processo.
- `./script/gate` verde; wasm não compila o módulo headless (cfg).

## testes esperados

teste de integração que renderiza uma cena pequena e confere pixels;
teste de determinismo (duas vezes, mesmo hash); teste do SVG com um nó de
cada tipo e o relatório de aproximação.

## riscos e perguntas abertas

- CI no ubuntu usa adapter de software (lavapipe ou llvmpipe): os pixels
  podem divergir do Metal. registrar e deixar para a task-77, sem mascarar.
- tamanho de textura máximo por adapter.

## commits esperados

1. `task-66: módulo headless sobre o mesmo caminho de present`
2. `task-66: text_probe usa o headless`
3. `task-66: showcase --render e --semantics`
4. `task-66: monster_player --frame`
5. `task-66: exportação SVG com relatório de aproximação`

## pare e reporte se

- o caminho de present não puder aceitar outro alvo sem mudar o
  comportamento com janela;
- o determinismo falhar na mesma máquina.
