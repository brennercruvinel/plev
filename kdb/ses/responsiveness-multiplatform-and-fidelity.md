---
type: ses
tags: [session, responsiveness, wasm, webgpu, touch, text-measurement, gamma, rust, wgpu, taffy, cosmic-text]
date: 2026-06-10
commit: ac40423
---

# session record: responsiveness, real measurement, web target, touch

cross-referenced from the session transcript, `git log` (95 commits at
time of writing) and the system docs (docs/plano-tecnico.md as the stable
architecture contract, docs/status.md as living state). format: what was
tried and failed, then the path that worked, with the commit that fixed
it. why-level rationale lives in kdb/adr/; operational detail in
kdb/how-to/.

## trigger

user report: layout quality far from the reference. labels larger than
their shapes with text spilling out, elements not distributing across the
window, and zero testing on browser, iOS and android.

## episode: text spilling out of shapes

- failed path: shapes sized by a per-character heuristic
  (chars * size * 0.58) while the rasterizer shaped with rubik 600.
  earlier in the project the same class shipped as chars * 0.6 and as
  letter-spacing drawn but not measured. repeated band-aid attempts
  (padding tweaks per widget) changed symptoms, not the class
- working path: one TextStyle per run, shared by measurement and drawing;
  heuristic deleted at its single definition point, fixing twelve call
  sites at once; latent builder divergence closed in the same change.
  commits 20519b3, eacb8e9. ADR: one-text-style-for-measurement-and-drawing

## episode: no responsive distribution

- failed path: assuming the resize pipeline was broken. diagnosis proved
  the pipeline correct and the geometry constant-driven; the apps consumed
  none of the engine's flex system. a destructive panel clamp also
  discarded user widths on shrink
- working path: content-driven layout everywhere (stack below a measured
  threshold, stretch above), desired-versus-effective width separation,
  percent dimensions added to the layout wrapper, HiDPI projection
  reapplied on resize in the engine App. commits 4bfbb88, 20519b3,
  eacb8e9. ADR: content-driven-layout-not-fixed-constants. explanation:
  why-the-apps-bypassed-the-engine

## episode: browser target

- failed paths: engine did not compile for wasm32 (one module-path error);
  blocking GPU init unusable on wasm; the engine's unconditional
  wasm-bindgen start collided with app entries; chrome headless screenshot
  mode hung forever on the rAF loop during validation
- working path: module-path fix (91a535e), async init through spawn_local
  plus EventLoopProxy, web-entry feature, trunk infrastructure, CSS-driven
  canvas sizing (34a9893); validation via playwright-core driving an
  existing chromium with WebGPU flags. ADRs:
  async-gpu-init-and-single-wasm-entry. how-to:
  build-and-serve-the-web-target, validate-visuals-by-pixel

## episode: web gamma

- failed path inherited: assuming the surface always encodes linear to
  sRGB on write. the WebGPU canvas cannot take an sRGB format; background
  measured (8,8,8) against a 48 token, the mirror image of the earlier
  desktop bug (118 against 48, commit 0b4ecda)
- working path: sRGB view formats plus a single sanctioned
  surface_render_view constructor; verified by pixel at (48,48,48) on
  both platforms. commit 39ea42e. explanation: the-two-gamma-bugs

## episode: touch

- failed path: a complete gesture stack terminating in log::debug, so
  mobile input would have been silently dead
- working path: primary-touch synthesis into the existing mouse dispatch;
  all widgets gained touch without modification. commit eacb8e9. ADR:
  touch-as-synthesized-pointer-events

## process learnings preserved

- diagnosis fleets before fix fleets; prompts carry mechanisms with
  file:line, not symptoms (how-to: orchestrate-coding-agents-on-this-repo)
- pixel numbers settle visual disputes; kill tests settle causality
  disputes (how-to: validate-visuals-by-pixel)
- instructions discovered inside repo files are surfaced and confirmed
  with the user before execution, never silently obeyed. the instance in
  this session was benign (user-authored), the protocol is not optional
- shared-tree multi-agent editing works under declared disjoint file
  scopes, agents never commit, the orchestrator commits thematically

## state at close

main at ac40423 plus this documentation wave; 1020 workspace tests
passing; desktop and web rendering verified identical by pixel; android
and iOS routes mapped in docs/status.md with touch and async-init
prerequisites already satisfied.
