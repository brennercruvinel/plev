---
project: plev
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-13
domain: competitive
---

# makepad visual feature inventory vs plev gap analysis

date: 2026-03-13

## legend
- [x] plev supports
- [ ] gap - plev does not support yet
- [~] partial support

---

## a. drawing primitives

| feature | makepad | plev | notes |
|---------|---------|------|-------|
| solid rect | drawquad/drawcolor | scenenode::rect | done |
| rounded rect | SDF box, per-corner radius | scenenode::roundedrect (uniform radius) | [~] plev has uniform radius only, makepad has per-corner |
| border | 2-layer beveled borders | roundedrect border_width + border_color | [~] plev has single border, no bevel |
| circle | SDF circle | -- | [ ] gap: need SDF circle or use roundedrect with radius=50% |
| arc | SDF arc with round/flat caps | -- | [ ] gap |
| hexagon | SDF hexagon | -- | [ ] gap |
| line segment | SDF line with aa | -- | [ ] gap: can fake with thin rect |
| triangle | SDF or path | path (lyon) | done |
| arbitrary path fill | path tessellation | path (lyon filltessellator) | done |
| arbitrary path stroke | path tessellation | path (lyon stroketessellator) | done |
| bezier curves | cubic/quadratic | pathbuilder cubic_to/quad_to | done |
| SVG parsing | drawsvg | -- | [ ] gap |

## b. color & gradients

| feature | makepad | plev | notes |
|---------|---------|------|-------|
| solid color | yes | yes | done |
| linear gradient | yes (h/v/angle) | -- | [ ] gap: shader needs gradient uniforms |
| radial gradient | yes | -- | [ ] gap |
| color mixing/interpolation | yes | color.rs helpers | done |
| HSV conversion | yes | -- | [ ] minor gap |
| premultiplied alpha | yes | yes | done |

## c. text

| feature | makepad | plev | notes |
|---------|---------|------|-------|
| shaped text (harfbuzz) | yes (rustybuzz) | yes (cosmic-text/harfbuzz) | done |
| glyph atlas | yes (msdf) | yes (r8unorm bitmap) | done (different approach) |
| multiple font sizes | yes | yes | done |
| text selection | yes | textinput selection | done |
| text cursor + blink | yes | textinput | done |
| rotated text | drawrotatedtext | -- | [ ] gap |
| markdown rendering | yes (widget) | -- | [ ] gap (complex, defer) |
| code blocks | yes | -- | [ ] gap |

## d. effects

| feature | makepad | plev | notes |
|---------|---------|------|-------|
| gaussian blur | yes | 13-tap separable (effects.rs) | done |
| box shadow | SDF shadow function | shadow silhouette pass | done |
| glow | SDF glow operation | -- | [ ] gap: can add to rect_sdf shader |
| opacity | per-layer | per-layer opacity | done |
| anti-aliasing | per-pixel SDF | rect_sdf smoothstep | done |
| clipping | viewport + custom | -- | [ ] gap: no clip rect system |
| blend modes | imageblend widget | -- | [ ] gap |

## e. layout

| feature | makepad | plev | notes |
|---------|---------|------|-------|
| flexbox | custom flow | taffy 0.9 | done |
| absolute positioning | yes | yes | done |
| padding/margin | yes | taffy | done |
| scroll | yes | scrollstate | done |
| responsive/adaptive | adaptiveview | -- | [ ] gap |

## f. widgets / components

| feature | makepad | plev | notes |
|---------|---------|------|-------|
| button | 6 variants | builder API button() | [~] basic only |
| checkbox | toggle + 3-state | -- | [ ] gap |
| radio button | yes | -- | [ ] gap |
| slider | animated handle | -- | [ ] gap |
| toggle/switch | yes | -- | [ ] gap |
| dropdown | yes | -- | [ ] gap |
| textinput | full IME | textinput component | done |
| label | yes | text() builder | done |
| image | async + scale | -- | [ ] gap |
| icon/SVG | yes | -- | [ ] gap |
| portallist (virtual scroll) | yes | -- | [ ] gap |
| filetree | yes | -- | [ ] gap |
| tabbar | yes | -- | [ ] gap |
| modal | yes | overlaymanager | done |
| tooltip | yes | overlaymanager | done |
| contextmenu | yes | overlaymanager | done |
| spinner/loading | yes | -- | [ ] gap |
| chart (line/bar) | yes | -- | [ ] gap |

## g. animation

| feature | makepad | plev | notes |
|---------|---------|------|-------|
| 31 easing functions | yes | yes (animation.rs) | done |
| tween (duration) | yes | tween<t> | done |
| spring (physics) | yes | spring<t> | done |
| keyframe sequences | yes | -- | [ ] documented as needed |
| state transitions | hover/down/focus | input system + manual | [~] no automatic state animation |
| repeat/reverse | yes | tween supports | done |

## h. input

| feature | makepad | plev | notes |
|---------|---------|------|-------|
| mouse events | yes | inputstate | done |
| touch events | yes | gesturerecognizer | done |
| keyboard | yes | yes | done |
| IME | yes | ime.rs | done |
| gestures (6-state) | touchgesture | gesturerecognizer | done |
| hit testing | yes | linear reverse | done |

## i. platform

| feature | makepad | plev | notes |
|---------|---------|------|-------|
| macos/metal | yes | yes | done |
| ios/metal | yes | yes | done |
| linux/vulkan | yes | yes | done |
| android/vulkan | yes | yes | done |
| windows/dx12 | yes | yes | done |
| browser/webgpu | yes | yes | done |
| safe area insets | no | yes | plev advantage |

---

## priority gaps for visual showcase

### can implement now (use existing primitives):

1. **per-corner radius** - extend roundedrect vertex to vec4 radii, update rect_sdf.wgsl
2. **circle** - roundedrect with radius = min(w,h)/2 (already works)
3. **glow effect** - add SDF glow in rect_sdf.wgsl fragment shader
4. **gradient fill** - add gradient mode to rect_sdf (direction + 2 colors)
5. **checkbox/radio/toggle** - compose from roundedrect + path + animation
6. **slider** - roundedrect track + circle handle + drag input
7. **loading spinner** - animated arc via path
8. **progress bar** - two overlapping roundedrects
9. **chip/badge/tag** - roundedrect + text
10. **color swatch grid** - already possible
11. **animated state transitions** - tween<color> on hover/press

### needs new subsystem (future tasks):

- gradient shader (linear/radial) - new uniform + shader branch
- clipping system - scissor rect or stencil
- image loading + texture binding
- SVG parsing (use usvg crate)
- virtual scrolling (portallist pattern)
- charts (line/bar tessellation)

---

## makepad uizoo tabs we can replicate

| tab | can replicate? | how |
|-----|---------------|-----|
| button | yes | roundedrect + text + hover/press animation |
| checkbox | yes | roundedrect + path checkmark + toggle animation |
| label | yes | text nodes |
| slider | yes | roundedrect track + circle handle |
| radiobutton | yes | circle + inner circle animation |
| scrollbar | yes | roundedrect |
| spinner | yes | animated path arc |
| textinput | yes | already have textinput |
| layout | yes | taffy flexbox demos |
| view | yes | roundedrect with shadow |
| linklabel | partial | text + underline rect |
| icon | no | need SVG/image support |
| image | no | need texture loading |
| video | no | out of scope |
| chart | partial | path tessellation for lines |
| markdown | no | complex parser needed |
