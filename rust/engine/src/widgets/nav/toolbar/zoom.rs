//! The kit's zoom panel (`components/Zoom`): the level in a field, the
//! zoom commands with their keys (one of them checked, the last run), a
//! rule, and the view toggles (each checked while on). The toolbar opens
//! it under its zoom control; the menus page reuses it on its own.
//! Every value is `theme.navigation.tool.zoom`; a key is the kit's cap
//! (`content::Kbd`).

use accord::{Role, SemId, SnapshotBuilder};

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::styling::theme::Theme;
use crate::widgets::content::{Kbd, KbdSize};
use crate::widgets::form::TextField;
use crate::widgets::form::tokens::Look;
use crate::widgets::icons;
use crate::widgets::recipe::{rounded_rect, rounded_rect_stroke, shadow_drops};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// One row: its title and the key that runs it.
#[derive(Clone, Debug, PartialEq)]
pub struct ZoomLine {
    pub label: String,
    pub keys: String,
}

impl ZoomLine {
    pub fn new(label: impl Into<String>, keys: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            keys: keys.into(),
        }
    }
}

/// The zoom panel.
#[derive(Clone, Debug)]
pub struct Zoom {
    pub level: String,
    /// The level's field, the form's bare text field (front A): a press
    /// on the level focuses it, and what is typed there is the level
    /// ([`Zoom::sync_level`]).
    pub field: TextField,
    pub commands: Vec<ZoomLine>,
    /// The command checked (the last one run).
    pub active: usize,
    pub toggles: Vec<(ZoomLine, bool)>,
    hovered: Option<usize>,
    sem: Option<SemId>,
}

impl Zoom {
    pub fn new(level: impl Into<String>) -> Self {
        let level = level.into();
        Self {
            field: TextField::new("").look(Look::Bare).with_text(&level),
            level,
            commands: Vec::new(),
            active: 0,
            toggles: Vec::new(),
            hovered: None,
            sem: None,
        }
    }

    pub fn command(mut self, label: &str, keys: &str) -> Self {
        self.commands.push(ZoomLine::new(label, keys));
        self
    }

    pub fn toggle(mut self, label: &str, keys: &str, on: bool) -> Self {
        self.toggles.push((ZoomLine::new(label, keys), on));
        self
    }

    pub fn active(mut self, i: usize) -> Self {
        self.active = i;
        self
    }

    /// The identity the panel projects as a menu; its rows are
    /// `<id>.line[<i>]`, commands then toggles.
    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// The panel's width and height: the field section, the commands,
    /// the rule between them and the toggles.
    pub fn size(&self, theme: &Theme) -> (f32, f32) {
        let z = &theme.navigation.tool.zoom;
        let field = z.pad * 2.0 + z.field_h + z.border_width;
        let rule = if self.toggles.is_empty() {
            0.0
        } else {
            z.pad * 2.0 + z.border_width
        };
        let lines = (self.commands.len() + self.toggles.len()) as f32 * z.line_h;
        (z.width, field + z.pad * 2.0 + lines + rule)
    }

    /// The field, the rule under its section, every row (commands then
    /// toggles) and the rule between them, in a panel at `bounds`.
    fn parts(&self, bounds: Rect, theme: &Theme) -> (Rect, f32, Vec<Rect>, Option<f32>) {
        let z = &theme.navigation.tool.zoom;
        let inner_w = bounds.w - z.pad * 2.0;
        let field = Rect::new(bounds.x + z.pad, bounds.y + z.pad, inner_w, z.field_h);
        let section_rule = field.y + field.h + z.pad;
        let mut y = section_rule + z.border_width + z.pad;
        let mut rows = Vec::with_capacity(self.commands.len() + self.toggles.len());
        for _ in &self.commands {
            rows.push(Rect::new(field.x, y, inner_w, z.line_h));
            y += z.line_h;
        }
        let mut between = None;
        if !self.toggles.is_empty() {
            between = Some(y + z.pad);
            y += z.pad * 2.0 + z.border_width;
        }
        for _ in &self.toggles {
            rows.push(Rect::new(field.x, y, inner_w, z.line_h));
            y += z.line_h;
        }
        (field, section_rule, rows, between)
    }

    /// Every row's rect in a panel at `bounds`, commands then toggles: where
    /// a host rings the row the keyboard is on.
    pub fn line_rects(&self, bounds: Rect, theme: &Theme) -> Vec<Rect> {
        self.parts(bounds, theme).2
    }

    /// The row under `(x, y)` in a panel at `bounds`.
    pub fn line_at(&self, x: f32, y: f32, bounds: Rect, theme: &Theme) -> Option<usize> {
        self.line_rects(bounds, theme)
            .iter()
            .position(|r| r.contains(x, y))
    }

    /// Row `i`'s title.
    pub fn line_label(&self, i: usize) -> Option<&str> {
        let n = self.commands.len();
        match i.checked_sub(n) {
            None => self.commands.get(i),
            Some(t) => self.toggles.get(t).map(|(l, _)| l),
        }
        .map(|l| l.label.as_str())
    }

    /// The level's text in the field: after the icon, to the field's end.
    pub fn text_rect(&self, bounds: Rect, theme: &Theme) -> Rect {
        let z = &theme.navigation.tool.zoom;
        let (f, _, _, _) = self.parts(bounds, theme);
        let x = f.x + z.field_text_x;
        Rect::new(x, f.y, (f.right() - x - z.pad).max(0.0), f.h)
    }

    /// The level is what the field holds.
    pub fn sync_level(&mut self) {
        self.level = self.field.text().to_string();
    }

    /// Hover on the rows; a click on a command checks it, on a toggle
    /// flips it. What lands in the panel is handled.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let (_, _, rows, _) = self.parts(bounds, theme);
        let text = self.text_rect(bounds, theme);
        let (x, y) = event.pos();
        let press = matches!(event, WidgetEvent::MouseDown { .. });
        if (press && text.contains(x, y)) || (!press && self.field.is_focused()) {
            let r = self.field.handle_event(event, text, theme);
            if press || r.changed {
                return EventResult { handled: true, ..r };
            }
        } else if press && self.field.is_focused() {
            self.field.unfocus();
        }
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let hit = rows.iter().position(|r| r.contains(x, y));
                if hit != self.hovered {
                    self.hovered = hit;
                    return EventResult::changed();
                }
                EventResult::IGNORED
            }
            WidgetEvent::MouseDown { x, y } if bounds.contains(x, y) => EventResult {
                handled: true,
                changed: false,
                clicked: false,
            },
            WidgetEvent::MouseUp { x, y } if bounds.contains(x, y) => {
                match rows.iter().position(|r| r.contains(x, y)) {
                    Some(i) => {
                        self.choose(i);
                        EventResult::clicked()
                    }
                    None => EventResult {
                        handled: true,
                        changed: false,
                        clicked: false,
                    },
                }
            }
            _ => EventResult::IGNORED,
        }
    }

    /// Run row `i`: a command is checked, a toggle flips.
    pub fn choose(&mut self, i: usize) {
        let n = self.commands.len();
        if i < n {
            self.active = i;
        } else if let Some((_, on)) = self.toggles.get_mut(i - n) {
            *on = !*on;
        }
    }

    /// Whether row `i` is checked.
    pub fn checked(&self, i: usize) -> bool {
        let n = self.commands.len();
        if i < n {
            i == self.active
        } else {
            self.toggles.get(i - n).is_some_and(|(_, on)| *on)
        }
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let z = &theme.navigation.tool.zoom;
        if let Some(id) = &self.sem {
            c.bind_to_layer(layer, id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let b = bounds;
        shadow_drops(c, layer, b, z.radius, &z.shadow);
        c.push_to_layer(layer, rounded_rect(b.x, b.y, b.w, b.h, z.radius, z.fill.0));
        c.push_to_layer(
            layer,
            rounded_rect_stroke(b.x, b.y, b.w, b.h, z.radius, z.border.0, z.border_width),
        );
        let (field, section_rule, rows, between) = self.parts(bounds, theme);
        // The field: its well, its rim, the frame icon centered in the
        // space before the text, the level.
        let f = field;
        c.push_to_layer(
            layer,
            rounded_rect(f.x, f.y, f.w, f.h, z.field_radius, z.field_fill.0),
        );
        c.push_to_layer(
            layer,
            rounded_rect_stroke(
                f.x,
                f.y,
                f.w,
                f.h,
                z.field_radius,
                z.field_border.0,
                z.border_width,
            ),
        );
        let ix = f.x + (z.field_text_x - z.field_icon) / 2.0;
        let iy = f.y + (f.h - z.field_icon) / 2.0;
        if let Some(node) = icons::icon_at("scan", z.field_icon, z.field_icon_color.0, ix, iy) {
            c.push_to_layer(layer, node);
        }
        let style = z.field_text.style(&theme.typography);
        if self.field.is_focused() {
            self.field
                .render_to_layer(c, layer, self.text_rect(bounds, theme), theme);
        } else {
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(&self.level, &style, None),
                    x: f.x + z.field_text_x,
                    y: f.y + (f.h - style.line_height) / 2.0,
                    color: z.field_text_color.0,
                },
            );
        }
        for y in [Some(section_rule), between].into_iter().flatten() {
            c.push_to_layer(
                layer,
                SceneNode::Rect {
                    x: b.x,
                    y,
                    w: b.w,
                    h: z.border_width,
                    color: z.rule.0,
                },
            );
        }
        let lines = self
            .commands
            .iter()
            .chain(self.toggles.iter().map(|(l, _)| l));
        let label = z.label.style(&theme.typography);
        let family = self.sem.as_ref().and_then(|id| id.child("line").ok());
        for (i, (line, r)) in lines.zip(&rows).enumerate() {
            if let Some(family) = &family {
                c.bind_to_layer(layer, &family.key(i.to_string()), [r.x, r.y, r.w, r.h]);
            }
            if self.hovered == Some(i) {
                c.push_to_layer(
                    layer,
                    rounded_rect(r.x, r.y, r.w, r.h, z.line_radius, z.line_hover.0),
                );
            }
            let slot_x = r.x + z.line_pad_x;
            if self.checked(i) {
                let at = (z.check_slot - z.check) / 2.0;
                let (x, y) = (slot_x + at, r.y + (r.h - z.check) / 2.0);
                if let Some(node) = icons::icon_at("check", z.check, z.check_color.0, x, y) {
                    c.push_to_layer(layer, node);
                }
            }
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(&line.label, &label, None),
                    x: slot_x + z.check_slot + z.check_gap,
                    y: r.y + (r.h - label.line_height) / 2.0,
                    color: z.label_color.0,
                },
            );
            let cap = Kbd::new(line.keys.as_str()).size(KbdSize::Sm);
            let (kw, kh) = cap.preferred_size(theme);
            let at = Rect::new(
                r.x + r.w - z.line_pad_x - kw,
                r.y + (r.h - kh) / 2.0,
                kw,
                kh,
            );
            cap.render_to_layer(c, layer, at, theme);
        }
    }

    /// A `Role::Menu` named "Zoom", the level its value; a command is a
    /// radio, a toggle a checkbox, each `checked`, each with `activate`.
    #[track_caller]
    pub fn project(&self, b: &mut SnapshotBuilder, id: &SemId, parent: Option<&SemId>) {
        let mut node = b.node(id, Role::Menu, "Zoom");
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("value", self.level.as_str());
        let Some(family) = id.child("line").ok() else {
            return;
        };
        let lines = self
            .commands
            .iter()
            .map(|l| (l, Role::Radio))
            .chain(self.toggles.iter().map(|(l, _)| (l, Role::Checkbox)));
        for (i, (line, role)) in lines.enumerate() {
            let at = family.key(i.to_string());
            let mut row = b.node(&at, role, line.label.as_str());
            row.parent(id);
            row.description(line.keys.as_str());
            row.state("checked", self.checked(i));
            row.action("activate");
        }
    }
}
