---
id: task-69/phase-4a
title: "task-69, phase 4a: the silences app and the fixed screen"
project: plev
type: section
parent: task-69
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [agents, platform]
relations: [adr-0053, task-93]
evidence: [4e3595ae, ae7f113d, 857cdd77, 436c0647, d8641a40, 72f49b00, f305e67e]
source: docs/log/steps/task-69-ephemeral-generative-surfaces.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase 4a: the silences app and the fixed screen

Records phase 4a: the silences app, with a pure model, the parser for the output of ffmpeg's `silencedetect`, and a worker that runs ffmpeg as a child process with a deadline, plus the review's fixed screen. The result is real detection of two segments, in 28 to 128 ms, and frames decoded in 26 to 30 ms each, operated through the gallery's dispatcher; the fixed screen is the reference for checkpoint b.


branch `task/69f-silences-app`, worktree próprio, sobre a `main` @
205d0e1d. o cenário da seção 4 da tese (o editor de vídeo) precisa, antes
da tela gerada (4b), do app com as capacidades de verdade e de uma tela
fixa bem desenhada para a mesma tarefa: é contra ela que o checkpoint b
compara a tela gerada. esta fase é esse app e essa tela, uma página do
grupo Apps Demo.

## sonda da fase 4a (antes do primeiro tipo, 2026-09-24)

o `ffmpeg` desta máquina é o 9.0.1 em `/opt/homebrew/bin/ffmpeg`. tudo
rodou num diretório temporário da sessão; um teste descartável do
showcase (`tests/silences_probe.rs`, rodado e jogado fora, nunca
commitado) fez a parte (b) que precisa do engine. nenhuma janela.

(a) o vídeo de teste, gerado por `lavfi` sem arquivo de entrada: 12 s,
`testsrc` 320x180 a 10 fps, áudio `sine` de 440 Hz a 44100 Hz, dois
trechos de silêncio (3 a 5 s e 8 a 10 s) pelo filtro `volume` com
`enable` na linha do tempo:

```text
ffmpeg -hide_banner -nostdin -y -v error \
  -f lavfi -i "testsrc=size=320x180:rate=10" \
  -f lavfi -i "sine=frequency=440:sample_rate=44100" -t 12 \
  -af "volume=enable='between(t,3,5)+between(t,8,10)':volume=0" \
  -c:v libx264 -preset ultrafast -crf 28 -pix_fmt yuv420p -c:a aac -b:a 64k \
  silences-test.mp4
```

162.908 bytes, os mesmos nas três gerações (o arquivo é determinístico);
`ffprobe`: h264 320x180 10/1, aac 44100 Hz mono, `duration=12.000000`;
81 a 83 ms de parede por geração. a detecção, `ffmpeg -hide_banner
-nostdin -nostats -i silences-test.mp4 -af silencedetect=noise=-30dB:d=0.5
-f null -`, o stderr inteiro (o stdout tem 0 bytes):

```text
Input #0, mov,mp4,m4a,3gp,3g2,mj2, from 'silences-test.mp4':
  Metadata:
    major_brand     : isom
    minor_version   : 512
    compatible_brands: isomiso2avc1mp41
    encoder         : Lavf63.1.101
  Duration: 00:00:12.00, start: 0.000000, bitrate: 108 kb/s
  Stream #0:0[0x1](und): Video: h264 (Constrained Baseline) (avc1 / 0x31637661), yuv420p(progressive), 320x180 [SAR 1:1 DAR 16:9], 59 kb/s, 10 fps, 10 tbr, 10240 tbn (default)
    Metadata:
      handler_name    : VideoHandler
      encoder         : Lavc63.1.101 libx264
  Stream #0:1[0x2](und): Audio: aac (LC) (mp4a / 0x6134706D), 44100 Hz, mono, fltp, 45 kb/s (default)
    Metadata:
      handler_name    : SoundHandler
Stream mapping:
  Stream #0:0 -> #0:0 (h264 (native) -> wrapped_avframe (native))
  Stream #0:1 -> #0:1 (aac (native) -> pcm_s16le (native))
Output #0, null, to 'pipe:':
  Metadata:
    major_brand     : isom
    minor_version   : 512
    compatible_brands: isomiso2avc1mp41
    encoder         : Lavf63.1.101
  Stream #0:0(und): Video: wrapped_avframe, yuv420p(progressive), 320x180 [SAR 1:1 DAR 16:9], q=2-31, 200 kb/s, 10 fps, 10 tbn (default)
    Metadata:
      encoder         : Lavc63.1.101 wrapped_avframe
      handler_name    : VideoHandler
  Stream #0:1(und): Audio: pcm_s16le, 44100 Hz, mono, s16, 705 kb/s (default)
    Metadata:
      encoder         : Lavc63.1.101 pcm_s16le
      handler_name    : SoundHandler
[Parsed_silencedetect_0 @ 0x71903dd40] silence_start: 3.018594
[Parsed_silencedetect_0 @ 0x71903dd40] silence_end: 5.01551 | silence_duration: 1.996916
[Parsed_silencedetect_0 @ 0x71903dd40] silence_start: 8.010884
[Parsed_silencedetect_0 @ 0x71903dd40] silence_end: 10.0078 | silence_duration: 1.996916
[out#0/null @ 0x71903c300] video:50KiB audio:1034KiB subtitle:0KiB other streams:0KiB global headers:0KiB muxing overhead: unknown
frame=  120 fps=0.0 q=-0.0 Lsize=N/A time=00:00:12.00 bitrate=N/A speed=2.07e+03x elapsed=0:00:00.00
```

os dois trechos: `silence_start: 3.018594`, `silence_end: 5.01551 |
silence_duration: 1.996916`; `silence_start: 8.010884`, `silence_end:
10.0078 | silence_duration: 1.996916` (o aac tem 18 ms de ataque e o
`d=0.5` conta a partir do limiar cruzado). a linha `Duration:
00:00:12.00` do mesmo stderr dá a duração sem um `ffprobe` à parte.
cinco corridas da detecção: 77 ms a primeira (o cache do disco), depois
41 a 43 ms. um áudio de 6 s calado a partir dos 4 s (o caso "silêncio
até o fim"): o ffmpeg 9 escreve `silence_end: 6 | silence_duration:
1.983039` no fim do stream, mas versões anteriores deixavam o
`silence_start` sem `silence_end`, e o parser cobre os dois.

(b) um frame do meio do primeiro trecho, `ffmpeg -hide_banner -nostdin
-v error -ss 4 -i silences-test.mp4 -frames:v 1 -f image2 -c:v png
pipe:1`: 12.978 bytes, `PNG image data, 320 x 180, 8-bit/color RGB`; com
`-vf "scale='min(160,iw)':-2"` aos 9 s: 6.954 bytes, 160x90. 39 a 42 ms
de parede por frame. o teste descartável leu os dois arquivos e chamou
`engine::gpu::image::load_image_bytes`: `12978 bytes -> ImageHandle {
id: 0, 320x180, atlas (1, 1) }` e `6954 bytes -> ImageHandle { id: 1,
160x90, atlas (513, 193) }`, sem GPU (o atlas é a metade de CPU).

(c) o `silencedetect` escreve no stderr, não no stdout: com o stderr
descartado o stdout tem 0 bytes; com o stdout descartado o stderr tem
as quatro linhas acima. o nível: `-loglevel quiet`, `error` e `warning`
não escrevem nenhuma linha com `silence` (o filtro loga em `info`);
`info` (o padrão) escreve as quatro mais a linha `Input #0`; `verbose`
escreve sete (duas linhas de configuração do filtro a mais);
`level+info` prefixa `[info]` depois da tag do filtro:
`[Parsed_silencedetect_0 @ 0xc1503dd40] [info] silence_start: 3.018594`.
a tag com o endereço muda a cada corrida e o parser não a lê: procura
`silence_start:`, `silence_end:` e `silence_duration:` em qualquer
lugar da linha. `-nostats` tira a linha de progresso (que, num vídeo
longo, vem com `\r`); `-nostdin` impede o filho de ler o terminal.

nenhuma condição de parada disparou: o ffmpeg gerou o vídeo, detectou os
dois trechos, decodificou o frame e o atlas o aceitou.

o que a sonda decide para o desenho:

- o trabalhador roda o ffmpeg com `-hide_banner -nostdin -nostats
  -loglevel info` na detecção e lê o stderr até o fim (a saída pode
  passar do buffer do pipe: um dreno numa thread, como o `run.rs` do
  `claude-model`); com `-v error` na decodificação e lê o png do stdout;
- o parser lê `Duration:` e as três chaves do `silencedetect` de
  qualquer linha, fecha um `silence_start` sem `silence_end` na duração
  e ignora o resto do texto;
- o vídeo de teste é gerado pelo mesmo trabalhador, pelo comando acima,
  em `std::env::temp_dir()/plev-silences/silences-test.mp4`, quando não
  existe;
- um frame por instante, em png de até 480 px de largura (o vídeo de
  teste fica em 320x180), carregado no atlas na thread do trabalhador;
  o `ImageHandle` cruza o canal.

## commits até a parada

- 4e3595ae task-69f: a sonda da fase 4a no arquivo da task
- ae7f113d task-69f: the silences model and the silencedetect parser,
  tested first (`apps/uigallery/src/domain/silences/mod.rs`, `silences/parse.rs`,
  `silences/probe.stderr` como fixture e semente, `silences/tests.rs`,
  11 testes; a linha `pub mod silences;` em `model/mod.rs`)

## parada: registrar a página passa pelo `dispatch.rs`

problema: a página do item 3 não se registra só com os arquivos da
lista. uma página do showcase é uma linha de `PAGES` (`table.rs`, na
lista) com um `Pane` (`view/pages/mod.rs`), o estado dela em `Panes`
(`view/panes.rs`) e as rotas de `view/dispatch.rs`, que casa `Pane` de
forma exaustiva em sete lugares: `page_content_height`, `page_event`,
`page_wake`, `page_render`, `semantics`, `act` e `assist_focus`. um
`Pane` novo não compila sem um braço em cada um. o briefing diz para
parar antes de mexer no `dispatch.rs` "que está congelado no tamanho";
o estado da árvore: o `dispatch.rs` saiu do `FROZEN` na fase 1 (o
guard `tool/tests/test_filepaths.rs` não o lista mais) e
tem 338 linhas para o teto 639; os sete braços somam umas 30. o
trabalhador do ffmpeg e a página, escritos sem o registro, seriam código
morto para o clippy (`-D warnings`), então os commits 3 e 4 esperam a
decisão.

opções:

1. um `Pane::Silences` novo, como o `Pane::Surface` da fase 1 (a
   resolução daquela parada liberou os mesmos arquivos). fora da lista:
   `view/pages/mod.rs` (a variante), `view/panes.rs` (o campo e a
   construção), `view/dispatch.rs` (os sete braços; de 338 para umas
   370 linhas) e `view/input.rs` (dois braços, `handle_edit_key` para o
   Tab entre os sliders e os botões e `close_top_overlay` para o Escape
   tirar o foco; opcionais, sem eles a página fica sem ciclo de Tab
   próprio). impacto: quatro arquivos fora da lista, nenhum congelado; a
   página se projeta para o agent port, o leitor de tela e o `--render
   --semantics` pelas mesmas rotas das outras, e o `regress` a abençoa.
2. nenhuma página nesta fase: o modelo (feito), o trabalhador e o
   `SilencesPage` com a projeção e o `execute` provados por testes de
   unidade sobre a struct (sem passar pela `ShowcaseView`), e o PNG
   saindo do renderer headless num teste `#[ignore]`. impacto: nenhum
   arquivo fora da lista; sem `--render Silences`, sem baseline do
   `regress`, sem a prova pela porta real, e a página só aparece quando
   a rota for autorizada. o checkpoint b compara contra uma tela que
   ninguém abre no showcase.

recomendação: 1. é a página que o briefing descreve, com o contrato das
outras, e o motivo da parada (o arquivo congelado) não vale mais.

decisão necessária: qual opção; na 1, a autorização para os quatro
arquivos.

## decisões de desenho fixadas até aqui (para objeção cedo)

- a página abre com a semente da sonda (`SilencesModel::seeded`: o vídeo
  de teste com 12 s e os dois trechos, tudo pendente, o instante 0
  mostrado), como o todo abre com três itens. razão: o golden, a
  baseline do `regress` e o `--render` ficam iguais em toda máquina, com
  ou sem ffmpeg, e `apply_cut`, `reject_cut` e `apply_all` funcionam
  também no wasm. a detecção real substitui a semente com ids novos.
- o ffmpeg só roda depois de um pedido de um ator (`detect_silence` ou
  `preview_segment`), nunca ao abrir a página. o briefing diz "gerado
  quando a página abre"; o headless (`stage_page`) abre a página do
  mesmo jeito que a janela, e um processo começado ali deixaria o frame
  da baseline dependente do tempo do filho (120 passos de settle contra
  80 ms de geração) e escreveria em `/tmp` num render. o gancho que
  distingue a janela é o shell (`connect_git` em `app.rs`, proibido
  nesta fase); um `connect_silences` ali, noutra fase, faz a geração ao
  abrir. o vídeo é gerado pelo primeiro job que precisa dele, em
  `std::env::temp_dir()/plev-silences/silences-test.mp4`, e fica lá (só
  o sistema limpa); existindo, é reutilizado.
- o trabalhador roda um job por vez (gerar se preciso, detectar,
  decodificar um frame), cada um um processo filho numa thread com
  prazo e morte pelo pid (a forma do `run.rs` do `claude-model`, sem
  depender dele); o resultado volta por canal e o `wake` da página
  recolhe (`Wake::after(100 ms)` enquanto há job). `detect_silence` e um
  `preview_segment` cujo frame ainda não existe devolvem
  `Execution::Started` e terminam por `complete`/`fail` no `wake`, pelo
  `with_dispatcher` da página; os frames dos trechos (o meio de cada
  um) são pedidos pela própria página depois de uma detecção, um por
  vez, e ficam em cache por instante (`ImageHandle`, carregado no atlas
  na thread do trabalhador). nos testes do crate o ffmpeg nunca é
  procurado (`cfg(test)`, como `Claude::find`); o teste que quer o
  falso o aponta, e o `#[ignore]` aponta o real.
- os sliders são do widget: o arraste da pessoa muda o valor no `Slider`
  sem request, como a tecla no campo de intenção e o clique no select
  da 3b-ii; `set_value` de um agente passa pelo dispatcher. o modelo
  guarda as configurações com que os trechos foram detectados
  (`detected_with`), não as dos sliders; quando os sliders se afastam
  delas o status diz "detect again".
- os ids: `silences` (região; `cuts`, a lista dos ids na ordem),
  `silences.video` (`Role::Media`; `file`, `duration`, `shown`),
  `silences.threshold` e `silences.min_duration` (`Slider::project`:
  `value`, `set_value`, `increment`, `decrement`), `silences.detect`
  (botão; `detect_silence` lê `silences:cuts`, os dois `value`),
  `silences.cut[<id>]` (item de lista; `start`, `end`, `duration`,
  `decision`; `preview_segment` lê `silences.video:shown`, `apply_cut` e
  `reject_cut` leem `:decision`, cada um desabilitado quando já é a
  decisão), `silences.counts` (status; `total`, `applied`, `rejected`,
  `pending`, `cut_secs`), `silences.apply_all` (botão; `apply_all` lê
  `silences.counts:pending`, habilitado com pendentes) e
  `silences.status` (`message`, `busy`). os botões aplicar e rejeitar de
  cada linha são o afforance do item, não nós próprios (como o x do
  todo), para que a superfície da 4b ligue um botão a
  `silences.cut[i] apply_cut`.
- `apply_all` aplica só o que está pendente: o que alguém rejeitou fica
  rejeitado.
- a tela: dois cards em cima (o vídeo com a linha do tempo por baixo, os
  trechos marcados pela decisão e a agulha no instante mostrado; ao
  lado, a detecção com os dois sliders e seus valores, o botão detectar
  e o status) e, embaixo, o card dos trechos (as contagens em `Stat` na
  cabeça com o aplicar todos à direita; uma linha por trecho com o frame
  do meio, o intervalo, a duração, a decisão e os dois botões). estreito,
  os cards empilham. sem scroll interno: a página cresce e o scroll do
  gallery cobre.

## resolução da parada (orquestrador, 2026-09-24)

opção 1. liberados, só para o registro da página, `view/pages/mod.rs`
(a variante), `view/panes.rs` (o campo e a construção),
`view/dispatch.rs` (os sete braços; saiu do `FROZEN` na fase 1, o
briefing estava desatualizado) e `view/input.rs` (os dois braços
opcionais); `model/mod.rs` para o `pub mod silences;`. as decisões de
desenho acima aprovadas: a semente da sonda, o ffmpeg só depois de um
pedido de um ator, um job por vez com prazo e morte pelo pid,
`Started` -> `complete`/`fail` no `wake`, frames em cache por instante,
`cfg(test)` nunca procura o ffmpeg, o arraste dos sliders fica com o
widget. o resto como no briefing.

## handoff da fase 4a (2026-09-24)

goal: task-69 fase 4a, o app dos silêncios com capacidades reais e a
tela fixa bem desenhada, contra a qual o checkpoint b compara a tela
gerada (4b): uma página do grupo Apps Demo, operável por pessoa, agente
pela porta e leitor de tela pelo mesmo dispatcher.

status: implementado. `cargo test -p uigallery` verde, `./tool/tasks/fullcheck.sh`
verde com a perna urnaui e a perna wasm, `regress check` verde, sobre a
`main` @ 205d0e1d (a contagem abaixo).

### commits

branch `task/69f-silences-app`:

- 4e3595ae task-69f: a sonda da fase 4a no arquivo da task
- ae7f113d task-69f: the silences model and the silencedetect parser, tested first
- 857cdd77 task-69f: a parada do registro da página e as decisões de desenho no arquivo da task
- 436c0647 task-69f: the ffmpeg worker, the Silences page and its projection, on the gallery's dispatcher
- d8641a40 task-69f: the golden projection and the regress baselines of the Silences page
- 72f49b00 task-69f: the projection carries seconds at ffmpeg's six decimals
- f305e67e task-69f: the frame's width cap lives with the desktop child
  (a constante estava morta no wasm sob `-D warnings`; a perna wasm do
  gate pegou)
- e o commit do ADR e deste handoff

o briefing pedia o trabalhador (3) e a página (4) em commits separados;
vieram juntos no 436c0647 porque o trabalhador sem a página é código
morto para o clippy do gate (`-D warnings`), e um commit dele sozinho
não passaria. o modelo (2) e a sonda (1) estão separados.

### a sonda

acima ("sonda da fase 4a"). em uma linha: o ffmpeg 9 gera o vídeo de
teste por `lavfi` em 81 ms (162.908 bytes, determinístico), detecta os
dois trechos (`3.018594` a `5.01551` e `8.010884` a `10.0078`) em 41 a
43 ms escrevendo só no stderr no nível `info`, e decodifica um frame em
png para `pipe:1` em 39 a 42 ms (12.978 bytes, 320x180), que o
`load_image_bytes` aceita. nenhuma condição de parada disparou.

### o modelo e o parser

`apps/uigallery/src/domain/silences/mod.rs` (303 linhas),
`silences/parse.rs` (76), `silences/probe.stderr` (o stderr real da
sonda, fixture e semente) e `silences/tests.rs` (11 testes). `Segment`
(início, fim; `duration`, `middle`, `contains`), `Decision` (pending,
apply, reject; `name`), `Cut` (id, segmento, decisão), `Settings`
(limiar em dB e duração mínima em s; faixas `-60..0` e `0.1..3.0`,
`clamped`; o padrão é o da sonda, `-30`/`0.5`), `Video` (nome, duração),
`Counts` (total, aplicados, rejeitados, pendentes, segundos cortados),
`Applied` (quantos o "aplicar todos" aplicou e as contagens depois),
`SilencesModel` (`new`, `seeded`, `set_detected` com ids novos a cada
detecção, `decide`, `apply_all` só nos pendentes, `counts`, `show`
clamped ao vídeo, `cut_at`). `parse(stderr) -> Detected { duration,
segments }`: lê `Duration: hh:mm:ss.cc` e as chaves `silence_start` e
`silence_end` em qualquer lugar da linha (a tag do filtro muda de
endereço), fecha um start sem end na duração (ffmpeg 9 escreve o end no
fim do stream; versões anteriores não), descarta um start sem end e sem
duração, ignora um end sem start, um valor que não é número e o resto do
texto. nada de ffmpeg no modelo.

### o trabalhador

`apps/uigallery/src/views/apps/silences/fixed_page/worker.rs` (600 linhas, as do
desktop num módulo `child` com `cfg`; fora do desktop `find` e `spawn`
respondem "ffmpeg runs on the desktop only" sem thread nem processo).

- jobs: `Detect(Settings)` e `Frame { ms }`; um por vez, numa fila que
  não repete um job igual (`submit`); cada um numa thread
  `silences-ffmpeg` que roda o filho e devolve `Done` por canal; `poll`
  recolhe o que acabou e começa o próximo; uma thread que some sem
  resposta vira um erro ("the worker ended without an answer");
- o filho: `run(command, timeout)` na forma do `run.rs` do
  `claude-model`, sem depender dele: stdin nulo, stdout e stderr drenados
  em threads (um png passa do buffer do pipe), `try_wait` a cada 10 ms,
  no prazo `child.kill()` mais `wait` (morte pelo pid, o erro diz "did
  not finish in N s: killed (pid P)"), 1 s de graça pelo fim da saída;
  prazos: 30 s para gerar, 60 s para detectar, 20 s por frame;
- o programa: `PLEV_FFMPEG`, ou `ffmpeg` num diretório do PATH, um
  arquivo ali, sem rodar; procurado no primeiro job e guardado (o
  caminho ou o motivo). em `cfg(test)` nada é procurado
  (`Program::Nothing`, todo job falha com "ffmpeg is not looked for in
  the crate's tests"); um teste aponta o falso (`Worker::fake(detected,
  hold)`: a detecção da fixture e um frame sintético 32x18 por instante,
  segurando a resposta até `release` quando `hold`) ou o real
  (`Worker::real`, o teste ignorado);
- o vídeo: `std::env::temp_dir()/plev-silences/silences-test.mp4` (nesta
  máquina `/var/folders/.../T/plev-silences/silences-test.mp4`), gerado
  pelo primeiro job que precisa dele quando o arquivo não existe, pelo
  comando da sonda (`-f mp4`, escrito em `.part` e movido inteiro, para
  um filho morto não deixar meio vídeo a reutilizar); existindo, é
  reutilizado. ninguém apaga: só a limpeza do sistema do diretório
  temporário. nada commitado;
- a detecção: `-hide_banner -nostdin -nostats -loglevel info -i <vídeo>
  -af silencedetect=noise=<dB>dB:d=<s> -f null -`, o stderr inteiro
  parseado; sem `Duration` é erro. o frame: `-v error -ss <s> -i <vídeo>
  -frames:v 1 -vf scale='min(480,iw)':-2 -f image2 -c:v png pipe:1`, o
  png carregado no atlas na thread do trabalhador (`load_image_bytes`),
  o `ImageHandle` cruza o canal; 480 é o teto de largura (o vídeo de
  teste fica em 320x180).

### a página

`apps/uigallery/src/views/apps/silences/mod.rs` (341 linhas: a struct, os
eventos, Tab/setas/Escape, o `wake`), `silences/semantics.rs` (605: os
ids, a projeção, o `execute`, o `dispatch` pelo `with_dispatcher` do
todo, `act`, `focus`, `ask`, `settle`, a mensagem do status),
`silences/layout.rs` (371: geometria pura), `silences/draw.rs` (344: a
cena e as bindings), `silences/tests.rs`, `tests/agent.rs` e
`tests/ffmpeg.rs`. registrada em `view/apps/mod.rs`, `view/pages/mod.rs`
(`Pane::Silences`), `view/pages/table.rs` (a linha, ícone `play`),
`view/panes.rs`, `view/dispatch.rs` (os sete braços: `content_height`,
`event`, `wake`, `render`, `semantics`, `act`, `assist_focus`) e
`view/input.rs` (`handle_edit_key` e `close_top_overlay`).

o dispatcher: a página é `Semantics + Operable` e despacha pelo
dispatcher do todo, emprestado por `TodoPage::with_dispatcher`, como a
Surface. razão: o agent port põe a sink uma vez só, nesse dispatcher
(`ShowcaseView::add_sink`), e casa o ato com os estágios por
`RequestId`; um dispatcher próprio da página seria surdo para a porta e
colidiria nos ids (os dois numeram de 1); a resolução da parada da fase
1 recusou "a porta ouvindo dois dispatchers". os receipts dos silêncios
ficam onde ficam os da galeria, numerados com eles.

a tela (1200x800 core, `cargo run -p uigallery -- --render Silences --out
silences.png --semantics silences.json`; e 390x844 hoff com `--theme
hoff --width 390 --height 844`): dois cards em cima e um embaixo. o card
do vídeo: a caixa 16:9 com o frame do instante mostrado (ajustado, nunca
ampliado; em repouso a nota "no frame decoded yet: detect, or preview a
cut, runs ffmpeg"), o relógio `0:00.0 / 0:12.0` numa pílula, a linha do
tempo com cada trecho marcado pela decisão (âmbar pendente, vermelho o
que sai, apagado o que fica) e a agulha no instante mostrado, `0:00.0` e
`0:12.0` nas pontas; um clique numa marca faz `preview_segment`. o card
da detecção: "threshold" com `-30 dB` e o slider, "minimum silence" com
`0.5 s` e o slider, o botão "detect silences" (ícone play, a largura do
card) e o status. o card dos trechos: as contagens em quatro `Stat` (2
silences, 0 applied, 0 rejected, 0.0 s cut), "apply all" à direita, e
uma linha por trecho: o frame do meio (ou o instante num tile escuro
até haver frame), "0:03.0 to 0:05.0" sobre "2.0 s of silence", a
decisão num badge (pending, cut, kept) e os botões "apply" e "reject"
(o escolhido lê escuro, `Secondary`; o outro `Outline`); a linha do
trecho que segura o instante mostrado leva a barra accent. estreito, os
cards empilham e a linha põe o badge e os botões sob os textos quando
não cabem ao lado. depois de uma detecção real (o PNG do teste ignorado,
`silences-detected.png`): o frame do `testsrc` aos 4,0 s na caixa, a
agulha dentro da primeira marca, um frame por linha, o status "2
silences in 28 ms at -30 dB, 0.5 s minimum".

a projeção em repouso (10 nós; `--semantics`, forma lean, sem os reads):

```json
[
  { "id": "silences", "role": "region", "label": "silences", "state": { "cuts": ["silences.cut[1]", "silences.cut[2]"] } },
  { "id": "silences.video", "role": "media", "label": "silences-test.mp4", "state": { "file": "silences-test.mp4", "duration": 12.0, "shown": 0.0 } },
  { "id": "silences.threshold", "role": "slider", "label": "threshold (dB)", "state": { "value": -30.0 }, "constraints": { "editable": true, "range": [-60, 0], "step": 1 }, "actions": ["set_value", "increment", "decrement"] },
  { "id": "silences.min_duration", "role": "slider", "label": "minimum silence (s)", "state": { "value": 0.5 }, "constraints": { "range": [0.1, 3.0], "step": 0.1 }, "actions": ["set_value", "increment", "decrement"] },
  { "id": "silences.detect", "role": "button", "label": "detect silences", "actions": ["detect_silence"] },
  { "id": "silences.cut[1]", "role": "list_item", "label": "0:03.0 to 0:05.0", "state": { "start": 3.018594, "end": 5.01551, "duration": 1.996916, "decision": "pending" }, "actions": ["preview_segment", "apply_cut", "reject_cut"] },
  { "id": "silences.cut[2]", "role": "list_item", "label": "0:08.0 to 0:10.0", "state": { "start": 8.010884, "end": 10.0078, "duration": 1.996916, "decision": "pending" }, "actions": ["preview_segment", "apply_cut", "reject_cut"] },
  { "id": "silences.counts", "role": "status", "label": "2 silences, 0 applied, 0 rejected, 0.0 s cut", "state": { "total": 2, "applied": 0, "rejected": 0, "pending": 2, "cut_secs": 0.0 } },
  { "id": "silences.apply_all", "role": "button", "label": "apply all", "actions": ["apply_all"] },
  { "id": "silences.status", "role": "status", "label": "status", "state": { "message": "2 silences from the recorded run at -30 dB, 0.5 s minimum: detect runs ffmpeg here", "busy": false } }
]
```

os reads: `detect_silence` lê `silences:cuts`, `silences.threshold:value`
e `silences.min_duration:value`; `preview_segment` lê
`silences.video:shown`; `apply_cut` e `reject_cut` leem
`silences.cut[i]:decision` e cada um fica desabilitado quando já é a
decisão; `apply_all` lê `silences.counts:pending` e fica desabilitado
sem pendentes; os sliders leem o próprio `value`; `detect` fica
desabilitado enquanto o trabalhador está ocupado (um segundo
`detect_silence` é `Rejected(Disabled)` antes do `execute`). o golden
`rust/plevir/tests/fixtures/uigallery-silences.snapshot.json` é essa projeção
inteira, mantida igual ao app por
`apps/uigallery/tests/plevir_fixture.rs::the_silences_projection_is_the_uir_fixture`
(`UIR_BLESS=1` regrava).

### a medição

nesta máquina (apple m4, ffmpeg 9.0.1), pelo teste ignorado
(`SILENCES_OUT=<dir> cargo test -p uigallery --lib silences::tests::ffmpeg
-- --ignored --nocapture`):

| o que | ffmpeg (ms, medido no trabalhador) | o request |
| --- | --- | --- |
| `detect_silence`, o vídeo ainda não existe (gera e detecta no mesmo job) | 194 | terminou 202 ms depois do `dispatch` |
| `detect_silence`, o vídeo existe (confere o arquivo e detecta) | 128 na primeira corrida, 28 a 29 nas seguintes | dentro de 10 ms da resposta (o poll do teste) |
| um frame (`-ss`, png, atlas) | 26 a 30, cada um dos três | um `preview_segment` de instante já decodificado: `completed` na hora |

na página em janela o `wake` olha o canal a cada 100 ms enquanto há
job, então um request termina até 100 ms depois da resposta e a tela
pede um frame só então; em repouso o `wake` é `idle` (o `stage_page`
diz `settled` em 2 passos).

### contagem do gate e o regress

| | workspace | perna urnaui | total |
| --- | --- | --- | --- |
| antes, `main` @ 205d0e1d (o handoff da 3b-ii sobre a mesma árvore) | 2404 passam, 0 falhas, 14 ignorados | 82 passam, 1 ignorado | 2486 |
| depois, esta branch @ f305e67e (`./tool/tasks/fullcheck.sh`, saída 0) | 2428 passam, 0 falhas, 15 ignorados (109 suítes) | 82 passam, 0 falhas, 1 ignorado (6 suítes) | 2510 |

24 testes novos: 11 no modelo, 11 na página, 1 pela porta real e 1 o
golden; e 1 ignorado (o ffmpeg de verdade). a contagem não cai. clippy
`-D warnings`, `cargo fmt --check` e o check wasm do showcase com `-D
warnings` verdes, dentro do gate, com a perna urnaui (nenhuma perna
pulada). `df -h /` antes de cada gate: 17, 16 e 16 GB livres. o gate
rodou três vezes nesta branch: a primeira caiu num teste do ide alheio a
esta fase (abaixo, em limitações), a segunda na perna wasm por uma
constante morta fora do desktop (o f305e67e), a terceira inteira verde.
`cargo run -p regress -- check`: 159 baselines
conferidas (as 157 de antes e as duas novas, `silences@core@1280x800` e
`silences@hoff@1280x800`, abençoadas com a imagem), 159 iguais, nenhum
frame desenhado. `view/dispatch.rs`: 338 linhas antes, 363 depois.

### testes

- `model/silences/tests.rs` (11): o parser sobre a fixture real (a
  duração e os dois trechos, com as durações que o ffmpeg imprimiu), sem
  linhas de silêncio, silêncio até o fim com e sem duração, texto
  estranho e as chaves em qualquer lugar da linha; a semente; as
  decisões e as contagens; id desconhecido; aplicar todos deixa os
  rejeitados; o instante mostrado clamped; uma detecção nova com ids
  novos; as faixas;
- `view/apps/silences/tests.rs` (11): a projeção com toda capacidade e o
  que cada uma lê; aplicar por clique e rejeitar por `Actor::Test` no
  mesmo estado, a decisão tomada desabilitada e o "apply all" sem
  pendentes recusado, a tela seguindo; aplicar todos por clique;
  `detect_silence` `Started` com o falso segurando a resposta (o status
  ocupado, um segundo detect `Disabled`, um `reject_cut` no meio
  `completed`, o `wake` pedindo o poll), a resposta no `wake` com
  `completed` e o outcome, ids 3 e 4, o trecho velho fora da projeção,
  os três frames pedidos um por vez e desenhados (três nós `Image`),
  depois `idle`; `preview_segment` por clique na linha (`Started`, o
  instante mostrado, o frame no `wake`, `completed`), de instante já
  decodificado (`completed` na hora), e por clique numa marca; o caso
  stale com o delta; inerte atrás de um overlay; os sliders por
  `set_value`, `increment` e `decrement`, fora da faixa `InvalidParam`,
  no topo `Failed`, e o status dizendo "detect again" e levando as
  configurações para a detecção; Tab, setas e Escape; largo e estreito
  com as mesmas bindings e nada fora da largura; a galeria roteando a
  página (`semantics`, `act` como `Assistive`, `assist_focus`, Tab e
  Escape pelo chrome, e um detect sem ffmpeg terminando `failed` com o
  motivo no `tick`);
- `tests/agent.rs` (1): pela porta real, sem janela, um agente observa o
  escopo `silences` com as bounds, aplica um corte da revisão que viu
  (`completed`), tenta de novo e é `rejected`, pede `detect_silence` (o
  ato `completed` depois, quando o falso responde no `tick` do loop do
  shell) e observa os trechos novos; a galeria guardou o ato como do
  agente, no alvo e na ação certos;
- `tests/ffmpeg.rs` (1, `#[ignore]`): o ffmpeg real gera o vídeo,
  detecta os dois trechos (a 0,1 s dos da sonda), decodifica os três
  frames 320x180, detecta de novo com o vídeo já lá, preview de um
  trecho já decodificado, e escreve o PNG e a projeção com
  `SILENCES_OUT`; sem o programa diz "skipping" e passa;
- `apps/uigallery/tests/plevir_fixture.rs` (+1): o golden dos silêncios;
- os testes da galeria (`every_page_renders_a_scene`,
  `every_page_renders_in_short_viewports`, os invariantes de `PAGES`) e
  os 31 do `regress` passaram sem mudança.

### validação executada

- `cargo test -p uigallery` (196 na lib com 6 ignorados, 8 no main, 2 no
  `uir_fixture`, 5 no `view_source`, 31 no `regress`);
- `cargo clippy -p uigallery --all-targets -- -D warnings`, `cargo fmt`;
- `./tool/tasks/fullcheck.sh` (abaixo) e `cargo run -p regress --
  check`;
- o teste ignorado com o ffmpeg real, duas vezes (com e sem o vídeo já
  gerado); `--render Silences` em 1200x800 core (com `--semantics`) e
  em 390x844 hoff, com e sem `--scroll 620`; `--render Surface` em
  390x844 para conferir que a sobreposição do blurb do header no card
  em largura de telefone é do shell e não da página. nenhuma janela foi
  aberta.

### linha de ADR

[`0053-the-silences-app-is-the-fixed-screen.md`](../../adr/structure/0053-the-silences-app-is-the-fixed-screen.md) (proposed): o
contexto (o cenário da tese e o checkpoint b), o modelo puro, a semente,
o ffmpeg só depois de um pedido de um ator (e por que não ao abrir), o
`Started`/`complete` no `wake`, o dispatcher da galeria, a projeção, a
tela, as consequências (a medição, o golden e as baselines), o que a 4b
precisa e o que evitar. para o orquestrador (o [`adr/`](../../adr/README.md) é
proibido aqui): uma linha no índice, algo como "the silences app is the
fixed screen (proposed): a página Silences do showcase é o app do editor
de vídeo da tese e a tela fixa do checkpoint b; o modelo puro com o
parser do `silencedetect`, a semente da sonda, o ffmpeg em processo
filho só depois de `detect_silence`/`preview_segment`, `Started` ->
`complete`/`fail` no `wake`, pelo dispatcher da galeria (o do todo)".

### decisões

- a semente: a página abre com a detecção gravada da sonda, como o todo
  abre com três itens. razão: golden, baseline e `--render` iguais em
  toda máquina; as decisões funcionam no wasm;
- o ffmpeg só depois de um pedido de um ator, nunca ao abrir (o desvio
  do briefing, aprovado na resolução): o headless abre a página como a
  janela; um `connect_silences` no shell faria a geração ao abrir, noutra
  fase;
- o dispatcher do todo emprestado, não um próprio (acima);
- um job por vez, na ordem: gerar, detectar, um frame de cada vez; os
  frames que a tela mostra (o instante mostrado e o meio de cada trecho)
  são pedidos pela própria página depois de uma detecção e depois de
  cada frame que chega, e ficam em cache por instante (o erro também,
  para não repetir em laço);
- `apply_all` aplica só os pendentes: o que alguém rejeitou fica;
- o arraste dos sliders fica com o widget (como o campo de intenção e o
  select da 3b-ii); o modelo guarda `detected_with` e o status diz
  "detect again" quando os sliders se afastam;
- os botões aplicar e rejeitar de uma linha não são nós: a linha é o nó e
  eles a affordance, como o x do todo; assim a superfície da 4b liga um
  botão a `silences.cut[i] apply_cut`;
- segundos com seis decimais na projeção (`micros`): a diferença e a
  soma do que o ffmpeg imprimiu não carregam ruído binário para o golden;
- o vermelho é o que sai: o badge "cut" e a marca vermelha na linha do
  tempo são o trecho aplicado (a semântica de diff); "kept" verde é o
  rejeitado; âmbar o pendente;
- `Said` guarda o que a última corrida deu (semente, detectado com o
  tempo, falhou com o motivo); a mensagem do status é derivada, com o
  job em curso na frente ("detecting silences at ...", "decoding the
  frame at ...").

### limitações

- o vídeo não é cortado de verdade: aplicar é uma decisão no modelo (o
  briefing deixou o corte para depois, se o checkpoint b pedir);
- o ffmpeg não roda ao abrir: até o primeiro `detect_silence` ou
  `preview_segment` não há frames, e a lista mostra o instante do meio
  num tile escuro;
- só um job por vez: com muitos trechos os frames chegam em fila (30 ms
  cada nesta máquina);
- o `find` do programa confere que é um arquivo, não o bit de execução;
- a thread do trabalhador não é juntada (`join`); um filho morto pelo
  prazo é morto e colhido, mas se o app fechar com um job em curso a
  thread morre com o processo e o filho termina sozinho;
- a mensagem de status muda a revisão da projeção a cada job que começa
  e acaba (uma detecção seguida de três frames são oito revisões);
- a sobreposição do blurb do header sobre o primeiro card em 390 px de
  largura é do shell (a página Surface mostra o mesmo) e não desta
  página;
- o teste `the_agent_asks_the_person_who_declines_and_it_stops` de
  `apps/agenteval/tests/experiment.rs` (task-93) falhou uma vez no primeiro gate
  desta branch ("the person's click at 1701 ms is in the session": casa
  um evento pelo milissegundo) e passou três vezes sozinho e no gate
  seguinte; é sensível à carga da máquina e não toca em nada desta
  fase;
- os testes de processo são só desktop; o ignorado só com o ffmpeg.

### dependências pendentes e riscos de integração

- para o orquestrador: a linha do índice de ADR (acima); o
  [`changelog.md`](../../CHANGELOG.md) (a página Silences); na entrada `[crates.showcase]`
  do `docs/ATLAS.toml` os módulos `model::silences` e
  `view::apps::silences` (com `worker`, `semantics`, `layout`, `draw`) e
  a página; o [`component-catalog.md`](../../ref/architecture/component-catalog.md) se listar as páginas do Apps Demo;
- `Cargo.toml` e `Cargo.lock` não mudaram: nenhuma dependência nova (o
  `image` que decodifica o png já vinha pelo engine);
- a página depende de `TodoPage::with_dispatcher` e `receipt` (públicos
  no crate desde a 3a); quem mover o dispatcher para a `ShowcaseView` (o
  refactor que a fase 1 deixou) troca uma linha em `semantics.rs`;
- a fixture `probe.stderr` é do ffmpeg 9; outra versão que mude o
  formato das linhas muda o parser, não a semente.

### o que a fase 4b (a tela gerada) precisa

- um componente de imagem no vocabulário do IR (o `media` ficou fora do
  v1): um nó que mostra o frame de um nó `media` num instante
  (`silences.video:shown`, ou o `start`/`middle` de um trecho), e um
  thumbnail por instância de um `repeat` sobre `silences:cuts`. a
  página decodifica um frame por instante e guarda o `ImageHandle`; a
  superfície pede instantes, nunca bytes. o host precisa de um jeito de
  a superfície ler o frame de um instante do app (hoje só a página tem o
  cache): um estado projetado com a chave do frame, ou um acesso do host
  ao cache pela `Dispatch`;
- as capacidades que a superfície de revisão liga: as contagens
  (`silences.counts:*`), o limiar (`silences.threshold` com `set_value`),
  `detect_silence`, um `repeat` sobre `silences:cuts` com o frame, o
  intervalo, `decision`, `apply_cut` e `reject_cut` de cada item, e
  `apply_all`; o prompt do proponente precisa dizer que os ids são do app
  (a chamada 3 da 3b-ii nomeou o molde com os ids do app);
- a comparação lado a lado: duas vistas `media` do mesmo vídeo em
  instantes diferentes (o original e o resultado) e as diferenças na
  linha do tempo são os trechos com `decision` apply: nenhuma capacidade
  nova, uma segunda leitura de `silences:cuts`;
- "corta os silêncios desse vídeo e depois quero revisar os cortes" pede
  `detect_silence` antes da superfície: o request `Started` termina no
  `wake`, e a superfície nasce quando ele completa; o padrão desta fase
  serve;
- cortar de verdade (um `export` com o ffmpeg tirando os trechos
  aplicados) não existe; se o checkpoint pedir, é uma capacidade nova
  com o mesmo trabalhador.
