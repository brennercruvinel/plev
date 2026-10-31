---
id: task-76/item-7-sleep-and-wake
title: "task-76, item 7: sleep and wake"
project: plev
type: section
status: reference
parent: task-76
date: 2026-09-24
date_basis: text
topics: [platform, rendering, performance]
relations: [adr-0060]
source: docs/log/steps/task-76-platform-integration.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
evidence: [7d88e4f3, f4c03f08, 022c423f, 1bd8a11f]
---

# handoff of item 7: sleep and wake

Records item 7: an occluded window or a sleeping app requests neither a frame nor a deadline from the platform, and, on showing or waking, each host is invalidated and requests a frame built from the state at that time. `engine::window::Background` tells background work that the app is asleep: the Surface's Claude is held back and starts on waking, and the ffmpeg response waits in the channel. The result was proven headless with `Occluded` injected, comparing byte for byte against a control host that was never occluded; system sleep on macOS is not heard, because the `NSWorkspace` feature of `objc2-app-kit` is not enabled, and enabling it changes the manifest.


branch `task/76g-sleep-wake`, base `e1386d5a` (main após o item 5).
worktree `.claude/worktrees/agent`, target `target/agents/one`. commits,
em ordem:

- `7d88e4f3` task-76: the sleep and wake probe
- `f4c03f08` task-76: a covered window and a sleeping app ask for no
  frame, and show with a new one
- `022c423f` task-76: the showcase sleeps, is covered, and wakes with the
  work beside the frames
- `1bd8a11f` task-76: the gallery's first frame after a sleep or an
  uncover is the state now, byte for byte
- (este) task-76: ADR and the item 7 handoff

os testes do item 5 do briefing saíram junto do comportamento que provam
(commits 2 e 3), como as diretrizes pedem; o commit 4 é o teste de
integração do showcase com pixels.

## a sonda

(a) o que o winit 0.30.13 entrega, lido na fonte (nenhuma janela aberta,
a máquina nunca dormiu):

- macOS: `platform_impl/macos/window_delegate.rs:348`,
  `windowDidChangeOcclusionState:` lê `NSWindowOcclusionState::Visible`
  ("ao menos parcialmente na tela") e enfileira `WindowEvent::Occluded(!visible)`.
  logo minimizada, escondida (cmd-H) ou totalmente coberta por outras
  janelas é `Occluded(true)`; parcialmente visível não é. `request_redraw`
  (`window_delegate.rs:916`) chama `app_delegate.queue_redraw(id)`: é a
  fila do próprio winit, não o `setNeedsDisplay` do AppKit, então uma
  janela coberta que anima recebe `RedrawRequested` a cada pedido e
  desenha numa surface que ninguém vê. `resumed` é emitido uma vez no
  `Init` (`app_state.rs:327`: "for consistency all platforms must emit a
  'resumed' event even though macOS..."); `suspended` nunca
  (`application.rs:125`: só android, ios e web têm ciclo de vida). o sleep
  do sistema não chega por winit: existe como
  `NSWorkspaceWillSleepNotification` e `NSWorkspaceDidWakeNotification`
  nos dois `objc2-app-kit` do lock (0.2.2 do winit, 0.3.2 do tray-icon e
  muda; `src/generated/NSWorkspace.rs`), atrás da feature `NSWorkspace`,
  que nenhum dos dois consumidores liga (a lista do winit vai de
  `NSAppearance` a `NSWindowTabGroup`, sem ela; a do tray-icon é a barra
  de status). e o engine não tem `objc2-app-kit` como dependência direta
  (`rust/engine/Cargo.toml` só o cita num comentário do item 5). ligar a
  feature é mudar o manifesto, arquivo proibido aqui: o gancho fica
  registrado, o item segue com o que o winit dá. o que acontece na tela
  quando o monitor dorme ou a tela trava não foi medido (proibido dormir
  a máquina); o AppKit muda o estado de oclusão nesses casos, mas o
  handoff não afirma o que não mediu.
- iOS: `Occluded(false)` em `applicationWillEnterForeground`,
  `Occluded(true)` em `applicationDidEnterBackground` (`event.rs:408`);
  web: a interseção do canvas com o viewport, sem contar borda, padding e
  transform; android, wayland e windows: não suportado (`event.rs:420`).
  `suspended`/`resumed` de verdade: android, ios e web.

(b) por teste headless (`rust/engine/tests/sleep_wake.rs`, o commit
`7d88e4f3` era a sonda, substituída pelos testes reais em `f4c03f08`):
dois `WindowHost::headless` sob `Windows`, a janela A com
`set_animating(true)` e um deadline, um frame vermelho desenhado.
`route(A, Occluded(true))` respondia `Handled` mas `handle_event` dizia
`false`: o evento não era de ninguém. nada pausava: `needs_redraw()`
seguia true, `request_pending_redraws()` dava 1, `earliest_deadline()`
mantinha o deadline. com o estado mudado durante a cobertura e nenhum
frame desenhado, `read()` devolvia o frame vermelho de antes (a textura
guarda o último frame); o primeiro frame desenhado depois era novo,
porque a cena se constrói do estado, mas ninguém o pedia ao descobrir
(o winit não pede frame no `Occluded(false)`): na tela, uma página
parada mostraria o frame de antes de cobrir até algo pedir um frame.
`Windows::suspend()`: os hosts saem do mapa (`is_empty`,
`has_suspended`), então `request_pending_redraws()` 0 e
`earliest_deadline()` None por ausência, `should_exit()` false, e um
evento para A cai em `Unknown`. o `resume` precisa de um `ActiveEventLoop`
vivo e não foi medido headless.

(c) os trabalhos de fundo dos três apps, e o que acontecia com o app
suspenso ou a janela coberta:

- showcase, Surface: `asking::Claude::start` sobe a thread
  `surface-page-claude` rodando `propose_verified(Llm)` sobre `claude -p`;
  devolve `Msg::Attempt(n)` e `Msg::Done(Answer)` num canal mpsc. não
  acorda o laço: a página olha o canal no `wake` e pede `Wake::after(100 ms)`
  enquanto espera, o que vira o `WaitUntil` do laço. suspenso, o laço
  acordaria a cada 100 ms pelo deadline, o `settle` aplicaria e pediria
  frame, e a janela desenharia coberta.
- showcase, Silences: `worker.rs` sobe `silences-ffmpeg`, um filho por
  vez, `Done::{Detected, Frame}` no canal; igual: polling de 100 ms pelo
  `wake` enquanto `busy()`.
- showcase, Git: o worker do `git` acorda o laço pelo
  `EventLoopProxy` (`UserEvent::Wake`), o shell faz `apply_git` e
  `changed()` (frame se a Git mostra). o resto que acorda o laço: a porta
  do agente (`UserEvent::Agent`), o leitor de tela (`Access`), barra,
  tray e notificação (`Clicked`).
- ide: `GitClient::spawn` acorda o laço por `AppEvent::Git(event)`; o
  shell aplica na hora, serve o agente, grava e `invalidate` (o winit
  enfileira o `RedrawRequested` e a janela desenha, coberta ou não). o
  watcher de disco (`AppEvent::FsChange`) e a porta (`AppEvent::Agent`)
  idem. o ide não usa `Windows` (handoff do item 2): registrado, não
  mudado.
- o `EventLoopProxy` acorda o laço com o app "suspenso" em qualquer
  plataforma (um user event é um evento); o que muda com este item é o
  que o shell faz ao acordar.

## os estados e a regra dos frames

`rust/engine/src/runtime/window/sleep.rs` (163 linhas), mais dois campos no
host (`host.rs` 571) e um no `Windows` (`windows.rs` 323):

- **coberta** é por janela: `WindowHost::occluded`, de
  `WindowEvent::Occluded` em `handle_event` (então `Windows::route`
  entrega; um shell que não roteia chama `set_occluded`). **dormindo** é
  por app: `Windows::asleep`, de `Windows::sleep()`/`wake()` (o shell) e
  de `suspend()`/`resume()` (a plataforma tomou as janelas); cada host
  ganha `sleeping` junto, e um host inserido num app dormindo dorme com
  ele. **acordada**: `WindowHost::awake()` = nem coberta nem dormindo;
  `Windows::awake(id)` para uma janela (false para id desconhecido).
- a regra: um host que não está acordado não pede nada à plataforma.
  `request_redraw` é retido (e ao cobrir o pedido pendente é retirado,
  `redraw_asked` false), `needs_redraw()` é false, `deadline()` é None,
  então `request_pending_redraws` e `earliest_deadline` passam por ele e
  um `Wake` contínuo vira "esperando mostrar" (`is_animating()` fica
  true). um `RedrawRequested` que a plataforma ainda mande não é
  desenhado: o `App` do engine (`lifecycle.rs`, `Routed::Redraw if
  awake`) e o `redraw` do showcase (`pace.rs`) checam `awake`. ao
  descobrir (`Occluded(false)`) ou ao acordar, cada host é invalidado e
  pede um frame: o primeiro frame depois é construído do estado como está
  então, nunca o de antes. a regra do render on demand não muda; é mais
  um motivo para não desenhar.
- os relógios saltam, não pausam. coberta, a view segue: o `catch_up` do
  shell tica com o tempo real quando algo acorda o laço, e um spring que
  teria terminado terminou quando a janela mostra. dormindo, o showcase
  não tica o relógio (`catch_up` e `advance` voltam cedo), então o tempo
  todo dormido cai inteiro no primeiro tick depois do wake, com o mesmo
  efeito. pausar exigiria os relógios da animação (`animation/**`, comps)
  saberem da cobertura, e mostraria a quem volta um floreio do momento em
  que saiu. decidido: saltam.
- `Windows::invalidate_all` e `request_redraw(id)` continuam marcando o
  compositor sujo; só o pedido à plataforma é retido. o `invalidate()`
  basta para o frame ser novo: o passe de composite redesenha toda layer
  na surface a cada frame, e uma textura de layer com conteúdo igual
  continua válida num device vivo; se o device não sobreviveu ao sleep,
  o `recover` da fase 1 (`drop_gpu_resources`, tudo sujo) roda antes do
  acquire desse mesmo frame.

## o contrato dos trabalhos de fundo e como cada app o aplica

`engine::window::Background` (em `sleep.rs`): `Background::sleeping(bool)`
é dito por `Windows::sleep`/`wake` (e por `suspend`/`resume`);
`Background::asleep()` é lido por quem começa um job ou aplica um
resultado. thread-local (`Cell<bool>`): um app por processo, as páginas
rodam na thread do laço, e num binário de teste cada teste é uma thread
com o seu app. o contrato: um resultado que chega dormindo fica onde o
worker o pôs (o canal) e é aplicado no primeiro `wake` depois de acordar;
o shell não faz `settle` nem desenha enquanto dorme, e os `settle` das
páginas não leem nada enquanto `Background::asleep()`; um job pedido
dormindo não começa, ou começa ao acordar, por app. nenhum worker mudou.

- showcase, shell (`app/sleep.rs` 50 de produção, 205 com os testes;
  `app.rs` 633; `pace.rs` 181; `events.rs` 213): `Occluded` vai para
  `covered(id, on)` (o host); `suspended` faz `sleep()`; um `resumed` com
  a janela principal já aberta faz `wake()`: `windows.wake()` (um frame
  novo por janela) e `catch_up()` (as respostas que chegaram dormindo são
  aplicadas, um ask retido começa, o que terminou é contado por
  notificação se ninguém tem o foco). `redraw` não desenha janela que não
  está acordada; `catch_up`/`advance` não ticam nem acordam a view
  dormindo. as surfaces não são soltas no `suspended` do showcase: isso é
  do bring-up mobile, e hoje o showcase não as soltava.
- showcase, Claude (`surface_page/asking.rs` 402; `page.rs` 332 de
  produção): `Claude::start` com `Background::asleep()` devolve um
  `Pending` com `Work::Held(Box<Held>)`, tudo o que a thread levaria
  (`ClaudeCli`, intent, snapshot, caps, surface atual, policy) sem thread;
  `Execution::Started` e o request fica aberto, o botão de propor
  desabilitado, o cancelar habilitado. `Pending::poll` (chamado do
  `settle`) com o app acordado troca `Held` por `Running(rx)` chamando o
  `spawn` (a mesma thread de antes, extraída de `start`), `started` a
  partir daí; dormindo, `(false, None)`. um cancel enquanto retido marca
  `cancelled`, o `dispatch` falha o request "cancelled by ..." e nenhum
  filho nasce. `Page::settle` volta `false` dormindo.
- showcase, ffmpeg (`silences/jobs.rs` 192 de produção): `settle` volta
  `false` dormindo; a resposta espera no canal. o adiar de um job novo
  não entrou: o `detect_silence` e o `preview_segment` submetem em
  `semantics.rs::execute` (`self.worker.submit(...)`), fora dos arquivos
  do item, e guardar o job adiado pede um campo em `SilencesPage`
  (`silences.rs`), também fora. a mudança é uma linha em cada:
  `if Background::asleep() { self.deferred.push(job); return Execution::Started }`
  no `execute`, e `for job in self.deferred.drain(..) { self.worker.submit(job) }`
  no começo do `settle` acordado. hoje não tem efeito observável: o
  desktop não dorme (sem gancho) e fora do desktop não há ffmpeg.
- ide (git): segue, por decisão: o resultado é o estado do repositório,
  dormindo ou não. e o ide não tem `Windows`; nada mudou (registrado).
  urnaui idem.

## os testes e os pixels

- `rust/engine/tests/sleep_wake.rs` (5): janela A animando com deadline,
  `Occluded(true)` roteado: `!needs_redraw`, `!redraw_asked`, `deadline()`
  None, `request_pending_redraws()` 1 (só B), `earliest_deadline` o de B,
  `set_animating`/`invalidate`/`request_redraw` retidos, a mesma palavra
  duas vezes não muda nada; A desenhada vermelha, coberta, o estado vira
  azul sem frame (a textura guarda o vermelho, `frames()` 1),
  `Occluded(false)`: `redraw_asked`, `needs_render`, um pedido, o frame
  desenhado é azul e igual byte a byte ao de um host de controle que
  nunca foi coberto, diferente do anterior, e em repouso nada mais é
  pedido; `Windows::sleep()`: `asleep`, `Background::asleep()`, nenhum
  pedido, deadline None, `invalidate_all` dormindo não pede, dormir duas
  vezes é dormir, um host inserido dormindo dorme, `wake()`: cada host
  acordado, com pedido e cena suja, 3 pedidos, o primeiro frame de A e B
  verde e igual byte a byte ao controle, diferente dos de antes de dormir,
  A segue animando e B descansa; `awake` nas quatro combinações e nas duas
  ordens (coberta e dormindo: acordar o app não basta, descobrir depois
  pede o frame então; `suspend()` dorme e o flag segue), `resume` só
  como `wake()` (precisa do laço); o flag é da thread (outra thread lê
  false).
- `apps/uigallery/tests/sleep_wake.rs` (2, desktop): `ShowcaseView` na
  Todo sobre um host headless 1200x800 sob `Windows`; frame; `sleep()`;
  `todo.item[1]` `toggle` por `Actor::Human` pelo dispatcher, `settle`;
  `invalidate_all` não pede, deadline None, a textura guarda o frame de
  antes; `wake()`: um pedido; o frame igual byte a byte ao de uma segunda
  view no mesmo estado num host que nunca dormiu, diferente do de antes,
  e diferente dentro do rect do item; em repouso de novo. o mesmo com
  `Occluded(true)`/`Occluded(false)` roteados (coberta não é dormindo:
  `Background::asleep()` false).
- `apps/uigallery/src/shell/system/sleep.rs` (2, o shell sobre janela nua com
  `FakeNotifier`, sem foco): uma detecção pedida na Silences termina no
  canal na hora (ffmpeg não é procurado nos testes), o `detect` fica
  desabilitado; `sleep()`, cinco `advance`+`catch_up`: segue desabilitado
  (não aplicado), nenhum frame pedido, ninguém avisado, `changed()` não
  pede; `wake()`: `detect` habilitado, o status fala de ffmpeg, um frame
  pedido, uma notificação enviada, acordar acordado não muda nada. e a
  principal coberta: um toast empurrado e `changed()` não pedem frame,
  setenta `advance(0.1)` sem pedido e sem deadline, o toast viveu o tempo
  dele coberto, descobrir pede um frame, servido, a mesma palavra de novo
  não pede.
- `surface_page/page.rs` (2, desktop, contra o `fake-claude` do
  `claude-model`): `Background::sleeping(true)`, `propose` pelo Claude:
  `Executing`, `waiting()`, `held()`, cinco `wake` sem `now`, nenhum
  `prompt.1` no diretório do fake (o filho nunca rodou);
  `sleeping(false)`, um `wake`: não mais `held`, e o request completa com
  `attempts` 1, `prompt.1` existe, a surface mostrada. e a resposta que
  chega dormindo: `propose` acordado, o filho roda (`prompt.1`), o app
  dorme, três `wake` sem `now` e `Executing`, a surface sem mudar;
  acordado, completa e mostra.
- `silences/jobs.rs` (1): `Worker::fake(hold)`, `detect_silence`,
  `sleeping(true)`, `release()`, `wake`: `!now`, `Executing`, `busy()`;
  `sleeping(false)`, `wake`: `now`, `Completed` com 2 cortes.

## gate e regress

`df -h /` antes do gate: 25 GB livres; 24 GB depois. `./tool/tasks/fullcheck.sh`
verde no worktree com `CARGO_NET_OFFLINE=true`, com a perna wasm e a
perna urnaui (`../urna` presente): antes (o fechamento do item 5, base
`e1386d5a`) 2565 passed, 0 failed, 16 ignored em 122 binários; depois
2577 passed, 0 failed, 16 ignored em 124 binários (+12: 5 de
`engine/tests/sleep_wake.rs`, 2 de `showcase/tests/sleep_wake.rs`, 2 de
`app/sleep.rs`, 2 de `surface_page/page.rs`, 1 de `silences/jobs.rs`; +2
binários, os dois `tests/sleep_wake.rs`). perna urnaui: 85 passed, 0
failed, 1 ignored em 6 binários, igual a antes (o urnaui não usa
`Windows` e compila sem mudança). clippy `-D warnings`, fmt e a checagem
wasm limpos nas duas pernas (o `Work` do `asking.rs` tem uma variante só
fora do desktop; o `match` com o braço em `cfg` compila nos dois). o
único aviso no log é o `block v0.1.6` de incompatibilidade futura, que
já estava.

`cargo run --offline -p regress -- check`: 161
baselines, 161 iguais, 0 mudaram, sem frame desenhado e sem GPU aberta
(840.5 ms). nada deste item muda um frame acordado: os goldens e os
`semantics.json` ficaram como estavam, nenhum bless.

linhas de produção (regra do guard, `file_hygiene_guard` verde): engine
`window/host.rs` 571 (era 551), `window/sleep.rs` 163 (novo),
`window/windows.rs` 323 (era 311), `window/events.rs` 197 (era 194),
`window/lifecycle.rs` 227 (era 225), `window/mod.rs` 161 (era 159);
showcase `app.rs` 633 (era 621; seis linhas do limite, o próximo item que
tocar o shell abre um módulo), `app/sleep.rs` 50 (205 com os testes),
`app/pace.rs` 181 (era 168), `app/events.rs` 213 (era 208),
`view/apps/surface_page/asking.rs` 402 (era 288), `surface_page/page.rs`
332 (era 320; 490 com os testes), `silences/jobs.rs` 192 (era 181; 246
com os testes). tudo abaixo de 639.

## ADR

[`0060-a-covered-window-does-not-draw-and-wakes-new.md`](../../adr/rendering/0060-a-covered-window-does-not-draw-and-wakes-new.md) (proposed): os
três estados e quem os entrega em cada plataforma, a regra dos frames e
das deadlines, os relógios que saltam, o contrato do `Background` e como
cada app o aplica, o que o sleep do sistema tem e não tem. linha pro
índice ([`adr/`](../../adr/README.md) está fora da lista, fica para a integração):

```text
| 2026-09-24 | [a-covered-window-does-not-draw-and-wakes-new](a-covered-window-does-not-draw-and-wakes-new.md) | proposed | task-76 item 7: `WindowHost` sabe quando está coberta (`WindowEvent::Occluded`, no macOS minimizada, escondida ou totalmente atrás; no iOS o background) e quando o app dorme (`Windows::sleep`/`wake`, e `suspend`/`resume`); enquanto uma das duas, `request_redraw` é retido, `needs_redraw` é false e o `deadline` é None, então nenhum frame nem `WaitUntil` por ela e um `Wake` contínuo espera; ao mostrar ou acordar cada host invalida e pede um frame, construído do estado de então, nunca o de antes; os relógios saltam (o catch-up tica com o tempo real); `engine::window::Background` é o mesmo fato para o trabalho ao lado dos frames (thread-local, um app por processo): resultado que chega dormindo fica no canal e aplica no primeiro wake acordado, o Claude da Surface fica retido em `Pending` e começa ao acordar, o git do ide segue; o desktop não ouve o sleep do sistema (a feature `NSWorkspace` do `objc2-app-kit` não está no lock) e o `suspended` de verdade é do bring-up mobile; provado headless com `Occluded` injetado e byte a byte contra um host de controle |
```

## interfaces adicionadas (só crescimento)

- `engine::window::Background::{sleeping, asleep}`;
  `WindowHost::{awake, occluded, sleeping, set_occluded}` (e
  `set_sleeping`, `pub(crate)`); `Windows::{asleep, awake, sleep, wake}`.
  `WindowHost::handle_event` toma `WindowEvent::Occluded` (antes
  `false`); `needs_redraw`, `deadline` e `request_redraw` passaram a
  respeitar `awake` (semântica nova, registrada); `Windows::insert` põe o
  host no sono do app; `suspend` dorme e `resume` acorda.
- showcase: `App::{sleep, wake, covered}` (pub(super));
  `ApplicationHandler::suspended` implementado; `asking::Pending::held`
  (test); o `Msg` e o `Answer` iguais; `Claude::start` com a mesma
  assinatura. nenhuma assinatura pública existente mudou; ide, urnaui,
  wasm e os exemplos compilam sem tocar.

## arquivos tocados fora da lista permitida, e por quê

nenhum. `rust/engine/src/runtime/window/sleep.rs`, `apps/uigallery/src/shell/system/sleep.rs`,
`rust/engine/tests/sleep_wake.rs` e `apps/uigallery/tests/sleep_wake.rs`
são novos dentro dos diretórios permitidos; `rust/engine/src/lib.rs`
não precisou mudar (`window` já é `pub mod`);
`rust/engine/src/runtime/platform/lifecycle.rs` não mudou (o `AppState` do
`LifecycleManager` já tem `Suspended`, e o `Windows` é onde o estado
vive agora).

## limitações e decisões, sem esconder

- nenhuma janela real foi coberta, minimizada ou descoberta nesta
  máquina, e a máquina não dormiu: o que a rodada prova é o engine e o
  showcase com o `Occluded` injetado sobre hosts headless. o caminho do
  winit está lido na fonte, não medido. a prova na tela é do
  orquestrador: `RUST_LOG=engine=debug cargo run --offline -p uigallery --
  Spinner`, minimizar, e ver "window paused (covered true, asleep
  false)" e nenhum "frame:" no log até restaurar, quando aparece "window
  shows again: a new frame" e um frame só, depois o spinner segue.
- o sleep do sistema no desktop não é ouvido (sonda (a)). a app com o
  monitor dormindo segue a cobertura que o AppKit reportar, se reportar;
  não medido. o gancho, quando o manifesto abrir: `objc2-app-kit` com
  `NSWorkspace` (e `objc2-foundation` com `NSNotificationCenter`), um
  observador em `NSWorkspace::sharedWorkspace().notificationCenter()`
  para as duas notificações, chamando `Windows::sleep`/`wake` na thread
  do laço (acordando-o pelo proxy, como o tray faz).
- o `Background` é thread-local, não um valor que o shell passa para as
  páginas: a assinatura do `wake` da view (`view/mod.rs`,
  `view/dispatch.rs`) e os structs das páginas (`proposing.rs`,
  `silences.rs`) estavam fora da lista. um trait `Background` que os
  donos dos jobs implementam e o shell chama é a forma a migrar quando
  esses arquivos abrirem; o flag passaria a ser deles.
- o adiar do ffmpeg não entrou (ver o contrato acima): dois arquivos fora
  da lista, uma linha em cada. sem efeito observável hoje.
- coberta, uma página que faz polling do worker (`Wake::after(100 ms)`)
  perde o deadline: o resultado do Claude ou do ffmpeg é aplicado quando
  outra coisa acorda o laço (input, um deadline de outra janela, o proxy
  de um worker, a porta do agente) ou quando a janela mostra (o frame do
  descobrir tica e faz `settle`). o request fica `Executing` por mais
  tempo enquanto ninguém olha; nada se perde. é o "não acorda a janela
  por conta própria" do briefing, dito.
- a janela da página (`D`) coberta é independente da principal: cada uma
  tem o seu `occluded`; o `view.wake` é um só e tica quando qualquer
  coisa acorda o laço.
- `suspended` no showcase dorme sem soltar as surfaces: no android isso
  é insuficiente (a surface morre com a janela) e é o que já era;
  bring-up.
- o teste da resposta que chega dormindo (`page.rs`) espera 200 ms
  depois do `prompt.1` aparecer para a resposta estar no canal; uma
  máquina muito lenta poderia ver o primeiro `wake` acordado ainda sem
  ela, o que o `pump` de 10 s cobre.

## o que fica para o bring-up e o que os itens 9 e 10 reaproveitam

- bring-up: o gancho de sleep do sistema no macOS (acima); o `suspended`
  do android soltando as surfaces pelo `Windows::suspend` (o engine `App`
  já faz; o showcase não); o `Occluded` do iOS como o background (já
  cai em `set_occluded`, não medido num aparelho); o adiar do ffmpeg;
  linux e windows sem `Occluded` (wayland e windows: não suportado no
  winit) ficam só com o sleep por shell.
- item 9 (impressão): um frame de impressão é um `WindowHost::headless`
  desenhando o estado como está, o mesmo caminho que o teste de controle
  usa aqui (`control()`): a janela pode estar coberta ou o app dormindo
  e a impressão não depende de `awake`, porque um host headless não é
  coberto nem pede frame à plataforma; e `Background::asleep()` diz se
  um job de impressão em fundo deve esperar o wake (um spool é um job
  como o ffmpeg).
- item 10 (câmera e microfone): a captura é um trabalho de fundo com o
  contrato daqui: enquanto o app dorme, os frames capturados ficam no
  canal (ou a captura pausa, decisão do item) e a textura só é atualizada
  no primeiro wake acordado; uma janela coberta não deve manter a câmera
  desenhando, e `Windows::awake(id)` é a pergunta; o `Occluded(true)` do
  iOS (background) é onde o sistema exige soltar a captura.

## comandos de validação

```text
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/one
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_PROFILE_TEST_DEBUG=line-tables-only
cargo test --offline -p engine --test sleep_wake
cargo test --offline -p uigallery --test sleep_wake
cargo test --offline -p uigallery --lib -- app::sleep silences::jobs surface_page::page
cargo test --offline -p engine -p uigallery
./tool/tasks/fullcheck.sh
cargo run --offline -p regress -- check
RUST_LOG=engine=debug cargo run --offline -p uigallery -- Spinner   (minimizar e restaurar; do orquestrador)
```
