---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-09-25
domain: generative-runtime
task: task-97
wave: 8
depends: [task-68, task-69, task-96]
branch: task/97a-code-in-ir
---

# task-97: código e diff no vocabulário do IR

## objetivo

a demo 5 da task-80 (a revisão de três commits no ide, task-96) ficou
pior que o painel fixo no diff: a superfície gerada mostra o diff numa
tabela, cinco linhas por vez e sem cor, contra o diff inteiro do painel.
o motivo não é do ide: o vocabulário fechado do IR (`uir::Component`,
`rust/plevir/src/vocabulary.rs`) não tem um componente para código nem para
diff. esta task põe um, e a demo 5 é medida de novo pelo mesmo critério
escrito antes.

## ligação com a tese

seção 28 (a direção é reavaliada com os dados, não defendida: o dado
disse onde a superfície perde); a demo 5 da task-80.

## estado atual no código

- `comps::content::CodeBlock` (`rust/engine/src/widgets/content/code_block.rs`)
  desenha texto no passo mono da rampa, com legenda; não sabe o que é
  uma linha adicionada ou removida.
- o painel de diff do ide desenha o próprio diff
  (`apps/idestudio/src/interfaces/panels/diff.rs`).
- a revisão das regras (`rust/offers/src/review/mod.rs`) liga o diff a
  uma `table`.

## escopo

1. um componente `code` no vocabulário do IR, ligado a um estado de
   texto (ou lista de linhas) do app, com a legenda e, quando o app diz,
   o tipo de cada linha (contexto, adicionada, removida) desenhado pelos
   tokens do tema; o verificador e o schema do prompt o conhecem.
2. o materializador desenha pelo `CodeBlock` do `comps`, estendido de
   forma aditiva para o tipo de linha, com rolagem quando o diff é maior
   que a área.
3. as regras da revisão ligam o diff ao `code` em vez da `table`.
4. a demo 5 medida de novo pelo laboratório, sem janela, pelo mesmo
   critério da task-80; o resultado escrito lá, ao lado do primeiro.

## fora de escopo

realce de sintaxe por linguagem; edição; o urnaui.

## dependências

68, 69 e 96 integradas.

## superfície sob responsabilidade

`rust/plevir/src/**` (vocabulário, verificador, schema), `rust/screen`
(materializar e desenhar), `rust/engine/src/widgets/content/code_block.rs`
(aditivo), `rust/offers/src/review/mod.rs`, o laboratório do ide para a
demo, testes.

## critérios de aceitação

- o `code` verifica, materializa e desenha um diff com o tipo de cada
  linha pelos tokens, sem cor fora do tema.
- a demo 5 medida de novo, e o resultado escrito ao lado do primeiro,
  qualquer que seja.
- os baselines existentes iguais; `./tool/tasks/fullcheck.sh` verde.

## pare e reporte se

- o componente pedir mudar tipo do `sem` ou o protocolo v1;
- desenhar o tipo de linha pedir cor que o tema não tem (é decisão de
  design do brenner).

## sonda (2026-09-25, antes de qualquer código)

base: `main` @ `ec0f0dbb`, branch `task/97a-code-in-ir`. só leitura:
nenhum código mudou. toda afirmação sobre código tem arquivo e linha.

### (a) o vocabulário: como um componente entra, e o que muda

- um componente é uma variante de `uir::Component`
  (`rust/plevir/src/vocabulary.rs:50-93`), lida e escrita pelo serde dele
  (`rename_all = "snake_case"`, `:48-49`), com um `Spec` como dado
  (`:125-134`): o papel no contrato, se tem filhos, como é nomeado
  (`Named`, `:97-106`), o estado que mostra (`StateProp`, nome e tipos,
  `:110-113`) e os eventos que manda. `Component::ALL` (`:307-332`),
  `name` (`:335-359`) e `spec` (`:361-385`) são as três tabelas; `names`
  (`:417-433`) junta os nomes de estado e de evento de todos, na ordem de
  `ALL`, cada um uma vez; `role_of` (`:396-407`) só tem caso especial para
  o chip; `is_control` (`:411-413`) é todo papel interativo e a tab strip.
- o verificador não tem regra por componente: lê o `Spec`. a regra 3
  (`rust/plevir/src/verify/mod.rs:304-357`) recusa filhos, layout e repeat
  onde o `Spec` não é container; as regras 4 e 5
  (`rust/plevir/src/verify/bind.rs:35-111`) conferem cada estado ligado
  pelo nome e pelos tipos do `StateProp`, e cada evento pelo nome e pelo
  parâmetro; a regra 7 (`rust/plevir/src/verify/access.rs:91-139`) pede o
  rótulo pelo `Named` e põe na ordem de foco o que `is_control` diz. o
  leitor (`rust/plevir/src/decode.rs:128`) passa o `component` ao serde do
  tipo.
- o schema do prompt (`rust/plevir/src/schema.rs:46-170`) é construído das
  mesmas tabelas: o enum de componentes de `Component::ALL` (`:48-52`,
  `:101-106`) e os nomes que uma ligação pode usar de `names()` (`:47`,
  `:139-150`). `rust/plevir/assets/plev-ir-1.json` é esse valor escrito, e
  um teste mantém os dois iguais (`rust/plevir/tests/golden.rs:206-221`,
  `UIR_BLESS=1`). o prompt do `propose` lista cada componente de
  `Component::ALL` com o papel, o nome e o que mostra
  (`rust/offers/src/prompt.rs:125-179`) e descreve a regra 7 em texto
  (`:49-53`).
- os testes que fixam o vocabulário: `vocab.rs:439-481` (cada componente
  escreve como o nome, todo interativo é nomeado e manda algo, o papel do
  chip); `rust/plevir/tests/properties.rs` (a estratégia escolhe de
  `Component::ALL`, `:158`; o round trip, `:282`; o schema aceita, `:275`);
  `rust/plevir/tests/golden.rs` (os três IR de fixture verificam, leem e
  escrevem de volta iguais, o schema os aceita, `:88-174`);
  `rust/offers/tests/llm.rs:168-170` (o prompt nomeia cada componente);
  `rust/plevir/tests/identity.rs:388`, `:554` (o `SwapView` percorre
  `ALL`).
- o que muda, só por adição: `Component::Code` no fim do enum e no fim de
  `ALL` (24 para 25), o nome `code`, o `Spec` dele (papel `region`, sem
  filhos, `Named::Always`, estado `lines`, lista ou texto, e `kinds`,
  lista; nenhum evento), e `uir::LineKind`, os nomes do tipo de uma linha
  (`context`, `added`, `removed`, `header`), que o app escreve no estado e
  o prompt descreve. `names()` ganha `lines` e `kinds` no fim; o schema
  ganha `code` no fim do enum de componentes e os dois nomes no fim do
  enum de estado, e o arquivo do schema é escrito de novo. nenhuma linha
  do verificador muda: a regra de ligação de sempre decide o `code`. a
  transformação não ganha nada: `code` fica fora das famílias do
  `SwapView` (`rust/plevir/src/transform/view.rs:38-50`) e o `Restyle`
  recusa ele como recusa todo componente desenhado nas cores do tema
  (`rust/plevir/src/transform/look.rs:252-262`).
- um IR de antes continua válido byte a byte: todo nome que ele pode ter
  continua querendo dizer o mesmo, o leitor não muda, os enums do schema
  só crescem no fim. o teste novo lê cada `.ir.json` de
  `rust/plevir/tests/fixtures` e `rust/screen/tests/fixtures`, verifica contra
  o registro dele e escreve de volta os mesmos bytes.

### (b) o estado que o `code` liga

- o ide projeta o diff na região `diff`
  (`apps/idestudio/src/capability/project.rs:129-150`): `target` (o id cujo diff
  aparece), `lines` (as primeiras `DIFF_LINES` = 200, `:43`) e `count`.
  cada linha vai com a marca do diff unificado na frente: `+`, `-` ou
  espaço (`:135-142`); o cabeçalho do hunk vai como é, sem marca
  (`:137`).
- o tipo de cada linha está na vista (`DiffLine::kind`,
  `apps/idestudio/src/interfaces/panels/diff.rs:16-30`) e chega à projeção só como a
  primeira letra. o cabeçalho não tem marca, e num diff de commit com mais
  de um arquivo ele começa pelo caminho (`path: @@ ...`,
  `rust/gitdrv/src/diff.rs:140-156`): um arquivo cujo nome começa com `-`,
  `+` ou espaço dá um cabeçalho que se lê como uma linha removida,
  adicionada ou de contexto. quem lê o tipo pela letra adivinha.
- o mínimo aditivo: `diff:kinds`, uma lista do mesmo tamanho de
  `diff:lines`, com o nome do tipo de cada linha (`context`, `added`,
  `removed`, `header`), pelos nomes de `uir::LineKind`. nenhum tipo do
  `sem` muda (é um `Value::List`), nenhuma linha de `lines` muda. o que
  isso move: o golden `rust/plevir/tests/fixtures/idestudio-workbench.snapshot.json`
  ganha `diff:kinds` (lista vazia ali), escrito de novo por
  `UIR_BLESS=1 cargo test -p ide --test uir_fixture`; o recorte que o
  Claude vê (`apps/idestudio/src/generative/scope.rs:63-81`) leva `kinds` na
  mesma condição de `lines` (só enquanto o diff mostra um dos commits
  pedidos), senão um `code` que o Claude propuser não verifica; uma
  observação do ide com um diff aberto fica até 200 palavras curtas mais
  pesada.

### (c) o desenho

- o `CodeBlock` hoje (`rust/engine/src/widgets/content/code_block.rs:13-116`):
  `code` e `caption`; o fundo `glass.field` no raio `shape.nav` com a borda
  `glass.edge_soft` (`:66-91`); a legenda em `caption_sm` na cor
  `glass.text_faint` (`:94-106`); o código em `typography.mono()` na cor
  `colors.text_mid`, quebrado na largura de dentro (`:107-115`). não sabe
  o que é uma linha, não rola, não tem `sem`.
- a extensão, só por adição: um `LineKind` (contexto, adicionada,
  removida, cabeçalho) e as linhas com o tipo (`rows`) e o deslocamento
  (`scroll`) no bloco. com `rows` vazio o bloco desenha exatamente como
  hoje (teste: a mesma cena de antes). com `rows`, uma linha por caixa de
  linha do `mono`, cortada na largura com reticências
  (`TextMeasurer::truncate_to_width`, como o painel de diff faz), recortada
  à área das linhas e deslocada por `scroll`.
- os tokens de cada tipo, só os que o tema já tem: atrás da linha, a
  lavagem de intenção do design system, o matiz do token da intenção no
  alfa `glass.wash_alpha` (como o chip selecionado,
  `rust/engine/src/widgets/content/chip.rs:196-206`, e a barra de progresso,
  `rust/engine/src/widgets/feedback/progress.rs:63-68`): `colors.success` na
  adicionada, `colors.danger` na removida, `glass.surface_hover` no
  cabeçalho, nada no contexto. o texto de todas em `colors.text`.
- por que `colors.text` e não o `colors.text_mid` do bloco de hoje: medido
  como a task-75 mede (`rust/engine/src/styling/theme/tests/contrast.rs`, o
  translúcido composto sobre o fundo em luz linear, a luminância do
  WCAG), o `text_mid` sobre a lavagem verde fica em 3,64 no hoff, e já
  sobre o bloco sem lavagem fica em 4,13 no core, 3,39 no dark e 2,80 no
  hoff_light. o `colors.text` passa AA (4,5) em todo tipo, nos quatro
  temas da task-75 e no hoff_light, sobre `colors.bg` e `colors.surface`:
  o menor é 4,59, a linha adicionada no hoff sobre `colors.bg` (conta
  prévia em python; o teste do bloco mede de novo pelo código). nenhuma
  cor nova.
- a lavagem sozinha não passa 3:1 contra o bloco (1,06 a 1,67): o tipo
  não pode ser só cor (WCAG 1.4.1). no ide cada linha já traz a marca do
  diff no texto (`+`, `-`), então a marca carrega o tipo para quem não vê
  o matiz; o bloco não desenha marca própria para não dobrar a do app.
  dito no handoff.
- a rolagem: o `surface` rola uma lista hoje pela tabela, com a lista
  virtual do comps retida no `Widget::Table`
  (`rust/screen/src/widget.rs:411-413`, a roda; `:448`, o `wake`),
  recortada à altura quando `size.max` a limita
  (`rust/screen/src/draw.rs:359-382`, task-96). o `code` faz o mesmo
  com um widget retido novo, `Widget::Code`: um `ScrollState` do engine e
  um `Scrollbar` do comps, o par do painel de diff fixo
  (`apps/idestudio/src/interfaces/panels/diff.rs:39-40`, `:131-132`, `:221`); a roda
  sobre o retângulo dele desloca as linhas, o `size.max` limita a altura,
  e a posição da rolagem não é projetada (não invalida ato nenhum).

### (d) a revisão das regras

- hoje (`rust/offers/src/review/mod.rs:270-318`) toda lista de um item ou
  da região vira uma `table` ligada a `rows`; na região (a que as ações
  dos itens leem fora deles, o diff) a tabela é limitada por
  `dropdown_max_h` (`:295-306`) e vem por último (`:400-412`).
- o que muda: uma lista que o nó chama `lines` vira um `code`, limitado
  pelo mesmo token na região, e quando o nó tem também uma lista chamada
  `kinds` ela é ligada ao `kinds` do mesmo `code` e não aparece sozinha.
  são os nomes com que o `code` mostra os dois: o app que projeta `lines`
  e `kinds` fala o vocabulário. toda outra lista segue tabela (os arquivos
  de cada commit). os testes das regras
  (`rust/offers/src/review/tests.rs:243-259`) mudam para o `code`.

### (e) a demo 5 de novo

- o laboratório: `ide-lab --arm review`
  (`apps/agenteval/src/arms/review.rs`), sem janela, só as regras; o painel
  fixo primeiro (`:224-273`), a superfície depois (`:280-360`), fechar
  (`:364-388`). as linhas do diff à vista na superfície são contadas pela
  geometria da tabela (`:329-335`); passam a ser contadas pela geometria
  do `code` (a área das linhas sobre a altura de uma linha), e as do
  painel fixo continuam como são (`:236-240`).
- o critério, escrito antes na task-80 (seção "demo 5, UI generativa em
  aplicação real (task-96)"), não muda: atos e tempo para ver os arquivos
  e o diff dos três; o que fica à vista sem rolar; cada ato da superfície
  no dispatcher do workbench, com o receipt da pessoa; nada da superfície
  depois de fechar. o teste `apps/agenteval/tests/review.rs` roda o braço
  no gate. o resultado novo vai na task-80 ao lado do primeiro, qualquer
  que seja.
- o que a sonda já prevê, antes de rodar: o `code` limitado por
  `dropdown_max_h` (320 px no hoff) mostra umas 14 linhas de 19,6 px de
  uma vez, contra 5 da tabela; o painel fixo mostra o diff inteiro destes
  três commits (14, 7 e 18 linhas). então o terceiro diff deve seguir
  perdendo algumas linhas para o painel fixo, e o número de linha continua
  só no painel fixo. a medida decide.

## handoff (2026-09-25)

```text
Goal: task-97, código e diff no vocabulário do IR, e a demo 5 de novo
Status: pronto para o orquestrador, na branch task/97a-code-in-ir
```

### commits

- `a82087d8` task-97: the probe, and the task file in ongoing
- `38960bf8` task-97: the code block draws rows, each with its kind by
  the theme's tokens, and scrolls
- `82658d2a` task-97: code in the IR's vocabulary, verified by the usual
  rules, in the schema and the prompt, materialized on the code block
- `2b4fe23c` task-97: the ide's diff says the kind of each line, by name,
  beside the lines
- `593ff9bf` task-97: the review shows the diff in a code, the lines with
  the kind of each
- `10f66729` task-97: the laboratory takes half a pixel of the layout's
  rounding as no row cut (a gravação da demo nomeia este build)
- `9c96bde4` task-97: the demo 5 again, with the code in the IR, written
  in task-80 beside the first
- `19ea59bd` task-97: the code in the two ADRs (o gate e o `regress`
  rodaram aqui)
- o commit deste handoff (só este arquivo)

a ordem mudou um pouco da do briefing: o widget veio antes do
vocabulário. cada commit compila, e o `match` do `surface` sobre
`Component` é exaustivo: com o `code` no vocabulário e sem o widget, o
`surface` não compilava. é também a regra do `vocab.rs` ("a new
component needs the widget in comps first").

### a sonda

a seção "sonda" acima, (a) a (e), escrita antes do código. o que a
execução acrescentou a ela:

- o tipo de linha pede dois tipos novos no comps (`LineKind`,
  `CodeLine`), e o módulo `content::code_block` é privado: eles só saem
  pela linha de `pub use` em `rust/engine/src/widgets/content/mod.rs`. é a única
  linha fora de `code_block.rs` no comps, fora da lista de arquivos
  permitidos; nada do que o comps desenha mudou por ela.
- a revisão liga pelo nome: o nó que projeta uma lista `lines` e uma
  `kinds` fala o vocabulário do `code`. o `code` é ancorado pelo primeiro
  estado dele, como o `Namer` ancora o nó da superfície que já está
  aberta (`rust/offers/src/ids.rs`, `anchor`), senão uma revisão sobre
  a aberta trocava o id dele.
- o layout arredonda o bloco a pixel inteiro: um bloco da altura das
  linhas sai uma fração de pixel mais baixo, e a primeira contagem do
  laboratório deu 13 de 14 e 6 de 7. o laboratório passou a tomar meio
  pixel como o arredondamento, como o `whole` já toma para um controle.

### o `code`, lugar por lugar

- `uir` (`rust/plevir/src/vocabulary.rs`): `Component::Code` no fim do enum e
  de `ALL` (25), nome `code`, `Spec` com papel `region`, sem filhos,
  `Named::Always`, estado `lines` (lista ou texto) e `kinds` (lista),
  nenhum evento; `LineKind` (`context`, `added`, `removed`, `header`) com
  `ALL`, `as_str` e `parse`, exportado em `lib.rs`. o verificador não
  mudou uma linha. o schema cresceu só no fim dos enums (`code`, `lines`,
  `kinds`) e o arquivo foi escrito de novo.
- `propose` (`rust/offers/src/prompt.rs`): o prompt diz o que o
  `code` mostra, os nomes dos tipos, que ele rola quando o `size` o
  limita e que o rótulo é a legenda; a regra 7 no texto inclui o `code`.
- `comps` (`rust/engine/src/widgets/content/code_block.rs`): `rows`, `scroll`,
  `CodeBlock::lines`, `row_h`, `content_h`, `rows_rect`, `max_scroll`; sem
  linhas o bloco desenha como antes (teste da cena).
- `surface` (`rust/screen/src/code.rs`, novo; `widget.rs`, `draw.rs`,
  `token.rs`, `lib.rs`): o widget retido `Widget::Code` (o bloco, um
  `ScrollState` e o `Scrollbar`), a altura das linhas como folha, o
  nome do tipo para o tipo do comps num `match` exaustivo em `token.rs`.
  o `project` não mudou: o nó sai como `region` com o rótulo, `lines` e
  `kinds`.
- testes: `rust/plevir/tests/code.rs` (verifica, round trip, o schema
  aceita, as regras de ligação recusam o que devem, e os oito IR de antes
  leem e escrevem de volta o mesmo documento, os três do `uir` byte a
  byte); o teste do `Spec` em `vocab.rs`; o do prompt em `prompt.rs`.

### o estado do diff

`diff:kinds` ao lado de `diff:lines`
(`apps/idestudio/src/capability/project.rs`): o mesmo tamanho, um nome de
`uir::LineKind` por linha, do `DiffLineKind` do painel. nenhum tipo do
`sem` mudou; nenhuma linha de `diff:lines` mudou. o golden
`rust/plevir/tests/fixtures/idestudio-workbench.snapshot.json` ganhou só o `kinds`
vazio. o recorte do Claude (`apps/idestudio/src/generative/scope.rs`) leva
`kinds` na mesma condição de `lines` (teste em `apps/idestudio/tests/claude.rs`,
que continua passando e agora confere os dois). teste novo:
`apps/idestudio/tests/review.rs`, `the_diff_says_the_kind_of_each_line_beside_it`.

### o desenho e o contraste de cada tipo de linha

atrás da linha, a lavagem de intenção do design system: `colors.success`
no alfa `glass.wash_alpha` na adicionada, `colors.danger` no mesmo alfa na
removida, `glass.surface_hover` no cabeçalho, nada no contexto. o texto de
todas em `colors.text`. nenhuma cor fora do tema (teste: toda cor da cena
é um desses tokens). o contraste do texto sobre a linha, medido como a
task-75 mede (`every_kind_reads_at_aa_over_its_row`):

| tema, sobre | contexto | adicionada | removida | cabeçalho |
| --- | --- | --- | --- | --- |
| core, `colors.bg` e `colors.surface` | 18,26 | 17,19 | 17,03 | 17,52 |
| hoff, `colors.bg` | 7,56 | 4,59 | 7,12 | 5,63 |
| hoff, `colors.surface` | 7,69 | 4,63 | 7,22 | 5,70 |
| dark, `colors.bg` | 9,11 | 5,48 | 6,65 | 6,48 |
| dark, `colors.surface` | 8,82 | 5,39 | 6,51 | 6,34 |
| light, `colors.bg` | 15,92 | 14,46 | 14,27 | 15,17 |
| light, `colors.surface` | 16,60 | 15,05 | 14,85 | 15,82 |
| hoff_light, `colors.bg` | 7,82 | 7,55 | 7,33 | 7,60 |
| hoff_light, `colors.surface` | 7,97 | 7,68 | 7,46 | 7,74 |

todos passam AA (4,5); o menor é 4,59. a rolagem: a roda sobre o bloco
limitado desloca as linhas, para na última, fora dele não é dele, a
posição não é projetada e linhas novas voltam ao topo
(`rust/screen/tests/code.rs`); o bloco recorta as linhas à área dele e
desenha só as que aparecem (`rows_scroll_inside_the_area_and_stop_at_the_last`).

### a revisão

`rust/offers/src/review/mod.rs`: uma lista que o nó chama `lines` vira um
`code` (limitado por `dropdown_max_h` na região); com uma lista `kinds` no
mesmo nó ela vai no mesmo `code` e não aparece sozinha; toda outra lista
segue tabela. testes em `rust/offers/src/review/tests.rs` (o da revisão
de três commits e um novo, `only_the_lines_are_a_code_and_the_kinds_go_with_them`)
e em `apps/idestudio/tests/review_acts.rs` (o `code` mostra as linhas e os
tipos que o dispatcher pediu).

### a demo 5 de novo, ao lado da primeira

na task-80, seção "de novo, com o `code` no IR (task-97)", gravada sem
janela no build `10f66729`, lida pelo `plev-inspect`. linhas do diff à
vista sem rolar: 14 de 14, 7 de 7 e 14 de 18 na superfície (eram 5, 5 e
5 na tabela), contra 14, 7 e 18 no painel fixo; com a cor do tipo de cada
linha. arquivos dos três de uma vez: 3 contra no máximo 1. atos: 5
cliques e 26 teclas contra 3 cliques. pelo critério escrito antes, a demo
continua caindo no diff, agora por 4 linhas do terceiro commit e pelo
número de linha, e continua de pé nos arquivos; nada fora do dispatcher e
nada depois de fechar.

### validação

com o ambiente do briefing em todo comando cargo (`CARGO_TARGET_DIR` em
`target/agents/one`, `CARGO_INCREMENTAL=0`, debug `line-tables-only`, o
`PLEV_CLAUDE` no stub) e `--offline`:

- `./tool/tasks/fullcheck.sh` em `19ea59bd`: verde (exit 0, 05:13 a 05:19), com a
  perna urnaui e o check wasm com `-D warnings`. workspace 2969 passam, 0
  falham, 26 ignorados (164 suítes); urnaui 85 passam, 1 ignorado (6
  suítes). total 3054 e 27, contra 3036 e 27 na base: os 18 testes novos.
  `df -h /`: 31 GB livres antes, 28 GB depois.
- `cargo run --offline -p showcase --example regress -- check`, com o
  `main.rs` tocado: 176 baselines, 176 iguais, nenhum frame desenhado,
  nenhuma GPU aberta.
- `cargo test -p ide --test lab_review` cinco vezes seguidas e `--test
  claude` três: todas verdes.
- nenhuma janela aberta, nenhum clique ou tecla do sistema (os do
  laboratório são pelo `Workbench::click`), nenhuma chamada ao `claude -p`,
  nenhuma dependência nova, o `Cargo.lock` igual.

### linhas para o índice de ADRs

- `plev-ir-typed-and-verified.md`: adendo de 2026-09-25, o componente
  `code` (task-97): o `Spec`, os nomes dos tipos de linha, o verificador e
  o schema iguais a não ser pelo fim.
- `materialized-surfaces-from-verified-ir.md`: adendo de 2026-09-25, um
  `code` desenha as linhas pelo tipo e rola (task-97): os tokens, o
  contraste medido, a marca no texto, o widget retido que rola.

### o que fica aberto

- a demo 5 ainda perde no diff: as 4 linhas do terceiro commit (o único
  token de altura do IR, `dropdown_max_h`; fechar pede um token de altura
  para um bloco de leitura, decisão de design do brenner, ou um layout em
  que um nó toma o que sobra da superfície, decisão do orquestrador) e o
  número de linha (a projeção não leva os números antigo e novo).
- três temas com nome fora dos quatro da task-75 ficam abaixo de AA numa
  linha adicionada (tokyo-night 4,30 a 4,38, gruvbox 3,79 a 3,94,
  one-dark 3,32 a 3,40; no one-dark também a removida, 4,00 a 4,12, e o
  cabeçalho sobre um card, 4,43): o `colors.text` deles é mais baixo.
  cor de tema é decisão de design.
- a lavagem sozinha fica de 1,06 a 1,67 contra o bloco: o tipo vem também
  da marca no texto, que o ide projeta; um app sem marca no texto teria o
  tipo só pela cor.
- a legenda do bloco segue em `glass.text_faint`, como sempre desenhou
  (3,76 no hoff sobre o fundo); não mudou porque o que o bloco desenha
  hoje não muda.
- a página do bloco no showcase não mostra as linhas (o showcase ficou de
  fora; nenhum baseline mudou).
- o `code` não entra numa família do `SwapView`.
- a observação do ide pesa até 200 nomes curtos a mais quando um diff está
  aberto.
- a linha de `pub use` em `rust/engine/src/widgets/content/mod.rs`, fora da lista
  de arquivos, pelo motivo acima.
- com a superfície aberta, o painel fixo mais estreito mostra o mesmo diff
  à esquerda: a pessoa vê o diff duas vezes.
- a prova com o Claude de verdade e a demo 5 com janela pelo brenner
  seguem com o orquestrador e o brenner.

## fechamento (2026-09-25, orquestrador)

integrada. aceito o `pub use` em `rust/engine/src/widgets/content/mod.rs`, fora
da lista do briefing: é a única forma de os tipos novos do `CodeBlock`
saírem do módulo privado, e nada que o comps desenha mudou. conferido nas
imagens da demo 5: o diff colorido pelo tipo de linha, 14, 7 e 14 linhas
à vista. dois defeitos visuais vistos na validação, que ficam como dívida
(P2): o cartão do diff tem como título o id cru do commit
(`stacks.commit[847f583…`) em vez da mensagem; e a coluna da superfície
gerada ocupa uns 370 px do ide o tempo todo, mesmo sem superfície aberta,
e aperta as vistas fixas.
