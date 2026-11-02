---
id: task-76/item-3-files-and-drop
title: "task-76, item 3: file dialogs and drag and drop"
project: plev
type: section
parent: task-76
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [platform, input, agents, urna]
relations: [adr-0055, task-58]
evidence: [6b8bd395, b1a2e686, c90d1e1d, e65564a3, fe5a9279, 77800d1b]
source: docs/log/steps/task-76-platform-integration.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# handoff of item 3: file dialogs and drag and drop

Records item 3: the file dialog moves from urnaui to `engine::platform::files`, behind a synchronous trait that returns a path, with `NativeDialogs` on desktop (`rfd` without default features), `FakeDialogs` for tests and `NoDialogs` on wasm and mobile. The drop becomes the `engine::input::FileDrop` event, with a single translation of winit's three events, and the showcase's Files page projects `open_file` out of the agent's reach and `drop_files` to every actor. The result is that urnaui uses the engine's dialog with no second implementation; the real panel was not opened on this machine, and the drop position is `None`, because winit 0.30 does not deliver it.


branch `task/76b-file-dialogs`, base `e3dc7e1f` (main). commits, em ordem:

- `6b8bd395` task-76: the file dialog and drop probe
- `b1a2e686` task-76: the file dialogs behind a trait, with a fake
- `c90d1e1d` task-76: the file drop as an event of the engine
- `e65564a3` task-76: the urnaui opens and exports through the engine's
  dialogs
- `fe5a9279` task-76: the Files page, the dialog and the drop as
  capabilities
- `77800d1b` task-76: the explorer's dialogs in their own module
- (este) task-76: ADR and the item 3 handoff

decisões do orquestrador depois da sonda: `rfd` sem features padrão,
inline em `rust/engine/Cargo.toml`, na seção do `arboard`, lock com
`--offline` (opção B); o shell do showcase ganha um braço só (opção 2);
`open_file` com `Actors { human, assistive, test: true; agent: false }`;
posição opcional, junção no `App` do engine, trait síncrono por caminho,
`web_picker.rs` intocado.

## a sonda

(a) o que o urnaui faz hoje com `rfd` e com os eventos de drop, por
linha:

- `apps/urnadbapp/Cargo.toml:45`: `rfd = "0.15"`, features padrão
  (`xdg-portal` e `async-std`), na seção
  `[target.'cfg(not(target_arch = "wasm32"))']`. o `rfd` 0.15.4 só tem
  backend para linux/bsd, macos, windows e wasm (`backend.rs:7-45`): em
  android e ios `FileDialog::pick_file` não compila (nenhum
  `FilePickerDialogImpl`). o urnaui só chama `rfd` sob
  `not(any(wasm32, android, ios))`, mas declara a dependência para todo
  não-wasm; no engine ela entra na seção do `arboard`, que já exclui
  android e ios.
- `apps/urnadbapp/src/view/explorer/mod.rs:400-407`, `Action::PickFile` no
  desktop: `rfd::FileDialog::new().set_title("Open a .urna corpus")
  .add_filter("urna corpus", &["urna", "nest"]).pick_file()`; `Some(path)`
  vira `open_database(path)`, `None` (cancelado) não faz nada. síncrono,
  modal, sem parent, sem diretório inicial.
- `apps/urnadbapp/src/view/explorer/mod.rs:445-476`, `export_blob` no desktop:
  `rfd::FileDialog::new().set_title("Export media blob")
  .set_file_name(&name).save_file()`; `Some(dest)` vira
  `UrnaCommand::ExportBlob { index, dest }` e um toast; `None` devolve
  `true` (tratado, nada a fazer). `explorer.rs:478-486`: em wasm, android e
  ios a exportação só toasta "blob export needs the desktop app".
- `apps/urnadbapp/src/view/explorer/mod.rs:393-399` e `app.rs:482-488`: em
  wasm `PickFile` marca `pick_requested`; o shell dispara
  `web_picker::trigger` no `about_to_wait`, e o `<input type=file>`
  devolve `UserEvent::FileLoaded { name, bytes }` (bytes, nunca caminho),
  que vira `view.file_picked`. é a entrada de arquivo do alvo web e não
  passa por `rfd`.
- `apps/urnadbapp/src/app.rs:362-377` (só não-wasm):
  `WindowEvent::DroppedFile(path)` vira `view.file_dropped(path)` mais
  invalidate; `HoveredFile(_)` vira `view.set_file_hover(true)`, o caminho
  do arquivo pairando é descartado; `HoveredFileCancelled` vira
  `set_file_hover(false)`.
- `apps/urnadbapp/src/view/mod.rs:217-233`: `file_dropped(path)` desliga o
  hover, chama `explorer.open_database(path)` e toasta "opening dropped
  file…"; `set_file_hover(bool)` repassa ao explorer.
- `apps/urnadbapp/src/view/explorer/mod.rs:67,121,186-190,869` e
  `open.rs:32-33,414-435`: `file_hover: bool` chega à tela Open como
  `OpenContext.file_hover` e desenha um contorno de acento em volta do
  conteúdo com a dica "release to open". `open.rs:276-279`: a dica de
  status muda por alvo ("drop a .urna file anywhere" no desktop, "pick a
  .urna file (drag-and-drop is not available on the web canvas)" na web).
- o que a view recebe: só o `PathBuf`, um por evento. posição, nenhuma.
  um drop de vários arquivos chega como N `DroppedFile`, e o urnaui
  chama `open_database` N vezes (o último ganha).

(b) o que compila no wasm no `rfd` 0.15.4
(`~/.cargo/registry/src/*/rfd-0.15.4/src/`):

- `lib.rs:114-119`: `FileDialog` (síncrono) é
  `#[cfg(not(target_arch = "wasm32"))]`; `AsyncFileDialog` existe em todo
  alvo.
- `backend.rs:29-30` e `backend/wasm.rs`: no wasm `AsyncFileDialog::
  pick_file` e `pick_files` abrem um overlay DOM próprio (`rfd-overlay`,
  com o `style.css` do crate) com um `<input type=file>` e devolvem um
  `FileHandle` que embrulha `web_sys::File`: `file_name()` e `read()`
  assíncrono em bytes, sem caminho. filtros viram o atributo `accept`.
  `set_directory` não existe no wasm.
- `backend/wasm/file_dialog.rs`: `save_file` resolve na hora com um
  handle "writable"; `FileHandle::write(bytes)` mostra um link de
  download. `file_dialog.rs:258-271`: `pick_folder` e `pick_folders` são
  `not(wasm32)`, "does not exist in WASM32".
- dependências do `rfd` no wasm (`Cargo.toml` dele): `js-sys`,
  `wasm-bindgen`, `wasm-bindgen-futures` e `web-sys` com doze features
  (Document, Element, HtmlInputElement, HtmlButtonElement,
  HtmlAnchorElement, Window, File, FileList, FileReader, Blob,
  BlobPropertyBag, Url). o engine já tem os quatro crates na seção wasm
  (`rust/engine/Cargo.toml:97-102`), então o `rfd` compilaria para wasm
  fora da seção de desktop, ao custo das features extras no `web-sys` e
  de um overlay DOM que não é do engine.
- conclusão: o trait do engine é síncrono e devolve caminho, porque as
  duas chamadas do urnaui querem um `PathBuf` (abrir por caminho, exportar
  para um caminho). no wasm o `rfd` só dá bytes e só assíncrono, então a
  resposta do engine no wasm é `Unsupported`, e o `web_picker.rs` do
  urnaui continua sendo a entrada de bytes da web (é outra capacidade,
  fora dos arquivos permitidos aqui). o `rfd` fica na seção de desktop:
  `[target.'cfg(all(not(target_arch = "wasm32"), not(target_os = "android"),
  not(target_os = "ios")))']`, a mesma do `arboard`.

(c) o molde do `clipboard.rs` (`rust/engine/src/runtime/platform/clipboard.rs`):

- `ClipboardProvider` (linhas 9-14): o trait, compilado em todo alvo.
- `LocalClipboard` (16-36): a implementação em memória, compilada em todo
  alvo; é o falso dos testes e o fallback de wasm e mobile.
- `SystemClipboard` (40-75): `arboard`, sob
  `#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]`;
  `new()` degrada para `None` quando o `arboard` falha (CI headless) e as
  operações viram no-op com um `log::warn!`.
- `default_clipboard() -> Box<dyn ClipboardProvider>` (78-89): o `cfg`
  escolhe `SystemClipboard` no desktop e `LocalClipboard` no resto.
- `rust/engine/Cargo.toml:82-85`: `arboard` só na seção
  `cfg(all(not(wasm32), not(android), not(ios)))`.
- por teste: quem consome guarda `Box<dyn ClipboardProvider>` ou o tipo
  concreto; teste usa `LocalClipboard`. o urnaui constrói
  `SystemClipboard::new()` direto (`explorer.rs:80,124`, `app.rs:333`).
- o módulo de diálogos segue o mesmo desenho: `FileDialogs` (trait),
  `FakeDialogs` (em todo alvo; responde o que o teste carregou e registra
  o que foi pedido), `NativeDialogs` (`rfd`, sob o cfg do `arboard`),
  `NoDialogs` (wasm e mobile: `Unsupported`), `default_dialogs()`.

o que entra de dependência (e o motivo da parada): o `Cargo.lock` da raiz
não tem `rfd`. o urnaui é workspace próprio (`apps/urnadbapp/Cargo.lock`
tem `rfd 0.15.4`); a raiz não. entrando no engine, o lock da raiz ganha:

- `rfd 0.15.4` e `block2 0.6.2` (a raiz só tem `block2 0.5.1`; o 0.6.2
  está no cache local do registry). `dispatch2 0.3.1`, `objc2 0.6.4`,
  `objc2-app-kit 0.3.2`, `objc2-core-foundation 0.3.2`,
  `objc2-foundation 0.3.2`, `raw-window-handle 0.6.2`, `log` já estão;
  `objc2-app-kit` ganha features (NSOpenPanel, NSSavePanel, NSAlert,
  NSPanel e outras). é tudo que compila no macOS.
- com as features padrão do `rfd` (`xdg-portal`, `async-std`), o lock
  ganha também `ashpd 0.11`, `urlencoding 2.1` e a árvore do `ashpd`
  (zbus 5 e o que vem com ele: async-fs, async-net, enumflags2,
  serde_repr, wayland-backend, wayland-client, wayland-protocols, rand,
  url e as dezenas de crates abaixo), como entradas só de linux e bsd,
  nunca compiladas aqui. o `.crate` do `ashpd` não está no cache local
  (o macOS nunca precisou baixar), só o índice.
- com `default-features = false`, entram só `rfd` e `block2 0.6.2`. no
  linux o `rfd` fica sem backend até o bring-up escolher `xdg-portal`
  (daemon de portal em runtime) ou `gtk3` (headers C), o que é decisão
  daquele bring-up, hoje fora de escopo.
- no wasm, o lock registra as dependências wasm do `rfd` de qualquer
  forma (todas já presentes); elas só compilam se o `rfd` estiver na
  seção wasm, e não estará.

o que o winit 0.30.13 entrega no drop
(`~/.cargo/registry/src/*/winit-0.30.13/src/platform_impl/macos/window_delegate.rs:367-430`):

- `draggingEntered:` enfileira um `HoveredFile(path)` por arquivo do
  pasteboard; `performDragOperation:` um `DroppedFile(path)` por arquivo;
  `draggingExited:` um `HoveredFileCancelled`. nenhum traz posição, e o
  `draggingUpdated:` não está implementado, então não há `CursorMoved`
  enquanto um arquivo é arrastado (o mouse pertence à sessão de drag, não
  à janela). o `event.rs:180-192` documenta o mesmo para todo backend
  (x11 e windows são os outros dois com drop). a posição lógica pedida
  pelo briefing não existe no winit 0.30: o evento do engine a carrega
  como `Option`, `None` até o winit expor a `draggingLocation`.
- os N `DroppedFile` de um drop múltiplo são enfileirados juntos, antes
  de qualquer redraw: o `App` do engine, que bufferiza eventos até o
  `RedrawRequested`, pode juntá-los num `Dropped` só; um shell que
  entrega evento a evento à view (urnaui, showcase) recebe um por
  arquivo, como hoje.

o shell do showcase: `apps/uigallery/src/shell/mod.rs:341-478` não trata
`DroppedFile`, `HoveredFile` nem `HoveredFileCancelled`, e o arquivo é
proibido nesta task. a página Files recebe o drop pelos testes, pelo
headless e pelo dispatcher; para o drop chegar numa janela de verdade o
shell precisa de um braço a mais no `match`, que repassa o evento do
engine a `ShowcaseView::handle_file_drop`.

## o trait, o falso e o que cada alvo faz

`rust/engine/src/runtime/platform/files.rs` (350 linhas de produção).
`FileDialogs` tem `pick_file`, `pick_files`, `save_file` e `pick_folder`;
cada um recebe `&DialogRequest` (título, `FileFilter { name, extensions }`
por `filter(name, &[ext])`, `directory`, `file_name`) e devolve
`Picked<T> = Result<Option<T>, DialogError>`: `Ok(None)` é a pessoa
cancelando, `Err(DialogError::Unsupported(DialogKind))` é alvo sem
diálogo. tudo síncrono e modal na thread que pediu, como o
`rfd::FileDialog`.

- `NativeDialogs` (só sob o cfg do `arboard`): monta o `rfd::FileDialog`
  por chamada (`set_title`, `add_filter`, `set_directory`,
  `set_file_name`), sem parent, e chama `pick_file`, `pick_files`,
  `save_file` ou `pick_folder`. comportamento igual ao que o urnaui fazia.
- `FakeDialogs` (todo alvo): `will_pick`, `will_pick_all`, `will_cancel`,
  `will_refuse` carregam respostas em ordem; fila vazia é cancel (uma
  página que pergunta sem o teste esperar não muda nada); cada chamada
  fica em `Asked { kind, request }` (`asked()`, `take_asked()`,
  `pending()`). `impl FileDialogs for Arc<Mutex<D>>` deixa um teste ficar
  com um handle e dar o outro à página como `Box<dyn FileDialogs>`.
- `NoDialogs` (todo alvo; o padrão em wasm, android e ios): todo tipo
  responde `Unsupported`.
- `default_dialogs() -> Box<dyn FileDialogs>`: `NativeDialogs` no
  desktop, `NoDialogs` no resto, pelo mesmo cfg do `default_clipboard`.

`rust/engine/Cargo.toml`: `rfd = { version = "0.15", default-features =
false }` na seção `cfg(all(not(wasm32), not(android), not(ios)))`, ao lado
do `arboard`, declarado inline (a raiz `Cargo.toml` é proibida). o
`Cargo.lock` da raiz ganhou `rfd 0.15.4` e `block2 0.6.2` (o `block2
0.5.1` do winit fica), nada mais; `objc2-app-kit` ganhou as features dos
painéis. no linux o `rfd` fica sem backend até o bring-up de linux
escolher `xdg-portal` ou `gtk3`; linux não é alvo. o wasm compila sem o
`rfd` (a perna wasm do gate prova).

testes (7, em `files.rs`): o falso responde em ordem e registra título,
filtro, diretório e nome; fila vazia cancela; um diálogo de um caminho
pega o primeiro e lista vazia cancela; `will_refuse` é o mesmo erro do
alvo sem diálogo, com o texto; o handle compartilhado responde por um
lado e reporta pelo outro; `NoDialogs` recusa os quatro tipos; o builder
do `rfd` carrega o pedido inteiro (conferido pelo `Debug` dele, sem abrir
nada). nenhum teste constrói um painel.

## o evento de drop

`rust/engine/src/control/input/drop.rs` (110 de produção): `FileDrop::{Hovered
{ paths, position }, Dropped { paths, position }, Cancelled}`, posição em
pixels lógicos e `Option`. `FileDrop::from_window_event(&WindowEvent)`
é a única tradução: `HoveredFile(p)` vira `Hovered` com um caminho,
`DroppedFile(p)` vira `Dropped` com um caminho, `HoveredFileCancelled`
vira `Cancelled`, o resto `None`. `join(next)` funde dois passos da mesma
fase (anexa os caminhos, guarda a primeira posição conhecida) e devolve
`next` quando as fases diferem ou um é cancel. `paths()`, `position()`,
`is_drop()`.

`input/types.rs`: `InputEvent::FileDrop(FileDropEvent { view_id:
Option<ViewId>, drop })`. `input/handlers.rs`:
`InputState::handle_file_drop(drop)` faz hit test na posição do drop,
quando há, e enfileira; sem posição, `view_id` é `None` e o app decide o
que um drop na janela inteira quer dizer. o cursor não serve de
substituto: durante o arraste o mouse pertence à sessão de drag e o
cursor da janela está parado onde estava antes.

`window/events.rs` (o `App` do engine): `BufferedEvent::FileDrop`; os três
eventos do winit passam por `from_window_event`; `buffer_event` funde o
drop novo com o último bufferizado da mesma fase, então os N
`DroppedFile` de um `performDragOperation:` chegam como um `Dropped` com
todos os caminhos antes do frame; `process_buffered_events` chama
`handle_file_drop`.

testes: 4 em `drop.rs` (os três eventos traduzem e mais nada, leitura de
cada fase, junção com a posição, fases diferentes e cancel não juntam) e
1 em `input/tests.rs` (o drop chega à view sob a posição, com caminhos e
posição, por hover e por drop; `begin_frame` preserva; sem região ou sem
posição chega a nenhuma view; o cancel chega; o pointer e o hover não se
mexem). `demo/inputs/src/state.rs` ganhou o braço
`InputEvent::FileDrop` que a variante nova exige (único match exaustivo
no repositório).

## o que mudou no urnaui

- `apps/urnadbapp/Cargo.toml`: `rfd` sai (o comentário aponta pro engine).
  `apps/urnadbapp/Cargo.lock`: 370 linhas a menos, 35 pacotes fora
  (`ashpd`, `url`, `icu_*`, `zbus`... a árvore de linux que o `rfd` com
  features padrão trazia); `rfd` continua no lock pelo engine, sem
  features.
- `apps/urnadbapp/src/view/explorer/mod.rs`: campo `dialogs: Box<dyn
  FileDialogs>` (`default_dialogs()`); `set_dialogs`, `opening` e
  `file_hover` sob `cfg(test)`; `Action::PickFile` chama `pick_file` fora
  do wasm (no wasm continua marcando `pick_requested` pro `web_picker`);
  `set_file_hover` perdeu o cfg (compila em todo alvo, a web nunca manda).
  992 linhas de produção (era 1019, congelado pelo guard de higiene).
- `apps/urnadbapp/src/view/explorer/dialogs.rs` (novo, 85 linhas):
  `pick_file` (título "Open a .urna corpus", filtro "urna corpus" com
  `urna` e `nest`; `Ok(Some)` abre, `Ok(None)` nada, `Err` toasta "the
  file picker needs the desktop app") e `export_blob`, uma versão só
  pra todo alvo (título "Export media blob", `file_name` do blob; `Err`
  toasta "blob export needs the desktop app", o texto que o mobile e a
  web já mostravam). as duas variantes cfg'd de antes viraram uma.
- `apps/urnadbapp/src/view/mod.rs`: `handle_file_drop(&FileDrop) -> bool`
  no lugar de `file_dropped` e `set_file_hover`: `Hovered` liga o hover
  da tela Open, `Cancelled` desliga, `Dropped` desliga e abre cada caminho
  em ordem com o toast "opening dropped file…" (o último ganha, como
  antes). compila em todo alvo.
- `apps/urnadbapp/src/app.rs`: os três braços cfg'd de drop saíram; o
  braço `other =>` entrega todo evento não tratado a
  `engine::input::FileDrop::from_window_event` e, se é drop, a
  `view.handle_file_drop` mais `invalidate`. `app.rs` não nomeia
  `DroppedFile`, `HoveredFile` nem `HoveredFileCancelled`; o engine
  decide o que é drop.
- `apps/urnadbapp/src/web_picker.rs` não mudou: é a entrada de bytes da
  web, outra capacidade, e o candidato a um `AsyncFileDialog` do engine
  numa fase futura.

como conferi sem abrir diálogo (3 testes novos, o falso no lugar do
nativo): `browse_opens_what_the_dialog_picked_and_nothing_when_cancelled`
(cancel não abre nada; um caminho inexistente abre e falha como um
digitado, voltando à tela Open com o erro; os dois pedidos registrados
com o título, o filtro e sem diretório; `NoDialogs` toasta e não abre),
`export_asks_a_save_dialog_named_after_the_blob` (cancel sem toast nem
exportação; um caminho manda `ExportBlob` ao worker, que sem arquivo
aberto responde a falha; refuse toasta; blob inexistente nem pergunta),
`a_file_drag_shows_the_hover_and_a_drop_opens_the_file` (a view: hover
liga, cancel desliga, drop desliga, abre e toasta).

## a página Files (PNG e projeção)

`apps/uigallery/src/views/apps/files/mod.rs` (394 linhas), `files/semantics.rs`
(289) e `files/tests.rs` (729, 8 testes), grupo Apps Demo, depois de
Silences, ícone `folder-open`.

modelo semântico: `files` (Region; `received`, a lista dos caminhos na
ordem, e `count`), `files.drop` (Region "drop files here"; `drop_files`
com param `List` não constrained, lendo `files:received`, todo ator: o
drop da pessoa e o de um agente pela porta são a mesma capacidade),
`files.open` (Button "open file", focusable; `open_file` lendo
`files:received`, policy `Actors { human: true, assistive: true, test:
true, agent: false }`) e `files.status` (`message`). as duas ações leem a
lista porque as duas mudam a lista. foco em `files.open` quando o botão
tem o teclado.

execução (`Operable`): `open_file` chama
`dialogs.pick_file(DialogRequest::new().title("Open a file"))`;
`Some(path)` recebe (outcome `path`, `count`), `None` completa com
`cancelled: true` e o status "the dialog was cancelled", `Err` falha com
o texto do erro e o status "no file dialog on this target".
`drop_files` recebe os caminhos do param (outcome `received`, `count`);
lista vazia falha "the drop carries no file".

entrada: clique no botão vira `open_file` por `Actor::Human` pelo
dispatcher da galeria (o do todo, emprestado como pra Silences).
`handle_file_drop`: `Hovered` acende a zona (borda de acento, "release
to receive"), `Cancelled` apaga, `Dropped` com posição fora da zona não
recebe, com posição dentro ou sem posição (o que o winit 0.30 manda)
vira `drop_files` por `Actor::Human` com os caminhos como `List`. Tab
foca e desfoca o botão, Enter pressiona (`open_file`), Escape solta.
`FilesPage::new()` usa `default_dialogs()` fora de teste e `FakeDialogs`
sob `cfg(test)`: nenhum teste do crate abre diálogo nativo, por
construção.

registro como a Silences: `Pane::Files` (`pages/mod.rs`), a linha em
`pages/table.rs`, o campo em `panes.rs`, `dispatch.rs` (`content_height`,
`page_event`, `page_file_drop` novo, `page_render`, `semantics`, `act`,
`assist_focus`), `input.rs` (`ShowcaseView::handle_file_drop` novo,
`handle_edit_key`, `handle_enter`, `close_top_overlay`). shell,
`apps/uigallery/src/shell/mod.rs:476-485` (aprovado, opção 2): o braço
`other =>` do `match` entrega todo evento não tratado a
`FileDrop::from_window_event` e `view.handle_file_drop`, com `changed()`.

testes (`files/tests.rs`): a projeção (roles, labels, policy do
`open_file` igual a `OPEN_FILE_ACTORS`, param e reads do `drop_files`,
status, sem foco); o drop pela pessoa (hover acende uma vez, cancel
apaga; sem posição recebe dois caminhos e o recibo 1 é `Human`,
`files.drop`, `drop_files`, param com os dois, `Completed` com `received
2`, `count 2`; fora da zona não recebe e nenhum recibo nasce; dentro
recebe; inert é `Rejected(Inert)`; vazio é `Failed`; o falso nunca é
perguntado; a zona é o binding de `files.drop`); o botão (clique pega
"/home/me/notes.md" e o recibo traz `path`; cancel completa com
`cancelled`; Tab foca, Enter com refuse é `Failed` com "pick_file: no
native file dialog on this target" e o status diz; Escape solta; três
pedidos `PickFile` com "Open a file" e sem filtro; `NoDialogs` falha
igual); o agente em processo (`open_file` é `Rejected(NotAuthorized)` e
o falso não é perguntado; `drop_files` completa como do agente; o mesmo
pedido com a observação velha é `Rejected(Stale)`; `Actor::Test` pode e
o falso responde); o agente pela porta real (`AgentPort` num diretório
temporário, thread do agente com `wire::Client`: observa `files` com
bounds e vê os quatro nós e as caixas da zona e do botão; `open_file` é
`requested, rejected`, um recibo e não um erro; `drop_files` com param
é `requested, accepted, executing, completed`; a lista mostra o caminho;
o dispatcher da galeria guarda os dois recibos como do agente "claude");
duas viewports (900 e 358 de largura: card na largura do conteúdo, zona,
botão e status dentro dele, botão abaixo da zona, status ao lado, o card
cresce com três arquivos e a altura é a do binding); as rotas da galeria
(a página Todo não recebe drop; na Files o drop chega pela view, `act`
assistivo completa, `assist_focus`, Tab, Escape, Enter cancela no falso,
os quatro bindings, em repouso a página não pede frame); o golden.

golden: `rust/plevir/tests/fixtures/uigallery-files.snapshot.json` (4 nós,
revision `d7a661b0bdabefd9`), mantido por
`files::tests::the_page_projection_is_the_uir_golden`; `UIR_BLESS=1
cargo test -p uigallery --lib files::tests::the_page_projection` regrava.
os testes do `uir` continuam verdes.

baselines: `tool/bench/regress/baselines/files@core@1280x800` e
`files@hoff@1280x800` (`semantics.json`, `scene.json`, `frame.png`; 4
layers, 4 controles bound, 4 nós, settled em 2 passos), abençoadas com
`cargo run -p regress -- bless --page Files --theme
core --png` e o mesmo com `hoff`.

render headless: `cargo run -p uigallery -- --render Files --theme core
--out files-core.png --semantics files-core.json` e o mesmo com `hoff`
deram dois PNG 1200x800 (o card centrado na largura legível, a zona com
"drop files here", o botão "open file" com o ícone de pasta, o status
"no files yet: drop some on the window, or open one" ao lado); o
`frame.png` de cada baseline é a mesma cena a 1280x800 e fica na árvore.
a projeção do render é o golden byte a byte (só o `\n` final que o teste
acrescenta).

## gate e regress

`df -h /` antes de cada gate: 15 GB livres no primeiro, 12 GB no
segundo. o primeiro `./tool/tasks/fullcheck.sh` ficou vermelho no
`file_hygiene_guard`: `apps/urnadbapp/src/view/explorer/mod.rs` está
congelado em 1019 linhas de produção ("pode encolher, nunca crescer") e
os diálogos o levaram a 1056; o commit `77800d1b` moveu `pick_file` e
`export_blob` pra `view/explorer/dialogs.rs` e o arquivo ficou em 992.
o segundo gate, verde no worktree, com a perna wasm e a perna urnaui
(`../urna` presente): antes `cargo test --workspace` 2465 passed, 0
failed, 16 ignored em 115 binários; depois 2485 passed, 0 failed, 16
ignored em 115 binários (+20: 7 do `files.rs` do engine, 5 do drop, 8 da
página). perna urnaui: antes 82 passed, 1 ignored; depois 85 passed, 0
failed, 1 ignored em 6 binários (+3). clippy `-D warnings`, fmt e a
checagem wasm limpos nas duas pernas.

`cargo run -p regress -- check`: 161 baselines (eram
159), 161 iguais, 0 mudaram, sem frame desenhado.

linhas de produção (regra do guard): engine `platform/files.rs` 350,
`input/drop.rs` 110, `input/handlers.rs` 165, `input/types.rs` 256,
`window/events.rs` 227; showcase `apps/files.rs` 394,
`files/semantics.rs` 289, `view/dispatch.rs` 415, `view/input.rs` 394;
urnaui `view/explorer.rs` 992 (era 1019, congelado, encolheu),
`view/explorer/dialogs.rs` 85, `app.rs` 562. tudo abaixo de 639, salvo o
congelado.

## ADR

[`0055-file-dialogs-and-drops-are-capabilities.md`](../../adr/Interface/0055-file-dialogs-and-drops-are-capabilities/0055-file-dialogs-and-drops-are-capabilities.md) (proposed): o
trait e o falso, o `rfd` sem features no desktop, o evento de drop com
posição opcional e a junção, o drop como request, a regra "diálogo
nativo só abre por pessoa" na policy, o que o wasm faz, o que os itens 4
e 8 reaproveitam. linha pro índice, que é do orquestrador
([`adr/`](../../adr/adr.md) é proibido aqui):

| 2026-09-24 | [file-dialogs-and-drops-are-capabilities](../../adr/Interface/0055-file-dialogs-and-drops-are-capabilities/0055-file-dialogs-and-drops-are-capabilities.md) | proposed | `engine::platform::files` (trait síncrono por caminho, `rfd` no desktop sem features padrão, `FakeDialogs` pra teste, `NoDialogs` em wasm e mobile) e `engine::input::FileDrop` (uma tradução dos três eventos do winit, posição opcional, junção dos N por arquivo no `App`); o urnaui usa os dois sem segunda implementação; a página Files do showcase projeta `open_file` só pra pessoa e `drop_files` pra todo ator, pelo dispatcher; diálogo nativo só abre por pessoa (task-76 item 3) |

## interfaces adicionadas (só crescimento)

`engine::platform::files::{FileDialogs, DialogRequest, FileFilter,
DialogKind, DialogError, Picked, FakeDialogs, Answer, Asked, NoDialogs,
NativeDialogs (desktop), default_dialogs}` e `impl FileDialogs for
Arc<Mutex<D>>`; `engine::input::{FileDrop, FileDropEvent}`,
`InputEvent::FileDrop`, `InputState::handle_file_drop`;
`ShowcaseView::handle_file_drop`; `UrnauiView::handle_file_drop` (no
lugar de `file_dropped` e `set_file_hover`, cujo único consumidor era o
`app.rs` do próprio urnaui). `InputEvent` ganhou uma variante; o exemplo
`input` do engine acompanhou e não há outro match exaustivo. nenhuma
assinatura existente mudou.

## arquivos tocados fora da lista permitida, e por quê

- `demo/inputs/src/state.rs`: o braço `InputEvent::FileDrop`
  (a variante nova quebrava o match exaustivo; clippy `--all-targets`
  compila o exemplo).
- `apps/urnadbapp/src/view/mod.rs`: `file_dropped` e `set_file_hover`
  viraram `handle_file_drop` (a tradução do drop do urnaui morava aí
  também, não só em `app.rs`); um teste novo.
- `apps/urnadbapp/src/view/explorer/dialogs.rs` (novo): o guard de higiene
  congela `explorer.rs`; sem o módulo filho o gate fica vermelho.
- `apps/uigallery/src/shell/mod.rs`: o braço aprovado (opção 2).
- `apps/urnadbapp/Cargo.lock`: consequência do `Cargo.toml`.

## limitações e decisões, sem esconder

- posição: `None` em todo backend do winit 0.30; a zona da página só
  recusa um drop fora dela quando há posição, o que na janela hoje nunca
  acontece (provado com `Some` nos testes). quando o winit expuser a
  `draggingLocation`, `from_window_event` preenche e nada mais muda.
- drop múltiplo: o `App` do engine entrega um `Dropped` com todos os
  caminhos; urnaui e showcase, que entregam evento a evento, recebem um
  por arquivo (o urnaui abre cada um e o último ganha, como antes; a
  Files recebe um `drop_files` por arquivo).
- wasm: `Unsupported`; o `web_picker.rs` do urnaui fica e é o candidato a
  um `AsyncFileDialog` do engine (bytes, assíncrono).
- mobile: o Browse do urnaui agora toasta "the file picker needs the
  desktop app" (antes o botão existia e não fazia nada em android e ios);
  a exportação já toastava o equivalente.
- linux: sem backend de diálogo até o bring-up escolher (features do
  `rfd` desligadas); o urnaui tinha `xdg-portal` por conta própria. só
  linux muda, e linux não builda hoje.
- `NativeDialogs` não passa `set_parent` (o urnaui nunca passou): o
  painel abre como modal da aplicação. o diálogo real não foi aberto
  nesta máquina (proibido); a prova é o builder por `Debug` e a leitura
  do `rfd`; a abertura de verdade fica pro orquestrador.
- `FilesPage::new()` só usa o falso sob `cfg(test)` do próprio crate: um
  teste de integração em `apps/uigallery/tests/` que construísse a view
  e disparasse `open_file` como pessoa abriria o painel nativo. nenhum
  faz (o `stage_page` nunca dispara nada), e a policy barra o agente
  antes; fica registrado como risco de quem escrever esse teste.
- o `Cargo.lock` da raiz cresceu dois pacotes e `objc2-app-kit` ganhou
  features; o custo de binário não foi medido.
- `explorer.rs` continua congelado (992, era 1019); o split completo
  por responsabilidade é dívida anterior a esta task.

## o que os itens 4 e 8 reaproveitam daqui

- item 4 (menus nativos): o item "Open…" de um menu é outra
  representação de `open_file`: dispara um `Request` por `Actor::Human`
  no mesmo dispatcher, e a policy "só pessoa" já está declarada no nó,
  não no menu. o molde é o mesmo desta fase: trait no
  `engine::platform`, implementação real sob o cfg do `arboard`, um
  falso que registra o que foi pedido, `default_*()` por alvo. o `rfd`
  não faz menu; o crate de menu é a dependência a aprovar.
- item 8 (alertas e confirmações nativas): `rfd::MessageDialog` e
  `AsyncMessageDialog` já vêm na dependência do engine; um trait `Alerts`
  com `NativeAlerts`, `FakeAlerts` e `NoAlerts` segue `files.rs` linha
  por linha, com o mesmo `Picked`-like (`Ok(resposta)`, `Unsupported`).
  a regra "painel nativo só por pessoa" vale igual, e a confirmação
  humana da task-58 (`Confirm::ForAgents`) pode ser o `MessageDialog`
  com OkCancel como uma das opções.
- os dois reaproveitam a forma do drop: um evento do engine, uma
  tradução `from_window_event`, o shell repassa pelo braço `other =>`
  sem nomear nada do winit.

## comandos de validação

```text
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/task-76
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_PROFILE_TEST_DEBUG=line-tables-only
cargo test --offline -p engine -p uigallery
CARGO_TARGET_DIR=$CARGO_TARGET_DIR/urnaui cargo test --offline --manifest-path apps/urnadbapp/Cargo.toml
./tool/tasks/fullcheck.sh
cargo run --offline -p regress -- check
cargo run --offline -p uigallery -- --render Files --theme core --out files.png --semantics files.json
UIR_BLESS=1 cargo test --offline -p uigallery --lib files::tests::the_page_projection   (regrava o golden)
cargo run --offline -p regress -- bless --page Files --theme core --png   (e --theme hoff)
```

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0055: file dialogs and file drops are capabilities of the engine, and a native dialog opens only for a person](../../adr/Interface/0055-file-dialogs-and-drops-are-capabilities/0055-file-dialogs-and-drops-are-capabilities.md)
- [task-58: action with precondition, dispatch and receipts](../task-58-actions-preconditions-receipts/task-58-actions-preconditions-receipts.md)
- [task-76: platform integration](task-76-platform-integration.md)
<!-- /related -->
