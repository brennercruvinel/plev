---
id: adr-0054
title: "a lost GPU device is recovered in place, and the next frame draws the same"
project: plev
type: decisao
status: aceito
date: 2026-09-24
date_basis: cabecalho
topics: [renderizacao, gpu, plataforma]
relations: [adr-0024, task-76]
source: docs/adr/rendering/0054-gpu-device-loss-recovers-in-place.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# a lost GPU device is recovered in place, and the next frame draws the same

Decide que a perda do dispositivo GPU (o driver reinicia, a GPU sai ou é desconectada, uma suspensão que o driver não sobrevive) é detectada por um sinalizador e recuperada no lugar: a engine pede um novo adaptador e um novo dispositivo, reconstrói os recursos que viviam nele e desenha o mesmo quadro de novo, sem que o app precise fazer nada além do estado que guarda no dispositivo. Antes, a perda não tinha sinal: a janela mantinha a última imagem sem erro, e a leitura headless entrava em pânico ao usar um recurso morto que o wgpu ainda validava. A sonda da task-76 mediu esses casos no Metal, e a recuperação vale para toda janela e para o renderizador headless.

## context

the engine handled one kind of loss: the surface. `SurfaceError::Lost`
and `Outdated` reconfigured the swapchain (`window/frame.rs`,
`window/render.rs`, hoje em `rust/engine/src/runtime/window/`) and the next redraw went on. the device itself (the
driver reset, the GPU switched or unplugged, a sleep the driver did not
survive, wgpu invalidating it) had no signal and no way back. everything
that lives on the device dies with it: device and queue, the surface's
configuration, every pipeline and layout, the projection buffer and bind
group, the image atlas texture, the glyph atlas of the `TextSystem`, the
effect pipelines, the texture pool, the compositor's layer textures and
vertex buffers, the timestamp query sets.

the probe (task-76, phase 1) measured what happens today on metal, with
`Device::destroy` as the loss a test can force:

- `destroy` alone tells nobody; the next `poll` delivers wgpu's lost
  callback with `(Destroyed, "")`. a driver loss is different: wgpu-core
  fires the callback at once with `Unknown` and the driver's message.
- a frame drawn on the dead device returns "1 draw, no panic": wgpu
  swallows every device-lost error (they bypass the fatal handler), so
  every `create_*` gives an invalid resource, the submit is dropped, and
  the compositor marks its layers clean over nothing. a silent black
  hole; the window keeps its last image.
- the first use of a dead resource that wgpu still validates is a fatal
  error: the headless readback panicked in `Buffer::map_async`
  ("headless_readback is invalid").
- a fresh context could not redraw an image: the atlas upload drained
  the store's only copy of the pixels.

## decision

one flag, one recovery, one place in the frame.

- **the signal** (`gpu::loss`, `rust/engine/src/drawing/gpu/loss.rs:49`). `GpuContext::from_device`, the body both
  constructors end in, installs `Device::set_device_lost_callback` on
  every device it builds. the callback raises an atomic flag and keeps
  the reason (`LostDevice { reason, message }`); the frame reads it
  without blocking (`device_lost`, `lost_device`).
  `poll_device_loss` polls once without waiting first, so a `Destroyed`
  that only arrives with a poll is seen too. a `Destroyed` the engine
  caused itself (the context's own drop) is expected and not a loss.
- **the recovery** (`gpu::recover`, `rust/engine/src/drawing/gpu/recover.rs:146`). `GpuContext::recover` asks the
  context's own instance for an adapter (compatible with the surface,
  when there is one) and a device again, and rebuilds the context through
  `from_device`: nothing is built a second way, and a hot reloaded
  shader goes through the same path as before. a window's surface
  belongs to the instance and survives the device; it is configured onto
  the new device with the same `surface_configuration` the constructor
  uses. the render config, the logical projection and the hot reload
  watchers carry over. `Err` (no adapter, no device) changes nothing:
  the context keeps its dead device and the flag, and a later attempt
  may succeed. `set_recovery_backends` lets a headless context ask
  elsewhere; a windowed one ignores it, its surface belongs to one
  instance.
- **what the frame owns beside the context** is rebuilt right after, in
  order, by `recover_frame_resources`: the glyph atlas
  (`TextSystem::recreate_atlas`: same size, empty, cache and allocator
  reset, `atlas_disturbed` raised so every layer re-resolves; the glyphs
  come back on the next frame through the raster path, a cache miss, so
  the [raster identity ADR](0024-glyph-raster-identity-and-atlas-isolation.md) holds unchanged), the effect pipelines
  (`EffectProcessor::new`), the texture pool (`clear`), the compositor's
  layer textures and buffers (`Compositor::drop_gpu_resources`, every
  layer dirty). `mark_all_dirty` was not enough: a layer texture is
  rebuilt only on a size change and a `GpuVec` only on a capacity
  change, and both would write into the dead device.
- **the image store keeps an append-only upload log**, pixels included
  (the same budget the grow-only atlas holds), and each `ImageAtlasGpu`
  remembers how far into it it wrote. an atlas that starts from nothing,
  the recovered context's or a second context's, writes the whole log on
  its next `prepare`. the store is never drained, so two atlases over the
  one global store each get every image.
- **the hook** is before the surface is acquired, where the shader poll
  already is: `present_frame` (every app's frame tail), the engine
  `App::render` and `headless::Renderer::draw` poll the device once and,
  if it is lost, `recover_or_skip`: recover and draw this frame on the
  new device, or skip the frame (`None`, `0` draws) with one error in
  the log per loss, not per frame; the next requested frame tries again.
  a `SurfaceError::Lost` on a lost device invalidates the compositor so
  a frame is requested, instead of reconfiguring the dead swapchain.
  render on demand holds: nothing happens outside a requested frame.
  the headless `read` answers `HeadlessError::DeviceLost` while the
  device is gone, where it panicked.
- **what the app does: nothing.** the scene it pushed is CPU data and is
  drawn again on the new device; the images it loaded come back from
  the log; the text comes back from the fonts. the exception is state an
  app keeps on the device itself, outside the engine (its own textures
  or buffers on `gpu.device`): that is dead after a loss, and
  `GpuContext::device_lost` before the frame, or a device that is not
  the one it saw before, is how it knows.

## consequences

- proven without a window, in `rust/engine/tests/device_loss.rs`
  through `engine::headless` (the window's frame path into a texture):
  frame A with text, a rounded rect and an image, `Device::destroy`,
  frame B pixel-identical to A (480x280, the image at its place, the
  context not lost any more); a loss between frames 1 and 2 of a
  four-frame sequence with a retained layer and no poll in between,
  every frame equal to a renderer that never lost its device and showing
  its own rect, so no stale frame; no adapter for the recovery (an empty
  backend set): two frames skipped with no panic, `read` says
  `DeviceLost`, one error logged, and the frame after the adapter is
  back equals frame A. the tests skip without an adapter.
- one non-blocking `device.poll(Poll)` per frame, a `maintain` wgpu
  recommends anyway (finished resources freed, map callbacks fired).
- the pixels of every image stay on the CPU for the app's lifetime,
  beside the texture; the atlas never evicted, so the budget is the one
  it already had.
- the recovery blocks the frame for the adapter and device requests,
  the time a headless context takes to come up. on wasm the request cannot block
  inside a frame: the frame is skipped and the loss logged once;
  `recover_frame_resources_async` exists for a shell that drives it.
- the surface texture of the frame in which a driver loss lands is still
  presented or dropped by that frame; the recovery runs in the next
  requested one, so no frame is drawn half on each device.
- a second window (task-76 item 2) gets its images from the same log
  and recovers its own context the same way; the store is shared, the
  atlases are per context.

## alternatives

O texto registra três caminhos recusados. `mark_all_dirty` não basta para recuperar o compositor: a textura de camada só é refeita quando muda o tamanho, e o `GpuVec` só quando muda a capacidade, então ambos escreveriam no device morto (decision). Drenar a loja de imagens no upload deixa a primeira atlas a preparar com os pixels de todas as outras (avoid). Reconstruir a partir do callback, na thread do wgpu, corre contra o frame, e reagir só a `SurfaceError::Lost` perde o contexto headless e a perda entre quadros (avoid).

## avoid

- reacting to a loss anywhere but before the acquire of a requested
  frame: a callback that rebuilds resources from wgpu's thread races the
  frame; a rebuild on `SurfaceError::Lost` alone misses a headless
  context and a device lost between frames.
- `mark_all_dirty` as the recovery of the compositor: it redraws into
  the dead textures.
- draining the image store on upload: whichever atlas prepares first
  takes the pixels from every other.
- an error log per skipped frame: a machine without a GPU would fill
  the log at the redraw rate.
- treating `destroy` as the only loss: a driver loss fires the callback
  with `Unknown` from whatever thread hit it, with no poll.
