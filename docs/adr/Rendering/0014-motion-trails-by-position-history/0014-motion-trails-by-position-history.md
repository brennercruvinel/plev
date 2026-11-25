---
id: adr-0014
title: "motion trails come from position history, not framebuffer accumulation"
project: plev
type: adr
status: accepted
date: 2025-09-24
date_basis: reconstructed
recorded: 2026-06-14
topics: [rendering, animation, effects]
relations: []
source: docs/adr/rendering/0014-motion-trails-by-position-history.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# motion trails come from position history, not framebuffer accumulation

Decides that motion trails come from a per-particle history of positions, and not from accumulation in the framebuffer: each layer is cleared every frame and the swapchain is multi-buffered, so an accumulated trail would flicker. The result is a trail that is shorter and coarser than the original demo's, which reads as motion but is not pixel-identical. Faithful accumulation is postponed, because it requires a feedback texture and a blit pass, new GPU work.

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

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [task-49: prime number creatures, port of entropic life](../../../Tasks/task-49-prime-creatures/task-49-prime-creatures.md)
<!-- /related -->
