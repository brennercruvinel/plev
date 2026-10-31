---
id: task-75/phase-2b-component-pages
title: "task-75 phase 2b: component pages"
project: plev
type: section
parent: task-75
status: reference
date: 2026-09-25
date_basis: text
recorded: 2026-09-23
topics: [accessibility, semantics, agents, determinism]
relations: [adr-0068, task-59]
evidence: [7fa4f84c, e5f738e5, 22819cc9, a64f9e93, 8f8cf475, a66e6e1d, 5e0d2c2f, 3badee06]
source: docs/log/steps/task-75-platform-accessibility.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase 2b: the probe (2026-09-25)

Records phase 2b: the 76 component pages that projected nothing, classified by the probe into three groups (30 pages with widgets already projectable in comps, 24 with widgets that have no projection, and 22 with nothing to operate). A `Component` trait and a 19-row table give each page a region with its own title, served by the gallery's dispatcher to the person, the reader and an agent, with the person's click landing on the widget. The `SILENT` list no longer has 76 pages without a reason: it now has 46, each with its reason, and there are 60 new baselines in core and hoff.

branch `task/75c-a11y-component-pages`, sobre `main` em `3048b320`. a
mesma leitura das fases 1 e 2a: o código com arquivo e linha, o AccessKit
pela fonte do lock (accesskit_macos 0.26.1, accesskit_windows 0.32.1,
accesskit_atspi_common 0.18.1). o escopo são as 76 páginas que não
projetam nada (82 na tabela `view/pages/table.rs`, menos as seis que
projetam). os caminhos abaixo são relativos a `apps/uigallery/src/views`
quando não dizem o crate.

### (a) as 76 páginas em três grupos

a regra: grupo 1 quando todo widget que a pessoa opera na página já tem
`project` no comps (os 19 arquivos da task-59); grupo 2 quando a página
mostra um widget sem projeção que carrega uma operação ou um dado que o
leitor precisa ouvir; grupo 3 quando a página não mostra nada que se
opere nem nada a ler além de uma amostra de desenho. uma página alias
(`Body::Alias`, `pages/mod.rs:121`) mostra o pane de outra, então vai
para o grupo do pane.

grupo 1: 30 páginas, 19 panes. o que o clique faz hoje é a referência do
"mesmo estado".

| página (pane) | arquivo | widgets e projeção no comps | o clique hoje |
|---|---|---|---|
| Dialog, Alert Dialog | `components/dialog.rs` | 2 `Button` (`comps/src/action/button.rs:119`) | abre o modal da view (`dispatch.rs:121-127`, `input.rs:247`) |
| Toast | `components/toast.rs` | 4 `Button` | põe um toast (`dispatch.rs:148-155`) |
| Button, Button Group | `components/button.rs` | 30 widgets: `Button`, `IconButton` (`action/icon_button.rs:94`), `SubmitButton` (`action/submit_button.rs:62`), em 14 grupos | nada além da pressão (`button.rs:314-329`) |
| Icon Button | `components/icon_button.rs` | 7 `IconButton` | nada (`icon_button.rs:46-58`) |
| Tooltip, Hover Card, Popover | `components/tooltip.rs` | 1 `Button`; o `Tooltip` não se projeta, o texto dele é a descrição do botão | nada; o hover mostra o tooltip (`tooltip.rs:35-54`) |
| Spinner | `components/spinner.rs` | 1 `Button`; os 3 `Spinner` não se projetam (resto no grupo 2) | liga e desliga o giro (`spinner.rs:64-76`) |
| Checkbox | `components/checkbox.rs` | 3 `Checkbox` (`form/checkbox.rs:63`), um desligado | o widget inverte `checked` (`comps/src/form/checkbox.rs:130-137`) |
| Switch | `components/switch.rs` | 3 `Switch` (`form/switch.rs:70`), um desligado | o widget chama `toggle` (`comps/src/form/switch.rs:159-166`) |
| Chip, Toggle | `components/chip.rs` | 6 `Chip` (`action/chip.rs:74`): 5 etiquetas e "click me" | a página inverte `selected` do "click me" (`chip.rs:58-61`) |
| Slider | `components/slider.rs` | 3 `Slider` (`form/slider.rs:74`): contínuo, com passo, desligado | o arrasto muda `value` no widget |
| Input, Field, Label | `components/input.rs` | 3 `TextField` (`form/text_field.rs:80`) e a linha "live value" | foco e caret (`input.rs:67-89`); digitar insere (`input.rs:91-97`) |
| Select, Combobox | `components/select.rs` | 1 `Select` (`form/select.rs:130`) | o widget abre, fecha e escolhe |
| Tabs | `components/tabs.rs` | 1 `Tabs` (`nav/tabs.rs:64`) | o widget muda `active` (`comps/src/nav/tabs.rs:147-151`) |
| Tree | `components/tree.rs` | 1 `Tree` (`content/tree.rs:146`) | folha: seleciona; ramo: abre ou fecha e seleciona (`comps/src/content/tree.rs:260-272`) |
| Data Table, Table | `components/table.rs` | 1 `Table` (`content/table.rs:127`), 2000 linhas virtualizadas | o widget seleciona a linha (`comps/src/content/list.rs:249-253`) |
| Scroll Area, Item | `components/scroll_area.rs` | 1 `VirtualList` (`content/list.rs:105`), 10000 linhas | seleciona a linha, rola |
| Breadcrumb | `components/breadcrumb.rs` | 1 `Breadcrumb` (`nav/breadcrumb.rs:62`) e o eco "clicked crumb n" | a página guarda o crumb e o desenha (`breadcrumb.rs:35-47, 53-62`) |
| Navigation Menu, Kbd | `components/navigation_menu.rs` | 4 `NavLink` (`nav/nav_link.rs:69`) e a forma de trilho | a página move a marca de ativo (`navigation_menu.rs:64-86`) |
| Resizable | `components/resizable.rs` | 1 `SplitPane` (`nav/split_pane.rs:96`) | o arrasto muda `ratio` |

três projeções que a sonda mostra erradas para páginas do escopo, as três
corrigidas na página (nada no comps):

- o chip "click me" é `interactive` (`chip.rs:23`), e o comps projeta um
  chip interativo como `Button` com `activate`
  (`comps/src/action/chip.rs:80-96`); a página usa o chip como alternância
  (inverte `selected` no clique, `chip.rs:58-61`). correção: o chip passa a
  `selectable(true)`, que é a alternância do comps (um `Toggle` com
  `selected` e `toggle`, e o próprio widget inverte no clique,
  `comps/src/action/chip.rs:170-176`), e a página deixa de inverter. o
  desenho é o mesmo: o chip lê só `pressable()`, que é `interactive ||
  selectable` (`comps/src/action/chip.rs:124-126`, 216).
- o rótulo do botão do Spinner muda dentro do `render`
  (`spinner.rs:101-105`): uma projeção tirada depois do pedido e antes do
  próximo quadro diria "Start spinners" com o giro ligado. correção: o
  rótulo muda junto com o giro.
- a Navigation Menu desenha o primeiro link duas vezes, na coluna e no
  trilho (`navigation_menu.rs:93`); com um `SemId`, o segundo `bind`
  seria recusado com aviso (`engine/src/compositor/binding.rs:147-152`).
  correção: o trilho é um `NavLink` próprio, sem id, que espelha o
  primeiro (ativo e hover).

grupo 2: 24 páginas. o que cada widget precisaria para entrar (papel,
estado, ações); nenhum é escrito nesta fase.

| página (pane) | widget sem projeção | o que precisaria |
|---|---|---|
| Theme | os cartões de paleta desenhados à mão (`foundations/theme.rs:169-194`) | um grupo de opções (`Radio` ou `Option`) com `selected`, cada um apertado pelo `shell.theme select` que já existe (`dispatch/shell.rs`) |
| Avatar | `Avatar` (`comps/src/content/avatar.rs:37`) | `Image` nomeado pelo nome, com `online` |
| Badge | `Badge` (`comps/src/action/badge.rs:27`) | `Label` (ou `Status` quando conta) com o texto e a intenção |
| Stat | `Stat` (`comps/src/content/stat.rs:26`) | `Label` com valor, unidade e o delta como descrição |
| Progress | `ProgressBar` (`comps/src/feedback/progress.rs:12`) | `ProgressBar` com `value` e `range` 0 a 1 (o padrão de `capture.level`) |
| Spinner (o resto) | `Spinner` (`comps/src/feedback/spinner.rs:53`) | `ProgressBar` sem valor (indeterminado) enquanto gira, ou `busy` no pai |
| Skeleton | `Skeleton` (`comps/src/content/skeleton.rs:28`) | um nó "carregando" (`busy`); o pulso ligado pelo clique (`skeleton.rs:50-60`) precisaria de uma ação |
| Code Block | `CodeBlock` (`comps/src/content/code_block.rs:79`) | `Label` com o texto e a legenda como nome |
| Card | `Card` (`comps/src/content/card.rs:140`) | um `Group` por variante com os textos e números dela |
| Chart | os gráficos de `comps::charts` (`components/chart.rs`) | `Image` com um texto alternativo por gráfico (ou uma tabela dos dados); o replay do clique (`chart.rs:95-104`) seria uma ação |
| Graph | `GraphView` (`comps/src/graph.rs:65`) | `Canvas` com um item por nó e `select`; pan e zoom como ações |
| Empty | `EmptyState` (`comps/src/content/empty_state.rs:22`) | `Group` nomeado pelo título, descrito pela mensagem, com o `cta` (`Button`, já projetável) dentro |
| Context Menu, Dropdown Menu | a área de clique direito (`components/context_menu.rs:69-99`) | um nó gatilho com uma ação que abre o menu; o menu aberto já se projeta (fase 2a) |
| Sidebar, Drawer, Sheet | `Sidebar` (`comps/src/nav/sidebar/mod.rs:49`) | uma região de navegação com os `NavLink` (projetáveis) e cada `NavGroup` com `expanded` e uma ação que dobra |
| Bubble, Message, Message Scroller, Dock | o dock desenhado à mão (`apps/dock.rs`) | lista de mensagens (`List`/`ListItem`), o campo (`TextInput`) e os contatos (`Button`) |
| Editor | `EditorView` (`comps/src/editor/view.rs:45`) | `TextInput` multilinha com `text`, seleção e caret (o AccessKit do lock só lê texto sobre `TextRun`, fase 1) |
| Git | as views do ide (`apps/git.rs`) | a projeção das views do ide; o ide está fora do escopo |
| Builder | os elementos do `engine::builder` (`apps/builder.rs`) | um `Button` por região de clique (os botões do contador) e o valor como `Status` |
| Panel Header | `PanelHeader` (`comps/src/nav/panel_header.rs:14`) | um cabeçalho: `Label` com o título, a descrição e o badge |

grupo 3: 22 páginas, nada: não há o que operar nem o que ler além de uma
amostra de desenho, e projetar os textos de amostra só daria ao leitor e
ao agente nós que não mudam nada. o nome da página é do cabeçalho do
shell, não da página.

| página | por quê |
|---|---|
| Typography | amostras da escala de tipo (`foundations/typography.rs:231-233`: nenhum evento) |
| Icons | grade de ícones; só hover (`foundations/icons.rs:53-65`) |
| Effects | amostras de sombra, blur, gradiente (`foundations/effects.rs:154-156`) |
| Panel | a superfície de vidro sozinha (`components/panel.rs:30-37`) |
| Separator | réguas entre legendas (`components/separator.rs:23-30`) |
| 17 placeholders (Accordion, Alert, Aspect Ratio, Attachment, Calendar, Carousel, Collapsible, Command, Date Picker, Input Group, Input OTP, Marker, Menubar, Pagination, Questionnaire, Radio Group, Textarea) | `Body::Placeholder` (`pages/mod.rs:123`): um `EmptyState` que diz "Not in comps yet" (`dispatch.rs:513-527`); o widget não existe |

contagem: 30 + 24 + 22 = 76.

### (b) o mecanismo

hoje `page_semantics` (`dispatch.rs:379-397`), `act` (`dispatch.rs:406-442`)
e `assist_focus` (`dispatch.rs:463-495`) conhecem cinco panes cada, e o
Escape (`input.rs:113-133`) conhece os que têm foco de teclado. o menor
mecanismo:

- um trait `Component` em `view/dispatch/component.rs` (novo, filho de
  `dispatch.rs`, que é o único que casa com `Pane` pela convenção): a
  página projeta seus nós sob uma raiz dela, roda um pedido e diz o que a
  view faz depois (um toast, um modal), toma o foco que um leitor pede e o
  solta no Escape.
- uma tabela só, uma linha por pane (`Pane::Checkbox => checkbox`), escrita
  uma vez por uma macro que gera os dois acessos (`&dyn` e `&mut dyn` com o
  todo emprestado para o dispatcher). são 19 linhas, não 70 braços.
- a view entra por quatro pontos, um braço cada: `page_semantics` projeta a
  página do grupo 1 sob uma região nomeada pelo título da página
  (`Role::Region`, inerte enquanto um overlay segura o ponteiro, como a
  todo em `apps/app/semantics.rs:160-162`); `act` roda o pedido por um
  `Operable` que empresta o dispatcher do todo, o padrão do overlay da fase
  2a (`dispatch/overlay.rs:277-290`); `assist_focus` passa o foco à página
  (recusado enquanto inerte); o Escape solta o foco que o leitor deixou.
- os ids: a raiz é `<pane>_page` (`checkbox_page`, `dialog_page`), o que
  não colide com `dialog`, `toasts` e `context_menu` da view
  (`dispatch/overlay.rs:36-38`); os widgets ganham o id pelo `.sem(id)` do
  comps, então o `render` liga cada retângulo (`bind`) sem mudar um nó da
  cena (`engine/src/compositor/binding.rs:1-12`: a ligação fica ao lado da
  cena, fora de todo hash).

o clique da pessoa: direto no widget, com a projeção seguindo o estado (a
regra da task-59). por quê:

1. os widgets do comps mudam o próprio estado no evento: o checkbox inverte
   `checked` (`comps/src/form/checkbox.rs:130-137`), o switch chama
   `toggle` (`switch.rs:159-166`), as tabs mudam `active` (`tabs.rs:147-151`),
   a árvore abre e seleciona (`tree.rs:260-272`), a lista e a tabela
   selecionam (`list.rs:249-253`), o slider e o split arrastam, o campo
   edita. passar o clique pelo dispatcher pediria desfazer o que o widget já
   fez, ou refazer o hit test dele na página; o comps não muda evento nesta
   fase.
2. toda ação do comps lê o que muda (`toggle` lê `checked`, `on` ou
   `selected`; `select` lê `selected`; `set_value` lê `value` ou `text`), e
   a projeção é leitura pura do widget: um pedido decidido antes de um
   clique da pessoa volta `Stale` com o delta, para o leitor e para o
   agente.
3. o que a página faz por cima do clique (o modal, o toast, o eco do crumb,
   a marca do link, o giro) é um método só, que o clique e o `run` chamam.

o custo: o clique da pessoa numa página de componente não deixa recibo no
dispatcher (o do leitor e o do agente deixam). as páginas de app seguem
passando o clique pelo dispatcher, como antes.

o requisito vira um teste por tipo de widget: a pessoa (clique no
retângulo ligado, ou teclas), o leitor (a árvore, `accessibility::translate`,
`act`) e um agente pelo port de verdade (`wire::AgentPort` e um `Client`
numa thread, como `dispatch/agent.rs`) partem de três views iguais e
chegam à mesma projeção.

### (c) quem mais lê a projeção

- as baselines: cada página do grupo 1 ganha um `semantics.json` e, no
  `scene.json`, o bloco `semantics` (de `null` para revisão e contagem) e o
  bloco `bindings` (os widgets). são 60 baselines (30 páginas, core e
  hoff); nenhum `frame.png` existe nelas. **um efeito que o briefing não
  previu**: o nome de uma camada no `scene.json` vem das raízes dos ids
  ligados nela (`examples/regress/scene.rs:6-12, 104-131`), e uma camada
  sem ligação se chama pelo z (`z0`). ligar os widgets renomeia a camada
  da página de `z0` para a raiz (`checkbox_page`), e na Scroll Area e na
  Item a camada da lista de `z10` para `scroll_area_page`. hash, contagem
  de nós, z e limites ficam; só o nome anda, e ele é espelho das
  ligações, como a revisão no bloco `semantics` foi na fase 2a. li a
  autorização como cobrindo esse espelho; o commit das baselines vai
  separado e um script confirma, camada a camada na ordem de composição,
  que z, hash, nós e limites são iguais e que o nome novo é a junção das
  raízes ligadas. a outra saída seria projetar sem ligar (a árvore do
  leitor sem retângulo nenhum nessas páginas, e o port sem `bounds`). as
  seis que já projetam e os goldens do `uir` não mudam.
- a barra de menu (`app/menu.rs:195-212`): `page_menu` lista as ações
  simples da página ativa. numa página de componente ela mostraria 30
  itens "Button" na Button, "Autosave on focus loss: toggle" na Checkbox,
  "Neutral", "Success" na Toast: amostras de widget, não capacidades da
  página. proposta: deixar fora, como o shell e o overlay já ficam
  (`view.is_component`), com teste; a Toggle Group mostra a todo e segue
  com a barra dela.
- o tray (`app/tray.rs:124-145`) e a notificação (`app/notify.rs:200-204`)
  leem só o shell; a impressão pede `shell.print` (`app/print.rs:48-52`); os
  deep links só pedem o que `OPEN` lista, `shell.page` e `shell.theme`
  (`app/links.rs:46`), e recusam o resto antes do pedido; o sleep lê a
  Silences. nada muda neles.
- a gravação (`app/record.rs:162-165`) passa a guardar a projeção das
  páginas de componente numa sessão gravada; é consequência, não pede
  mudança. a demo gravada (`fixtures/demo/`) é da Silences, dado.
- o port (`dispatch/agent.rs:61-67`) serve `gallery_semantics`, que passa a
  levar a página de componente, com os `bounds` das ligações.
- `rust/socket/tests` e `apps/idestudio/tests`: nenhum lê a galeria (grep por
  `ShowcaseView` e `gallery_semantics`: zero). o laboratório é do ide.
- os testes da galeria que usam uma página de componente como "página sem
  projeção" mudam de página: `apps/app/semantics/assistive.rs:333-345`
  (Button) e `app/headless.rs:549-560` (Button); o de view source que usa um
  texto formatado da Data Table (`apps/uigallery/tests/view_source.rs:147-151`)
  cai dentro de uma linha ligada e muda de exemplo. `tests/access_tree.rs`
  encolhe a lista página por página.

### (d) as promessas de privacidade no escopo

- valor secreto: nenhuma página do grupo 1 tem senha, OTP ou campo marcado
  como segredo (os campos da Input são nome, email e codinome,
  `input.rs:22`); a Input OTP é placeholder (grupo 3).
- pixel de câmera: nenhuma página do escopo tem câmera (só a Capture, fora
  do escopo).
- ação `PERSON_ONLY`: nenhuma página do escopo a demonstra (o
  `PERSON_ONLY` é do shell, da Files e da Capture).
- a preferência de accessibility: só a Silences a tem; nenhuma página do
  escopo.

nenhum caso presente; o teste guarda a ausência: se uma página do grupo 1
passar a projetar um segredo, uma mídia, uma ação que recusa agente ou um
estado só do leitor, ele falha e pede o teste do caso.

### (e) o que cada papel vira por plataforma

a matriz da fase 1 cobre o botão, o switch e a tab; o resto pela fonte do
lock:

| papel (sem) | macOS (accesskit_macos 0.26.1 `node.rs`) | windows (accesskit_windows 0.32.1 `node.rs`) | AT-SPI (accesskit_atspi_common 0.18.1 `node.rs`) |
|---|---|---|---|
| `Region` (a raiz da página) | `AXGroup` (148) com subrole `AXLandmarkRegion` (268) | `Group` (169) | `Landmark` (207) |
| `Button` | `AXButton` (65), press (668) | `Button` (89), `Invoke` (571-573) | `Button` (81-87), `click` |
| `Toggle` (switch, chip) | `AXCheckBox` subrole switch (82, 275) | `Button` (99) com `Toggle` (559-561) | `ToggleButton` (93), `Pressed` (344-345) |
| `Checkbox` | `AXCheckBox` (56) | `CheckBox` (74), `Toggle` | `CheckBox` (92), `Checked` (347) |
| `Slider` | `AXSlider` (158), increment e decrement | `Slider` (188), `RangeValue` (582-584), sem increment | `Slider` (225), valor numérico, sem increment |
| `TextInput` | `AXTextField` (58-64), `setAccessibilityValue` (555) | `Edit` (76-88), `Value` (575-580) | texto (239); sem `SetValue` de texto (fase 1) |
| `Select` e `Option` | fase 2a | fase 2a | fase 2a |
| `TabList`, `Tab` | `AXTabGroup` (166); a tab na fase 1 | `Tab` (196), `TabItem` (195), `SelectionItem` (631-649) | `PageTabList` (233), `PageTab` (231) |
| `Tree`, `TreeItem` | `AXOutline` (174); `AXRow` (50) com subrole `AXOutlineRow` (281), selecionável (1030-1039) | `Tree` (204), `TreeItem` (68), `SelectionItem` | `Tree` (258), `TreeItem` (259), `Selected` (320-326) |
| `Table`, `Row` | `AXTable` (78), `AXRow` (47) | `Table` (95), `DataItem` (65); a linha não é `SelectionItem` ("TODO: tables (#29)", 632-634) | `Table` (232), `TableRow` (209) |
| `List`, `ListItem` | `AXList` (77), `AXGroup` (48) | `List` (94), `ListItem` (66), `SelectionItem` | `List` (171), `ListItem` (176) |
| `Link` | `AXLink` (46) | `Hyperlink` (64) | `Link` (170) |
| `Group` (breadcrumb, split, grupos da Button) | `AXGroup` (123), subrole `AXApplicationGroup` (256) | `Group` (144) | `Panel` (156) |
| `Label` (chips estáticos, a linha da Input) | `AXStaticText` (44) | `Text` (62) | `Label` (228) |
| `Status` (o eco do crumb) | fase 1 | fase 1 | fase 1 |

o que isso diz às páginas novas:

- `expanded` não é dito por nenhum adapter do lock: o macOS 0.26.1 não tem
  disclosure (grep "xpand" em `src/`: só o papel `DisclosureTriangle`,
  113), o windows não tem `ExpandCollapse` (fase 1) e o AT-SPI não põe
  estado de expandido (grep: zero). um ramo da Tree é ouvido pelo botão
  apresentado "toggle expanded" (fase 2a) e pelos filhos que aparecem.
- a linha selecionada da tabela: a árvore só marca filhos pelo `selected`
  de um contêiner (`engine/src/accessibility/tree.rs:445-469`), e a `Table`
  do comps guarda `selected_row` (`comps/src/content/table.rs:135-147`).
  correção na árvore, para o papel `Table`: `selected_row` marca a linha
  como `selected` marca o filho. o windows ainda não a diz (a linha não é
  `SelectionItem`); o macOS e o AT-SPI dizem.
- o `SplitPane` é um `Group` com `set_ratio` (`comps/src/nav/split_pane.rs:96-120`):
  o leitor não tem como pedir (a árvore só tem `SetValue` para `set_value`
  e o `select` do select, `engine/src/accessibility/action.rs:136-148`), e
  o papel que caberia (`Splitter` no AccessKit) não existe no `sem`, cujos
  tipos não mudam nesta fase. fica na lista `UNREACHABLE` com o motivo; a
  página diz a proporção como descrição do grupo, e o agente a pede por
  `set_ratio`.
- `active` de um `NavLink`, `active` de um `IconButton` de barra e `busy`
  do `SubmitButton` não são ditos: a árvore só lê `toggled`, `expanded`,
  `selected` e valor (`engine/src/accessibility/tree.rs:281-360`). fica
  para a próxima fase.

## fase 2b: o roteiro do VoiceOver (para o brenner rodar)

o que cada passo espera vem dos testes (`apps/uigallery/tests/access_tree.rs`,
`apps/uigallery/tests/component_pages/`, `view/dispatch/component/tests.rs`)
e do mapeamento do `accesskit_macos` 0.26.1 (sonda da fase 2b, (e)); a
frase exata é do VoiceOver, e o que ele disser de fato é o registro. o
preparo é o da fase 1 (`cargo run -p uigallery -- <página>`, VoiceOver com
Cmd-F5, VO é Control-Option). os passos das fases 1 e 2a seguem valendo;
estes são os das páginas de componente. nenhum passo pede o Claude.

| passo | o que fazer | o que ouvir |
|---|---|---|
| botões 1 | `-- Toast`, VO-Right a partir da janela | "Toast", região (landmark); "Neutral, button", "Success", "Error", "Info" |
| botões 2 | VO-Space em "Error" | "Failed to push: remote rejected." anunciado com prioridade alta; o cursor fica no botão |
| botões 3 | `-- Dialog`, VO-Space em "Delete repository…" | o diálogo "Delete repository?" abre com o foco em "Cancel" (fase 2a); a página some da navegação (inerte) |
| botões 4 | `-- Button`, andar pelos grupos | "Button, medium", grupo; dentro, "Button, button" com a descrição "Default, Hover, Press, Focus", e o do cartão "Disabled" dimmed; os de ícone pelo ícone ("x", "plus", "send"), o de envio "submit"; VO-Space num ligado não muda nada (é o que o clique faz) |
| botões 5 | com o foco do teclado seguindo o cursor, parar num botão | o anel de foco aparece no botão; Esc tira; um clique em outro lugar tira |
| botões 6 | `-- Tooltip`, parar em "Hover me" | "Hover me, button"; a descrição (VO-Shift-H, ou a espera) é o texto do tooltip, sem passar o mouse |
| botões 7 | `-- Spinner`, VO-Space em "Start spinners" | o botão passa a dizer "Stop spinners" e os arcos giram (eles mesmos não têm nó: grupo 2) |
| valores 1 | `-- Checkbox` | "Autosave on focus loss, checkbox, checked"; VO-Space: "unchecked"; "Managed by your organization" dimmed |
| valores 2 | `-- Switch` | "Focus mode", switch, ligado; VO-Space desliga e o knob anda |
| valores 3 | `-- Chip` | as etiquetas lidas como texto ("exact", "ann", ...); "click me", switch, desligado; VO-Space liga |
| valores 4 | `-- Slider`, "stepped" | "stepped, slider, 4"; VO-Shift-Up (ou as setas de interação): 5 |
| valores 5 | o slider "continuous" | "continuous, slider, 65"; ele não tem passo, a árvore só oferece `SetValue`: registrar se o VoiceOver consegue movê-lo (gap provável: o VoiceOver move slider por incremento) |
| valores 6 | `-- Input`, "Full name", digitar "Ada" | campo de texto; o valor "Ada"; mais abaixo, "live value: Ada" como texto |
| valores 7 | `-- Select` | "appearance, pop-up button, System default"; VO-Space abre; "Always light"; VO-Space escolhe e fecha |
| escolhas 1 | `-- Tabs` | "sections", grupo de abas; "Account, tab, selecionada, 1 de 3"; VO-Space em "About": selecionada |
| escolhas 2 | `-- Tree` | "files", outline; as linhas "src", "ui", "widgets.rs"... pelo nome; VO-Space em "main.rs": selecionada |
| escolhas 3 | na Tree, entrar em "assets" (fechado) | "toggle expanded, button" dentro da linha; VO-Space: os filhos aparecem. "expandido" não é dito (nenhum adapter do lock diz): registrar o que o VoiceOver fala |
| escolhas 4 | `-- Data Table` | "files", tabela; cada linha pelas células ("rust/engine/src/widgets/file_0.rs, modified, 3 KB, 0000000"); VO-Space numa linha: selecionada (novo na árvore) |
| escolhas 5 | `-- Scroll Area` | "items", lista; "Item 0", "Item 1"...; VO-Space: selecionada; rolar com o trackpad troca as linhas que a árvore tem |
| escolhas 6 | `-- Breadcrumb` | "path", grupo; "crates, link", ..., "table.rs" texto; VO-Space em "comps": "clicked crumb 1" anunciado, polido |
| escolhas 7 | `-- Navigation Menu` | "Workspace, link" com a descrição "1"; "History" com "4"; VO-Space em "Branches": a marca de ativo anda na tela (o "atual" não é dito: gap) |
| escolhas 8 | `-- Resizable` | "panes", grupo, "the first pane takes 35%"; nenhuma ação para o leitor (gap listado); arrastar com o mouse e voltar: a descrição muda |
| barra | na Button, abrir a barra de menu | só "plev" e "view": a página de componente não põe menu |
| sem projeção | `-- Card` e `-- Separator` | só a janela (grupos 2 e 3) |

no windows e no AT-SPI, quando o alvo rodar: a linha selecionada da
tabela não é dita no windows (a linha não é `SelectionItem`); o switch e o
chip são botão com `Toggle` no windows e `ToggleButton` no AT-SPI; o
slider se ajusta só por valor.

## handoff da fase 2b (2026-09-25)

branch `task/75c-a11y-component-pages`, sobre `main` em `3048b320`. sem
push.

### commits

1. `7fa4f84c` a sonda (seção "fase 2b: a sonda" acima).
2. `e5f738e5` o mecanismo (`Component` e a tabela), com as páginas Dialog
   e Toast.
3. `22819cc9` as páginas de botão: Button, Icon Button, Tooltip, Spinner.
4. `a64f9e93` as de alternância e valor: Checkbox, Switch, Chip, Slider,
   Input, Select.
5. `8f8cf475` as de coleção e navegação: Tabs, Tree, Data Table, Scroll
   Area, Breadcrumb, Navigation Menu, Resizable; a árvore diz a linha
   selecionada da tabela.
6. `a66e6e1d` a barra de menu deixa as páginas de componente de fora.
7. `5e0d2c2f` as promessas de privacidade, guardadas.
8. `3badee06` as 60 baselines.
9. este commit: roteiro, ADR e handoff.

### o mecanismo

`apps/uigallery/src/views/dispatcher/component/mod.rs`: o trait `Component`
(raiz, `project`, `run`, `focus`, `blur`), a tabela (uma macro, uma linha
por pane, 19 linhas) e o `Operable` que empresta o dispatcher do todo. a
view entra por quatro braços: `page_semantics`, `act`, `assist_focus` e o
Escape. a página se projeta sob uma região com o título da página, inerte
com um overlay por cima. o clique da pessoa fica no widget (a projeção
segue o estado); o que a página faz por cima do clique (modal, toast) é um
método só, `after_component`, chamado pelo clique e pelo pedido.

### as páginas, antes e depois

| grupo | antes | depois |
|---|---|---|
| projetam | 6 | 36: as 6 e 30 páginas de componente (19 panes) |
| grupo 1 (widget com projeção no comps) | 30 sem projeção | 0: todas projetam |
| grupo 2 (widget sem projeção) | 24 | 24, na lista `SILENT` com o que falta |
| grupo 3 (nada a operar) | 22 | 22, na lista `SILENT` com o motivo |

as listas de `apps/uigallery/tests/access_tree.rs`: `SILENT` 46 (era 76
sem motivo); `UNREACHABLE` 2 (a fase 2a deixou 1; entra `Resizable:
resizable_page.split set_ratio`); `FOCUS_NOT_PROJECTED` e `OFFERED_DISABLED`
seguem vazias com as 36 páginas.

três correções na página (nada no comps): o chip "click me" virou chip de
filtro (a projeção dizia botão); o rótulo do Spinner muda com o giro, não
no quadro; o trilho da Navigation Menu é um `NavLink` próprio sem id. uma
na árvore (`rust/engine/src/control/accessibility/tree/mod.rs`): a tabela diz a
linha selecionada por `selected_row`.

### os testes

- por tipo de widget, a pessoa, o leitor e um agente pelo port de verdade
  no mesmo estado (`apps/uigallery/tests/component_pages/`, 10 testes):
  botão (Toast, Dialog, Alert Dialog, Spinner), botão que não faz nada
  (Button, Icon Button, Tooltip), alternância (Checkbox, Switch, Chip,
  Toggle), slider (valor e passo), campo de texto, select, escolha numa
  coleção (Tabs, as duas tabelas, as duas listas, os dois menus de
  navegação, Breadcrumb), árvore (folha e ramo), split (pessoa e agente; o
  leitor não pede), e o foco que o leitor deixa (Escape e clique).
- o mecanismo (`view/dispatch/component/tests.rs`, 4): a região com o
  título, o dispatcher da galeria ouvindo leitor e agente, a página inerte
  sob o modal.
- a árvore (`accessibility/tree/tests.rs`): a linha selecionada da tabela.
- a barra de menu (`app/menu.rs`): `the_bar_leaves_the_component_pages_out`.
- as promessas (`component_pages/promises.rs`).
- consequência: `app/access.rs`, `app/headless.rs` e
  `apps/app/semantics/assistive.rs` usavam a Button como página sem
  projeção e passam a usar a Card; `tests/view_source.rs` troca o texto
  formatado da Data Table (agora dentro de uma linha ligada, que leva à
  projeção dela) pelo "step 1: 4" da Slider.

### a barra de menu

deixa as páginas de componente de fora (`view.is_component`), como já
deixa o shell e o overlay: numa página de componente ela mostraria
amostras ("Button" 30 vezes). a Toggle Group segue com a barra da todo.

### as promessas

nenhum caso no escopo: nenhum segredo, nenhuma câmera, nenhuma ação só da
pessoa, nenhum estado só do leitor. o teste guarda a ausência e confere que
o que sai pelo port é o que a página projeta.

### as baselines

60, as 30 páginas em core e hoff: alert-dialog, breadcrumb, button,
button-group, checkbox, chip, combobox, data-table, dialog, field,
hover-card, icon-button, input, item, kbd, label, navigation-menu,
popover, resizable, scroll-area, select, slider, spinner, switch, table,
tabs, toast, toggle, tooltip, tree (`@core@1280x800` e `@hoff@1280x800`).
em cada uma: um `semantics.json` novo; no `scene.json`, o bloco
`semantics` (de `null` para revisão e contagem) e o bloco `bindings`; e o
nome da camada da página, que o regress tira das raízes ligadas nela
(o espelho das ligações, ver a sonda (c)). nenhum `frame.png` existe
nelas. as outras 116 baselines, as seis páginas que já projetavam e os
goldens do `uir` não mudaram.

a confirmação por script (compara o `scene.json` de antes e de depois,
camada a camada na ordem de composição: z, hash, contagem de nós e limites
iguais; o nome só muda de `z<z>` para a junção das raízes ligadas na
camada; as ligações e a projeção são novas; o `semantics.json` tem a
contagem do `scene.json`; nenhum `frame.png`):

```text
60 baselines checked
  layer z0 -> breadcrumb_page: 2
  layer z0 -> button_page: 4
  layer z0 -> checkbox_page: 2
  layer z0 -> chip_page: 4
  layer z0 -> dialog_page: 4
  layer z0 -> icon_button_page: 2
  layer z0 -> input_page: 6
  layer z0 -> navigation_menu_page: 4
  layer z0 -> resizable_page: 2
  layer z0 -> select_page: 4
  layer z0 -> slider_page: 2
  layer z0 -> spinner_page: 2
  layer z0 -> switch_page: 2
  layer z0 -> table_page: 4
  layer z0 -> tabs_page: 2
  layer z0 -> toast_page: 2
  layer z0 -> tooltip_page: 6
  layer z0 -> tree_page: 2
  layer z10 -> scroll_area_page: 4
frame.png touched: False
problems: 0
```

os diffs inteiros, um por tipo de widget, estão no anexo no fim deste
arquivo.

### gate e regress

- antes (o briefing): 3086 passam, 27 ignorados, 176 baselines.
- depois, em `3badee06`: `./tool/tasks/fullcheck.sh` verde, pernas urnaui e wasm
  rodaram; 3103 passam (+17: 3018 no workspace, 85 no urnaui), 27
  ignorados, 0 falhas. `regress check`: 176 baselines, 176 iguais (antes
  do bless: 60 mudadas, as desta fase).
- `Cargo.toml` e `Cargo.lock` intocados (md5 do lock igual), nenhuma
  dependência nova, AccessKit na mesma versão; `rust/engine/src/widgets`,
  `rust/plevir`, `rust/screen`, `rust/accord`, `rust/socket/src`,
  `rust/offers` e `apps/idestudio/src` intocados. sempre com `PLEV_CLAUDE`
  no stub e `--offline`; nenhuma janela, nenhum leitor ligado, nenhum
  clique sintético por coordenada de tela, nenhum diálogo nativo.

### linha para o índice de ADRs

| 2026-09-25 | [accessibility-is-production-ready-per-platform](../../adr/interface/0068-accessibility-is-production-ready-per-platform.md) | proposed | fase 2b: as páginas de componente cujos widgets se projetam no comps projetam o que mostram (30 páginas, 19 panes), por um trait e uma tabela, sob uma região com o título da página, cada pedido pelo dispatcher da galeria; o clique da pessoa fica no widget e a projeção segue o estado; a pessoa, o leitor e um agente pelo port chegam ao mesmo estado; a barra de menu as deixa de fora; a árvore diz a linha selecionada da tabela; as 46 páginas que ficam, cada uma com o motivo (task-75 fase 2b) |

### o que fica para a próxima fase

- o grupo 2 (24 páginas): a projeção no comps de `Avatar`, `Badge`,
  `Stat`, `ProgressBar`, `Spinner`, `Skeleton`, `CodeBlock`, `Card`, os
  gráficos, `GraphView`, `EmptyState`, `Sidebar`, `PanelHeader`,
  `EditorView`, e o que é desenhado à mão (os cartões de tema, a área do
  menu de contexto, o dock, o builder); a Git espera o ide. o que cada um
  precisa está na sonda (a).
- o teclado completo nos widgets: Tab na ordem da projeção (hoje só as
  páginas com foco próprio andam com Tab), setas em listas, abas, árvore e
  menus, o trap de foco no modal, o anel nas pílulas.
- o que a árvore ainda não diz: `active` de um link de navegação e de um
  botão de barra, `busy` do botão de envio; `expanded`, que nenhum adapter
  do lock diz; a proporção do split como valor que o leitor ajusta (pede
  um papel de splitter no contrato).
- o slider contínuo no VoiceOver (sem passo, só `SetValue`): registrar no
  roteiro.
- bounds para os controles apresentados (fase 2a).
- windows, AT-SPI, iOS e android seguem esperando o alvo ou o adapter.

## anexo da fase 2b: os diffs das baselines, por tipo de widget

o agente colou aqui um diff inteiro por tipo de widget (cerca de 3100
linhas de JSON); o orquestrador tirou na integração, porque o diff mora no
git: o `semantics.json` de cada página é arquivo novo e o `scene.json` muda
nos blocos `semantics`, `bindings` e no nome das camadas. para ler um tipo:
`git show 3badee06 -- tool/bench/regress/baselines/<página>@core@1280x800/`.
as páginas escolhidas, uma por tipo:

- botão (Button, IconButton, SubmitButton): `spinner@core@1280x800`
- checkbox: `checkbox@core@1280x800`
- switch: `switch@core@1280x800`
- chip: `chip@core@1280x800`
- slider: `slider@core@1280x800`
- campo de texto: `input@core@1280x800`
- select: `select@core@1280x800`
- tabs: `tabs@core@1280x800`
- árvore: `tree@core@1280x800`
- tabela: `data-table@core@1280x800`
- lista virtual: `scroll-area@core@1280x800`
- breadcrumb: `breadcrumb@core@1280x800`
- link de navegação: `navigation-menu@core@1280x800`
- split: `resizable@core@1280x800`
