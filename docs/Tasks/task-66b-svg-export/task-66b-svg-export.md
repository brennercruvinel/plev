---
id: task-66b
title: "task-66b: SVG export and `--semantics` in headless mode"
project: plev
type: task
status: done
date: 2026-09-23
date_basis: header
recorded: 2026-09-23
topics: [rendering, formats, text]
relations: [adr-0041, task-57b, task-59, task-62, task-63, task-66, task-67, task-82]
evidence: [efc87b6]
source: docs/log/steps/task-66b-svg-export.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-66b: SVG export and `--semantics` in headless mode

Closes what task-66 left out: exports a frame as SVG with reported equivalence and connects `--semantics` to `showcase --render`. The text comes out as outlines of the glyphs that the `TextSystem` shaped for the frame, along a single shaping path, and each approximation (shadow, gradient, backdrop blur, image, clip) enters the report per node. Merged in `efc87b6` with the green gate (1969 tests) and a side-by-side visual comparison of Todo and Effects; embedded image and backdrop blur were left out.

## objetivo

fechar o que a task-66 deixou de fora: exportar um frame como SVG com
equivalência honesta e reportada, e ligar `showcase --render --semantics`
agora que a projeção semântica existe (task-59 fase a).

## decisão já tomada (orquestrador, 2026-09-23)

texto vira contorno de glifo (opção A do handoff da task-66): o shaping
do `TextSystem` ganha um ponto `pub(crate)` que devolve, para um
`TextNodeKey`, os glifos posicionados, e o SVG pede o contorno de cada um
ao `SwashCache` (`get_outline_commands`). a string do run vai no
`aria-label` do grupo. nenhum segundo caminho de shaping.

## escopo

- `engine::headless::svg` (novo): rect, rounded rect (borda como stroke
  inset), gradiente linear (interpolação em linearRGB, reportada), path
  tessellado agrupado por cor, texto como contorno, sombra como
  `feGaussianBlur` aproximado, backdrop blur descartado; tudo que não é
  exato entra num relatório devolvido junto (como o importador de svg faz
  no sentido contrário).
- `showcase --render <page> --svg f.svg` e `--semantics f.json` (a
  projeção da página, quando ela implementa `Semantics`; hoje o todo).

## superfície

`rust/engine/src/drawing/headless/**`, `rust/engine/src/drawing/text/system.rs` (só o
ponto de shaping `pub(crate)`), `rust/engine/src/drawing/text/fonts.rs` (só se o
contorno precisar dos bytes), `rust/engine/src/drawing/gpu/image.rs` (só um
acessor de pixels para imagem embutida), o CLI do showcase.

## aceite

SVG de três páginas do showcase abre num navegador com o mesmo layout do
png; o relatório lista cada aproximação; o texto do SVG tem os mesmos
glifos do png (comparar a largura medida de três runs); `--semantics` do
todo igual à projeção em processo; gate verde.

## fechamento (2026-09-23)

integrada na `main` em `efc87b6` (sete commits, rebaseados sobre a
`main` com o laboratório da 62c sem conflito), validada pelo orquestrador:
diff inteiro lido (o ponto de shaping é uma extração pura do que
`resolve_for_layer` já fazia, mais um leitor; o export só lê a cena), gate
verde no diretório da task (1969 passed, 0 failed, 6 ignored; a `main`
tinha 1949), e a comparação visual lado a lado do Todo e da Effects (png
do engine e raster do svg) conferida pelo orquestrador. ADR:
[`0041-svg-export-reads-the-scene.md`](../../adr/Rendering/0041-svg-export-reads-the-scene/0041-svg-export-reads-the-scene.md).

aceitos: `export` com o quarto parâmetro `background` (sem ele a página
sai transparente); `stage_page` recebendo `Viewport` (a projeção não abre
GPU); `serde`, `serde_json` e `svg` (dev) no manifest do showcase; o
módulo `headless` do showcase movido para `app/headless.rs` por limite de
linhas; imagem não embutida até a retenção de pixels da task-63 fase d;
efeitos de layer reportados como deixados de fora, como o caminho de frame
compartilhado já faz. dívida anotada para a task-67: `Approximation.node`
tipado em vez de texto.

## revisão do orquestrador (2026-09-23, antes do briefing)

- estado: `showcase --render` existe (`apps/uigallery/src/main.rs`, módulo
  `headless` em `apps/uigallery/src/shell/mod.rs:498-600`, `render_page`);
  `--semantics` é recusado com "not available yet" (`main.rs:99-103`),
  embora a projeção do todo exista desde a task-59 e o `wire` já sirva a
  forma serde enxuta (task-57b). o `--semantics` sai como a mesma forma
  serde do `Snapshot` (JSON), sem `source` por padrão, com `--source` para
  incluí-lo, igual ao `plev observe`.
- ownership: a task-62 fase c (ide, wire) e a task-82 (comps form/editor,
  shell do showcase em `app.rs:380-420`) rodam em paralelo. esta task fica
  em `rust/engine/src/drawing/headless/**`, `rust/engine/src/drawing/text/system.rs` (o
  ponto de shaping), `apps/uigallery/src/main.rs` e o módulo `headless`
  de `apps/uigallery/src/shell/mod.rs`. não toca o loop de eventos do showcase,
  o ide, o comps nem o wire.

## estado (agente, 2026-09-23, branch `task/66b-svg-export`)

implementado, sete commits pequenos na branch, cada um compilando e
passando clippy e fmt:

1. `1efda0e` o `TextSystem` entrega os glifos que shapeou
   (`shaped_glyphs`, `pub(crate)`) e `engine::headless::svg` escreve rect,
   rounded rect (borda como stroke inset) e texto como contorno de glifo
   dentro de `<g aria-label>`; clip vira `clipPath` e entra no relatório.
2. `31708df` sombra (drop e inset) como `feGaussianBlur` da silhueta,
   gradiente como `<linearGradient>` entre os dois pontos que o shader
   projeta, path tessellado como o polígono que delimita os triângulos
   (fallback para triângulos, reportado), imagem e backdrop blur
   deixados de fora com comentário no lugar; tudo no relatório, um item
   por nó, em ordem de push.
3. `24dede0` o módulo `headless` do showcase vai para
   `apps/uigallery/src/shell/window/headless.rs`, sem mudança (o `app.rs` ia
   estourar o limite de 639 linhas).
4. `1603133` um `ScaleContext` do swash novo por contorno de glifo, como o
   raster já faz: com um só contexto compartilhado a sidebar da página
   Card saía com os glifos errados. kill test feito: o teste de regressão
   falha com o contexto compartilhado e passa com o fix.
5. `9a9b573` `showcase --render <page> [--out f.png] [--svg f.svg]
   [--svg-report f.txt] [--semantics f.json] [--source]`, um frame só
   para todas as saídas; `--semantics` sem GPU.
6. README (a linha do `--render`) e este arquivo.

### handoff

Goal: task-66b, exportação SVG e `--semantics` no headless.
Status: implementado; gate verde no worktree (ver validação).

Implementado:
- `engine::headless::svg::export(compositor, text, viewport, background)
  -> Result<SvgExport, HeadlessError>`, `SvgExport { svg, report }`,
  `Approximation { node, what }` com `Display` ("layer 0, node 12
  (shadow): ..."), `SvgExport::write` e `report_text`.
  `Renderer::svg(&mut self, compositor, background)` chama o export com o
  text system que desenhou o frame.
- `TextSystem::shaped_glyphs(&mut self, key) -> Vec<ShapedGlyph>`
  (`pub(crate)`): glifo (`CacheKey` no tamanho do glifo, sem bin de
  subpixel), origem na baseline em px lógicos relativa ao nó, advance,
  linha. shaping pelo mesmo `ensure_shaped` que `resolve_for_layer` usa;
  uma chave desenhada volta como desenhada, uma não desenhada shapeia
  como vai ser.
- `showcase::app::{stage_page, StagedPage, semantics_json}`;
  `render_page` continua como stage + render. `--semantics` escreve o
  `Snapshot` na forma serde enxuta, pretty, sem `source` por padrão,
  `--source` inclui (igual ao `plev observe`). página sem projeção dá
  `RenderError::NoSemantics` com o nome da página.
- CLI: `--out` deixa de ser obrigatório; pelo menos uma saída é.
  `--svg-report` exige `--svg`; `--source` exige `--semantics`. o
  relatório vai para stderr, uma linha por item.

Arquivos principais:
- `rust/engine/src/drawing/headless/svg/{mod,write,shape,text,tests}.rs` (novos)
- `rust/engine/src/drawing/headless/mod.rs` (`pub mod svg`, `Renderer::svg`)
- `rust/engine/src/drawing/text/system.rs` (`ensure_shaped`, `ShapedGlyph`,
  `shaped_glyphs`)
- `apps/uigallery/src/shell/window/headless.rs` (movido de `app.rs`, depois
  `stage_page`, `StagedPage`, `semantics_json`, testes)
- `apps/uigallery/src/shell/mod.rs` (só o `pub use` e o `mod headless;`)
- `apps/uigallery/src/main.rs` (o CLI)
- `apps/uigallery/Cargo.toml`, `Cargo.lock`, `README.md`

Interfaces adicionadas/alteradas:
- `export` ganhou um quarto parâmetro, `background: Option<[f32; 4]>` em
  sRGB (o `Color::to_array` do bg do tema), fora do contrato do briefing:
  o frame começa do clear e sem ele a página do SVG sai transparente e o
  layout não bate com o png. `None` deixa transparente.
- `stage_page` recebe `Viewport`, não `&Renderer`, para a projeção sair
  sem abrir GPU.
- `RenderError` ganhou `NoSemantics(String)` e `Json(serde_json::Error)`.
- `apps/uigallery/Cargo.toml`: `serde` e `serde_json` nas deps de
  desktop (o json do `--semantics`), `svg` em dev-dependencies (o
  importador lê os exports de volta nos testes). fora da lista de
  arquivos permitidos, necessário para o item 3 e o teste (a).

Invariantes introduzidas:
- um caminho de shaping: `resolve_for_layer` e `shaped_glyphs` passam
  por `ensure_shaped`; o svg nunca shapeia por conta própria.
- ordem do svg = ordem do composite (layers por z-order, sort estável) e
  ordem de push dentro da layer; cull igual ao do frame (nó fora do
  viewport ou sob clip vazio não sai).
- cor no svg é o sRGB da cena, sem linearizar.
- toda aproximação é um item do relatório; um nó exato não gera item.
- o svg é determinístico: dois exports da mesma cena são idênticos
  (os cantos dos polígonos são visitados em ordem de coordenada).

Testes (antes -> depois):
- `cargo test -p engine headless`: 13 -> 27 (14 novos em
  `headless::svg::tests`)
- `cargo test -p engine --lib`: 755 -> 769
- `cargo test -p uigallery`: 122 + 2 ignorados (lib) + 7 (bin) ->
  127 + 2 ignorados (lib) + 8 (bin); 6 novos no headless, 1 novo no cli
- do contrato: (a) Todo, Button e Card exportam SVG que o importador
  `rust/import/src/formats/svg` lê de volta, viewBox igual ao viewport, `<g aria-label>` e
  `<rect>` contados, `<g>` balanceado; (b) a soma dos advances de três
  runs (Inclusive Sans 16, Inter 600 14, tracking 1.0 700 20) é a largura
  do `TextMeasurer::measure_styled` (diferença < 0,01 px), e o pen termina
  nela; (c) `--source` é byte a byte
  `serde_json::to_string_pretty(&snapshot)`; sem `--source` nenhum nó tem
  `source`, o json lê de volta um `Snapshot` com a mesma revision, os
  mesmos ids na mesma ordem e o mesmo focus; (d) o relatório da Effects
  lista drop shadow, inset shadow, gradient e backdrop blur; uma cena só
  de rects e texto tem relatório vazio; (e) todo teste com GPU pula com
  "no GPU adapter; skipping".
- regressão: o mesmo run de texto (a sidebar) exporta o mesmo contorno em
  Card, Effects e Typography; falha com o `SwashCache` compartilhado.

Validação executada (com `CARGO_TARGET_DIR=target/agents/task-66b`):
- `cargo test -p engine headless && cargo test -p uigallery`
- `cargo run -p uigallery -- --render Todo --out target/agents/t66b-todo.png
  --svg target/agents/t66b-todo.svg --svg-report target/agents/t66b-todo-report.txt
  --semantics target/agents/t66b-todo.json`; o mesmo para Button (core),
  Card (hoff) e Effects (hoff) em `target/agents/t66b-{button,card,effects}.*`
- `./tool/tasks/fullcheck.sh`
- comparação manual: cada svg rasterizado com `rsvg-convert` e medido
  contra o png do engine (1200x800, `magick compare -fuzz 8%`): Todo
  0,13% dos pixels diferem, Button 0,2%, Effects 0,56%, Card 1,07% (as
  duas imagens descartadas). o png e o raster do svg lado a lado em
  `target/agents/t66b-{todo,effects,card}-side.png`.

Relatório de aproximações por página (a saída do `--svg-report`):
- Todo / core, 8 itens: 3 clip, 4 gradient rect, 1 path (fallback para
  triângulos, um ícone com stroke)
- Button / core, 23 itens: 2 clip, 5 gradient rect, 4 path, 12 shadow
- Card / hoff, 24 itens: 2 clip, 13 gradient rect, 2 image (88x88 e
  352x248, deixadas de fora), 7 shadow
- Effects / hoff, 27 itens: 1 backdrop blur, 4 clip, 16 gradient rect,
  6 shadow

Decisões:
- texto como contorno: `cosmic_text::Command` (reexport de
  `swash::zeno::Command`) evita dependência nova; o contorno sai no
  tamanho do glifo, y para cima, e é virado ao escrever. a origem é a
  que `atlas::emit_glyphs` deriva antes de arredondar para o pixel físico
  (o raster arredonda em y e agrupa x em bins de subpixel; o svg guarda a
  posição exata, o que não conta como aproximação).
- `shaped_glyphs` shapeia numa falta em vez de devolver vazio, pelo mesmo
  `ensure_shaped`: o resultado é o mesmo do frame e o svg de uma cena
  ainda não desenhada é o svg do frame que ela vai ser.
- path tessellado: o `TessellatedPath` só guarda triângulos e o hash, não
  os comandos; o polígono de borda (arestas que só um triângulo percorre,
  encadeadas) reconstrói o contorno sem costura para fills. malha que não
  é partição limpa (alguns strokes do lyon) cai para triângulos e diz.
- imagem: `gpu/image.rs` não tem acessor de pixel (a `ImageStore` drena
  os pixels no upload) e o arquivo era só leitura; a imagem fica de fora,
  reportada, com um comentário `<!-- image WxH at x y -->` no lugar.
- efeitos de layer (`LayerEffect::Blur`/`Shadow`) entram no relatório
  como deixados de fora, com a razão: o `draw_frame` compartilhado passa
  `&[]` de effect_results, só o `window::App` do engine aplica; nenhum
  app usa. registrado, não corrigido (fora do escopo).
- clip: reportado por `PushClip` (o scissor do frame é em pixel físico
  inteiro, o clipPath é lógico exato). gera 2 a 4 linhas por página; se o
  orquestrador preferir tratar clip como exato, é remover um `note`.
- `semantics_json`: `SnapshotWire` é privado no `sem` e o crate é
  proibido; o showcase espelha a forma com uma struct de dois campos e o
  teste pina que, com `--source`, o texto é o do próprio `Snapshot`.

Limitações e dívida:
- imagem não embutida (precisa de um acessor CPU em `gpu/image.rs` ou de
  ler o atlas de volta com `read_texture`, que exige expor a textura).
- strokes tessellados que não são partição limpa saem como triângulos;
  costura fina possível no viewer. guardar os comandos do path no
  `TessellatedPath` resolveria.
- `color-interpolation="linearRGB"` não é honrado pelo Chrome: gradiente
  com stop translúcido difere um pouco do frame.
- backdrop blur descartado: o que está sob o vidro fica nítido.
- `--semantics` só para a Todo (única projeção hoje).
- a comparação por raster (`rsvg-convert`) foi manual; não há
  rasterizador de svg nas deps, então o teste automático é o estrutural
  mais o round trip pelo importador.
- `Approximation.node` é texto formatado; um tipo `(LayerId, índice,
  kind)` serviria melhor ao harness da task-67.

Dependências pendentes: nenhuma.

Riscos de integração: baixo. `app.rs` do showcase mudou só o `pub use`
e o `mod headless;` (o loop de eventos é da task-82, intocado);
`text/system.rs` só ganhou métodos; `Cargo.lock` mudou pelas deps do
showcase.

Proposta de parágrafo para o ADR (o orquestrador escreve):

> a exportação vetorial lê a cena, não os pixels: `engine::headless::svg`
> percorre as layers na ordem do composite e os nós na ordem de push e
> escreve um elemento por nó, com o texto como contorno dos glifos que o
> `TextSystem` shapeou para o frame (um caminho de shaping, o mesmo do
> raster) dentro de um grupo com a string em `aria-label`. o que não tem
> equivalente exato (sombra analítica, gradiente em linear, backdrop
> blur, imagem, o scissor em pixel inteiro) é aproximado ou deixado de
> fora e entra num relatório por nó, como o importador de svg reporta no
> sentido contrário; um svg sem relatório é o frame. a projeção
> semântica de uma página sai pelo mesmo comando que a desenha, na forma
> serde enxuta que o agente recebe, sem `source` a menos que peçam.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0041: the svg export reads the scene, not the pixels](../../adr/Rendering/0041-svg-export-reads-the-scene/0041-svg-export-reads-the-scene.md)
- [ADR 0061: printing is the frame on paper](../../adr/Rendering/0061-printing-is-the-frame-on-paper/0061-printing-is-the-frame-on-paper.md)
- [task-57b: lean JSON wire form and a single redaction](../task-57b-lean-wire-form-and-one-redaction/task-57b-lean-wire-form-and-one-redaction.md)
- [task-59: semantic binding with the render and one operational path](../task-59-semantic-binding-and-one-path/task-59-semantic-binding-and-one-path.md)
- [task-62: proof in the ide: stale-state rejection and human-agent handoff](../task-62-ide-human-agent-proof/task-62-ide-human-agent-proof.md)
- [task-63: session flight recorder](../task-63-flight-recorder/task-63-flight-recorder.md)
- [task-66: headless as a first-class citizen](../task-66-headless-first-class/task-66-headless-first-class.md)
- [task-67: structural regression before pixels](../task-67-structural-regression/task-67-structural-regression.md)
- [task-82: the caret cannot request 60 frames per second](../task-82-caret-blink-render-on-demand/task-82-caret-blink-render-on-demand.md)
- [task-63 phase c: session file, persistence and excerpt](../task-63-flight-recorder/phase-c.md)
- [task-82 phase a: the caret blink becomes a deadline](../task-82-caret-blink-render-on-demand/phase-a.md)
- [program: from GPU UI engine to generative interaction runtime](../../Research/Synthesis/generative-runtime-program.md)
- [how to code against the plev engine](../../Research/Workflows/code-against-the-plev-engine.md)
<!-- /related -->
