---
id: adr-0061
title: "printing is the frame on paper"
project: plev
type: decisao
status: aceito
date: 2026-09-24
date_basis: cabecalho
topics: [plataforma, renderizacao, agentes]
relations: [adr-0060, task-66, task-66b, task-76]
source: docs/adr/rendering/0061-printing-is-the-frame-on-paper.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# printing is the frame on paper

Decide que a impressão é o mesmo quadro da janela, renderizado pelo caminho headless numa escala de papel (`paper_scale`: dpi sobre pontos por polegada) e enviado ao CUPS pelo processo filho `lp`, sem biblioteca de impressão. Imprimir é ação da pessoa: um agente é recusado, como é recusado abrir uma janela, e o trabalho roda ao lado do laço, respondendo por canal. A medição mostrou que a mesma página sai com o mesmo layout e glifos nítidos em duas escalas; nenhuma impressão real foi feita nesta máquina, que não tem destino de impressão.

## context

the program (task-76, item 9; the thesis, section 22) asks for printing
as the integration in which the screen becomes paper. the engine already
draws any page without a window through the window's own frame path
(`engine::headless::Renderer`, task-66; `WindowHost::headless`, item 2;
`showcase --render`) and exports the same frame as svg with every
approximation reported (task-66b). nothing reached the system's printing
queue, and no app offered "print this page" as a capability.

the probe (the handoff of item 9 in
[task-76-platform-integration.md](../../tarefas/task-76-platform-integration.md)) measured:

- this machine has no destination: `lpstat -p -d` says "No destinations
  added." (exit 1, 0.14 s), `lp <file>` says "No default destination.",
  `lp -d Nowhere <file>` says "No such file or directory" about the
  destination, and `lpstat -o <job-id>` is refused ("Invalid destination
  name in list"): a job's status is read from its destination's queue.
- the Todo page rendered headless at the screen's scale (2.0, 2400x1600,
  231 KB of png) and at a paper scale (300 dpi over 72 points per inch,
  4.1667, 5000x3333, 704 KB) has the same layout and sharp glyphs at
  both: the ink of one text run is 585 px wide at 2.0 and 1219 at
  4.1667, a ratio of 2.084 for a scale ratio of 2.083, so the atlas
  rasterizes at the physical scale (task-66) instead of scaling a
  bitmap. warm, the whole render and encode of the page takes 0.3 to
  0.5 s in a debug build; cold, the device and the pipelines add 0.2 s.
- the svg export writes text as glyph outlines (`<path d=>` inside a
  `<g aria-label>`, no `<text>`), rects and rounded rects exactly,
  gradients and clips approximated and reported (8 approximations on the
  Todo page): the geometry a vector pdf would take is there; what is
  missing is a pdf writer (page objects, a content stream of the same
  paths, a font-free text layer), which nothing here has without a
  dependency.

## decision

- **the document is the frame.** `engine::platform::print::PrintDocument` (`rust/engine/src/runtime/platform/print/mod.rs:147`)
  is the same scene the window draws, rendered through the headless
  frame path at a paper scale (`paper_scale(dpi)`: dots per inch over
  points per inch, since a logical pixel is a point on paper) and
  encoded as png, which every CUPS filter chain takes; `from_frame` and
  `from_headless` build it from a `Frame` or a `Renderer` and a
  compositor. the frame's svg rides along when asked (`with_svg`), the
  registered path to a vector pdf. `fit_scale` clamps the scale to the
  device's largest texture (`wgpu::Limits::default()` asks 8192 a side),
  so a wide window prints at the largest scale that fits.
- **the printer is a trait, the real one a child process.** `Printer` is
  `destinations`, `print(document, options)` and `status(job)`, with the
  few options a page needs (destination, copies, title, fit to page,
  media). `CupsPrinter` runs `lpstat -p -d`, `lp -d <dest> -t <title> -o
  fit-to-page -- <file>` and `lpstat -W <which> -o <dest>` as children
  with a deadline and a kill by pid, the form of the ffmpeg and the
  `claude` workers, never a library: the engine takes no dependency for
  printing. the document is written private to the user in a spool
  directory and removed once `lp` took it; no destination is an empty
  list, not an error. `FakePrinter` keeps every document it is given;
  `NoPrinter` is windows (no `lp`, no dependency) until its bring-up,
  wasm and mobile; `default_printer` chooses. `PLEV_LP` and `PLEV_LPSTAT`
  name a test's fakes.
- **printing is a person's, and a background job.** the showcase projects
  `shell.print` (a button, `activate`, `Actors { human, assistive, test
  }`: an agent is refused as it is refused a window), enabled while a
  destination is known and no job runs, with the destinations and the
  last job's destination, id and status as its state, and `shell:print`
  as one line for an agent ("no destination", "ready on Office",
  "printing on Office", "Office-7 on Office: queued", "failed on Office:
  ..."). cmd-p and the "view" menu's item ask it through the gallery's
  one dispatcher. `execute` answers `Started`; the view builds the scene
  for paper on the loop's thread (the main window's layers and scale put
  back after; a frame's worth of work, tens of microseconds warm) and a
  thread of its own opens a headless renderer over a device of its own,
  draws, reads back, encodes, hands the document to the printer and
  asks the job's place once; `settle_print` on the loop's catch-up
  completes or fails the request with the job, the destination, the
  status and the document's measures. under [`Background`](0060-a-covered-window-does-not-draw-and-wakes-new.md) (item 7) a
  print asked asleep waits for the wake and is built from the state
  then; an answer that lands asleep waits on the channel. the
  destinations are looked for once, off the loop, when the printer is
  given, and projected when they land: without one the button is
  disabled and the projection says so, and nothing fails.

## reason

- one frame path: what prints is what the window draws, through the same
  compositor, the same passes and the same atlas, at a scale the paper
  wants; a second raster path for paper would drift from the screen as
  the old text probe did from the window.
- the platform's own queue: CUPS is what every desktop unix has, and
  `lp` is a process like ffmpeg; a library would add a dependency for a
  job that ends in a child anyway.
- the person's: a print leaves the machine, like a window opening or a
  file dialog; the policy is declared on the node once and holds for the
  key, the menu, a screen reader and the port alike.
- the loop stays free: the render and the spool take hundreds of
  milliseconds in a debug build, more than a frame, so they run beside
  the loop; the scene build stays on it because the view lives there and
  it costs less than a frame.

## consequences

- proven without a printer: the fakes (`rust/engine/tests/print.rs`
  over `tests/fake-lp`, `tests/fake-lpstat`, `tests/fake-lpstat-none`),
  the document from the render (size, dpi, a png that is the frame's
  pixels), a person's print of the Todo page through the fake printer
  (`apps/uigallery/tests/print.rs`: the document is, byte for byte, the
  frame a view in the same state draws at the paper scale; the main
  window's scene and scale untouched; a print asked asleep started on
  the wake), the shell's projection and policies, the bar's item with its
  keys. no `lp` or `lpstat` runs in any test or in the gate.
- nothing was printed on this machine: it has no destination, and a real
  print is the person's (the handoff carries the script).
- what the paper shows is the main window's frame: the chrome and the
  page, at the window's logical size, and of a page that scrolls, what
  the viewport shows; pagination of content taller than the window is
  not here.
- the job's status is asked once, right after the spool; nothing polls
  the queue afterwards, so a job that completes later stays "queued" in
  the projection until the next print.
- the png carries no dpi chunk: with `fit-to-page` the printer scales the
  document to the paper, and the document's dpi is in the receipt.
- `NativeNotifier` and the tray were not touched: a print that ends while
  nobody looks is not told by notification (the receipt and `shell:print`
  say it).

## bring-up

- windows: a printer over the spooler API (or a `NoPrinter` until then);
  linux needs cups installed, which `CupsPrinter` assumes.
- a vector pdf from the svg the export already writes (text as outlines;
  a pdf writer without a dependency, or one approved).
- the native print panel (`NSPrintPanel` and the `lp -o` options it
  chooses: media, sides, copies) as another representation of the same
  capability, with the parent window of item 2.
- pagination of scrolling content; a follow-up poll of the queue for the
  projection.

## alternatives

As alternativas recusadas estão em `reason` e em `avoid`. Em `reason`: uma segunda rota de raster para o papel, que derivaria da janela como a sonda de texto antiga derivou, e uma biblioteca de impressão, que acrescentaria dependência para um job que termina num processo filho de qualquer forma. Em `avoid`: um segundo renderizador offscreen para o papel, e uma chamada a `Printer` dentro de `execute` ou na thread do laço, que bloquearia enquanto o filho roda.

## avoid

- a second offscreen renderer for paper: the document comes from
  `engine::headless`, the window's path.
- a `Printer` call from `execute` or from the loop's thread: a child
  blocks; the job runs beside the loop and answers through a channel.
- a print capability an agent may ask for; the agent reads `shell:print`.
- a test that runs the machine's `lp` or `lpstat`: the fakes by path,
  never a real queue.

## integração (2026-09-24, orquestrador)

aceito com uma correção feita na revisão, no mesmo branch: o job e a
busca de destinos respondiam num canal que nada acordava o laço para
ler. sem animação na tela, a barra dizia "printing on ..." e o pedido
ficava `Executing` até o próximo input. agora, enquanto uma thread de
impressão está fora ou um job espera o wake, o `Wake` do frame pede uma
olhada a cada 100 ms (`ShowcaseView::print_wake`, somado em
`App::frame_wake`), a mesma forma do Claude da Surface. o teste
`a_print_thread_out_keeps_a_deadline_until_it_answers` prova o prazo
enquanto a thread está fora e o fim dele quando responde.
