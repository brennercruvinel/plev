//! The dev mode: the engine's own view of what it does, drawn over any
//! app. A small gear sits at the page's lower right corner; it opens a
//! panel docked in the window, below the page or beside it, and the page
//! gets the rest of the window ([`DevMode::page_size`]), so none of it
//! hides under the panel. The panel's edge drags to give it more or less
//! of the window.
//!
//! The panel is a tool's: a strip of small tabs, each with its icon, the
//! frame's vital numbers on one line under them whatever the tab, and
//! the view chosen. It reads the frames a [`Recorder`] keeps (the stages,
//! the zones, the layers and the GPU's passes of each one, what code said
//! and what the heap did) and shows one of them in seven views:
//!
//! - Frame: where the frame's time went, phase by phase, and what the
//!   numbers mean (`insights`): what is over the budget, what was drawn
//!   for nothing, what the heap and the log say.
//! - GPU: every pass with the layer it drew, the layers, the counters.
//! - Zones: the CPU's named zones on a time axis, their statistics, and
//!   how the runs of one zone spread over the frames kept.
//! - Plots: what each frame counted, and the values code gave, as charts.
//! - Memory: the heap block by block, and what the engine holds on the GPU.
//! - Log: the lines code said and the process logged, by level.
//! - Inspect: the page's controls as a tree, and one of them from its
//!   meaning to its cost; the pointer picks one on the page.
//!
//! Nothing here measures: the clocks are `perf::stages`, `perf::zones`
//! and `perf::gpu_time`, the counters `perf::marks` and `perf::heap`. The
//! shell that mounts it turns the recorder on while the panel is open,
//! hands it the pointer first ([`DevMode::event`]) and draws it after the
//! page ([`DevMode::draw`]). An app launched without [`ENV`] has no dev
//! mode: nothing is drawn and no event is taken.
//!
//! A view draws and, in the same pass, says where a press does something
//! (`Painter::region` with an [`Act`]); the next pointer event is read
//! against those places. So a view has no widget to keep in step with
//! what it drew, and the pointer over any of them is one comparison.

mod frame;
mod gpu;
mod head;
mod insights;
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
use crate::runtime::perf::{FrameReport, Level, Recorder, heap, marks};
use crate::styling::theme::Theme;
use crate::widgets::prelude::{Rect, WidgetEvent};
use paint::Painter;

/// The environment variable that gives a launch its dev mode: `1` for the
/// gear alone, a view's name (`gpu`, `zones`, ...) for the panel open on
/// that view.
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

    /// The icon beside the view's name.
    pub fn icon(self) -> &'static str {
        match self {
            Tab::Frame => "gauge",
            Tab::Gpu => "cpu",
            Tab::Zones => "flame",
            Tab::Plots => "chart-line",
            Tab::Memory => "memory-stick",
            Tab::Log => "terminal",
            Tab::Inspect => "scan",
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
    /// The latest frame the app drew for itself, once it is whole.
    #[default]
    Latest,
    /// The slowest frame the recorder still holds.
    Slowest,
    /// One frame, chosen in the history.
    Frame(u64),
}

/// What a press on a place of the panel does.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Act {
    /// The gear: open the panel.
    Open,
    Close,
    Tab(Tab),
    /// Arm the pointer to choose a control on the page.
    Pick,
    Follow(Follow),
    /// Move the panel to the other dock.
    Dock,
    /// The panel's edge: a drag gives it more or less of the window.
    Resize,
    /// A row of the Zones view's table.
    Zone(&'static str),
    /// A level of the Log view; `None` for every line.
    Level(Option<Level>),
    ClearLog,
    /// A control in the Inspect view's tree.
    Node(SemId),
}

/// What a pointer event did here: `taken` keeps it from the page,
/// `changed` asks for a frame, `resized` gives the page a new size.
#[derive(Clone, Copy, Debug, Default)]
pub struct Took {
    pub taken: bool,
    pub changed: bool,
    pub resized: bool,
}

/// The frames a chart covers, at most.
const FRAMES_MOST: usize = 120;

/// The least and the most of the window the panel takes.
const SHARE_LEAST: f32 = 0.2;
const SHARE_MOST: f32 = 0.8;

/// The gear's side, and how far it sits from the page's corner.
const GEAR: f32 = 28.0;
const GEAR_MARGIN: f32 = 14.0;
/// How far either side of the panel's edge a drag takes hold.
const GRIP: f32 = 3.0;

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

/// A frame's whole time: its CPU phases and the GPU's span.
pub(super) fn whole(frame: &FrameReport) -> u64 {
    frame.pipeline.cpu_nanos + gpu_span(frame).unwrap_or(0)
}

/// Where the page, the gear and the panel go in a window.
struct Layout {
    page: Rect,
    gear: Rect,
    panel: Option<Rect>,
}

pub struct DevMode {
    available: bool,
    open: bool,
    dock: Dock,
    tab: Tab,
    follow: Follow,
    /// The panel's share of the window, along the side it is docked on.
    share: f32,
    /// Made in the window's compositor, kept above every layer of the frame.
    layer: Option<LayerId>,
    /// Where the last frame drawn offered a press, in draw order: the
    /// later one is on top.
    regions: Vec<(Rect, Act)>,
    /// The place the pointer is over.
    hot: Option<Act>,
    /// The panel's edge is held.
    dragging: bool,
    /// The release of a press taken here is not the page's either.
    held: bool,
    /// The zone whose spread the Zones view shows; the heaviest of the
    /// frame while `None`.
    zone: Option<&'static str>,
    log: log::LogView,
    inspect: inspect::Inspect,
    /// The frame the panel last drew as the latest, and whether it had
    /// its GPU time then.
    seen: Option<(u64, bool)>,
}

impl DevMode {
    /// A dev mode that exists (`available`) or one that never draws. One
    /// that exists turns the heap's counters on for the process
    /// (`perf::heap`) and holds what code says (`perf::marks`), and they
    /// stay on: a count that stopped with the panel would miss the blocks
    /// given back while it was closed, and the log would miss what was
    /// said then.
    pub fn new(available: bool) -> Self {
        if available {
            heap::enable(true);
            marks::hold(true);
        }
        Self {
            available,
            open: false,
            dock: Dock::Bottom,
            tab: Tab::Frame,
            follow: Follow::Latest,
            share: 0.5,
            layer: None,
            regions: Vec::new(),
            hot: None,
            dragging: false,
            held: false,
            zone: None,
            log: log::LogView::new(),
            inspect: inspect::Inspect::new(),
            seen: None,
        }
    }

    /// The dev mode of this launch: there with [`ENV`] set to `1`, and
    /// open on a view with [`ENV`] set to the view's name.
    pub fn from_env() -> Self {
        let value = std::env::var(ENV).unwrap_or_default();
        let tab = Tab::parse(&value);
        let mut dev = Self::new(value == "1" || tab.is_some());
        if let Some(tab) = tab {
            dev.open = true;
            dev.tab = tab;
        }
        dev
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
    }

    pub fn dock(&self) -> Dock {
        self.dock
    }

    pub fn set_dock(&mut self, dock: Dock) {
        self.dock = dock;
    }

    pub fn follow(&self) -> Follow {
        self.follow
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

    /// The control the Inspect view shows.
    pub fn picked(&self) -> Option<&SemId> {
        self.inspect.picked()
    }

    /// The page's share of a window `w` by `h`: what the open panel
    /// leaves below or beside it, all of it otherwise.
    pub fn page_size(&self, w: f32, h: f32) -> (f32, f32) {
        match (self.is_open(), self.dock) {
            (false, _) => (w, h),
            (true, Dock::Bottom) => (w, (h * (1.0 - self.share)).round()),
            (true, Dock::Side) => ((w * (1.0 - self.share)).round(), h),
        }
    }

    /// Where the gear is in a window `w` by `h`, while the panel is
    /// closed: a press there opens it.
    pub fn gear(&self, w: f32, h: f32) -> Rect {
        self.layout(w, h).gear
    }

    /// The panel asks for one more frame: the frame it would show as the
    /// latest is not the one it drew, or the latest still waits for its
    /// GPU timestamps, which a later frame reads back. The shell marks
    /// that frame as the tool's (`Recorder::mark_tool`), so it never asks
    /// for another on its own account.
    pub fn wants_frame(&self, recorder: &Recorder) -> bool {
        self.is_open() && (whole_key(recorder) != self.seen || recorder.settling())
    }

    fn layout(&self, w: f32, h: f32) -> Layout {
        let (pw, ph) = self.page_size(w, h);
        let gear = Rect::new(
            (pw - GEAR - GEAR_MARGIN).round(),
            (ph - GEAR - GEAR_MARGIN).round(),
            GEAR,
            GEAR,
        );
        let panel = self.is_open().then(|| match self.dock {
            Dock::Bottom => Rect::new(0.0, ph, w, h - ph),
            Dock::Side => Rect::new(pw, 0.0, w - pw, h),
        });
        Layout {
            page: Rect::new(0.0, 0.0, pw, ph),
            gear,
            panel,
        }
    }

    /// A pointer event in a window `w` by `h`, before the page gets it.
    /// `c` is the frame on screen, for the Inspect view to find the
    /// control under the pointer.
    pub fn event(&mut self, ev: &WidgetEvent, w: f32, h: f32, c: &Compositor) -> Took {
        let mut took = Took::default();
        if !self.available {
            return took;
        }
        let l = self.layout(w, h);
        let (x, y) = ev.pos();
        let inside = l.panel.is_some_and(|p| p.contains(x, y));
        let over = self
            .regions
            .iter()
            .rev()
            .find(|(r, _)| r.contains(x, y))
            .map(|(_, act)| act.clone());
        let picking = self.inspect.is_armed() && over.is_none() && l.page.contains(x, y);
        match *ev {
            WidgetEvent::MouseMove { .. } if self.dragging => {
                let share = match self.dock {
                    Dock::Bottom => 1.0 - y / h.max(1.0),
                    Dock::Side => 1.0 - x / w.max(1.0),
                };
                let share = share.clamp(SHARE_LEAST, SHARE_MOST);
                if share != self.share {
                    self.share = share;
                    took.resized = true;
                    took.changed = true;
                }
                took.taken = true;
            }
            WidgetEvent::MouseMove { .. } => {
                if over != self.hot {
                    self.hot = over.clone();
                    took.changed = true;
                }
                if picking {
                    took.changed |= self.inspect.hover(c.hit(x, y).cloned());
                } else {
                    took.changed |= self.inspect.hover(None);
                }
            }
            WidgetEvent::MouseDown { .. } => {
                if let Some(act) = over.clone() {
                    self.apply(act, &mut took);
                    took.changed = true;
                    self.held = true;
                } else if picking {
                    self.inspect.pick(c.hit(x, y).cloned());
                    self.tab = Tab::Inspect;
                    took.changed = true;
                    self.held = true;
                } else if inside {
                    self.held = true;
                }
            }
            WidgetEvent::MouseUp { .. } => {
                took.taken = std::mem::take(&mut self.held) || self.dragging;
                self.dragging = false;
            }
            WidgetEvent::Scroll { delta, .. } if inside => {
                took.changed = match self.tab {
                    Tab::Log => self.log.scroll(delta),
                    Tab::Inspect => self.inspect.scroll(delta),
                    _ => false,
                };
            }
            WidgetEvent::Scroll { .. } => {}
        }
        took.taken |= took.resized || inside || over.is_some() || picking || self.held;
        took
    }

    fn apply(&mut self, act: Act, took: &mut Took) {
        match act {
            Act::Open => {
                self.open = true;
                took.resized = true;
            }
            Act::Close => {
                self.open = false;
                self.inspect.arm(false);
                took.resized = true;
            }
            Act::Tab(tab) => self.tab = tab,
            Act::Pick => {
                let arm = !self.inspect.is_armed();
                self.inspect.arm(arm);
                if arm {
                    self.tab = Tab::Inspect;
                }
            }
            Act::Follow(follow) => self.follow = follow,
            Act::Dock => {
                self.dock = match self.dock {
                    Dock::Bottom => Dock::Side,
                    Dock::Side => Dock::Bottom,
                };
                took.resized = true;
            }
            Act::Resize => self.dragging = true,
            Act::Zone(zone) => self.zone = Some(zone),
            Act::Level(level) => self.log.filter(level),
            Act::ClearLog => self.log.clear(),
            Act::Node(id) => self.inspect.pick(Some(id)),
        }
    }

    /// Over the frame the app built, in a window `w` by `h`: the gear,
    /// or the open panel with the view chosen. `snapshot` is what the
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
        let l = self.layout(w, h);
        self.seen = whole_key(recorder);
        let shown = match self.follow {
            Follow::Latest => recorder.settled(),
            Follow::Slowest => recorder.slowest(),
            Follow::Frame(index) => recorder.get(index).or_else(|| recorder.settled()),
        };
        let mut regions = std::mem::take(&mut self.regions);
        regions.clear();
        let mut p = Painter {
            c,
            layer,
            theme,
            regions: &mut regions,
            hot: self.hot.clone(),
        };
        match l.panel {
            None => gear(&mut p, l.gear),
            Some(panel) => {
                p.fill(panel, theme.colors.bg.0);
                let (edge, grip) = match self.dock {
                    Dock::Bottom => (
                        Rect::new(panel.x, panel.y, panel.w, 1.0),
                        Rect::new(panel.x, panel.y - GRIP, panel.w, GRIP * 2.0),
                    ),
                    Dock::Side => (
                        Rect::new(panel.x, panel.y, 1.0, panel.h),
                        Rect::new(panel.x - GRIP, panel.y, GRIP * 2.0, panel.h),
                    ),
                };
                let held = p.region(grip, Act::Resize) || self.dragging;
                let state = head::State {
                    tab: self.tab,
                    follow: self.follow,
                    dock: self.dock,
                    picking: self.inspect.is_armed(),
                };
                let body = head::draw(&mut p, panel, &state, recorder, shown);
                match self.tab {
                    Tab::Frame => frame::draw(&mut p, body, recorder, shown),
                    Tab::Gpu => gpu::draw(&mut p, body, recorder, shown),
                    Tab::Zones => zones::draw(&mut p, body, recorder, shown, self.zone),
                    Tab::Plots => plots::draw(&mut p, body, recorder, shown),
                    Tab::Memory => memory::draw(&mut p, body, recorder, shown),
                    Tab::Log => self.log.draw(&mut p, body, recorder, shown),
                    Tab::Inspect => self.inspect.draw(&mut p, body, shown, snapshot),
                }
                if self.tab == Tab::Inspect || self.inspect.is_armed() {
                    self.inspect.overlay(&mut p, l.page, snapshot);
                }
                // the edge last, over the head's ground: the line the
                // panel ends at, thicker while it is held
                let color = if held {
                    theme.colors.info.0
                } else {
                    theme.colors.divider.0
                };
                let line = if held {
                    edge.outset_along(self.dock)
                } else {
                    edge
                };
                p.fill(line, color);
            }
        }
        self.regions = regions;
    }
}

/// The frame a panel that follows the app shows, and whether its GPU
/// time is in: what [`DevMode::wants_frame`] compares.
fn whole_key(recorder: &Recorder) -> Option<(u64, bool)> {
    recorder
        .settled()
        .map(|frame| (frame.index, frame.gpu.is_some()))
}

/// The gear that opens the panel: a small round button on the page.
fn gear(p: &mut Painter<'_>, at: Rect) {
    let hot = p.region(at, Act::Open);
    let colors = &p.theme.colors;
    let (ground, ink) = if hot {
        (colors.bg_hover.0, colors.text.0)
    } else {
        (colors.bg.0, colors.text_mid.0)
    };
    p.pill(at.outset(1.0), colors.divider.0);
    p.pill(at, ground);
    p.icon_in("settings", at, paint::ICON, ink);
}

/// The panel's edge while it is held: two more pixels into the panel.
trait Thicker {
    fn outset_along(self, dock: Dock) -> Rect;
}

impl Thicker for Rect {
    fn outset_along(self, dock: Dock) -> Rect {
        match dock {
            Dock::Bottom => Rect::new(self.x, self.y, self.w, self.h + 1.0),
            Dock::Side => Rect::new(self.x, self.y, self.w + 1.0, self.h),
        }
    }
}
