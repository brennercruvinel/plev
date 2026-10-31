//! The kit's toolbar (`components/Toolbar`): a pill of tool buttons, the
//! zoom control that opens the [`Zoom`] panel, the history tools, and the
//! export button after a divider. One tool is the active one when the
//! bar is selectable (edit mode); in play mode the tools are commands,
//! with a divider after the ones that leave it. Hovering a tool shows its
//! tooltip with the key that runs it (`feedback::Tooltip`).
//!
//! Every value is `theme.navigation.tool.toolbar`. The parts the kit
//! draws in the same page are here too: [`VideoPlayer`] and
//! [`ViewController`].

mod player;
mod view;
mod zoom;

pub use player::{PlayerLayout, VideoPlayer};
pub use view::ViewController;
pub use zoom::{Zoom, ZoomLine};

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::Theme;
use crate::widgets::buttons::{Button, Press};
use crate::widgets::feedback::Tooltip;
use crate::widgets::icons;
use crate::widgets::recipe::{rounded_rect, rounded_rect_stroke, shadow_drops, shadow_insets};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

use super::tokens::ToolbarRecipe;

/// One tool button (`Toolbar/Action`): an icon, the name and key its
/// tooltip shows.
#[derive(Clone, Debug)]
pub struct Tool {
    pub icon: &'static str,
    pub label: String,
    pub key: String,
    pub disabled: bool,
    press: Press,
    sem: Option<SemId>,
}

impl Tool {
    pub fn new(icon: &'static str, label: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            icon,
            label: label.into(),
            key: key.into(),
            disabled: false,
            press: Press::default(),
            sem: None,
        }
    }

    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        self
    }

    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// The box, the well while `active`, the wash while hovered, the
    /// icon; all faded while disabled.
    pub fn render_to_layer(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        rect: Rect,
        active: bool,
        theme: &Theme,
    ) {
        if let Some(id) = &self.sem {
            c.bind_to_layer(layer, id, [rect.x, rect.y, rect.w, rect.h]);
        }
        let r = &theme.navigation.tool.toolbar.tool;
        let fade = |col: [f32; 4]| {
            let a = if self.disabled { r.disabled_alpha } else { 1.0 };
            [col[0], col[1], col[2], col[3] * a]
        };
        if active {
            shadow_drops(c, layer, rect, r.radius, &r.active_shadow);
            c.push_to_layer(
                layer,
                rounded_rect(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    r.radius,
                    fade(r.active_fill.0),
                ),
            );
            shadow_insets(c, layer, rect, r.radius, &r.active_shadow);
            c.push_to_layer(
                layer,
                rounded_rect_stroke(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    r.radius,
                    fade(r.active_border.0),
                    r.border_width,
                ),
            );
        } else if self.press.hovered || self.press.pressed {
            c.push_to_layer(
                layer,
                rounded_rect(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    r.radius,
                    fade(r.hover_fill.0),
                ),
            );
        }
        let (x, y) = (
            rect.x + (rect.w - r.icon) / 2.0,
            rect.y + (rect.h - r.icon) / 2.0,
        );
        if let Some(node) = icons::icon_at(self.icon, r.icon, fade(r.icon_color.0), x, y) {
            c.push_to_layer(layer, node);
        }
    }

    /// A `Role::Button` named by its label, its key as the description,
    /// `active` as its state when the bar selects.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
        active: Option<bool>,
    ) -> NodeBuilder<'b> {
        let mut node = b.node(id, Role::Button, self.label.as_str());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.description(self.key.as_str());
        node.enabled(!self.disabled);
        if let Some(on) = active {
            node.state("active", on);
        }
        node.action("activate");
        node
    }
}

/// Where each part of a toolbar lands.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolbarLayout {
    pub tools: Vec<Rect>,
    pub divider: Option<Rect>,
    pub zoom: Rect,
    pub history: Vec<Rect>,
    /// The vertical rule before the export side.
    pub split: Rect,
    pub export: Rect,
}

/// The toolbar of the kit.
#[derive(Clone, Debug)]
pub struct Toolbar {
    pub tools: Vec<Tool>,
    /// The tools after the zoom control (undo, redo).
    pub history: Vec<Tool>,
    /// The active tool, when the bar selects one.
    pub active: Option<usize>,
    pub selectable: bool,
    /// A divider after this many tools (play mode).
    pub divider_after: Option<usize>,
    pub zoom: Zoom,
    pub open: bool,
    pub export: Button,
    zoom_press: Press,
    zoom_sem: Option<SemId>,
    tooltip: Option<Tooltip>,
}

impl Toolbar {
    pub fn new(tools: Vec<Tool>, zoom: Zoom) -> Self {
        Self {
            tools,
            history: Vec::new(),
            active: None,
            selectable: false,
            divider_after: None,
            zoom,
            open: false,
            export: Button::new("Export"),
            zoom_press: Press::default(),
            zoom_sem: None,
            tooltip: None,
        }
    }

    /// The tools select: `active` is the one on.
    pub fn selecting(mut self, active: usize) -> Self {
        self.selectable = true;
        self.active = Some(active);
        self
    }

    pub fn history(mut self, tools: Vec<Tool>) -> Self {
        self.history = tools;
        self
    }

    pub fn divider_after(mut self, n: usize) -> Self {
        self.divider_after = Some(n);
        self
    }

    pub fn export(mut self, button: Button) -> Self {
        self.export = button;
        self
    }

    /// The identity the zoom control binds and projects.
    pub fn zoom_sem(mut self, id: SemId) -> Self {
        self.zoom_sem = Some(id);
        self
    }

    pub fn zoom_sem_id(&self) -> Option<&SemId> {
        self.zoom_sem.as_ref()
    }

    fn recipe(theme: &Theme) -> &ToolbarRecipe {
        &theme.navigation.tool.toolbar
    }

    /// The zoom control's width: its value and chevron, never under the
    /// control's least width.
    fn zoom_w(&self, theme: &Theme) -> f32 {
        let r = Self::recipe(theme);
        let text = r.zoom_text.style(&theme.typography);
        let tw = TextMeasurer::measure_styled(&self.zoom.level, &text, None).0;
        (r.zoom_pad_x * 2.0 + tw + r.zoom_gap + r.chevron)
            .ceil()
            .max(r.zoom_min_w)
    }

    fn divider_w(&self, theme: &Theme) -> f32 {
        if self.divider_after.is_some() {
            Self::recipe(theme).border_width
        } else {
            0.0
        }
    }

    /// The bar's size: the tools, the zoom control and the history in a
    /// row, then the export side.
    pub fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        let r = Self::recipe(theme);
        let n =
            self.tools.len() + self.history.len() + 1 + usize::from(self.divider_after.is_some());
        let row = (self.tools.len() + self.history.len()) as f32 * r.tool.size
            + self.zoom_w(theme)
            + self.divider_w(theme)
            + n.saturating_sub(1) as f32 * r.gap;
        let w = r.pad * 2.0 + row + r.border_width + r.pad * 2.0 + r.export_w;
        (w, r.pad * 2.0 + r.tool.size)
    }

    /// Every part's rect in a bar drawn at `bounds`' top-left.
    pub fn layout(&self, bounds: Rect, theme: &Theme) -> ToolbarLayout {
        let r = Self::recipe(theme);
        let (w, h) = self.preferred_size(theme);
        let y = bounds.y + r.pad;
        let mut x = bounds.x + r.pad;
        let mut place = |w: f32| {
            let at = Rect::new(x, y, w, r.tool.size);
            x += w + r.gap;
            at
        };
        let mut tools = Vec::with_capacity(self.tools.len());
        let mut divider = None;
        for i in 0..self.tools.len() {
            tools.push(place(r.tool.size));
            if self.divider_after == Some(i + 1) {
                let at = place(r.border_width);
                divider = Some(Rect::new(at.x, bounds.y, at.w, h));
            }
        }
        let zoom = place(self.zoom_w(theme));
        let history = self.history.iter().map(|_| place(r.tool.size)).collect();
        let export_x = bounds.x + w - r.pad - r.export_w;
        let split = Rect::new(
            export_x - r.pad - r.border_width,
            bounds.y,
            r.border_width,
            h,
        );
        let export_h = self.export.preferred_size(theme).1;
        let export = Rect::new(
            export_x,
            bounds.y + (h - export_h) / 2.0,
            r.export_w,
            export_h,
        );
        ToolbarLayout {
            tools,
            divider,
            zoom,
            history,
            split,
            export,
        }
    }

    /// The zoom panel's rect: under the control, its end on the
    /// control's end.
    pub fn panel_rect(&self, bounds: Rect, theme: &Theme) -> Rect {
        let r = Self::recipe(theme);
        let zoom = self.layout(bounds, theme).zoom;
        let (w, h) = self.zoom.size(theme);
        Rect::new(zoom.x + zoom.w - w, zoom.y + zoom.h + r.panel_gap, w, h)
    }

    /// Hover, press and click on every part; the open panel takes what
    /// lands in it, a click elsewhere closes it.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let at = self.layout(bounds, theme);
        let mut out = EventResult::IGNORED;
        if self.open {
            let panel = self.panel_rect(bounds, theme);
            let r = self.zoom.handle_event(event, panel, theme);
            if r.handled || r.changed {
                return r;
            }
            if let WidgetEvent::MouseDown { x, y } = *event
                && !panel.contains(x, y)
                && !at.zoom.contains(x, y)
            {
                self.open = false;
                out = EventResult::changed();
            }
        }
        let mut hovered = None;
        for (i, (tool, rect)) in self.tools.iter_mut().zip(&at.tools).enumerate() {
            let r = tool.press.handle(event, *rect, tool.disabled);
            if r.clicked && self.selectable {
                self.active = Some(i);
            }
            if tool.press.hovered {
                hovered = Some((tool.label.clone(), tool.key.clone(), *rect));
            }
            out = merge(out, r);
        }
        for (tool, rect) in self.history.iter_mut().zip(&at.history) {
            let r = tool.press.handle(event, *rect, tool.disabled);
            if tool.press.hovered {
                hovered = Some((tool.label.clone(), tool.key.clone(), *rect));
            }
            out = merge(out, r);
        }
        let r = self.zoom_press.handle(event, at.zoom, false);
        if r.clicked {
            self.open = !self.open;
        }
        out = merge(out, r);
        out = merge(out, self.export.handle_event(event, at.export));
        let tip = hovered.map(|(label, key, rect)| {
            let mut tip = Tooltip::new(label).key(key);
            tip.show(rect);
            tip
        });
        if tip.as_ref().map(|t| (&t.text, &t.key))
            != self.tooltip.as_ref().map(|t| (&t.text, &t.key))
        {
            out = merge(out, EventResult::changed());
        }
        self.tooltip = tip;
        out
    }

    /// The bar; the open panel and the tooltip go on top, from
    /// [`Toolbar::render_overlay`].
    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let r = Self::recipe(theme);
        let (w, h) = self.preferred_size(theme);
        let bar = Rect::new(bounds.x, bounds.y, w, h);
        shadow_drops(c, layer, bar, r.radius, &r.shadow);
        c.push_to_layer(
            layer,
            rounded_rect(bar.x, bar.y, bar.w, bar.h, r.radius, r.fill.0),
        );
        c.push_to_layer(
            layer,
            rounded_rect_stroke(
                bar.x,
                bar.y,
                bar.w,
                bar.h,
                r.radius,
                r.border.0,
                r.border_width,
            ),
        );
        let at = self.layout(bounds, theme);
        for (i, (tool, rect)) in self.tools.iter().zip(&at.tools).enumerate() {
            let active = self.selectable && self.active == Some(i);
            tool.render_to_layer(c, layer, *rect, active, theme);
        }
        for rule in at.divider.iter().chain([&at.split]) {
            c.push_to_layer(layer, rule_node(*rule, r));
        }
        self.render_zoom_control(c, layer, at.zoom, theme);
        for (tool, rect) in self.history.iter().zip(&at.history) {
            tool.render_to_layer(c, layer, *rect, false, theme);
        }
        self.export.render_to_layer(c, layer, at.export, theme);
    }

    fn render_zoom_control(&self, c: &mut Compositor, layer: LayerId, rect: Rect, theme: &Theme) {
        let r = Self::recipe(theme);
        if let Some(id) = &self.zoom_sem {
            c.bind_to_layer(layer, id, [rect.x, rect.y, rect.w, rect.h]);
        }
        let hot = self.zoom_press.hovered || self.zoom_press.pressed;
        let fill = if hot { r.zoom_hover_fill } else { r.zoom_fill };
        let border = if self.open {
            r.zoom_open_border
        } else if hot {
            r.zoom_hover_border
        } else {
            r.zoom_border
        };
        c.push_to_layer(
            layer,
            rounded_rect(rect.x, rect.y, rect.w, rect.h, r.zoom_radius, fill.0),
        );
        c.push_to_layer(
            layer,
            rounded_rect_stroke(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                r.zoom_radius,
                border.0,
                r.border_width,
            ),
        );
        let style = r.zoom_text.style(&theme.typography);
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&self.zoom.level, &style, None),
                x: rect.x + r.zoom_pad_x,
                y: rect.y + (rect.h - style.line_height) / 2.0,
                color: r.zoom_text_color.0,
            },
        );
        let chevron = if self.open {
            "chevron-up"
        } else {
            "chevron-down"
        };
        let x = rect.x + rect.w - r.zoom_pad_x - r.chevron;
        let y = rect.y + (rect.h - r.chevron) / 2.0;
        if let Some(node) = icons::icon_at(chevron, r.chevron, r.chevron_color.0, x, y) {
            c.push_to_layer(layer, node);
        }
    }

    /// The open zoom panel and the hovered tool's tooltip, kept inside
    /// `room` (the card or the window).
    pub fn render_overlay(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        bounds: Rect,
        room: Rect,
        theme: &Theme,
    ) {
        if self.open {
            self.zoom
                .render_to_layer(c, layer, self.panel_rect(bounds, theme), theme);
        }
        if let Some(tip) = &self.tooltip {
            tip.render_in(c, layer, theme, room);
        }
    }

    /// A `Role::Toolbar` named `label`, its tools, the zoom control (its
    /// panel's menu while open) and the export button under it.
    #[track_caller]
    pub fn project(
        &self,
        b: &mut SnapshotBuilder,
        id: &SemId,
        label: &str,
        parent: Option<&SemId>,
    ) {
        let mut node = b.node(id, Role::Toolbar, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        for (i, tool) in self.tools.iter().enumerate() {
            if let Some(tid) = tool.sem_id() {
                let active = self.selectable.then_some(self.active == Some(i));
                tool.project(b, tid, Some(id), active);
            }
        }
        if let Some(zid) = &self.zoom_sem {
            let mut zoom = b.node(zid, Role::Button, "Zoom");
            zoom.parent(id);
            zoom.description(self.zoom.level.as_str());
            zoom.state("expanded", self.open);
            zoom.action("activate");
            if self.open
                && let Some(menu) = self.zoom.sem_id()
            {
                self.zoom.project(b, menu, Some(zid));
            }
        }
        for tool in &self.history {
            if let Some(tid) = tool.sem_id() {
                tool.project(b, tid, Some(id), None);
            }
        }
        if let Some(eid) = self.export.sem_id() {
            self.export.project(b, eid, Some(id));
        }
    }
}

fn rule_node(rect: Rect, r: &ToolbarRecipe) -> SceneNode {
    SceneNode::Rect {
        x: rect.x,
        y: rect.y,
        w: rect.w,
        h: rect.h,
        color: r.divider.0,
    }
}

fn merge(a: EventResult, b: EventResult) -> EventResult {
    EventResult {
        handled: a.handled || b.handled,
        changed: a.changed || b.changed,
        clicked: a.clicked || b.clicked,
    }
}
