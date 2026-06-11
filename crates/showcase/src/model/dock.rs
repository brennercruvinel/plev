//! Floating message dock state machine. Pure backend: no GPU, no window.
//!
//! Absorbed from examples/message_dock and rebuilt dt-based: the example
//! interpolated with per-frame `smooth()` (frame-rate dependent); here
//! every motion is a `plev::animation::Tween` ticked by `update(dt)`,
//! with dt fed by the app's `FrameClock::tick()`. Views read geometry
//! through getters against an available-space `Rect`; the only constants
//! are element sizes, gaps and a max expanded width.
//!
//! Render on demand: `update` returns true while anything still moves
//! (morph, lifts, fades, send flash, or the blinking cursor while the
//! input panel is visible). A false return means the frame is at rest.

use plev::animation::{Easing, Tween};
use plev::ui::widgets::Rect;

/// Clickable avatars in the dock roster.
pub const AVATARS: usize = 4;

const AVATAR: f32 = 40.0;
const GAP: f32 = 10.0;
const PAD: f32 = 10.0;
/// Row | separator | trailing button breathing room.
const SEP_GAP: f32 = 16.0;
const DOCK_H: f32 = 56.0;
const BOTTOM_GAP: f32 = 24.0;
/// Max hover lift in logical px.
const LIFT: f32 = 8.0;
const EXPANDED_MAX: f32 = 480.0;

const MORPH_S: f32 = 0.28;
const LIFT_S: f32 = 0.15;
const FADE_S: f32 = 0.20;
const FLASH_S: f32 = 0.45;
/// Full cursor blink cycle (visible half, hidden half).
const BLINK_PERIOD: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DockState {
    /// Collapsed, nothing under the pointer.
    Idle,
    /// Collapsed, this avatar lifted under the pointer.
    Hover(usize),
    /// Width morphing open toward the message panel for this avatar.
    Expanding(usize),
    /// Panel open, cursor blinking.
    Expanded(usize),
    /// Send fired: flash decays while the dock morphs shut.
    Sending(usize),
    /// Width morphing back to the collapsed row.
    Collapsing,
}

pub struct DockModel {
    state: DockState,
    /// 0 collapsed .. 1 expanded; drives width and the selected slide.
    morph: Tween<f32>,
    /// Input + send alpha; the separator cross-fades as its complement.
    panel: Tween<f32>,
    lifts: [Tween<f32>; AVATARS],
    fades: [Tween<f32>; AVATARS],
    /// Remaining send-flash time, counts down only in `Sending`.
    flash: f32,
    /// Time accumulated while the input panel is visible.
    blink_t: f32,
}

/// Retarget a tween from its current value (no jump); skip the no-op
/// case so a resting tween does not report `is_animating` for nothing.
fn retarget(t: &mut Tween<f32>, target: f32) {
    if !t.is_animating() && (t.get() - target).abs() < 1e-4 {
        return;
    }
    t.set_target(target);
}

impl Default for DockModel {
    fn default() -> Self {
        Self::new()
    }
}

impl DockModel {
    pub fn new() -> Self {
        Self {
            state: DockState::Idle,
            morph: Tween::new(0.0, MORPH_S, Easing::EaseInOutCubic),
            panel: Tween::new(0.0, FADE_S, Easing::EaseInOutSine),
            lifts: std::array::from_fn(|_| Tween::new(0.0, LIFT_S, Easing::EaseOutCubic)),
            fades: std::array::from_fn(|_| Tween::new(1.0, FADE_S, Easing::EaseInOutSine)),
            flash: 0.0,
            blink_t: 0.0,
        }
    }

    // -- transitions --------------------------------------------------------

    /// Advance all motion by `dt` seconds. Returns true while another
    /// frame is needed (render on demand).
    pub fn update(&mut self, dt: f32) -> bool {
        self.morph.tick(dt);
        self.panel.tick(dt);
        for t in &mut self.lifts {
            t.tick(dt);
        }
        for t in &mut self.fades {
            t.tick(dt);
        }

        match self.state {
            DockState::Expanding(i) if !self.morph.is_animating() => {
                self.state = DockState::Expanded(i);
            }
            DockState::Sending(_) => {
                self.flash = (self.flash - dt).max(0.0);
                if self.flash <= 0.0 {
                    self.state = if self.morph.is_animating() {
                        DockState::Collapsing
                    } else {
                        DockState::Idle
                    };
                }
            }
            DockState::Collapsing if !self.morph.is_animating() => {
                self.state = DockState::Idle;
            }
            _ => {}
        }

        if self.input_alpha() > 0.01 {
            self.blink_t += dt;
        } else {
            self.blink_t = 0.0;
        }
        self.is_animating()
    }

    /// Pointer entered avatar `Some(i)` or left the row (`None`). Only
    /// the collapsed states react; returns true when state changed.
    pub fn on_hover(&mut self, idx: Option<usize>) -> bool {
        match (self.state, idx) {
            (DockState::Idle | DockState::Hover(_), Some(i)) if i < AVATARS => {
                if self.state == DockState::Hover(i) {
                    return false;
                }
                self.state = DockState::Hover(i);
                for (j, lift) in self.lifts.iter_mut().enumerate() {
                    retarget(lift, if j == i { 1.0 } else { 0.0 });
                }
                true
            }
            (DockState::Hover(_), None) => {
                self.state = DockState::Idle;
                for lift in &mut self.lifts {
                    retarget(lift, 0.0);
                }
                true
            }
            _ => false,
        }
    }

    /// Click on avatar `idx`: expand toward it, re-aim a running morph,
    /// or collapse when the selected avatar is clicked again.
    pub fn on_click(&mut self, idx: usize) -> bool {
        if idx >= AVATARS {
            return false;
        }
        match self.state {
            DockState::Idle | DockState::Hover(_) | DockState::Collapsing => self.expand(idx),
            DockState::Expanding(sel) | DockState::Expanded(sel) if sel == idx => self.collapse(),
            DockState::Expanding(_) => self.expand(idx),
            DockState::Expanded(_) => {
                self.state = DockState::Expanded(idx);
                self.refade(idx);
            }
            DockState::Sending(_) => return false,
        }
        true
    }

    /// Send the message: flash, then morph shut. Valid only with the
    /// panel open (or opening).
    pub fn on_send(&mut self) -> bool {
        match self.state {
            DockState::Expanding(i) | DockState::Expanded(i) => {
                self.state = DockState::Sending(i);
                self.flash = FLASH_S;
                retarget(&mut self.morph, 0.0);
                retarget(&mut self.panel, 0.0);
                for fade in &mut self.fades {
                    retarget(fade, 1.0);
                }
                true
            }
            _ => false,
        }
    }

    fn expand(&mut self, idx: usize) {
        self.state = DockState::Expanding(idx);
        self.blink_t = 0.0;
        retarget(&mut self.morph, 1.0);
        retarget(&mut self.panel, 1.0);
        for lift in &mut self.lifts {
            retarget(lift, 0.0);
        }
        self.refade(idx);
    }

    fn collapse(&mut self) {
        self.state = DockState::Collapsing;
        retarget(&mut self.morph, 0.0);
        retarget(&mut self.panel, 0.0);
        for fade in &mut self.fades {
            retarget(fade, 1.0);
        }
    }

    fn refade(&mut self, selected: usize) {
        for (j, fade) in self.fades.iter_mut().enumerate() {
            retarget(fade, if j == selected { 1.0 } else { 0.0 });
        }
    }

    // -- state getters ------------------------------------------------------

    pub fn state(&self) -> DockState {
        self.state
    }

    pub fn selected(&self) -> Option<usize> {
        match self.state {
            DockState::Expanding(i) | DockState::Expanded(i) | DockState::Sending(i) => Some(i),
            _ => None,
        }
    }

    pub fn is_animating(&self) -> bool {
        self.morph.is_animating()
            || self.panel.is_animating()
            || self.lifts.iter().any(Tween::is_animating)
            || self.fades.iter().any(Tween::is_animating)
            || matches!(self.state, DockState::Sending(_))
            || self.input_alpha() > 0.01 // blinking cursor keeps frames coming
    }

    // -- geometry, derived from the available area --------------------------

    fn collapsed_width(&self, area: Rect) -> f32 {
        let natural = PAD + AVATARS as f32 * (AVATAR + GAP) - GAP + SEP_GAP + AVATAR + PAD;
        natural.min(area.w)
    }

    fn expanded_width(&self, area: Rect) -> f32 {
        area.w.min(EXPANDED_MAX).max(self.collapsed_width(area))
    }

    /// Current dock width: the morph interpolates collapsed..expanded,
    /// both clamped to the area so a resize never overflows.
    pub fn width(&self, area: Rect) -> f32 {
        let (c, e) = (self.collapsed_width(area), self.expanded_width(area));
        c + (e - c) * self.morph.get()
    }

    /// Dock bounds: bottom-anchored, horizontally centered in `area`.
    pub fn dock_rect(&self, area: Rect) -> Rect {
        let w = self.width(area);
        let y = (area.y + area.h - DOCK_H - BOTTOM_GAP).max(area.y);
        Rect::new(area.x + (area.w - w) / 2.0, y, w, DOCK_H)
    }

    /// Avatar bounds including hover lift; the selected avatar slides to
    /// the front slot as the morph progresses.
    pub fn avatar_rect(&self, idx: usize, area: Rect) -> Rect {
        let dock = self.dock_rect(area);
        let row_x = dock.x + PAD + idx as f32 * (AVATAR + GAP);
        let x = if self.selected() == Some(idx) {
            row_x + (dock.x + PAD - row_x) * self.morph.get()
        } else {
            row_x
        };
        let y = dock.y + (DOCK_H - AVATAR) / 2.0 - self.avatar_lift(idx);
        Rect::new(x, y, AVATAR, AVATAR)
    }

    /// Hover lift of avatar `idx` in logical px (0 when not lifted).
    pub fn avatar_lift(&self, idx: usize) -> f32 {
        self.lifts.get(idx).map_or(0.0, |t| t.get() * LIFT)
    }

    pub fn avatar_alpha(&self, idx: usize) -> f32 {
        self.fades.get(idx).map_or(0.0, Tween::get)
    }

    /// Message input (and send button) opacity.
    pub fn input_alpha(&self) -> f32 {
        self.panel.get()
    }

    /// Separator opacity, the cross-fade complement of the panel.
    pub fn separator_alpha(&self) -> f32 {
        1.0 - self.panel.get()
    }

    /// Blinking caret opacity: square wave over `blink` time, gated by
    /// the panel fade so the caret never outshines a hidden input.
    pub fn cursor_alpha(&self) -> f32 {
        if self.blink_t % BLINK_PERIOD < BLINK_PERIOD * 0.5 {
            self.input_alpha()
        } else {
            0.0
        }
    }

    /// "Sent" feedback intensity, decaying linearly during `Sending`.
    pub fn flash_alpha(&self) -> f32 {
        if matches!(self.state, DockState::Sending(_)) {
            self.flash / FLASH_S
        } else {
            0.0
        }
    }
}
