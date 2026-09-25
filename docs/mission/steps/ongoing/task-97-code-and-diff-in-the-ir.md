---
project: plev
audience: [ai-agents, contributors]
status: ongoing
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
`crates/uir/src/vocab.rs`) não tem um componente para código nem para
diff. esta task põe um, e a demo 5 é medida de novo pelo mesmo critério
escrito antes.

## ligação com a tese

seção 28 (a direção é reavaliada com os dados, não defendida: o dado
disse onde a superfície perde); a demo 5 da task-80.

## estado atual no código

- `comps::content::CodeBlock` (`crates/comps/src/content/code_block.rs`)
  desenha texto no passo mono da rampa, com legenda; não sabe o que é
  uma linha adicionada ou removida.
- o painel de diff do ide desenha o próprio diff
  (`crates/ide/src/views/diff_view.rs`).
- a revisão das regras (`crates/propose/src/review.rs`) liga o diff a
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

`crates/uir/src/**` (vocabulário, verificador, schema), `crates/surface`
(materializar e desenhar), `crates/comps/src/content/code_block.rs`
(aditivo), `crates/propose/src/review.rs`, o laboratório do ide para a
demo, testes.

## critérios de aceitação

- o `code` verifica, materializa e desenha um diff com o tipo de cada
  linha pelos tokens, sem cor fora do tema.
- a demo 5 medida de novo, e o resultado escrito ao lado do primeiro,
  qualquer que seja.
- os baselines existentes iguais; `./script/gate` verde.

## pare e reporte se

- o componente pedir mudar tipo do `sem` ou o protocolo v1;
- desenhar o tipo de linha pedir cor que o tema não tem (é decisão de
  design do brenner).

## sonda (2026-09-25, antes de qualquer código)

base: `main` @ `ec0f0dbb`, branch `task/97a-code-in-ir`. só leitura:
nenhum código mudou. toda afirmação sobre código tem arquivo e linha.

### (a) o vocabulário: como um componente entra, e o que muda

- um componente é uma variante de `uir::Component`
  (`crates/uir/src/vocab.rs:50-93`), lida e escrita pelo serde dele
  (`rename_all = "snake_case"`, `:48-49`), com um `Spec` como dado
  (`:125-134`): o papel no contrato, se tem filhos, como é nomeado
  (`Named`, `:97-106`), o estado que mostra (`StateProp`, nome e tipos,
  `:110-113`) e os eventos que manda. `Component::ALL` (`:307-332`),
  `name` (`:335-359`) e `spec` (`:361-385`) são as três tabelas; `names`
  (`:417-433`) junta os nomes de estado e de evento de todos, na ordem de
  `ALL`, cada um uma vez; `role_of` (`:396-407`) só tem caso especial para
  o chip; `is_control` (`:411-413`) é todo papel interativo e a tab strip.
- o verificador não tem regra por componente: lê o `Spec`. a regra 3
  (`crates/uir/src/verify/mod.rs:304-357`) recusa filhos, layout e repeat
  onde o `Spec` não é container; as regras 4 e 5
  (`crates/uir/src/verify/bind.rs:35-111`) conferem cada estado ligado
  pelo nome e pelos tipos do `StateProp`, e cada evento pelo nome e pelo
  parâmetro; a regra 7 (`crates/uir/src/verify/access.rs:91-139`) pede o
  rótulo pelo `Named` e põe na ordem de foco o que `is_control` diz. o
  leitor (`crates/uir/src/decode.rs:128`) passa o `component` ao serde do
  tipo.
- o schema do prompt (`crates/uir/src/schema.rs:46-170`) é construído das
  mesmas tabelas: o enum de componentes de `Component::ALL` (`:48-52`,
  `:101-106`) e os nomes que uma ligação pode usar de `names()` (`:47`,
  `:139-150`). `crates/uir/schema/plev-ir-1.json` é esse valor escrito, e
  um teste mantém os dois iguais (`crates/uir/tests/fixtures.rs:206-221`,
  `UIR_BLESS=1`). o prompt do `propose` lista cada componente de
  `Component::ALL` com o papel, o nome e o que mostra
  (`crates/propose/src/prompt.rs:125-179`) e descreve a regra 7 em texto
  (`:49-53`).
- os testes que fixam o vocabulário: `vocab.rs:439-481` (cada componente
  escreve como o nome, todo interativo é nomeado e manda algo, o papel do
  chip); `crates/uir/tests/prop.rs` (a estratégia escolhe de
  `Component::ALL`, `:158`; o round trip, `:282`; o schema aceita, `:275`);
  `crates/uir/tests/fixtures.rs` (os três IR de fixture verificam, leem e
  escrevem de volta iguais, o schema os aceita, `:88-174`);
  `crates/propose/tests/llm.rs:168-170` (o prompt nomeia cada componente);
  `crates/uir/tests/identity.rs:388`, `:554` (o `SwapView` percorre
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
  `SwapView` (`crates/uir/src/transform/view.rs:38-50`) e o `Restyle`
  recusa ele como recusa todo componente desenhado nas cores do tema
  (`crates/uir/src/transform/look.rs:252-262`).
- um IR de antes continua válido byte a byte: todo nome que ele pode ter
  continua querendo dizer o mesmo, o leitor não muda, os enums do schema
  só crescem no fim. o teste novo lê cada `.ir.json` de
  `crates/uir/fixtures` e `crates/surface/tests/fixtures`, verifica contra
  o registro dele e escreve de volta os mesmos bytes.

### (b) o estado que o `code` liga

- o ide projeta o diff na região `diff`
  (`crates/ide/src/workbench/project.rs:129-150`): `target` (o id cujo diff
  aparece), `lines` (as primeiras `DIFF_LINES` = 200, `:43`) e `count`.
  cada linha vai com a marca do diff unificado na frente: `+`, `-` ou
  espaço (`:135-142`); o cabeçalho do hunk vai como é, sem marca
  (`:137`).
- o tipo de cada linha está na vista (`DiffLine::kind`,
  `crates/ide/src/views/diff_view.rs:16-30`) e chega à projeção só como a
  primeira letra. o cabeçalho não tem marca, e num diff de commit com mais
  de um arquivo ele começa pelo caminho (`path: @@ ...`,
  `crates/git/src/diff.rs:140-156`): um arquivo cujo nome começa com `-`,
  `+` ou espaço dá um cabeçalho que se lê como uma linha removida,
  adicionada ou de contexto. quem lê o tipo pela letra adivinha.
- o mínimo aditivo: `diff:kinds`, uma lista do mesmo tamanho de
  `diff:lines`, com o nome do tipo de cada linha (`context`, `added`,
  `removed`, `header`), pelos nomes de `uir::LineKind`. nenhum tipo do
  `sem` muda (é um `Value::List`), nenhuma linha de `lines` muda. o que
  isso move: o golden `crates/uir/fixtures/ide-workbench.snapshot.json`
  ganha `diff:kinds` (lista vazia ali), escrito de novo por
  `UIR_BLESS=1 cargo test -p ide --test uir_fixture`; o recorte que o
  Claude vê (`crates/ide/src/generated/scope.rs:63-81`) leva `kinds` na
  mesma condição de `lines` (só enquanto o diff mostra um dos commits
  pedidos), senão um `code` que o Claude propuser não verifica; uma
  observação do ide com um diff aberto fica até 200 palavras curtas mais
  pesada.

### (c) o desenho

- o `CodeBlock` hoje (`crates/comps/src/content/code_block.rs:13-116`):
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
  `crates/comps/src/action/chip.rs:196-206`, e a barra de progresso,
  `crates/comps/src/feedback/progress.rs:63-68`): `colors.success` na
  adicionada, `colors.danger` na removida, `glass.surface_hover` no
  cabeçalho, nada no contexto. o texto de todas em `colors.text`.
- por que `colors.text` e não o `colors.text_mid` do bloco de hoje: medido
  como a task-75 mede (`crates/engine/src/theme/tests_contrast.rs`, o
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
  (`crates/surface/src/widget.rs:411-413`, a roda; `:448`, o `wake`),
  recortada à altura quando `size.max` a limita
  (`crates/surface/src/draw.rs:359-382`, task-96). o `code` faz o mesmo
  com um widget retido novo, `Widget::Code`: um `ScrollState` do engine e
  um `Scrollbar` do comps, o par do painel de diff fixo
  (`crates/ide/src/views/diff_view.rs:39-40`, `:131-132`, `:221`); a roda
  sobre o retângulo dele desloca as linhas, o `size.max` limita a altura,
  e a posição da rolagem não é projetada (não invalida ato nenhum).

### (d) a revisão das regras

- hoje (`crates/propose/src/review.rs:270-318`) toda lista de um item ou
  da região vira uma `table` ligada a `rows`; na região (a que as ações
  dos itens leem fora deles, o diff) a tabela é limitada por
  `dropdown_max_h` (`:295-306`) e vem por último (`:400-412`).
- o que muda: uma lista que o nó chama `lines` vira um `code`, limitado
  pelo mesmo token na região, e quando o nó tem também uma lista chamada
  `kinds` ela é ligada ao `kinds` do mesmo `code` e não aparece sozinha.
  são os nomes com que o `code` mostra os dois: o app que projeta `lines`
  e `kinds` fala o vocabulário. toda outra lista segue tabela (os arquivos
  de cada commit). os testes das regras
  (`crates/propose/src/review/tests.rs:243-259`) mudam para o `code`.

### (e) a demo 5 de novo

- o laboratório: `ide-lab --arm review`
  (`crates/ide/src/bin/lab/review.rs`), sem janela, só as regras; o painel
  fixo primeiro (`:224-273`), a superfície depois (`:280-360`), fechar
  (`:364-388`). as linhas do diff à vista na superfície são contadas pela
  geometria da tabela (`:329-335`); passam a ser contadas pela geometria
  do `code` (a área das linhas sobre a altura de uma linha), e as do
  painel fixo continuam como são (`:236-240`).
- o critério, escrito antes na task-80 (seção "demo 5, UI generativa em
  aplicação real (task-96)"), não muda: atos e tempo para ver os arquivos
  e o diff dos três; o que fica à vista sem rolar; cada ato da superfície
  no dispatcher do workbench, com o receipt da pessoa; nada da superfície
  depois de fechar. o teste `crates/ide/tests/lab_review.rs` roda o braço
  no gate. o resultado novo vai na task-80 ao lado do primeiro, qualquer
  que seja.
- o que a sonda já prevê, antes de rodar: o `code` limitado por
  `dropdown_max_h` (320 px no hoff) mostra umas 14 linhas de 19,6 px de
  uma vez, contra 5 da tabela; o painel fixo mostra o diff inteiro destes
  três commits (14, 7 e 18 linhas). então o terceiro diff deve seguir
  perdendo algumas linhas para o painel fixo, e o número de linha continua
  só no painel fixo. a medida decide.
