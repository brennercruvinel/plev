---
id: task-76/item-9-printing
title: "task-76, item 9: printing"
project: plev
type: section
parent: task-76
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [platform, rendering, gpu]
relations: [adr-0061, task-66]
evidence: [3e13a7f1, 9c33b535, 84a95b34, dd14f99b]
source: docs/log/steps/task-76-platform-integration.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# item 9 handoff: printing, the frame on paper

Records item 9: `engine::platform::print` puts printing behind the `Printer` trait, with destinations, printing of a `PrintDocument` and job status, and the document is the window's frame rendered at 300 dpi through the headless path, at a scale of 4.1667 over the logical points. The `CupsPrinter` runs `lp` and `lpstat` as child processes with a deadline and kill by pid, and the showcase puts `shell.print` behind person-only triggers, `cmd-p` and the menu, with the scene built on the loop thread and the render, the png and the spool on a thread of their own. The result was proven with `FakePrinter` and with scripts that imitate CUPS, without a real `lp`; on this machine there is no destination, and the button stays disabled with `no destination`.


branch `task/76h-print`, base `f92e56d1` (main após o item 7). worktree
`.claude/worktrees/agent`, target `target/agents/one`. commits, em ordem:

- `3e13a7f1` task-76: the print probe
- `9c33b535` task-76: the printer behind a trait, with the fake and CUPS
  as a child (o trait, o falso, o CUPS por filho e o documento a partir
  do render, `PrintDocument::from_frame` e `from_headless`, num commit só:
  a assinatura do trait pede o documento)
- `84a95b34` task-76: the entry points move to app/entry.rs (só
  movimento, antes de tocar o `app.rs`: ver "arquivos tocados")
- `dd14f99b` task-76: the showcase prints the page: shell.print, cmd-p and
  the job beside the loop
- (este) task-76: ADR and the item 9 handoff

## a sonda

(a) esta máquina, 2026-09-24, sem impressora (nenhuma foi adicionada):

```text
$ lpstat -p -d
lpstat: No destinations added.          (stderr)
no system default destination           (stdout)
exit 1, 0,14 s
$ lpstat -a
lpstat: No destinations added.          exit 1
$ lpstat -r
scheduler is running                    exit 0
$ lpstat -o
(nada)                                  exit 0
$ lpstat -W not-completed -o Office-42
lpstat: Invalid destination name in list "Office-42".   exit 1
$ lp tiny.png
lp: Error - No default destination.     exit 1
$ lp -d Nowhere -t "plev test" -o fit-to-page tiny.png
lp: No such file or directory           exit 1 (é sobre o destino; o arquivo existia)
$ lp -d Nowhere /nonexistent.png
lp: Error - unable to access "/nonexistent.png" - No such file or directory   exit 1
$ lpoptions
(nada)                                  exit 0
```

`man lp`: `-d destino`, `-n cópias`, `-t título` (o nome do job na fila),
`-o media=a4|letter|legal`, `-o fit-to-page` (a opção genérica do CUPS que
escala o documento ao papel; o `man` desta versão lista `media`,
`number-up`, `orientation-requested`, `sides`, `print-quality` e
`job-sheets`), `--` antes do arquivo, e o `request id is <dest>-<n> (1
file(s))` na saída. sem `-d`, `LPDEST`/`PRINTER`, o `lpoptions -d` e o
`lpadmin -d`, nessa ordem. `man lpstat`: `-p` impressoras, `-d` o padrão,
`-W completed|not-completed` antes de `-o`, e `-o [destino(s)]` os jobs de
um destino: o `-o <job-id>` não existe (a sonda acima), então o status de
um job se lê na fila do destino dele (`Office-42` é o job 42 de `Office`),
uma linha que começa com o id.

(b) a página Todo sem janela em duas escalas, pelo `showcase --render`
(binário debug, `target/agents/one`):

| escala | físico | png | wall frio | wall quente |
|---|---|---|---|---|
| 2,0 (tela retina) | 2400x1600 | 231 289 bytes | 1,16 s | 0,11 a 0,12 s |
| 4,1667 (papel) | 5000x3333 | 704 377 bytes | 0,51 s | 0,29 a 0,46 s |

a conta: 1 pixel lógico = 1 ponto = 1/72 pol, então a escala de `d` dpi é
`d / 72`; a 300 dpi, `300 / 72 = 4,1667`, e a janela de 1200x800 vira
5000x3333 dots, 16,7 x 11,1 pol (mais largo que um A4 deitado, 11,69 pol;
o `fit-to-page` reduz ao papel, o que ainda dá uns 428 dpi efetivos). o
limite do device: o engine pede `wgpu::Limits::default()` nos dois
construtores (`gpu/context.rs:295`), então `max_texture_dimension_2d` é
8192 em qualquer máquina; 5000 cabe, e `fit_scale` escolhe a maior escala
que cabe quando não (uma janela de 2000 de largura imprime a 4,09). o
atlas de glifos não estourou em nenhuma das duas escalas.

por teste (`rust/engine/tests/print.rs`, `--nocapture`), um run de
texto de 15 px em 600x80 lógicos, cada peça medida:

```text
screen 2.0:   1200x160 px, device 176.5 ms, draw+read 86.4 ms, png 1.6 ms (18900 bytes), ink 585 px    (frio: o primeiro device e os pipelines)
paper 4.1667: 2500x333 px, device   9.7 ms, draw+read  7.7 ms, png 6.8 ms (53429 bytes), ink 1219 px   (quente)
```

a tinta do mesmo run vai de 585 a 1219 px: razão 2,084 para uma razão de
escalas de 2,083. os glifos são rasterizados na escala física (o atlas
da task-66), não um bitmap ampliado: o texto continua nítido. o png de
cada escala está em `scratchpad/probe/todo-2.png` e `todo-4.1667.png`, e
os recortes da mesma região lógica (a linha "Absorb the plev demos into
the showcase") em `crop-2.png` (640x150) e `crop-4.png` (1334x313): a
mesma diagramação, bordas limpas nos dois.

(c) o que `engine::headless::svg` já entrega, no Todo: 397 `<path d=>` e
zero `<text>`: texto como contorno de glifo dentro de `<g aria-label>`,
rect e rounded rect exatos, gradiente como `linearGradient` (mistura em
linearRGB, que nem todo viewer honra), clip como `clipPath` em bounds
lógicos (o frame corta em pixel físico inteiro), path tesselado como
triângulos (pode mostrar costura). 8 aproximações relatadas no Todo
(218 115 bytes de svg, 0,14 s). para um PDF vetorial sem dependência
faltaria: um escritor de PDF (objetos de página, um content stream com os
mesmos paths, sem fonte porque o texto já é contorno), a conversão de
`clipPath` e gradiente para operadores PDF, e o sRGB/linear acertado.
registrado como bring-up; o `PrintDocument::with_svg` guarda o svg ao lado
do png para esse caminho.

## o trait e o que cada alvo faz

`rust/engine/src/runtime/platform/print/mod.rs` (470 linhas de produção, 645 com os
testes) e `platform/print/cups.rs` (417, 563 com os testes), pelo molde
do `alerts.rs` e do `notify.rs`: trait, real sob o cfg, falso em todo
alvo, nenhum em wasm e mobile, `default_printer()` escolhendo.

- `Destination { name, default }`; `PrintOptions { destination:
  Option<String>, copies, title, fit_to_page, media: Option<String> }`
  com `Default` (1 cópia, "plev", fit, sem media) e `.to()`, `.title()`,
  `.copies()` (nunca menos de 1), `.fit_to_page()`, `.media()`;
  `PrintDocument { png, width, height, dpi, svg: Option<String> }` (o
  `Debug` mostra tamanhos, nunca os bytes), `from_frame(&Frame, dpi)`,
  `from_headless(&mut Renderer, &mut Compositor, clear_linear, dpi)`,
  `with_svg`, `paper_size()` em polegadas; `paper_scale(dpi)` e
  `fit_scale(w, h, scale, max_side)`; `POINTS_PER_INCH = 72`, `PAPER_DPI
  = 300`. `JobId(String)`; `JobStatus::{Queued, Completed, Gone}`;
  `PrintError::{Unsupported, NoDestination, UnknownDestination(String),
  Spooler(String), Timeout { program, secs }, Io(String)}`; `Shown<T>`.
- `trait Printer { destinations() -> Shown<Vec<Destination>>;
  print(&PrintDocument, &PrintOptions) -> Shown<JobId>; status(&JobId) ->
  Shown<JobStatus> }`. cada chamada pode bloquear (um filho): o app chama
  de uma thread ao lado do laço, nunca do `execute`.
- `CupsPrinter` (unix desktop: macOS e linux): `new()` lê `PLEV_LP` e
  `PLEV_LPSTAT` ou fica com `lp` e `lpstat` do PATH (nada roda ao
  construir); `with(lp, lpstat)`, `.timeout(d)` (20 s por padrão),
  `.spool_dir(dir)` (`$TMPDIR/plev-print` por padrão, 0o700). cada
  chamada é um filho com prazo, `try_wait` a cada 10 ms, morte pelo pid
  e `wait` (reap), stdout e stderr drenados em threads (um neto segurando
  o pipe não segura o chamador), 1 s de graça pro fim da saída: a forma
  do `worker.rs`, reescrita e não dependência. `destinations()`: `lpstat
  -p -d`, uma `Destination` por linha `printer <nome> ...`, o padrão pela
  linha `system default destination: <nome>`; exit 1 com "No
  destinations added" é lista vazia, qualquer outra falha é
  `Spooler(última linha)`. `print()`: escreve o png em
  `<spool>/plev-<pid>-<n>.png` (0o600), roda `lp [-d dest] [-n cópias]
  -t título [-o fit-to-page] [-o media=x] -- arquivo`, remove o arquivo
  (o `lp` copiou pro spool, ou nunca pegou), lê `request id is X`;
  stderr "No default destination" vira `NoDestination`, "No such file or
  directory" sem "unable to access" vira `UnknownDestination(dest)` (é o
  que o `lp` diz de destino desconhecido, sonda (a)), o resto `Spooler`.
  `status(job)`: destino = tudo antes do último `-`; `lpstat -W
  not-completed -o <dest>` lista o id? `Queued`; senão `-W completed`
  lista? `Completed`; senão `Gone`; um destino que o `lpstat` recusa é
  `Spooler`.
- `FakePrinter`: `.destination(nome, default)`, `.will_refuse(err)` (a
  próxima impressão que chega ao spooler, isto é, com destino escolhido),
  `.will_report(status)`, `printed()`/`take_printed()` com `Printed {
  document, options, job }`; sem destino recusa `NoDestination`, destino
  desconhecido `UnknownDestination`, job impresso é `Queued`, outro é
  `Gone`; ids `<dest>-<n>`.
- `NoPrinter`: `Unsupported` nas três. é o windows até o bring-up (sem
  `lp`, e nenhuma dependência aqui), o wasm e o mobile.
- `impl Printer for Arc<Mutex<P>>` e `for Box<P>`: a thread do job segura
  uma ponta, o teste a outra. `default_printer() -> Box<dyn Printer +
  Send>`.

## o documento: escala, DPI, bytes, tempo

no showcase, a escala é `fit_scale(width, height, paper_scale(300),
8192)`: 4,1667 na janela de 1200x800 (5000x3333, 300 dpi); numa janela
mais larga que 1966 lógicos a escala cai pra caber e o dpi do documento
segue (`scale * 72`). o png sai com 700 460 a 704 377 bytes no Todo (a
diferença é o item marcado). o teste de integração mede o job inteiro na
thread dele: device 16 a 48 ms, render + readback + png 224 a 356 ms,
spool no falso 30 µs; a cena construída na thread do laço 71 µs quente e
24 ms frio (o primeiro layout da página naquela escala). o png não leva
chunk de dpi (o `image::codecs::png::PngEncoder` do lock não expõe o
`pHYs` sem mais código, e o `fit-to-page` escala ao papel de qualquer
jeito); o dpi vai no receipt e no `PrintDocument`.

## o trabalho de fundo

decidido: a cena na thread do laço, o resto fora. a view mora na thread
do laço e não é `Send`; o `Compositor` é (compilou sem `unsafe` e sem
wrapper), então `render_for_print(scale)` constrói a cena da janela
principal num compositor próprio (o `layers` e o `scale_factor` da view
são trocados e postos de volta: o único uso do `scale_factor` na cena é o
clip físico da ScrollArea, `render.rs:98`) em microssegundos, e a thread
`print-job` abre um `engine::headless::Renderer` próprio (device
próprio; o mesmo caminho do `regress`, só que fora do laço), desenha,
lê, codifica, entrega ao `Printer` e pergunta o status uma vez. custo
medido acima: o laço nunca espera mais que a construção de uma cena, e o
spool não toca o laço. a alternativa (render síncrono no laço com
contexto próprio) custaria 240 a 400 ms parados por impressão, mais de
um frame, e foi descartada.

`Execution::Started` no `execute` do shell, `complete`/`fail` no
`settle_print` (chamado do `advance` do `pace.rs`, o mesmo `catch_up` que
aplica o ffmpeg e o Claude), com `job`, `destination`, `status`, `width`,
`height`, `dpi`, `bytes`, `seconds` no outcome. sob `Background`: pedido
dormindo fica em `held` e começa no primeiro `settle_print` acordado (a
cena é construída aí, do estado de então); resposta que chega dormindo
fica no canal. os destinos também são um job (`print-destinations`),
disparado no `set_printer`, então o `resumed` não espera o `lpstat` (0,14
s aqui; 20 s no pior caso de um scheduler travado) e o botão liga quando
a resposta cai. um job por vez: o botão desabilita enquanto um roda.

## o showcase: o botão, o menu, a projeção

`apps/uigallery/src/views/dispatcher/shell/print.rs` (528 de produção),
`shell.rs` (538), `app/print.rs` (62, 189 com os testes), `app/menu.rs`
(253), `app/pace.rs` (188), `app/events.rs` (219), `app.rs` (567),
`app/entry.rs` (95, movimento).

- `shell.print` (Button, "print this page"), `PERSON_ONLY` (human,
  assistive, test; agente recusado `NotAuthorized` antes do `execute`),
  `enabled` só com destino conhecido e sem job em curso, `focusable`
  false; estado `destinations` (List), `destination` (o padrão, ou o
  primeiro), `job`, `status` (do último). `shell:print`, uma linha pro
  agente: "no destination", "ready on Office", "printing on Office",
  "Office-7 on Office: queued", "failed on Office: <por quê>".
- cmd-p (ctrl-p fora do macOS; `PRINT_KEYS`, `is_print_key`: com shift
  não é) em `events.rs`, logo depois das teclas de gravação:
  `App::print_key` -> `ShowcaseView::request_print` ->
  `ask_shell_receipt(Print, "activate")` (novo, devolve o receipt; o
  `ask_shell` de sempre chama ele). o item "print this page" com o atalho
  no menu "view", depois de "page in a window of its own" (e no menu do
  tray, que reusa `view_menu`), desabilitado sem destino: `MenuItem::of`
  lê o `enabled` da projeção. `install_print()` no `resumed`, depois das
  notificações: `view.set_printer(default_printer())`.
- nesta máquina: `lpstat` responde sem destino, a lista fica vazia, o
  botão e o item ficam desabilitados e `shell:print` diz `no
  destination`. nada foi impresso.

## testes

- `rust/engine/src/runtime/platform/print/mod.rs` (8): escala e `paper_size`,
  `fit_scale`, as opções, o `Debug` do documento, o falso (padrão e
  nomeado, `printed`, status), as recusas do falso e os textos dos
  erros, o compartilhado por `Arc<Mutex>`, `NoPrinter` e
  `default_printer()` construído sem rodar nada. `cups.rs` (6): os
  programas por env ou PATH, o parse do `lpstat -p -d` (com o padrão, o
  vazio desta máquina, sem padrão, a falha do scheduler), os argumentos
  do `lp`, o id do job e o destino dele, a listagem de um job, as
  recusas do `lp` como esta máquina as deu.
- `rust/engine/tests/print.rs` (7): a sonda (b), o documento a partir do
  render (tamanho do frame, dpi, png que decodifica pros mesmos bytes do
  `read()`, `paper_size`, o svg ao lado), e o CUPS sobre `tests/fake-lp`,
  `tests/fake-lpstat` (Office padrão e Home; Office-7 na fila, Home-7
  completo) e `tests/fake-lpstat-none` (esta máquina): destinos com o
  padrão marcado, lista vazia e `NoDestination` sem `-d`, o comando
  montado (`-d Office -n 2 -t "plev, Todo" -o fit-to-page -o media=a4 --
  <spool>/plev-*.png`, lido do `lp.args` que o falso deixa ao lado do
  documento, e o documento removido depois), `Queued`/`Completed`/`Gone`
  e o destino inválido, as recusas em palavras do `lp`, o programa que
  não existe, e o prazo: `fake-lp -t slow` vira `sleep 30`, o prazo de 2
  s mata pelo pid (`kill -0` falha depois) e o documento sai do spool
  também. nenhum `lp` nem `lpstat` real roda.
- `apps/uigallery/src/views/dispatcher/shell/tests.rs` (+1, e as listas de
  ids com `shell.print`): a projeção sem destino (desabilitado, estados
  vazios, `no destination`), o pedido da pessoa recusado `Disabled`, os
  destinos do falso caindo pela thread e habilitando (o padrão é o
  destino), o agente `NotAuthorized` e nada impresso, o `NoPrinter`
  deixando `no destination`.
- `apps/uigallery/src/shell/system/print.rs` (2): as teclas; a barra com o item,
  desabilitado, com `cmd-p`, as teclas pedindo e recusadas, um `p` solto
  é da página, e o item habilitando depois do `set_printer` e do
  `sync_menu`.
- `apps/uigallery/tests/print.rs` (2, GPU, pula sem adapter): a pessoa
  imprime o Todo com um item marcado pelo falso: `Executing`, `printing
  on Office`, botão desabilitado, o job responde, o receipt `Completed`
  com `Office-1`, 5000x3333, 300 dpi, bytes e segundos; o falso recebeu
  um documento com o tamanho da página, `-d Office`, "plev, Todo", 1
  cópia, fit; o png decodificado é, byte a byte, o frame que uma view no
  mesmo estado desenha a 4,1667 por um `Renderer` (o controle); a
  projeção depois (`Office-1 on Office: queued`, `job`, `status`,
  habilitado de novo); a cena da janela principal igual nó a nó e o
  `scale_factor` 1,0. e o pedido dormindo (`Background::sleeping(true)`):
  retido, nada roda em 250 ms, acorda e imprime.

## gate e regress

`df -h /` antes do gate: 21 GB livres; 21 GB depois. `./tool/tasks/fullcheck.sh` verde
no worktree com `CARGO_NET_OFFLINE=true`, com a perna wasm e a perna
urnaui (`../urna` presente): antes (o fechamento do item 7, base
`f92e56d1`) 2577 passed, 0 failed, 16 ignored em 124 binários; depois
2603 passed, 0 failed, 16 ignored em 126 binários (+26: 14 de
`platform::print` e `print::cups`, 7 de `engine/tests/print.rs`, 3 no lib
do showcase, `dispatch::shell` +1 e `app::print` 2, e 2 de
`showcase/tests/print.rs`; +2 binários, os dois `tests/print.rs`). perna
urnaui: 85 passed, 0 failed, 1 ignored, igual a antes (o urnaui não toca o
`platform::print` e compila sem mudança). clippy `-D warnings`, fmt e a
checagem wasm limpos nas duas pernas; o único aviso no log é o `block
v0.1.6` de incompatibilidade futura, que já estava. antes do gate,
`cargo test --offline -p engine -p uigallery`: 1208 passed, 0 failed, 7
ignored.

`cargo run --offline -p regress -- check`: 161
baselines, 161 iguais, 0 mudaram, sem frame desenhado e sem GPU aberta
(702,6 ms). o shell não está no frame de nenhuma página e
`ShowcaseView::semantics` não mudou: os goldens ficaram como estavam,
nenhum bless.

linhas de produção (regra do guard, `file_hygiene_guard` verde): engine
`platform/print.rs` 470 (novo), `platform/print/cups.rs` 417 (novo);
showcase `app.rs` 567 (era 633; o movimento levou a 561), `app/entry.rs`
95 (novo, movimento), `app/print.rs` 62 (novo), `app/menu.rs` 253 (era
234), `app/pace.rs` 188 (era 181), `app/events.rs` 219 (era 213),
`view/dispatch/shell.rs` 538 (era 497), `view/dispatch/shell/print.rs`
528 (novo). tudo abaixo de 639.

## ADR

[`0061-printing-is-the-frame-on-paper.md`](../../adr/Rendering/0061-printing-is-the-frame-on-paper/0061-printing-is-the-frame-on-paper.md) (proposed): o documento como
render do mesmo frame numa escala de papel, o CUPS como filho, o que é só
pessoa e trabalho de fundo, o svg como caminho pro PDF, o que fica pro
bring-up. linha pro índice ([`adr/`](../../adr/README.md) está fora da lista, fica
para a integração):

```text
| 2026-09-24 | [printing-is-the-frame-on-paper](printing-is-the-frame-on-paper.md) | proposed | task-76 item 9: `engine::platform::print` põe "imprimir esta página" atrás de um trait (`Printer`: destinos, imprimir um `PrintDocument` com destino, cópias, título, fit-to-page e media, status do job); o documento é o mesmo frame da janela renderizado pelo caminho headless numa escala de papel (300 dpi sobre 72 pontos = 4,1667: a janela de 1200x800 vira 5000x3333, glifos nítidos porque o atlas rasteriza na escala física), png para o CUPS, o svg do headless ao lado como caminho para PDF vetorial; `CupsPrinter` roda `lp` e `lpstat` como filhos com prazo e morte pelo pid, sem dependência, `FakePrinter` guarda o que recebeu, `NoPrinter` é windows até o bring-up, wasm e mobile; no showcase `shell.print` (só pessoa, cmd-p e o menu) constrói a cena na thread do laço e faz o render, o png e o spool numa thread própria sob `Background`, com `shell:print` projetado para o agente; sem destino a projeção diz e o botão desabilita; provado com os falsos, byte a byte contra um render de controle, nenhum `lp` real |
```

## interfaces adicionadas (só crescimento)

- `engine::platform::print::{Destination, PrintOptions, PrintDocument,
  JobId, JobStatus, PrintError, Shown, Printer, FakePrinter, Printed,
  NoPrinter, CupsPrinter, default_printer, paper_scale, fit_scale,
  POINTS_PER_INCH, PAPER_DPI, LP_ENV, LPSTAT_ENV}`. nada do `headless`
  mudou: `Frame::encode_png` e `Renderer::render` já eram públicos.
- showcase: `ShowcaseView::{set_printer, request_print, settle_print,
  ask_shell_receipt}` (as duas primeiras desktop), `ShellTarget::Print`;
  `App::{install_print, print_key}` (pub(super)),
  `app::print::{PRINT_KEYS, is_print_key}`; `app::entry::{run, run_web,
  android_main, showcase_ios_main}` reexportados de `app`. a projeção da
  galeria (`gallery_semantics`) ganhou o nó `shell.print` e o estado
  `shell:print`; `ShowcaseView::semantics` (a da página) não mudou: os
  161 baselines e o golden do todo ficaram iguais.

## arquivos tocados fora da lista permitida, e por quê

nenhum. `apps/uigallery/src/shell/mod.rs` estava em 633 linhas e recebeu o
`mod print;` (com doc) e o `install_print()`: o primeiro commit do
showcase (`84a95b34`) foi só movimento, os quatro entry points de
plataforma (`run`, `run_web`, `android_main`, `showcase_ios_main`) para
`app/entry.rs`, palavra por palavra, reexportados; `app.rs` foi a 561 e
está em 567. `app/events.rs`, `app/menu.rs`, `app/pace.rs` são `app/**`.
`apps/uigallery/src/views/dispatcher/mod.rs` (fora da lista) não mudou: por
isso o tipo `Last` do `shell/print.rs` não é reexportado e a projeção
carrega o último job.

## limitações e decisões, sem esconder

- nada foi impresso nesta máquina, e nenhum `lp` real rodou em teste: o
  `CupsPrinter` foi provado contra os scripts falsos, que reproduzem as
  saídas medidas na sonda (a). o formato das linhas do `lpstat -p` e do
  `lpstat -o` com impressora de verdade vem da memória do CUPS e dos
  falsos, não desta máquina; o parser lê só o segundo token de `printer
  ...` e o começo de linha de um job, o que é robusto a variações de
  data e estado. a prova real é do brenner (roteiro abaixo).
- o status é perguntado uma vez, logo depois do spool; ninguém faz poll
  da fila depois, então um job que completa fica "queued" na projeção
  até a próxima impressão. um poll (um `Wake::after`) é uma linha no
  `settle_print` quando for pedido.
- o que vai pro papel é o frame da janela principal inteiro (chrome e
  página), no tamanho lógico da janela; conteúdo que rola imprime o que
  a viewport mostra. paginação fica registrada. a janela da página (`D`)
  não é o que imprime.
- o `render_for_print` troca `layers` e `scale_factor` da view e põe de
  volta: é o único jeito sem tocar `view/render.rs` (fora da lista). o
  método certo, quando o arquivo abrir, é um `render_at(c, scale)` na
  view que não guarde os layers em `self`.
- a thread do job abre um device wgpu por impressão (16 a 48 ms) e
  fecha; não há renderer retido. imprimir é raro.
- `PrintOptions.media` e `copies` existem no trait e no `lp`, mas o
  showcase imprime sempre 1 cópia no media da impressora com
  `fit-to-page`: escolher é do painel nativo (bring-up).
- `Timeout.secs` é o prazo em segundos inteiros: um prazo de 300 ms diz
  "0 s". o teste do prazo usa 2 s por isso e porque, ao lado dos testes
  de GPU do mesmo binário, um filho levou mais de 300 ms só pra começar
  (a primeira rodada falhou assim; sozinho passou em 0,32 s).
- linux: o `CupsPrinter` assume o cups instalado; sem ele o `lp` não
  existe e o `Io("could not start lp")` aparece na projeção como
  "failed on ...". windows: `NoPrinter`.
- notificação: um job que termina com o app sem foco não é contado por
  notificação (o `JobSink` do `notify.rs` ouve só `detect_silence`);
  `shell:print` e o receipt dizem. uma linha no `JobSink` resolve, se
  quiserem.

## o que fica para o bring-up e o que o item 10 reaproveita

- bring-up: o windows com o spooler dele (ou `NoPrinter` até lá); o PDF
  vetorial a partir do svg (texto já em contorno); o painel nativo
  (`NSPrintPanel`, escolhendo media, sides e cópias como `-o` do `lp`),
  outra representação da mesma capacidade com a janela pai do item 2;
  paginação; o poll da fila; o chunk de dpi no png.
- item 10 (câmera e microfone): a captura é o mesmo molde: trait, real,
  falso, nenhum, `default_*()`; um trabalho de fundo com canal, começado
  e aplicado sob `Background` (não começa dormindo, resposta espera o
  wake), com `Execution::Started` e `complete`/`fail` no `settle` do
  `catch_up`; e o `Scene`/`Renderer` próprio da thread mostra que um
  device próprio numa thread ao lado do laço funciona (um frame
  capturado vira textura no device da janela só no `wake` acordado, como
  o item 7 registrou). a permissão do sistema (câmera, microfone) é a
  diferença: pede `Unsupported` sem ela e um estado projetado
  ("permission denied") em vez de falhar, como o `no destination` aqui.

## o roteiro da prova real (do brenner, nunca de teste)

sem impressora física, o CUPS do macOS aceita um destino que imprime em
PDF; nada disto foi feito aqui (a máquina fica sem destino):

```text
lpadmin -p pdf -E -v cups-pdf:/ -m everywhere      (ou o "Save as PDF" de uma fila IPP; ou uma impressora real por System Settings)
lpstat -p -d                                         (o destino aparece; "system default destination: pdf" com lpadmin -d pdf)
RUST_LOG=showcase=info cargo run --offline -p uigallery -- Todo
   cmd-p, ou view > print this page: o log diz "print: the Todo page's scene built for paper at scale 4.167 (300 dpi) in ..." e
   "print: pdf-<n> queued: 5000x3333 at 300 dpi, ... bytes; device ..., render and png ..., spool ..."
lpstat -W completed -o pdf                            (o job listado; o arquivo onde o driver escreve, ~/Desktop ou /var/spool/cups-pdf)
PLEV_AGENT=1 ... e `plev observe`: shell:print = "pdf-<n> on pdf: queued" e shell.print:job
lpadmin -x pdf                                        (o destino sai; a máquina volta a "No destinations added")
```

o que olhar no papel: o frame inteiro da janela (sidebar, cabeçalho,
página) na folha, escalado pelo `fit-to-page`, texto nítido de perto.

## comandos de validação

```text
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/one
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_PROFILE_TEST_DEBUG=line-tables-only
cargo test --offline -p engine --lib platform::print
cargo test --offline -p engine --test print -- --nocapture          (a sonda (b) imprime as medidas)
cargo test --offline -p uigallery --lib -- dispatch::shell app::print app::menu
RUST_LOG=showcase=info cargo test --offline -p uigallery --test print -- --nocapture --test-threads=1
cargo test --offline -p engine -p uigallery
./tool/tasks/fullcheck.sh
cargo run --offline -p regress -- check
cargo run --offline -p uigallery -- --render Todo --scale 4.1667 --out todo-paper.png   (o png do papel, sem impressora)
lpstat -p -d                                                          (esta máquina: No destinations added)
```

## integração do item 9 (orquestrador, 2026-09-24)

- merge da `task/76h-print` na `main` com um commit a mais no branch:
  `task-76: a print thread that is out keeps the loop looking until it
  answers`. o job e a busca de destinos respondiam num canal que nada
  acordava o laço para ler; agora `ShowcaseView::print_wake` pede uma
  olhada a cada 100 ms enquanto uma thread está fora, somado em
  `App::frame_wake`. teste novo em `app/print.rs`.
- o `summary()` perdeu o `let _ = id` que só existia para usar a
  variável.
- fica para depois: o poll do status da fila depois do spool (a
  projeção fica em "queued"), paginação, PDF vetorial, painel nativo.
