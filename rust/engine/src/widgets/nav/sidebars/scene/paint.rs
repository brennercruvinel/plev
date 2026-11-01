//! The scene sidebar's drawing: the head, the tab band, the rows and
//! their toggles or the asset sections, the search at the foot.

use crate::drawing::compositor::Compositor;
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::Theme;
use crate::widgets::icons;
use crate::widgets::recipe::{
    focus_ring, gradient_fill, rounded_rect, rounded_rect_stroke, shadow_node,
};
use crate::widgets::widget::Rect;

use super::super::{rule, shell, text};
use super::{ScenePart, SceneSidebar, SceneTab, TOGGLES};

impl SceneSidebar {
    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let t = &theme.navigation.sidebars;
        let k = &t.canvas;
        let l = self.layout(bounds, theme);
        let frame = Rect::new(bounds.x, bounds.y, bounds.w, bounds.h.max(t.min_h));
        shell(c, frame, true, theme);
        if let Some(id) = &self.sem {
            c.bind(id, [frame.x, frame.y, frame.w, frame.h]);
        }
        let x = bounds.x + k.pad;
        // the logo's symbol, the panel button
        let logo = super::super::logo_rect(&self.logo, bounds, k.pad, theme);
        self.logo.render(c, logo, theme);
        self.button(c, ScenePart::Panel, l.panel, "kit-panel", theme);
        // the project's title, its chevron, its kind
        let style = k.title.style(&theme.typography);
        c.push(text(&self.title, &style, x, l.title_y, k.ink.0, None));
        let tile = Rect::new(
            l.project.right() - k.chevron_tile,
            l.title_y,
            k.chevron_tile,
            k.chevron_tile,
        );
        if self.hovered == Some(ScenePart::Project) {
            c.push(rounded_rect(
                tile.x,
                tile.y,
                tile.w,
                tile.h,
                k.button_radius,
                k.button_hover.0,
            ));
        }
        let ci = tile.x + (tile.w - k.chevron) / 2.0;
        if let Some(n) = icons::icon_at(
            "kit-chevron-down",
            k.chevron,
            k.ink.0,
            ci,
            tile.y + (tile.h - k.chevron) / 2.0,
        ) {
            c.push(n);
        }
        self.ring(c, ScenePart::Project, l.project, k.button_radius, theme);
        let sub = k.subtitle.style(&theme.typography);
        c.push(text(
            &self.subtitle,
            &sub,
            x,
            l.subtitle_y,
            k.subtle.0,
            None,
        ));
        rule(c, frame.x, l.head_rule, frame.w, theme);
        self.tabs.render(c, l.tabs, theme);
        rule(c, frame.x, l.tabs_rule, frame.w, theme);
        c.push_clip(l.body.x, l.body.y, l.body.w, l.body.h);
        match self.current_tab() {
            SceneTab::Scene => {
                for (i, r) in l.items.iter().enumerate() {
                    self.row(c, i, *r, theme);
                }
            }
            SceneTab::Assets => {
                for (s, r) in self.sections.iter().zip(&l.sections) {
                    let h = self.grid_h(s, r.w, theme);
                    s.head.render(c, *r, h, theme, |c, at| {
                        for cell in self.cells(s, at, theme) {
                            picture(c, cell, s.large, theme);
                        }
                    });
                }
            }
        }
        c.pop_clip();
        rule(c, frame.x, l.search_rule, frame.w, theme);
        self.search.render(c, l.search, theme);
    }

    fn ring(&self, c: &mut Compositor, part: ScenePart, r: Rect, radius: f32, theme: &Theme) {
        if self.focused == Some(part) {
            c.push(focus_ring(r, radius, theme));
        }
    }

    fn button(&self, c: &mut Compositor, part: ScenePart, r: Rect, icon: &str, theme: &Theme) {
        let k = &theme.navigation.sidebars.canvas;
        let hot = self.hovered == Some(part);
        if hot {
            c.push(rounded_rect(
                r.x,
                r.y,
                r.w,
                r.h,
                k.button_radius,
                k.button_hover.0,
            ));
        }
        let tone = if hot { k.ink } else { k.mark };
        let i = (r.w - k.button_icon) / 2.0;
        if let Some(n) = icons::icon_at(icon, k.button_icon, tone.0, r.x + i, r.y + i) {
            c.push(n);
        }
        self.ring(c, part, r, k.button_radius, theme);
    }

    fn row(&self, c: &mut Compositor, i: usize, r: Rect, theme: &Theme) {
        let k = &theme.navigation.sidebars.canvas;
        let item = &self.items[i];
        let selected = self.selected == Some(i);
        let hot =
            matches!(self.hovered, Some(ScenePart::Item(h) | ScenePart::Toggle(h, _)) if h == i);
        if let Some(id) = self.part_id(ScenePart::Item(i)) {
            c.bind(&id, [r.x, r.y, r.w, r.h]);
        }
        if selected || hot {
            c.push(rounded_rect(r.x, r.y, r.w, r.h, k.row_radius, k.row_fill.0));
        }
        if selected {
            c.push(rounded_rect_stroke(
                r.x,
                r.y,
                r.w,
                r.h,
                k.row_radius,
                k.row_edge.0,
                1.0,
            ));
        }
        self.ring(c, ScenePart::Item(i), r, k.row_radius, theme);
        let tile = Rect::new(
            r.x + k.row_pad + 1.0,
            r.y + (r.h - k.tile) / 2.0,
            k.tile,
            k.tile,
        );
        let raised = selected || hot;
        if selected && let Some(n) = shadow_node(tile, k.tile_radius, &k.tile_shadow) {
            c.push(n);
        }
        let fill = if raised { k.tile_raised } else { k.tile_fill };
        c.push(rounded_rect(
            tile.x,
            tile.y,
            tile.w,
            tile.h,
            k.tile_radius,
            fill.0,
        ));
        let ii = (k.tile - k.tile_icon) / 2.0;
        if let Some(n) = icons::icon_at(item.icon, k.tile_icon, k.ink.0, tile.x + ii, tile.y + ii) {
            c.push(n);
        }
        let toggles = self.toggle_rects(r, theme);
        let shown: Vec<usize> = (0..3).filter(|t| hot || item.on[*t]).collect();
        let end = shown
            .first()
            .map_or(r.right() - k.label_gap, |t| toggles[*t].x - k.toggle_gap);
        let style = k.label.style(&theme.typography);
        let lx = tile.right() + k.label_gap;
        let ly = r.y + (r.h - style.line_height) / 2.0;
        // one line, as a layer's row: a name longer than the room left by
        // the toggles is cut with an ellipsis (its projection keeps it)
        let room = (end - lx).max(0.0);
        let title = TextMeasurer::truncate_to_width(&item.title, &style, room);
        c.push(text(&title, &style, lx, ly, k.ink.0, Some(room)));
        for t in shown {
            let (_, off, on) = TOGGLES[t];
            let lit = item.on[t] || self.hovered == Some(ScenePart::Toggle(i, t));
            let tone = if lit { k.ink } else { k.mark };
            let b = toggles[t];
            if let Some(id) = self.part_id(ScenePart::Toggle(i, t)) {
                c.bind(&id, [b.x, b.y, b.w, b.h]);
            }
            let icon = if item.on[t] { on } else { off };
            if let Some(n) = icons::icon_at(icon, k.toggle, tone.0, b.x, b.y) {
                c.push(n);
            }
            self.ring(c, ScenePart::Toggle(i, t), b, k.button_radius, theme);
        }
    }
}

/// An asset's tile: its well and its picture's stand-in, centered.
fn picture(c: &mut Compositor, cell: Rect, large: bool, theme: &Theme) {
    let k = &theme.navigation.sidebars.canvas;
    c.push(rounded_rect(
        cell.x,
        cell.y,
        cell.w,
        cell.h,
        k.cell_radius,
        k.tile_fill.0,
    ));
    let room = (cell.w - k.cell_pad * 2.0).max(0.0);
    let side = if large { k.picture_large } else { k.picture }.min(room);
    let at = Rect::new(
        cell.x + (cell.w - side) / 2.0,
        cell.y + (cell.h - side) / 2.0,
        side,
        side,
    );
    let [top, bottom] = k.picture_fill;
    c.push(gradient_fill(at, k.tile_radius, top.0, bottom.0));
}
