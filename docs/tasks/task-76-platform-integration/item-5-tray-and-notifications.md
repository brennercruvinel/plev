---
id: task-76/item-5-tray-and-notifications
title: "task-76, item 5: system notifications and tray"
project: plev
type: section
parent: task-76
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [platform, agents, semantics]
relations: [adr-0059]
evidence: [b66c601f, 2550c324, fbb7641d, aeaf95b6]
source: docs/log/steps/task-76-platform-integration.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# item 5 handoff: system notifications and tray as capabilities

Records item 5: `engine::platform::tray` and `engine::platform::notify` put the tray icon and the notification behind a trait, in the mold of the dialogs, and the click comes back as the menu's `Clicked`, which becomes the button's request through `Actor::Human`. The showcase projects `shell.show`, for the person only, and `shell:notice`; Files asks for a notification for loose files with the window out of focus, and the shell announces the end of a Silences detection that nobody was watching. The result has a model, a fake and a test, but a real notification on macOS needs a bundle: without a `.app`, the `NativeNotifier` answers `NeedsBundle` and the app keeps the projected text.


branch `task/76f-tray-notify`, base `2f8274e9` (main após o item 8).
worktree `.claude/worktrees/agent`, target `target/agents/one`. commits,
em ordem:

- `b66c601f` task-76: the tray and notification probe
- `2550c324` task-76: the tray as another view of the capabilities, with
  the fake and the tray-icon one
- `fbb7641d` task-76: the notification as a capability out of the window,
  with the fake and why the desktop cannot yet
- `aeaf95b6` task-76: the showcase tray, the Files notice and the job that
  ends in the background
- (este) task-76: ADR and the item 5 handoff

## a sonda

(a) o lock. `tray-icon = { version = "0.25", default-features = false }`
e `objc2-user-notifications = { version = "0.3", default-features =
false, features = [std, block2, UNUserNotificationCenter,
UNNotificationContent, UNNotificationRequest, UNNotificationResponse,
UNNotificationAction, UNNotificationCategory, UNNotification] }` em
`rust/engine/Cargo.toml`, resolvidos com `cargo metadata --offline`:
"Locking 2 packages to latest compatible versions: Adding
objc2-user-notifications v0.3.2, Adding tray-icon v0.25.1". o
`Cargo.lock` da raiz ganhou só essas duas entradas `[[package]]`; nada
mais entrou: `crossbeam-channel 0.5.15`, `muda 0.20.0`, `objc2 0.6.4`,
`objc2-app-kit 0.3.2`, `objc2-core-foundation 0.3.2`, `objc2-core-graphics
0.3.2`, `objc2-foundation 0.3.2`, `once_cell 1.21.4`, `png 0.18.1`,
`thiserror 2.0.18`, `windows-sys 0.61.2`, `bitflags 2.13.0` e `block2
0.6.2` já estavam. arestas novas: `objc2-foundation 0.3.2` ganhou `block2
0.6.2` (feature), e a entrada `objc2-user-notifications 0.2.2` que o lock
já tinha (via o winit, alvo ios) passou a ser nomeada com a versão no
consumidor, porque haveria duas. features do `objc2-app-kit 0.3.2` que o
`tray-icon` liga a mais: `NSStatusBar`, `NSStatusItem`,
`NSStatusBarButton`, `NSTrackingArea`, `NSButton`, `NSCell`, `NSControl`
(as outras, `NSMenu`, `NSImage`, `NSEvent`, `NSView`, `NSWindow`, o winit,
o muda e o rfd já ligavam). `cargo tree --offline -p engine -i tray-icon`
mostra só o engine como consumidor. nada faltou no cache; não parei.

depois da sonda, o `objc2-user-notifications` saiu do `Cargo.toml` (ver
(b)): o lock final da raiz tem só o `tray-icon` (21 linhas). o `tray-icon`
entrou em `cfg(any(target_os = "macos", target_os = "windows"))`, não na
seção de desktop toda: no linux o crate tem `compile_error!` sem a feature
`libappindicator` ou `ksni` (escolha do bring-up de linux), então o linux
fica com `NoTray`.

o `apps/urnadbapp/Cargo.lock` (workspace próprio, depende do engine por
caminho) não pôde ser resolvido pelo cargo: aquele workspace não declara
`resolver` e a edição 2024 lhe dá o resolver 3, ciente de MSRV, e qualquer
relock ali ("Locking 23 packages to latest Rust 1.85 compatible versions")
rebaixa 22 pacotes que não têm nada com o tray (async-trait, bytes,
blocking...) e ainda pede download de um que não está no cache. escrevi a
entrada do `tray-icon` e as duas arestas (`engine -> tray-icon`,
`objc2-foundation 0.3.2 -> block2 0.6.2`) à mão, na forma que aquele lock
usa (`thiserror 2.0.20`, `crossbeam-channel` sem versão), e `cargo check
--locked --offline --manifest-path apps/urnadbapp/Cargo.toml` aceitou: o
lock é exatamente o que o cargo quer, sem tocar nos outros 22. a perna
urnaui do gate rodou verde sobre ele. na raiz o `resolver = "2"` explícito
não tem esse comportamento.

(b) `UNUserNotificationCenter` num binário sem bundle, por leitura (não
rodei). o `objc2-user-notifications` 0.3.2 é binding gerado:
`currentNotificationCenter() -> Retained<UNUserNotificationCenter>`,
`requestAuthorizationWithOptions_completionHandler`,
`addNotificationRequest_withCompletionHandler(&UNNotificationRequest,
Option<&DynBlock>)`, `setDelegate(Option<&ProtocolObject<dyn
UNUserNotificationCenterDelegate>>)` com
`userNotificationCenter_didReceiveNotificationResponse_withCompletionHandler`
para o clique, `setNotificationCategories(&NSSet<UNNotificationCategory>)`
para o botão nomeado. o que a Apple faz: `+[UNUserNotificationCenter
currentNotificationCenter]` procura o bundle principal do processo
(`bundleProxyForCurrentProcess`) e, quando o executável não está dentro de
um `.app`, levanta `NSInternalInconsistencyException` ("bundleProxyForCurrentProcess
is nil: mainBundle.bundleURL = file:///.../target/debug/"); o objc2 não
apanha exceção por padrão, então o `cargo run` de hoje abortaria na
primeira chamada. e escrever esse caminho pede `objc2` (o `define_class!`
do delegate, `Retained`, `MainThreadMarker`) e `objc2-foundation`
(`NSString`, `NSSet`, `NSArray`) como dependências diretas, além das duas
aprovadas (as duas já estão no lock, mas não estavam na aprovação).

alternativas pesadas: `NSUserNotification` (deprecado desde o 10.14,
também quer bundle identifier; quem usa sem bundle troca o
`bundleIdentifier` por swizzling, uma gambiarra); `osascript -e 'display
notification'` como processo filho (aparece como Script Editor, não tem
botão, e o clique não volta: não cumpre o contrato do `Clicked`);
`Unsupported` até o app ter bundle. recomendação, e o que fiz:
`NativeNotifier` no macOS responde `NotifyError::NeedsBundle` quando o
executável está fora de um `.app` (lido pelo caminho, `bundled(exe)`:
`<Nome>.app/Contents/MacOS/<exe>`, o mesmo lugar por onde o sistema acha o
bundle) e `Unsupported` dentro de um, até o caminho do framework ser
escrito contra um bundle que exista; o `objc2-user-notifications` não
entra no manifesto sem uso. o app loga e segue; o texto projetado é a
verdade. registro, não parada, como o briefing previa.

(c) no `tray-icon` 0.25.1 (`src/lib.rs`, `platform_impl/macos/mod.rs`):
`TrayIcon::new` exige `MainThreadMarker::new()` (senão
`Error::NotMainThread`) e cria o `NSStatusItem`
(`NSStatusBar::systemStatusBar().statusItemWithLength(..)`) na hora, com
o ícone, o tooltip e uma `TrayTarget` (uma `NSView` que recebe os cliques)
dentro do botão: o ícone aparece ao ser criado; `set_visible(false)` só o
remove depois. logo não dá para criar um sem aparecer, e nenhum teste
cria um real. a nota do crate para winit: criar do `StartCause::Init` em
diante, com o laço rodando (o showcase cria no `resumed`, depois da janela,
como o menu). os eventos: `TrayIconEvent::{Click {id, position, rect,
button, button_state}, DoubleClick (windows), Enter, Move, Leave}`
chegam na thread da plataforma pelo handler único do processo
(`set_event_handler`, um `OnceCell`) ou pelo canal global
(`receiver`); com handler, o canal não recebe. o menu do ícone é
`Box<dyn muda::ContextMenu>` (`Menu` e `Submenu` implementam), e os
cliques dos itens saem pelo `MenuEvent` do `muda`, o mesmo handler que a
barra do item 4 já registrou: logo a barra e o tray dividem uma fila.
com `menu_on_left_click` ligado, o `mouseDown` esquerdo abre o menu num
laço aninhado (`performClick`) e o `mouseUp` envia `Click { Up }` depois.

(d) da projeção, o que entra no tray e o que uma notificação da Files
leva: do shell, `shell.theme#select#<tema>` (14 opções),
`shell.page#select#<título>` (81), `shell.detach#toggle`,
`shell.save_session#activate` (só gravando), todos já na barra; e o novo
`shell.show#activate` (só pessoa). da Files, `files.open#open_file` e
`files.warn#warn` (só pessoa) e `files.drop#drop_files` (param lista);
a notificação "N files received" leva `open_file`. da Silences, um job
que termina é o request `detect_silence` completando (`cuts` no outcome)
ou falhando; a notificação leva `shell.show`, não `apply_all`: o
`apply_all` aplica cortes de uma página que pode ter mudado até a pessoa
clicar, e "mostrar a janela" é o que quem lê "2 silences found" quer.

## os dois modelos e o que cada alvo faz

`rust/engine/src/runtime/platform/tray/mod.rs` (346 linhas de produção, 562 com os
testes) e `tray/native.rs` (181), pelo molde do `menu.rs`:

- `TrayImage { width, height, rgba, template }`: `from_rgba(w, h,
  bytes)` (recusa tamanho vazio e bytes que não batem), `mark(size)` (um
  anel que o engine mesmo desenha, mínimo 4 px, como template para a
  plataforma tingir), `template(bool)`.
- `TrayModel { icon, tooltip, menu: Submenu, on_click: Option<Clicked>
  }`: o menu é um `Submenu` do `platform::menu` (os mesmos `MenuItem`,
  separadores, `Predefined`, submenus; o rótulo não aparece), e não um
  `MenuModel`, porque um menu de tray tem itens no topo (o "show", o
  Quit) e o `MenuModel` só tem submenus no topo. `on_click` é a
  capacidade que o clique no ícone pede: com ela, o botão esquerdo pede e
  o direito abre o menu; sem ela, os dois abrem o menu. `items()`,
  `find()`, `same_shape()` (a do menu: ícone, tooltip, rótulos, marcas e o
  clique mudam no lugar).
- `trait Tray { install, update, poll -> Vec<Clicked> }`;
  `TrayError::{Unsupported, BadIcon(String), Platform(String)}` com
  `From<MenuError>`.
- `FakeTray`: guarda o modelo, conta `installs` e `updates`, `click(sem,
  action, param)` só em item mostrado e habilitado, `click_icon()`
  enfileira o `on_click`, `poll` devolve. `NoTray` (linux até o bring-up,
  wasm, mobile) responde `Unsupported`. `impl Tray for Arc<Mutex<T>>`.
  `default_tray(wake)`.
- `NativeTray` (macOS e windows; windows escrito contra a API e não
  compilado): `install` monta o `muda::Submenu` com o `build` do
  `menu/native.rs` (agora `pub(in crate::platform)`, com o prefixo
  `tray:` nos ids), converte o `TrayImage` em `tray_icon::Icon` e cria o
  `TrayIcon` com `with_icon`, `with_icon_as_template`, `with_tooltip`,
  `with_menu`, `with_menu_on_left_click(on_click.is_none())`; um segundo
  `install` (forma nova) troca o menu no mesmo ícone (`set_menu`), sem
  recriar o item. `update`: igual não faz nada; mesma forma muda ícone
  (`set_icon_with_as_template`), tooltip, o modo do clique e as entradas
  no lugar (`refresh` do menu); outra forma refaz o menu. `poll`: os
  cliques no ícone (contados pelo handler do `tray-icon`, um por
  processo, só `Left`/`Up`) viram o `on_click`, e os ids `tray:` saem da
  fila compartilhada e mapeiam para o `Clicked`.
- a fila dividida: `menu/native.rs` ganhou `TRAY_PREFIX`,
  `route_clicks_to(wake)` (o handler único do `muda`, o waker é o último
  dado), `wake()` e `take_clicks(mine)` (retira da fila só os ids que o
  chamador reconhece, deixa os outros); o `MudaMenu::poll` toma os ids
  sem prefixo, o `NativeTray::poll` os com. `build(submenu, ids,
  prefix)`, `refresh`, `Installed` e `platform` ficaram visíveis dentro
  de `platform`. um id de menu (`sem#action[#param]`) nunca começa com
  `tray:` (id não tem `:`).

`rust/engine/src/runtime/platform/notify.rs` (404 de produção, 584 com os
testes), pelo molde do `alerts.rs`:

- `Action { label, sem, action, param }`: `Action::new(label,
  clicked)`; `Action::of(snapshot, sem, action)` lê pelo `MenuItem::of`
  e recusa (None) o que uma pessoa não poderia apertar (policy sem
  `human`, ou desabilitado); `label()`, `clicked()`.
- `Notification { level: alerts::Level, title, body, action:
  Option<Action> }` com `new`, `info`, `warning`, `error`, `action()`.
- `NotificationId(u64)` (`Display` "notification-N");
  `NotifyError::{Unsupported, NeedsBundle, Platform(String)}`;
  `Shown<T>`.
- `trait Notifier { notify(&Notification) -> Shown<NotificationId>, poll
  -> Vec<Clicked> }`: `notify` volta na hora; o clique na notificação ou
  no botão é o `Clicked` da ação, e uma sem ação não devolve nada.
- `FakeNotifier`: `sent()`, `take_sent()` (`Sent { id, notification }`),
  `click(id)` (só de notificação enviada e com ação), `will_refuse()`
  (`Unsupported`), `will_want_a_bundle()` (`NeedsBundle`), `pending()`.
  `NoNotifier`. `impl Notifier for Arc<Mutex<N>>` e `for Box<N>`.
  `default_notifier(wake)`.
- `NativeNotifier` (desktop): no macOS `NeedsBundle` fora do bundle,
  `Unsupported` dentro (até o caminho existir); windows e linux
  `Unsupported`; `poll` vazio. `bundled(exe: &Path)` pública e testada.

## o showcase

- `apps/uigallery/src/views/dispatcher/shell/mod.rs` (495 de produção):
  `shell.show` (Button "show plev", `activate`, `PERSON_ONLY`, `enabled`
  no desktop, não focável); `execute` marca `asks.show`;
  `take_show_request()`. o nó `shell` ganhou `notice` (`shell:notice`):
  a última coisa que o shell contou por notificação por conta própria,
  `set_notice(text)`. `set_window_focused(on)` repassa à Files;
  `take_notifications()` recolhe o que a Files pediu. `menu_click` serve
  igual para barra, tray e notificação (documentado; não renomeei).
  `ShellTarget` não ganhou `Show`: o `shell.show` só é pedido de fora da
  janela, por `Clicked`.
- `apps/uigallery/src/shell/system/tray.rs` (145 de produção; 1 teste):
  `TrayIcon { native, shown }`, `install_tray` (no `resumed`, depois do
  menu; um alvo sem tray é logado), `sync_tray` (depois de cada frame da
  janela principal, junto do `sync_menu`), `tray_clicks` (dispacha pela
  view), `set_tray` (teste). o modelo: `TrayImage::mark(36)` (18 pontos na
  barra, nítido em 2x), tooltip "plev", menu "show plev" (e o `on_click`
  do ícone é o mesmo `shell.show#activate`), separador, as entradas do
  submenu "view" da barra (`view_menu`, agora `pub(super)`: tema, go to,
  a página em janela própria, o save gravando), separador, Quit. as
  ações da página ficam na barra: o tray é do app atrás da janela. o
  clique na barra, no tray ou numa notificação acorda o laço pelo mesmo
  `UserEvent::Clicked` (era `UserEvent::Menu`), que faz `menu_clicks`,
  `tray_clicks`, `notice_clicks`, depois `show_requested`,
  `save_requested`, `detach_requested`.
- `apps/uigallery/src/shell/system/notify.rs` (205 de produção; 2 testes):
  `Notices { notifier: Box<dyn Notifier>, jobs }`; `install_notices`
  (`default_notifier`, no `resumed`) e `set_notices(notifier)` (o teste
  passa o falso), que também põe um `JobSink: ReceiptSink` no dispatcher
  da galeria (`ShowcaseView::add_sink`): ele ouve todo receipt e guarda o
  fim dos `detect_silence` (`Completed` com `cuts`, ou `Failed` com o
  erro). `focus_changed(id, on)` (do `WindowEvent::Focused` em
  `events.rs`) guarda qual janela nossa tem o foco e diz à view; no
  `resumed` a view começa "sem foco" até a plataforma dizer (com
  `PLEV_BACKGROUND=1` nunca diz: o que cai é contado). `notify_pending`
  (no fim de `advance`, a cada acordar do laço, logo depois do `wake` da
  view, onde o `settle` da Silences completa o request): recolhe o que a
  Files pediu e, sem foco, o que os jobs deram ("2 silences found", "1
  silence found", ou o erro como `Notification::error`), com "show"
  (`Action::of(shell.show)`) como botão e o texto em `shell:notice`; envia
  cada um; recusa é logada. `notice_clicks` dispacha pela view.
  `show_requested`: `set_visible(true)`, `set_minimized(false)`,
  `focus_window()` na janela principal.
- a Files (`files.rs` 485 linhas, `files/semantics.rs` 385):
  `window_focused` (padrão true: uma página sozinha é olhada),
  `set_window_focused`, `notices` (a caixa de saída) e
  `take_notifications`, `notice` projetado como `files:notice`. no
  `execute` de `drop_files`, fora de foco: `received_notice(n)`
  (`Notification::info("Files", "N files received")`, "1 file" no
  singular, com `Action::of(snapshot, files.open, "open_file")` como
  botão, rótulo "open file"), o corpo vai para `files:notice` e para o
  `notice` do outcome, a notificação para a caixa. quem soltou não
  importa (um agente pela porta também): a pessoa não estava olhando. a
  página não tem `Notifier`: o shell é dono do da plataforma, como do
  menu, e mostra o que a página pediu; o texto é estado mostrado ou não.
- silences: nenhum arquivo tocado. o fim do job chega ao shell pelo
  receipt do próprio request (o `settle` chama `d.complete`/`d.fail` no
  dispatcher da galeria), que é o contrato; não precisei de uma chamada
  ao `Notifier` no fim do job nem de um campo na página.

## testes

engine: `platform/tray.rs` 5 unitários (o anel e os bytes, o modelo e a
forma, o falso com clique no menu e no ícone, o handle compartilhado,
`NoTray`), `tray/native.rs` 1 (a imagem vira `tray_icon::Icon` ou diz
por quê, sem tocar a plataforma), `platform/notify.rs` 7 (a ação é uma
capacidade que uma pessoa pode pedir agora, o falso e o clique, as
recusas, o handle compartilhado e o `Box`, `NoNotifier`, `bundled`, o
`NativeNotifier` dizendo por quê), `tests/tray.rs` 4 (o menu do tray a
partir do golden da todo; um clique no falso gera o `Request` por
`Actor::Human` igual ao do botão e o mesmo receipt, o clique no ícone pede
o `show`, um clique velho é `Stale`, um agente é `NotAuthorized` no
`show`; o tray segue a projeção, desabilitado e sumido; `NoTray`),
`tests/notify.rs` 4 sobre um inbox de teste (a notificação com botão, o
clique é o request do botão com o mesmo receipt, o texto projetado, um
clique velho é `Stale`; um agente arquiva e o botão sumiu: o clique não
gera request nenhum; uma sem botão não devolve clique; a capacidade de
agente nunca é botão e o agente é recusado no botão; sem plataforma o
texto ainda é projetado).

showcase: `dispatch/shell/tests.rs` (os 7 de antes estendidos: o
`shell.show` e a policy, `shell:notice`, `menu_click` no show deixa
receipt e `take_show_request` toma uma vez, o agente recusado no show,
+1 nó na galeria), `files/tests.rs` +1
(`files_that_land_while_no_window_has_the_focus_are_told_by_notification`:
olhada não conta; sem foco conta com o botão open, o texto é estado e
sai no outcome, o clique do botão é o `open_file` da pessoa pelo
dispatcher, o singular, o drop do agente conta, o último notice fica),
`app/tray.rs` 1 (o tray sobre uma janela nua com `FakeTray`: show
primeiro e no ícone, tema, go to, detach, sem save, sem ações da página,
Quit por último; o ícone pede o show; um tema clicado pela view e a
atualização no lugar; troca de página com a mesma forma e 1 install; o
detach espera o shell; o save aparece gravando), `app/notify.rs` 2 (a
Files fora de foco pela App com `FakeNotifier`: contado uma vez, o clique
abre pelo diálogo falso e o status diz; com foco de volta não conta; a
detecção pedida por uma pessoa na Silences, cujo job termina falhando
porque ffmpeg não é procurado nos testes do crate: `advance` conta o erro
com "show" e projeta em `shell:notice`, o clique pede o show, com foco de
volta o fim não é contado; o texto de 1 e 2 cortes).

## gate e regress

`df -h /` antes do gate: 27 GB livres; 25 GB depois. `./tool/tasks/fullcheck.sh`
verde no worktree com `CARGO_NET_OFFLINE=true`, com a perna wasm e a
perna urnaui (`../urna` presente): antes (o fechamento do item 8, base
`2f8274e9`) 2540 passed, 0 failed, 16 ignored em 120 binários; depois
2565 passed, 0 failed, 16 ignored em 122 binários (+25: 13 unitários do
engine, 4 de `tests/tray.rs`, 4 de `tests/notify.rs`, 1 da Files, 1 do
tray do app, 2 das notificações do app; +2 binários, os dois `tests/*.rs`
novos). perna urnaui: 85 passed, 0 failed, 1 ignored em 6 binários, igual
a antes. clippy `-D warnings`, fmt e a checagem wasm limpos nas duas
pernas (o showcase compila para wasm com `NoTray` e `NoNotifier`). o
único aviso no log é o `block v0.1.6` de incompatibilidade futura, que já
estava.

`cargo run -p regress -- check`: antes do bless, 161
baselines, 159 iguais, 2 mudaram (`files@core@1280x800`,
`files@hoff@1280x800`: "semantics changed: notice none -> ''; visual
unchanged, layer files"); `bless --page Files --theme core` e `--theme
hoff` (sem `--png`: só `scene.json`, a revisão, e `semantics.json`
mudaram; os `frame.png` são os mesmos), depois 161 iguais, 0 mudaram, sem
frame desenhado e sem GPU aberta. o bless é dito aqui e o motivo é o
`files:notice` que o briefing pediu; se a integração preferir o texto só
no shell, os dois voltam com `git checkout` e o `notice` sai da Files. o
golden `rust/plevir/tests/fixtures/uigallery-files.snapshot.json` foi regravado
(`UIR_BLESS=1`; +4 linhas, o `notice`). o golden da todo e os das
silences continuam iguais.

linhas de produção (regra do guard, `file_hygiene_guard` verde): engine
`platform/tray.rs` 346, `platform/tray/native.rs` 181,
`platform/notify.rs` 404, `platform/menu/native.rs` 356 (era 317),
`platform/menu.rs` 606, `platform/mod.rs` 61; showcase `app.rs` 621
totais (era 580; a exceção `cfg(test)` desconta), `app/tray.rs` 145,
`app/notify.rs` 205, `app/menu.rs` 239, `app/events.rs` 208,
`app/pace.rs` 166, `view/dispatch/shell.rs` 495 (era 430),
`view/apps/files.rs` 485 totais (era 452), `view/apps/files/semantics.rs`
385 (era 353). tudo abaixo de 639.

## ADR

[`0059-tray-and-notifications-are-capabilities-out-of-the-window.md`](../../adr/interface/0059-tray-and-notifications-are-capabilities-out-of-the-window.md)
(proposed): os dois modelos, o clique de volta como o request do botão,
a fila com dois donos, quem notifica, o que exige bundle no macOS e o que
fica para o bring-up, o showcase. linha pro índice ([`adr/`](../../adr/README.md)
está fora da lista, fica para a integração):

```text
| 2026-09-24 | [tray-and-notifications-are-capabilities-out-of-the-window](tray-and-notifications-are-capabilities-out-of-the-window.md) | proposed | task-76 item 5: `engine::platform::tray` com `TrayModel` (imagem RGBA ou o anel do engine como template, tooltip, um `Submenu` dos mesmos `MenuItem` da barra, e a capacidade que o clique no ícone pede) atrás do trait `Tray` (`install`, `update`, `poll`), `NativeTray` (`tray-icon`, macOS e windows; a fila de cliques do `muda` dividida com a barra por prefixo), `FakeTray`, `NoTray` (linux até o bring-up, wasm, mobile); `engine::platform::notify` com `Notification` (nível, título, corpo, no máximo uma `Action` que nomeia uma capacidade da projeção) atrás do trait `Notifier` (`notify` -> id, `poll`), `FakeNotifier`, `NoNotifier`, `NativeNotifier` que no macOS diz `NeedsBundle` fora de um `.app` (o framework recusa processo sem bundle; o bundle é do bring-up); todo clique volta como o `Clicked` do menu e vira o mesmo `Request::observe` por `Actor::Human` do botão, recusado onde o botão seria e sem request quando a capacidade sumiu; o showcase projeta `shell.show` (só pessoa) e `shell:notice`, mostra o tray com o shell, a Files conta por notificação os arquivos que caem sem janela em foco (`files:notice`, botão `open_file`) e o shell conta o fim da detecção da Silences ouvindo o receipt (`ReceiptSink`), sem mexer na página |
```

## interfaces adicionadas (só crescimento)

- `engine::platform::tray::{TrayImage, TrayModel, TrayError, Tray,
  FakeTray, NoTray, default_tray}` e `NativeTray` (macOS, windows; `new`,
  `installed`); `impl Tray for Arc<Mutex<T>>`; `impl From<MenuError> for
  TrayError`.
- `engine::platform::notify::{Level (o de alerts), Action, Notification,
  NotificationId, NotifyError, Shown, Notifier, Sent, FakeNotifier,
  NoNotifier, bundled, default_notifier}` e `NativeNotifier` (desktop);
  `impl Notifier for Arc<Mutex<N>>` e `for Box<N>`.
- `engine::platform::menu::native` passou a `pub(in crate::platform)`
  (era privado), com `TRAY_PREFIX`, `route_clicks_to`, `wake`,
  `take_clicks`, `build(submenu, ids, prefix)`, `refresh`, `Installed`
  visíveis dentro de `platform`; `MudaMenu::new` e o trait `NativeMenu`
  não mudaram.
- `ShowcaseView::{take_show_request, set_notice, set_window_focused,
  take_notifications}`; a projeção do shell ganhou `shell.show` e
  `shell:notice`; a da Files ganhou `files:notice` (e `notice` no outcome
  do `drop_files` fora de foco). `FilesPage::{set_window_focused,
  take_notifications}`. `UserEvent::Menu` virou `UserEvent::Clicked`
  (privado do app). `App::{install_tray, sync_tray, tray_clicks,
  install_notices, set_notices, focus_changed, notify_pending,
  notice_clicks, show_requested}` (pub(super)); `menu::view_menu`
  (pub(super)). nenhuma assinatura pública existente mudou; ide, urnaui,
  wasm e os exemplos compilam sem tocar.

## arquivos tocados fora da lista permitida, e por quê

- `apps/urnadbapp/Cargo.lock`: consequência do `tray-icon` no
  `Cargo.toml` do engine, como `muda` e `rfd` antes; à mão, pelo motivo
  da sonda (a), aceito por `--locked` e pela perna urnaui do gate.
- nenhum outro. `rust/engine/src/lib.rs` não precisou mudar (`platform`
  já é `pub mod`). `silences/**` não foi tocado.

## limitações e decisões, sem esconder

- nenhum ícone de tray e nenhuma notificação de verdade nesta máquina: o
  `NativeTray` está compilado e não rodou (criar um mostra na barra), a
  notificação real não existe (bundle). o que a rodada prova é o modelo,
  o falso, o clique de volta pelo dispatcher e o showcase sobre janela
  nua. a prova na tela é do orquestrador: `cargo run --offline -p uigallery
  -- Files`, o anel na barra de menus; clique direito abre o menu (show
  plev, theme, go to, page in a window of its own, Quit); clique esquerdo
  traz a janela para frente; com a janela atrás de outra, soltar arquivos
  nela mostra "notification not shown (notification: the system's
  notification center wants an app bundle...)" no log com `RUST_LOG=
  showcase=info` e `files:notice` pela porta diz "2 files received".
- o `objc2-user-notifications` não entrou: seria dependência sem uso. a
  linha exata para quando o bundle existir está na sonda (a); o caminho
  precisa ainda de `objc2` e `objc2-foundation` diretos, um `define_class!`
  para o `UNUserNotificationCenterDelegate` (o clique volta em
  `didReceiveNotificationResponse`, com `actionIdentifier` igual ao id da
  nossa `Action` e `UNNotificationDefaultActionIdentifier` para o toque no
  corpo), uma `UNNotificationCategory` com a `UNNotificationAction` do
  botão, `requestAuthorization` na primeira vez, e `Info.plist` com
  `CFBundleIdentifier`. tudo do bring-up de plataforma.
- o `NativeNotifier` é, hoje, um `NoNotifier` que sabe dizer por quê;
  existe para o molde (real, falso, nenhum) e para o `bundled`. quando o
  real chegar, ele pode não ser `Send` (objetos objc): o showcase o guarda
  em `Box<dyn Notifier>` sem `Arc`, de propósito.
- um `NativeTray` por processo (a contagem de cliques no ícone e o
  handler do `tray-icon` são globais) e um laço por processo (o waker do
  `muda` é o último dado; barra e tray dão o mesmo proxy). a perna windows
  está escrita e não compilada.
- o tray segue a projeção depois de cada frame da janela principal, como
  a barra: mais uma projeção da galeria por frame (a ordem de dezenas de
  microssegundos, medida no item 4). o menu do tray carrega o acelerador
  `cmd-shift-s` do save gravando, o mesmo da barra; num menu de contexto
  ele vale só com o menu aberto.
- o app continua fechando ao fechar a janela principal: viver só no tray
  (`Windows::set_exit_when_last_closes(false)`, reabrir a janela pelo
  `show`) fica para depois; o `shell.show` hoje traz para frente, tira de
  minimizado e torna visível a janela que existe.
- a Files conta o drop de quem quer que seja fora de foco, agente
  incluído; um agente que soltasse arquivos em série fora de foco geraria
  uma notificação por drop. é o desejado (a pessoa não estava olhando) e
  é novo; um agente nunca pede notificação por si.
- a notificação de fundo da Silences leva `shell.show` e não `apply_all`
  (sonda (d)); o fim de uma detecção com a janela em foco não é contado
  (a pessoa vê na tela). o `Failed` também é contado, como erro: um job
  que morreu enquanto ninguém olhava merece aviso.
- a projeção da Files mudou (`files:notice`) e com ela o golden e duas
  baselines, por significado; o subtítulo da página em `pages/table.rs`
  continua sem falar de notificação (fora da lista).
- o `ShellTarget` não ganhou `Show`: nada dentro da janela pede o show
  por alvo, e a variante daria dead code; se uma tecla ou um botão vier a
  pedir, ela entra.

## o que fica para o bring-up e o que o item 6 reaproveita

- bring-up de plataforma: o bundle do app (e com ele o caminho
  UserNotifications acima); um backend de tray no linux (`libappindicator`
  ou `ksni` como feature do `tray-icon`); compilar e provar a perna
  windows do `NativeTray`.
- item 6 (deep links): um `Request` vindo de fora tem a mesma forma do
  `Clicked` (uma capacidade nomeada, um param opcional) e o mesmo caminho
  (chega na thread da plataforma, acorda o laço pelo proxy, é `poll`ado
  e vira `Request::observe(gallery, Actor::Human, ..)` por `menu_click`);
  o `UserEvent::Clicked` é o acordar que serve; a recusa quando a
  capacidade sumiu e a policy valem iguais. a diferença: o esquema de URL
  do app no macOS também exige bundle (`CFBundleURLTypes`), o mesmo
  bring-up.

## comandos de validação

```text
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/one
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_PROFILE_TEST_DEBUG=line-tables-only
export CARGO_NET_OFFLINE=true
cargo metadata --offline --format-version 1 > /dev/null   (o lock da raiz resolve offline)
cargo check --locked --offline --manifest-path apps/urnadbapp/Cargo.toml   (o lock do urnaui é o que o cargo quer)
cargo test --offline -p engine --lib platform::tray platform::notify
cargo test --offline -p engine --test tray --test notify --test menu
cargo test --offline -p uigallery --lib -- dispatch::shell app::menu app::tray app::notify files::
cargo test --offline -p engine -p uigallery
./tool/tasks/fullcheck.sh
cargo run --offline -p regress -- check
UIR_BLESS=1 cargo test --offline -p uigallery --lib files::tests::the_page_projection   (regrava o golden)
cargo run --offline -p regress -- bless --page Files --theme core   (e --theme hoff; sem --png)
RUST_LOG=showcase=info cargo run --offline -p uigallery -- Files   (o anel na barra, o menu, o show; do orquestrador)
```
