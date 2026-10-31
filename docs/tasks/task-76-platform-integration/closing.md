---
id: task-76/closing
title: "task-76, closing: what went in and what remains"
project: plev
type: section
parent: task-76
status: reference
date: 2026-09-24
date_basis: header
recorded: 2026-09-23
topics: [platform, process]
relations: [adr-0054, adr-0055, adr-0056, adr-0057, adr-0058, adr-0059, adr-0060, adr-0061, adr-0062, adr-0063, task-58]
source: docs/log/steps/task-76-platform-integration.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# closing of task-76

Records the closing of task-76: ten items, of which nine are in `main` and the sixth, deep links, is on branch `task/76j-links` awaiting integration. For each item, it states what went in and what is left for the bring-up: the macOS bundle unblocks three items at once (the notification, the link and the process responsible for the camera), Linux needs a backend choice across three crates, and Windows needs to compile the legs that were written but not compiled. The on-screen proofs for each item are in brenner's walkthroughs.


a task-76 tinha dez itens: nove estão na `main`, e o sexto é esta
branch, à espera da integração. o que cada um deixou e o que fica para o
bring-up de plataforma (bundle macOS, linux e windows como alvo, mobile),
item por item:

1. perda de device de GPU (`gpu-device-loss-recovers-in-place`). entrou:
   o callback de device lost em todo `GpuContext`, a recriação no lugar
   (adapter, device, surface, pipelines, atlas, imagens pelo log de
   uploads) antes do acquire, o frame pulado com um erro por perda,
   provado headless com `Device::destroy` e pixels iguais antes e depois.
   fica: a perda real (driver, sleep, GPU trocada) nunca provocada numa
   máquina; wasm só sinaliza, não recupera; o `recreate_surface` do
   android pelo caminho próprio.
2. múltiplas janelas (`windows-are-hosts-the-app-owns-the-state`).
   entrou: `WindowHost` e `Windows` no engine (um host por janela, rota
   por id, fechar a última sai ou não), a página do showcase numa janela
   própria (`D`, `shell.detach`) com o mesmo estado e o mesmo
   dispatcher. fica: a janela real vista pelo brenner; o `source` dos nós
   com a janela aberta (um `SnapshotBuilder::extend` no `sem`); a janela
   da página sem acessibilidade própria; wasm e mobile com uma janela.
3. diálogos de arquivo e drag and drop (`file-dialogs-and-drops-are-capabilities`).
   entrou: `engine::platform::files` (o `rfd` do engine, falso, nenhum),
   o urnaui migrado sem segunda implementação, o drop como evento do
   engine, a página Files. fica: o backend do linux (portal ou gtk3), o
   diálogo assíncrono do wasm (bytes), a posição do drop (o winit 0.30
   não dá), o painel real aberto pelo brenner.
4. menus nativos (`native-menus-are-another-view-of-capabilities`).
   entrou: o `MenuModel` puro a partir da projeção, o `muda` no desktop,
   o clique como o request do botão por `Actor::Human`, o shell projetado
   (`shell.page`, `shell.theme`, `shell.detach`, `shell.save_session`) e a
   barra do showcase. fica: a perna windows (`init_for_hwnd`,
   `TranslateAcceleratorW`) compilada e provada; o linux (backend `noop`
   do `muda`); o clique num item real visto pelo brenner.
5. notificações e tray (`tray-and-notifications-are-capabilities-out-of-the-window`).
   entrou: `TrayModel` e `NativeTray` (`tray-icon`, macOS e windows), a
   fila de cliques dividida com a barra, `Notification` com uma `Action`
   que volta como o request do botão, `shell.show`, `shell:notice`, a
   Files e a Silences contando o que aconteceu sem ninguém olhando. fica:
   a notificação real no macOS (UserNotifications quer o bundle:
   `NeedsBundle` hoje), o tray do linux, a perna windows provada, viver só
   no tray.
6. deep links (`a-link-is-not-a-person`, este). entrou: a gramática
   estrita, o ator `agent:link` sem mudar o `sem`, a lista aberta do app,
   a recusa na tela e em `shell:notice`, as fontes (argumento em todo
   desktop, falso, nenhum), o showcase (`plev-showcase`, só página e
   tema), a espera pelo wake, a gravação. fica: no macOS o `.app` com
   `CFBundleURLTypes` e o handler `GURL` pelo `NSNotificationCenter` (três
   dependências diretas, três linhas do lock); o registro do windows e o
   `.desktop` do linux; encaminhar o link da segunda instância para a que
   roda; iOS e android pela entrada deles; a prova com `open` depois do
   bundle.
7. sleep e wake (`a-covered-window-does-not-draw-and-wakes-new`). entrou:
   coberta e dormindo por host e por app, nenhum frame nem deadline
   enquanto isso, um frame novo ao mostrar ou acordar, os relógios que
   saltam, o `Background` para o trabalho ao lado dos frames (o Claude da
   Surface retido, o ffmpeg esperando). fica: o sleep do sistema no macOS
   (`NSWorkspace` will sleep e did wake, feature ainda não ligada), o
   `suspended` do android soltando as surfaces, o adiar do job novo do
   ffmpeg dormindo.
8. diálogos nativos de alerta e confirmação (`native-dialogs-answer-the-same-contract`).
   entrou: `engine::platform::alerts` (o `MessageDialog` do `rfd`, falso,
   nenhum), a confirmação humana da task-58 pelo diálogo nativo como opção
   do ide (`PLEV_NATIVE_CONFIRM=1`), o alerta só de pessoa na Files. fica:
   o `AsyncMessageDialog` (o diálogo bloqueia o laço), rótulos no windows
   (`common-controls-v6`), linux, o sheet real visto pelo brenner.
9. impressão (`printing-is-the-frame-on-paper`). entrou: o frame em PNG a
   300 dpi pelo render headless, o CUPS por filho (`lp`, `lpstat`),
   `shell.print` só de pessoa, cmd-p e o menu, o job ao lado do laço. fica:
   o windows (spooler), o painel nativo (`NSPrintPanel`), o PDF vetorial,
   paginação, o poll da fila depois do spool, a impressão real do brenner.
10. câmera e microfone (`capture-is-a-job-the-person-turns-on`). entrou:
    `engine::platform::capture` com o ffmpeg por filho, a permissão como
    estado, a imagem de fluxo no atlas, a página Capture só de pessoa, o
    dispositivo solto na hora quando coberto, dormindo ou fora da página.
    fica: AVFoundation direto e o `.app` com as `UsageDescription`, os
    nomes do dshow no windows, v4l2 e alsa numa máquina linux real, as
    APIs do mobile, a luz da câmera vista pelo brenner.

o que atravessa a task e fica junto para o bring-up: o bundle do macOS
destrava três itens de uma vez (a notificação do 5, o link do 6 e o
processo responsável pela câmera do 10); o linux precisa de uma escolha
de backend em três crates (`rfd`, `muda`, `tray-icon`); o windows
precisa compilar as pernas escritas e não compiladas (menu, tray, dshow);
e as provas na tela de cada item estão nos roteiros do brenner, nenhuma
feita por agente. os critérios de aceitação da task (por item: macOS
funcionando, a variante mobile ou web registrada, teste automático onde
dá, roteiro onde não dá; device lost sobrevive a uma perda forçada) estão
cumpridos no que o agente pode provar; o "funcionando no macOS" com
janela, dispositivo, impressora e sistema de verdade é o que os roteiros
pedem.
