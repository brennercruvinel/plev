//! Dock section: the floating message dock, absorbed from
//! examples/message_dock and rebuilt on the engine. Every motion is a
//! dt-based Tween inside `showcase::model::dock::DockModel` (backend
//! tested first); this view only derives rects and pushes scene nodes.
//!
//! One source of truth for geometry: `handle_event` hit-tests exactly the
//! rects `render` draws: the model getters plus the input/send rects
//! derived below from those same getters (no layout constant is restated
//! here). Glass is the real HOFF recipe (menu shadow, BackdropBlur,
//! edge-lit translucent pill) and all text centering is measured with one
//! TextStyle per run for measurement and drawing.

use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::Theme;
use engine::ui::icons;
use engine::ui::widgets::{
    EventResult, Rect, WidgetEvent, glass_pill, menu_shadow, rounded_rect, rounded_rect_stroke,
};
use showcase::model::dock::{AVATARS, DockModel};

use super::{group_label, panel, text, with_alpha};

#[cfg(test)]
mod tests;

/// Roster shown in the dock. The hue is the only chromatic note: HOFF
/// keeps every surface graphite, people get color.
const CONTACTS: [(&str, [f32; 4]); AVATARS] = [
    ("Ana", [0.36, 0.55, 0.50, 1.0]),
    ("Bea", [0.70, 0.55, 0.34, 1.0]),
    ("Caio", [0.52, 0.47, 0.66, 1.0]),
    ("Duda", [0.64, 0.42, 0.47, 1.0]),
];

/// Stage wallpaper bubbles (text, contact hue); the first one sits under
/// the dock so the frost has something visible to blur.
const BUBBLES: [(&str, usize); 3] = [
    ("the dock floats over real content", 0),
    ("so the glass frosts what is underneath", 2),
    ("hover lifts a contact, click morphs the dock open", 1),
];

pub struct DockSection {
    model: DockModel,
    send_hover: bool,
}

impl DockSection {
    pub fn new() -> Self {
        Self {
            model: DockModel::new(),
            send_hover: false,
        }
    }

    /// The stage fills the content viewport; the dock floats inside it.
    pub fn layout(&self, content: Rect) -> Rect {
        content
    }

    pub fn content_height(&self, content: Rect) -> f32 {
        content.h
    }

    /// Roster metrics recovered from the model getters so the dock layout
    /// constants live in exactly one place: (pad, avatar size, gap).
    fn metrics(&self, stage: Rect) -> (f32, f32, f32) {
        let dock = self.model.dock_rect(stage);
        let a0 = self.model.avatar_rect(0, stage);
        let pad = a0.x - dock.x;
        // Avatar 1 probes the slot pitch, unless it is the one sliding.
        let j = if self.model.selected() == Some(1) {
            2
        } else {
            1
        };
        let aj = self.model.avatar_rect(j, stage);
        let gap = (aj.x - dock.x - pad) / j as f32 - a0.w;
        (pad, a0.w, gap)
    }

    /// Trailing circular button (compose at rest, send when open).
    fn send_rect(&self, stage: Rect) -> Rect {
        let dock = self.model.dock_rect(stage);
        let (pad, s, _) = self.metrics(stage);
        Rect::new(dock.x + dock.w - pad - s, dock.y + (dock.h - s) / 2.0, s, s)
    }

    /// Message field between the front avatar slot and the send button.
    fn input_rect(&self, stage: Rect) -> Rect {
        let dock = self.model.dock_rect(stage);
        let (pad, s, gap) = self.metrics(stage);
        let x = dock.x + pad + s + gap;
        let w = (self.send_rect(stage).x - gap - x).max(0.0);
        let h = s - 6.0;
        Rect::new(x, dock.y + (dock.h - h) / 2.0, w, h)
    }

    /// Avatar under the pointer; faded-out ones are not hit (when the
    /// dock is open their rects sit invisible under the message field).
    fn avatar_hit(&self, x: f32, y: f32, stage: Rect) -> Option<usize> {
        (0..AVATARS).find(|&i| {
            self.model.avatar_alpha(i) > 0.5 && self.model.avatar_rect(i, stage).contains(x, y)
        })
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, content: Rect) -> EventResult {
        let stage = self.layout(content);
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let mut changed = self.model.on_hover(self.avatar_hit(x, y, stage));
                let over_send = self.send_rect(stage).contains(x, y);
                if over_send != self.send_hover {
                    self.send_hover = over_send;
                    changed = true;
                }
                if changed {
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                if let Some(i) = self.avatar_hit(x, y, stage) {
                    if self.model.on_click(i) {
                        return EventResult::clicked();
                    }
                } else if self.send_rect(stage).contains(x, y) && self.model.on_send() {
                    return EventResult::clicked();
                }
                if self.model.dock_rect(stage).contains(x, y) {
                    return EventResult {
                        handled: true,
                        ..EventResult::IGNORED
                    };
                }
                EventResult::IGNORED
            }
            _ => EventResult::IGNORED,
        }
    }

    /// Advance the dock model; true while anything still moves.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.model.update(dt)
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let stage = self.layout(content);
        self.render_stage(c, stage, theme);
        self.render_dock(c, stage, theme);
    }

    /// Stage backdrop: hint plus measured chat bubbles for the frost.
    fn render_stage(&self, c: &mut Compositor, stage: Rect, theme: &Theme) {
        panel(c, stage, theme);
        group_label(c, "MESSAGE DOCK", stage.x + 24.0, stage.y + 20.0, theme);
        text(
            c,
            "Hover lifts a contact; click morphs the dock open.",
            13.0,
            400,
            stage.x + 24.0,
            stage.y + 44.0,
            theme.colors.text_dim.0,
        );

        let style = TextStyle::new(13.0);
        let mw = (stage.w - 96.0).max(40.0);
        let bottom = stage.y + stage.h;
        let mut y = bottom - 64.0;
        for (i, (line, hue)) in BUBBLES.iter().enumerate() {
            let (tw, th) = TextMeasurer::measure_styled(line, &style, Some(mw));
            let (w, h) = (tw + 28.0, th + 14.0);
            y -= h + 14.0;
            if y < stage.y + 72.0 {
                break; // short viewport: keep the wallpaper off the header
            }
            let x = match i {
                0 => stage.x + (stage.w - w) / 2.0, // under the dock
                1 => stage.x + 24.0,
                _ => stage.x + stage.w - 24.0 - w,
            };
            c.push(rounded_rect(
                x,
                y,
                w,
                h,
                h / 2.0,
                with_alpha(CONTACTS[*hue].1, 0.30),
            ));
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(line, &style, Some(mw)),
                x: x + (w - tw) / 2.0,
                y: y + (h - th) / 2.0,
                color: theme.colors.text_mid.0,
            });
        }
    }

    fn render_dock(&self, c: &mut Compositor, stage: Rect, theme: &Theme) {
        let glass = &theme.glass;
        let dock = self.model.dock_rect(stage);
        let radius = dock.h / 2.0;
        // Real glass: deep shadow, frost, edge-lit translucent pill.
        c.push(menu_shadow(dock, radius));
        c.draw_backdrop_blur(
            dock.x,
            dock.y,
            dock.w,
            dock.h,
            radius,
            theme.effects.blur_sigma,
        );
        for node in glass_pill(dock, radius, glass.edge.0, 1.5, glass.button.0) {
            c.push(node);
        }
        self.render_roster(c, stage, theme);
        self.render_panel(c, stage, theme);
        self.render_send(c, stage, theme);
    }

    /// Avatars: circular-ish rounded rects, lift and fade from the model,
    /// initials centered by measurement (one style measures and draws).
    fn render_roster(&self, c: &mut Compositor, stage: Rect, theme: &Theme) {
        let style = TextStyle::new(15.0).with_weight(600);
        let edge = theme.glass.edge.0;
        let ink = theme.colors.text.0;
        for (i, (name, hue)) in CONTACTS.iter().enumerate() {
            let alpha = self.model.avatar_alpha(i);
            if alpha < 0.01 {
                continue;
            }
            let r = self.model.avatar_rect(i, stage);
            let radius = r.w * 0.45;
            c.push(rounded_rect(
                r.x,
                r.y,
                r.w,
                r.h,
                radius,
                with_alpha(*hue, alpha),
            ));
            c.push(rounded_rect_stroke(
                r.x,
                r.y,
                r.w,
                r.h,
                radius,
                with_alpha(edge, edge[3] * alpha),
                1.0,
            ));
            let initial = &name[..1];
            let (tw, th) = TextMeasurer::measure_styled(initial, &style, None);
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(initial, &style, None),
                x: r.x + (r.w - tw) / 2.0,
                y: r.y + (r.h - th) / 2.0,
                color: with_alpha(ink, ink[3] * alpha),
            });
        }
    }

    /// Separator and message field cross-fade as the width morphs.
    fn render_panel(&self, c: &mut Compositor, stage: Rect, theme: &Theme) {
        let glass = &theme.glass;
        let dock = self.model.dock_rect(stage);
        let send = self.send_rect(stage);
        let (pad, s, gap) = self.metrics(stage);

        let sep_a = self.model.separator_alpha();
        if sep_a > 0.01 {
            let roster_right = dock.x + pad + AVATARS as f32 * (s + gap) - gap;
            c.push(SceneNode::Rect {
                x: (roster_right + send.x) / 2.0,
                y: dock.y + (dock.h - 24.0) / 2.0,
                w: 1.0,
                h: 24.0,
                color: with_alpha(glass.edge.0, glass.edge.0[3] * sep_a),
            });
        }

        let a = self.model.input_alpha();
        let field = self.input_rect(stage);
        if a < 0.01 || field.w < 60.0 {
            return;
        }
        let fill = glass.field.0;
        c.push(rounded_rect(
            field.x,
            field.y,
            field.w,
            field.h,
            field.h / 2.0,
            with_alpha(fill, fill[3] * a),
        ));
        let hint = format!("Message {}", CONTACTS[self.model.selected().unwrap_or(0)].0);
        let style = TextStyle::new(14.0);
        let (_, th) = TextMeasurer::measure_styled(&hint, &style, None);
        let (tx, ty) = (field.x + 16.0, field.y + (field.h - th) / 2.0);
        let ph = glass.text_placeholder.0;
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(&hint, &style, None),
            x: tx + 4.0,
            y: ty,
            color: with_alpha(ph, ph[3] * a),
        });
        // Blinking caret, gated by the panel fade (the model is the clock).
        let ca = self.model.cursor_alpha();
        if ca > 0.01 {
            let ink = theme.colors.text.0;
            c.push(SceneNode::Rect {
                x: tx,
                y: ty + 1.0,
                w: 2.0,
                h: (th - 2.0).max(2.0),
                color: with_alpha(ink, ink[3] * ca),
            });
        }
    }

    /// Trailing button: plus at rest cross-fades to send when open, with
    /// a success flash while a send is in flight.
    fn render_send(&self, c: &mut Compositor, stage: Rect, theme: &Theme) {
        let glass = &theme.glass;
        let send = self.send_rect(stage);
        let radius = send.w / 2.0;
        let fill = if self.send_hover {
            glass.surface_active.0
        } else {
            glass.surface_hover.0
        };
        c.push(rounded_rect(send.x, send.y, send.w, send.h, radius, fill));

        let dim = glass.text_faint.0;
        let ink = theme.colors.text.0;
        let mut icon = |name: &str, size: f32, color: [f32; 4]| {
            if color[3] < 0.01 {
                return;
            }
            let (ix, iy) = (
                send.x + (send.w - size) / 2.0,
                send.y + (send.h - size) / 2.0,
            );
            if let Some(node) = icons::icon_at(name, size, color, ix, iy) {
                c.push(node);
            }
        };
        icon(
            "plus",
            18.0,
            with_alpha(dim, dim[3] * self.model.separator_alpha()),
        );
        icon(
            "play",
            16.0,
            with_alpha(ink, ink[3] * self.model.input_alpha()),
        );

        let flash = self.model.flash_alpha();
        if flash > 0.01 {
            let go = theme.colors.success.0;
            c.push(rounded_rect(
                send.x,
                send.y,
                send.w,
                send.h,
                radius,
                with_alpha(go, 0.85 * flash),
            ));
        }
    }
}
