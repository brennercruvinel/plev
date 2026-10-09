//! Apps Demo, Capture: the camera and the microphone as capabilities a
//! person turns on (task-76, item 10). Two switches, `capture.camera` and
//! `capture.microphone`, ask through the gallery's one dispatcher (the
//! todo page lends it, as to the Files page) and only a person may flip
//! them: an agent is refused before the page runs anything, and reads the
//! state the page projects instead (`capture:camera`, `capture:microphone`:
//! "off", "starting", "on 640x360 at 15 fps", "permission denied", "no
//! camera", "failed: ...", and "paused" while nobody can see the page).
//!
//! The devices are `engine::platform::capture`'s: the page holds none of
//! its own until the shell gives it the platform's (`set_capture`), so a
//! page built headless, in a test or for a baseline never reaches one.
//! A device runs only while the person wants it and the page can be
//! seen: the page is the active one, a window that shows it is awake
//! (the shell says so, `set_shown`), and the app does not sleep
//! (`engine::window::Background`). Covered, asleep or left, the device is
//! stopped and released (the child dies); shown again, it starts again if
//! the person had turned it on. The camera's newest frame goes into one
//! stream image (`engine::gpu::image::replace_image_rgba`) on the wake
//! that follows it, never in a frame and never asleep, and the frame box
//! draws it; after a stop the box waits for a new frame instead of
//! showing the last one. The microphone's last chunk sets the level
//! meter. While a device is live the page asks the loop for its next look
//! a frame period later; with none live it asks for nothing.
//!
//! Wiring contract for the chrome (view/dispatch.rs, view/input.rs): the
//! pointer, Tab over the two switches, Enter on the focused one, Escape
//! to let it go, as for the Files page; `wake` from the view's wake with
//! whether the page is the active one.

mod semantics;
#[cfg(test)]
mod tests;

use std::time::Duration;

use engine::widgets::prelude::{
    EventResult, ProgressBar, Rect, Switch, Wake, WidgetEvent, rounded_rect, rounded_rect_stroke,
};
use engine::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::drawing::gpu::image::{ImageHandle, replace_image_rgba, stream_image_rgba};
use engine::runtime::platform::capture::{
    Capture, CaptureSettings, CaptureState, DeviceKind, Frame, NoCapture,
};
use engine::drawing::text::{TextMeasurer, TextStyle};
use engine::styling::theme::{ControlSize, Theme};
use engine::runtime::window::Background;

use super::{EditKey, panel};
use semantics::{Ids, bounds};

/// The quietest level the meter shows, in dB below full scale: a room's
/// hush sits near it, a voice well above.
const FLOOR_DB: f32 = -60.0;

/// What the status line says: what the last request came to.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Said {
    Nothing,
    Turned(DeviceKind, bool),
}

pub struct CapturePage {
    /// The platform's devices, given by the shell; none until then.
    capture: Box<dyn Capture>,
    settings: CaptureSettings,
    /// What the person turned on: each switch's `on`, projected as the
    /// toggle's state.
    camera: Switch,
    microphone: Switch,
    /// The devices this page started and has not stopped, by
    /// `DeviceKind` order.
    started: [bool; 2],
    /// Each device's state as the page last looked at it: what the
    /// projection and the frame say, so both tell the same moment.
    seen: [CaptureState; 2],
    /// Why a start did not happen (the capture's error), per device.
    refused: [Option<String>; 2],
    /// The stream image the camera's frames go into, made at the first
    /// frame and kept for the page's life: turning the camera off and on
    /// reuses it.
    slot: Option<ImageHandle>,
    /// What the frame box shows: the slot under its latest frame's id;
    /// `None` until a frame came since the camera last started.
    showing: Option<ImageHandle>,
    /// Frames put on screen so far.
    frames: u64,
    /// The microphone's level, 0 to 1 over `FLOOR_DB`, of its last chunk.
    level: f32,
    /// A window that shows the page is awake, as the shell says.
    shown: bool,
    /// The page is the active one, as the view says on every wake.
    active: bool,
    focused: Option<DeviceKind>,
    ids: Option<Ids>,
    inert: bool,
    said: Said,
}

/// One frame of geometry: the card, a row per switch with its label and
/// its state, the frame box, the meter, the status.
struct Layout {
    panel: Rect,
    switches: [Rect; 2],
    labels: [Rect; 2],
    frame: Rect,
    meter: Rect,
    status: Rect,
}

fn slot(kind: DeviceKind) -> usize {
    match kind {
        DeviceKind::Camera => 0,
        DeviceKind::Microphone => 1,
    }
}

/// The meter's reach for a level: `rms` in dB over the floor, 0 to 1.
pub(crate) fn meter(rms: f32) -> f32 {
    if rms <= 0.0 {
        return 0.0;
    }
    ((20.0 * rms.log10() - FLOOR_DB) / -FLOOR_DB).clamp(0.0, 1.0)
}

impl CapturePage {
    /// The page with no device: the shell gives it the platform's.
    pub fn new(theme: &Theme) -> Self {
        let ids = Ids::new()
            .inspect_err(|e| log::error!("capture: the page ids do not parse: {e}"))
            .ok();
        let mut camera = Switch::new(false).with_motion(&theme.motion);
        let mut microphone = Switch::new(false).with_motion(&theme.motion);
        if let Some(ids) = &ids {
            camera = camera.sem(ids.camera.clone());
            microphone = microphone.sem(ids.microphone.clone());
        }
        Self {
            capture: Box::new(NoCapture),
            settings: CaptureSettings::default(),
            camera,
            microphone,
            started: [false; 2],
            seen: [CaptureState::Off, CaptureState::Off],
            refused: [None, None],
            slot: None,
            showing: None,
            frames: 0,
            level: 0.0,
            shown: true,
            active: false,
            focused: None,
            ids,
            inert: false,
            said: Said::Nothing,
        }
    }

    /// The devices behind the switches: the platform's from the shell, a
    /// fake in a test. What ran on the ones before is stopped first.
    pub fn set_capture(&mut self, capture: Box<dyn Capture>) {
        for kind in DeviceKind::ALL {
            if self.started[slot(kind)] {
                self.capture.stop(kind);
                self.started[slot(kind)] = false;
            }
        }
        self.capture = capture;
        self.showing = None;
        self.follow();
        self.look();
    }

    /// The shell's word on the windows: `true` while a window that shows
    /// the page is awake (not covered, the app not asleep). A device
    /// stops the moment it turns false and starts again as it turns
    /// true. `true` when the page's state moved.
    pub fn set_shown(&mut self, shown: bool) -> bool {
        self.shown = shown;
        let moved = self.follow();
        self.look() || moved
    }

    /// The view says whether an overlay holds the pointer.
    pub fn set_inert(&mut self, inert: bool) {
        self.inert = inert;
    }

    fn switch(&self, kind: DeviceKind) -> &Switch {
        match kind {
            DeviceKind::Camera => &self.camera,
            DeviceKind::Microphone => &self.microphone,
        }
    }

    fn switch_mut(&mut self, kind: DeviceKind) -> &mut Switch {
        match kind {
            DeviceKind::Camera => &mut self.camera,
            DeviceKind::Microphone => &mut self.microphone,
        }
    }

    /// Whether the person turned `kind` on.
    pub(crate) fn wanted(&self, kind: DeviceKind) -> bool {
        self.switch(kind).on
    }

    /// Whether the page can be seen now: active, in an awake window, the
    /// app awake.
    fn visible(&self) -> bool {
        self.shown && self.active && !Background::asleep()
    }

    /// Starts what the person wants while the page can be seen, stops
    /// what it does not (the device released, what it handed dropped,
    /// the frame box waiting for a new frame). A device that ended on its
    /// own (refused, failed) is not started again until it is stopped
    /// first: by the person, or by the page going out of sight. `true`
    /// when something started or stopped.
    fn follow(&mut self) -> bool {
        let visible = self.visible();
        let mut moved = false;
        for kind in DeviceKind::ALL {
            let i = slot(kind);
            let want = self.wanted(kind) && visible;
            if want && !self.started[i] {
                self.started[i] = true;
                self.refused[i] = self
                    .capture
                    .start(kind, &self.settings)
                    .err()
                    .map(|e| e.to_string());
                log::info!("capture: the page starts the {}", kind.noun());
                moved = true;
            } else if !want && self.started[i] {
                self.capture.stop(kind);
                self.started[i] = false;
                self.refused[i] = None;
                match kind {
                    DeviceKind::Camera => self.showing = None,
                    DeviceKind::Microphone => self.level = 0.0,
                }
                log::info!("capture: the page stops the {}", kind.noun());
                moved = true;
            }
        }
        moved
    }

    /// Each device's state now, kept for the projection and the frame;
    /// `true` when one moved.
    fn look(&mut self) -> bool {
        let mut moved = false;
        for kind in DeviceKind::ALL {
            let i = slot(kind);
            let now = if self.started[i] {
                self.capture.state(kind)
            } else {
                CaptureState::Off
            };
            if now != self.seen[i] {
                self.seen[i] = now;
                moved = true;
            }
        }
        moved
    }

    /// One line per device, as the page projects it.
    pub(crate) fn line(&self, kind: DeviceKind) -> String {
        let i = slot(kind);
        if !self.wanted(kind) {
            return "off".to_owned();
        }
        if !self.started[i] {
            return "paused while the page is not seen".to_owned();
        }
        if let Some(why) = &self.refused[i] {
            return format!("failed: {why}");
        }
        match &self.seen[i] {
            CaptureState::Off => "off".to_owned(),
            CaptureState::Starting => "starting".to_owned(),
            CaptureState::On => match kind {
                DeviceKind::Camera => format!(
                    "on {}x{} at {} fps",
                    self.settings.width, self.settings.height, self.settings.fps
                ),
                DeviceKind::Microphone => format!("on {} Hz", self.settings.rate),
            },
            CaptureState::Denied => "permission denied".to_owned(),
            CaptureState::NoDevice => format!("no {}", kind.noun()),
            CaptureState::Failed(why) => format!("failed: {why}"),
        }
    }

    /// The microphone's level as projected: to a tenth, so a voice does
    /// not move the projection at every chunk.
    pub(crate) fn level_tenths(&self) -> f64 {
        (f64::from(self.level) * 10.0).round() / 10.0
    }

    /// Whether the frame box shows a frame.
    pub(crate) fn showing(&self) -> Option<ImageHandle> {
        self.showing
    }

    /// The time between two looks while a device is live: a frame period.
    fn period(&self) -> Duration {
        Duration::from_secs_f64(1.0 / f64::from(self.settings.fps.max(1)))
    }

    /// The newest frame into the stream image; `false` when it could not
    /// go in (said in the log).
    fn put(&mut self, frame: &Frame) -> bool {
        let slot = match self.slot {
            Some(slot) => slot,
            None => match stream_image_rgba(frame.width, frame.height) {
                Ok(slot) => *self.slot.insert(slot),
                Err(e) => {
                    log::warn!("capture: no stream image for the camera: {e}");
                    return false;
                }
            },
        };
        match replace_image_rgba(&self.showing.unwrap_or(slot), &frame.rgba) {
            Ok(handle) => {
                self.showing = Some(handle);
                self.frames += 1;
                true
            }
            Err(e) => {
                log::warn!("capture: the frame does not go in: {e}");
                false
            }
        }
    }

    /// The page's tick, from the view's wake with whether the page is the
    /// active one: the devices follow what can be seen; while the page
    /// can be seen, the states are looked at, the newest frame goes into
    /// the stream image and the last chunk sets the level. Live, the next
    /// look is a frame period away; nothing live, nothing asked. The
    /// switches' knobs spring meanwhile.
    pub fn wake(&mut self, dt: f32, active: bool) -> Wake {
        self.active = active;
        let mut moved = self.follow();
        let springs = self.camera.tick(dt) | self.microphone.tick(dt);
        if !self.visible() {
            return Wake::moving(springs);
        }
        moved |= self.look();
        if self.started[0]
            && self.seen[0] == CaptureState::On
            && let Some(frame) = self.capture.latest_frame()
        {
            moved |= self.put(&frame);
        }
        if self.started[1] {
            let chunks = self.capture.take_audio();
            if let Some(last) = chunks.last() {
                let level = meter(last.rms());
                moved |= level != self.level;
                self.level = level;
            }
        }
        let live = (0..2).any(|i| self.started[i] && self.seen[i].is_live());
        let mut wake = if moved { Wake::now() } else { Wake::idle() };
        if live {
            wake = wake.merge(Wake::after(self.period()));
        }
        wake.merge(Wake::moving(springs))
    }

    fn label_style(theme: &Theme) -> TextStyle {
        theme.typography.base_2r()
    }

    fn line_style(theme: &Theme) -> TextStyle {
        theme.typography.caption_r()
    }

    /// The card fills the readable width, centered; a row per switch, the
    /// frame box at the camera's own size (narrower when the card is),
    /// the meter across, the status under it.
    fn layout(&self, content: Rect, theme: &Theme) -> Layout {
        let pad = theme.spacing.lg;
        let w = content.w.min(theme.size.readable_max_w);
        let x = (content.x + (content.w - w) / 2.0).floor();
        let inner_w = (w - pad * 2.0).max(0.0);
        let mut y = content.y + pad;
        let row = theme.control.height(ControlSize::Sm);
        let [sw, sh] = theme.control.switch_size;
        let label_x = x + pad + sw + theme.spacing.md;
        let label_w = (x + pad + inner_w - label_x).max(0.0);
        let mut switches = [Rect::default(); 2];
        let mut labels = [Rect::default(); 2];
        for i in 0..2 {
            switches[i] = Rect::new(x + pad, y + ((row - sh) / 2.0).floor(), sw, sh);
            labels[i] = Rect::new(label_x, y, label_w, row);
            y += row + theme.spacing.xs;
        }
        y += theme.spacing.md;
        let frame_w = inner_w.min(self.settings.width as f32);
        let frame_h =
            (frame_w * self.settings.height as f32 / self.settings.width.max(1) as f32).floor();
        let frame = Rect::new(x + pad, y, frame_w, frame_h);
        y += frame_h + theme.spacing.md;
        let meter = Rect::new(x + pad, y, frame_w, theme.spacing.lg);
        y += meter.h + theme.spacing.sm;
        let status_h = Self::line_style(theme).line_height;
        let status = Rect::new(x + pad, y, inner_w, status_h);
        y += status_h;
        Layout {
            panel: Rect::new(x, content.y, w, y - content.y + pad),
            switches,
            labels,
            frame,
            meter,
            status,
        }
    }

    /// Natural height: the card as it stands.
    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme).panel.h
    }

    /// The switch rect of `kind` for `content` (the tests click it).
    #[cfg(test)]
    pub(crate) fn switch_rect(&self, kind: DeviceKind, content: Rect, theme: &Theme) -> Rect {
        self.layout(content, theme).switches[slot(kind)]
    }

    /// A click on a switch is the person's `toggle`, through the
    /// dispatcher: the widget's own flip is put back, and the page's
    /// `execute` flips it when the request runs.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
        todo: &mut super::todo::TodoPage,
        inert: bool,
    ) -> EventResult {
        self.set_inert(inert);
        let l = self.layout(content, theme);
        let mut r = EventResult::IGNORED;
        if let WidgetEvent::MouseDown { .. } = event
            && self.focused.is_some()
        {
            self.set_focus(None);
            r.changed = true;
        }
        for kind in DeviceKind::ALL {
            let rect = l.switches[slot(kind)];
            let got = self.switch_mut(kind).handle_event(event, rect);
            r = r.merge(got);
            if got.clicked {
                self.switch_mut(kind).toggle();
                r = r.merge(self.ask(todo, kind));
            }
        }
        r
    }

    /// Tab walks the keyboard focus over the two switches and off again.
    pub fn handle_edit_key(&mut self, key: EditKey) -> bool {
        if self.inert || key != EditKey::Tab {
            return false;
        }
        let next = match self.focused {
            None => Some(DeviceKind::Camera),
            Some(DeviceKind::Camera) => Some(DeviceKind::Microphone),
            Some(DeviceKind::Microphone) => None,
        };
        self.set_focus(next);
        true
    }

    /// Enter flips the focused switch: `toggle` by the person.
    pub fn handle_enter(&mut self, todo: &mut super::todo::TodoPage, inert: bool) -> bool {
        self.set_inert(inert);
        let Some(kind) = self.focused.filter(|_| !inert) else {
            return false;
        };
        self.ask(todo, kind);
        true
    }

    /// Escape lets the keyboard focus go; `false` when there was none.
    pub fn handle_escape(&mut self) -> bool {
        if self.focused.is_none() {
            return false;
        }
        self.set_focus(None);
        true
    }

    fn set_focus(&mut self, on: Option<DeviceKind>) {
        self.focused = on;
        self.camera.set_focused(on == Some(DeviceKind::Camera));
        self.microphone
            .set_focused(on == Some(DeviceKind::Microphone));
    }

    /// The card, the two switches with their names and states, the frame
    /// box (the newest frame, or the camera's state in words), the meter
    /// and the status. Bound beside the scene: the card as `capture`,
    /// the box as `capture.frame`, the meter as `capture.level`, the
    /// status; each switch binds itself.
    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let l = self.layout(content, theme);
        panel(c, l.panel, theme);
        if let Some(ids) = &self.ids {
            c.bind(&ids.region, bounds(l.panel));
            c.bind(&ids.frame, bounds(l.frame));
            c.bind(&ids.level, bounds(l.meter));
            c.bind(&ids.status, bounds(l.status));
        }
        let label = Self::label_style(theme);
        let small = Self::line_style(theme);
        for kind in DeviceKind::ALL {
            let i = slot(kind);
            self.switch(kind).render(c, l.switches[i], theme);
            let rect = l.labels[i];
            let name = kind.noun();
            let (nw, _) = TextMeasurer::measure_styled(name, &label, None);
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(name, &label, None),
                x: rect.x,
                y: rect.y + ((rect.h - label.line_height) / 2.0).floor(),
                color: theme.colors.text.0,
            });
            let line = self.line(kind);
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(&line, &small, None),
                x: rect.x + nw + theme.spacing.md,
                y: rect.y + ((rect.h - small.line_height) / 2.0).floor(),
                color: theme.colors.text_mid.0,
            });
        }
        let f = l.frame;
        c.push(rounded_rect(
            f.x,
            f.y,
            f.w,
            f.h,
            theme.radius.md,
            theme.glass.field.0,
        ));
        c.push(rounded_rect_stroke(
            f.x,
            f.y,
            f.w,
            f.h,
            theme.radius.md,
            theme.glass.edge.0,
            theme.control.edge_width,
        ));
        match self.showing {
            Some(handle) => c.draw_image(f.x, f.y, f.w, f.h, handle, theme.radius.md),
            None => {
                let words = self.line(DeviceKind::Camera);
                let (tw, th) = TextMeasurer::measure_styled(&words, &label, None);
                c.push(SceneNode::Text {
                    key: TextNodeKey::from_style(&words, &label, None),
                    x: f.x + ((f.w - tw) / 2.0).floor(),
                    y: f.y + ((f.h - th) / 2.0).floor(),
                    color: theme.glass.text_faint.0,
                });
            }
        }
        ProgressBar::new(self.level).render(c, l.meter, theme);
        let status = self.status_text();
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(&status, &small, None),
            x: l.status.x,
            y: l.status.y,
            color: theme.colors.text_mid.0,
        });
    }

    fn status_text(&self) -> String {
        match self.said {
            Said::Nothing => "nothing is captured until a person turns a device on".to_owned(),
            Said::Turned(kind, on) => {
                format!(
                    "the {} was turned {}",
                    kind.noun(),
                    if on { "on" } else { "off" }
                )
            }
        }
    }
}

/// A page that goes away stops what it started.
impl Drop for CapturePage {
    fn drop(&mut self) {
        for kind in DeviceKind::ALL {
            if self.started[slot(kind)] {
                self.capture.stop(kind);
            }
        }
    }
}
