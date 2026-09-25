---
project: plev
audience: [ai-agents, contributors]
status: ongoing
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

- `crates/parser`: parse (tree-sitter: `tsx.rs`, `sass.rs`, `gpui.rs`),
  resolve (`resolve_react.rs` via `css_map.rs`, `resolve_gpui.rs`) para o
  `ParserNode` IR, emit (`emit.rs`) de código Rust contra
  `engine::builder`. o droplist reporta com arquivo:linha tudo que não
  passa (`crates/parser/src/lib.rs`, ADR
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

`crates/parser/**`, proposta de ADR `docs/ADR/importers-target-plev-ir.md`.
dependência nova (grammar HTML, cliente do export Figma) só com
justificativa aprovada.

## interfaces que consome

o IR e o verifier (68), host de superfícies (69), headless (66).

## interfaces que pode alterar

`crates/parser` inteiro; o formato do relatório é contrato com quem o lê
(documentado).

## critérios de aceitação

- as duas fixtures em IR verificado, com relatório em quatro categorias.
- comparação de pixels entre o IR materializado e o builder emitido, com a
  diferença explicada quando houver.
- uma fonte nova (HTML/CSS subset ou Figma) com fixture e relatório.
- `./script/gate` verde.

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

o `ParserNode` (`crates/parser/src/ir.rs:7-13`) tem `tag`, `props` e
`children`; `Tag` é `Div`, `Text` e `Slot` (`ir.rs:15-22`); os props já são
nomes de método do builder com argumentos literais ou expressões de token
(`ir.rs:33-62`). o IR pede identidade (`IrNode.sem`,
`crates/uir/src/ir.rs:62-83`), um componente de vocabulário fechado
(`crates/uir/src/vocab.rs:50-93`), props só por token
(`crates/uir/src/token.rs:174-192`), layout por primitivas
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
forma aditiva dentro do `crates/parser`: um `origin` no `ParserNode`, a
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
capacidade (`crates/uir/src/ir.rs:104-110`). com o registro vazio o
verificador recusa pela regra 5 ou 4 dizendo exatamente o que falta ("the
app declares no action `open` on `note_card.props`",
`crates/uir/src/verify/bind.rs:176` e `:139`); o relatório lista as
mesmas referências em `bindings`. o teste declara o registro, como o app
faria, e o mesmo IR passa. deixar o controle sem ligação não serve: a
regra 7 recusa controle só visual (`crates/uir/src/verify/access.rs:120-127`)
sem dizer qual capacidade falta.

### (f) pixels

os dois lados no mesmo `engine::headless::Renderer`
(`crates/engine/src/headless/mod.rs:138-270`), mesmo tamanho, escala 1,
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
