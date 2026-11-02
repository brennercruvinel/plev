---
id: task-76/item-10-camera-and-microphone
title: "task-76, item 10: camera and microphone"
project: plev
type: section
parent: task-76
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [platform, recording, gpu]
relations: [adr-0062, task-58]
evidence: [6d0d1c53, ab221e79, 1331b644, f9c678e2, 2644e032, 824ee8ab, de4a4d98]
source: docs/log/steps/task-76-platform-integration.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# item 10 handoff: camera and microphone

Records item 10: `engine::platform::capture` puts the camera and microphone behind the `Capture` trait, with one ffmpeg per device as a child process, which reads the raw pipe and stderr as state; a denied permission appears as `Denied`, and the text that leads to that state is still a guess. Each frame goes to a stream image swapped in place, and the two switches on the Capture page are person-only; when covered, asleep or off the page, the device is released immediately and comes back when shown, if the person had turned it on. The result was proven with fakes and with ffmpeg's `lavfi`, without opening the camera or microphone: the real text of a refusal and the camera light were left for brenner's test script.


branch `task/76i-capture`, base `9ea37ea9` (main após o item 9).
worktree `.claude/worktrees/agent`, target `target/agents/one`. commits,
em ordem:

- `6d0d1c53` task-76: the capture probe
- `ab221e79` task-76: a stream image changes its pixels in place
- `1331b644` task-76: camera and microphone behind a trait, with ffmpeg
  as a child
- `f9c678e2` task-76: the Capture page, camera and microphone as a
  person's toggles
- `2644e032` task-76: covered or asleep, the shell releases the camera at
  once
- `824ee8ab` task-76: the captured frame is on screen in the first frame
  after the wake
- `de4a4d98` task-76: the camera drops frames before it scales them
- (este) task-76: ADR and the item 10 handoff

a rodada parou uma vez, depois da sonda: trocar os pixels de um
`ImageHandle` exigia `engine/src/gpu/image.rs`, fora da lista. o
orquestrador liberou a opção 1 (`gpu/image.rs` e uma linha de
`gpu/mod.rs`), com a API e cinco condições; o commit `ab221e79` é ela, e a
seção "a imagem de fluxo" diz como cada condição foi cumprida. os testes
de cada comportamento saíram no commit dele; `824ee8ab` é o teste de
integração com pixels.

nenhum dispositivo foi aberto nesta máquina, em sonda, teste ou gate: o
ffmpeg só leu `-f lavfi` (`testsrc`, `sine`), nenhum `-f avfoundation`
com entrada, nenhum `-list_devices`, nenhuma chamada a AVFoundation. o
`fake-ffmpeg` recusa (sai 2) se sobrar uma entrada de dispositivo depois
do `-i`.

## a sonda

(a) o pipe, com `-re` (o lavfi no ritmo de um dispositivo), medido em
python e depois nos testes (`rust/engine/tests/capture.rs`,
`--nocapture`):

```text
camera 640x360 at 15: 921600 bytes a frame, first frame 32.7 a 63.7 ms, 15.04 a 15.08 fps depois do burst
   (sem -re o testsrc sai a 3726 fps: o fake precisa do -re; com ele o primeiro meio segundo sai de uma vez, o readrate_initial_burst)
microphone 16000 Hz mono f32: first bytes 17.0 a 55.7 ms, leituras de 4096 a 24576 bytes (mediana 4096),
   16 555 a 16 921 amostras/s em janelas curtas depois do burst, RMS de 100 ms 0.0884 (o sine do lavfi tem amplitude 1/8)
load_image_rgba 640x360: 0.71 a 0.84 ms; draw e readback headless: 5.6 a 15.8 ms o primeiro (device), 1.5 a 7.2 ms depois
replace_image_rgba 640x360 (depois da decisão): 0.02 a 0.17 ms
```

e o achado que parou a rodada: cada frame carregado como imagem fixa é
um handle novo e um upload novo que o store guarda para sempre. 244
frames de 640x360 encheram o atlas (1024 até o frame 3, 2048 até o 14,
4096 até o 57, 8192 depois), 225 MB no log, 16,3 s de uma câmera a 15
fps; o seguinte volta `TooLarge`, para toda imagem do app (o store é
global). o teste ignorado `a_stream_of_frames_fills_the_grow_only_atlas`
mede isso de novo.

(b) a entrada real por plataforma, só lendo (`man ffmpeg-devices`, `ffmpeg
-hide_banner -h demuxer=avfoundation`, `ffmpeg -devices`, que só lista o
que foi compilado, e as strings de `libavdevice.63.1.101.dylib`):

- macOS: `-f avfoundation -i "<vídeo>:<áudio>"`, por índice ou nome,
  `none` para o outro lado e `default` para o padrão;
  `-video_device_index`/`-audio_device_index` sobrepõem o nome.
  `-pixel_format` padrão yuv420p (um formato que a câmera não tem é
  trocado com aviso, "Overriding selected pixel format"), `-framerate`
  padrão ntsc (29,97). o que o engine usa: `-f avfoundation -i 0:none` e
  `-i none:0`, sem `-video_size` nem `-framerate` na entrada (decisão do
  orquestrador), e a saída `-vf fps=15,scale=640:360:force_original_aspect_ratio=increase,crop=640:360
  -f rawvideo -pix_fmt rgba pipe:1` e `-f f32le -ac 1 -ar 16000 pipe:1`.
  este ffmpeg (homebrew 9.0.1) tem só `avfoundation` e `lavfi` como
  dispositivos de entrada.
- windows: `-f dshow -i video="<nome>"` e `audio="<nome>"`; o dshow abre
  por nome, e o índice só desempata nomes iguais (`video_device_number`).
  sem nome o engine responde `NoDevice("dshow opens a camera by its
  name, and none was given")`: a lista é do bring-up.
- linux: `-f v4l2 -i /dev/video<n>` e `-f alsa -i default` (ou
  `hw:<n>`).
- o que se sabe do erro de permissão no macOS: a libavdevice não tem
  `authorizationStatus` nem `requestAccess` nas strings, então o ffmpeg
  não pede nem confere a permissão; o sistema decide quando a entrada é
  criada, e a recusa sai como "Failed to create AV capture input device:
  %s", com a descrição do sistema no `%s`. o texto dessa descrição não
  foi visto: "Cannot use <câmera>" e "not authorized" são palpite (é o
  que o `classify` lê como `Denied`). o pedido de permissão é do processo
  responsável (o terminal que rodou o `cargo run`, ou o `.app`). o
  microfone negado pode não falhar e entregar silêncio (palpite). as
  outras linhas lidas das strings e usadas: "Invalid device index",
  "Video device not found", "Audio device not found", "No AV capture
  device found" (`NoDevice`), "Selected framerate (%f) is not supported
  by the device.", "Supported modes:" e `  %dx%d@[%f %f]fps` (o retry).
  a última linha do stderr costuma ser genérica ("Error opening input
  files: Input/output error"): o parser lê todas.

(c) uma página "Capture" de rascunho no Apps Demo, depois de Files, com
corpo de placeholder: `regress check` deu 161 baselines, 161 iguais. a
sidebar a 800 px termina em "Badge", então o grupo não entra em nenhum
baseline. o rascunho foi revertido; a página de verdade também deu 161
iguais, e ganhou os dois baselines dela.

## a imagem de fluxo

`rust/engine/src/drawing/gpu/image.rs` (560 de produção, era 411; 898 com os
testes) e a reexportação em `gpu/mod.rs` (a linha de `pub use` que o
rustfmt quebra em três):

- `ImageStore::stream(width, height)` aloca um slot fora do dedupe, com
  pixels transparentes; `ImageStore::replace(&handle, &pixels)` troca os
  pixels da entrada do slot no log, sem entrada nova, e devolve o handle
  com `id` novo e o mesmo lugar (o nó da cena muda, a layer redesenha,
  nada no compositor sabe de fluxo). recusa `NotAStream { id }` e
  `WrongSize { width, height, bytes }`, com o texto dito.
  `stream_image_rgba` e `replace_image_rgba` sobre o store global.
- condição 1: cada `ImageAtlasGpu` guarda um `AtlasCursor` (entradas
  escritas, revisão vista); cada entrada trocada leva a revisão da troca,
  e `unwritten` devolve, uma vez, as trocadas desde a última olhada e as
  novas. teste com dois cursores (`two_atlases_write_a_replacement_once_each`)
  e na GPU com dois renderers (`a_stream_image_draws_its_latest_frame_in_every_context`).
- condição 2: um cursor novo escreve o log como está, então o último
  frame (`a_new_atlas_gets_the_latest_frame`); na GPU um renderer criado
  depois e um cujo device foi destruído e recuperado desenham o último.
- condição 3: cinquenta trocas deixam `upload_count`, `image_count` e
  `atlas_size` como estavam, e a entrada guarda um frame
  (`a_stream_is_one_slot_whose_pixels_change_in_place`). a página cria o
  slot no primeiro frame e o guarda: desligar e religar reusa (no ADR).
- condição 4: o dedupe segue para as outras imagens
  (`the_dedupe_holds_beside_streams`), e nenhum teste que existia no
  `image.rs` mudou. `image_count` passa a contar os slots de fluxo
  (semântica nova, registrada); o `prepare` olha o log em vez do mapa do
  dedupe, o que é o mesmo para imagens fixas.
- condição 5, dito sem consertar: a gravação (`PLEV_RECORD=1`,
  `rust/record`) guarda um asset de imagem por handle que viu, e cada
  frame da câmera é um `id` novo, então um asset por frame gravado, só com
  o tamanho: `put_asset` escreve imagem viva sem pixels, nenhum pixel da
  câmera vai para o `.monster`. a memória do gravador segue o orçamento
  dele (os assets têm referência e saem com o último chunk). uma sessão
  lida de volta desenha esses nós como nada; um replay vivo no mesmo
  processo desenharia o que o slot tem agora, não o frame de então. o
  `--svg` (`engine::headless::svg`) marca a imagem como deixada de fora,
  um comentário com o tamanho e uma aproximação relatada, como para toda
  imagem; e o `--svg` do showcase monta a view sem captura, então a
  página sai sem imagem nenhuma.

## o trait e os estados

`rust/engine/src/runtime/platform/capture/mod.rs` (486 de produção, 609 com os
testes), `capture/ffmpeg.rs` (218), `capture/ffmpeg/args.rs` (258, 461
com os testes), `capture/ffmpeg/child.rs` (317):

- `DeviceKind::{Camera, Microphone}`; `Device { kind, name, index }`
  (`first`, `at`, `named`); `CaptureSettings { width, height, fps, rate,
  camera, microphone }` com `Default` 640x360 a 15 fps, 16 kHz, índice 0;
  `Frame { width, height, rgba, at }` e `AudioChunk { samples, rate, at }`
  (os `Debug` dão tamanhos, nunca os bytes), `rms`;
  `CaptureState::{Off, Starting, On, Denied, NoDevice, Failed(String)}`
  (`is_live`: `Starting` ou `On`); `CaptureError::{Unsupported,
  NoDevice(String), Io(String)}`.
- `trait Capture { start(kind, &settings); stop(kind); state(kind);
  latest_frame(); take_audio() }`: nada bloqueia. o frame mais novo vence
  (um slot de um: o que ninguém pegou é trocado, e o buffer dele é
  reusado na leitura seguinte); o áudio em chunks de 1/20 s num anel de 2
  s, os mais velhos saem primeiro. `stop` solta o dispositivo e descarta o
  que ninguém pegou.
- `FfmpegCapture`: um filho por dispositivo, `PLEV_FFMPEG` ou o `ffmpeg`
  do PATH, nada roda ao construir; uma thread lê o stdout, uma o stderr
  (as últimas 128 linhas), e um vigia pergunta ao filho a cada 20 ms se
  acabou: o estado vem de todas as linhas (`classify`); sem nada em 15 s
  mata pelo pid e fica `Failed("nothing from the camera in 15 s")`; uma
  câmera que recusou o padrão e listou os modos recomeça uma vez com o
  menor modo que cobre o tamanho pedido e aceita a taxa pedida
  (`-framerate` e `-video_size` só nesse caso, com o que a câmera
  listou). `stop` mata pelo pid e colhe (`wait`); `Drop` para os dois.
  um pai que morre sem parar deixa o ffmpeg escrevendo num pipe sem
  leitor: a escrita seguinte falha e ele sai. `pid(kind)` para os
  testes; `input(Input)` e `first_frame(d)` para os falsos.
- `FakeCapture` (frames de uma cor por frame, `FakeCapture::pixel(n)`; um
  seno de 440 Hz de amplitude 1/8, `FakeCapture::RMS`; o estado de cada
  start alinhável com `will_come_to`; `starts`, `stops`, `frames`),
  `NoCapture` (wasm e mobile, `Unsupported`), `default_capture()`, e
  `Capture` para `Arc<Mutex<C>>` e `Box<C>`.

## o showcase: a página, a privacidade, coberto e dormindo

`view/apps/capture/mod.rs` (607 de produção), `capture/semantics.rs`
(237), `capture/tests.rs`; `view/dispatch.rs` 517 (era 467),
`view/input.rs` 423 (era 417), `view/panes.rs` 141 (era 136),
`view/pages/table.rs` 520 (era 513), `view/pages/mod.rs` 158 (era 157),
`view/apps/mod.rs` 20 (era 17), `app.rs` 571 (era 567),
`app/capture.rs` 37 (138 com o teste), `app/sleep.rs` 55 (era 50),
`app/detached.rs` 97 (era 96).

- a página "Capture" no Apps Demo, depois de Files: dois switches
  (`capture.camera`, `capture.microphone`, `Role::Toggle`, `on` é o que a
  pessoa ligou, ação `toggle` com `PERSON_ONLY`, a política do shell,
  agora reexportada de `view::dispatch`), a caixa do frame no tamanho da
  câmera (mais estreita se o card for), o medidor (`ProgressBar`, dB sobre
  um piso de -60, `FLOOR_DB`) e o status. a região projeta
  `capture:camera` ("off", "starting", "on 640x360 at 15 fps", "permission
  denied", "no camera", "failed: ...", e "paused while the page is not
  seen" quando a pessoa ligou e a página não está à vista) e
  `capture:microphone` (o mesmo, "on 16000 Hz"); `capture.frame:showing`,
  `capture.level:level` (em décimos, para a voz não mexer a projeção a
  cada chunk). pixels e amostras nunca são projetados.
- privacidade: um agente pedindo `toggle` recebe `NotAuthorized` antes do
  `execute`, nada liga, e ele lê as linhas. a página nasce com
  `NoCapture`; só o shell, com a janela de pé, dá o `default_capture()`
  (`App::install_capture`, ao lado do `install_print`). então uma galeria
  headless, um baseline, o `--render` e todo teste de integração não
  alcançam um dispositivo.
- coberto ou dormindo: o dispositivo só roda enquanto a pessoa quer e a
  página pode ser vista (é a página ativa, uma janela que a mostra está
  acordada, o app não dorme). o shell passa a palavra das janelas no
  próprio evento (`App::capture_follows` em `covered`, `sleep`, `wake` e
  ao fechar a janela da página), porque uma janela coberta só tica quando
  algo acorda o laço; a página para o dispositivo (o filho morre, a luz
  apaga), esvazia a caixa e descarta o que não foi pego; ao mostrar ou
  acordar, liga de novo se a pessoa tinha ligado, e a caixa espera um
  frame novo. sair da página faz o mesmo (o `page_wake` diz à página, a
  cada wake, se ela é a ativa). decidido: soltar, não pausar; está no
  ADR. um dispositivo que acabou sozinho (negado, falhou) não recomeça a
  cada wake: só quando a pessoa desliga e liga, ou a página sai de vista
  e volta.
- o frame novo só entra na textura no `wake` com a página visível (nunca
  num frame, nunca dormindo); com um dispositivo vivo o `Wake` pede
  `after(1/fps)`; sem nenhum, nada.
- teclas: Tab anda pelos dois switches, Enter liga o focado, Escape solta
  (os braços em `input.rs`).

## testes

- `engine::gpu::image` (+5): um slot por fluxo e cinquenta trocas; as
  recusas e os textos; dois cursores; o cursor novo com o último frame; o
  dedupe ao lado.
- `engine::platform::capture` (+11): as settings; o falso (frames novos,
  o seno, o desligado); o falso alinhado; o nível; `NoCapture` e o
  compartilhado; a entrada de cada plataforma por índice e nome; a saída;
  os modos e o retry; o `classify` de todas as linhas; o anel; construir
  sem rodar e o programa que não existe.
- `rust/engine/tests/capture.rs` (10 e 1 ignorado; binário novo): a
  sonda (câmera, microfone, frames num fluxo e desenhados, a imagem de
  fluxo em três contextos com device recuperado), e o `FfmpegCapture`
  sobre os falsos: frames do tamanho pedido a uns 15 fps com o mais novo
  vencendo e um frame só esperando; o microfone com chunks de 800 e o anel
  em 32 000 amostras depois de 3 s sem ninguém pegar; parar mata o pid
  (câmera e microfone são filhos diferentes; o drop mata o que sobra); o
  prazo mata o mudo em 2 s; negado, sem dispositivo e falha pelas
  palavras; o retry com `-framerate 15 -video_size 640x480`. os falsos
  deixam os argumentos em `$TMPDIR/plev-capture-fakes/<pid>.args`, e cada
  teste apaga os seus.
- `view::apps::capture` (+9): a projeção em repouso e a política; o
  agente recusado, a pessoa e a tecnologia assistiva aceitas; o frame
  entra no wake e não antes (o mesmo slot, id novo, desenhado); o prazo só
  com dispositivo vivo; coberto, dormindo e fora da página solta e volta;
  as recusas como linhas sem recomeçar; o medidor; o clique e as teclas; o
  card num telefone e numa janela larga.
- `app::capture` (+1): o shell sobre janela nua, o agente recusado, a
  pessoa liga, coberto solta na hora e cinco ticks sem frame, mostrar
  religa, dormir solta, acordar religa, outra página solta, voltar religa.
- `apps/uigallery/tests/capture.rs` (1, GPU; binário novo): o frame
  desenhado antes do wake mostra a caixa vazia, o primeiro depois do wake
  mostra o frame 0 do falso dentro da caixa, o seguinte o frame 1 no mesmo
  lugar; ligado, um prazo de um período; não visto, a caixa vazia no
  frame seguinte e nada pedido; visto de novo, um frame novo.

## gate e regress

`df -h /` antes de cada gate: 18 GB livres; 18 GB depois. `./tool/tasks/fullcheck.sh`
verde no worktree com `CARGO_NET_OFFLINE=true`, com a perna wasm e a
perna urnaui (`../urna` presente), duas vezes (a segunda depois de
`de4a4d98`): antes (a base `9ea37ea9`, medida neste worktree) 2604
passed, 0 failed, 16 ignored em 126 binários; depois 2641 passed, 0
failed, 17 ignored em 128 binários (+37: 5 do `gpu::image`, 11 do
`platform::capture`, 10 de `engine/tests/capture.rs`, 9 da página, 1 de
`app::capture`, 1 de `showcase/tests/capture.rs`; +1 ignorado, o
atlas cheio; +2 binários, os dois `tests/capture.rs`). perna urnaui: 85
passed, 0 failed, 1 ignored em 6 binários, igual a antes. clippy `-D
warnings`, fmt e a checagem wasm limpos nas duas pernas; o único aviso no
log é o `block v0.1.6` de incompatibilidade futura, que já estava.

`cargo run --offline -p regress -- check`: 163
baselines, 163 iguais, 0 mudaram, sem frame desenhado e sem GPU aberta.
os 161 de antes não mudaram; os dois novos são `capture@core@1280x800` e
`capture@hoff@1280x800` (`bless --page Capture --theme core --png` e
`--theme hoff --png`), a página em repouso: os dois switches desligados,
a caixa vazia dizendo "off", o medidor em zero.

linhas de produção: acima; tudo abaixo de 639 (`file_hygiene_guard`
verde). a página está em 607: o próximo que tocá-la tira o `render` para
um `draw.rs`.

## ADR

[`0062-capture-is-a-job-the-person-turns-on.md`](../../adr/Interface/0062-capture-is-a-job-the-person-turns-on/0062-capture-is-a-job-the-person-turns-on.md) (proposed): o trait, o
ffmpeg como filho e por quê, a permissão como estado, só pessoa, a imagem
de fluxo, coberto ou dormindo solta o dispositivo, o que o bring-up muda.
linha para o índice ([`adr/`](../../adr/README.md) está fora da lista, fica para a
integração):

```text
| 2026-09-24 | [capture-is-a-job-the-person-turns-on](capture-is-a-job-the-person-turns-on.md) | proposed | task-76 item 10: `engine::platform::capture` põe câmera e microfone atrás de um trait (`Capture`: começar e parar cada `DeviceKind` sem bloquear o laço, `CaptureState` Off, Starting, On, Denied, NoDevice, Failed, o `Frame` RGBA mais novo, um só guardado, e os `AudioChunk` mono f32 num anel de 2 s); `FfmpegCapture` roda um ffmpeg por dispositivo na entrada da plataforma (avfoundation, dshow, v4l2 e alsa), lê o pipe cru, lê todo o stderr como estado, mata pelo pid no prazo de 15 s e ao parar, e recomeça uma câmera que recusou a taxa padrão com um modo que ela listou, sem dependência; a permissão é estado, nunca falha (o texto da recusa é palpite até a prova real); o frame vai para uma imagem de fluxo (`stream_image_rgba`/`replace_image_rgba`: um slot trocado no lugar com id novo, escrito uma vez por atlas pela revisão, o último frame num atlas novo) no wake; no showcase `capture.camera` e `capture.microphone` são só de pessoa (`PERSON_ONLY`), o agente lê `capture:camera` e `capture:microphone`; coberta, dormindo ou fora da página o dispositivo é solto na hora e volta ao mostrar se a pessoa o tinha ligado; provado com falsos e lavfi, nenhum dispositivo aberto |
```

## interfaces adicionadas (só crescimento, salvo o dito)

- `engine::gpu::image::{ImageStore::stream, ImageStore::replace,
  stream_image_rgba, replace_image_rgba}`, `ImageError::{NotAStream,
  WrongSize}` (o `match` de fora não existia; nada quebrou), reexportadas
  em `engine::gpu`. `ImageStore::image_count` conta os slots de fluxo
  (semântica nova); `ImageAtlasGpu::prepare` olha o log.
- `engine::platform::capture::{DeviceKind, Device, CaptureSettings,
  Frame, AudioChunk, rms, CaptureState, CaptureError, Capture,
  FakeCapture, NoCapture, default_capture, FfmpegCapture, Input,
  FFMPEG_ENV}` e `engine::platform::capture` em `platform/mod.rs` (uma
  linha).
- showcase: `Pane::Capture`, a página (`view::apps::capture::CapturePage`),
  `ShowcaseView::{set_capture, set_capture_shown}` (em `dispatch.rs`),
  `view::dispatch::PERSON_ONLY` (`pub(crate)`, reexportado do `shell`),
  `App::{install_capture, capture_follows}` (`pub(super)`). a projeção da
  galeria ganha "Capture" nas opções de `shell.page`; nenhuma assinatura
  pública mudou; ide, urnaui, wasm e os exemplos compilam sem tocar.

## arquivos tocados fora da lista permitida, e por quê

- `rust/engine/src/drawing/gpu/image.rs` e `rust/engine/src/drawing/gpu/mod.rs`:
  liberados pelo orquestrador depois da parada (a opção 1).
- `apps/uigallery/src/views/apps/mod.rs`: a linha `pub mod capture;` (e
  o doc do grupo), sem a qual o diretório novo não é módulo.
- `apps/uigallery/src/shell/mod.rs`: `mod capture;` e a chamada
  `self.install_capture();` no `resumed`, ao lado do `install_print()`. a
  lista dizia `app/**`; o `app.rs` é a raiz desse diretório, e o
  `resumed` mora nele.
- `tool/bench/regress/baselines/capture@core@1280x800` e
  `capture@hoff@1280x800`: os baselines novos da página, que o briefing
  permitia se a sonda (c) mostrasse os outros iguais.
- `apps/uigallery/src/views/dispatcher/mod.rs` está na lista pelos braços; além
  deles recebeu a reexportação do `PERSON_ONLY` e os dois métodos de
  `ShowcaseView` que o shell chama (o `view/mod.rs` está fora).

## limitações e decisões, sem esconder

- nenhum dispositivo real foi aberto: a câmera e o microfone de verdade,
  o diálogo de permissão, a luz, o texto de uma recusa, os modos de uma
  câmera que não aceita 29,97 e o silêncio de um microfone negado não
  foram vistos. o que o `classify` lê como `Denied` é palpite; o retry lê
  o formato das strings da libavdevice, não uma saída vista. a prova real
  é do brenner (roteiro abaixo), e o que ela mostrar entra no `args.rs`.
- sem `-video_size` na entrada, uma câmera que abre em 1080p é escalada
  pelo ffmpeg a cada frame que fica (o `fps` vem antes do `scale` desde
  `de4a4d98`, então os que saem não são escalados); o custo de CPU do
  ffmpeg não foi medido com um dispositivo.
- o frame não é espelhado (a vista de selfie costuma ser); o áudio é mono
  a 16 kHz e a página só lê o nível: o buffer está no trait para quem o
  quiser.
- o slot de fluxo nunca é liberado (o atlas não despeja); um por página.
- o prazo do primeiro frame é 15 s: uma pessoa que demora mais no
  diálogo vê "failed: nothing from the camera in 15 s" e liga de novo
  depois de responder.
- no windows o índice 0 sem nome vira "failed: capture: dshow opens a
  camera by its name, and none was given" até o bring-up dar a lista.
- o shell só recalcula "visível" nos eventos de janela e no fechar a
  janela da página; abrir a janela da página com a principal coberta não
  acontece (o `D` é apertado numa janela visível).
- a projeção muda o `shell.page` (uma opção a mais) na projeção da
  galeria; a da página (`ShowcaseView::semantics`) das outras páginas não
  mudou.
- a página tem 607 linhas de produção.
- gravação e `--svg`: condição 5, acima, sem conserto.

## o que fica para o bring-up e o que o item 6 reaproveita

- bring-up: AVFoundation direto (a permissão antes do start, `Denied`
  dito pelo sistema, a lista de dispositivos e a escolha na página); um
  `.app` com `NSCameraUsageDescription` e `NSMicrophoneUsageDescription`
  no `Info.plist` (o bundle passa a ser o processo responsável, também
  pelo ffmpeg filho); windows com os nomes do dshow; linux com v4l2 e
  alsa ou pulse numa máquina real; mobile com as APIs do sistema.
- item 6 (deep links): um deep link vira `Request` pelo dispatcher com um
  ator; `capture.camera` e `capture.microphone` são `PERSON_ONLY`, então
  um deep link nunca liga câmera nem microfone, a menos que o item 6
  decida que um deep link é uma pessoa, e aí a câmera pede a confirmação
  humana da task-58 antes. o mesmo vale para `shell.print` e as janelas.

## o roteiro da prova real (do brenner, nunca de teste)

a luz da câmera acende nesta parte; nada disto foi feito aqui.

```text
RUST_LOG=engine=debug,showcase=info cargo run --offline -p uigallery -- Capture
  clicar no switch "camera" (ou Tab e Enter). no log:
    "capture: the page starts the camera"
    "capture: camera with ffmpeg -hide_banner -nostdin -loglevel error -f avfoundation -i 0:none -vf fps=15,scale=640:360:... -f rawvideo -pix_fmt rgba pipe:1"
  o macOS pergunta se o app que rodou o cargo (Terminal, iTerm, Ghostty, o terminal do Zed) pode usar a câmera:
  o pedido é do processo responsável, não do plev nem do ffmpeg.
  permitir: a luz acende, "capture: camera on" no log, a caixa mostra a câmera, a linha diz "on 640x360 at 15 fps".
  se o log disser "the camera refused its default, again at <w>x<h> and <n> fps", o retry rodou: anotar a linha e os modos.
  negar: a linha deve dizer "permission denied". as linhas do stderr aparecem como "capture: camera: ..." (engine=debug):
  se a página disser "failed: ..." em vez disso, esse texto é o da recusa real e falta no classify (capture/ffmpeg/args.rs).
  sem responder o diálogo por 15 s: "failed: nothing from the camera in 15 s"; responder e ligar de novo.
o microfone: o switch "microphone", o diálogo do microfone, falar e ver o medidor subir; a linha "on 16000 Hz".
  negado: o palpite é silêncio (o medidor parado em 0 com "on 16000 Hz"); anotar o que acontecer.
cobrir: cmd-H ou minimizar. no log "capture: the page stops the camera"; a luz apaga;
  pgrep -fl "ffmpeg.*avfoundation"   (nada). restaurar: a luz acende de novo e a caixa espera um frame novo.
  ir para outra página (Files) e voltar: o mesmo.
o agente, em outro terminal, com o app rodando com PLEV_AGENT=1:
  cargo run --offline -p socket --bin plev -- observe --scope capture       (capture:camera, capture:microphone)
  cargo run --offline -p socket --bin plev -- act capture.camera toggle     (recusado: NotAuthorized; nada liga)
desfazer: Ajustes do Sistema > Privacidade e Segurança > Câmera (e Microfone): desligar o app do terminal;
  ou tccutil reset Camera e tccutil reset Microphone (todos os apps), ou só um: tccutil reset Camera com.apple.Terminal
a lista dos dispositivos, se quiser escolher outra câmera: ffmpeg -f avfoundation -list_devices true -i ""   (também pode pedir a permissão)
```

o que olhar: a luz acesa só com a página à vista e a pessoa tendo ligado;
nenhum ffmpeg vivo depois de cobrir, dormir, sair da página ou fechar o
app; o texto real da recusa e dos modos, para o `args.rs`.

## comandos de validação

```text
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/one
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_PROFILE_TEST_DEBUG=line-tables-only
cargo test --offline -p engine --lib -- gpu::image platform::capture
cargo test --offline -p engine --test capture -- --nocapture           (a sonda imprime as medidas; precisa do ffmpeg no PATH)
cargo test --offline -p engine --test capture -- --ignored --nocapture  (o atlas cheio: 244 frames, 225 MB)
cargo test --offline -p uigallery --lib -- apps::capture app::capture
cargo test --offline -p uigallery --test capture
cargo test --offline -p engine -p uigallery
./tool/tasks/fullcheck.sh
cargo run --offline -p regress -- check              (163 iguais)
cargo run --offline -p uigallery -- --render Capture --out capture.png   (a página em repouso, sem dispositivo)
```

## integração do item 10 (orquestrador, 2026-09-24)

- merge da `task/76i-capture` na `main` sem commit a mais. a revisão
  conferiu a imagem de fluxo (o crescimento do atlas copia no mesmo
  lugar, o cursor vale), a privacidade (nenhum teste chega a dispositivo,
  a página começa com `NoCapture`) e o prazo do laço (o `Starting` conta
  como vivo).
- P3: no tema core o interruptor da câmera e do microfone quase não
  aparece (baseline `capture@core`).
