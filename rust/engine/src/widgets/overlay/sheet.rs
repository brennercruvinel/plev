//! The sheet: a panel held to one edge of the container that receives it
//! (top, right, bottom or left), over a scrim, blocking what is beneath
//! until it is answered. It holds a title, a description, option rows
//! and the dialog's two answers, with a close button in its head; a
//! press on the scrim, the close button and Escape cancel. Every value is
//! `theme.overlay.sheet` ([`SheetTokens`]); the footer is the dialog's
//! (`theme.overlay.modal`).
//!
//! The geometry is relations, never the window: across its edge a side
//! sheet is the token's extent, capped by the container less a touch
//! target of scrim to close it by ([`edge_rect`]); a top or a bottom
//! sheet is as tall as its content, capped alike, and as wide as the
//! container. It slides in from its edge with its overlay entry's
//! spring ([`slide`]) and the scrim fades with it; with reduced motion
//! the spring ends on its first frame, a cut.
//!
//! The behavior (open, answer, focus, projection) is the app's and the
//! overlay stack's, as for the modal: the view pushes an entry and draws
//! the sheet with the entry's progress. The drawer composes the same
//! edge panel ([`paint_panel`]) with navigation inside.
//!
//! With a [`SemId`] ([`Sheet::sem`]) the render binds the panel and its
//! buttons (`<id>[close]`, `<id>[cancel]`, `<id>[confirm]`), and
//! [`Sheet::project`] declares it: a `Role::Dialog` named by its title.

mod tokens;

pub use tokens::{DrawerTokens, EdgePanelTokens, SheetTokens};

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::Theme;
use crate::widgets::buttons::{
    Button, ButtonVariant, IconButton, IconButtonSize, IconButtonVariant,
};
use crate::widgets::feedback::ModalAction;
use crate::widgets::recipe::{rounded_rect, rounded_rect_stroke, shadow_node};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// The edge of the container a panel is held to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Edge {
    Top,
    #[default]
    Right,
    Bottom,
    Left,
}

impl Edge {
    pub const ALL: [Edge; 4] = [Edge::Top, Edge::Right, Edge::Bottom, Edge::Left];

    /// Held to the left or the right: its extent is a width.
    pub fn is_side(self) -> bool {
        matches!(self, Edge::Left | Edge::Right)
    }

    pub fn name(self) -> &'static str {
        match self {
            Edge::Top => "top",
            Edge::Right => "right",
            Edge::Bottom => "bottom",
            Edge::Left => "left",
        }
    }
}

/// The panel held to `edge` of `bounds`, settled: `across` its edge
/// (capped by `bounds` less a touch target of scrim), the whole edge
/// along it.
pub fn edge_rect(edge: Edge, across: f32, bounds: Rect, theme: &Theme) -> Rect {
    let reserve = theme.control.min_touch_target;
    let b = bounds;
    if edge.is_side() {
        let w = across.min(b.w - reserve).max(0.0);
        let x = if edge == Edge::Left {
            b.x
        } else {
            b.right() - w
        };
        Rect::new(x, b.y, w, b.h)
    } else {
        let h = across.min(b.h - reserve).max(0.0);
        let y = if edge == Edge::Top {
            b.y
        } else {
            b.bottom() - h
        };
        Rect::new(b.x, y, b.w, h)
    }
}

/// `rect` pushed back toward its edge by the part not yet shown:
/// `shown` 1 is settled, 0 just out of the container.
pub fn slide(rect: Rect, edge: Edge, shown: f32) -> Rect {
    let off = 1.0 - shown.clamp(0.0, 1.0);
    let mut r = rect;
    match edge {
        Edge::Left => r.x -= r.w * off,
        Edge::Right => r.x += r.w * off,
        Edge::Top => r.y -= r.h * off,
        Edge::Bottom => r.y += r.h * off,
    }
    r
}

fn intersect(a: Rect, b: Rect) -> Rect {
    let x = a.x.max(b.x);
    let y = a.y.max(b.y);
    let w = (a.right().min(b.right()) - x).max(0.0);
    let h = (a.bottom().min(b.bottom()) - y).max(0.0);
    Rect::new(x, y, w, h)
}

fn push_clip(c: &mut Compositor, layer: LayerId, r: Rect) {
    c.push_to_layer(
        layer,
        SceneNode::PushClip {
            x: r.x,
            y: r.y,
            w: r.w,
            h: r.h,
        },
    );
}

/// The scrim over `bounds`, `shown` of its alpha.
pub fn paint_scrim(
    c: &mut Compositor,
    layer: LayerId,
    bounds: Rect,
    t: &EdgePanelTokens,
    shown: f32,
) {
    let mut color = t.scrim.0;
    color[3] *= shown.clamp(0.0, 1.0);
    let b = bounds;
    c.push_to_layer(
        layer,
        SceneNode::Rect {
            x: b.x,
            y: b.y,
            w: b.w,
            h: b.h,
            color,
        },
    );
}

/// The rounded rect a panel at `panel` held to `edge` is drawn as, and
/// its radius: it runs past the held edge, and past the two sides along
/// it by twice the rule's width (the rule, and the pixel a clip rounded
/// out to the device grid may still show).
fn shape(panel: Rect, edge: Edge, t: &EdgePanelTokens) -> (Rect, f32) {
    let r = t.radius.min(panel.w / 2.0).min(panel.h / 2.0);
    let side = t.rule_w * 2.0;
    let past = r + side;
    let p = panel;
    let ext = match edge {
        Edge::Left => Rect::new(p.x - past, p.y - side, p.w + past, p.h + side * 2.0),
        Edge::Right => Rect::new(p.x, p.y - side, p.w + past, p.h + side * 2.0),
        Edge::Top => Rect::new(p.x - side, p.y - past, p.w + side * 2.0, p.h + past),
        Edge::Bottom => Rect::new(p.x - side, p.y, p.w + side * 2.0, p.h + past),
    };
    (ext, r)
}

/// The surface of a panel at `panel` held to `edge`, inside `clip` (the
/// container): its shadows, its fill, and the rule of its free edge,
/// rounded at the two corners away from the edge. The rect drawn runs
/// past the held edge (and the rule past the two sides along it), so
/// only the free edge shows its corners and its rule.
pub fn paint_panel(
    c: &mut Compositor,
    layer: LayerId,
    panel: Rect,
    clip: Rect,
    edge: Edge,
    t: &EdgePanelTokens,
) {
    let (ext, r) = shape(panel, edge, t);
    push_clip(c, layer, clip);
    for spec in &t.shadow {
        if let Some(node) = shadow_node(ext, r, spec) {
            c.push_to_layer(layer, node);
        }
    }
    c.push_to_layer(layer, SceneNode::PopClip);
    push_clip(c, layer, intersect(panel, clip));
    c.push_to_layer(layer, rounded_rect(ext.x, ext.y, ext.w, ext.h, r, t.fill.0));
    if t.rule.0[3] > 0.0 {
        let e = ext;
        let ring = rounded_rect_stroke(e.x, e.y, e.w, e.h, r, t.rule.0, t.rule_w);
        c.push_to_layer(layer, ring);
    }
    c.push_to_layer(layer, SceneNode::PopClip);
}

/// One option row: what it is, a hint under it, its value at the end.
#[derive(Clone, Debug, PartialEq)]
pub struct SheetRow {
    pub label: String,
    pub hint: Option<String>,
    pub value: String,
}

impl SheetRow {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            hint: None,
            value: value.into(),
        }
    }

    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

/// One of the sheet's three buttons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SheetPart {
    Close,
    Cancel,
    Confirm,
}

impl SheetPart {
    pub const ALL: [SheetPart; 3] = [SheetPart::Close, SheetPart::Cancel, SheetPart::Confirm];

    /// Its key under the sheet's id.
    pub fn key(self) -> &'static str {
        match self {
            SheetPart::Close => "close",
            SheetPart::Cancel => "cancel",
            SheetPart::Confirm => "confirm",
        }
    }
}

/// Where the parts of a sheet go inside its panel.
struct Parts {
    close: Rect,
    title: (f32, f32),
    description_y: f32,
    /// The rows' band and each row's top.
    rows: Rect,
    row_y: Vec<f32>,
    footer_y: f32,
    cancel: Rect,
    confirm: Rect,
}

/// An edge panel with a title, a description, option rows and two answers.
#[derive(Clone, Debug)]
pub struct Sheet {
    pub edge: Edge,
    pub title: String,
    pub description: String,
    pub rows: Vec<SheetRow>,
    close: IconButton,
    cancel: Button,
    confirm: Button,
    sem: Option<SemId>,
}

impl Sheet {
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        confirm: impl Into<String>,
        cancel: impl Into<String>,
    ) -> Self {
        Self {
            edge: Edge::default(),
            title: title.into(),
            description: description.into(),
            rows: Vec::new(),
            close: IconButton::new("x")
                .variant(IconButtonVariant::Ghost)
                .size(IconButtonSize::Md),
            cancel: Button::new(cancel).variant(ButtonVariant::Primary),
            confirm: Button::new(confirm).variant(ButtonVariant::Secondary),
            sem: None,
        }
    }

    pub fn edge(mut self, edge: Edge) -> Self {
        self.edge = edge;
        self
    }

    pub fn row(mut self, row: SheetRow) -> Self {
        self.rows.push(row);
        self
    }

    /// The identity the render binds the panel to; the buttons bind as
    /// `<id>[close]`, `<id>[cancel]` and `<id>[confirm]`.
    pub fn sem(mut self, id: SemId) -> Self {
        self.close = self.close.sem(id.key(SheetPart::Close.key()));
        self.cancel = self.cancel.sem(id.key(SheetPart::Cancel.key()));
        self.confirm = self.confirm.sem(id.key(SheetPart::Confirm.key()));
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn label(&self, part: SheetPart) -> &str {
        match part {
            SheetPart::Close => "Close",
            SheetPart::Cancel => self.cancel.label.as_str(),
            SheetPart::Confirm => self.confirm.label.as_str(),
        }
    }

    /// Keyboard focus on one of its buttons, driven by the owner; `None`
    /// clears it.
    pub fn set_focused(&mut self, part: Option<SheetPart>) {
        self.close.set_focused(part == Some(SheetPart::Close));
        self.cancel.set_focused(part == Some(SheetPart::Cancel));
        self.confirm.set_focused(part == Some(SheetPart::Confirm));
    }

    fn text_h(text: &str, style: &TextStyle, w: f32) -> f32 {
        let (_, h) = TextMeasurer::measure_styled(text, style, Some(w.max(1.0)));
        h.max(style.line_height)
    }

    fn head_h(&self, theme: &Theme, w: f32) -> f32 {
        let scale = &theme.typography;
        let t = &theme.overlay.sheet;
        let text_w = w - t.pad_x * 2.0;
        let description = Self::text_h(&self.description, &scale.run(t.description), text_w);
        t.pad_y + scale.run(t.title).line_height + t.gap + description + t.pad_y
    }

    fn value_w(row: &SheetRow, theme: &Theme) -> f32 {
        let style = &theme.typography.run(theme.overlay.sheet.value);
        TextMeasurer::measure_styled(&row.value, style, None)
            .0
            .ceil()
    }

    /// The label's width in a row `w` wide: what the value leaves.
    fn label_w(row: &SheetRow, theme: &Theme, w: f32) -> f32 {
        let t = &theme.overlay.sheet;
        (w - t.pad_x * 2.0 - Self::value_w(row, theme) - t.value_gap).max(1.0)
    }

    fn row_h(row: &SheetRow, theme: &Theme, w: f32) -> f32 {
        let scale = &theme.typography;
        let t = &theme.overlay.sheet;
        let lw = Self::label_w(row, theme, w);
        let mut text = Self::text_h(&row.label, &scale.run(t.label), lw);
        if let Some(hint) = &row.hint {
            text += t.gap + Self::text_h(hint, &scale.run(t.hint), lw);
        }
        (t.pad_y * 2.0 + text).max(t.row_min_h)
    }

    fn footer_h(&self, theme: &Theme) -> f32 {
        let m = &theme.overlay.modal;
        m.pad + self.confirm.size.height(theme) + m.pad + m.border_w
    }

    /// Its content's height in a panel `w` wide.
    pub fn content_h(&self, theme: &Theme, w: f32) -> f32 {
        let rows: f32 = self.rows.iter().map(|r| Self::row_h(r, theme, w)).sum();
        self.head_h(theme, w) + rows + self.footer_h(theme)
    }

    /// The panel settled in `bounds`: a side sheet the token's extent
    /// wide, a top or bottom one as tall as its content.
    pub fn panel_rect(&self, theme: &Theme, bounds: Rect) -> Rect {
        let across = if self.edge.is_side() {
            theme.overlay.sheet.panel.extent
        } else {
            self.content_h(theme, bounds.w)
        };
        edge_rect(self.edge, across, bounds, theme)
    }

    fn parts(&self, theme: &Theme, p: Rect) -> Parts {
        let scale = &theme.typography;
        let t = &theme.overlay.sheet;
        let m = &theme.overlay.modal;
        let (cw, ch) = self.close.preferred_size(theme);
        let top = p.y + t.pad_y;
        let close = Rect::new(
            p.right() - t.pad_y - cw,
            top + (scale.run(t.title).line_height - ch) / 2.0,
            cw,
            ch,
        );
        let head = p.y + self.head_h(theme, p.w);
        let footer_y = (p.bottom() - self.footer_h(theme)).max(head);
        let mut row_y = Vec::with_capacity(self.rows.len());
        let mut y = head;
        for row in &self.rows {
            row_y.push(y);
            y += Self::row_h(row, theme, p.w);
        }
        let btn_h = self.confirm.size.height(theme);
        let by = footer_y + m.border_w + m.pad;
        let w = ((p.w - m.pad * 2.0 - m.button_gap) / 2.0).max(0.0);
        let cancel = Rect::new(p.x + m.pad, by, w, btn_h);
        let confirm = Rect::new(cancel.right() + m.button_gap, by, w, btn_h);
        Parts {
            close,
            title: (p.x + t.pad_x, top),
            description_y: top + scale.run(t.title).line_height + t.gap,
            rows: Rect::new(p.x, head, p.w, footer_y - head),
            row_y,
            footer_y,
            cancel,
            confirm,
        }
    }

    /// Routes an event to the sheet settled in `bounds`: a button's
    /// answer (the close is a cancel), a press on the scrim cancels, and
    /// everything else is swallowed (it blocks).
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        theme: &Theme,
        bounds: Rect,
    ) -> (ModalAction, EventResult) {
        let panel = self.panel_rect(theme, bounds);
        let parts = self.parts(theme, panel);
        let close = self.close.handle_event(event, parts.close);
        let cancel = self.cancel.handle_event(event, parts.cancel);
        let confirm = self.confirm.handle_event(event, parts.confirm);
        let merged = close.merge(cancel).merge(confirm);
        if confirm.clicked {
            return (ModalAction::Confirm, merged);
        }
        if close.clicked || cancel.clicked {
            return (ModalAction::Cancel, merged);
        }
        if let WidgetEvent::MouseDown { x, y } = *event
            && !panel.contains(x, y)
        {
            return (ModalAction::Cancel, EventResult::clicked().merge(merged));
        }
        let handled = EventResult {
            handled: true,
            ..merged
        };
        (ModalAction::None, handled)
    }

    /// The scrim over `bounds` and the sheet sliding in from its edge,
    /// `shown` of the way (its overlay entry's progress).
    pub fn render(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        theme: &Theme,
        bounds: Rect,
        shown: f32,
    ) {
        paint_scrim(c, layer, bounds, &theme.overlay.sheet.panel, shown);
        let panel = slide(self.panel_rect(theme, bounds), self.edge, shown);
        self.render_in(c, layer, theme, panel, bounds);
    }

    /// The sheet at `panel`, inside `clip`, without a scrim: on screen,
    /// or shown as a state in a page's card.
    pub fn render_in(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        theme: &Theme,
        panel: Rect,
        clip: Rect,
    ) {
        let scale = &theme.typography;
        let t = &theme.overlay.sheet;
        let m = &theme.overlay.modal;
        if let Some(id) = &self.sem {
            c.bind_to_layer(layer, id, [panel.x, panel.y, panel.w, panel.h]);
        }
        paint_panel(c, layer, panel, clip, self.edge, &t.panel);
        push_clip(c, layer, intersect(panel, clip));
        let parts = self.parts(theme, panel);
        let text_w = panel.w - t.pad_x * 2.0;
        let title_w = parts.close.x - parts.title.0 - t.gap;
        let text = |c: &mut Compositor,
                    s: &str,
                    style: &TextStyle,
                    w: f32,
                    x: f32,
                    y: f32,
                    color: [f32; 4]| {
            let key = TextNodeKey::from_style(s, style, Some(w.max(1.0)));
            c.push_to_layer(layer, SceneNode::Text { key, x, y, color });
        };
        let (tx, ty) = parts.title;
        text(
            c,
            &self.title,
            &scale.run(t.title),
            title_w,
            tx,
            ty,
            t.title_color.0,
        );
        let dy = parts.description_y;
        text(
            c,
            &self.description,
            &scale.run(t.description),
            text_w,
            tx,
            dy,
            t.description_color.0,
        );
        self.close.render_to_layer(c, layer, parts.close, theme);

        // The rows, each under its rule, inside their band.
        push_clip(c, layer, intersect(parts.rows, clip));
        for (row, y) in self.rows.iter().zip(&parts.row_y) {
            let rule = SceneNode::Rect {
                x: panel.x,
                y: *y,
                w: panel.w,
                h: t.panel.rule_w,
                color: t.panel.rule.0,
            };
            c.push_to_layer(layer, rule);
            let h = Self::row_h(row, theme, panel.w);
            let lw = Self::label_w(row, theme, panel.w);
            let mut block = Self::text_h(&row.label, &scale.run(t.label), lw);
            if let Some(hint) = &row.hint {
                block += t.gap + Self::text_h(hint, &scale.run(t.hint), lw);
            }
            let ly = y + (h - block) / 2.0;
            text(
                c,
                &row.label,
                &scale.run(t.label),
                lw,
                tx,
                ly,
                t.label_color.0,
            );
            if let Some(hint) = &row.hint {
                let hy = ly + Self::text_h(&row.label, &scale.run(t.label), lw) + t.gap;
                text(c, hint, &scale.run(t.hint), lw, tx, hy, t.hint_color.0);
            }
            let vw = Self::value_w(row, theme);
            let vx = panel.right() - t.pad_x - vw;
            let vy = y + TextMeasurer::vertical_center(&scale.run(t.value), h);
            text(
                c,
                &row.value,
                &scale.run(t.value),
                vw + 1.0,
                vx,
                vy,
                t.value_color.0,
            );
        }
        c.push_to_layer(layer, SceneNode::PopClip);

        // The dialog's footer, when the theme has one: its band in the
        // panel's shape (its free corners), its rule on top.
        if m.footer.0[3] > 0.0 {
            let band = Rect::new(
                panel.x,
                parts.footer_y,
                panel.w,
                panel.bottom() - parts.footer_y,
            );
            let (e, r) = shape(panel, self.edge, &t.panel);
            push_clip(c, layer, intersect(band, clip));
            c.push_to_layer(layer, rounded_rect(e.x, e.y, e.w, e.h, r, m.footer.0));
            c.push_to_layer(layer, SceneNode::PopClip);
            let rule = SceneNode::Rect {
                x: band.x,
                y: band.y,
                w: band.w,
                h: m.border_w,
                color: m.footer_rule.0,
            };
            c.push_to_layer(layer, rule);
        }
        self.cancel.render_to_layer(c, layer, parts.cancel, theme);
        self.confirm.render_to_layer(c, layer, parts.confirm, theme);
        c.push_to_layer(layer, SceneNode::PopClip);
    }

    /// Declares the sheet as `id`: a `Role::Dialog` named by its title,
    /// described by its description, with `confirm` and `cancel`; its
    /// three buttons as children with an `activate` each, and each row a
    /// label (`<id>.row[i]`) named by its label and value, described by
    /// its hint. Pass the id given to [`Sheet::sem`].
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        for part in SheetPart::ALL {
            let mut node = b.node(id.key(part.key()), Role::Button, self.label(part));
            node.parent(id);
            node.action("activate");
        }
        if let Ok(rows) = id.child("row") {
            for (i, row) in self.rows.iter().enumerate() {
                let name = format!("{}: {}", row.label, row.value);
                let mut node = b.node(rows.key(i.to_string()), Role::Label, name);
                node.parent(id);
                if let Some(hint) = &row.hint {
                    node.description(hint.as_str());
                }
            }
        }
        let mut node = b.node(id, Role::Dialog, self.title.as_str());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.description(self.description.as_str());
        node.state("edge", self.edge.name());
        node.action("confirm");
        node.action("cancel");
        node
    }
}
