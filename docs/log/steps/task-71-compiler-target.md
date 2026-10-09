---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-09-25
domain: generative-runtime
task: task-71
wave: 7
depends: [task-68]
branch: task/71-compiler-target
---

# task-71: plev como compiler target (importers para IR)

## objetivo

o parser deixa de ser só um conversor de React para código builder e vira
front-end de um compiler target: fontes de UI (React, gpui hoje; HTML/CSS,
Figma, SwiftUI e Flutter depois) convergem para plev IR, com um relatório
honesto do que foi suportado, transformado, aproximado ou é impossível. o
runtime nunca carrega o modelo de execução da origem.

## ligação com a tese

seção 3 (os front-ends convergindo para plev IR), seção 14 (plev como
compiler target; suportado, transformado, aproximado, impossível; nunca
fingir equivalência silenciosa), e a nota das anotações: "Figma -> plev IR
<- React importer <- LLM -> live application". goal 9 do programa.

## estado atual no código

- `rust/import`: parse (tree-sitter: `tsx.rs`, `sass.rs`, `gpui.rs`),
  resolve (`resolve_react.rs` via `css_map.rs`, `resolve_gpui.rs`) para o
  `ParserNode` IR, emit (`emit.rs`) de código Rust contra
  `engine::builder`. o droplist reporta com arquivo:linha tudo que não
  passa (`rust/import/src/lib.rs`, ADR
  `transpiler-reports-every-unmapped-construct`). escopo brutalmente
  pequeno: um componente react (o card) e um widget gpui (o separador).
  marcado experimental.
- ADR `import-foreign-formats-by-conversion-not-embedding`: conversão, nunca
  runtime estrangeiro embutido.
- deps já presentes: `tree-sitter`, `tree-sitter-typescript`,
  `tree-sitter-rust`.

## escopo

1. segundo emissor: `ParserNode -> plev IR` (68), ao lado do emissor de
   código builder existente. o emissor de código continua até uma decisão
   explícita de removê-lo.
2. relatório em quatro categorias substituindo ou estendendo o droplist:
   `supported`, `transformed` (com a regra aplicada),
   `approximated` (com a diferença estimada, por exemplo a cor fora do
   token mais próximo), `impossible` (com o motivo). arquivo:linha em
   todas.
3. inferência de semântica honesta: tag e handler que indicam controle
   (button, `onClick`, input) viram componente com role e um
   `binding required` no relatório; o importer nunca inventa capacidade, só
   marca onde o app precisa ligar uma.
4. as duas fixtures atuais produzem IR que verifica contra um registro de
   capacidades de teste e materializa (69) com pixels próximos do código
   builder emitido hoje (comparação por headless, 66).
5. próximas fontes, cada uma com o mesmo formato de parse, resolve, emit:
   - HTML/CSS (subset; dependência nova de grammar justificada);
   - Figma (export JSON da API REST: frames e auto layout para
     `Stack`/`Grid`, estilos para tokens pelo token medido mais próximo, com
     `approximated` quando fica fora da tolerância);
   - SwiftUI e Flutter registrados como próximas, sem implementação nesta
     task.

## fases

- fase 1 (aberta em 2026-09-25): itens 1 a 4 do escopo. o segundo
  emissor, o relatório em quatro categorias, o `binding required` e as
  fixtures em IR com a comparação de pixels. dependências autorizadas pelo
  orquestrador: `uir` no parser e, nos testes, `surface`, `sem` e `comps`,
  todos do workspace; nenhum pacote novo no lock.
- fase 2: a fonte nova (item 5). HTML/CSS pede grammar nova e o Figma
  pede um export real; as duas passam pelo brenner.

## fora de escopo

LLM como fonte (já é o proponente da 69); runtime de qualquer framework de
origem.

## dependências

68 integrada; 69 e 66 para a comparação de pixels.

## superfície sob responsabilidade

`rust/import/**`, proposta de ADR `docs/adr/importing/0069-importers-target-plev-ir.md`.
dependência nova (grammar HTML, cliente do export Figma) só com
justificativa aprovada.

## interfaces que consome

o IR e o verifier (68), host de superfícies (69), headless (66).

## interfaces que pode alterar

`rust/import` inteiro; o formato do relatório é contrato com quem o lê
(documentado).

## critérios de aceitação

- as duas fixtures em IR verificado, com relatório em quatro categorias.
- comparação de pixels entre o IR materializado e o builder emitido, com a
  diferença explicada quando houver.
- uma fonte nova (HTML/CSS subset ou Figma) com fixture e relatório.
- `./tool/tasks/fullcheck.sh` verde.

## testes esperados

goldens de IR e de relatório por fixture; teste de que nenhuma construção
some sem aparecer numa categoria; teste de `binding required`.

## riscos e perguntas abertas

- Figma exige token de API e arquivo real; a fixture usa um export salvo,
  nunca rede no teste.
- a tolerância de "aproximado" para cor e espaço precisa de número; vem da
  medição dos tokens existentes, não chute.

## commits esperados

1. `task-71: emissor de plev IR a partir do ParserNode`
2. `task-71: relatório em quatro categorias`
3. `task-71: inferência de semântica com binding required`
4. `task-71: fixtures em IR e comparação de pixels`
5. `task-71: importer de <fonte nova>`

## pare e reporte se

- o `ParserNode` não carregar informação suficiente para o IR sem mudar o
  parse;
- uma fonte nova exigir dependência pesada.

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
- `cargo run --offline -p showcase --example regress -- check`: 175
  baselines, 175 iguais.
- no parser: 29 testes de unidade, 11 em `tests/ir.rs`, 2 em
  `tests/pixels.rs` (pulam sem adaptador de GPU).

### linha para o índice de ADR

`| 2026-09-25 | [importers-target-plev-ir](importers-target-plev-ir.md) | proposed | o parser ganha o segundo emissor, ParserNode para plev IR, ao lado do código builder (que não muda um byte); relatório em quatro categorias com arquivo:linha que estende a droplist, forma JSON fixa; tolerância por token medida no tema (metade da distância ao vizinho); controle vira componente com role ligado a uma ação nomeada em <superfície>.props (binding required: registro vazio recusado dizendo o que falta); pixels contra o builder com cada diferença explicada (task-71 fase 1) |`

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

`cargo test -p parser --test pixels -- --ignored write_the_frames`
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
- `cargo run --offline -p showcase --example regress -- check`: 176
  baselines, 176 iguais.

### linha para o índice de ADR

`| 2026-09-25 | [importers-target-plev-ir](importers-target-plev-ir.md) | proposed | o parser ganha o segundo emissor, ParserNode para plev IR, ao lado do código builder (que não muda um byte); relatório em quatro categorias com arquivo:linha que estende a droplist, forma JSON fixa; tolerância por token medida no tema (metade da distância ao vizinho); controle vira componente com role ligado a uma ação nomeada em <superfície>.props (binding required); pixels contra o builder com cada diferença explicada (task-71 fase 1); fase 2: HTML/CSS subset (tree-sitter-html e -css) e Figma (export REST de um .fig por tool/tasks/figmapull.py) pelo mesmo Resolution e os mesmos emissores, regra de posição approximated, cor por variável no modo da tela |`

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

## fechamento (orquestrador, 2026-09-25)

fechada pelo critério de aceite, item por item:

- itens 1 e 2: fechados na fase 1 (as duas fixtures em IR verificado com
  relatório; pixels contra o builder com cada diferença explicada).
- item 3: duas fontes novas, como o brenner pediu ("html e figma"), cada
  uma com fixture, goldens e relatório em quatro categorias:
  - HTML/CSS: o note card escrito como página (`fixtures/html/`): 32
    supported, 22 transformed, 17 approximated, 13 impossible, 2
    bindings (`reply` e `open`, os mesmos do react).
  - Figma: a tela "settings" do `Artify App.fig` do brenner
    (`fixtures/figma/settings.json`, 98 nós): 251, 27, 119, 196, nenhum
    binding (o design não nomeia capacidade). o `.fig` fica fora do repo;
    `tool/tasks/figmapull.py` o lê só com a stdlib do python 3.14, e o
    orquestrador reproduziu a fixture byte a byte.
- item 4: gate abaixo.

validação do orquestrador: diff lido (o `emit.rs` só ganha o `Align` no
import quando uma fonte nova o usa; os goldens de react e gpui passam sem
regravar); o lock só com os dois pacotes aprovados; os PNGs abertos: o
note card em HTML desenha como o do react; no IR do Figma só aparece o
texto da tela, porque fundos fora da tolerância e ícones (vetores) vão ao
relatório como impossible, como o relatório diz.

pendência além das do handoff: o texto quebrado em coluna estreita
desenha por cima da linha de baixo ("Terms of Service" sobre o subtítulo
no IR do Figma); a medida é do `surface`.

o agente gastou uns 440 mil tokens.
