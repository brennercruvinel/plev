---
project: plev
audience: [ai-agents, contributors]
status: ongoing
last-updated: 2026-09-25
domain: generative-runtime
task: task-75
wave: trilha
depends: [task-60]
branch: task/75-a11y-platforms
---

# task-75: accessibility de plataforma production-ready

## objetivo

a accessibility que a task-60 projeta do contrato passa a funcionar bem
com os leitores de tela reais de cada plataforma, com navegação de teclado
completa e com as semânticas de foco e ação que um produto grande exige
para poder adotar o plev.

## ligação com a tese

seção 21 (esse ponto sozinho mata frameworks alternativos; estar presente
e estar production-ready são coisas diferentes; VoiceOver, NVDA, TalkBack,
keyboard navigation, focus semantics e action semantics são cidadãos do
runtime).

## estado atual no código

- depois da 60: árvore AccessKit a partir do snapshot, `NodeId` estável,
  ações do leitor pelo dispatcher, ordem de foco semântica, ponte no
  showcase, verificado com VoiceOver no macOS.
- AccessKit 0.24 com `accesskit_winit` 0.32. alvos do plev: macOS e web
  shipando; android e ios em andamento (tasks 18 e 19); linux e windows
  pendentes.
- web: AccessKit não tem adapter para navegador.
- recipe de focus ring em `crates/comps/src/recipe.rs`; reduced motion e
  contraste wcag declarados como requisitos nativos no AGENTS.md.

## escopo

1. matriz por plataforma, com o suporte real do AccessKit verificado (não
   presumido): VoiceOver no macOS e no iOS, TalkBack no Android, NVDA e
   Narrator no Windows, Orca no Linux (AT-SPI). onde o adapter não existir
   ou não servir, o registro diz isso.
2. web: decisão por ADR entre um espelho DOM oculto com ARIA gerado a
   partir do snapshot e nenhuma accessibility no web por enquanto; se o
   espelho for o caminho, ele é outra projeção do mesmo snapshot, nunca uma
   segunda fonte.
3. teclado: Tab e Shift-Tab pela ordem semântica, setas em listas, menus,
   tabs e árvores (padrões WAI-ARIA), trap de foco em modal e retorno do
   foco ao fechar, atalhos anunciados.
4. live regions: toast e status anunciados sem roubar foco.
5. reduced motion (a preferência do sistema desliga transições e springs)
   e contraste (verificação dos pares de token de texto e fundo contra
   wcag AA nos temas).
6. roteiros manuais por plataforma, executados e com resultado registrado
   neste arquivo ou num anexo.

## fora de escopo

accessibility de texto rico (74, fase 9); bring-up de linux e windows
como alvo (arc: pendente; esta task só entra lá quando o alvo existir).

## dependências

60 integrada; por plataforma, o alvo precisa rodar.

## superfície sob responsabilidade

`crates/engine/src/accessibility/**`, os shells dos apps na parte da
ponte, `crates/comps` na parte de teclado e foco dos widgets, testes de
contraste em `crates/engine/src/theme` (só testes), ADRs por decisão.

## interfaces que consome

a ponte e o construtor de árvore (60), `Snapshot` (57), dispatcher (58).

## interfaces que pode alterar

`crates/engine/src/accessibility` por inteiro; widgets de comps na parte de
teclado com aprovação.

## critérios de aceitação

- para cada plataforma com alvo rodando: roteiro executado com o leitor
  real sobre o showcase (todo, formulários, modal, tabs, lista) e o ide,
  resultado registrado.
- testes automáticos de árvore para toda página do showcase.
- teste de contraste dos tokens de texto nos temas.
- teclado completo nas páginas de componente, com teste.
- `./script/gate` verde.

## testes esperados

asserções de árvore por página, navegação de teclado simulada, contraste,
reduced motion desligando animação.

## riscos e perguntas abertas

- suporte do AccessKit por plataforma muda por versão; atualizar a versão é
  decisão com aprovação.
- teste manual não roda em CI; o registro é a evidência.

## commits esperados

um grupo por plataforma e por item (teclado, live regions, reduced motion,
contraste), com `task-75:` no início.

## pare e reporte se

- o web exigir o espelho DOM (decisão por ADR);
- um leitor real se comportar diferente do que a árvore promete.

## fase 1: a sonda (2026-09-25)

o que segue foi lido na fonte das versões travadas no `Cargo.lock`
(accesskit 0.24.0, accesskit_winit 0.32.2, accesskit_macos 0.26.1,
accesskit_windows 0.32.1, accesskit_unix 0.21.1 com
accesskit_atspi_common 0.18.1, accesskit_consumer 0.35.0 para o windows e
0.36.0 para o macos e o unix, winit 0.30.13), no registry local do cargo.
cada afirmação traz arquivo e linha; nenhuma vem de documentação.

### (a) a matriz por plataforma

quem escolhe o adapter é o `accesskit_winit` 0.32.2, em
`src/platform_impl/mod.rs`: windows (9-11), macos (13-15), linux e os bsd
com a feature `accesskit_unix` (17-28), android só com a feature
`accesskit_android` (30-32), e todo o resto cai em `null.rs` (34-50), um
adapter que não faz nada (`null.rs:21`, `update_if_active` vazio). o
workspace pede `accesskit_winit = "0.32"` com as features padrão
(`Cargo.toml:128`), que ligam `accesskit_unix` e não ligam
`accesskit_android`.

| plataforma | adapter no lock | papéis | ações | live regions | foco | texto |
|---|---|---|---|---|---|---|
| macOS | `accesskit_macos` 0.26.1 | `ns_role` em `node.rs:36-231` e `ns_sub_role` em `node.rs:233+`, todos os papéis do AccessKit; `Status` vira grupo com subrole `AXApplicationStatus` (161, 273), `Alert` grupo com `AXApplicationAlert` (90, 238), `Switch` checkbox com subrole switch (82, 275), `Tab` radio com subrole tab (165, 276), `ComboBox` popup button (102), `ListItem` grupo (48) | `Click` por `accessibilityPerformPress` (668), `accessibilityPerformPick` (1112) e `setAccessibilitySelected` (1041); `Focus` por `setAccessibilityFocused` (638); `SetValue` por `setAccessibilityValue` (555); `Increment` e `Decrement` (687, 706); `ScrollIntoView` por `AXScrollToVisible` (1180-1205); custom actions: nenhuma ocorrência em `src/` | anúncio `NSAccessibilityAnnouncementRequestedNotification` na janela (`event.rs:65-101`); `Assertive` vira prioridade alta, o resto média (`event.rs:35-44`); dispara quando um nó live entra (`event.rs:236-239`) ou muda de value (`event.rs:298-306`), e só se o nó tem **value** | `NSAccessibilityFocusedUIElementChangedNotification` quando o foco da árvore muda (`event.rs:133-138`, `314-321`); o foco da janela chega por `process_event` (`accesskit_winit` `platform_impl/macos.rs:48-54`) | text ranges (`node.rs:730-1006`) só sobre nós `TextRun`; o plev não emite `TextRun`, então um campo expõe o value inteiro, sem caret nem seleção |
| iOS | nenhum | nenhum | nenhuma | nenhuma | nenhum | nenhum |
| android | nenhum no lock: `accesskit_android` 0.7.2 é dependência opcional do `accesskit_winit` (`Cargo.toml:111-114` do crate), e a feature está desligada | (o adapter existiria em `platform_impl/android.rs`, injetado no `mSurfaceView` da GameActivity, 27-40) | nenhuma | nenhuma | nenhum | nenhum |
| windows | `accesskit_windows` 0.32.1 | `control_type` em `node.rs:55+`; `Status` vira `StatusBar` (191), `Alert` vira `Text` (102), `Switch` vira `Button` (99), `Tab` vira `TabItem` (195) | patterns em `node.rs:1260-1420`: `Invoke` (`Click`, 1268), `Toggle` (1261, só com toggled e sem selection item, 559-561), `Value` (`SetValue` com texto, 1273-1282), `RangeValue` (`SetValue` numérico, 1284-1296), `SelectionItem` (1310), `ScrollItem` (1298); `Focus` por `SetFocus` (1067); sem `Increment`, `Decrement` e `ExpandCollapse` (grep: zero) | propriedade `LiveSetting` (`node.rs:534-540`, 1248); evento `UIA_LiveRegionChangedEventId` quando o nó live entra (`adapter.rs:225-232`) ou muda de **nome** (`adapter.rs:271-281`) | `UIA_AutomationFocusChangedEventId` (`adapter.rs:32`, 298) | `Text` pattern (1357) sobre `TextRun` |
| linux (AT-SPI) | `accesskit_unix` 0.21.1 sobre `accesskit_atspi_common` 0.18.1 | `node.rs:65+`; `Status` vira `StatusBar` (229), `Alert` vira `Notification` (71), `Switch` vira `ToggleButton` (93), `Tab` vira `PageTab` (231) | interface Action com uma ação só, `click`, quando o nó é clicável (`node.rs:467-480`, `do_action` 980-990); `Focus` por `grab_focus` (1044-1052); `SetValue` numérico por `set_current_value` (1604-1611); sem `SetValue` de texto (nenhuma EditableText), sem `Increment` e `Decrement` | `ObjectEvent::Announcement(nome, politeness)` quando o nó live entra (`adapter.rs:71-76`) ou muda de **nome** (`node.rs:543-556`) | `StateChanged(Focused)` (`adapter.rs:322-340`) | interface Text sobre `TextRun` |
| web | nenhum: o AccessKit do lock não tem adapter de navegador | o `Bridge` do engine não compila em wasm32 (`accessibility/mod.rs:13-14`, `bridge.rs:16`) | nenhuma | nenhuma | nenhum | nenhum |

o que isso diz ao plev:

- só o macOS está servido hoje, e é o único alvo que roda com o
  VoiceOver. windows e linux têm adapter completo no lock, mas o alvo
  ainda não existe (arc: pendente). iOS e android não têm adapter nesta
  versão: ligar o do android é uma crate nova no lock (fora desta fase) e
  o do iOS não existe no `accesskit_winit` 0.32.2.
- o anúncio de live region diverge por plataforma: o macOS fala o
  **value** do nó, o windows e o AT-SPI falam o **nome**. uma status que
  guarda a mensagem num estado e tem nome fixo (`silences.status`,
  `surface_page.status`) só pode ser ouvida nos três se nome e value
  mudarem juntos.
- nenhum adapter do lock implementa custom actions, embora o contrato do
  AccessKit 0.24 as tenha. um nó com várias ações (o corte da Silences:
  `preview_segment`, `apply_cut`, `reject_cut`) só chega ao leitor se a
  página projetar cada ação como um controle próprio.
- o windows e o AT-SPI não têm `Increment` e `Decrement`: um slider lá se
  ajusta só por `SetValue`.

### (b) a árvore de hoje

rodei a projeção de cada página da galeria (82, a tabela inteira de
`view/pages/table.rs`), montei a árvore como a ponte monta
(`accessibility::build` sobre um frame desenhado) e percorri cada nó.

- 76 das 82 páginas não projetam nada: o leitor ouve a janela e só. as 6
  que projetam são Toggle Group (mostra o exemplo do todo), Todo,
  Surface, Silences, Files e Capture. `ShowcaseView::semantics`
  (`view/dispatch.rs:358`) só conhece cinco panes; toda página de
  componente fica de fora.
- sete botões focáveis sem `Click`: `surface_page.propose`,
  `silences.detect`, `silences.apply_all`, `silences_surface.propose`,
  `silences_surface.close`, `files.open`, `files.warn`. a ação deles tem
  nome próprio (`propose`, `detect_silence`, `apply_all`, `close`,
  `open_file`, `warn`) e o `action::primary` só aceitava `activate`,
  `toggle`, `submit` e `select`. o leitor focava e não conseguia apertar.
- dois grupos sem nome na quick todo da Surface (`todo_panel.entry`,
  `todo_panel.counts`): o leitor para em "grupo" e não ouve nada.
- as linhas de status com a mensagem num estado (`silences.status`,
  `surface_page.status`) chegam à árvore só com o nome ("status",
  "proposal"); a mensagem nunca era lida.
- o foco: a ordem do Tab e a ordem em que o leitor anda batem em todas as
  páginas, e toda página aceita o foco que o leitor pede. mas treze
  controles aceitam o foco e não o projetam de volta (os filtros do todo,
  os filtros e checkboxes da quick todo, os selects de proposer, o detect
  e o apply all da Silences): a árvore continua dizendo que o foco é da
  janela.
- ações declaradas que nenhuma ação da árvore alcança: o `remove` de cada
  linha do todo, as três ações de cada corte da Silences, o `select` com
  parâmetro, o `open` e o `close` dos selects de proposer, o `drop_files`
  da Files.
- o campo do todo (`todo.input`, e o da quick todo) oferece `Click`
  enquanto o `submit` está desligado (campo vazio): o dispatcher recusa e
  o leitor não ouve nada acontecer. a task-60 fez do `submit` o aperto do
  campo, e o teste da página conta com esse `Click`, então fica como está.
- papel: a linha do todo (`todo.item[n]`) é um `ListItem` que alterna; o
  macOS mapeia `ListItem` para grupo e manda o toggled como value
  booleano. o certo é a linha conter um checkbox, como a quick todo faz.
  os itens da quick todo são grupos com o nome "1", "2", "3" (a chave).
  `capture.level` é uma barra de progresso cujo número está em `level`, e
  a árvore só lê `value`.
- nenhum nó sem nome entre os controles, nenhum controle interativo fora
  do foco, nenhuma ação oferecida que o dispatcher recuse por política.

### (c) live regions

- o AccessKit 0.24 tem `Live` com `Off`, `Polite` e `Assertive`
  (`accesskit` `lib.rs:534-538`, setter em 2064), e o consumer herda o
  live do pai (`accesskit_consumer` 0.36.0 `node.rs:897-901`).
- hoje nada no engine liga live: nenhuma status nem alerta é anunciada
  sem foco.
- o toast não está na árvore. o `ToastManager` do comps sabe se projetar
  (`feedback/toast.rs:161`, `Status` ou `Alert` por intent), mas os toasts
  da galeria (`ShowcaseView::toasts`) não entram em
  `ShowcaseView::semantics`.
- `shell:notice` não está na árvore: é estado da região `shell` em
  `gallery_semantics` (`view/dispatch/shell.rs:218`), e a ponte monta a
  árvore de `view.semantics()`, a página sozinha (`app/pace.rs:55`).
- as status da Surface e da Silences estão na árvore, sem live e sem a
  mensagem (ver (b)).

### (d) reduced motion

- o winit 0.30.13 não expõe a preferência: nada em `src/` fala de reduzir
  movimento; o que ele dá é `WindowEvent::Focused(bool)`
  (`event.rs:197`), útil para reler quando o app volta, e
  `ThemeChanged` (`event.rs:397`), só do tema escuro.
- macOS sem dependência nova: processo filho
  `/usr/bin/defaults read com.apple.universalaccess reduceMotion`.
  conferido nesta máquina (macOS 26.6.2): o domínio lê (163 chaves), a
  chave não existe porque nunca foi ligada, e o `defaults` sai com
  código 1 e "does not exist", em uns 10 ms. ligada, ele imprime `1`. o
  `objc2-app-kit` não é dependência direta do engine.
- windows: `SystemParametersInfo` com `SPI_GETCLIENTAREAANIMATION` pede a
  crate `windows`, que o engine não tem direta. linux: `gsettings get
  org.gnome.desktop.interface enable-animations` como processo filho
  daria, mas o alvo não roda. web: `matchMedia("(prefers-reduced-motion:
  reduce)")` pede a feature `MediaQueryList` do `web-sys`, que o engine
  não declara (`Cargo.toml:123`; o winit liga a dele, `Cargo.toml:377` do
  crate, e contar com isso é depender por acaso). iOS e android: UIKit e
  JNI, nenhum dos dois é dependência do engine. os cinco ficam para
  quando o alvo rodar, ou para uma mudança de `Cargo.toml` aprovada.
- o ponto único de animação existe: `engine::animation`. toda transição
  do workspace passa por `Spring::tick` ou `Tween::tick`: no comps o knob
  do switch, o fade da scrollbar, o toast e o overlay (modal e menu); no
  `surface` o `motion.rs`; na galeria o todo, o dock e o reveal do chart;
  no engine o scroll suave de `input/scroll.rs`. os laços ambientes
  (ângulo do spinner, pulso do skeleton, `comps` `feedback/spinner.rs:75`
  e `content/skeleton.rs:48`) somam `dt` por conta própria e ficam de
  fora: são do comps.
- a ponte do engine (`Bridge`) já vê todo evento da janela, é criada na
  janela principal da galeria e na do app genérico do engine
  (`window/lifecycle.rs:81`), e roda na thread da interface. é o lugar de
  ler a preferência sem mexer no shell.

### (e) contraste

301 pares de texto e fundo nos quatro temas (core, hoff, dark, light),
medidos como a janela mostra (token translúcido composto sobre o fundo em
luz linear, como a GPU mistura na superfície sRGB). 102 falham o AA. a
lista inteira, com a razão de cada par, está em
`crates/engine/src/theme/tests_contrast.rs` (`KNOWN`) e no handoff
abaixo.
