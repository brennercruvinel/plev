---
id: task-96/phase-b-title-and-column
title: "task-96, phase b: the diff card title and the column shown only with a surface"
project: plev
type: section
parent: task-96
status: reference
date: 2026-09-25
date_basis: text
recorded: 2026-09-25
topics: [generated-surfaces, layout, git]
relations: [task-80, task-97]
evidence: [e71222cf, 3959d958, 4be4bb2f, e42bd571, a427d2b8]
source: docs/log/steps/task-96-ide-generated-surface.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase b (2026-09-25): the diff card title and the column shown only with a surface

Phase b fixes two defects that the task-97 validation saw in the images of demo 5 and that the tests did not catch: the diff card showed the commit's raw id, and the intent-line column took 368 px of the window even with no surface open. This phase's probe measured the before state with the lab, and the result was the line in a strip below the fixed views, with the column present only while a surface is open. The card now says "diff", and demo 5, recorded again, does not change any number in the criterion.

a validação da task-97 abriu as imagens da demo 5 e viu dois defeitos
que os testes não pegam, porque contam linhas e atos: o cartão do diff,
à direita da revisão gerada, tem como título o id cru do commit
(`stacks.commit[847f583aab69146fe62a5936…`); e sem superfície aberta a
coluna da linha de intenção ocupa 368 px da janela e aperta as vistas
fixas. branch `task/96b-ide-surface-polish`, base `main` @ `8704ba3a`.

## sonda (antes de qualquer código)

só leitura, e uma corrida do laboratório na base para as imagens de
antes (`ide-lab --arm review --out <scratchpad>/demo5b/before`, sem
janela, os mesmos números da task-97: 14, 7 e 18 linhas no painel fixo;
14, 7 e 14 na superfície). toda afirmação sobre código tem arquivo e
linha.

### (a) de onde vem o título do cartão do diff

- não é o rótulo do nó nem o materializador: é um texto da região que as
  regras põem primeiro e no tipo de título. a revisão faz da região que
  as ações dos itens leem fora deles um cartão
  (`rust/offers/src/review/mod.rs:458-479`, `regions`, e `:433-445`,
  `region`): os textos dela primeiro, depois os números, as listas por
  último. o primeiro texto ganha `Base2sm`, o tipo do que nomeia
  (`review.rs:304-312`). a região é `diff`, e o primeiro estado dela é
  `target`, o id do nó cujo diff aparece
  (`apps/idestudio/src/capability/project.rs:152-157`): num commit,
  `stacks.commit[<sha>]`. é o estado que o `show_diff` de cada commit lê
  (`project.rs:204`, `:231`), a palavra do dispatcher para "qual item",
  não a de uma pessoa.
- o rótulo do cartão é o da região, `"diff"` (`review.rs:440`), mas um
  `card` não desenha o rótulo (`rust/screen/src/draw.rs:291-297`, só o
  painel); um `text` desenha o valor ligado, ou o rótulo quando não liga
  nada (`draw.rs:71-76`, `:317-329`).
- a palavra certa que a projeção tem, sem inventar estado: a mensagem do
  commit que o diff mostra existe, como o rótulo e o `summary` do nó que
  `target` nomeia (`project.rs:217-227`), mas num outro nó. o IR liga um
  estado por uma referência fixa (`uir::Bind`,
  `rust/plevir/src/ir.rs:112`), e nenhum componente segue um valor até o
  rótulo de outro nó (`text` mostra o `text` ligado,
  `rust/plevir/src/vocabulary.rs:332`). um texto pode mostrar "o summary de
  `stacks.commit[847f…]`", não "o summary do commit que `diff:target`
  nomeia agora". a mensagem que muda com o diff pediria um estado novo na
  região (a projeção muda, e o golden
  `rust/plevir/tests/fixtures/idestudio-workbench.snapshot.json` com ela) ou o
  `surface` seguindo a referência: os dois fora desta fase. escrever na
  proposta a mensagem que `target` nomeia na hora também não serve: o
  diff segue o `target` e o rótulo fixo não, e depois do próximo "show
  diff" o título mentiria. o painel fixo mostra a mensagem no cabeçalho
  (`apps/idestudio/src/interfaces/panels/diff.rs:84-92`), que não é projetado.
- então a palavra é o rótulo da região, `"diff"` (`project.rs:153`), o
  que o cartão mostra. o que muda, só em `review.rs`: um texto da região
  que as ações dos itens leem, cujo valor é o id de um nó do app (ou
  vazio), é o ponteiro do dispatcher e não aparece; uma região sem texto
  próprio abre com o rótulo dela num texto no tipo de título, primeiro no
  cartão. esse texto não liga nada, então o `Namer` não o guarda entre
  duas propostas (`rust/offers/src/ids.rs:98-113`: guarda os nós
  ligados e os containers com filhos); a revisão procura na superfície
  aberta o texto sem ligação que abre o cartão da mesma região e mantém o
  id dele. o ato da pessoa não muda: o request é observado no snapshot
  do app (`rust/screen/src/surface.rs:386`), e um estado que a
  superfície não mostra é lido como o app o guarda
  (`rust/screen/src/project.rs:26-28`).

### (b) onde a coluna nasce e o desenho mínimo

- a coluna nasce ligada ao bloco: `host_surfaces_over`
  (`apps/idestudio/src/generative/mod.rs:59-64`) e o `lay_out`
  (`:103-112`) dão ao workspace a janela menos `column_w`
  (`apps/idestudio/src/generative/row.rs:61-70`): a largura de um card
  (`size.card_w`, 368 px no hoff, `rust/engine/src/styling/theme/scales.rs:291`)
  sem superfície, metade da janela com uma. `Workbench::resize`
  (`generated.rs:93-99`) refaz o mesmo a cada tamanho. a coluna vai de
  cima a baixo, a linha em cima e a superfície embaixo (`row.rs:73-82`).
  a janela liga o bloco sempre (`apps/idestudio/src/main.rs:565`).
- o que isso custa sem superfície: o workspace tem 912 de 1280 px. o
  `apply_panel_widths` (`apps/idestudio/src/interfaces/panels/layout/mod.rs:380-392`)
  guarda as larguras de changes e de diff e dá o resto a stacks, então é
  a coluna de stacks que perde os 368 px, e a mensagem de cada commit sai
  cortada (`apps/idestudio/src/interfaces/panels/stacks.rs:272-276`). na
  imagem de antes `demo5b/before/fixed.png`: "readme: how…", "login
  rememb…", "session timeo…".
- o desenho mínimo: a linha numa faixa ao longo de baixo da janela, na
  largura inteira, e a coluna só com superfície aberta. sem superfície o
  workspace recebe a largura inteira da janela, a de antes da task-96, e
  só a altura da faixa a menos; com superfície, a coluna à direita, em
  cima da faixa, metade da janela, como hoje. na faixa: o campo (um card
  de largura, no máximo), o proponente, propor e fechar numa linha, e a
  mensagem ao lado deles enquanto a largura dá, embaixo deles quando não
  dá. tudo pelo `resize` do workspace; nenhum arquivo de
  `apps/idestudio/src/interfaces/panels/**` muda.
- por que embaixo e não em cima: o header do workspace desenha em y = 0
  (`apps/idestudio/src/interfaces/panels/layout/render.rs:28-40`), e deslocar a origem
  é o `set_frame` (`views/workspace/mod.rs:252-257`), que liga o modo
  embutido e tira o header: mudaria uma vista fixa. dentro do header
  também não: ele centra o repositório e a branch
  (`apps/idestudio/src/interfaces/panels/header.rs:91-100`), e a linha passaria por cima.
- o que a faixa embaixo pede: o proponente é o `Select` do comps, que
  abre o painel embaixo do retângulo que recebe
  (`rust/engine/src/widgets/form/select/panel.rs:73-86`), e embaixo da faixa
  não há janela. o comps está fora desta fase; a linha dá ao `Select`,
  para o painel, o retângulo do controle levantado pela altura do painel
  e pelas duas folgas (o `dropdown_rect` dá as duas), então o painel abre
  para cima, por cima do workspace, com o controle desenhado e clicado
  onde está. um `Select` que abre para cima quando falta espaço embaixo
  é o jeito limpo, no comps (fica como P3).
- o que não muda: os mesmos widgets, os mesmos ids, a mesma projeção
  (`apps/idestudio/src/generative/mod.rs:255-313`); só os retângulos se movem.
  a pessoa chega à linha pelo clique no campo, um agente pela porta
  (`set_value` e `propose` em `ide_surface.*`, `Workbench::dispatch`),
  o teclado como hoje (`apps/idestudio/src/interfaces/window/keyboard.rs:70-95`: com o campo
  focado, as teclas são dele, Enter propõe, Escape solta).
- o preço: a altura da faixa, uns 60 px de 800, tirada das vistas fixas,
  que rolam na vertical; o diff mais longo da demo (18 linhas) cabe
  inteiro nas duas alturas. as imagens de antes e depois de cada caso
  (sem superfície, com superfície) ficam em `demo5b/`, citadas no
  handoff desta fase.

## handoff da fase b (2026-09-25)

```text
Goal: task-96 fase b, o título do cartão do diff e a coluna só com superfície
Status: pronto para o orquestrador, na branch task/96b-ide-surface-polish
```

### commits

- `e71222cf` task-96: the probe of phase b, the diff card's title and the
  column only with a surface
- `3959d958` task-96: the diff card says what it shows, "diff", never the
  id of the commit it shows
- `4be4bb2f` task-96: the line in a strip under the fixed views, and a
  column only while a surface is open
- `e42bd571` task-96: the laboratory says how many of the three commits'
  messages the stacks panel cuts (a gravação da demo nomeia este build)
- `a427d2b8` task-96: the line's test finds each repository's own commit
  card (o primeiro gate pegou: dois repositórios, dois shas; o teste só
  passava quando os dois commits caíam no mesmo segundo)
- o commit deste handoff (esta seção e a demo 5 na task-80)

um commit a mais que os quatro do briefing: a medida nova do laboratório
foi para um commit próprio antes da gravação, para a gravação nomear um
build com ela.

### a sonda

a seção "sonda" acima, (a) e (b), escrita antes do código.

### o título

- `rust/offers/src/review/mod.rs`: um texto da região que as ações dos
  itens leem, cujo valor é o id de um nó do app ou vazio (`diff:target`),
  não aparece; uma região sem texto próprio abre com o rótulo dela
  (`"diff"`), no tipo de título (`Base2sm`), sem ligar nada. o `Namer`
  não guarda um nó sem ligação e sem filhos entre duas propostas, então a
  revisão procura na superfície aberta o texto sem ligação que abre o
  cartão da mesma região e mantém o id (`proposed.diff.heading`); um nó
  assim o `Namer` nunca entrega, então o id não sai duas vezes.
- por que "diff" e não a mensagem do commit: a sonda (a). a mensagem
  pediria um estado novo na região ou o `surface` seguindo a referência.
- o botão "show diff" de cada commit na superfície passou a ler
  `diff:target` como o app o guarda, porque a superfície não mostra mais
  esse estado (`rust/screen/src/project.rs:26-28`); o stale é o mesmo
  (`apps/idestudio/tests/review_acts.rs` passa sem mudança).
- testes: `rust/offers/src/review/tests.rs`
  (`the_region_says_what_it_shows_never_the_id_of_the_commit_it_shows`,
  novo; o da revisão de três commits confere o título e que nada liga
  `diff:target`; o de propor de novo confere que o id do título fica em
  duas propostas seguidas) e `apps/idestudio/tests/review.rs`
  (`the_diff_card_says_diff_never_the_id_of_the_commit_it_shows`, novo:
  para cada um dos três diffs aberto pela superfície, o cartão abre com
  "diff" e nenhum nó da superfície tem o id do commit no nome ou no
  estado).

### a coluna

- a linha numa faixa ao longo de baixo da janela, na largura inteira
  (`apps/idestudio/src/generative/row.rs`, `place`, `row`): o campo (um card de
  largura, no máximo), o proponente, propor e fechar numa linha e a
  mensagem ao lado deles enquanto a largura deixa a ela o mínimo de um
  card; a mensagem embaixo deles quando não; os três embaixo do campo
  numa janela estreita. a faixa tem 60 px numa janela de 1280 por 800.
- sem superfície: o workspace tem a janela inteira de largura e a altura
  menos a faixa. com superfície: a coluna à direita, do topo até a faixa,
  metade da janela como antes. fechar devolve a largura. tudo pelo
  `resize` do workspace (`apps/idestudio/src/generative/mod.rs`,
  `lay_out`); nenhum arquivo de `apps/idestudio/src/interfaces/panels/**` mudou.
- o proponente abre o painel para cima: a linha dá ao `Select` do comps,
  para o painel, o controle levantado pela altura do painel e pelas duas
  folgas (`row.rs`, `lifted`); o controle é desenhado e clicado onde está.
- a linha continua a mesma para a pessoa, o agente e o teclado: os mesmos
  widgets, ids e projeção; só os retângulos mudaram
  (`apps/idestudio/src/generative/column.rs`).
- testes: `apps/idestudio/tests/line.rs` (novo, seis testes): sem superfície,
  as vistas fixas no mesmo x e com a mesma largura de um workbench sem a
  linha, e a linha inteira na faixa; a superfície abre na coluna e fechar
  devolve a largura; a pessoa chega à linha pelo clique e pelas teclas
  (Enter propõe, Escape solta); o painel do proponente abre acima do
  controle, dentro da janela, e a opção é escolhida pelo clique; a linha
  quebra na faixa em três larguras (1280, 800, 480) e o workspace fica com
  a largura inteira; um agente pela porta vê a linha na faixa (os
  retângulos da observação) e escreve, propõe e fecha por ela.
  `apps/idestudio/tests/surface.rs` mudou as asserções da coluna para a
  faixa.

### a demo 5 de novo

gravada sem janela no build `e42bd571`, lida pelo `plev-inspect`; o
resultado inteiro na task-80, seção "de novo, com a linha embaixo
(task-96, fase b)". em resumo: nenhum número do critério mudou de lado
(atos, arquivos, linhas do diff à vista e cortadas na largura iguais; o
tempo varia de corrida a corrida). o que mudou: no painel fixo as três
mensagens dos commits agora aparecem inteiras (0 de 3 cortadas, eram 3 de
3, medido pelo dado novo do laboratório sobre o layout de antes), e o
cartão do diff na superfície abre com "diff". o teste
`apps/agenteval/tests/review.rs` segura as 0 mensagens cortadas.

### as imagens

na scratchpad da sessão, `demo5b/`:

- antes (base `8704ba3a`): `before/fixed.png` (sem superfície: a coluna
  de 368 px, as mensagens cortadas), `before/surface.png` e
  `before/surface-diff.png` (com superfície: o título
  `stacks.commit[847f…`), `before/closed.png`.
- depois (build `e42bd571`): `after/fixed.png` (sem superfície: a
  largura inteira, as mensagens inteiras, a linha na faixa),
  `after/surface.png` e `after/surface-diff.png` (com superfície: a coluna
  do topo até a faixa, o cartão abrindo com "diff"), `after/closed.png`.
- `proposer/proposer-open.png`: o painel do proponente aberto para cima
  (uma corrida descartável do laboratório com um clique no proponente,
  fora do git).
- `before-measure/report.json`: a medida nova do laboratório sobre o
  layout de antes (3 de 3 mensagens cortadas).

### validação

com o ambiente do briefing em todo comando cargo (`CARGO_TARGET_DIR` em
`target/agents/one`, `CARGO_INCREMENTAL=0`, debug `line-tables-only`, o
`PLEV_CLAUDE` no stub que só sai com erro) e `--offline`
(`CARGO_NET_OFFLINE=true` no gate):

- `./tool/tasks/fullcheck.sh` em `a427d2b8`: verde (exit 0, 05:55 a 05:58), com a
  perna urnaui e o check wasm com `-D warnings`. workspace 2977 passam, 0
  falham, 26 ignorados (165 suítes); urnaui 85 passam, 1 ignorado (6
  suítes). total 3062 e 27, contra 3054 e 27 na base: os 8 testes novos.
  `df -h /`: 28 GB livres antes e depois.
- um gate anterior, em `e42bd571`, caiu em `apps/idestudio/tests/line.rs`
  (o teste que o `a427d2b8` corrige); o que rodou antes dele passava.
- `cargo run --offline -p regress -- check`, com o
  `main.rs` tocado: 176 baselines, 176 iguais, nenhum frame desenhado,
  nenhuma GPU aberta.
- `cargo test -p idestudio --test line` cinco vezes seguidas: verde.
- nenhuma janela aberta (o binário `ide` não rodou), nenhum clique ou
  tecla do sistema (os cliques são pelo `Workbench::click`, no meio do
  retângulo que o frame ligou), nenhuma chamada ao `claude -p`, nenhuma
  dependência nova, `Cargo.toml` e `Cargo.lock` iguais.

### decisões

- a faixa embaixo, não em cima: em cima pediria tirar o header do
  workspace (o modo embutido), uma vista fixa.
- a linha fica na faixa também com a superfície aberta: ela não pula de
  lugar quando uma superfície abre ou fecha.
- "diff" e não a mensagem do commit: sem estado novo, o título não
  consegue seguir o diff.
- o painel do proponente para cima por um retângulo levantado, sem mudar
  o comps.
- o id do título guardado em `review.rs`, porque `ids.rs` está fora da
  lista de arquivos.

### limitações e o que fica aberto

- o painel para cima é um arranjo em cima do `Select` do comps; um
  `Select` que abre para cima quando falta espaço embaixo é o jeito limpo
  (comps, P3).
- a faixa cresce quando a mensagem pede mais linhas do que ela tem (uma
  recusa longa numa janela estreita): o workspace perde esses pixels e os
  controles sobem junto. em 1280 de largura, uma mensagem de até duas
  linhas cabe na altura do campo; três linhas somam uns 4 px.
- sem coluna depois de fechar, o que resta da superfície some por cima
  do workspace (a área acima da faixa) durante o fade.
- as vistas fixas perdem 60 px de altura com a linha ligada.
- a mensagem do commit no título do cartão do diff pede um estado por
  região (junto do P3 dos três diffs de uma vez).
- seguem com o orquestrador e o brenner: a prova com o Claude de verdade
  e a demo 5 com janela.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [task-80: proof of qualitatively different software](../task-80-proof-qualitatively-different-software/task-80-proof-qualitatively-different-software.md)
- [task-96: a generated surface in the ide](task-96-ide-generated-surface.md)
- [task-97: code and diff in the IR vocabulary](../task-97-code-and-diff-in-the-ir/task-97-code-and-diff-in-the-ir.md)
- [five demos of qualitatively different software, each with its method written before it ran](../../Research/Exploring/five-demos.md)
<!-- /related -->
