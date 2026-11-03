//! The profiler (task-105, a POC behind the `profiler` feature): two
//! small buttons over any page of the main window, Puffin and Tracy. Each
//! opens its part of a panel docked in the window, below the page or
//! beside it (its "Side" and "Bottom" move it), the two parts side by
//! side when both are open. The page gets the rest of the window
//! ([`Profiler::page_size`], which `App::viewport_changed` hands the
//! view), so none of it hides under the panel.
//!
//! Puffin's part draws the frames the engine's scopes and wgpu-profiler's
//! GPU passes recorded (`flame.rs`); Tracy's says whether the Tracy GUI
//! is reading the gallery and opens it (`tracy.rs`): Tracy's analysis is
//! Tracy's own window. Built without the feature it draws nothing, takes
//! no event and leaves the page the whole window.

#[cfg(feature = "profiler")]
mod flame;
mod tracy;

use std::time::Duration;

use engine::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::drawing::text::{TextMeasurer, TextStyle};
use engine::styling::theme::{Theme, TypeRole};
use engine::widgets::prelude::{
    Button, ButtonSize, ButtonVariant, IconButton, IconButtonSize, Rect, Wake, WidgetEvent,
    rounded_rect,
};
use winit::window::WindowId;

use super::super::App;

/// The profiler exists in this build.
const ENABLED: bool = cfg!(feature = "profiler");
const TITLES: [&str; 2] = ["Puffin", "Tracy"];

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Dock {
    #[default]
    Bottom,
    Side,
}

pub(crate) struct Profiler {
    open: [bool; 2],
    dock: Dock,
    /// The two buttons over the page.
    toggles: [Button; 2],
    /// Each part's header: the dock's switch, and its close.
    docks: [Button; 2],
    closes: [IconButton; 2],
    /// Made in the main window, kept above every layer of the frame.
    layer: Option<LayerId>,
    #[cfg(feature = "profiler")]
    frames: Option<flame::Recorder>,
    tracy: tracy::Tracy,
}

/// What a pointer event did here: `taken` keeps it from the page,
/// `changed` asks for a frame, `resized` gives the page a new size.
#[derive(Clone, Copy, Default)]
pub(crate) struct Took {
    pub taken: bool,
    pub changed: bool,
    pub resized: bool,
}

/// Where everything goes in a window `w` by `h`.
struct Layout {
    toggles: [Rect; 2],
    panel: Option<Rect>,
    /// Each open part: its header, then its body.
    parts: [Option<(Rect, Rect)>; 2],
}

fn small(label: &str) -> Button {
    Button::new(label)
        .size(ButtonSize::Sm)
        .variant(ButtonVariant::Outline)
}

impl Default for Profiler {
    fn default() -> Self {
        Self {
            open: [false; 2],
            dock: Dock::Bottom,
            toggles: TITLES.map(small),
            docks: [small("Side"), small("Side")],
            closes: std::array::from_fn(|_| IconButton::new("x").size(IconButtonSize::Sm)),
            layer: None,
            #[cfg(feature = "profiler")]
            frames: None,
            tracy: tracy::Tracy::default(),
        }
    }
}

impl Profiler {
    /// The page's share of a window `w` by `h`: half of it below or
    /// beside an open panel, all of it otherwise.
    pub(crate) fn page_size(&self, w: f32, h: f32) -> (f32, f32) {
        match (self.open.contains(&true), self.dock) {
            (false, _) => (w, h),
            (true, Dock::Bottom) => (w, (h / 2.0).round()),
            (true, Dock::Side) => ((w / 2.0).round(), h),
        }
    }

    /// Puffin's frames come in while its part is open: the part follows
    /// them four times a second. Tracy's says the GUI's state each second.
    pub(crate) fn wake(&self) -> Wake {
        match self.open {
            [true, _] => Wake::after(Duration::from_millis(250)),
            [false, true] => Wake::after(Duration::from_secs(1)),
            _ => Wake::idle(),
        }
    }

    fn header_h(theme: &Theme) -> f32 {
        small("Side").preferred_size(theme).1 + theme.spacing.sm * 2.0
    }

    fn layout(&self, w: f32, h: f32, theme: &Theme) -> Layout {
        let (pw, ph) = self.page_size(w, h);
        let gap = theme.spacing.sm;
        let sizes = self.toggles.each_ref().map(|b| b.preferred_size(theme));
        let mut x = ((pw - sizes[0].0 - gap - sizes[1].0) / 2.0).round();
        let y = (ph - sizes[0].1 - theme.spacing.lg).round();
        let toggles = sizes.map(|(bw, bh)| {
            let r = Rect::new(x, y, bw, bh);
            x += bw + gap;
            r
        });
        let panel = self.open.contains(&true).then(|| match self.dock {
            Dock::Bottom => Rect::new(0.0, ph, w, h - ph),
            Dock::Side => Rect::new(pw, 0.0, w - pw, h),
        });
        let mut parts = [None; 2];
        if let Some(p) = panel {
            let both = self.open == [true; 2];
            let split = |i: usize| match (both, self.dock) {
                (false, _) => p,
                (true, Dock::Bottom) => Rect::new(p.x + p.w / 2.0 * i as f32, p.y, p.w / 2.0, p.h),
                (true, Dock::Side) => Rect::new(p.x, p.y + p.h / 2.0 * i as f32, p.w, p.h / 2.0),
            };
            let head = Self::header_h(theme);
            for (i, part) in parts.iter_mut().enumerate() {
                if self.open[i] {
                    let r = split(i);
                    let body = Rect::new(r.x + gap, r.y + head, r.w - gap * 2.0, r.h - head - gap);
                    *part = Some((Rect::new(r.x, r.y, r.w, head), body));
                }
            }
        }
        Layout {
            toggles,
            panel,
            parts,
        }
    }

    /// The header's controls: the dock's switch, then the close, at the
    /// header's right.
    fn header_controls(&self, i: usize, head: Rect, theme: &Theme) -> (Rect, Rect) {
        let gap = theme.spacing.sm;
        let (cw, ch) = self.closes[i].preferred_size(theme);
        let close = Rect::new(
            head.right() - gap - cw,
            head.y + (head.h - ch) / 2.0,
            cw,
            ch,
        );
        let (dw, dh) = self.docks[i].preferred_size(theme);
        let dock = Rect::new(close.x - gap - dw, head.y + (head.h - dh) / 2.0, dw, dh);
        (dock, close)
    }

    /// A pointer event in the main window, `w` by `h`.
    pub(crate) fn event(&mut self, ev: &WidgetEvent, w: f32, h: f32, theme: &Theme) -> Took {
        let mut took = Took::default();
        if !ENABLED {
            return took;
        }
        let l = self.layout(w, h, theme);
        for i in 0..2 {
            let r = self.toggles[i].handle_event(ev, l.toggles[i]);
            took.changed |= r.changed || r.clicked;
            if r.clicked {
                self.set_open(i, !self.open[i]);
                took.resized = true;
            }
            let Some((head, body)) = l.parts[i] else {
                continue;
            };
            let (dock, close) = self.header_controls(i, head, theme);
            let r = self.docks[i].handle_event(ev, dock);
            took.changed |= r.changed || r.clicked;
            if r.clicked {
                self.dock = match self.dock {
                    Dock::Bottom => Dock::Side,
                    Dock::Side => Dock::Bottom,
                };
                let label = if self.dock == Dock::Side {
                    "Bottom"
                } else {
                    "Side"
                };
                self.docks
                    .iter_mut()
                    .for_each(|b| b.label = label.to_owned());
                took.resized = true;
            }
            let r = self.closes[i].handle_event(ev, close);
            took.changed |= r.changed || r.clicked;
            if r.clicked {
                self.set_open(i, false);
                took.resized = true;
            }
            if i == 1 {
                took.changed |= self.tracy.event(ev, body, theme);
            }
        }
        let (x, y) = match *ev {
            WidgetEvent::MouseMove { x, y }
            | WidgetEvent::MouseDown { x, y }
            | WidgetEvent::MouseUp { x, y }
            | WidgetEvent::Scroll { x, y, .. } => (x, y),
        };
        took.taken = took.resized
            || l.toggles.iter().any(|r| r.contains(x, y))
            || l.panel.is_some_and(|p| p.contains(x, y));
        took
    }

    fn set_open(&mut self, i: usize, open: bool) {
        self.open[i] = open;
        #[cfg(feature = "profiler")]
        if i == 0 {
            self.frames = open.then(flame::Recorder::start);
        }
    }

    /// Over the frame the view built, in the main window `w` by `h`.
    pub(crate) fn draw(&mut self, c: &mut Compositor, theme: &Theme, (w, h): (f32, f32)) {
        if !ENABLED {
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
        if let Some(p) = l.panel {
            c.push_to_layer(
                layer,
                rounded_rect(p.x, p.y, p.w, p.h, 0.0, theme.colors.bg_panel.0),
            );
            let edge = match self.dock {
                Dock::Bottom => Rect::new(p.x, p.y, p.w, 1.0),
                Dock::Side => Rect::new(p.x, p.y, 1.0, p.h),
            };
            let divider = theme.colors.divider.0;
            c.push_to_layer(
                layer,
                rounded_rect(edge.x, edge.y, edge.w, edge.h, 0.0, divider),
            );
        }
        for (i, part) in l.parts.iter().enumerate() {
            let Some((head, body)) = *part else {
                continue;
            };
            if i == 1 && self.open[0] {
                let edge = match self.dock {
                    Dock::Bottom => Rect::new(head.x, head.y, 1.0, p_h(&l)),
                    Dock::Side => Rect::new(head.x, head.y, head.w, 1.0),
                };
                let divider = theme.colors.divider.0;
                c.push_to_layer(
                    layer,
                    rounded_rect(edge.x, edge.y, edge.w, edge.h, 0.0, divider),
                );
            }
            let (dock, close) = self.header_controls(i, head, theme);
            let title = self.title(i);
            let room = dock.x - head.x - theme.spacing.sm * 2.0;
            let y = head.y + (head.h - title_style(theme).line_height) / 2.0;
            text(
                c,
                layer,
                &title,
                head.x + theme.spacing.sm,
                y,
                room,
                theme.colors.text.0,
                &title_style(theme),
            );
            self.docks[i].render_to_layer(c, layer, dock, theme);
            self.closes[i].render_to_layer(c, layer, close, theme);
            match i {
                #[cfg(feature = "profiler")]
                0 => {
                    if let Some(frames) = &self.frames {
                        frames.draw(c, layer, body, theme);
                    }
                }
                1 => self.tracy.draw(c, layer, body, theme),
                _ => {}
            }
        }
        for (b, r) in self.toggles.iter().zip(l.toggles) {
            b.render_to_layer(c, layer, r, theme);
        }
    }

    fn title(&self, i: usize) -> String {
        #[cfg(feature = "profiler")]
        if let (0, Some(frames)) = (i, &self.frames) {
            return frames.title();
        }
        TITLES[i].to_owned()
    }
}

/// The panel's height, for the line between the parts side by side.
fn p_h(l: &Layout) -> f32 {
    l.panel.map_or(0.0, |p| p.h)
}

fn title_style(theme: &Theme) -> TextStyle {
    theme.typography.run(TypeRole::Caption.strong())
}

pub(super) fn caption_style(theme: &Theme) -> TextStyle {
    theme.typography.run(TypeRole::Caption.regular())
}

/// `s` on one line at `(x, y)` in `layer`, cut to `room`.
#[allow(clippy::too_many_arguments)] // a text's where, how wide, how
pub(super) fn text(
    c: &mut Compositor,
    layer: LayerId,
    s: &str,
    x: f32,
    y: f32,
    room: f32,
    color: [f32; 4],
    style: &TextStyle,
) {
    if room <= 0.0 {
        return;
    }
    let s = TextMeasurer::truncate_to_width(s, style, room);
    let key = TextNodeKey::from_style(&s, style, None);
    c.push_to_layer(layer, SceneNode::Text { key, x, y, color });
}

impl App {
    /// A pointer event in window `id` for the profiler first; `Some`
    /// with whether a frame is needed when it kept the event from the
    /// page. A panel that opened, closed or moved resizes the page.
    pub(in crate::shell) fn profiler_event(
        &mut self,
        id: WindowId,
        ev: &WidgetEvent,
    ) -> Option<bool> {
        if !ENABLED || Some(id) != self.main {
            return None;
        }
        let (w, h) = self.windows.get(id)?.logical_size();
        let took = self.profiler.event(ev, w, h, &self.view.theme);
        if took.resized {
            self.viewport_changed(id);
        }
        if took.changed {
            self.invalidate_in(id);
        }
        took.taken.then_some(took.changed)
    }
}
