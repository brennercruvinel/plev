---
id: task-76/item-4-native-menus
title: "task-76, item 4: native menus"
project: plev
type: section
status: reference
parent: task-76
date: 2026-09-24
date_basis: text
topics: [platform, semantics, agents]
relations: [adr-0057, task-59]
source: docs/log/steps/task-76-platform-integration.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
evidence: [3338c38e, da659be1, 5c351c62, 32794c25, 79ecbaed, af75dd50]
---

# item 4 handoff: native menus as capabilities

Records item 4: a pure `MenuModel`, built by the app from the projection, becomes the menu bar through `muda` on desktop, behind the `NativeMenu` trait, with `FakeMenu` for tests and `NoMenu` on wasm and mobile. Each item is a `SemId` with an action, and the click comes back as `Request::observe` through `Actor::Human`, the same as the button, refused where the button would be refused. The result is the showcase's menu bar with `shell.page`, `shell.theme`, `shell.detach`, `shell.save_session` and the active page's actions; the real menu bar was neither seen nor clicked on this machine, and the windows leg is written but not compiled.


branch `task/76d-menus`, base `8ce7225c` (main). commits, em ordem:

- `3338c38e` task-76: the native menu probe
- `da659be1` task-76: the menu model and the trait, with a fake
- `5c351c62` task-76: the muda menu on the desktop, and the poll
- `32794c25` task-76: the shell projected, and its keys through the
  dispatcher
- `79ecbaed` task-76: the showcase menu bar
- `af75dd50` task-76: the urnaui lock follows the engine's muda
- (este) task-76: ADR and the item 4 handoff

## a sonda

(a) o lock, com `muda = { version = "0.20", default-features = false }`
na seção de desktop não móvel de `rust/engine/Cargo.toml` (a do
`arboard` e do `rfd`), resolvido com `cargo metadata --offline`: "Locking
2 packages: Adding keyboard-types v0.8.3, Adding muda v0.20.0". o
`Cargo.lock` da raiz ganhou só esses dois pacotes; `objc2-app-kit 0.3.2`
ganhou a aresta `objc2-core-foundation` (feature). o resto do que o
`muda` pede já estava no lock: `crossbeam-channel 0.5.15`, `dpi 0.1.2`,
`objc2 0.6.4`, `objc2-core-foundation 0.3.2`, `objc2-foundation 0.3.2`,
`once_cell 1.21.4`, `png 0.18.1`, `thiserror 2.0.18` e, como entrada só
de windows, `windows-sys 0.61.2`. as features padrão (`gtk3`, `libxdo`)
ficaram desligadas: no linux o `muda` compila no backend `noop`
(`platform_impl/mod.rs`) até o bring-up escolher. `cargo tree --offline
-p engine -i muda` mostra só o engine como consumidor. nada faltou no
cache; não parei. o `apps/urnadbapp/Cargo.lock` (workspace próprio, que
depende do engine por caminho) ganhou `muda`, `keyboard-types` e um
`crossbeam-channel 0.5.17`, como ganhou `rfd` no item 3; a perna urnaui
do gate o resolveu offline.

(b) o código do `muda` 0.20 (`~/.cargo/registry/src/*/muda-0.20.0/src/`):

- o clique: cada `NSMenuItem` criado pelo `muda` tem como alvo um
  `NsMenuItem` (`platform_impl/macos/mod.rs:820-840`) cuja ação chama
  `MenuEvent::send(MenuEvent { id })`; para um `CheckMenuItem` ela
  inverte o `checked` antes de enviar. `MenuEvent::send`
  (`menu_event.rs`) entrega ao handler único do processo se
  `set_event_handler(Some(..))` foi chamado, senão ao canal global
  (`MENU_CHANNEL`, um `crossbeam_channel::unbounded`) que
  `MenuEvent::receiver()` lê; com handler, o canal não recebe nada. o
  handler é um `OnceCell`: uma vez por processo. o `id` é o `MenuId`
  (uma `String`) do item, o que o app passou em `MenuItem::with_id` ou um
  contador. a nota do próprio `muda` para winit: usar o handler e acordar
  o laço pelo `EventLoopProxy`.
- por app e por janela: no macOS `Menu::init_for_nsapp` faz
  `NSApplication::sharedApplication().setMainMenu(Some(menu))`
  (`macos/mod.rs:164-168`), a barra é do aplicativo, uma para todas as
  janelas; um `Menu` só aceita `Submenu` no topo. no windows
  `Menu::init_for_hwnd(hwnd)` prende a barra a uma janela. no gtk,
  `init_for_gtk_window`. `Menu::new` e `Submenu::new` exigem a thread
  principal (`MainThreadMarker::new().expect(..)`), então nenhum teste
  do cargo constrói um `Menu`.
- o laço do winit: o winit 0.30.13 instala a barra padrão dele (About,
  Services, Hide, Hide Others, Show All, Quit) em
  `applicationDidFinishLaunching` quando `default_menu` está ligado
  (`platform_impl/macos/app_state.rs:139-142`, `menu.rs`), logo antes de
  `dispatch_init_events`, que dispara o `resumed`. chamar
  `init_for_nsapp` no `resumed` substitui a barra padrão sem mexer no
  laço; `Menu::drop` só tira do `NSApp` a barra que ainda é a principal,
  então a nova entra antes da antiga sair. ide e urnaui não precisam
  mudar nada.
- uma janela real: um teste não prova o `setMainMenu` num `NSApp` vivo,
  então rodei o showcase uma vez com `PLEV_BACKGROUND=1` (`Todo`, 8 s,
  morto pelo pid exato, nenhum clique nem tecla): o log diz "menu: 100
  items shown" e "menu: 100 items in the bar" (14 temas, 81 páginas, o
  destacar, 4 ações da Todo), sem pânico. com a política de ativação
  proibida a barra não aparece, então o clique num item real fica pro
  orquestrador.

(c) da projeção do showcase, o que hoje podia virar item de menu (ação
sem parâmetro, permitida a `human`):

- `showcase-todo.snapshot.json`: `todo.input` `submit` (desabilitado com
  o campo vazio), `todo.filter[all|active|completed]` `select`,
  `todo.item[n]` `toggle` e `remove` (destrutivo). `set_value` fica de
  fora (param texto).
- `showcase-silences.snapshot.json`: `silences.threshold` e
  `silences.min_duration` `increment` e `decrement`, `silences.detect`
  `detect_silence`, `silences.cut[n]` `preview_segment`, `apply_cut`,
  `reject_cut`, `silences.apply_all` `apply_all`. `set_value` fora. a
  projeção viva da página (`projection()`) traz ainda a linha de
  propor (`proposer: open/close`, `propose`, `cancel`, `close the
  surface`), que o golden não tem.
- `showcase-files.snapshot.json`: `files.open` `open_file` (só pessoa);
  `drop_files` fora (param lista).
- atalhos do shell só como tecla: `T` (cicla core, hoff, dark, light),
  os dígitos `1` a `9` e `0` (página pelo índice), `D` (a página numa
  janela própria, `panes.detach_asked`), `Esc` (fecha overlay, fecha a
  janela da página, ou sai) e `cmd-shift-s` (salva a gravação, só com
  `PLEV_RECORD`). o clique na sidebar e nos cards da página Theme
  também mudavam página e tema direto (`set_page`, `set_theme`).

## o modelo e o trait

`rust/engine/src/runtime/platform/menu/mod.rs` (606 linhas de produção) e
`platform/menu/native.rs` (317). a mesma forma do `files.rs`: trait,
real sob o cfg do `arboard`, falso em todo alvo, nenhum em wasm e
mobile, `default_menu(wake)` escolhendo.

- `MenuModel { menus: Vec<Submenu> }`; `Submenu { label, entries }`;
  `Entry::{Item(MenuItem), Separator, Predefined(Predefined),
  Submenu(Submenu)}`; `Predefined::{About, Hide, HideOthers, ShowAll,
  Quit}` (os itens da plataforma, em palavras dela; nunca capacidade).
- `MenuItem { sem: SemId, action: ActionName, param: Option<Value>,
  label, shortcut: Option<Shortcut>, enabled, checked: Option<bool> }`.
  `MenuItem::of(snapshot, sem, action)` lê uma ação de um nó (label da
  ação ou do nó; `enabled` quando nó e ação estão e o nó não é inert; a
  marca do `on` de um Toggle ou do `checked` de um Checkbox);
  `MenuItem::options(snapshot, sem, action)` um item por opção de
  `constraints.options` (param `Text(opção)`, marca na `selected`);
  `MenuItem::plain(snapshot)` toda ação sem parâmetro que uma pessoa pode
  pedir (`policy.actors.human`), em ordem de projeção. `key()` é o texto
  `sem#action[#param]` que nomeia o item no `muda`. o app decide o que
  entra e em que submenu; a plataforma nunca.
- `Shortcut { primary, shift, alt, ctrl, key }`, `Shortcut::parse("cmd-shift-s")`
  (aceita `cmd`, `ctrl`, `shift`, `alt`, `option`; `cmd` é o primário,
  cmd no macOS e ctrl fora); exige um modificador: uma letra solta como
  key equivalent roubaria a tecla de todo campo de texto (por isso `D` e
  `T` não têm atalho no menu).
- `Clicked { sem, action, param }` e `Clicked::request(&seen)` =
  `Request::observe(seen, Actor::Human, sem, action, param)`, o mesmo
  request do botão.
- `trait NativeMenu { install(&MenuModel), update(&MenuModel), poll() ->
  Vec<Clicked> }`. `FakeMenu` guarda o modelo, conta `installs` e
  `updates`, `click(sem, action, param)` enfileira só item mostrado e
  habilitado, `poll` devolve. `NoMenu` responde `MenuError::Unsupported`.
  `impl NativeMenu for Arc<Mutex<M>>` deixa um teste segurar o falso
  enquanto o shell segura o `Box<dyn NativeMenu>`.
- `MudaMenu::new(wake)`: registra o handler único do `muda` (uma vez por
  processo, `Once`) que empilha o `MenuId` numa fila global e chama o
  waker atual; `install` monta `Menu` + `Submenu`s + `MenuItem`/
  `CheckMenuItem`/`PredefinedMenuItem` com `with_id(key)`, guarda `id ->
  Clicked`, `init_for_nsapp` no macOS (`init_for_hwnd` no windows quando
  `attach_window` deu o hwnd); `update` compara: igual não faz nada,
  mesma forma (`MenuModel::same_shape`: as mesmas chaves nos mesmos
  lugares, mesmo atalho, mesmo tipo marcado ou não) muda `set_text`,
  `set_enabled`, `set_checked` no lugar, forma diferente refaz a barra
  (item cuja capacidade sumiu sai); `poll` drena a fila e mapeia os ids
  (um id de barra refeita é logado e descartado). um `MudaMenu` por
  processo: a fila é do processo.
- `MenuError::{Unsupported, BadShortcut(String), Platform(String)}`.

## por janela e por app

macOS: a barra é do app (`NSApp.mainMenu`), uma para todas as janelas; o
shell do showcase guarda um `menu: Option<menu::Menu>` (`app.rs`) ao lado
do estado que projeta, instala no `resumed` depois da janela principal
(substitui a barra padrão do winit) e a janela da página mostra a mesma
página, logo a mesma barra. windows: a barra é da janela;
`MudaMenu::attach_window(&Window)` extrai o hwnd pelo
`raw_window_handle` e chama `init_for_hwnd`, escrito e não compilado
aqui (windows não é alvo). linux: backend `noop` do `muda` até o
bring-up. `WindowHost`, `Windows::route`, `lifecycle.rs` não mudaram: a
janela não guarda menu porque no macOS ele não é dela. o `poll` acontece
no laço do app: o clique acorda o laço (`UserEvent::Menu` pelo proxy),
`user_event` drena (`menu_clicks`), despacha pela view e toma o que é do
shell (`detach_requested`, `save_requested`); a barra segue a projeção
depois de cada frame da janela principal (`sync_menu` em `pace.rs`).

## o shell projetado

`apps/uigallery/src/views/dispatcher/shell/mod.rs` (430 de produção;
`shell/tests.rs` com 7 testes). `Shell` mora em `Panes` no lugar de
`detach_asked`; o `Operable` é um `ShellOp` que projeta a partir da view
(tema, página, destacada) e escreve os pedidos (`Asks`), despachado pelo
dispatcher da galeria (`TodoPage::with_dispatcher`, como Surface,
Silences e Files). nós:

- `shell` (Region, "plev").
- `shell.page` (Select, "page"): `selected` = título da página ativa,
  `Constraints { editable, options: títulos de PAGES }`, `select` com
  param `Text` constrained lendo `shell.page:selected`; policy padrão
  (qualquer ator). `execute` guarda o índice; a view aplica `set_page`
  logo após o dispatch. os dígitos e o clique na sidebar pedem por ele.
- `shell.theme` (Select, "theme"): `selected` = `theme_name`, opções =
  `THEMES`, `select` idem; qualquer ator. `T` e o card da página Theme
  pedem por ele; a view aplica `set_theme`.
- `shell.detach` (Toggle, "page in a window of its own"): `on` =
  destacada, `enabled` só no desktop e com página que tem pane
  (placeholder não), `toggle` lendo `shell.detach:on`, policy `Actors {
  human, assistive, test: true; agent: false }` (a mesma do
  `open_file`). `execute` marca o pedido; o shell lê com
  `take_detach_request` (como antes) e abre ou fecha a janela.
- `shell.save_session` (Button, "save the session recording"), só
  enquanto `set_recording(true)` (o `App::record` chama com
  `PLEV_RECORD`): `activate`, só pessoa. `execute` marca; o shell lê com
  `take_save_request` e chama `save_session("asked")`. `cmd-shift-s`
  (`record_key`) pede por ele (`ShowcaseView::request_save`).
- nenhum é `focusable` (nada na tela toma foco por eles).

`ShowcaseView::semantics()` continua a projeção da página (com o nó
`window` quando destacada), byte a byte: o golden do todo, os 161
baselines, a árvore de acessibilidade, a gravação e o cmd+click leem
essa. o shell entra em `gallery_semantics()` (shell + página, um
snapshot, os nós da página reconstruídos por `detached::extend`, que
`with_window` também usa), que a porta do agente passa a servir
(`Served::project`) e o menu lê. `act(req)` roteia por `is_shell(target)`
para `act_shell`, senão para a página. `ask_shell(target, action, param)`
é o pedido da pessoa observado no shell; `menu_click(&Clicked)` é o
pedido observado na galeria e roteado por `act`.

## o menu do showcase

`apps/uigallery/src/shell/system/menu.rs` (234 de produção; 2 testes). a barra,
de `gallery_semantics`: "plev" (About, separador, Hide, Hide Others,
Show All, separador, Quit, como a barra padrão do winit menos Services),
"view" (submenu "theme" com uma opção por tema e a marca na atual,
submenu "go to" com as 81 páginas e a marca na ativa, separador, o item
marcável "page in a window of its own", e, gravando, separador e "save
the session recording" com `cmd-shift-s`) e o título da página ativa com
as ações simples dela (`MenuItem::plain` sem os nós do shell e sem
linhas: ListItem, Row, Cell, TreeItem, Option; rótulo: label da ação se
declarada, Button pelo label do nó, Tab por "lista: aba" ("filter:
Active"), o resto "nó: ação" ("new todo: submit")), só quando há alguma.
na Todo: "new todo: submit" (desabilitado sem texto), "filter: All",
"filter: Active", "filter: Completed"; na Silences: os increment e
decrement dos dois sliders, "detect silences", "apply all" e a linha de
propor; na Card, sem submenu de página. trocar de página muda a forma e
a barra é refeita; digitar no campo do todo muda só o `enabled` do
submit, no lugar. o clique num item corre a mesma capacidade que o
botão, pelo mesmo `Request::observe` por `Actor::Human`: inert atrás de
um modal, disabled, stale e a policy valem iguais.

## testes

engine: `platform/menu.rs` 8 unitários (atalho, item e opções de um
snapshot, ações simples, o clique é o request do botão, forma do modelo,
o falso, o handle compartilhado, `NoMenu`), `menu/native.rs` 2 (o
acelerador do `muda`, o `&` do rótulo), `tests/menu.rs` 6 sobre os
goldens do todo e das silences: os itens, rótulos e habilitados; um
clique no falso gera o `Request` por `Actor::Human` igual ao do botão e o
mesmo receipt (mesmos estágios, mesmo outcome) num `Operable` de teste,
e um clique com observação velha é `Stale`; a atualização desabilita o
que ficou `enabled: false` e tira o item cuja capacidade sumiu (a forma
muda); atalho num item e letra solta recusada; `NoMenu` não quebra.

showcase: `dispatch/shell/tests.rs` 7 (a projeção e as policies; as
teclas deixam receipts, `take_detach_request` e `take_save_request`; os
cards da Theme e a sidebar pedem igual; um agente troca tema e página por
`act` e é recusado no destacar como `NotAuthorized`, `Assistive` e
`Test` podem, título fora das opções é `InvalidParam`; um pick velho é
`Stale` com o delta; a galeria carrega shell e página e a projeção da
página fica sozinha; um clique de menu roda a capacidade da página como
o botão, disabled é recusado, item inexistente não pede nada);
`dispatch/agent.rs` 1 pela porta real (o agente vê `shell.*` ao lado de
`todo`, troca tema e página a partir da revisão vista, é recusado no
destacar); `app/menu.rs` 2 com `App::with_bare_window` e um `FakeMenu`
compartilhado (a barra, as marcas, um tema e um filtro clicados pela
view, o destacar esperando o shell, a troca de página refazendo a barra,
o save com o atalho enquanto grava). os testes de hoje de `T`, dos
dígitos, da sidebar, do `D` (detached.rs) e do save key (record.rs)
continuam verdes.

## gate e regress

`df -h /` antes do gate: 31 GB livres; 29 GB depois. `./tool/tasks/fullcheck.sh`
verde no worktree, com a perna wasm e a perna urnaui (`../urna`
presente): antes `cargo test --workspace` 2494 passed, 0 failed, 16
ignored em 117 binários; depois 2520 passed, 0 failed, 16 ignored em 118
binários (+26: 10 unitários do `platform/menu`, 6 de `engine/tests/menu.rs`,
7 do shell, 1 da porta, 2 do menu do app; +1 binário, o
`tests/menu.rs`). perna urnaui: 85 passed, 0 failed, 1 ignored em 6
binários, igual a antes. clippy `-D warnings`, fmt e a checagem wasm
limpos nas duas pernas (o showcase compila para wasm com `NoMenu`).

`cargo run -p regress -- check`: 161 baselines, 161
iguais, 0 mudaram, sem frame desenhado e sem GPU aberta (807 ms). o
golden do todo (`tests/uir_fixture.rs`) e os goldens das silences e da
files continuam iguais: a projeção da página não mudou.

linhas de produção (regra do guard): engine `platform/menu.rs` 606,
`platform/menu/native.rs` 317, `platform/mod.rs` 59; showcase `app.rs`
580 (era 546), `app/menu.rs` 234, `app/pace.rs` 161, `app/record.rs`
330, `view/dispatch.rs` 467 (era 452), `view/dispatch/shell.rs` 430,
`view/dispatch/detached.rs` 229, `view/dispatch/agent.rs` 101,
`view/input.rs` 416 (era 400), `view/panes.rs` 136. tudo abaixo de 639.

## ADR

[`0057-native-menus-are-another-view-of-capabilities.md`](../../adr/interface/0057-native-menus-are-another-view-of-capabilities.md) (proposed):
o modelo puro do menu, o clique como o request do botão, o trait com o
real, o falso e o nenhum, por app no macOS e por janela onde a
plataforma diz, o shell projetado e onde ele aparece, a barra do
showcase, o que o tray e os diálogos reaproveitam. linha pro índice, que
é do orquestrador ([`adr/`](../../adr/README.md) é proibido aqui):

| 2026-09-24 | [native-menus-are-another-view-of-capabilities](../../adr/interface/0057-native-menus-are-another-view-of-capabilities.md) | proposed | `engine::platform::menu`: um `MenuModel` puro que o app monta da própria `Snapshot` (cada item é um `SemId` e uma ação, com label, atalho, habilitado e marca como a projeção diz), `NativeMenu` (`install`, `update`, `poll`) com o `muda` no desktop (macOS: a barra do app, uma pra todas as janelas; windows: por hwnd), `FakeMenu` pra teste e `NoMenu` em wasm e mobile; o clique volta como `Clicked` e vira o mesmo `Request::observe` por `Actor::Human` do botão, pelo mesmo dispatcher; o showcase projeta o shell (`shell.page`, `shell.theme`, `shell.detach` só pessoa, `shell.save_session` só pessoa) na projeção da galeria que a porta serve, e monta a barra dela e da página ativa; a projeção da página não muda (task-76 item 4) |

## interfaces adicionadas (só crescimento)

`engine::platform::menu::{MenuModel, Submenu, Entry, MenuItem,
Predefined, Shortcut, Clicked, MenuError, NativeMenu, FakeMenu, NoMenu,
locked, default_menu}`, `MudaMenu` (desktop; `new`, `installed`,
`attach_window` no windows) e `impl NativeMenu for Arc<Mutex<M>>`.
`ShowcaseView::{shell_semantics, gallery_semantics, is_shell,
request_save, take_save_request, set_recording, menu_click}`,
`ask_shell` (pub(crate)), `view::dispatch::{Shell, ShellTarget}`;
`take_detach_request` mudou de módulo (`detached.rs` para `shell.rs`),
mesma assinatura. `Panes.shell` no lugar de `Panes.detach_asked` (o
único leitor era a própria view). `UserEvent::Menu` no shell do
showcase. `App::{install_menu, sync_menu, menu_clicks, save_requested}`
(pub(super)). nenhuma assinatura pública existente mudou; ide, urnaui,
wasm e os exemplos compilam sem tocar.

## arquivos tocados fora da lista permitida, e por quê

- `apps/urnadbapp/Cargo.lock`: consequência do `muda` no `Cargo.toml` do
  engine (o urnaui depende dele por caminho; a perna urnaui do gate
  resolve o lock). o mesmo que o `rfd` fez no item 3.
- `apps/uigallery/src/views/mod.rs` foi tocado num rascunho (um
  re-export) e voltou ao da base antes do commit: `ShellTarget` fica
  dentro de `view`; o `app` nomeia as capacidades pelos ids
  (`shell.theme`...) e pede o save por `ShowcaseView::request_save`.

## limitações e decisões, sem esconder

- a barra real não foi vista nem clicada: com `PLEV_BACKGROUND=1` a
  política de ativação é proibida e o macOS não mostra a barra de um app
  assim; o que a rodada prova é o `setMainMenu` no `NSApp` vivo sem
  pânico e os 100 itens montados. o clique num item real, o `Quit` pelo
  menu (o `terminate:` do `muda`, o mesmo do winit) e o `About` são do
  orquestrador. o caminho do clique está lido na fonte do `muda` e
  provado com o falso.
- `ShowcaseView::semantics()` (página) e `gallery_semantics()` (shell +
  página) são duas projeções do mesmo estado: a porta observa a galeria,
  a acessibilidade, a gravação, o `--semantics`, o regress e o golden
  leem a página. escolhi assim porque a projeção da página é o contrato
  com os 161 baselines e o golden; a alternativa (o shell em
  `semantics()`) movia todos. a árvore de acessibilidade não tem os nós
  do shell (eles não têm binding na tela); é candidata a ganhar quando
  houver um widget que os desenhe.
- `gallery_semantics` reconstrói os nós da página por `SnapshotBuilder`
  (como `with_window`), então o `source` deles passa a ser o
  `detached.rs`; a porta descarta `source` a menos que peçam. o
  `SnapshotBuilder::extend` no `sem` continua sendo a mudança que
  guardaria.
- um `MudaMenu` por processo: a fila de cliques e o waker são globais
  (o handler do `muda` é um por processo). dois menus no mesmo processo
  dividiriam a fila; o primeiro a `poll` levaria os cliques do outro.
- a barra segue a projeção depois de cada frame da janela principal:
  cada frame custa uma projeção da galeria e a montagem do modelo (a
  ordem de dezenas de microssegundos; na Todo com 103 itens uns 50 us,
  pela medição do task-59), mesmo quando só o caret piscou. um frame da
  janela da página sozinha não sincroniza a barra (a página é a mesma).
- windows: `attach_window` e `init_for_hwnd` estão escritos contra a API
  do `muda` e do `raw-window-handle` e não compilaram aqui. o windows
  precisa também do `TranslateAcceleratorW` no laço para os aceleradores
  (nota do `muda`), que o winit não faz; fica pro bring-up.
- `D` e `T` não têm atalho na barra: uma letra solta como key equivalent
  roubaria a tecla de todo campo de texto (o AppKit resolve o key
  equivalent antes do `keyDown` da view). o item existe e a tecla
  continua no shell, pelo mesmo `shell.detach` e `shell.theme`. só o
  save tem atalho (`cmd-shift-s`, com modificador).
- com o atalho na barra, o `cmd-shift-s` chega pelo menu (o AppKit toma
  o key equivalent antes da janela) e não mais pelo `record_key`; os
  dois caminhos pedem o mesmo `shell.save_session`, então o resultado é
  o mesmo. em teste sem barra o `record_key` é o caminho.
- os dígitos, a sidebar e os cards da Theme agora deixam um receipt cada
  no dispatcher da galeria (retenção 256): um agente que lê receipts vê
  a pessoa trocando de página; é o desejado (uma capacidade, várias
  representações), mas é novo.
- a página Theme continua com a própria geometria dos cards
  (`card_rects` privado em `foundations/theme.rs`, fora da lista); o
  teste dos cards acha um card perguntando à página, não ao layout.
- `Predefined` tem só About, Hide, HideOthers, ShowAll e Quit; Services
  (que a barra padrão do winit tinha) ficou de fora, e Copy, Cut, Paste,
  Undo não entraram: são capacidades do editor, não itens da plataforma,
  e o editor não as projeta ainda.
- o `page` submenu usa o título da página como rótulo, então a barra é
  refeita a cada troca de página (uma `setMainMenu`); é o comportamento
  esperado do `same_shape`, não medido em tempo.
- `menu.rs` do engine está em 606 linhas de produção, perto do limite;
  um crescimento (ícones, mais itens da plataforma) pede o split em
  `model.rs` e `fake.rs`.

## o que os itens 5 e 8 reaproveitam daqui

- item 5 (tray e notificações): o tray é outro `MenuModel` mostrado por
  outro objeto da plataforma (o `tray-icon` do mesmo autor consome
  `muda::Menu` como menu do ícone); o `Clicked` e o `menu_click` da view
  servem iguais, e `Windows::set_exit_when_last_closes(false)` do item 2
  deixa o app viver só no tray. o `wake` pelo proxy e a fila global são
  os mesmos; um segundo consumidor da fila (barra e tray) pede que o
  `MudaMenu` passe a distribuir por id em vez de drenar tudo. uma
  notificação que tem ação é um `Clicked` também.
- item 8 (diálogos nativos, com janela pai e `rfd::MessageDialog`): a
  policy "só pessoa" (`PERSON_ONLY` do shell, a mesma do `open_file`) e o
  molde trait, real, falso, nenhum; um item "Save session..." ou
  "Open..." do menu já dispara a capacidade da página pelo mesmo
  caminho, então um diálogo de confirmação nativo para uma ação
  `destructive` ou `Confirm::Always` entra no dispatcher entre o
  `NeedsConfirmation` e o `confirm`, onde o botão também pararia; a
  janela pai vem do `Windows` do item 2.

## comandos de validação

```text
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/one
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_PROFILE_TEST_DEBUG=line-tables-only
cargo metadata --offline --format-version 1 > /dev/null   (o lock resolve offline)
cargo test --offline -p engine --test menu
cargo test --offline -p uigallery --lib -- dispatch::shell app::menu dispatch::agent
cargo test --offline -p engine -p uigallery
./tool/tasks/fullcheck.sh
cargo run --offline -p regress -- check
PLEV_BACKGROUND=1 RUST_LOG=showcase=info cargo run --offline -p uigallery -- Todo   (a barra instala; sem clique aqui)
cargo run --offline -p uigallery -- Todo   (a barra plev / view / Todo; do orquestrador)
```
