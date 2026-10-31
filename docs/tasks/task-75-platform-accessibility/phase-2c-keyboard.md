---
id: task-75/phase-2c-keyboard
title: "task-75 phase 2c: keyboard on the component pages"
project: plev
type: section
status: reference
parent: task-75
date: 2026-09-25
date_basis: text
topics: [accessibility, input, semantics]
relations: []
source: docs/log/steps/task-75-platform-accessibility.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
evidence: [8ba4ef6d, 92635707, f1e04c32, 5dddcf6a, 917cdd2a]
---

# phase 2c: the keyboard probe (2026-09-25)

Records phase 2c: the keyboard on the 30 component pages. Tab and Shift-Tab now move in projection order, the same order the screen reader follows; the arrows serve tabs, tree, context menu, select and slider; Enter and Space activate as the person's request; and the modal traps focus and returns it on close. Before, Tab moved through 6 of the 19 panes, each with its own index, and the order already existed in the engine (`focus::next`), so the phase reuses that order instead of creating another.

branch `task/75d-keyboard`, sobre `main` em `73f99094`. o escopo é o
item 3 do escopo acima nas 30 páginas de componente (os 19 panes da
tabela em `view/dispatch/component.rs:108-128`). os caminhos são relativos
a `apps/uigallery/src/views` quando não dizem o crate. a leitura de cada
página foi feita com um teste sem janela que projeta a página, lista
`focus::order` e aperta Tab pela view (`handle_edit_key(EditKey::Tab)`).

### (a) o que Tab faz hoje

a ponte manda Tab como `EditKey::Tab`, sem olhar o Shift
(`src/keys.rs:18`): Shift-Tab é Tab. a view só passa Tab a seis panes
(`input.rs:145-167`), e cada um tem a sua própria ordem:

| página (pane) | focáveis na projeção | Tab hoje | por quê |
|---|---|---|---|
| Input, Field, Label (Input) | 3 campos | anda e dá a volta | `components/input.rs:127-133`, índice próprio |
| Checkbox | 2 caixas | anda | `components/checkbox.rs:71-77`, o `Focus` da página |
| Switch | 2 switches | anda | `components/switch.rs:70-76`, idem |
| Slider | 2 sliders | anda | `components/slider.rs:78-84`, idem |
| Select, Combobox (Select) | a pílula | fica na pílula | `components/select.rs:68-74` |
| Tabs | 3 abas | fica na aba ativa | `components/tabs.rs:76-82`: liga o anel da faixa inteira, e a projeção diz a ativa |
| Button, Button Group, Icon Button, Tooltip, Hover Card, Popover, Spinner, Toast, Dialog, Alert Dialog, Chip, Toggle, Breadcrumb, Navigation Menu, Kbd, Tree | de 1 a 16 | nada | `input.rs:165`: o braço `_ => false` |
| Table, Data Table, Scroll Area, Item, Resizable | 0 | nada | nenhum nó focável (ver (c)) |

o `Focus` das páginas (`components/mod.rs:73-92`) é uma segunda fonte de
ordem: um índice que a página avança, que não lê a projeção. Enter,
Espaço e as setas também não chegam às páginas de componente:
`handle_enter` só conhece os panes de app (`input.rs:172-200`), Espaço vira
`handle_key(" ")` e cai no `_ => false` (`input.rs:96`), cima e baixo só
servem à Git (`mod.rs:181-183`), e esquerda, direita, Home e End só editam
o campo focado.

### (b) onde a ordem da projeção dirige o Tab

o engine já tem a ordem: `accessibility::focus::order` e `focus::next`
(`rust/engine/src/control/accessibility/focus.rs:33-62`) andam o snapshot na
ordem da árvore que o leitor recebe, pulando o que é inerte, desabilitado
ou não focável, e dão a volta. o teste
`tab_and_the_reader_walk_the_same_order_and_the_page_takes_the_focus`
(`apps/uigallery/tests/access_tree.rs:397-423`) já prova que essa ordem é
a do leitor em toda página que projeta, e que toda página toma o foco que
o leitor pede (`assist_focus`, `dispatch.rs:464-495`, que numa página de
componente chega em `Component::focus`, `dispatch/component.rs:230-241`).

então Tab é `focus::next(&self.semantics(), direção)` seguido de
`assist_focus` no id que sai: a mesma projeção que o leitor anda, o mesmo
método que o leitor usa para mover o foco, e nenhuma ordem própria. o
Shift vem das teclas modificadoras que a view já recebe
(`mod.rs:169-171`), sem mexer na ponte. com isso o `Focus::next` e os seis
braços de Tab somem.

o que o contrato do engine diz e o padrão WAI-ARIA recomenda diferente: o
engine põe cada nó focável na sequência de Tab (`focus.rs:1-13`), então as
três abas e as nove linhas da árvore são nove paradas de Tab, e o padrão
recomenda uma parada por widget composto (o tabindex que roda). fazer isso
é dizer, na projeção, qual item do composto é focável, o que muda o
`semantics.json` das baselines. fica como pendência; as setas desta fase
funcionam por cima da sequência de hoje.

### (c) os widgets que pedem setas

| widget (papel) | páginas | padrão | o que já tem | o que falta |
|---|---|---|---|---|
| abas (`TabList`, `Tab`) | Tabs | esquerda e direita, Home e End; Enter e Espaço selecionam | `select` na projeção; anel só na faixa inteira (`rust/engine/src/widgets/nav/tabs.rs:93-99`, `176-178`) | anel na aba focada |
| árvore (`Tree`, `TreeItem`) | Tree | cima e baixo nas linhas visíveis, Home e End; direita abre o ramo fechado ou desce ao primeiro filho; esquerda fecha o ramo aberto ou sobe ao pai; Enter e Espaço selecionam | `select`, `toggle_expanded`, `expanded` e o pai de cada item na projeção | anel na linha (`components/tree.rs:19`: "a row draws no ring") |
| menu (`Menu`, `MenuItem`) | o menu de contexto, overlay | cima e baixo dão a volta, Home e End; Enter e Espaço acionam | `activate` por item; o destaque do hover (`rust/engine/src/widgets/feedback/context_menu.rs:102`) | o destaque seguir o foco |
| select (`Select`, `Option`) | Select, Combobox | Enter, Espaço, cima ou baixo abrem na opção escolhida; cima e baixo andam as opções, Home e End; Enter e Espaço escolhem; Escape fecha | `open`, `close`, `select`; o destaque por tecla no comps (`rust/engine/src/widgets/form/select/panel.rs:163-196`), sem uso | o destaque seguir a opção focada |
| slider (`Slider`) | Slider | direita e cima sobem, esquerda e baixo descem, Home e End vão aos extremos | `increment` e `decrement` no que tem passo, `set_value` com a faixa (`rust/engine/src/widgets/form/slider.rs:74-104`) | o contínuo não tem passo: a tecla pede `set_value` |
| lista e tabela (`List`, `Table`) | Scroll Area, Item, Table, Data Table | cima e baixo nas linhas | `select` por linha | nenhuma linha é focável na projeção; ver abaixo |
| radio | nenhuma página de componente | | | |

os outros controles não pedem seta, só anel e acionar: botões (Enter e
Espaço), caixas e switches (Espaço), links de navegação e crumbs (Enter).
três desenham o foco hoje sem anel: o chip de filtro
(`rust/engine/src/widgets/content/chip.rs`), o crumb (`rust/engine/src/widgets/nav/breadcrumb.rs`)
e o link de navegação (`rust/engine/src/widgets/nav/nav_link.rs`). Enter e Espaço
pedem a ação primária do nó focado, a mesma que o clique do leitor pede
(`rust/engine/src/control/accessibility/action/mod.rs:102`), pelo dispatcher da
galeria, como pessoa.

as listas e tabelas: as linhas projetam `select` mas não são focáveis, e a
lista e a tabela também não. nenhum Tab chega nelas sem a projeção dizer
um nó focável, o que muda as oito baselines dessas páginas (core e hoff).
é a condição de parar do briefing; não mexi, está no handoff como decisão.
o split da Resizable pede um papel de splitter no contrato (já na lista da
fase 2b).

### (d) o modal

- o trap já sai da projeção: com o modal aberto a região da página é
  inerte (`dispatch/component.rs:132-135`), e `focus::next` pula o que é
  inerte, então Tab sobre `semantics()` só anda entre os dois botões do
  modal. os toasts são live regions, não focáveis.
- o foco do modal é o cancel, ou o botão que o leitor escolheu
  (`dispatch/overlay.rs:108-115`); o comps `Modal` tem dois `Button` com
  `set_focused` (`rust/engine/src/widgets/feedback/modal.rs:47-48`), mas nenhum
  jeito de a view ligar o anel neles.
- o retorno já sai da projeção: a página guarda o foco dela enquanto o
  modal está aberto, e ele volta a ser o foco da projeção quando o modal
  fecha (`dispatch/overlay.rs:12-16`, `dispatch/detached.rs:226`). falta a
  página ter o foco no botão que abriu, o que Tab e Enter dão.
- o Escape tira o foco da página antes de fechar o overlay
  (`input.rs:113-133`): com o modal aberto por Enter, o primeiro Escape
  apagaria o anel do botão que abriu e deixaria o modal aberto. o overlay
  que segura o ponteiro fecha primeiro.

### (e) fora desta fase

os atalhos anunciados: o `sem::Node` não tem campo de atalho
(`rust/accord/src/controls.rs:35-73`), e dizer um atalho ao leitor pede o
contrato. vai para as pendências.

## handoff da fase 2c (2026-09-25)

branch `task/75d-keyboard`, sobre `main` em `73f99094`. sem push.

### commits

1. `8ba4ef6d` a sonda (seção "fase 2c: a sonda do teclado" acima).
2. `92635707` Tab e Shift-Tab na ordem da projeção, com o anel em todo
   controle.
3. `f1e04c32` as setas por widget; Enter e Espaço acionam.
4. `5dddcf6a` o modal: o trap, o anel no botão e o foco de volta.
5. `917cdd2a` os testes, sem janela.
6. este commit: o handoff.

### a sonda, curta

- (a) antes, Tab andava em 6 dos 19 panes, cada um com um índice próprio;
  Shift-Tab era Tab; Enter, Espaço e as setas não chegavam às páginas de
  componente.
- (b) a ordem já existia no engine (`focus::next`), e é a que o leitor
  anda. Tab virou `focus::next` sobre `semantics()` seguido de
  `assist_focus`; o índice das páginas saiu.
- (c) pedem setas: abas, árvore, menu de contexto, select e slider.
  listas e tabelas não têm nó focável (ver pendências); nenhuma página de
  componente tem radio.
- (d) o trap e o retorno já saíam da projeção. faltavam o anel no botão do
  modal e o Escape fechar o modal antes de tirar o foco da página.

### o mecanismo

`apps/uigallery/src/views/dispatcher/keyboard/mod.rs`. a view manda para ele o
Tab, as setas, Home, End, Enter e Espaço quando a página é de componente
ou quando um overlay segura o ponteiro (`input.rs`, `mod.rs`). o Shift
vem das teclas modificadoras que a view já recebia; a ponte
(`src/keys.rs`) não mudou.

- Tab: `focus::next` na projeção inteira e `assist_focus` no id que sai.
- setas: escolhem, na mesma ordem, os nós do widget em que o foco está.
- Enter e Espaço: a ação primária do nó focado
  (`accessibility::primary`), a mesma do clique do leitor.
- tudo que muda estado é pedido da pessoa (`Actor::Human`) pelo
  dispatcher da galeria (`act`). nenhum caminho novo para agente.

### o que mudou por widget

| widget | teclas | comps | página |
|---|---|---|---|
| botões (Button, Icon Button, Tooltip, Spinner, Toast, Dialog) | Tab, Enter, Espaço | nada (o anel já existia) | nada |
| caixa, switch, chip de filtro | Tab, Espaço, Enter | `Chip::set_focused` e o anel | o índice de Tab saiu; o chip liga o anel |
| campo de texto | Tab; as setas editam | nada | o índice de Tab saiu |
| link de navegação, crumb | Tab, Enter | `NavLink::set_focused`, `Breadcrumb::set_focused` e o anel | ligam o anel |
| abas | Tab; esquerda e direita dão a volta; Home, End; Enter e Espaço selecionam | `Tabs::set_focused_tab`: o anel na aba, por dentro | o foco é a aba focada |
| árvore | Tab; cima e baixo; direita abre ou entra; esquerda fecha ou sobe; Home, End; Enter seleciona | `Tree::set_focused` e o anel na linha, por dentro | liga o anel |
| select | Tab; cima, baixo, Enter ou Espaço abrem na opção escolhida; cima e baixo nas opções; Home, End; Enter escolhe; Escape fecha | `Select::set_highlighted`, que rola até a opção | o destaque segue a opção focada |
| slider | Tab; direita e cima sobem, esquerda e baixo descem; Home, End | nada | o índice de Tab saiu |
| menu de contexto | cima e baixo dão a volta e pulam o separador e o item desligado; Home, End; Enter escolhe | `ContextMenu::set_highlighted` | o destaque segue o foco |
| modal | Tab e Shift-Tab ficam nos dois botões; Enter e Espaço respondem; Escape fecha | `Modal::set_focused` e `focused` | o modal aberto pelo teclado mostra o anel no cancel; o foco volta ao botão que abriu |

o slider contínuo (sem passo) anda um centésimo da faixa por tecla, com
`set_value`; o com passo usa `increment` e `decrement`. o anel dentro da
linha e da aba é `recipe::focus_ring_inside`, o mesmo traço do
`focus_ring`, para não invadir a vizinha. o Escape agora fecha primeiro o
overlay que segura o ponteiro, e nada debaixo dele recebe letra ou tecla
de edição.

### os testes

- `apps/uigallery/tests/component_pages/keyboard.rs` (6):
  - Tab em todas as 30 páginas: cada controle focável é alcançado na
    ordem da projeção, dá a volta, Shift-Tab volta, o foco nunca some, o
    anel aparece no controle focado (um anel a mais no quadro), e o
    Escape tira. as cinco páginas sem nó focável são a lista conhecida.
  - as setas nas abas, na árvore, no select e nos sliders.
  - Enter e Espaço: caixa, switch e chip terminam no mesmo estado do
    clique; link, crumb e botão do Toast.
  - o modal: o foco entra no cancel com anel, Tab e Shift-Tab ficam
    dentro, Espaço responde, o foco volta ao botão que abriu; o mesmo
    com Escape.
- `apps/uigallery/src/views/dispatcher/keyboard/tests.rs` (1): o menu de
  contexto com as setas, o destaque, o Tab preso e o Enter.
- `rust/engine/src/widgets/tests/focus.rs` (+5): os anéis e destaques novos.
- mudaram dois testes de teclado, nenhum de clique: o de Tab da página
  Input virou "o campo toma o foco que o Tab pede" (o Tab é da view); e
  em `apps/app/tests/semantics.rs` o Enter com o modal aberto agora
  responde o modal (antes caía na página inerte).

### os PNGs (sem janela, tema core, 1280x800)

- `target/task-75d/tree-after-3-tabs.png`: a árvore depois de três Tabs,
  o anel em widgets.rs.
- `target/task-75d/tabs-tab-then-right.png`: Tab e seta para a direita, o
  anel em Appearance, Account segue ativa.
- `target/task-75d/dialog-modal-open-focus-on-cancel.png`: dois Tabs e
  Enter na página Dialog; o modal aberto com o anel no Cancel.

o caminho é relativo ao worktree
(`/Users/nn/Dev/hoff/plevdev/.claude/worktrees/agent`).

### gate e regress

- `./tool/tasks/fullcheck.sh` verde, pernas urnaui e wasm rodaram: 3213 passam (3128
  no workspace, 85 no urnaui), 28 ignorados, 0 falhas. a branch soma 13
  testes e não tira nenhum.
- `regress check`: 176 baselines, 176 iguais. nenhuma baseline mudou.
- `Cargo.toml` e `Cargo.lock` intocados; engine, tema, sem e wire
  intocados. sempre `--offline`, com `PLEV_CLAUDE` no stub, sem janela e
  sem tecla do sistema.

### pendências

- listas e tabelas (Table, Data Table, Scroll Area, Item): nenhum nó
  focável, então o Tab não chega. é decisão, porque muda baseline.
  opções: (1) toda linha visível focável: 11 a 20 paradas de Tab por
  página, ruim; (2) uma linha focável por vez (a selecionada, ou a
  primeira visível) e as setas andam nas linhas; (3) a lista focável com
  um item ativo, o que pede um conceito novo no `sem`. recomendo (2),
  junto com o item abaixo, num bless só: 8 baselines das listas e
  tabelas, mais 4 de Tabs e Tree.
- uma parada de Tab por widget composto (o tabindex que roda, como o
  WAI-ARIA recomenda): hoje cada aba e cada linha da árvore é uma parada,
  porque o engine põe todo nó focável na sequência. pede a projeção dizer
  qual item é focável; muda o `semantics.json` de Tabs e Tree.
- o split da Resizable: pede um papel de splitter no contrato.
- os atalhos anunciados: o `sem::Node` não tem campo de atalho.
- o menu de contexto não abre pelo teclado (Shift-F10 ou a tecla de
  menu), e PageUp e PageDown não chegam: a ponte (`src/keys.rs`) não
  manda essas teclas.
- o modal aberto por clique não mostra o anel no cancel até o primeiro
  Tab; um clique não dá foco a controle, como antes.
- o roteiro do VoiceOver da fase 2b pode ganhar um passo de teclado
  (Tab, setas e o modal), para o brenner rodar.

## fase 2c integrada (orquestrador, 2026-09-25)

o item "teclado completo nas páginas de componente, com teste" avançou,
sem fechar: o Tab anda na ordem da projeção (a do leitor de tela) nas 25
páginas de componente que têm algo focável, com o anel de foco; setas
pelo padrão WAI-ARIA em abas, árvore, select, slider e menu de contexto;
Enter e Espaço acionam pelo dispatcher como pedido da pessoa; o modal
prende o foco e o devolve ao fechar. 13 testes novos sem janela.

validação do orquestrador: diff lido (`view/dispatch/keyboard.rs` usa
`focus::next` e `assist_focus`, e todo pedido sai como `Actor::Human`
pelo dispatcher da galeria); os PNGs abertos: o anel numa linha da
árvore depois de três Tabs e no Cancelar do modal. o botão que abriu o
modal continua com o anel por baixo do véu (pendência pequena).

o que falta para fechar o item: Table, Data Table, Scroll Area e Item
não têm nó focável, e abas e árvore são uma parada de Tab por item. a
recomendação do agente (uma linha focável por vez, setas andando nas
linhas, cada widget composto uma parada só) muda 12 baselines, que o
orquestrador revisa antes de aceitar. o split da Resizable pede um papel
de splitter no contrato do `sem` e fica como pendência.

o agente gastou uns 350 mil tokens.
