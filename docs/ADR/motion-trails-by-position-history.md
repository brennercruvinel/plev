---
type: adr
status: accepted
tags: [rendering, particles, trails, render-on-demand]
date: 2025-09-24
---

# motion trails come from position history, not framebuffer accumulation

## context

the reference canvas demo (Entropic Life XVI, ported in crates/prime_creatures)
draws motion trails for free: each frame it paints a translucent
`rgba(5,5,8, trailFade)` rect over the whole canvas instead of clearing, so the
previous frame fades under the new one. the plev engine is the opposite by
design: every layer is cleared each frame (render on demand), and there is no
feedback primitive, `EffectProcessor` does blur, not accumulation. accumulating
directly on the swapchain surface is not an option either: it is multi-buffered,
so "the previous frame" is whichever image the queue cycled to, and the trails
would flicker.

## decision

each particle keeps a short ring buffer of its last positions (params::TRAIL_LEN).
the renderer draws that history as faded circles, oldest faint to newest bright,
behind the links and cores. the scene stays fully cleared and rebuilt every
frame, so nothing depends on swapchain contents.

## consequences

- the tail is shorter and coarser than the canvas demo's smooth accumulation.
  it reads as motion, it is not pixel-identical to the source.
- cost is TRAIL_LEN extra circles per particle. they are the same kind as the
  cores (sdf rects), so the compositor merges them into the same draw.
- the faithful upgrade is real accumulation: render the field into a persistent
  offscreen texture, fade it with a translucent quad each frame, blit to the
  surface. that is net-new gpu work (a feedback texture and a blit pass) and is
  deferred until the trail fidelity is worth the plumbing.

## avoid

- never try to accumulate on the swapchain surface directly; it is not a single
  persistent canvas, trails will flicker as images cycle.
- never reach for `EffectProcessor` expecting feedback; it blurs a frame, it
  does not carry one forward.
