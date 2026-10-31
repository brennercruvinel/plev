---
id: task-71/fase-2-html-e-figma
title: "task-71 fase 2: sonda e handoff das fontes HTML/CSS e Figma"
project: plev
type: secao
parent: task-71
status: referencia
date: 2026-09-25
date_basis: texto
topics: [importacao, formatos, ir]
relations: [adr-0069, task-68]
evidence: [e3201af7, 6e99cec8, 7ca7d41e, 0cb09a90, 4e16c349]
source: docs/log/steps/task-71-compiler-target.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# task-71 fase 2: sonda e handoff das fontes HTML/CSS e Figma

Registra a sonda e o handoff da fase 2 da task-71: as fontes novas HTML/CSS (subset) e Figma, cada uma com parse, resolve para o mesmo `Resolution` e os dois emissores já existentes. A fixture HTML é o note card escrito como página (32 supported, 22 transformed, 17 approximated, 13 impossible, 2 bindings); a fixture Figma é a tela settings, de 98 nós (251, 27, 119 e 196, sem nenhum binding). A fonte Figma entra por um export salvo, sem rede no teste, e o IR recebeu só mudanças aditivas (`gap`, `align_items`, `DropKind::Approximated` e `image`).

## fase 2: a sonda (2026-09-25, agente, branch task/71b-importers)

escopo desta rodada: item 5 do escopo, as duas fontes (HTML/CSS subset e
Figma), escolhidas pelo brenner ("html e figma"). o formato é o das fontes
que existem: parse, resolve para `ParserNode`, os dois emissores e o
relatório em quatro categorias.

### (a) onde cada fonte entra

- o caminho que existe: parse (`tsx.rs`, `sass.rs`, `gpui.rs`), resolve
  para um `Resolution` (`ir.rs:236`), e os dois emissores sobre ele,
  `emit::emit` (`emit.rs:9`) e `emit_ir::emit_ir` (`emit_ir/mod.rs:54`).
  cada fonte tem sua entrada em `lib.rs` (`resolve_react` em `lib.rs:58`,
  `resolve_gpui` em `lib.rs:72`, `import_*` em `lib.rs:101` e `:112`).
- HTML/CSS entra como um parse novo, `html.rs` (tree-sitter-html para os
  elementos, tree-sitter-css para o `<style>` e para cada `style=""`), e
  um resolve novo, `resolve_html.rs`, que entrega o mesmo `Resolution`.
  as declarações que o subset react já mapeia (tamanho, padding, cor,
  fundo, raio, fonte) passam pelo mesmo `css_map::map_decl`
  (`css_map.rs:19`), depois de normalizar o que o CSS escreve e o sass do
  corpus não (cor hex e por nome, peso `bold`). flex (`display`,
  `flex-direction`, `gap`, `align-items`, `justify-content`, `flex-grow`)
  fica no resolve do HTML e não no `map_decl`: um braço novo lá mudaria o
  que o react produz para a mesma declaração, que hoje cai na droplist
  como "not covered" (`css_map.rs:103`).
- por que não reusar o `resolve_react`: as reescritas dele são do corpus
  react (overlay `inset: 0`, camadas `:before`/`:after`, classe
  condicional, `resolve_react.rs:1-16`); a herança de tipografia dele vai
  só ao texto filho direto (`resolve_react.rs:301`), a do CSS vai a todo
  descendente; e o conflito é "o primeiro vence"
  (`resolve_react.rs:404-410`), no CSS vence o mais específico e depois o
  último. o resolve do HTML compartilha `css_map` e os tipos de `ir.rs`.
- Figma entra como `figma.rs`, que lê o JSON do export com `serde_json`
  (já dependência do parser) e acha a linha de cada nó e de cada chave no
  texto (o `serde_json` não guarda posição; o script escreve uma chave por
  linha e `children` por último), e `resolve_figma.rs`, que entrega o
  `Resolution`. cor passa por `css_map::color_token` (`css_map.rs:301`),
  e os dois emissores a leem como leem as das outras fontes
  (`Tokens::color_of`, `tokens.rs:164`).

### (b) o que o `ParserNode` precisa

a struct não muda (`ir.rs:12-20`). o que entra é aditivo, e nenhuma das
fontes que existem produz:

1. o prop `gap` (o `.gap(v)` do builder,
   `rust/engine/src/compose/builder/element/builders.rs:77`). nenhum resolve o
   escreve hoje; o container do emissor de IR manda todo nome que não
   conhece para `lost` (`emit_ir/props.rs:490`), então ele aprende o gap
   do stack, pela mesma medida de token dos spacers
   (`emit_ir/props.rs:378`).
2. `align_items(Align::X)` (`builders.rs:173`) para início, fim e
   esticar. o `emit.rs` só importa `Justify` quando usado
   (`emit.rs:16-33`); passa a importar `Align` do mesmo jeito. react e
   gpui nunca escrevem `Align::`, o código deles fica igual byte a byte.
3. `DropKind::Approximated` (`ir.rs:189-194`). o `Resolution` hoje só
   diz perdido ou reescrito (`ir.rs:189-194`, notas em `ir.rs:227`), e o
   emissor os leva a `impossible` e `transformed` (`emit_ir/mod.rs:77-84`);
   a regra de posição do Figma precisa de `approximated` com a diferença
   em px. variante nova, que react e gpui não produzem.
4. `Origin.tag == "img"` vira o componente `image` do `uir`
   (`rust/plevir/src/vocabulary.rs:339`, nome sempre, `source` opcional): um
   braço novo no `node()` do emissor antes do container
   (`emit_ir/mod.rs:226-227`). a fixture react não tem `<img>`; um
   `<img>` do corpus viraria `image` em vez de grupo, só no IR.

atributos: `onclick`, `oninput` e `onchange` do HTML são normalizados
para os nomes que o `control.rs` lê (`onClick`, `onChange`;
`emit_ir/control.rs:53`, `:145`, `:282`), com o texto da fonte guardado
no `Attr.src`. nada disso muda o que react ou gpui produzem: os goldens
atuais passam sem regravar.

### (c) mapeamento para o IR

HTML/CSS (subset):

| construção | `ParserNode` | IR | categoria |
|---|---|---|---|
| `div`, `section`, `header`, `ul`, `li` | `Div` | grupo | supported |
| `p`, `span`, `h1` a `h3`, `label` com texto | `Text` literal | `text`, label | supported |
| estilo padrão do navegador (margem de `p`, `h1` a `h3`, `ul`; tamanho e peso de título) sem declaração | nada | nada | impossible |
| `<button>` | origem `button` | `button`, `activate` ligado | transformed, binding required |
| `<input>` de texto, `<textarea>` | origem `input` | `text_field`, `set_value` ligado | transformed, binding required |
| `onclick="f()"` | `onClick` = `f` | ação `f` | transformed; outro código, approximated |
| `<img>` | `Div` com `w`/`h`, origem `img` | `image`, label do `alt` | transformed; `src` impossible |
| `display: flex`, `flex-direction` | `row`/`col` | eixo do stack | supported |
| `gap`, `margin-bottom` | `gap`, spacer | gap do stack | pela medida |
| `align-items` | `align_items` | align do stack | supported |
| `justify-content` | `justify` | nada | impossible (o IR não justifica) |
| `flex-grow`, `flex: n` | `grow` | `size.flex` | supported |
| `padding`, cor, fundo, raio, fonte, largura e altura em px | props do builder | tokens | pela medida (tolerância da fase 1) |
| `display: grid`, `position`, `top`/`left`, `z-index`, `float` | nada | nada | impossible |
| `@media`, pseudo-classe, seletor que não é `.a`, `#a` ou `tag` | nada | nada | impossible (a regra cobre as declarações) |
| `<script>`, outro `on*` | nada | nada | impossible |
| `border`, `box-shadow`, `overflow` | props do builder | nada | impossible (motivos da fase 1) |

Figma (export JSON da API REST):

| construção | `ParserNode` | IR | categoria |
|---|---|---|---|
| `DOCUMENT`, `CANVAS`, `SECTION` | nada | a superfície é o primeiro `FRAME` | transformed |
| `FRAME`/`INSTANCE`/`COMPONENT` com `layoutMode` | `Div` `row`/`col` | stack | supported |
| `itemSpacing` | `gap` | gap do stack | pela medida |
| `padding*` | `p`/`px`/`py`/... | padding (o menor lado) | pela medida |
| `counterAxisAlignItems` | `align_items` | align do stack | supported |
| `primaryAxisAlignItems` diferente de `MIN` | `justify` | nada | impossible |
| `layoutGrow` | `grow` | `size.flex` | supported |
| `layoutAlign: STRETCH` | nada | o stack estica | supported quando o pai estica, senão impossible |
| tamanho fixo (`*SizingMode: FIXED`, sem auto layout) | `w`/`h` | token de tamanho | pela medida |
| frame sem auto layout, filhos sem sobreposição num eixo | `col`/`row` na ordem da posição | stack | approximated (a diferença em px) |
| frame sem auto layout com filhos sobrepostos | `col` na ordem da fonte | stack | impossible |
| `fills` sólido | `bg` / `text_color` | papel de cor | pela medida |
| `fills` de imagem ou gradiente | nada | nada | impossible |
| `cornerRadius` | `rounded` | shape | pela medida |
| `TEXT` `characters` | `Text` literal | `text`, label | supported |
| `style` `fontSize`, `lineHeightPx`, `fontWeight` | props de texto | tipografia | pela medida |
| `style` `fontFamily`, `letterSpacing`, alinhamento | nada | nada | impossible |
| `strokes`, `effects`, `opacity`, `clipsContent` | `border`/`clip_children` ou nada | nada | impossible |
| `RECTANGLE` com fill sólido | `Div` | grupo com fundo | supported |
| `VECTOR`, `ELLIPSE`, `STAR`, `LINE`, `BOOLEAN_OPERATION` | nada | nada | impossible (o IR não tem ícone nem desenho) |
| `visible: false` | fora | fora | supported (não desenha) |
| override que o export não aplicou | nada | nada | impossible |

### (d) o script do Figma

`tool/tasks/figmapull.py` (python 3.14, só stdlib: `zipfile`, `zlib`,
`compression.zstd`, `json`) lê o `.fig` (um zip com `canvas.fig` em
fig-kiwi: o schema kiwi em deflate cru, a mensagem em zstd), monta a
árvore por `parentIndex` (ordem por `position`), acha a página cujo nome
tem "Dark Theme", a seção "settings" e o frame "settings", e escreve o
JSON só com o que o importer lê, uma chave por linha, valores compactos,
`children` por último, ASCII (o nome da página tem emoji; o arquivo o
guarda escapado).

instâncias: um `INSTANCE` recebe uma cópia dos filhos do `SYMBOL` de
`symbolData.symbolID`, recursiva (instância dentro de símbolo também
expande), com ids `I<instância>;<filho>` como os da API. cada override de
`symbolData.symbolOverrides` casa pelo `guidPath` com o `guid` ou o
`overrideKey` dos nós copiados (o caminho desce uma instância por guid) e
aplica os campos de desenho que o importer lê (fills, strokes, texto,
visibilidade, tamanho, nome, opacidade, fonte, raio, efeitos); o
`derivedSymbolData` dá tamanho e posição dos filhos de uma instância
redimensionada. o que não aplica (campo fora da lista, como variável,
estilo por referência, propriedade de componente; ou caminho que não
casa) fica em `unappliedOverrides` da instância, e o importer reporta
cada um como impossible. imagem: só o tipo do fill, nunca os bytes; o
lugar e o tamanho ficam em `absoluteBoundingBox`, composto das
translações; uma rotação fica em `rotation` e vai ao relatório. não
cobre: geometria de vetor, máscara, constraints, propriedades de
componente, variáveis e modos.

## handoff da fase 2 (2026-09-25, agente, branch task/71b-importers)

Goal: task-71 fase 2, duas fontes novas no parser, HTML/CSS (subset) e
Figma, cada uma com fixture, goldens e relatório em quatro categorias.
Status: pronto para validação do orquestrador. fecha os critérios 3 e 4;
com a integração, a task-71 vai para `checked`. nada no `uir`, no
`surface`, no `sem` nem no código builder das fontes antigas mudou.

### commits

1. `e3201af7` a sonda (a) a (d), acima.
2. `6e99cec8` as duas dependências.
3. `7ca7d41e` o importer HTML/CSS com a fixture (`fixtures/html/`).
4. `0cb09a90` o `tool/tasks/figmapull.py` e a tela exportada
   (`fixtures/figma/settings.json`).
5. `4e16c349` o importer Figma, os goldens e o desenho headless das duas.
6. este commit: a adição da fase 2 no ADR e este handoff.

### as duas fontes

- HTML/CSS: `html.rs` (parse), `html_map.rs` (o subset de CSS),
  `resolve_html/` (cascata por especificidade, herança de texto, a
  moldura do documento). a fixture é o note card escrito como página, a
  mesma forma do react; liga o mesmo app (`reply`, `open`). relatório: 32
  supported, 22 transformed, 17 approximated, 13 impossible, 2 bindings.
- Figma: `figma.rs` (JSON e a linha de cada chave), `resolve_figma/`. a
  tela "settings" da página "Dark Theme" (98 nós, instâncias expandidas).
  relatório: 251 supported, 27 transformed, 119 approximated, 196
  impossible, nenhum binding (o design não nomeia capacidade; o IR
  verifica sem nada declarado e não inventa controle).
- a regra de posição (approximated): frame sem auto layout com filhos que
  não se sobrepõem em y vira coluna na ordem de y (em x, linha); o
  deslocamento de cada filho vai ao relatório em px. sobreposto é
  impossible.
- o script resolve cor ligada a variável no modo da tela (a status bar
  sai branca no modo dark); o que não aplica fica em
  `unappliedOverrides` e vira impossible. o emoji do nome da página fica
  escapado no JSON e no relatório.
- aditivo, que nenhuma fonte antiga produz: props `gap` e
  `align_items(Align::..)`, `DropKind::Approximated`, `<img>` como
  componente `image`. os goldens react e gpui passam sem regravar.

### PNGs (headless, não são golden)

`cargo test -p import --test pixels -- --ignored write_the_frames`
escreve em `/Users/nn/Dev/hoff/plevdev/target/agents/one/tmp/`:
`html-note-card-ir.png`, `figma-settings-ir.png` (o IR materializado) e
`html-note-card-builder.png`, `figma-settings-builder.png` (o código
builder). o IR do Figma desenha só texto: os fundos da tela estão fora da
tolerância dos tokens (impossible) e os ícones são vetores.

### diff do lock

```text
@@ -3514,6 +3514,8 @@ dependencies = [
  "surface",
  "thiserror 2.0.18",
  "tree-sitter",
+ "tree-sitter-css",
+ "tree-sitter-html",
  "tree-sitter-rust",
  "tree-sitter-typescript",
  "uir",
@@ -4836,6 +4838,26 @@ dependencies = [
+[[package]]
+name = "tree-sitter-css"
+version = "0.25.0"
+dependencies = [ "cc", "tree-sitter-language" ]
+
+[[package]]
+name = "tree-sitter-html"
+version = "0.23.2"
+dependencies = [ "cc", "tree-sitter-language" ]
```

arestas novas: `parser -> tree-sitter-html`, `parser -> tree-sitter-css`;
as duas usam `cc` 1.2.63 e `tree-sitter-language` 0.1.7, que o lock já
tinha. nenhum outro pacote.

### validação

- `./tool/tasks/fullcheck.sh` verde, pernas wasm e urnaui: 3163 passam, 0 falham, 28
  ignorados. o parser foi de 49 para 69 testes (+20), nenhum removido.
  disco com 13 GB livres no fim.
- `cargo run --offline -p regress -- check`: 176
  baselines, 176 iguais.

### linha para o índice de ADR

`| 2026-09-25 | [importers-target-plev-ir](../../decisoes/importacao/0069-importers-target-plev-ir.md) | proposed | o parser ganha o segundo emissor, ParserNode para plev IR, ao lado do código builder (que não muda um byte); relatório em quatro categorias com arquivo:linha que estende a droplist, forma JSON fixa; tolerância por token medida no tema (metade da distância ao vizinho); controle vira componente com role ligado a uma ação nomeada em <superfície>.props (binding required); pixels contra o builder com cada diferença explicada (task-71 fase 1); fase 2: HTML/CSS subset (tree-sitter-html e -css) e Figma (export REST de um .fig por tool/tasks/figmapull.py) pelo mesmo Resolution e os mesmos emissores, regra de posição approximated, cor por variável no modo da tela |`

### pendências

- no IR do Figma, "Terms of Service" quebra em duas linhas e desenha
  por cima do subtítulo: é a medida de altura de texto quebrado do
  `surface` numa coluna estreita, fora deste escopo.
- ícone e forma (vetor, elipse) não têm componente no IR; o `src` de uma
  imagem também não. crescem com o vocabulário do `uir`.
- HTML: fluxo inline vira coluna (approximated), unidades `em`/`rem`/`%`
  não são lidas, `<a>`, `<form>`, `<select>` e checkbox ficam fora do
  subset.
- Figma: propriedades de componente, variável de biblioteca, constraints
  e geometria de vetor não entram; a regra de posição só vale sem
  sobreposição.
- um `<img>` do card do corpus react passaria a `image` (só no IR); o
  corpus não está neste checkout, então não foi conferido.
