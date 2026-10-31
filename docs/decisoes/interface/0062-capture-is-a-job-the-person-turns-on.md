---
id: adr-0062
title: "capture is a job the person turns on"
project: plev
type: decisao
status: aceito
date: 2026-09-24
date_basis: cabecalho
topics: [plataforma, renderizacao, semantica, agentes]
relations: [adr-0060, task-76]
source: docs/adr/interface/0062-capture-is-a-job-the-person-turns-on.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# capture is a job the person turns on

Registra que câmera e microfone são capturas que a pessoa liga, feitas por um processo `ffmpeg` filho sem dependência nova, com a permissão e o estado do dispositivo projetados como estado, não como falha. O resultado é um slot de imagem de fluxo que reescreve o próprio quadro, a liberação do dispositivo quando a página não pode ser vista, e testes que nunca abrem um dispositivo. Importa porque a câmera é o recurso mais privado que um app pode ligar, e a decisão mantém essa escolha com a pessoa, inclusive frente a agentes. Vem do item 10 da task-76.

## context

the program (task-76, item 10; the thesis, section 22) asks for the
camera and the microphone as capabilities: the frame drawn by the
compositor as a texture, the sound as a buffer, the system's permission
as a projected state instead of a failure. the engine had no capture,
and the lock has no crate for it (`objc2-av-foundation`, `cpal`,
`coreaudio` are absent; no dependency may be added). ffmpeg is on the
desktop machines the showcase already runs it on (the Silences page),
with an input device per platform.

the probe (the handoff of item 10 in
[task-76](../../tarefas/task-76-platform-integration.md)) opened no
device, ever: ffmpeg read `-f lavfi` sources only, paced by `-re` as a
device paces itself, and the rest was read from `ffmpeg-devices(1)` and
the strings of libavdevice.

- a 640x360 RGBA frame through the pipe is 921600 bytes; the first one
  arrives 33 to 64 ms after the spawn, then 15 a second as asked. 16 kHz
  mono f32 arrives in reads of 4 to 24 KB, the first in 17 to 56 ms.
- a frame loaded into the image atlas as a fixed image costs 0.7 ms and
  is a new handle and a new upload whose pixels the store keeps for good:
  244 frames of 640x360 filled the 8192 atlas, 16.3 s of a 15 fps
  camera, with 225 MB held, and the next image of the whole app was
  refused. a camera needs one image whose pixels change in place.
- macOS input: `-f avfoundation -i "0:none"` and `-i "none:0"`, by index
  or name. its default rate is ntsc (29.97); a camera that does not take
  it logs "Selected framerate (%f) is not supported by the device.", its
  "Supported modes:" as `%dx%d@[%f %f]fps` lines, and ends. libavdevice
  holds no call that asks for or checks the permission: the system
  decides when the capture input is created, the refusal is "Failed to
  create AV capture input device: %s" with the system's description in
  it, and that description is not known here. the prompt belongs to the
  process that launched the app (the terminal of a `cargo run`, the
  `.app` of a bundle). windows: `-f dshow -i video=<name>`, by name only.
  linux: `-f v4l2 -i /dev/video<n>` and `-f alsa -i default`.
- a page in the Apps Demo group moves none of the 161 regress baselines:
  at 800 px the sidebar ends before that group.

## decision

- **capture is a trait over devices, the real one a child.**
  `engine::platform::capture::Capture` starts and stops each
  `DeviceKind` without blocking the loop and answers where it is
  (`CaptureState`: `Off`, `Starting`, `On`, `Denied`, `NoDevice`,
  `Failed(why)`), the newest camera `Frame` (RGBA of the asked size; a
  frame nobody took is replaced by the next, one frame held) and the
  microphone's `AudioChunk`s since the last take (a twentieth of a second
  of mono f32 each, in a ring bounded to two seconds). `FfmpegCapture`
  runs one ffmpeg per device on the platform's input, scales and crops
  the frames on the output (no size or rate forced on the input) and
  reads raw frames and samples from its pipe on a thread; a watcher
  thread reads every stderr line into the state, kills the child by its
  pid when nothing came within 15 s (a camera warming up, a person at the
  prompt), and starts a camera that refused its default rate once more
  with a mode it listed. stopping kills and reaps the child, and a
  dropped capture stops both; a parent that dies anyway leaves ffmpeg
  writing into a pipe nobody reads, and its next write ends it.
  `FakeCapture` makes frames and a sine without a process, `NoCapture`
  is wasm and mobile, `default_capture` chooses, `PLEV_FFMPEG` names a
  test's fake.
- **the permission is a state.** a refusal is `Denied`, a missing device
  `NoDevice`, anything else `Failed` in ffmpeg's words, read from every
  line (the last is often a generic "Input/output error"). the page says
  it in one line an agent reads; nothing fails.
- **turning a device on is a person's.** the showcase's Capture page
  projects `capture.camera` and `capture.microphone` as toggles with
  `PERSON_ONLY` (the shell's window and print policy): a person, a screen
  reader and a test may flip them, an agent is refused before anything
  runs and reads `capture:camera` and `capture:microphone` ("off",
  "starting", "on 640x360 at 15 fps", "on 16000 Hz", "permission
  denied", "no camera", "failed: ...", "paused while the page is not
  seen"). the pixels and the samples are never projected.
- **the frame is a stream image.** `ImageStore::stream` packs one slot
  out of the dedupe; `replace` rewrites its entry in the log in place and
  answers the same placement under a new id, so the node that draws it
  differs and its layer draws again with no compositor change; every
  atlas over the store (a context per window, the headless one) writes a
  replaced entry once by its revision, and an atlas that starts from
  nothing, or a recovered device, writes the latest pixels. the page
  makes its slot at the first frame and keeps it for its life: turning
  the camera off and on reuses it, one frame of memory per page. the
  newest frame goes in on the view's wake after it arrived, never while
  a frame is built and never asleep.
- **covered, asleep or left, the device is released.** a device runs
  only while the person wants it and the page can be seen: it is the
  active page, a window that shows it is awake, the app does not sleep.
  the shell passes the windows' word the moment it hears it (covered,
  shown, asleep, awake, the page's own window closed), since a covered
  window ticks only when something wakes the loop; the page stops the
  device (the child dies, the camera's light goes off), empties the frame
  box and drops what was handed and not taken, and starts it again when
  it can be seen, if the person had it on. what shows first is a frame
  captured after that, never the last one from before. a device that
  ended on its own is not started again at every wake. released, not
  paused: a paused device would keep the camera open for nobody.
- **render on demand holds.** while a device is live the page asks for
  its next look a frame period later (`Wake::after(1/fps)`); with none
  live it asks for nothing.

## reason

- no dependency: ffmpeg as a child is the form the engine already has
  for printing and the showcase for its ffmpeg jobs; one process per
  device keeps a failing microphone from taking the camera with it.
- the permission cannot be asked for without AVFoundation; stating what
  the child said as the device's state keeps a refusal a fact the person
  and an agent can read, and the retry with the camera's own modes keeps
  a default the camera refuses from being a failure.
- the person's: a camera is the most private thing an app can turn on;
  the policy on the node holds for the pointer, the keys, a screen reader
  and the port alike, and a gallery built without a shell holds no
  device at all.
- release over pause: what nobody sees is not captured, and the light
  says so.
- one slot per stream: frame after frame, the atlas and the store stay
  the size of one frame, where fixed images would fill the atlas in
  seconds.

## consequences

- proven without a device: the stream image (unit tests over the log and
  two cursors, and on the GPU two renderers, a late one and a recovered
  one), the capture over three fakes (`rust/engine/tests/fakes/ffmpeg`
  runs the real ffmpeg on the capture's own output arguments with a lavfi
  source; `fake-ffmpeg-refuses`; `fake-ffmpeg-mute`), the page over
  `FakeCapture`, the shell covering and waking over a bare window, and the
  first frame after the wake read back from a headless host. no test and
  no gate step opens a device.
- the text of a denied permission is a guess ("not authorized", "Cannot
  use"), and so is the microphone: a denied microphone may hand silence
  instead of failing. the real proof is the person's.
- the recording (`PLEV_RECORD=1`) keeps an image asset per frame id it
  saw, with its size and without pixels (the store's pixels are never
  written into a session); a session read back draws those nodes as
  nothing, and a live replay in the same process would draw whatever the
  stream holds now. `--svg` marks the frame as an image left out, as for
  every image. neither was changed.
- a stream slot is never freed: the grow-only atlas keeps it, as every
  other image.
- ffmpeg must be installed for the camera to come on; without it the
  page says "failed: capture: could not start ffmpeg: ...".

## alternatives

- Dependência nova para captura (`objc2-av-foundation`, `cpal`, `coreaudio`): o lock não tem essas crates e nenhuma dependência pode ser acrescentada. O ffmpeg como processo filho já é a forma que o engine usa (*context*, *reason*).
- Pausar o dispositivo em vez de liberá-lo: a câmera ficaria aberta para ninguém (*reason*).
- Quadro da câmera como imagem fixa: enche o atlas em segundos. Um slot de fluxo ocupa a memória de um quadro (*context*, *reason*).
- Permissão negada como falha: `Denied` é um fato que a pessoa e o agente leem (*reason*).
- Captura ligada pelo agente: a pessoa liga, e o agente só lê o estado (*decision*, *avoid*).

## bring-up

- AVFoundation directly (`AVCaptureDevice` and its
  `authorizationStatus`/`requestAccess`, when a crate for it is approved):
  the permission known before a start, `Denied` said by the system, the
  device list for a choice.
- a `.app` bundle: `NSCameraUsageDescription` and
  `NSMicrophoneUsageDescription` in its `Info.plist`, since the bundle is
  then the process the system asks about, for ffmpeg as a child too.
- windows (dshow names from a device list) and linux (v4l2 and alsa or
  pulse) run on a real machine; mobile (the system's camera APIs) with
  its bring-up.
- a device list and a choice in the page; index 0 today.

## avoid

- a test or a gate step that reaches a device: the fakes by path, lavfi
  sources only.
- a capability an agent may turn on; the agent reads the state.
- a camera frame loaded as a fixed image: it fills the atlas.
- a frame replaced while a frame is built, or while the app sleeps.
- a device that runs while nobody can see the page.

## integração (2026-09-24, orquestrador)

aceito sem mudança de código. a revisão conferiu: a imagem de fluxo
reescreve a entrada no lugar e cada atlas a reenvia uma vez pela
revisão; o atlas cresce copiando a textura antiga no mesmo lugar, então
o cursor continua valendo; nenhum teste chega a um dispositivo (o
`fake-ffmpeg` sai com erro se sobrar uma entrada de dispositivo); a
página começa com `NoCapture` e só o shell de janela real instala o
`default_capture()`; o estado `Starting` conta como vivo e pede prazo
ao laço. P3 registrado: no tema core o interruptor da câmera e do
microfone quase não aparece.
