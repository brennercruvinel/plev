---
id: task-75/phase-2d-lists-and-tables
title: "task-75 phase 2d: lists and tables with keyboard"
project: plev
type: section
parent: task-75
status: reference
date: 2026-09-25
date_basis: text
recorded: 2026-09-23
topics: [accessibility, input, semantics]
relations: []
evidence: [e61cfa14, 63e36503, 503b9bef, cc6fafc9, 10891b80, 2a19ad66]
source: docs/log/steps/task-75-platform-accessibility.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase 2d handoff (2026-09-25)

Records phase 2d: lists and tables (Scroll Area, Item, Table and Data Table) now take focus one row at a time, with arrow keys, Home and End, and scrolling follows focus, which completes keyboard coverage in 29 of the 30 component pages. Resizable stays out because the split needs a splitter role in the `sem` contract. The first delivery also made tabs and tree a single stop, which took focus away from the other items for the screen reader; the orchestrator undid that.

branch `task/75e-roving`, sobre `main` em `12e18669`. sem push. listas e
tabelas ganham o teclado; abas e árvore ficam como na fase 2c (cada item
uma parada de Tab), por decisão do orquestrador (ver pendências). a
Resizable fica de fora (pede o papel de splitter).

### commits

1. `e61cfa14` listas e tabelas focáveis, uma linha por vez.
2. `63e36503` abas e árvore como uma parada só (desfeito no 6).
3. `503b9bef` os testes, sem janela.
4. `cc6fafc9` as 12 baselines, só significado (4 delas voltam no 6).
5. `10891b80` o primeiro handoff.
6. `2a19ad66` a correção do orquestrador: abas e árvore voltam à fase 2c
   (projeção, testes e o teste do leitor iguais à main), e as baselines
   de tabs e tree voltam às da main, sem bless novo.
7. este commit: o handoff corrigido.

### o mecanismo

a lista e a tabela dizem na projeção qual linha é focável (o tabindex
que roda do WAI-ARIA): só essa entra na sequência de Tab, então
`focus::next` para uma vez em cada uma. quem decide é o comps, no
`project`: a linha focada, senão a selecionada em vista, senão a
primeira em vista. as setas (`view/dispatch/keyboard.rs`) andam entre
todas as linhas, focáveis ou não, e a linha que recebe o foco vira a
parada na projeção seguinte. nenhuma ordem própria na view, nada novo no
`sem` nem no engine.

### o que mudou por widget

| widget | parada de Tab | teclas | comps | página |
|---|---|---|---|---|
| lista (Scroll Area, Item) | a linha focada, senão a selecionada em vista, senão a primeira em vista | cima e baixo; Home e End às pontas da lista inteira (10 mil linhas), guardada ou não; Enter e Espaço selecionam | `VirtualList::set_focused` (rola até a linha), `focused` (segue a rolagem), `tab_stop`, `shown`; o anel dentro da linha | toma e larga o foco; o clique larga, como na árvore |
| tabela (Table, Data Table) | a mesma regra, nas linhas | as mesmas | `Table::project` usa a parada da lista | a mesma |
| abas, árvore | como na fase 2c: cada item é uma parada | as da fase 2c | nada | nada |

o foco segue a rolagem nos dois sentidos: a seta rola o mínimo para a
linha focada caber inteira, e a roda, quando tira a linha focada de
vista, leva o foco para a linha mais perto que está em vista. o foco
nunca some. a seta que rola mostra a barra de rolagem, como a roda.

### os testes

- `apps/uigallery/tests/component_pages/keyboard.rs`: o Tab em todas
  as 30 páginas agora chega em 29 (só a Resizable fica sem nó focável),
  e em cada lista e tabela para uma vez só (4 páginas, lista conferida).
  novo `the_arrows_walk_the_rows_of_a_list_and_a_table`, nas 4 páginas:
  Tab cai na primeira linha, setas, Home e End às pontas com rolagem, o
  anel na linha, a linha focada é a única parada, Enter e Espaço
  selecionam como o clique (o estado inteiro igual ao do clique), Escape
  e Tab voltam na selecionada, a roda leva o foco junto e ele nunca
  some. os testes de abas e árvore são os da fase 2c.
- `rust/engine/src/widgets/tests/widgets/semantics/roving.rs` (3): a parada da
  lista e da tabela; a linha que rola para a vista; o foco seguindo a
  roda.
- `rust/engine/src/widgets/tests/focus.rs` (+1): o anel dentro da linha da lista
  e da tabela.
- `apps/uigallery/tests/access_tree.rs` igual à main. digitar num campo,
  o clique e o mouse: os testes de 2b e 2c seguem verdes sem mudança.

### as baselines regravadas

o antes e o depois de cada uma estão em `target/task-75e/baselines/<página>@<tema>/`
(`before.png`, `after.png`, `before-after.png` lado a lado,
`before.semantics.json`, `after.semantics.json`, `semantics.diff`). os
PNGs antes e depois são iguais byte a byte; o regress disse "visual
unchanged", sem desenhar quadro. o `scene.json` de cada uma muda só a
revisão da projeção.

- data-table@core: a linha 0 passa a ser focável.
- data-table@hoff: a linha 0 passa a ser focável.
- table@core: a linha 0 passa a ser focável.
- table@hoff: a linha 0 passa a ser focável.
- scroll-area@core: o item 0 passa a ser focável.
- scroll-area@hoff: o item 0 passa a ser focável.
- item@core: o item 0 passa a ser focável.
- item@hoff: o item 0 passa a ser focável.

tabs e tree (core e hoff) foram regravadas no commit 4 e voltaram às da
main no commit 6; as pastas delas em `target/task-75e/baselines/` são do
primeiro handoff e não valem mais. nenhuma outra baseline mudou.

### o PNG

`target/task-75e/data-table-tab-then-two-downs.png` (sem janela, tema
core, 1280x800): Tab e duas setas para baixo na Data Table, o anel na
linha file_2.rs.

### gate e regress

- `./tool/tasks/fullcheck.sh` verde, pernas urnaui e wasm rodaram: 3218 passam (3133
  no workspace, 85 no urnaui), 28 ignorados, 0 falhas. a branch soma 5
  testes e não tira nenhum.

- `regress check` antes do bless: 176 baselines, 12 mudaram (só
  significado); depois da correção: 176 iguais.
- `Cargo.toml`, `Cargo.lock`, engine, tema, `sem` e wire intocados.
  sempre `--offline`, com `PLEV_CLAUDE` no stub, sem janela e sem tecla do
  sistema.

### pendências

- parada única por widget composto pede no contrato um focável fora da
  sequência de Tab. hoje o contrato liga `focusable` à sequência de Tab,
  e o engine só oferece o `Focus` do leitor ao nó focável: tirar uma aba
  ou uma linha da árvore da sequência tira o `Focus` do leitor dela. por
  isso abas e árvore seguem com uma parada por item. as linhas de lista e
  tabela não perdem nada (antes nenhuma era focável).
- o split da Resizable: pede um papel de splitter no contrato.
- os atalhos anunciados: o `sem::Node` não tem campo de atalho.
- PageUp e PageDown nas listas, e o menu de contexto pelo teclado: a
  ponte (`src/keys.rs`) não manda essas teclas.
- a Data Table desenha as linhas do overscan abaixo do painel (já era
  assim; os PNGs antes e depois são iguais). o anel nunca cai nelas: o
  foco fica nas linhas em vista.
- o roteiro do VoiceOver pode ganhar um passo de teclado (Tab, setas,
  listas e o modal), para o brenner rodar.

## fase 2d integrada (orquestrador, 2026-09-25)

o item "teclado completo nas páginas de componente, com teste" fecha em
29 das 30 páginas: listas e tabelas (Table, Data Table, Scroll Area,
Item) recebem o foco uma linha por vez, as setas andam nas linhas, Home e
End às pontas, Enter e Espaço selecionam, e o foco segue a rolagem. o
teste percorre as 29 com Tab. a Resizable fica de fora: o split pede um
papel de splitter no contrato do `sem` (pendência).

validação do orquestrador: as 8 baselines de listas e tabelas mudam só o
significado (a linha ou o item 0 focável); os PNGs são idênticos byte a
byte. a primeira entrega também fazia de abas e árvore uma parada só de
Tab, mas isso tirava o `Focus` do leitor dos outros itens (regressão para
quem usa leitor de tela); voltou ao comportamento da fase 2c, e a parada
única por widget composto ficou como pendência (pede no contrato um
focável fora da sequência de Tab). o PNG aberto: o anel na linha
`file_2.rs` depois de Tab e duas setas.

o que continua aberto na task: o roteiro com o leitor real (o brenner
roda), as cores que falham no contraste (decisão do brenner) e as
pendências de contrato acima.

o agente gastou uns 500 mil tokens (a entrega e a correção).
