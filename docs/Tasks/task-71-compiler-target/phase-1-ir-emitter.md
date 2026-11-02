---
id: task-71/phase-1-ir-emitter
title: "task-71 phase 1: probe and handoff of the plev IR emitter"
project: plev
type: section
parent: task-71
status: reference
date: 2026-09-25
date_basis: text
recorded: 2026-09-23
topics: [importing, ir, formats]
relations: [task-66, task-68, task-69, adr-0064, adr-0069]
evidence: [0c08e77a, 9f7e9cfd, 14cd3dfd, 7a240577, 9707a2a6]
source: docs/log/steps/task-71-compiler-target.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-71 phase 1: probe and handoff of the plev IR emitter

Records the probe done before any code and the phase 1 handoff of task-71: the parser gains a second emitter, from `ParserNode` to plev IR, alongside the builder-code emitter, which stays byte for byte the same. The report in four categories (`supported`, `transformed`, `approximated`, `impossible`) extends the droplist with `arquivo:linha`, the controls become components with a role and `binding required`, and the IR of the fixtures is compared pixel by pixel with the builder code. The pixel and test numbers recorded are from this machine.

## sonda da fase 1 (2026-09-25, agente, branch task/71a-compiler-target)

feita antes de qualquer código, contra a `main` em `0f32f69d`. toda
afirmação aponta arquivo e linha.

### (a) o `ParserNode` campo a campo contra o IR do `uir`

o `ParserNode` (`rust/import/src/tree.rs:7-13`) tem `tag`, `props` e
`children`; `Tag` é `Div`, `Text` e `Slot` (`ir.rs:15-22`); os props já são
nomes de método do builder com argumentos literais ou expressões de token
(`ir.rs:33-62`). o IR pede identidade (`IrNode.sem`,
`rust/plevir/src/ir.rs:62-83`), um componente de vocabulário fechado
(`rust/plevir/src/vocabulary.rs:50-93`), props só por token
(`rust/plevir/src/token.rs:174-192`), layout por primitivas
(`token.rs:202-224`), tamanho por token e peso (`token.rs:236-245`) e
ligação a estado e ação que o app declara (`ir.rs:111-130`).

| do `ParserNode` | no IR | como |
|---|---|---|
| `Tag::Div` | nó sem componente (container, `Group`) | direto |
| `Tag::Text(Literal)` | `text` com `label` | direto |
| `Tag::Text(Param)` | `text` com `bind.state.text` | regra: o parâmetro vira estado que o app declara (`binding required`) |
| `Tag::Slot` | nada | perde: o IR não guarda elemento montado pelo chamador |
| `row`, `align_center` | `layout.stack` horizontal, `align: center` | direto |
| `justify`, `shrink` | nada | perde: o stack do IR não tem justify nem shrink |
| `grow(n)` | `size.flex` | direto quando `n` é inteiro de 1 a 12 |
| `w(px)` | `size.min` e `size.max` no token de `theme.size` | regra, por medida (d) |
| `h(px)` | nada, salvo o token de altura (`dropdown_max_h`) | por medida (d); o resto perde |
| `p`, `px`, `py`, `pt`, `pb`, `pl`, `pr` | `props.padding`, um token para os quatro lados | por medida (d); lados diferentes aproximam |
| `bg`, `text_color` | `props.background`, `props.color` (papel de cor) | direto quando é `theme.colors.<papel>`; o resto por medida (d) |
| `rounded` | `props.shape` | por medida (d) |
| `font_size`, `line_height`, `font_weight` | `props.typography` (um papel da rampa) | por medida (d) |
| `border`, `border_color`, `shadow_inset`, `shadow_drop`, `backdrop_blur`, `clip_children` | nada | perde: o IR não tem borda, sombra solta, blur nem clip |
| o espaçador `div().h(v)` da reescrita de `margin-bottom` | `layout.stack.gap` do pai | regra (o menor espaçador vira o gap; os maiores aproximam) |
| `Resolution.fn_name` | `surface.id` e título | regra |

o que falta no `ParserNode` para o IR, sem mudar o parse: a tag de
origem (um `<button>` e um `<div>` viram o mesmo `Tag::Div`), os
atributos que não são `className` (`onClick`, `placeholder`; o
`tsx.rs:99-112` só lê `className`), a linha de cada nó e a construção de
origem de cada prop (o resolve só conta, `resolve_react.rs:241-265`,
`resolve_gpui.rs:130-133`). pela autorização do orquestrador isso entra de
forma aditiva dentro do `rust/import`: um `origin` no `ParserNode`, a
lista de origens em cada `Prop` (fora do `PartialEq`, que continua
comparando nome e argumentos), notas para as construções contadas que não
viram prop, e o `covers` e o tipo (perdido ou reescrito) em cada
`Dropped`. o emissor de código não lê nenhum desses campos: o código
emitido continua byte a byte igual (golden do gpui e o card do corpus,
conferido fora do repo).

dois silêncios do parse atual aparecem no caminho e ficam com entrada, não
com silêncio: filho `{expressão}` que não é identificador é ignorado
(`tsx.rs:126-133`), e estilo de texto de um elemento sem texto some
(`resolve_react.rs:136-203`). os dois entram na droplist. nenhum dos dois
existe no card do corpus nem no separador: as contagens congeladas (51/38
e 18/6) não mudam.

o IR não precisou mudar. a construção tipada usa os tipos do `uir`; os ids
e as referências são escritos como texto e lidos pelo parse do próprio
`sem` (sem dependência nova).

### (b) as fixtures

- react: o card mora fora do repo. `tests/golden.rs:35-40` lê
  `refs/parsecomponentes/UIvisualREFs/cards/hoff-research-cards/react`,
  ausente neste checkout e na `main`; os três testes do react retornam
  cedo (passam sem rodar) e o `dump_droplists` é ignorado. o dono aposentou
  as cópias em `f93a57ea`.
- gpui: `fixtures/gpui/separator.rs` e o golden `fixtures/gpui/expected.rs`
  rodam no gate (`golden.rs:111-115`, `golden.rs:160-181`,
  `tests/emitted_compiles.rs`).
- fixture desta fase no react: uma nova, escrita para a fase, não copiada
  (`fixtures/react/`): um card na mesma forma do card do corpus (overlay,
  inner, preview com slot, detalhes com título e conteúdo) mais um campo
  de texto e um botão com `onClick`, valores da paleta HOFF, para o gate
  exercitar o react sem o corpus. o card do corpus continua ao vivo: o
  mesmo teste de IR roda nele quando o corpus existe (sem golden, como o
  `golden.rs`).
- goldens novos por fixture: IR (`expected.ir.json`), relatório
  (`expected.report.json`) e, para o react, o código builder emitido
  (`expected.rs`), que a comparação de pixels inclui e executa.

### (c) o relatório em quatro categorias

cada construção da fonte vira exatamente uma entrada com `at`
(`arquivo:linha`), `construct` (o nome que a droplist já usa) e uma
categoria: `supported` (com `ir`, onde caiu), `transformed` (com `ir` e
`rule`), `approximated` (com `ir` e `difference`) ou `impossible` (com
`reason`). a droplist é estendida, não perdida: toda entrada dela vira
entrada do relatório no mesmo `at` (`impossible` com o `why` como motivo;
as duas reescritas do separador, `mx_auto` e a linha absoluta, como
`transformed`), e as construções que ela só contava ganham entrada pela
origem. entradas que respondem por um bloco inteiro (uma regra não
aplicada, uma subárvore decorativa, um braço de `match`) listam em
`covers` as linhas que cobrem. as ligações que o app precisa fazer vão em
`bindings`. formato JSON com nomes estáveis, escrito num lugar só
(`report.rs`), documentado no módulo e no ADR; é gerado com
`serde_json::json!`, porque o `serde` com derive não está entre as
dependências autorizadas.

prova de que nada some: um censo independente do resolve, lido direto da
saída do parse (cada elemento, texto e `{expressão}` do tsx, cada classe
condicional, cada declaração de cada regra do sass; cada chamada de
método alcançável a partir do `render` do gpui), e o teste exige que cada
item do censo seja o `at` de uma entrada ou esteja no `covers` de uma; e o
inverso, que todo nó do IR tenha ao menos uma entrada que o nomeia.

### (d) tokens e aproximação

o emissor de IR mede contra os valores do tema em que o código builder foi
escrito (`Theme::hoff()`, o que o golden usa). o `parser` não depende do
`engine`, então recebe esses valores como tabela (`Tokens`) montada por
quem chama; nos testes, do próprio `Theme::hoff()`.

regra: o valor vai para o token mais próximo. distância 0 é
`transformed`; até a tolerância do token é `approximated` com a
diferença; além dela é `impossible` e o prop fica fora do IR. a tolerância
de cada token é metade da distância até o vizinho mais próximo de valor
diferente, medida no tema: dentro dela o valor está mais perto desse
token do que de qualquer outro.

- cor: distância é a maior diferença entre os quatro canais em unidades
  de 8 bits (0 a 255, alfa incluído), a mesma medida do ADR
  `what-identical-means`. no hoff: `bg` e `surface` 1 nível (estão a 2 um
  do outro, `#303030` e `#2E2E2E`), `bg_hover` e `divider` 3,5, `text_mid`
  25,5, `text` 31,9. exemplo: `theme.glass.inset_highlight` (`n2(0.06)`)
  fica a 2,55 de `bg_hover`, aproxima; `theme.glass.button` (`n3(0.70)`)
  fica a 76,5 de `bg`, impossível; `CARD_OVERLAY` é igual a `surface`,
  transformado.
- espaço: `theme.spacing` 4, 8, 12, 16, 24, 32; tolerância 2 px de `xs` a
  `lg` e 4 px em `xl` e `xxl`.
- raio (`theme.shape`), largura (`theme.size`) e tipografia (tamanho e
  entrelinha da rampa) seguem a mesma regra, com os números no ADR.

### (e) semântica e `binding required`

`<button>`, qualquer elemento com `onClick` e `<input>` viram componente
com role (`button`, `text_field`). o controle liga o evento do componente
(`activate`, `set_value`) a uma referência nomeada pelo importer num nó
do app, `<superfície>.props`, com o nome do handler (`onOpen` vira `open`)
ou, sem handler, do rótulo; o parâmetro de texto liga `text` a
`<superfície>.props:<parâmetro>`. o IR só referencia, nada nele cria
capacidade (`rust/plevir/src/ir.rs:104-110`). com o registro vazio o
verificador recusa pela regra 5 ou 4 dizendo exatamente o que falta ("the
app declares no action `open` on `note_card.props`",
`rust/plevir/src/verify/bind.rs:176` e `:139`); o relatório lista as
mesmas referências em `bindings`. o teste declara o registro, como o app
faria, e o mesmo IR passa. deixar o controle sem ligação não serve: a
regra 7 recusa controle só visual (`rust/plevir/src/verify/access.rs:120-127`)
sem dizer qual capacidade falta.

### (f) pixels

os dois lados no mesmo `engine::headless::Renderer`
(`rust/engine/src/drawing/headless/mod.rs:138-270`), mesmo tamanho, escala 1,
tema `Theme::hoff()` e fundo `theme.colors.bg`: o código builder emitido
(o golden do gpui e o `expected.rs` do react, incluídos e executados como
o `tests/emitted_compiles.rs` já faz) desenhado pelo `render` do
`Element`, e o IR verificado e materializado pelo `surface`
(`Surface::materialize` e `render`). a métrica é a do ADR
`what-identical-means`: quantos pixels diferem, o maior e o médio delta
de canal e o retângulo que os contém. os dois caminhos são diferentes de
propósito (o IR não carrega pixel), então o veredito esperado é uma
diferença, e cada região dela é explicada pelos retângulos dos nós (o do
builder contra o do `surface`, que o `Compositor::binding` dá) e pelas
entradas do relatório. sem adaptador de GPU o teste pula.

### o que a fase 2 precisa (registrado, fora do escopo)

- HTML/CSS: uma grammar HTML (tree-sitter-html ou parecida, dependência
  nova a aprovar) e um leitor de CSS além do sass de identação que
  existe; o resolve e os dois emissores são os mesmos.
- Figma: um export salvo da API REST (JSON de frames, auto layout e
  estilos) como fixture, nunca rede no teste; frames e auto layout viram
  `stack`/`grid`, estilos viram token pela mesma medida de (d).

## handoff da fase 1 (2026-09-25, agente, branch task/71a-compiler-target)

Goal: task-71 fase 1, o parser emite plev IR verificado com relatório em
quatro categorias e `binding required`.
Status: pronto para validação do orquestrador. nada no `uir`, no
`surface`, no `sem` nem no código builder emitido mudou.

### commits

1. `0c08e77a` a sonda (a) a (f), acima.
2. `9f7e9cfd` o `ParserNode` carrega a origem de cada construção.
3. `14cd3dfd` o segundo emissor, `ParserNode` para plev IR, com o
   relatório em quatro categorias.
4. `7a240577` controles da fonte viram componente com role, `binding
   required`; a fixture react da fase.
5. `9707a2a6` o IR materializado e comparado por pixel com o código
   builder.
6. este commit: ADR `importers-target-plev-ir` (proposed) e este handoff.

### o emissor e o que se perde

- `rust/import/src/emit/plevir/` (`mod.rs`, `props.rs`, `control.rs`) lê o
  mesmo `Resolution` do `emit.rs` e escreve `uir::Ir`. o `emit.rs` não
  mudou. a origem entrou de forma aditiva (`ir.rs`: `Origin`, `Prop::src`
  fora do `PartialEq`, `Note`, `DropKind`, `covers`).
- código builder igual byte a byte: o golden do gpui passa; o card do
  corpus e 128 componentes react reais (três apps locais) emitidos antes e
  depois dão o mesmo código (conferido trocando os fontes do parser pelos
  da `main` no próprio worktree e voltando).
- a droplist ganhou entradas para dois silêncios do parse: filho
  `{expressão}` que não é identificador e estilo de texto que nenhum texto
  recebe (26 dos 128 componentes; o código deles não muda). card do corpus
  e separador sem mudança (51/38 e 18/6).
- o que o IR v1 não diz e vira `impossible`: tamanho fora de token (a
  altura de 160 do preview), borda, sombra solta, blur, clip, padding por
  lado com 0, justify, shrink, slot de elemento, variante por prop, media
  query, hover, animação, atributo sem uso (`role`).

### o relatório e o formato

- `rust/import/src/report.rs`: uma entrada por construção com `at`,
  `construct` e a categoria (`supported`+`ir`, `transformed`+`ir`+`rule`,
  `approximated`+`ir`+`difference`, `impossible`+`reason`), `covers` nas
  entradas de bloco, `bindings` à parte. JSON `plev_report: 1`, nomes
  fixos, chaves em ordem sem depender da feature do `serde_json`.
  documentado no módulo e no ADR.
- a droplist inteira está no relatório, na mesma linha (perdida como
  `impossible` com o `why`; as duas reescritas do separador como
  `transformed`).
- prova: o censo do parse (tsx, sass, chamadas do gpui) contra o
  relatório, e todo nó do IR com entrada (`tests/ir.rs`).
- contagens: separador 8 supported, 9 transformed, 3 approximated, 6
  impossible; note card 8, 32, 16, 19; card do corpus (ao vivo) 10, 33,
  15, 45.

### a tolerância medida

metade da distância do token ao vizinho mais próximo de outro valor, no
`Theme::hoff()`. cor em níveis de 8 bits com alfa (medida do ADR
`what-identical-means`): `bg` e `surface` 1, `bg_hover` e `divider` 3,5,
`text_mid` 25,5, `text` 31,9. espaço: 2 px de `xs` a `lg`, 4 px em `xl`
e `xxl`. raio `pill` 4 px, largura `card_w` 16 px, `caption_r` 1 px. o
teste `the_tolerances_measured_on_hoff` fixa esses números. distância 0 é
`transformed`, dentro da tolerância `approximated`, fora `impossible`.
padding e gap tomam o token do menor valor (o IR nunca põe espaço que a
fonte não tem).

### binding required

controle (`<button>`, `onClick`, `<input>` de texto, `textarea`) vira
`button` ou `text_field` ligado a uma ação em `<superfície>.props`
nomeada pelo handler (`onOpen` vira `open`) ou pelo rótulo; parâmetro de
texto liga `text` a `<superfície>.props:<nome>`. com registro vazio o
verificador recusa uma vez por ligação, nomeando cada uma (regras 4 e 5),
e o relatório lista as mesmas; com o registro que o teste declara (como o
app), verifica. note card: `title`, `content` (estado), `reply` (texto) e
`open`; separador: `label`; card do corpus: `title`, `content`,
`discover`.

### fixtures e pixels

- react: `rust/import/tests/fixtures/react/` escrita para a fase, não copiada
  (forma do card do corpus mais um campo e um botão com `onClick`); o card
  do corpus segue fora do repo e roda ao vivo quando existe. gpui: o
  separador de sempre. goldens novos: `expected.ir.json`,
  `expected.report.json` (os dois) e `expected.rs` (react).
- pixels (`tests/pixels.rs`, headless, escala 1, `Theme::hoff()`, M4
  metal): separador, 1266 pixels diferentes (max 164); a linha fica 6 px
  mais baixa porque o separador do surface ocupa 36 px numa linha; movido
  de volta, sobram 750 (max 18), só nas duas linhas da reta (o surface
  centra a reta em 17,5) e 4 px de reta a mais de cada lado do rótulo
  (padding `xs` contra 8); o rótulo é o mesmo pixel. note card, 99001
  pixels (max 197): tudo abaixo do preview sobe 160 px e vai 16 px para a
  esquerda; título e conteúdo são os mesmos pixels, só movidos; a linha
  meta difere em 217 pixels por no máximo 5 níveis (cor aproximada); campo
  e botão são widgets do comps; o card fica 180 px mais baixo, cada px
  explicado. os testes afirmam a estrutura, não as contagens.

### diff do lock (só o pacote parser)

```text
@@ -3489,14 +3489,19 @@ dependencies = [
 name = "parser"
 version = "0.1.0"
 dependencies = [
+ "comps",
  "criterion",
  "engine",
  "env_logger",
  "pollster",
+ "sem",
+ "serde_json",
+ "surface",
  "thiserror 2.0.18",
  "tree-sitter",
  "tree-sitter-rust",
  "tree-sitter-typescript",
+ "uir",
 ]
```

`uir` e `serde_json` em `[dependencies]`; `surface`, `sem` e `comps` em
`[dev-dependencies]`. nenhum pacote novo.

### validação

- `./tool/tasks/fullcheck.sh` verde, pernas wasm e urnaui: 2990 passam, 0 falham, 27
  ignorados (antes 2960 e 26: +30 testes, +1 ignorado, o diagnóstico que
  escreve os pngs). disco com 16 GB livres.
- `cargo run --offline -p regress -- check`: 175
  baselines, 175 iguais.
- no parser: 29 testes de unidade, 11 em `tests/ir.rs`, 2 em
  `tests/pixels.rs` (pulam sem adaptador de GPU).

### linha para o índice de ADR

`| 2026-09-25 | [importers-target-plev-ir](../../adr/Importing/0069-importers-target-plev-ir/0069-importers-target-plev-ir.md) | proposed | o parser ganha o segundo emissor, ParserNode para plev IR, ao lado do código builder (que não muda um byte); relatório em quatro categorias com arquivo:linha que estende a droplist, forma JSON fixa; tolerância por token medida no tema (metade da distância ao vizinho); controle vira componente com role ligado a uma ação nomeada em <superfície>.props (binding required: registro vazio recusado dizendo o que falta); pixels contra o builder com cada diferença explicada (task-71 fase 1) |`

### limitações e riscos

- o card do corpus não tem golden (está fora do repo); o gate roda a
  fixture escrita para a fase.
- o builder alinha filhos no início, o stack do IR estica: explica a
  largura do `<input>` na comparação.
- `Tokens` é preenchido por quem chama (o parser não depende do engine);
  os testes preenchem do `Theme::hoff()`. um token sem valor vira
  `impossible`, nunca silêncio (`Tokens::missing`).
- os números de pixel são desta máquina; os testes valem por estrutura em
  outra GPU.
- um `<input>` de outro tipo (checkbox, range) não é inferido na fase 1 e
  o relatório diz.

### fase 2

HTML/CSS pede uma grammar nova (tree-sitter-html ou parecida, a aprovar)
e um leitor de CSS; Figma pede um export salvo da API REST como fixture,
nunca rede no teste. os dois passam pelo mesmo resolve e pelos dois
emissores. SwiftUI e Flutter ficam registrados; controles do gpui
(`on_click`) e listas do react (`items.map` como `repeat`) também.
