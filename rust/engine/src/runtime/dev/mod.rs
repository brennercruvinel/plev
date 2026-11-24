//! The dev mode: the engine's own view of what it does, drawn over any
//! app. A small "Dev" button sits over the page; it opens a panel docked
//! in the window, below the page or beside it, and the page gets the rest
//! of the window ([`DevMode::page_size`]), so none of it hides under the
//! panel. The panel reads the frames a [`Recorder`] keeps (the stages,
//! the zones, the layers and the GPU's passes of each one, what code said
//! and what the heap did) and shows the one chosen in seven views:
//!
//! - Frame: the whole frame at a high level, phase by phase.
//! - GPU: every pass with the layer it drew, the layers, the counters.
//! - Zones: the CPU's named zones on a time axis, their statistics, and
//!   how the runs of one zone spread over the frames kept.
//! - Plots: what each frame counted, and the values code gave, as charts.
//! - Memory: the heap block by block, and what the engine holds on the GPU.
//! - Log: the lines code said, frame by frame.
//! - Inspect: one control of the page, from its meaning to its cost.
//!
//! Nothing here measures: the clocks are `perf::stages`, `perf::zones`
//! and `perf::gpu_time`, the counters `perf::marks` and `perf::heap`. The
//! shell that mounts it turns the recorder on
//! while the panel is open, hands it the pointer first ([`DevMode::event`])
//! and draws it after the page ([`DevMode::draw`]). An app launched
//! without [`ENV`] set to `1` has no dev mode: nothing is drawn and no
//! event is taken.

mod frame;
mod gpu;
mod inspect;
mod log;
mod memory;
mod paint;
mod plots;
#[cfg(test)]
mod tests;
mod zones;

use accord::{SemId, Snapshot};

use crate::drawing::compositor::{Compositor, LayerId};
use crate::runtime::perf::{FrameReport, Recorder, heap};
use crate::styling::theme::Theme;
use crate::widgets::prelude::{
    Button, ButtonSize, ButtonVariant, IconButton, IconButtonSize, Rect, WidgetEvent,
};
use paint::Painter;

/// The environment variable that gives a launch its dev mode.
pub const ENV: &str = "PLEV_DEV";

/// A view of the panel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Frame,
    Gpu,
    Zones,
    Plots,
    Memory,
    Log,
    Inspect,
}

/// The views there are.
const TABS: usize = 7;

impl Tab {
    pub const ALL: [Tab; TABS] = [
        Tab::Frame,
        Tab::Gpu,
        Tab::Zones,
        Tab::Plots,
        Tab::Memory,
        Tab::Log,
        Tab::Inspect,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Tab::Frame => "Frame",
            Tab::Gpu => "GPU",
            Tab::Zones => "Zones",
            Tab::Plots => "Plots",
            Tab::Memory => "Memory",
            Tab::Log => "Log",
            Tab::Inspect => "Inspect",
        }
    }

    /// The view named `name`, in any case.
    pub fn parse(name: &str) -> Option<Tab> {
        Tab::ALL
            .into_iter()
            .find(|tab| tab.name().eq_ignore_ascii_case(name))
    }
}

/// Where the panel sits in the window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Dock {
    #[default]
    Bottom,
    Side,
}

/// Which frame the views show.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Follow {
    /// The latest frame the app drew for itself.
    #[default]
    Latest,
    /// The slowest frame the recorder still holds.
    Slowest,
    /// One frame, chosen in the history.
    Frame(u64),
}

/// What a pointer event did here: `taken` keeps it from the page,
/// `changed` asks for a frame, `resized` gives the page a new size.
#[derive(Clone, Copy, Debug, Default)]
pub struct Took {
    pub taken: bool,
    pub changed: bool,
    pub resized: bool,
}

/// Where the Frame view drew its history, for a press to choose a frame.
pub(super) struct Strip {
    rect: Rect,
    /// The slot of the first frame: the history fills from the right.
    first: usize,
    /// The index of each frame drawn, left to right.
    frames: Vec<u64>,
}

/// The frames a chart covers, at most.
const FRAMES_MOST: usize = 120;

/// The frames a chart covers, oldest first: the latest the app drew for
/// itself, the dev mode's own left out.
pub(super) fn frames(recorder: &Recorder) -> Vec<&FrameReport> {
    let mut kept: Vec<&FrameReport> = recorder
        .history()
        .rev()
        .filter(|frame| !frame.tool)
        .take(FRAMES_MOST)
        .collect();
    kept.reverse();
    kept
}

/// The GPU's time over the frame, from the frame's own timestamps.
pub(super) fn gpu_span(frame: &FrameReport) -> Option<u64> {
    frame.gpu.as_ref().and_then(|gpu| gpu.span)
}

/// Where everything goes in a window `w` by `h`.
struct Layout {
    page: Rect,
    toggle: Rect,
    panel: Option<Rect>,
    body: Rect,
    tabs: [Rect; TABS],
    follow: Rect,
    dock: Rect,
    close: Rect,
}

fn small(label: &str, variant: ButtonVariant) -> Button {
    Button::new(label).size(ButtonSize::Sm).variant(variant)
}

fn tab_buttons(on: Tab) -> [Button; TABS] {
    Tab::ALL.map(|tab| {
        let variant = if tab == on {
            ButtonVariant::Secondary
        } else {
            ButtonVariant::Ghost
        };
        small(tab.name(), variant)
    })
}

pub struct DevMode {
    available: bool,
    open: bool,
    dock: Dock,
    tab: Tab,
    follow: Follow,
    toggle: Button,
    tabs: [Button; TABS],
    follow_button: Button,
    dock_button: Button,
    close: IconButton,
    /// Made in the window's compositor, kept above every layer of the frame.
    layer: Option<LayerId>,
    strip: Option<Strip>,
    /// The zone whose spread the Zones view shows; the heaviest of the
    /// frame while `None`.
    zone: Option<&'static str>,
    /// Where the Zones view drew its table, for a press to choose a zone.
    zone_rows: zones::Rows,
    inspect: inspect::Inspect,
    /// The latest frame of the app's own the panel has drawn.
    seen: Option<u64>,
}

impl DevMode {
    /// A dev mode that exists (`available`) or one that never draws. One
    /// that exists turns the heap's counters on for the process
    /// (`perf::heap`), and they stay on: a count that stopped with the
    /// panel would miss the blocks given back while it was closed.
    pub fn new(available: bool) -> Self {
        if available {
            heap::enable(true);
        }
        Self {
            available,
            open: false,
            dock: Dock::Bottom,
            tab: Tab::Frame,
            follow: Follow::Latest,
            toggle: small("Dev", ButtonVariant::Outline),
            tabs: tab_buttons(Tab::Frame),
            follow_button: small("Latest", ButtonVariant::Outline),
            dock_button: small("Side", ButtonVariant::Outline),
            close: IconButton::new("x").size(IconButtonSize::Sm),
            layer: None,
            strip: None,
            zone: None,
            zone_rows: zones::Rows::new(),
            inspect: inspect::Inspect::new(),
            seen: None,
        }
    }

    /// The dev mode of this launch: there with [`ENV`] set to `1`.
    pub fn from_env() -> Self {
        Self::new(std::env::var_os(ENV).is_some_and(|v| v == "1"))
    }

    pub fn is_available(&self) -> bool {
        self.available
    }

    /// The panel shows: the recorder it reads should be on.
    pub fn is_open(&self) -> bool {
        self.available && self.open
    }

    pub fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    pub fn tab(&self) -> Tab {
        self.tab
    }

    pub fn set_tab(&mut self, tab: Tab) {
        self.tab = tab;
        self.tabs = tab_buttons(tab);
    }

    pub fn set_dock(&mut self, dock: Dock) {
        self.dock = dock;
    }

    pub fn set_follow(&mut self, follow: Follow) {
        self.follow = follow;
    }

    /// Choose the zone whose spread the Zones view shows, as a press on
    /// its row would; `None` for the heaviest of the frame.
    pub fn set_zone(&mut self, zone: Option<&'static str>) {
        self.zone = zone;
    }

    /// Choose the control `id` in the Inspect view, as a press on it would.
    pub fn pick(&mut self, id: Option<SemId>) {
        self.inspect.pick(id);
    }

    /// The page's share of a window `w` by `h`: half of it below or
    /// beside the open panel, all of it otherwise.
    pub fn page_size(&self, w: f32, h: f32) -> (f32, f32) {
        match (self.is_open(), self.dock) {
            (false, _) => (w, h),
            (true, Dock::Bottom) => (w, (h / 2.0).round()),
            (true, Dock::Side) => ((w / 2.0).round(), h),
        }
    }

    /// The panel asks for one more frame: the app drew a frame the panel
    /// has not shown yet, or the one it shows still waits for its GPU
    /// timestamps, which a later frame reads back. The shell marks that
    /// frame as the tool's (`Recorder::mark_tool`), so it never asks for
    /// another on its own account.
    pub fn wants_frame(&self, recorder: &Recorder) -> bool {
        let latest = recorder.latest().map(|frame| frame.index);
        self.is_open() && (latest != self.seen || recorder.settling())
    }

    fn layout(&self, w: f32, h: f32, theme: &Theme) -> Layout {
        let (pw, ph) = self.page_size(w, h);
        let gap = theme.spacing.sm;
        let (tw, th) = self.toggle.preferred_size(theme);
        let toggle = Rect::new(
            ((pw - tw) / 2.0).round(),
            (ph - th - theme.spacing.lg).round(),
            tw,
            th,
        );
        let panel = self.is_open().then(|| match self.dock {
            Dock::Bottom => Rect::new(0.0, ph, w, h - ph),
            Dock::Side => Rect::new(pw, 0.0, w - pw, h),
        });
        let p = panel.unwrap_or_default();
        // the head: the views at the left and the controls at the right
        // on one line, or the views on a line of their own under the
        // controls when the panel is too narrow for both
        let button_h = small("Side", ButtonVariant::Outline)
            .preferred_size(theme)
            .1;
        let line_h = button_h + gap;
        let sizes = self
            .tabs
            .each_ref()
            .map(|button| button.preferred_size(theme));
        let close_size = self.close.preferred_size(theme);
        let dock_size = self.dock_button.preferred_size(theme);
        let follow_size = self.follow_button.preferred_size(theme);
        let tabs_w = sizes.iter().map(|size| size.0).sum::<f32>() + gap / 2.0 * (TABS - 1) as f32;
        let controls_w = follow_size.0 + dock_size.0 + close_size.0 + gap * 2.0;
        let stacked = tabs_w + gap + controls_w > p.w - gap * 2.0;
        let lines = if stacked { 2.0 } else { 1.0 };
        let head_h = gap + line_h * lines;
        let place = |size: (f32, f32), x: f32, line: f32| {
            let y = p.y + gap + line_h * line + (button_h - size.1) / 2.0;
            Rect::new(x, y, size.0, size.1)
        };
        let mut x = p.x + gap;
        let tabs = sizes.map(|size| {
            let r = place(size, x, lines - 1.0);
            x += r.w + gap / 2.0;
            r
        });
        let close = place(close_size, p.right() - gap - close_size.0, 0.0);
        let dock = place(dock_size, close.x - gap - dock_size.0, 0.0);
        let follow = place(follow_size, dock.x - gap - follow_size.0, 0.0);
        let body = Rect::new(
            p.x + gap,
            p.y + head_h,
            (p.w - gap * 2.0).max(0.0),
            (p.h - head_h - gap).max(0.0),
        );
        Layout {
            page: Rect::new(0.0, 0.0, pw, ph),
            toggle,
            panel,
            body,
            tabs,
            follow,
            dock,
            close,
        }
    }

    /// A pointer event in a window `w` by `h`, before the page gets it.
    /// `c` is the frame on screen, for the Inspect view to find the
    /// control under the pointer.
    pub fn event(
        &mut self,
        ev: &WidgetEvent,
        w: f32,
        h: f32,
        theme: &Theme,
        c: &Compositor,
    ) -> Took {
        let mut took = Took::default();
        if !self.available {
            return took;
        }
        let l = self.layout(w, h, theme);
        let (x, y) = match *ev {
            WidgetEvent::MouseMove { x, y }
            | WidgetEvent::MouseDown { x, y }
            | WidgetEvent::MouseUp { x, y }
            | WidgetEvent::Scroll { x, y, .. } => (x, y),
        };
        let r = self.toggle.handle_event(ev, l.toggle);
        took.changed |= r.changed || r.clicked;
        if r.clicked {
            self.open = !self.open;
            took.resized = true;
        }
        let mut inspected = false;
        if l.panel.is_some() && !took.resized {
            for (i, tab) in Tab::ALL.into_iter().enumerate() {
                let r = self.tabs[i].handle_event(ev, l.tabs[i]);
                took.changed |= r.changed || r.clicked;
                if r.clicked && tab != self.tab {
                    self.set_tab(tab);
                }
            }
            let r = self.follow_button.handle_event(ev, l.follow);
            took.changed |= r.changed || r.clicked;
            if r.clicked {
                self.follow = match self.follow {
                    Follow::Latest => Follow::Slowest,
                    Follow::Slowest | Follow::Frame(_) => Follow::Latest,
                };
            }
            let r = self.dock_button.handle_event(ev, l.dock);
            took.changed |= r.changed || r.clicked;
            if r.clicked {
                self.dock = match self.dock {
                    Dock::Bottom => Dock::Side,
                    Dock::Side => Dock::Bottom,
                };
                took.resized = true;
            }
            let r = self.close.handle_event(ev, l.close);
            took.changed |= r.changed || r.clicked;
            if r.clicked {
                self.open = false;
                took.resized = true;
            }
            match self.tab {
                Tab::Frame => {
                    if let (WidgetEvent::MouseDown { .. }, Some(strip)) = (ev, &self.strip)
                        && strip.rect.contains(x, y)
                        && let Some(index) = strip.frame_at(x)
                    {
                        self.follow = Follow::Frame(index);
                        took.changed = true;
                    }
                }
                Tab::Inspect => {
                    let (taken, changed) = self.inspect.event(ev, l.page, l.body, theme, c);
                    inspected = taken;
                    took.changed |= changed;
                }
                Tab::Zones => {
                    if let WidgetEvent::MouseDown { .. } = ev
                        && let Some(&(_, zone)) =
                            self.zone_rows.iter().find(|(row, _)| row.contains(x, y))
                    {
                        self.zone = Some(zone);
                        took.changed = true;
                    }
                }
                Tab::Gpu | Tab::Plots | Tab::Memory | Tab::Log => {}
            }
        }
        took.taken = took.resized
            || inspected
            || l.toggle.contains(x, y)
            || l.panel.is_some_and(|p| p.contains(x, y));
        took
    }

    /// Over the frame the app built, in a window `w` by `h`: the button,
    /// and the open panel with the view chosen. `snapshot` is what the
    /// page means now, for the Inspect view; `None` when it projects
    /// nothing.
    pub fn draw(
        &mut self,
        c: &mut Compositor,
        theme: &Theme,
        (w, h): (f32, f32),
        recorder: &Recorder,
        snapshot: Option<&Snapshot>,
    ) {
        if !self.available {
            return;
        }
        let top = c.layers().iter().filter(|l| Some(l.id) != self.layer);
        let z = top.map(|l| l.z_order).max().unwrap_or(0) + 1;
        let layer = match self.layer {
            Some(layer) => {
                c.set_layer_z_order(layer, z);
                layer
            }
            None => *self.layer.insert(c.create_layer(z)),
        };
        let l = self.layout(w, h, theme);
        self.seen = recorder.latest().map(|frame| frame.index);
        let shown = match self.follow {
            Follow::Latest => recorder.latest(),
            Follow::Slowest => recorder.slowest(),
            Follow::Frame(index) => recorder.get(index).or_else(|| recorder.latest()),
        };
        self.follow_button.label = match (self.follow, shown) {
            (Follow::Latest, _) => "Latest".to_owned(),
            (Follow::Slowest, _) => "Slowest".to_owned(),
            (Follow::Frame(_), Some(frame)) => format!("Frame {}", frame.index),
            (Follow::Frame(index), None) => format!("Frame {index}"),
        };
        self.dock_button.label = match self.dock {
            Dock::Bottom => "Side".to_owned(),
            Dock::Side => "Bottom".to_owned(),
        };
        let mut p = Painter { c, layer, theme };
        if let Some(panel) = l.panel {
            p.fill(panel, theme.colors.bg_panel.0);
            let edge = match self.dock {
                Dock::Bottom => Rect::new(panel.x, panel.y, panel.w, 1.0),
                Dock::Side => Rect::new(panel.x, panel.y, 1.0, panel.h),
            };
            p.fill(edge, theme.colors.divider.0);
            for (button, rect) in self.tabs.iter().zip(l.tabs) {
                button.render_to_layer(p.c, layer, rect, theme);
            }
            self.follow_button
                .render_to_layer(p.c, layer, l.follow, theme);
            self.dock_button.render_to_layer(p.c, layer, l.dock, theme);
            self.close.render_to_layer(p.c, layer, l.close, theme);
            match self.tab {
                Tab::Frame => self.strip = frame::draw(&mut p, l.body, recorder, shown),
                Tab::Gpu => gpu::draw(&mut p, l.body, recorder, shown, layer),
                Tab::Zones => {
                    self.zone_rows = zones::draw(&mut p, l.body, recorder, shown, self.zone);
                }
                Tab::Plots => plots::draw(&mut p, l.body, recorder, shown),
                Tab::Memory => memory::draw(&mut p, l.body, recorder, shown),
                Tab::Log => log::draw(&mut p, l.body, recorder, shown),
                Tab::Inspect => self.inspect.draw(&mut p, l.body, shown, snapshot, layer),
            }
        }
        self.toggle.render_to_layer(p.c, layer, l.toggle, theme);
    }
}

impl Strip {
    /// The frame whose bar is at `x`.
    fn frame_at(&self, x: f32) -> Option<u64> {
        let slots = self.first + self.frames.len();
        let slot = ((x - self.rect.x) / (self.rect.w / slots.max(1) as f32)) as usize;
        slot.checked_sub(self.first)
            .and_then(|i| self.frames.get(i))
            .copied()
    }
}
