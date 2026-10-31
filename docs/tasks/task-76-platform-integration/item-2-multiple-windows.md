---
id: task-76/item-2-multiple-windows
title: "task-76, item 2: multiple windows"
project: plev
type: section
parent: task-76
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [platform, rendering, input]
relations: [adr-0056, task-58]
evidence: [3b642d82, 6da95f2a, bd97c817, 6e46fd28]
source: docs/log/steps/task-76-platform-integration.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# handoff of item 2: multiple windows in the engine

Records item 2: `engine::window` gains `WindowHost`, which groups what dies with the window (the GPU context with the surface, the compositor, input, IME, scale and clock), and `Windows`, which keeps several hosts by `WindowId` and delivers each event to its owner. The showcase puts the current page in a second window (`D` or `shell.detach`) over the same state and the same dispatcher, and the `layers` example opens two windows while the engine's `App` keeps its behavior with one. The result was proven headless with hosts under fake ids; the real window was not opened on this machine, and the `source` of the nodes with the second window open is a limit that calls for `SnapshotBuilder::extend` in `sem`.


branch `task/76c-windows`, base `153d4c65` (main). commits, em ordem:

- `3b642d82` task-76: the windows probe
- `6da95f2a` task-76: the WindowHost, the Windows map, and the App over
  them with one window
- `bd97c817` task-76: the layers example opens a second window
- `6e46fd28` task-76: the showcase page in a window of its own
- (este) task-76: ADR and the item 2 handoff

os commits 2 e 3 do briefing (o host; o `Windows` e o `App`) saíram
fundidos em `6da95f2a`: o `App` do engine é o único consumidor do
`Windows` dentro do crate, e cada commit tem que compilar.

## a sonda

(a) o que é "da janela" (morre com ela) e o que é "do app", campo a
campo, como estava antes desta fase:

- `App` do engine (`rust/engine/src/runtime/window/mod.rs:49`): da janela
  `window`, `state: GpuState` (o `GpuContext` com a surface, o
  `TextSystem`, o `EffectProcessor`, o `TexturePool`: tudo no device
  daquela janela), `compositor` (as layers e a tabela de binding são o
  frame de uma janela), `input_state` (hit regions, hover, foco, cursor),
  `touch_input` e `touch_pointer`, `ime_state`, `safe_area`,
  `scale_factor`, `frame_clock` e `animation_tick`, `perf` e `perf_hud`
  (contam frames de uma janela e desenham nela), `event_buffer`, `a11y`
  (o `Bridge` do AccessKit se prende a uma janela). do app: `theme`,
  `frame_read` e `frame_write` (o sinal reativo de frame), `lifecycle`
  (Active/Suspended é do processo), `is_animating` (o pedido do app,
  aplicado a cada janela ao abrir), `event_loop_proxy`,
  `narrate_watcher`.
- shell do showcase (`apps/uigallery/src/shell/mod.rs:82`): da janela
  `window`, `state: GpuState`, `compositor`, `cursor`, `scale_factor`.
  do app: `view: ShowcaseView` (o estado das páginas, o dispatcher da
  galeria, o tema, o chrome), `clock` e `caught_up` (um relógio do loop:
  o tick anda uma vez por iteração, a janela que redesenha só lê o tempo
  real desde o último), `modifiers` (estado do teclado), `proxy`,
  `access` (o bridge é da janela principal; a porta do agente é do app),
  `recording`, `source_click`. dentro da view, três coisas são da janela
  e não do app: `layers: Option<Layers>` (ids de layer nomeiam layers de
  um compositor), `page_scroll` (o scroll de uma janela) e
  `width/height/scale_factor` (o layout de uma janela). elas ficaram na
  view para a janela principal; a segunda janela ganhou as suas em
  `view::dispatch::Detached`.
- o ide (`apps/idestudio/src/main.rs:106`) e o urnaui
  (`apps/urnadbapp/src/app.rs:75`) repetem a forma do showcase; fora do
  escopo, não mudaram (nenhum usa o `App` do engine).
- o que o briefing supunha e não é: nenhum exemplo do engine usa
  `engine::window::App`. `counter`, `builder`, `charts`, `layers` e os
  outros têm o próprio `ApplicationHandler` com `GpuContext::new` e
  `TextSystem::new`; o `App` do engine só é usado por
  `lib.rs::run_event_loop`, pelo `android_main` e pelo `wasm_main` do
  engine, e `hot_reload/mod.rs` já o descrevia como "experimental, no
  app uses it". então "os exemplos compilam e rodam sem mudança de
  comportamento" é verdade por não passarem pelo `App`; o exemplo que
  mudou é o `layers`, de propósito.

(b) `rust/engine/tests/windows.rs`,
`two_contexts_draw_two_scenes_in_the_same_loop_and_share_only_the_image_log`
(ficou como teste real): dois `GpuContext::new_headless` (via
`headless::Renderer`) com dois compositores, um rect vermelho, o texto
"one" e uma imagem 8x8 em A, um rect azul e o texto "two" em B, três
rodadas alternadas no mesmo laço. cada frame mostra só o que foi
empurrado nele: o rect da sua cor, o texto do seu atlas de glifos, a
imagem só em A, e os dois frames diferem byte a byte. o que é
compartilhado por desenho: o `ImageStore` global (`gpu/image.rs:232`), o
log de uploads da fase 1: a imagem carregada antes de B existir aparece
em B no frame seguinte a ser empurrada lá, sem segundo `load`. os outros
globais do engine são CPU e por thread (`MEASURE_CTX` do texto, o
`RUNTIME` de sinais, `DEFAULT_TOLERANCE_BITS` do path, os mapas do hot
reload). nada em `gpu/**` precisou mudar; não parei.

(c) o winit 0.30.13 entrega todo `WindowEvent` em
`ApplicationHandler::window_event(event_loop, window_id, event)`, e no
macOS o delegate de cada janela enfileira com o próprio id
(`platform_impl/macos/window_delegate.rs:819`,
`maybe_queue_window_event(self.window().id(), event)`). o shell do
showcase ignorava o id (`_id`). se um segundo `create_window` fosse
chamado nele: `self.window = Some(window)` sobrescreve o `Arc` da
primeira; o `Drop` do `Window` do winit no macOS chama `close()`
(`window.rs:20`), a primeira janela fecha, o winit enfileira
`Destroyed` para ela, e qualquer evento que ainda chegasse com o id da
primeira seria roteado para a view como se fosse da segunda. dois
`window` não existiam: o segundo apagava o primeiro. `WindowId::dummy()`
é `WindowId(0)` no macOS (`window.rs:77`) e `From<u64>` existe: os testes
usam `dummy()` e `from(2)`.

## o host e o `Windows`

`rust/engine/src/runtime/window/host.rs` (551 linhas de produção).
`WindowHost` agrupa o que a sonda (a) disse ser da janela: `window`,
`gpu: Option<Gpu>` (o contexto e, construídos nele, `TextSystem`,
`EffectProcessor` e `TexturePool`), `target` (a textura de um host
headless), `compositor`, `input`, `touch` e `touch_pointer`, `ime`,
`safe_area`, `scale_factor`, `size` (físico, para o host saber o tamanho
lógico mesmo sem device), `clock` e `tick`, `perf` e `perf_hud`,
`events` (o buffer até o próximo frame), `animating`, `redraw_asked`,
`deadline`, `frames`, `a11y`. constrói por etapas para caber em todo
alvo: `bare()` (sem janela e sem device: cena e input que nunca
desenham, para teste sem GPU), `attach_window`, `attach_gpu`; `open` e
`open_with_config` fazem as três no nativo bloqueando no adapter como os
construtores sempre fizeram; no wasm o shell faz o spawn e prende o
resultado. `headless(w, h, scale, config)` é um host sem janela
desenhando numa textura, `read()` devolve os pixels
(`headless::read_texture`).

o que ele faz: `resize` (reconfigura a surface, reaplica a projeção
lógica, invalida, pede frame), `scale_factor_changed`, `handle_event`
(em `events.rs`: `Resized` e `ScaleFactorChanged` valem na hora; pointer,
teclado, roda, modificadores, touch, IME e drop são bufferizados e
invalidam; `CloseRequested` e `RedrawRequested` não são dele),
`process_events` (o buffer entra no `InputState`, o touch tica),
`request_redraw` (a janela, ou só a flag num host sem janela),
`needs_redraw` (animando, ou pedido, ou compositor sujo, ou HUD),
`set_animating`, `set_deadline` e `deadline` (o `Wake` por janela: o
instante em que esta janela precisa de frame sozinha), `begin_frame`
(tica o relógio, consome o pedido, limpa o texto do IME), `present`
(o `present_frame` de todo app para uma janela; `draw_frame` no target
para um host headless, com o poll e a recuperação de device antes),
`render` (em `render.rs`: a cauda própria do `App` do engine, com efeitos
de layer e HUD, para surface ou target), `device_lost` e
`recover_or_skip` (por contexto, da fase 1), `suspend` e `resume`
(android), `purge_caches`, `set_accessibility`.

`rust/engine/src/runtime/window/windows.rs` (311). `Windows` guarda
`Vec<(WindowId, WindowHost)>` na ordem de abertura (`first()` é a
principal), mais os hosts suspensos. `open(event_loop, attrs)` cria a
janela e o host (no nativo com o contexto; no wasm sem, até
`attach_gpu(id, gpu)`); `insert(id, host)` para um host que o chamador
construiu (um headless num teste). `route(id, event) -> Routed`: o host
dono toma input, IME, drop, resize e escala (`Handled`); `RedrawRequested`
processa o buffer e responde `Redraw` (o chamador constrói a cena e
apresenta); `CloseRequested` responde `CloseRequested` (só o chamador
sabe se fecha); id desconhecido responde `Unknown` (o `Destroyed` do
winit depois de um `close` cai aqui). `close(id)` tira o host do mapa e
devolve; soltar o host solta a janela e o contexto. `should_exit()`:
abriu alguma, não sobrou nenhuma e `exit_when_last_closes` (padrão
ligado; desligado, o loop segue sem janela). `request_pending_redraws`
é o render on demand por janela: cada host com motivo pede o próprio
frame, os outros ficam parados. `earliest_deadline` para o `WaitUntil`
do loop. `invalidate_all`, `set_animating`, `purge_caches`, `suspend` e
`resume(event_loop, attrs)` (android: um host guarda o device, ganha
janela nova e id novo).

a segunda janela reaproveita o `Instance`? não: cada `GpuContext` nasce
com o próprio `wgpu::Instance` (é o que `GpuContext::new_with_config` e
`new_headless` fazem, e o `recover` da fase 1 pergunta ao instance do
próprio contexto). reaproveitar pediria um construtor novo em `gpu/**`,
proibido aqui; o custo é uma enumeração de adapters por janela aberta,
o que todo construtor já pagava uma vez. fica registrado, não parei.

## o `App` do engine e os exemplos

`rust/engine/src/runtime/window/mod.rs` (159): o `App` fica com `windows:
Windows`, `theme`, `frame_read` e `frame_write`, `lifecycle`,
`is_animating`, o proxy e o `narrate_watcher`. `GpuState` e `state.rs`
saíram (o host tem o device). `AppEvent::GpuReady { window, gpu }`
nomeia a janela e leva só o contexto (o host constrói o resto).
`lifecycle.rs` (225): `resumed` abre a janela por `Windows::open`,
prende o `Bridge` (a janela nasce oculta e aparece depois, como antes),
no wasm faz o spawn do device; `suspended` é `windows.suspend()`;
`resumed` com suspensos é `windows.resume`; `handle_window_event` é um
`match` sobre `route`: `Redraw` desenha aquela janela (`render_window`:
o sinal de frame avança, `host.render(&theme)`, e o próximo frame só se
`needs_redraw`); `CloseRequested` fecha aquela e sai do app quando
`should_exit`. `about_to_wait` é `request_pending_redraws` (mais o
narrate reload, que invalida todas). `render_passes.rs` só trocou
`impl super::App` por `impl super::host::WindowHost` em
`apply_layer_effects` (613 linhas, não cresceu).

`demo/layers/src` (`main.rs` 123, `render.rs` 143): abre
duas janelas em `resumed`. a primeira é a demo de layers de antes (fundo
de pontos, chrome, o card do foreground que muda a cada 60 frames), com
`set_animating(true)`; a segunda é um card parado ("the other window")
que mostra quantos frames desenhou. `window_event` roteia por
`Windows::route`: `Redraw` desenha a janela pedida (`render_layers` ou
`render_side`, cada um constrói no compositor do seu host e chama
`host.present`, o `present_frame` de todo app, no lugar dos passes que
o exemplo codificava à mão), e pede o próximo só se `needs_redraw`;
`CloseRequested` fecha aquela e sai no `should_exit`. os `LayerId` da
demo são criados no compositor do host principal em `resumed` (um id
nomeia uma layer de um compositor). não rodei a janela de verdade:
o exemplo não tem `PLEV_BACKGROUND` (isso é do showcase) e abrir uma
janela que toma o foco está proibido aqui. o que prova o caminho são os
testes headless do `Windows`; o `create_window` duas vezes no mesmo
loop é o uso comum do winit. `cargo run -p layers` é
do orquestrador.

## o showcase

o shell (`apps/uigallery/src/shell/mod.rs` 546, `app/events.rs` 195,
`app/detached.rs` 96, `app/pace.rs` 158) dirige as janelas por
`Windows`: `main: Option<WindowId>` e `page_window:
Option<detached::PageWindow>` (desktop). `resumed` abre a principal por
`Windows::open` (nativo com o device; wasm faz o spawn e `attach_gpu` no
`GpuReady`), prende o bridge e `viewport_changed(id)` faz o
`view.resize` com o tamanho lógico do host. o shell não roteia o
pointer pelo `Windows::route`: continua entregando `WidgetEvent` à view
na hora, como antes (bufferizar no `InputState` do host pediria frame a
cada movimento de mouse, e o `InputState` não é o caminho do showcase);
ele pede ao host o resize, a escala e o frame. cada evento sabe a
janela: o cursor é por janela (`cursor_in`), o pointer vai para
`view.detached_event` na janela da página e para `view.handle_event` na
principal; hover e scroll invalidam só a janela onde aconteceram
(`invalidate_in`), um clique que mudou algo invalida todas (`changed`),
porque a mudança aparece nas duas. teclas vão para a view seja qual for
a janela com foco.

o destacar: `D` fora de campo de texto (`view.handle_key` dá
precedência ao campo focado, como o `t` do tema) marca
`panes.detach_asked`; o shell lê com `take_detach_request` e
`toggle_page_window`: abre uma janela de 900x640 ("plev, <página>"),
`view.detach(lw, lh, sf)`; página placeholder não tem o que mostrar e
não abre. a janela principal continua com o chrome e a página; a segunda
mostra a página sozinha, sobre o mesmo estado e o mesmo dispatcher:
`view::dispatch::Detached` (`view/dispatch/detached.rs`, 226) guarda o
tamanho, a escala, as `Layers` criadas no compositor daquela janela no
primeiro frame, o scroll daquela janela e o `Wake` da página;
`render_detached(c)` faz o layout num viewport com gutter, clipa e chama
o mesmo `pane_render`; `detached_event` chama o mesmo `pane_event` com o
rect daquela janela e depois o scroll da página dela; `page_wake` passou
a guardar uma cópia do ask da página em `Detached::wake`
(`detached_wake` entrega ao shell), e a principal continua recebendo o
dela. escolhi espelhar, não mover: o aceite pede "um clique numa reflete
na outra no próximo frame", o que só existe com a página nas duas.
Escape na janela da página, o botão de fechar dela ou `D` de novo fecham
só ela (`close_page_window`: `view.attach()`, `windows.close`,
invalida); fechar a principal sai do app, como antes. um clique em
qualquer das duas passa pelo mesmo `Request::observe` por
`Actor::Human` no dispatcher da galeria.

a projeção: enquanto a janela da página existe, `ShowcaseView::semantics`
acrescenta um nó `window` (`Role::Window`, label "plev, <página>",
estado `page` com o título e `shows` com a lista dos ids raiz da página
mostrados nela) ao lado dos nós da página, que ficam com os mesmos ids,
árvore, estado e ações. escolhi um nó de janela e não um estado no nó da
página: as páginas (`apps/**`) são proibidas, e um nó só existe quando há
segunda janela, então com uma janela a projeção é a da página, byte a
byte (o golden do todo e os `semantics.json` das 161 baselines não
mudam). o nó entra reconstruindo o snapshot por `SnapshotBuilder`
(`with_window`): o `sem` não tem como levar um snapshot pronto para
dentro de outro (`Snapshot::from_parts` é `pub(crate)`). a reconstrução
mantém tudo menos o `source` de cada nó (o builder grava o call site,
que passa a ser o `detached.rs`); ver limitações. a porta do agente lê
os bounds do compositor da janela da página enquanto ela existe
(`page_window_id`). a capacidade `shell.detach` no contrato não entrou:
uma ação pelo dispatcher pede um `Operable` do shell no dispatcher da
galeria (emprestado por `TodoPage::with_dispatcher`) e um nó com ação,
o que é a forma que o item 4 (menus) vai precisar de todo jeito para o
"destacar" virar item de menu; fica lá.

a gravação: só os frames da janela principal (`record_frame` usa o
compositor dela); um clique na janela da página é gravado com o
controle sob ele naquele compositor, com a posição nas coordenadas
daquela janela (registrado como limite). o bridge de acessibilidade fica
na principal; a janela da página não tem (limite). o `regress` e o
`--render` não mudam: uma janela, `stage_page` sem host.

teste (`apps/uigallery/tests/detached.rs`, dois `WindowHost::headless`,
1200x800 e 900x640, um `ShowcaseView` na Todo): com uma janela não há nó
`window`; a página se liga nos dois compositores com dois layouts
(`binding(todo.item[1])` em cada, rects diferentes) e a principal
continua com a página; o nó `window` tem role, label, `page` e `shows`
certos, e todo nó da página é igual ao de antes; um clique na janela da
página (o checkbox do item 1) alterna `completed` e o frame seguinte da
principal mostra o item mudado (pixels do rect antes e depois), e o da
janela da página também; um clique na principal desfaz e o frame da
janela da página volta ao de antes; `attach()` devolve a projeção com a
mesma revision de antes. e: a Card numa janela de 400x300 rola pela roda
sem mexer no scroll da principal; uma página placeholder não destaca.
os testes do shell (`source.rs`, `pace/tests.rs`) usam
`App::with_bare_window` (um `WindowHost::bare()` sob `WindowId::dummy()`)
e continuam sem GPU.

## gate e regress

`df -h /` antes do gate: 39 GB livres. `./tool/tasks/fullcheck.sh` verde no worktree, com a
perna wasm e a perna urnaui (`../urna` presente): antes `cargo test
--workspace` 2485 passed, 0 failed, 16 ignored em 115 binários; depois
2494 passed, 0 failed, 16 ignored em 117 binários (+9: 7 de
`engine/tests/windows.rs`, 2 de `showcase/tests/detached.rs`; +2
binários, os dois arquivos de teste novos). perna urnaui: 85 passed, 0
failed, 1 ignored em 6 binários, igual a antes (o ide e o urnaui não
usam o `App` do engine e compilam sem mudança). clippy `-D warnings`,
fmt e a checagem wasm limpos nas duas pernas. 33 GB livres depois.

`cargo run -p regress -- check`: 161 baselines, 161 iguais,
0 mudaram, sem frame desenhado e sem GPU aberta (788.9 ms). o golden do
todo (`uir_fixture`) e os testes do `regress` também verdes: com uma
janela a projeção não mudou.

linhas de produção (regra do guard): engine `window/host.rs` 551,
`window/windows.rs` 311, `window/events.rs` 194, `window/render.rs`
244, `window/lifecycle.rs` 225, `window/mod.rs` 159,
`window/render_passes.rs` 613 (era 633: só trocou o alvo do `impl`),
`examples/layers/main.rs` 123, `examples/layers/render.rs` 143; showcase
`app.rs` 546 (era 620; foi a 715 e o `window_event` saiu para
`app/events.rs`), `app/events.rs` 195, `app/detached.rs` 96,
`app/pace.rs` 158, `app/record.rs` 316, `app/source.rs` 67,
`view/dispatch.rs` 452 (era 415; foi a 665 e o destacar saiu para
`view/dispatch/detached.rs`), `view/dispatch/detached.rs` 226,
`view/input.rs` 400, `view/panes.rs` 134. tudo abaixo de 639.

## ADR

[`0056-windows-are-hosts-the-app-owns-the-state.md`](../../adr/structure/0056-windows-are-hosts-the-app-owns-the-state.md) (proposed): o que
é da janela e o que é do app, o host por etapas, o `Windows` e o
roteamento, render on demand por janela, o fechar, o destacar do
showcase, o que a projeção diz, os limites. linha pro índice, que é do
orquestrador ([`adr/`](../../adr/README.md) é proibido aqui):

| 2026-09-24 | [windows-are-hosts-the-app-owns-the-state](../../adr/structure/0056-windows-are-hosts-the-app-owns-the-state.md) | proposed | `engine::window::WindowHost` agrupa o que morre com a janela (a janela, o `GpuContext` com a surface e a perda de device, compositor, `InputState`, touch, IME, escala, área segura, relógio, perf, eventos bufferizados) e `Windows` guarda vários por `WindowId`: `open`, `route` (o dono toma input e resize; close e redraw voltam ao chamador), `close` de uma sem o app, `should_exit` na última (configurável), redraw por janela (parada não desenha enquanto outra anima), deadline mais cedo; o `App` do engine com uma janela sem mudança, o `layers` com duas; o showcase destaca a página atual numa segunda janela sobre o mesmo estado e o mesmo dispatcher, a projeção ganha um nó `window` só enquanto ela existe; provado headless com hosts sob `WindowId::dummy()` (task-76 item 2) |

## interfaces adicionadas (só crescimento, salvo o `AppEvent`)

`engine::window::{WindowHost, Windows, Routed}`; `WindowHost::{bare,
new, open, open_with_config, headless, attach_window, attach_gpu,
window, id, has_gpu, gpu, gpu_mut, text_system, compositor,
compositor_mut, input, input_mut, ime, safe_area, scale_factor,
physical_size, logical_size, tick, frames, resize,
scale_factor_changed, handle_event, process_events, pending_events,
request_redraw, redraw_asked, needs_redraw, set_animating,
is_animating, set_deadline, deadline, begin_frame, present,
device_lost, read, suspend, resume, purge_caches, set_accessibility,
accessibility_mut}`; `Windows::{new, len, is_empty, ids, first,
contains, get, get_mut, iter, iter_mut, insert, open, open_with_config,
attach_gpu, route, close, should_exit, set_exit_when_last_closes,
exit_when_last_closes, request_redraw, request_pending_redraws,
invalidate_all, set_animating, earliest_deadline, purge_caches, suspend,
has_suspended, resume}`; `App::{windows, windows_mut}`.
`ShowcaseView::{is_detached, take_detach_request, detach, attach,
resize_detached, render_detached, detached_event, detached_wake}`,
`view::dispatch::Detached`. o que mudou: `engine::window::AppEvent::GpuReady`
passou de `{ gpu, text_system, effect_processor, texture_pool }` para
`{ window: WindowId, gpu }` (o único construtor era o próprio engine);
`App::{set_animating, is_animating}` mantidos. `present_frame`,
`draw_frame`, `resolve_layer_text`, `encode_*` sem mudança: ide, urnaui,
o headless e os outros exemplos compilam sem tocar.

## arquivos tocados fora da lista permitida, e por quê

- `apps/uigallery/src/views/dispatcher/detached.rs` (novo): o destacar
  levou `view/dispatch.rs` a 665 linhas de produção; o módulo filho é a
  forma que `dispatch/agent.rs` já tem, e acessa os métodos privados
  (`pane_render`, `pane_event`) por ser descendente.
- `apps/uigallery/src/shell/window/events/mod.rs` (novo, previsto pelo briefing para
  `app.rs` passar de 639): o `window_event` do shell.
- `rust/engine/src/runtime/window/state.rs` removido (o `GpuState` virou o
  `Option<Gpu>` do host).

## limitações e decisões, sem esconder

- a janela de verdade não foi aberta nesta máquina: nem o `layers` com
  duas nem o showcase com `D`. o exemplo não tem `PLEV_BACKGROUND` e
  ninguém pode apertar `D` por mim (tecla sintética proibida). a prova é
  headless: o `Windows` com hosts sob ids falsos e a view com dois
  hosts. o caminho do winit (`create_window` duas vezes, eventos por id)
  é o comum e está lido na fonte, não medido.
- `source` dos nós enquanto a janela da página existe: a reconstrução
  por `SnapshotBuilder` põe o call site do `detached.rs` em todo nó, então
  o cmd+click e o `state` da gravação apontam para a view nesse
  intervalo. a forma certa é um `SnapshotBuilder::extend(&Snapshot)` (ou
  um `Snapshot::with_node`) no `sem`, que guardaria o `source`; é mudança
  de contrato e fica para decisão do orquestrador. com uma janela nada
  disso acontece.
- a janela da página não tem bridge de acessibilidade; a árvore de um
  leitor de tela é a da principal, que continua com a página. a
  gravação guarda só os frames da principal; os cliques na janela da
  página entram com o alvo certo (o compositor dela) e a posição nas
  coordenadas dela. o clique direito é só da principal (o menu da Git e
  a demo de Context Menu fazem layout lá). os overlays que uma página
  abre (modal, toast) desenham na principal, onde estão as layers de
  overlay. um hover é estado compartilhado dos widgets: aparece na outra
  janela no frame seguinte dela, não na hora.
- a janela da página segue a página ativa: trocar de página na
  principal troca o que a segunda mostra; o título dela fica o da página
  que a abriu.
- wasm e mobile: uma janela. `Windows::open` no wasm abre sem device
  (o shell faz o spawn), `D` loga e não abre. o `resume` do android
  passou pelo `Windows` sem ser medido num aparelho.
- `apps/uigallery/src/renderer.rs` (`render_frame`) ficou sem chamador
  no shell (o `pace.rs` usa `host.present`, o mesmo `present_frame`);
  está fora da lista permitida e ficou.
- o showcase não passa o pointer por `Windows::route`; o `App` do engine
  e o `layers` passam tudo. dois caminhos de entrada, como já eram
  (`arc.toml` já anota que os shells entregam o pointer direto à view).
- `is_animating` do `App` do engine é um pedido do app aplicado a todas
  as janelas; por janela é `WindowHost::set_animating`.

## o que os itens 4, 7 e 8 precisam disto

- item 4 (menus nativos): no macOS a barra de menu é do app e o menu de
  contexto é da janela; o host é onde um item de menu por janela se
  liga, e o `route` é onde um evento de menu de uma janela chega ao
  dono. o "destacar" do showcase como item de menu é a capacidade que
  não entrou aqui: um `Operable` do shell no dispatcher da galeria e um
  nó com a ação, pelo mesmo `Actor::Human`.
- item 7 (sleep e wake): pausar é por host (`set_animating(false)`, o
  `deadline` limpo, `request_pending_redraws` não pede nada); acordar é
  `invalidate_all` e um frame por janela, e a recuperação de device já é
  por contexto (fase 1). o "sem frame velho" da fase 1 vale por janela.
- item 8 (diálogos com janela pai): um diálogo é um `Windows::open` com
  atributos de filho (`with_parent_window` do winit no macOS) e o id do
  pai guardado pelo shell; `set_exit_when_last_closes` já permite fechar
  o diálogo sem fechar o app, e `route` entrega ao diálogo o que é dele.
  a confirmação humana da task-58 como janela cabe aqui.

## comandos de validação

```text
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/one
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_PROFILE_TEST_DEBUG=line-tables-only
cargo test --offline -p engine --test windows
cargo test --offline -p uigallery --test detached
cargo test --offline -p engine -p uigallery
./tool/tasks/fullcheck.sh
cargo run --offline -p regress -- check
cargo run --offline -p layers          (duas janelas; do orquestrador)
cargo run --offline -p uigallery -- Todo                  (D destaca a página; do orquestrador)
```
