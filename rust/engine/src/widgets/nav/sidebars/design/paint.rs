//! The design sidebar's drawing: the head, the tab band, each group's
//! title, end and body, and the open select's list over everything.

use crate::drawing::compositor::{Compositor, LayerId};
use crate::styling::theme::Theme;
use crate::widgets::icons;
use crate::widgets::recipe::{focus_ring, gradient_fill, rounded_rect, rounded_rect_stroke};
use crate::widgets::widget::Rect;

use super::super::{rule, shell, text};
use super::{Body, DesignSidebar, End};

/// An icon of `size` centered in `r`.
fn icon_in(c: &mut Compositor, name: &str, r: Rect, size: f32, color: [f32; 4]) {
    let (x, y) = (r.x + (r.w - size) / 2.0, r.y + (r.h - size) / 2.0);
    if let Some(n) = icons::icon_at(name, size, color, x, y) {
        c.push(n);
    }
}

/// A picture's stand-in: the canvas' gradient.
fn picture(c: &mut Compositor, r: Rect, radius: f32, theme: &Theme) {
    let [top, bottom] = theme.navigation.sidebars.canvas.picture_fill;
    c.push(gradient_fill(r, radius, top.0, bottom.0));
}

impl DesignSidebar {
    /// The tiles of a tile group in its body.
    pub fn cells(body: Rect, n: usize, cols: usize, theme: &Theme) -> Vec<Rect> {
        let d = &theme.navigation.sidebars.canvas.design;
        let colsf = cols as f32;
        let cell = (body.w - (colsf - 1.0) * d.gap) / colsf;
        (0..n)
            .map(|i| {
                let (col, row) = ((i % cols) as f32, (i / cols) as f32);
                Rect::new(
                    body.x + col * (cell + d.gap),
                    body.y + row * (cell + d.gap),
                    cell,
                    cell,
                )
            })
            .collect()
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let t = &theme.navigation.sidebars;
        let k = &t.canvas;
        let d = &k.design;
        let frame = Rect::new(bounds.x, bounds.y, bounds.w, bounds.h.max(t.min_h));
        shell(c, frame, true, theme);
        if let Some(id) = &self.sem {
            c.bind(id, [frame.x, frame.y, frame.w, frame.h]);
        }
        // who is here: the first avatar over the second, each ringed
        let head_h = self.head_h(theme);
        let ay = bounds.y + (head_h - d.avatar) / 2.0;
        for (i, tone) in d.avatars.iter().enumerate().rev() {
            let x = bounds.x + d.head_pad_start + (d.avatar - d.avatar_overlap) * i as f32;
            let (r, ring) = (d.avatar / 2.0, d.avatar_ring);
            c.push(rounded_rect(
                x - ring,
                ay - ring,
                d.avatar + ring * 2.0,
                d.avatar + ring * 2.0,
                r + ring,
                t.fill.0,
            ));
            c.push(rounded_rect(x, ay, d.avatar, d.avatar, r, tone.0));
        }
        self.share.render(c, self.share_rect(bounds, theme), theme);
        rule(c, frame.x, bounds.y + head_h, frame.w, theme);
        let (tabs, top) = self.band(bounds, theme);
        self.tabs.render(c, tabs, theme);
        rule(c, frame.x, top - t.edge_width, frame.w, theme);
        let style = d.header.style(&theme.typography);
        for (n, l) in self.layout(bounds, theme).into_iter().enumerate() {
            let block = &self.blocks[l.index];
            if n > 0 {
                rule(c, frame.x, l.header.y, frame.w, theme);
            }
            let ty = l.header.y + (l.header.h - style.line_height) / 2.0;
            c.push(text(
                &block.title,
                &style,
                l.header.x + k.pad,
                ty,
                k.ink.0,
                None,
            ));
            match &block.end {
                End::Switch(s) => s.render(c, l.end, theme),
                End::Add => icon_in(c, "kit-plus", l.end, d.add_icon, k.mark.0),
                End::Repeat => icon_in(c, "kit-loop", l.end, d.add_icon, k.mark.0),
            }
            self.body(c, l.index, l.body, theme);
        }
    }

    /// The button beside a select, the kit's own glyph.
    fn action(&self, c: &mut Compositor, select: Rect, icon: &str, theme: &Theme) {
        let d = &theme.navigation.sidebars.canvas.design;
        let r = Rect::new(
            select.right() + d.gap,
            select.y + (select.h - d.add) / 2.0,
            d.add,
            d.add,
        );
        icon_in(
            c,
            icon,
            r,
            d.add_icon,
            theme.navigation.sidebars.canvas.mark.0,
        );
    }

    /// A value's well: a field or a chip, its fill and radius.
    fn well(c: &mut Compositor, r: Rect, theme: &Theme) {
        let d = &theme.navigation.sidebars.canvas.design;
        c.push(rounded_rect(
            r.x,
            r.y,
            r.w,
            r.h,
            d.field_radius,
            d.field_fill.0,
        ));
    }

    /// A chip: its icon, then its value, centered.
    fn chip(c: &mut Compositor, r: Rect, icon: &str, value: &str, theme: &Theme) {
        let k = &theme.navigation.sidebars.canvas;
        let d = &k.design;
        Self::well(c, r, theme);
        let style = d.field_text.style(&theme.typography);
        let tw = crate::drawing::text::TextMeasurer::measure_styled(value, &style, None).0;
        let x = r.x + (r.w - d.chip_icon - d.stack_gap - tw) / 2.0;
        icon_in(
            c,
            icon,
            Rect::new(x, r.y, d.chip_icon, r.h),
            d.chip_icon,
            k.mark.0,
        );
        let tx = x + d.chip_icon + d.stack_gap;
        c.push(text(
            value,
            &style,
            tx,
            r.y + (r.h - style.line_height) / 2.0,
            k.ink.0,
            None,
        ));
    }

    fn body(&self, c: &mut Compositor, b: usize, r: Rect, theme: &Theme) {
        let k = &theme.navigation.sidebars.canvas;
        let d = &k.design;
        let style = d.field_text.style(&theme.typography);
        let mid = |row: Rect| row.y + (row.h - style.line_height) / 2.0;
        match &self.blocks[b].body {
            Body::None => {}
            Body::Artboard(select, values) => {
                let sr = Rect::new(r.x, r.y, r.w - d.gap - d.add, select.height_in(theme));
                select.render(c, sr, theme);
                self.action(c, sr, "kit-lock-on", theme);
                let y = sr.bottom() + d.gap;
                let fields = r.w - d.gap - d.unit_w;
                let fw = (fields - d.stack_gap) / 2.0;
                for (i, (prefix, value)) in ["W", "H"].iter().zip(values).enumerate() {
                    let f = Rect::new(r.x + (fw + d.stack_gap) * i as f32, y, fw, d.field_h);
                    Self::well(c, f, theme);
                    c.push(text(
                        prefix,
                        &style,
                        f.x + d.field_prefix_x,
                        mid(f),
                        k.subtle.0,
                        None,
                    ));
                    let at = Rect::new(
                        f.x + d.field_text_x,
                        f.y,
                        (f.w - d.field_text_x).max(0.0),
                        f.h,
                    );
                    value.render(c, at, theme);
                }
                let unit = Rect::new(r.right() - d.unit_w, y, d.unit_w, d.field_h);
                let uw = crate::drawing::text::TextMeasurer::measure_styled("px", &style, None).0;
                c.push(text(
                    "px",
                    &style,
                    unit.x + (unit.w - uw) / 2.0,
                    mid(unit),
                    k.subtle.0,
                    None,
                ));
            }
            Body::Tiles {
                n,
                cols,
                well,
                small,
                picked,
            } => {
                let radius = if *small {
                    d.tile_radius_small
                } else {
                    d.tile_radius
                };
                for (i, cell) in Self::cells(r, *n, *cols, theme).into_iter().enumerate() {
                    if let Some(id) = self.tile_id(b, i) {
                        c.bind(&id, [cell.x, cell.y, cell.w, cell.h]);
                    }
                    if *well {
                        c.push(rounded_rect(
                            cell.x,
                            cell.y,
                            cell.w,
                            cell.h,
                            radius,
                            k.tile_fill.0,
                        ));
                        let side = k.picture.min(cell.w - k.cell_pad * 2.0);
                        let at = Rect::new(
                            cell.x + (cell.w - side) / 2.0,
                            cell.y + (cell.h - side) / 2.0,
                            side,
                            side,
                        );
                        picture(c, at, k.tile_radius, theme);
                    } else {
                        picture(c, cell, radius, theme);
                    }
                    if *picked == Some(i) {
                        let ring = cell.inset(d.strip_handle / 2.0);
                        c.push(rounded_rect_stroke(
                            ring.x,
                            ring.y,
                            ring.w,
                            ring.h,
                            radius,
                            k.mark.0,
                            d.strip_handle * 0.75,
                        ));
                    }
                    let focused = self.focused.is_some() && self.focused == self.tile_id(b, i);
                    if focused {
                        c.push(focus_ring(cell, radius, theme));
                    }
                }
            }
            Body::Picker(p) => p.render(c, r, theme),
            Body::Camera(tabs, value) => {
                let tr = Rect::new(r.x, r.y, r.w, crate::widgets::nav::Tabs::height(theme));
                tabs.render(c, tr, theme);
                let ly = tr.bottom() + k.band_pad_y;
                c.push(text("Distortion", &style, r.x, ly, k.ink.0, None));
                let y = ly + style.line_height + d.gap;
                let range = Rect::new(r.x, y, r.w - d.chip_w - d.stack_gap, d.field_h);
                c.push(rounded_rect(
                    range.x,
                    range.y,
                    range.w,
                    range.h,
                    d.range_radius,
                    d.field_fill.0,
                ));
                let filled = range.w * value.clamp(0.0, 1.0);
                c.push(rounded_rect(
                    range.x,
                    range.y,
                    filled,
                    range.h,
                    d.range_radius,
                    d.range_fill.0,
                ));
                let chip = Rect::new(r.right() - d.chip_w, y, d.chip_w, d.field_h);
                Self::chip(c, chip, "kit-distortion", &format!("{value}"), theme);
            }
            Body::Loop(tabs, [from, to, total]) => {
                let strip = Rect::new(r.x, r.y, r.w, d.strip_h);
                picture(c, strip, d.strip_radius, theme);
                let (a, z) = (
                    strip.x + strip.w * from / total,
                    strip.x + strip.w * to / total,
                );
                c.push(rounded_rect(
                    a,
                    strip.y,
                    z - a,
                    strip.h,
                    0.0,
                    d.strip_span.0,
                ));
                for x in [a, z - d.strip_handle] {
                    c.push(rounded_rect(
                        x,
                        strip.y,
                        d.strip_handle,
                        strip.h,
                        0.0,
                        theme.navigation.sidebars.fill.0,
                    ));
                }
                let edge = theme.navigation.sidebars.edge;
                c.push(rounded_rect_stroke(
                    strip.x,
                    strip.y,
                    strip.w,
                    strip.h,
                    d.strip_radius,
                    edge.0,
                    1.0,
                ));
                if let Some(tr) = Self::inner_tabs(&self.blocks[b].body, r, theme) {
                    tabs.render(c, tr, theme);
                }
                let y = strip.bottom() + d.gap;
                let chip = Rect::new(r.right() - d.chip_w_narrow, y, d.chip_w_narrow, d.field_h);
                Self::chip(c, chip, "kit-duration", &format!("{}s", to - from), theme);
            }
            Body::Lens(list) => {
                let mut y = r.y;
                let actions = ["kit-view", "kit-more", "kit-more"];
                for (s, icon) in list.iter().zip(actions) {
                    let sr = Rect::new(r.x, y, r.w - d.gap - d.add, s.height_in(theme));
                    s.render(c, sr, theme);
                    self.action(c, sr, icon, theme);
                    y = sr.bottom() + d.stack_gap;
                }
            }
        }
    }

    /// The open select's list, over every card.
    pub fn render_overlay(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        for (b, i, r) in self.select_rects(bounds, theme) {
            let select = match &self.blocks[b].body {
                Body::Artboard(s, _) => Some(s),
                Body::Lens(list) => list.get(i),
                _ => None,
            };
            if let Some(s) = select.filter(|s| s.is_open()) {
                s.render_dropdown(c, LayerId::DEFAULT, r, theme);
            }
        }
    }
}
