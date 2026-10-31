---
id: task-75/phase-1-probe-and-scripts
title: "task-75 phase 1: the probe and the manual test scripts"
project: plev
type: section
parent: task-75
status: reference
date: 2026-09-25
date_basis: text
recorded: 2026-09-23
topics: [accessibility, platform, semantics, design-system]
relations: [adr-0068, task-60]
evidence: [1d2aa2eb, 20096a0f, ab951681, e7ca19c1, d811876f]
source: docs/log/steps/task-75-platform-accessibility.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase 1: the probe (2026-09-25)

Records phase 1 of task-75: the AccessKit probe against the source of the versions pinned in `Cargo.lock`, the per-platform matrix, the tree of each gallery page, the live regions, contrast and reduced motion. The result is that only macOS runs with VoiceOver today: windows and linux have an adapter in the lock with no target, and iOS, android and web have no adapter in this version. The manual test scripts are left for Brenner to run, because no agent turns on a screen reader; 102 of the 301 contrast pairs fail AA and stay unchanged at the token level, by design decision.

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
`rust/engine/src/styling/theme/tests/contrast.rs` (`KNOWN`) e no handoff
abaixo.

## fase 1: roteiros manuais

### VoiceOver no macOS (para o brenner rodar)

o que cada passo espera vem da árvore que os testes provam
(`apps/uigallery/tests/access_tree.rs`, `app/access.rs`) e do
mapeamento do `accesskit_macos` 0.26.1; a frase exata é do VoiceOver, e
o que ele disser de fato é o registro. onde a coluna diz "gap", é o
comportamento esperado hoje e está na lista da fase 2.

preparo: `cargo run -p uigallery -- Todo` (ou a página do passo), VoiceOver
com Cmd-F5, VO é Control-Option. o rotor (VO-U) ajuda a listar controles.
nenhum passo pede o Claude: na Surface, nunca aperte propose com o
proposer em `claude` (é chamada paga); troque para `rules` com o mouse
antes.

| passo | o que fazer | o que ouvir |
|---|---|---|
| todo 1 | VO-Right a partir da janela | "todo", região (landmark) |
| todo 2 | chegar no campo | "new todo", campo de texto, vazio |
| todo 3 | VO-Space no campo vazio | nada acontece (o `submit` está desligado; gap conhecido: o aperto é oferecido e recusado) |
| todo 4 | focar o campo, digitar "buy milk", VO-Space | a linha nova aparece e o contador é **anunciado sem o foco sair do campo**: "3 active · 1 done" (novo nesta fase) |
| todo 5 | andar até os filtros | "All", tab, selecionada, 1 de 3; "Active"; "Completed" |
| todo 6 | andar até uma linha e VO-Space | a linha alterna e o contador anuncia a nova conta; registrar como o VoiceOver lê o estado da linha (é um grupo com value booleano: pode sair "0"/"1" ou nada; gap de papel, a linha deveria conter um checkbox) |
| todo 7 | andar até o contador | "2 active · 1 done", status; registrar se o texto sai duas vezes (nome e value são o mesmo texto) |
| formulário 1 | `-- Silences`, andar até "threshold (dB)" | slider, "-30"; VO-Shift-Up e Down (ou as setas de interação) mudam o valor e ele é lido |
| formulário 2 | "detect silences", VO-Space | aperta (novo nesta fase: antes o botão só focava); a linha de status anuncia a mensagem da detecção, polida |
| formulário 3 | "apply all", VO-Space | aplica; "silences.counts" anuncia a nova conta |
| formulário 4 | os cortes da lista | cada um lido pelo intervalo ("0:03.0 to 0:05.0"); aplicar ou rejeitar um corte pelo leitor não existe (gap: várias ações, nenhum adapter tem custom actions) |
| formulário 5 | `-- Surface`, andar até "intent", digitar | campo de texto; "proposer", popup button, "claude"; VO-Space no proposer não abre a lista (gap) |
| formulário 6 | `-- Files`, "open file", VO-Space | abre o diálogo do sistema (novo nesta fase); cancelar volta; a linha de status anuncia |
| toast 1 | `-- Toast`, com o cursor do VoiceOver em qualquer lugar, clicar "Success" com o mouse | "Saved! Everything went fine." anunciado polido; o cursor do VoiceOver não se move (novo) |
| toast 2 | clicar "Error" | "Failed to push: remote rejected." anunciado com prioridade alta (assertive), por cima do que estiver sendo lido |
| modal 1 | `-- Dialog`, abrir o diálogo com o mouse | gap: o diálogo não é projetado, o VoiceOver só tem a janela |
| modal 2 | confirmar com o mouse | "Settings applied." anunciado (é um toast) |
| tabs | `-- Tabs` | gap: a página de componente não projeta nada; ouvir só a janela. as tabs com árvore são os filtros do todo (todo 5) |
| lista | `-- Todo`, VO-Right pelas linhas | cada linha pelo texto, na ordem da tela; o foco do Tab e o do leitor andam na mesma ordem |
| ide | `cargo run -p idestudio`, ou a página Git da galeria | gap: o ide não liga a ponte, a página Git não projeta; só a janela |
| reduced motion | Ajustes, Acessibilidade, Tela, Reduzir movimento; voltar à janela do showcase | um toast novo aparece sem deslizar, o modal abre sem fade, o knob do switch pula; desligar e voltar à janela: o movimento volta. com `RUST_LOG=info`, o log diz `accessibility: reduce motion on` e `off` |

### as outras plataformas

- windows (NVDA, Narrator): quando o alvo rodar. conferir o anúncio das
  status: lá o que se anuncia é o nome, então `silences.status` e
  `surface_page.status` ficam mudas até o nome seguir a mensagem.
- linux (Orca): quando o alvo rodar. mesma nota do nome; lá o slider só
  se ajusta por valor, sem incremento.
- iOS (VoiceOver): sem adapter no `accesskit_winit` 0.32.2; quando houver.
- android (TalkBack): quando o adapter entrar no lock (decisão com
  aprovação) e o alvo rodar.
- web: depende do ADR `accessibility-is-production-ready-per-platform`
  (espelho DOM ou nada por enquanto).

## handoff da fase 1 (2026-09-25)

branch `task/75a-a11y`, sobre `main` em `bd03459d`. sem push.

### commits

1. `1d2aa2eb` a sonda e a matriz, pela fonte do lock (seção "fase 1: a
   sonda" acima).
2. `20096a0f` os testes de árvore de todas as páginas
   (`apps/uigallery/tests/access_tree.rs`) e as duas correções no
   `accessibility`.
3. `ab951681` live regions: status polida, alerta assertivo, o toast na
   árvore do leitor, o foco parado.
4. `e7ca19c1` contraste dos quatro temas contra o AA
   (`rust/engine/src/styling/theme/tests/contrast.rs`).
5. `d811876f` reduced motion no ponto único de animação, lido pela ponte.
6. este commit: roteiros, ADR e handoff.

### a sonda e a matriz

a matriz está na seção (a) acima. em uma linha: macOS, windows e linux
têm adapter no lock e só o macOS roda; iOS e web não têm adapter;
android tem um opcional e desligado, fora do lock. nenhum adapter tem
custom actions; windows e AT-SPI anunciam o nome de uma live region, o
macOS anuncia o value.

### a árvore: o que estava incoerente e o que foi corrigido

82 páginas, 6 projetam (Toggle Group, Todo, Surface, Silences, Files,
Capture), 76 mostram só a janela. corrigido em
`rust/engine/src/control/accessibility/**`:

- sete botões que o leitor focava e não apertava (`surface_page.propose`,
  `silences.detect`, `silences.apply_all`, `silences_surface.propose`,
  `silences_surface.close`, `files.open`, `files.warn`): um botão, link
  ou item de menu sem nenhuma das quatro ações primárias e com uma única
  ação sem parâmetro é apertado por ela (`action::primary`).
- os grupos sem nome da quick todo (`todo_panel.entry`,
  `todo_panel.counts`) viram `GenericContainer`, que os adapters tiram da
  árvore da plataforma subindo os filhos.
- as status com a mensagem num estado agora dizem a mensagem (ver live
  regions).

os testes (6 em `access_tree.rs`) seguram: árvore válida em toda página;
todo controle com nome e foco; nenhum grupo mudo; a ordem do Tab igual à
do leitor; a página aceita o foco pedido; toda ação oferecida vira um
pedido que o dispatcher admite de `Actor::Assistive`; toda status é live
e diz suas palavras. nada executa pedido (a Files abriria o diálogo do
sistema, a Surface chamaria o modelo).

### a lista para a fase 2 (comps e páginas; não mexi)

- as 76 páginas sem projeção, todas as de componente. `ShowcaseView::
  semantics` conhece cinco panes.
- ações que nenhuma ação da árvore alcança (lista exata `UNREACHABLE` no
  teste): o `remove` das linhas do todo; `preview_segment`, `apply_cut` e
  `reject_cut` de cada corte da Silences; `select`, `open` e `close` dos
  selects de proposer (as opções não são nós); `drop_files` da Files. a
  saída é a página projetar cada ação como um controle, porque custom
  actions não existem nos adapters do lock.
- foco aceito e não projetado (lista `FOCUS_NOT_PROJECTED`): os filtros
  do todo, os filtros e checkboxes da quick todo, os dois selects de
  proposer, `silences.detect` e `silences.apply_all`.
- o campo do todo e o da quick todo oferecem o aperto (`submit`) com o
  campo vazio, e o dispatcher recusa (lista `OFFERED_DISABLED`); a
  task-60 fez desse aperto o caminho do VoiceOver e o teste da página
  conta com ele.
- papel: `todo.item[n]` é um `ListItem` que alterna (deveria conter um
  checkbox); os itens da quick todo são grupos com nome "1", "2", "3";
  `capture.level` guarda o número em `level`, e a árvore só lê `value`.
- `silences.status` e `surface_page.status` têm nome fixo ("status",
  "proposal"): no windows e no AT-SPI, que anunciam o nome, ficarão
  mudas até o nome seguir a mensagem.
- o toast deveria estar na projeção da própria view (hoje é o shell que o
  põe na árvore do leitor, `app/access.rs`); `shell:notice` só vira live
  region mudando a projeção do shell da view.
- o modal e o menu de contexto (overlays) não são projetados.
- os laços ambientes do comps (spinner, pulso do skeleton) não leem a
  preferência de movimento.
- o teclado completo nos widgets (o escopo original da fase 2).

### live regions

`Status` é live `Polite` e `Alert` é `Assertive` (`tree::live`); o foco
do update nunca muda por elas (teste
`an_announcement_never_moves_the_focus`). o que se ouve é o value: o do
nó, senão o estado `message`, senão o label; mensagem vazia não diz
nada; um nó secreto não é live. o nome continua sendo o label, porque o
windows e o AT-SPI anunciam o nome. o toast: `app/access.rs::shown` põe
os toasts na tela, como o `ToastManager::project` do comps os declara,
no snapshot que a ponte recebe (`app/pace.rs` chama `shown` no lugar de
`view.semantics()`), só enquanto há toast; sem toast o snapshot é o da
página, idêntico. testes: 4 no engine (`tree/tests.rs`), 4 em
`app/access.rs` (toast polido e erro assertivo com o foco no campo, o
contador do todo anunciando a conta nova, toast numa página sem
projeção, página intocada sem toast) e 1 por página em `access_tree.rs`.
efeito colateral do macOS a registrar no roteiro: um nó live que aparece
é anunciado, então trocar de página anuncia as status dela uma vez.

### contraste

301 pares, 102 falham (core 36, hoff 15, dark 11, light 40). razão
medida como a janela mostra (composição em luz linear). nenhum token
mudou. por tema, texto e fundo (texto normal precisa de 4,5; gradiente
de título, intents, acento, borda ativa e o botão ghost, que desenha
ícone, precisam de 3):

| tema | frente | fundos e razão |
|---|---|---|
| core | `colors.text_mid` | bg 4.13, surface 4.13, bg_panel 3.96, bg_hover 3.75, popover 4.13, tooltip 4.43 |
| core | `colors.text_dim` | bg 4.13, surface 4.13, bg_panel 3.96, bg_hover 3.75, popover 4.13 |
| core | `glass.text_default` | bg 4.13, surface 4.13, bg_panel 3.96, bg_hover 3.75, popover 4.13 |
| core | `glass.text_faint` | bg 4.13, surface 4.13, bg_panel 3.96, bg_hover 3.75, popover 4.13 |
| core | `glass.text_placeholder` | bg 1.61, surface 1.61, bg_panel 1.61, bg_hover 1.60, popover 1.61 |
| core | `colors.success` | bg 2.41, surface 2.41 |
| core | `colors.border_active` | bg 1.26, surface 1.26 |
| core | botão warning, texto | rest 3.37 e 3.65 (as duas pontas do gradiente), hover 2.70 e 2.86, active 3.37 e 3.65 |
| hoff | `glass.text_placeholder` | bg 3.86, surface 3.95, bg_panel 3.39, bg_hover 2.73, popover 3.39 |
| hoff | `colors.text_dim` | bg_hover 4.45 |
| hoff | `glass.text_faint` | bg_hover 3.76 |
| hoff | `colors.danger` | bg 2.27, surface 2.34 |
| hoff | `colors.border_active` | bg 2.22, surface 2.26 |
| hoff | botão secondary, texto | hover 1.57 |
| hoff | botão warning, texto | rest 3.27, hover 3.40, active 3.27 |
| dark | `colors.text_dim` | bg 2.82, surface 2.66, bg_panel 2.53, bg_hover 2.33, popover 2.53 |
| dark | `glass.text_placeholder` | surface 4.50 (abaixo de 4,5 na terceira casa), bg_panel 4.33, bg_hover 4.05, popover 4.33 |
| dark | `colors.border_active` | bg 2.16, surface 2.03 |
| light | `colors.text_dim` | bg 4.50 (abaixo de 4,5 na terceira casa), bg_panel 4.12, bg_hover 3.84, popover 4.12 |
| light | `glass.text_active` | bg 3.50, surface 3.53, bg_panel 3.45, bg_hover 3.41, popover 3.45 |
| light | `glass.text_default` | bg 2.11, surface 2.12, bg_panel 2.10, bg_hover 2.09, popover 2.10 |
| light | `glass.text_faint` | bg 1.60, surface 1.61, bg_panel 1.60, bg_hover 1.59, popover 1.60 |
| light | `glass.text_placeholder` | bg 1.31, surface 1.31, bg_panel 1.30, bg_hover 1.30, popover 1.30 |
| light | `glass.text_gradient[1]` | bg 1.89, surface 1.89 |
| light | `colors.warning` | bg 2.15, surface 2.24 |
| light | `colors.info` | bg 2.76, surface 2.87 |
| light | `colors.border_active` | bg 2.73, surface 2.84 |
| light | botão primary, texto | rest 4.06, active 4.06 |
| light | botão secondary, texto | rest 1.09, hover 1.17, active 1.09 |
| light | botão warning, texto | rest 2.15, hover 1.83, active 2.15 |

o que salta: no core, o tema padrão, o cinza `#7B7B7B` (shade6) de todo
texto secundário fica em 4.13 sobre o papel; no light, os textos de
vidro vêm de `GlassTokens::derive` com alfa sobre fundo claro e a receita
derivada do botão secondary dá 1.09. são decisões de design do brenner;
o teste guarda a lista até lá.

### reduced motion

caminho sem dependência nova, no ponto único: `engine::animation`
(`mod.rs`, `spring.rs`, `tween.rs`). a preferência é por thread
(`set_reduced_motion`, `reduced_motion`); ligada, `Spring::tick` pousa no
alvo e `Tween::tick` completa uma animação finita no primeiro tick,
mesmo com passo zero; um tween infinito continua. a leitura é
`accessibility::motion` (`/usr/bin/defaults read
com.apple.universalaccess reduceMotion`, chave ausente é desligado), e
quem a aplica é a `Bridge`: ao nascer e a cada `WindowEvent::Focused(true)`,
na thread do event loop. o shell não mudou para isso. outros sistemas
respondem `None` (sem mudança) até o alvo ou a dependência. testes: 5 em
`animation/tests_motion.rs` (a mola e o tween terminam no primeiro
frame, sem a preferência seguem no meio, o laço infinito continua, a
preferência é da thread), 2 em `accessibility/motion.rs` (a leitura do
`defaults`, e o `defaults` real responde no macOS sem mudar nada), 3 em
`apps/uigallery/tests/reduced_motion.rs` (switch, toast, modal e a
galeria inteira assentam no primeiro frame; sem a preferência, estão em
movimento).

### o roteiro do VoiceOver

na seção "fase 1: roteiros manuais" acima: todo, formulário (Silences,
Surface, Files), toast, modal, tabs, lista, o ide e reduced motion, com o
que ouvir em cada passo. as outras plataformas ficam marcadas para quando
o alvo rodar.

### gate e regress

- antes (em `bd03459d`, rodado aqui): `./tool/tasks/fullcheck.sh` verde, 2932
  passam, 26 ignorados, pernas urnaui e wasm rodaram; `regress check`
  175 baselines, 175 iguais.
- depois (em `d811876f`): `./tool/tasks/fullcheck.sh` verde, 2960 passam (+28), 26
  ignorados, 0 falhas, pernas urnaui e wasm rodaram; `regress check` 175
  baselines, 175 iguais.
- `Cargo.toml` e `Cargo.lock` intocados (md5 do lock igual), nenhuma
  dependência nova, AccessKit na mesma versão. sempre com
  `PLEV_CLAUDE` no stub e `--offline`; nenhuma janela aberta, nenhum
  leitor de tela ligado.

### linha para o índice de ADRs

| 2026-09-25 | [accessibility-is-production-ready-per-platform](../../adr/interface/0068-accessibility-is-production-ready-per-platform.md) | proposed | a matriz do que o AccessKit do lock entrega por plataforma (macOS roda; windows e linux esperam o alvo; iOS e web sem adapter; android fora do lock); a árvore de toda página testada; status polida e alerta assertivo sem mover o foco, o toast na árvore do leitor; 301 pares de contraste medidos, 102 falhando listados sem mudar token; reduced motion no ponto único de animação, lido do macOS pela ponte; o web em aberto entre espelho DOM gerado do snapshot e nada por enquanto (task-75 fase 1) |
