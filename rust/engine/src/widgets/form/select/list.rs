//! The options list a select opens, and a combobox under its filter: a
//! panel below the control (the recipe's `theme.form.select.panel`), its
//! rows one option tall, scrolling in the strip inside the panel's
//! padding and clipped to it.
//!
//! [`Window`] is the one computation of which rows show and where: the
//! draw, the binding and the owner's hit test all read it, and each row
//! is bound with the rect it is drawn at, under the same clip, so
//! `Compositor::hit` and a click agree at every point of the panel.

use std::ops::Range;

use accord::SemId;

use crate::control::input::scroll::ScrollState;
use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::widgets::form::tokens::PanelBody;
use crate::widgets::recipe::{
    glass_pill, menu_shadow, rounded_rect, rounded_rect_stroke, shadow_drops, shadow_insets,
};
use crate::widgets::widget::Rect;

/// The height of the strip `n` rows scroll in: all of them, never more
/// than the panel's most leaves inside its padding.
pub(crate) fn view_h(n: usize, theme: &Theme) -> f32 {
    let p = &theme.form.select.panel;
    let fit = (p.max_h - p.pad * 2.0).max(0.0);
    (n as f32 * p.row_h).min(fit)
}

/// The panel of `n` rows `gap` below the control `bounds`, as wide.
pub(crate) fn panel(bounds: Rect, n: usize, gap: f32, theme: &Theme) -> Rect {
    let p = &theme.form.select.panel;
    Rect::new(
        bounds.x,
        bounds.y + bounds.h + gap,
        bounds.w,
        view_h(n, theme) + p.pad * 2.0,
    )
}

/// `scroll` synced to `n` rows in `theme`.
pub(crate) fn sync(scroll: &mut ScrollState, n: usize, theme: &Theme) {
    scroll.set_content(n as f32 * theme.form.select.panel.row_h);
    scroll.set_viewport(view_h(n, theme));
}

/// Scrolls `scroll` the least that shows row `i` whole.
pub(crate) fn reveal(scroll: &mut ScrollState, i: usize, n: usize, theme: &Theme) {
    let row_h = theme.form.select.panel.row_h;
    let view = view_h(n, theme);
    let top = i as f32 * row_h;
    let bottom = top + row_h;
    if top < scroll.offset() {
        scroll.scroll_to(top);
    } else if bottom > scroll.offset() + view {
        scroll.scroll_to(bottom - view);
    }
}

/// The rows a panel shows: the strip they scroll in, how far they are
/// scrolled, and the rows that reach into the strip.
pub(crate) struct Window {
    pub(crate) view: Rect,
    pub(crate) offset: f32,
    row_h: f32,
    pub(crate) rows: Range<usize>,
}

impl Window {
    /// The window of `n` rows in the panel `dd`, scrolled `offset`.
    pub(crate) fn new(dd: Rect, n: usize, offset: f32, theme: &Theme) -> Self {
        let p = &theme.form.select.panel;
        let view = Rect::new(dd.x, dd.y + p.pad, dd.w, view_h(n, theme));
        let rows = if view.h > 0.0 && p.row_h > 0.0 {
            let first = (offset / p.row_h).floor() as usize;
            let end = ((offset + view.h) / p.row_h).ceil() as usize;
            first.min(n)..end.min(n)
        } else {
            0..0
        };
        Self {
            view,
            offset,
            row_h: p.row_h,
            rows,
        }
    }

    /// Where row `i` is drawn: the panel's full width, one row tall,
    /// scrolled. The strip clips whatever of it reaches past.
    pub(crate) fn row(&self, i: usize) -> Rect {
        Rect::new(
            self.view.x,
            self.view.y + i as f32 * self.row_h - self.offset,
            self.view.w,
            self.row_h,
        )
    }

    /// The shown row under `(x, y)`, found the way `Compositor::hit` finds
    /// a bound one: inside the clip and the row, edges included, the later
    /// row on top where two rows share an edge.
    pub(crate) fn at(&self, x: f32, y: f32) -> Option<usize> {
        if !self.view.contains(x, y) {
            return None;
        }
        self.rows
            .clone()
            .rev()
            .find(|&i| self.row(i).contains(x, y))
    }
}

/// One row as the owner sees it: its label, the id it binds to, and
/// whether it is the selected option or the highlighted one.
pub(crate) struct Row<'a> {
    pub(crate) label: &'a str,
    pub(crate) id: Option<SemId>,
    pub(crate) selected: bool,
    pub(crate) highlighted: bool,
}

/// Draws the panel `dd`'s body: the recipe's flat surface, or the glass
/// popover with its edge light and floating shadow.
pub(crate) fn draw_body(c: &mut Compositor, layer: LayerId, dd: Rect, theme: &Theme) {
    let p = &theme.form.select.panel;
    match &p.body {
        PanelBody::Flat(surface) => {
            shadow_drops(c, layer, dd, p.radius, &surface.shadow);
            c.push_to_layer(
                layer,
                rounded_rect(dd.x, dd.y, dd.w, dd.h, p.radius, surface.fill.0),
            );
            c.push_to_layer(
                layer,
                rounded_rect_stroke(
                    dd.x,
                    dd.y,
                    dd.w,
                    dd.h,
                    p.radius,
                    surface.border.0,
                    p.border_width,
                ),
            );
            shadow_insets(c, layer, dd, p.radius, &surface.shadow);
        }
        // Solid body, edge light, deep shadow. No path icons inside (the
        // marker is an SDF dot), so the whole panel can use the gradient
        // edge-light recipe.
        PanelBody::Glass => {
            let glass = &theme.glass;
            for node in menu_shadow(dd, p.radius, theme) {
                c.push_to_layer(layer, node);
            }
            for node in glass_pill(
                dd,
                p.radius,
                glass.edge_soft.0,
                p.border_width,
                glass.popover.0,
                theme,
            ) {
                c.push_to_layer(layer, node);
            }
        }
    }
}

/// Draws the rows `window` shows, each from `row(i)` in `style`, under a
/// clip of the strip, each bound to its id with the rect it is drawn at.
pub(crate) fn draw_rows<'a>(
    c: &mut Compositor,
    layer: LayerId,
    dd: Rect,
    window: &Window,
    row: impl Fn(usize) -> Row<'a>,
    style: &TextStyle,
    theme: &Theme,
) {
    let p = &theme.form.select.panel;
    let view = window.view;
    c.push_to_layer(
        layer,
        SceneNode::PushClip {
            x: view.x,
            y: view.y,
            w: view.w,
            h: view.h,
        },
    );
    for i in window.rows.clone() {
        let rect = window.row(i);
        let r = row(i);
        let oy = rect.y;
        if let Some(id) = &r.id {
            c.bind_to_layer(layer, id, [rect.x, oy, rect.w, rect.h]);
        }
        let fill = if r.selected {
            p.selected_fill
        } else if r.highlighted {
            p.hover_fill
        } else {
            Color::rgba(0.0, 0.0, 0.0, 0.0)
        };
        if fill.0[3] > 0.0 {
            c.push_to_layer(
                layer,
                rounded_rect(
                    dd.x + p.pad,
                    oy,
                    dd.w - p.pad * 2.0,
                    p.row_h,
                    p.row_radius.min(p.row_h / 2.0),
                    fill.0,
                ),
            );
        }
        let color = if r.highlighted || r.selected {
            p.active_color
        } else {
            p.text_color
        };
        let room = (dd.w - p.text_inset * 2.0).max(0.0);
        let label = TextMeasurer::truncate_to_width(r.label, style, room);
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&label, style, None),
                x: dd.x + p.text_inset,
                y: oy + TextMeasurer::vertical_center(style, p.row_h),
                color: color.0,
            },
        );
        if let (true, Some(dot)) = (r.selected, p.marker) {
            // A dot on the right marks the active option.
            c.push_to_layer(
                layer,
                rounded_rect(
                    dd.x + dd.w - p.text_inset - dot,
                    oy + (p.row_h - dot) / 2.0,
                    dot,
                    dot,
                    dot / 2.0,
                    p.marker_color.0,
                ),
            );
        }
    }
    c.push_to_layer(layer, SceneNode::PopClip);
}
