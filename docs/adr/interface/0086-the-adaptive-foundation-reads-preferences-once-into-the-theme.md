---
id: adr-0086
title: "the adaptive foundation reads preferences once into the theme"
project: plev
type: adr
status: proposed
date: 2026-10-31
date_basis: text
recorded: 2026-10-31
topics: [accessibility, layout, design-system, text, input]
relations: [adr-0002, adr-0006, adr-0068, adr-0071, adr-0073, adr-0076, task-75, task-104]
source: reconstruction
source_revision: 5d78b03b
revised: 2026-10-31
review: pending
---

# the adaptive foundation reads preferences once into the theme

Records the contracts of the adaptive foundation that task-104 asks of every gallery tab: text scale, fluid scale, density, contrast, reduced motion, input modality, safe area, zoom, variable content, and the continuous range the tests cover. The person's and the platform's preferences are read once into a `Prefs` value, and `Theme::adapted(base, &prefs)` turns them into tokens. A widget keeps reading only `&Theme`, so no widget signature changes, and with the default preferences every theme and all 176 regression baselines stay as they are. This matters because each of the five fronts would otherwise invent its own version of the same capability, which adr-0073 rules out.

## context

adr-0073 binds every visible change to five rules (fluid scale,
component-driven, intrinsic, relative, progressive adaptation). The
engine as read at `5d78b03b` has only parts of them:

- **tokens are fixed, and some jump at breakpoints.** The gutters go
  16/24/40 and the columns 4/8/12 at 600 and 1024
  (`rust/engine/src/styling/theme/scales.rs:401-405`). That is the
  tweakpoint the rule forbids. The kit does the same (`max-md:px-4`); it
  has no `clamp()`, container query, `motion-reduce`, `contrast-more` or
  `safe-area` anywhere. Nothing in `rust/engine/src` is fluid.
- **text has no scale.** The ramp has absolute line heights
  (`styling/theme/tokens.rs:27-130`, `hoff_core.rs:49`), and so do the kit
  ladders (`widgets/buttons/tokens.rs:328-362`, read by
  `ButtonSize::text_style`, `button.rs:65`). One `TextStyle` measures and
  draws (adr-0006), and `TextNodeKey::from_style` keys on its size and
  line bits (`drawing/compositor/scene.rs:144`). So a scaled style scales
  both, and caches correctly. Literal `TextStyle::new(..)` outside the
  ramp: urnadbapp 24, uigallery 16, idestudio 3, inspector 1, and 5 inside
  `engine::widgets` (`charts/line.rs:122`, `charts/bars.rs:104`,
  `content/avatar.rs:95`, `graph/live_graph/draw.rs:39`,
  `editor/config.rs:64`).
- **density exists twice and is used by nothing in the engine.**
  `Density { Comfortable, Compact }` with a factor of 0.85
  (`scales.rs:368-382`) and `touch_height` (`:246`) are reached only by
  `theme/tests/scales.rs:79`. The IR has its own: `Op::Density { step }`
  moves every gap and padding one step of the spacing scale
  (`rust/plevir/src/transform/look.rs:157`). `Preference::{LargerText,
  MoreContrast, Denser, Sparser}` already exists
  (`plevir/src/transform/prefer.rs:50`).
- **contrast is measured, not adapted.** 301 pairs are measured against
  WCAG AA, and 102 fail (core 36, hoff 15, dark 11, light 40). They are
  listed in `KNOWN` (`styling/theme/tests/contrast.rs:196`), and the
  decision on them is open in task-75. OKLCH is available
  (`theme/color_space.rs:16`, `oklch_shift` `:97`).
- **reduced motion is read on macOS only**, through `defaults`
  (`control/accessibility/reduced_motion.rs:27`). It is held per thread
  (`compose/animation/mod.rs:17-41`), and every finite tween and spring
  ends at once (`tween.rs:117`, `spring.rs:103`). Only the accessibility
  `Bridge` reads it (`bridge.rs:101`, `:124`). The bridge is native only,
  so the web never reads the preference.
- **input carries no source.** `WidgetEvent` (`widgets/widget.rs:72`) has
  no pointer kind, and winit 0.30 has no pen. Touch has two paths: the
  engine's `TouchPointerSynth` (`runtime/window/events.rs:66-83`) and the
  gallery's own `Finger` (`apps/uigallery/src/shell/system/touch.rs:20`).
  The engine's recognized gestures are drained into a log line and reach
  no widget (`events.rs:101-107`). `min_touch_target` (44) is unused.
- **safe area exists twice and is wired to nothing.** `SafeArea`
  (logical, `widgets/shell.rs:18`) is consumed by `AppShell::layout`.
  `SafeAreaInsets` (physical, `runtime/platform/mod.rs:19`) is filled on
  Android only (`:52`). No caller outside a test
  (`widgets/tests/components.rs:634`) passes one to the other.
- **there is no zoom.** The scale factor comes from the window
  (`runtime/window/host.rs:199`, `:332`). The bridge reads
  `window.scale_factor()` directly (`bridge.rs:149`).
- **the only channel every widget receives is `&Theme`**, in `render` and,
  where layout is token-driven, in `handle_event`. `Theme` derives `Clone`
  and `Debug`, not `PartialEq` (`styling/theme/mod.rs:28`); its token
  structs do.
- **dependencies already in `Cargo.lock`:** `ashpd` 0.11.1 on Linux,
  through `rfd`'s portal backend. It has `Settings::contrast()`,
  `read<T>()` and `receive_*_changed()`, and no `reduced_motion()`.
  `objc2-app-kit` 0.3.2 and `windows` 0.62.2 are in the lock too. CI runs
  macOS, Ubuntu and wasm32; it has no Windows, iOS or Android runner.

What the reference engines taught, checked at the sources named in the
task-104 study:

- Flutter turned `double textScaleFactor` into the `TextScaler` function
  across painting, rendering and widgets (3.16). Android 14's nonlinear
  curve made a scalar wrong.
- Android 14 scales text up to 200% on a lookup-table curve. Line
  height follows only when it is in sp, and padding is never in sp.
- Apple's Dynamic Type is a table per style: Body goes from 17 to 53 pt
  (Large to AX5), Large Title from 34 to 60. `@ScaledMetric` scales a
  non-text metric by a style's curve. macOS doesn't support Dynamic Type
  (HIG, checked).
- Flutter's `VisualDensity` runs from -4 to 4, in 4 px steps (checked),
  and never touches text.
- In CSS, fluid type in viewport units fails WCAG 1.4.4 under zoom. Browser
  zoom changes `devicePixelRatio`, which winit reports as the scale
  factor. The canvas is never forced by `forced-colors`.

## decision

### the channel

- `Prefs` is one plain value (`Clone, Debug, Default, PartialEq`). Its
  fields: `text_scale`, `density`, `contrast`, `reduced_motion`,
  `input`, `zoom`. Platform readers fill it once, in
  `runtime/platform/prefs`, one reader per platform; a reader that cannot
  answer returns `None`, and the field keeps its default. The app may
  override a field (the person's choice in the app), and a test sets it
  directly.
- `Theme::adapted(base, &prefs) -> Theme` is the only place a preference
  becomes a token. Every token slot (the fronts add theirs) has an `adapt`
  step. A widget reads only `&Theme`, as today.
- `Theme::adapted(base, &Prefs::default())` yields the base theme slot for
  slot.
- `reduced_motion` stays the thread's flag that tweens and springs
  read, but it is written from `Prefs`, in one place, and no longer by the
  bridge directly.

### text scale

- `TextScale` is a curve: points `(size, scaled)` with linear
  interpolation and a ceiling, the shape of Android's
  `FontScaleConverter` tables. `TextScale::linear(f)` is the two-point
  case, and `IDENTITY` is the default.
- `adapted` scales every ramp style and every kit ladder:
  - font size: `scale(size)`;
  - line height: by the same ratio;
  - tracking: it is in em, so it follows the size;
  - ladder heights: they become minimums that grow by the line-height
    delta (`h' = h + (lh' - lh)`), so padding is kept;
  - padding and gaps: never scaled.
- A metric that follows text (an icon beside a label) uses
  `theme.text_scale.metric(v, relative_to)`, as `@ScaledMetric` does.
  Icon glyphs scale as metrics, not as text.
- The 5 literal styles inside `engine::widgets` move onto the ramp in
  this work. Consumers' literal styles keep working unscaled until they
  move.
- The app never lowers the platform's value to make a layout fit.
- Readers: GNOME `org.gnome.desktop.interface text-scaling-factor`
  (0.5 to 3.0, checked in the schema), through the portal and then
  `gsettings`. On the web, the root font size over 16: desktop browsers
  fold their default-font-size setting into it (CSSWG explainer,
  checked). Reading it from the canvas app's script is **not verified**.
  macOS has no system value, so the value is the app's.

### fluid scale

- A `Fluid { min, max, from_w, to_w }` token is resolved against the
  width of the `Rect` the component receives (`fluid.at(rect.w)`), never
  against the window.
- It replaces what jumps today: the page gutter, the grid gap, the spacing
  between sections, and the page header's title and display size.
- Its points give today's value from 1024 up, so the 1280 baselines are
  unchanged.
- Text scale applies after the fluid value.
- Kit ladders stay discrete, because their size is part of the design.
- A change of composition (label to icon, toolbar to menu) chooses the
  first candidate that fits, the way `ViewThatFits` does. That is a
  breakpoint of composition, which adr-0073 allows.

### density

- Compact moves one rung down the kit ladders (`ControlTokens.heights`,
  `pad_x`, `ButtonTokens.sizes`) and one step down the spacing scale.
  This is the meaning `Op::Density` already has in the IR, and it uses no
  number the kit lacks.
- The multiplicative factor goes.
- Comfortable is the default.
- Touch input is not density (see input).

### contrast

- `Contrast { Standard, More }`.
- In More, `adapted` moves each text and border token's OKLCH lightness
  until its pair reaches the target. It makes glass opaque and widens the
  border and the focus ring (tokens). A surface whose only separation
  is a shadow gains a border.
- Standard is today's theme, so the 102 pairs stay in `KNOWN` until
  task-75 decides them.
- Readers: macOS `increaseContrast`, through the same path as reduced
  motion. GNOME: `ashpd` `contrast()` and its change stream, then
  `gsettings` `org.gnome.desktop.a11y.interface high-contrast`. The web:
  `matchMedia('(prefers-contrast: more)')`.

### reduced motion

- The semantics stay as task-75 phase 1 set them: every finite
  transition ends on its first tick.
- Readers:
  - macOS (moves into the common module);
  - Linux: the portal key `org.freedesktop.appearance reduced-motion`
    (xdg-desktop-portal 1.21.0 and later, checked), read through
    `ashpd::Settings::read`, then `gsettings`
    `org.gnome.desktop.interface enable-animations` (checked in the
    schema);
  - the web: `matchMedia('(prefers-reduced-motion: reduce)')` with its
    `change` event. This closes the gap the native-only bridge leaves.
- Where something travels (Sheet, Drawer, Accordion, Collapsible,
  Carousel, the Toggle Group indicator), the duration is
  `theme.duration.travel(distance, extent)`. It runs between `fast` and
  `slow`, by the share of the container the motion crosses.

### input modality

- Each window keeps `InputModality` (`Mouse`, `Touch`, `Pen`, `Keyboard`)
  from its last event, and `hover_capable` with it.
- A widget reads it where it decides:
  - with Touch, the hit area is `max(visual, min_touch_target)`, centered
    on an unchanged visual;
  - with no hover, a hover-only widget (Tooltip, Hover Card) opens on
    focus and on long press;
  - the focus ring shows only after keyboard input.
- The gallery's `Finger` and the engine's `TouchPointerSynth` become one
  path, and the recognizer's long press reaches widgets.
- Pen stays indistinguishable until winit 0.31.

### safe area

- One logical `SafeArea`. The host converts the platform's physical
  insets once, on attach and on every resize, and hands them to the
  shell.
- A widget inside a page reads nothing, because its `Rect` is already
  inside the safe area.
- `ShellLayout` gains `safe: Rect` (the window minus the insets). The
  overlay manager clamps against it: popovers, menus, sheets, drawers,
  dialogs and toasts. A bottom sheet or drawer reaches the edge and pads
  its content by the bottom inset.
- Android fills the insets. iOS stays at zero until winit 0.31's
  `safe_area`. Desktop and detached windows are zero.

### zoom

- Desktop zoom is part of the scale factor:
  `WindowHost::scale_factor()` returns device × zoom, and
  `device_scale_factor()` returns the device's own.
- Everything that reads the host follows, and the bridge takes the
  effective factor instead of `window.scale_factor()`.
- Cmd/Ctrl `=`, `-`, `0` change it, in token steps, and the value is
  kept between sessions.
- The web does not take those keys: browser zoom already arrives as the
  scale factor.

### variable content and the continuous range

- The engine holds shared content fixtures: long (×1.4, and ×3 for
  labels), an unbreakable compound word, CJK, RTL, emoji, empty, and wide
  numerals. A page takes a fixture without being edited.
- One test sweeps every page in `PAGES`, without a GPU:
  - widths 280 to 1600 every 8 px, at heights of 800 and 360;
  - text scale 1.0, 1.5 and 2.0;
  - both densities;
  - normal, long and empty content;
  - More contrast, Touch and safe-area insets at one narrow and one wide
    width.
- The sweep asserts:
  - no panic;
  - every scene node lies horizontally inside the content rect, unless it
    is inside a scroll region's clip;
  - no two interactive binding rects overlap (pages in `PROJECTING`);
  - every text node's measured width fits its allotment, unless it was
    truncated with `truncate_to_width`;
  - with Touch, every hit area is at least `min_touch_target`.
- The width sweep is also the zoom test: 1280 px at 200% is 640 logical
  pixels, and at 400% it is 320 (WCAG 1.4.10).

### outside this record

- **viewport breakpoints for pixels, and size classes as a component
  API.** adr-0073 forbids the first. A component gets its `Rect`, and the
  only size class is the shell's `SidebarMode`.
- **`forced-color-adjust` and CSS system colors.** The browser does not
  force a canvas.
- **macOS Dynamic Type.** The platform has none.
- **magnifier and pinch zoom of content.** They belong to the OS.
- **pen pressure and tilt, foldables, tvOS, watchOS, visionOS.** No tab
  uses them, and they are not targets.
- **reduce transparency, inverted colors, differentiate without color.**
  They are not in task-104's table. More contrast already makes glass
  opaque.
- **readers on Windows, iOS and Android.** Their contract is in `Prefs`.
  The readers wait for a platform that is validated (open question 7).
- Android 15's edge-to-edge does apply: the engine targets SDK 35
  (`rust/engine/Cargo.toml`). The Android inset reader already exists.

## consequences

- **themes.** Each token slot gains an `adapt`, and the five fronts add
  theirs with their slot. A slot without one does not scale, and the
  sweep at text scale 2.0 shows it as overflow.
- **consumers.** idestudio, urnadbapp, inspector and `rust/screen` (34
  calls to `Theme::hoff_core()`, 11 in `src/`) change nothing until they
  call `adapted`. Their literal text styles stay fixed. The editor's own
  font size stays an editor zoom on top of the text scale. `rust/screen`
  keeps its own `set_reduced_motion` (`screen/src/host/mod.rs:510`).
- **baselines.** Default preferences produce today's tokens, so the
  `core@1280x800` and `hoff@1280x800` baselines do not move with the
  foundation alone. Fluid gutters change 360 and 768 only; those diffs
  are expected, reviewed and approved before the bless.
- **a test keeps it true:** `adapted(default)` equals the base slot for
  slot in every built-in theme. With More contrast, the 301 pairs pass
  with no `KNOWN` list, and the fluid tokens reproduce today's values
  from 1024 up.
- **sweep cost.** 82 pages × 166 widths × 3 text scales × 2 densities is
  about 82,000 layouts before the content axis. It is to be measured on
  the HF before the step is fixed.
- **dependencies.** `ashpd` becomes a direct Linux dependency of the
  engine (no new crate in the lock). `web-sys` gains `MediaQueryList` and
  `MediaQueryListEvent`.
- **made hard on purpose:** a front cannot add its own text factor,
  density or contrast palette. It extends its slot's `adapt` instead.

## alternatives

- **an environment passed beside `&Theme`** (Flutter's `MediaQuery`):
  every widget and consumer signature changes. Flutter itself had to
  split it into aspects to avoid rebuilding on every change. plev
  invalidates per app, so it would gain little here.
- **thread-local preferences, as reduced motion is:** hidden, wrong for
  per-window values (zoom, scale, insets), and a test cannot compare
  two adaptations on one thread.
- **a scalar text factor:** it is the API Flutter had to break when the
  curve arrived.
- **multiplicative density (×0.85):** it leaves the kit's grid (36 →
  30.6).
- **hand-written high-contrast palettes:** 16 palettes to keep, and the
  kit offers none.
- **pointer kind on every `WidgetEvent`:** it changes an enum every
  consumer matches on, for a distinction winit 0.30 cannot make for pens.
- **text-only zoom:** it duplicates text scale.

## open questions for Brenner

1. **Linear or curved text in the factor-only platforms** (macOS,
   GNOME, Windows, web). Recommended: a plev curve as a token, shaped
   like Android's, so titles grow less than body. Linear is simpler, but
   display 48 at 2.0 is 96 and does not fit 360. A curve keeps the
   hierarchy, but its points are a design choice to review.
2. **Ceiling.** Recommended: 2.0, Android 14's maximum. Higher, up to
   iOS Body's ×3.1, makes layouts reflow more and doubles what the sweep
   must hold.
3. **macOS text scale, which the system does not give.** Recommended: an
   app control with a launch argument, default 1. Without it, the
   capability cannot be seen on the platform where Brenner works.
4. **Density.** Recommended: a rung on the ladder, two densities, and the
   old `Density` removed. Additive steps (Flutter) bring numbers the kit
   lacks. Also decide whether idestudio defaults to Compact; that changes
   its look.
5. **More contrast.** Recommended: derived in OKLCH, with an AA target
   (4.5 for text, 3 for non-text). The other choices:
   - AAA (7) changes more colors and is further from the kit;
   - hand palettes cost 16 palettes;
   - forced colors (Windows `GetSysColor`, `forced-colors` on the web)
     respect the person's palette, but they matter only if Windows or
     forced colors are in scope.
6. **Reduced motion semantics.** Recommended: keep "every finite
   transition ends at once" now. Apple's model (movement goes, color and
   opacity fades stay) is closer to WCAG 2.3.3, and 76 of the kit's
   transitions are color fades; it changes what task-75 accepted. Also
   decide whether the spinner should stand still, since progress is
   essential motion.
7. **Platforms with readers now.** Recommended: macOS, Linux and web,
   where the 82 tabs are validated. Windows, iOS and Android readers are
   small, but no CI runs them, so they would ship untested.
8. **macOS reader path.** Keep `defaults` (no dependency, about 10 ms per
   read) or use `NSWorkspace` through `objc2-app-kit`, which is in the
   lock (exact, with a change notification). Recommended: `NSWorkspace`
   when contrast is added, since it brings change notification for both
   preferences.
9. **Desktop zoom in task-104.** Recommended: yes, it is small. Decide the
   steps (tokens) and the persistence. Without it, the zoom rule is
   covered only on the web.
10. **Input.** Recommended: `InputModality` per window, unify the touch
    paths, and deliver long press. Pen waits for winit 0.31, an upgrade
    that is still in beta.
11. **Fluid tokens and anchors.** Recommended: gutter, grid gap, section
    spacing and page header title, from 360 to 1024. Accepting means
    diffs at 360 and 768 in every page's screenshots.
12. **Variant baselines.** Recommended: tests and screenshots only, no
    `regress` baselines per text scale or contrast. Baselines would
    multiply 164 entries by each variant.
13. **RTL layout (logical start and end).** Recommended: out of
    task-104. The kit has no RTL design, and every widget with a leading
    icon would change.
14. **Keyboard (IME) inset.** Recommended: out of task-104, since no tab
    depends on it.
15. **Sweep step.** Keep 8 px on every axis, as the epic says, or keep 8
    px only at text scale 1.0 and Comfortable, after the HF measurement.
