---
id: task-76
title: "task-76: platform integration"
project: plev
type: task
status: done
date: 2026-09-24
date_basis: header
recorded: 2026-09-23
topics: [platform, rendering, agents, input]
relations: [adr-0054, adr-0055, adr-0056, adr-0057, adr-0058, adr-0059, adr-0060, adr-0061, adr-0062, adr-0063, task-58, task-60, task-66, generative-runtime-thesis]
evidence: [7b9f860f, 3173d2de]
source: docs/log/steps/task-76-platform-integration.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-76: platform integration

Records task-76, which makes plev respond to the edges between the renderer and the application runtime: GPU device loss, multiple windows, file dialogs and file drop, menus, tray, notifications, sleep and wake, native alerts, printing, camera, microphone and deep links. Each integration that is an action becomes a capability of the contract, with a representation on screen, in the menu or at the agent's port, and the same dispatcher decides who may request it. The result is the ten items on `main`, each with its own ADR (0054 to 0063); the last one to land, deep links, was integrated at `7b9f860f` and closed the task at `3173d2de`. The on-screen proofs were left to brenner's scripts.

## objetivo

o plev responde de forma consistente às bordas que separam renderer de
application runtime: menus, tray, notificações, diálogos de arquivo, drag
and drop, clipboard, IME, múltiplas janelas, diálogos nativos, impressão,
câmera, microfone, deep links, sleep e wake, perda de device de GPU e
accessibility. e cada integração que é uma ação vira capacidade no
contrato: o item de menu nativo e o botão da toolbar são duas
representações da mesma capacidade.

## ligação com a tese

seção 22 (integração de plataforma continua existindo; é parte da
diferença entre renderer e application runtime), seção 6 (uma capacidade,
várias representações).

## estado atual no código

- clipboard: `rust/engine/src/runtime/platform/clipboard.rs` (arboard no desktop,
  `LocalClipboard` em teste, wasm e mobile).
- diálogo de arquivo nativo (rfd) e drop de arquivo: só no urnaui
  (`apps/urnadbapp/src/app.rs`, `apps/urnadbapp/src/view/explorer/mod.rs`).
- IME básico: `rust/engine/src/runtime/platform/ime.rs`.
- lifecycle (estados de app, aviso de memória):
  `rust/engine/src/runtime/platform/lifecycle.rs`; safe areas em
  `rust/engine/src/runtime/platform/mod.rs`.
- sem menus nativos, tray, notificações do sistema, múltiplas janelas,
  diálogos nativos no engine, impressão, câmera, microfone, deep links,
  sleep e wake, ou recuperação de device de GPU perdido.

## escopo

ordem recomendada (robustez primeiro, depois o que os apps atuais pedem):

1. perda de device de GPU: callback de device lost do wgpu, recriação de
   surface, pipelines, atlas de glifo, texturas; teste forçando a perda.
2. múltiplas janelas no engine (hoje cada app tem uma).
3. diálogos de arquivo e drag and drop: mover do urnaui para
   `engine::platform`, sem segunda implementação; o urnaui passa a usar o do
   engine.
4. menus nativos: cada item é um `SemId` com ação no contrato; o menu é
   outra representação da capacidade, e o clique no menu vai pelo
   dispatcher como `Actor::Human`.
5. notificações do sistema e tray.
6. deep links (esquema de URL do app) que viram `Request` pelo dispatcher,
   com policy.
7. sleep e wake: pausar animação e trabalho de fundo, retomar sem frame
   velho.
8. diálogos nativos (alerta, confirmação), incluindo a confirmação humana
   da task-58 como opção.
9. impressão (render headless da task-66 para o sistema de impressão).
10. câmera e microfone (captura para textura e buffer de áudio), com
    permissão do sistema.

cada item: API no engine, a capacidade no contrato quando é ação, teste, e
dependência nova justificada (por exemplo menus e tray em crate
especializado) e aprovada antes.

## fora de escopo

bring-up de linux e windows como alvo; cada item entra nessas plataformas
quando o alvo existir.

## dependências

independente; os itens que viram capacidade usam 58.

## superfície sob responsabilidade

`rust/engine/src/runtime/platform/**`, `rust/engine/src/runtime/window` na parte de
múltiplas janelas e device lost (coordenado com a 60 se ela estiver
rodando), o urnaui na troca para o diálogo do engine, ADR por item.

## interfaces que consome

`wgpu`, `winit`, `sem` (58) para capacidades.

## interfaces que pode alterar

`engine::platform` por adição; `engine::window` com aprovação.

## critérios de aceitação

por item: funcionando no macOS (alvo principal), variante mobile ou web
registrada (feita ou declarada fora), teste automático onde é possível,
roteiro manual onde não é. device lost sobrevive a uma perda forçada sem
reiniciar o app.

## testes esperados

device lost forçado; múltiplas janelas com estado separado; menu como
capacidade (request pelo dispatcher); deep link com policy.

## riscos e perguntas abertas

- cada item pode puxar dependência grande; o custo de binário é medido.
- permissões de câmera e microfone mudam por sistema.

## commits esperados

um grupo por item, `task-76:` no início.

## pare e reporte se

- um item exigir mudar o shell de todos os apps;
- dependência nova (sempre aprovação).

## Phases

1. [item 1: GPU device loss](item-1-device-loss.md): the device-lost callback, in-place recreation and the headless proof with `Device::destroy`.
2. [item 3: file dialogs and drag and drop](item-3-files-and-drop.md): the engine's dialog trait, the `FileDrop` event and the Files page with `open_file` out of the agent's reach.
3. [item 2: multiple windows](item-2-multiple-windows.md): `WindowHost` and `Windows`, and the showcase page in a second window over the same state.
4. [item 4: native menus](item-4-native-menus.md): the projection's `MenuModel`, `muda` on desktop, and the click as the button's request.
5. [item 8: native alert and confirmation dialogs](item-8-native-alerts.md): the `Alerts` trait, the ide's human confirmation and the Files warning.
6. [item 5: system notifications and tray](item-5-tray-and-notifications.md): the menu bar icon, the notification with an action, and the bundle that macOS requires.
7. [item 7: sleep and wake](item-7-sleep-and-wake.md): covered and asleep with no frame or deadline, and a new frame when shown or woken.
8. [item 9: printing](item-9-printing.md): the frame at 300 dpi through headless rendering and CUPS as a child process, with no destination on this machine.
9. [item 10: camera and microphone](item-10-camera-and-microphone.md): ffmpeg per device, the stream image and person-only switches, released when the page is not in view.
10. [item 6: deep links](item-6-deep-links.md): the link grammar, the `link` actor and the list of open capabilities, on the branch awaiting integration.
11. [task-76 closure](closing.md): the closing written with nine items on main, what each one left and what remained for bring-up; deep links landed after it.
