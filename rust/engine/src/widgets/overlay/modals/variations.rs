//! Variations (`components/Variations`): a small panel that offers the
//! variations of a render, two to a row, "<n> of <count>" in its head and
//! a close. A press on a tile chooses it; the chosen and the hovered tile
//! wear the ring. The tiles stand in for the kit's renders with the
//! picture tokens (`theme.overlay.modals.pictures.variations`).
//!
//! Projected: a `Role::Group` named "Variations", described by its count,
//! the close (`<id>.close`) and the tiles as a group of radios
//! (`<id>.tile`, `<id>.tile[<i>]`, `select`).

use accord::{Role, SemId, SnapshotBuilder, Value};

use crate::drawing::compositor::Compositor;
use crate::styling::theme::Theme;
use crate::widgets::buttons::{IconButton, IconButtonSize, IconButtonVariant};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

use super::{
    Controls, Item, ItemRole, Kind, ModalBody, child, middle, panel, picture, stroke, text,
    text_end, text_w, tokens,
};

/// The close, then one tile per variation.
const CLOSE: usize = 0;

pub struct Variations {
    controls: Controls,
    count: usize,
    chosen: Option<usize>,
    tiles: Option<SemId>,
}

impl Variations {
    /// `count` variations, none chosen, under `id`.
    pub fn new(count: usize, id: Option<&SemId>) -> Self {
        let mut controls = Controls::default();
        let close = IconButton::new("x")
            .variant(IconButtonVariant::Ghost)
            .size(IconButtonSize::Sm);
        controls.push(Kind::Icon(close, "Close".into()), child(id, "close"));
        let tiles = child(id, "tile");
        for i in 0..count {
            let item = Item::new(format!("Variation {}", i + 1), ItemRole::Radio);
            controls.push(
                Kind::Item(item),
                tiles.as_ref().map(|t| t.key(i.to_string())),
            );
        }
        Self {
            controls,
            count,
            chosen: None,
            tiles,
        }
    }

    pub fn chosen(&self) -> Option<usize> {
        self.chosen
    }

    fn choose(&mut self, i: usize) -> bool {
        let changed = self.chosen != Some(i);
        self.chosen = Some(i);
        self.controls.choose(1..1 + self.count, self.chosen);
        changed
    }

    /// "<n> of <count>", n the chosen one, 0 for none.
    pub fn count_text(&self) -> String {
        let n = self.chosen.map_or(0, |i| i + 1);
        format!("{n} of {}", self.count)
    }

    /// The panel's width in a room `w` wide, and a tile's edge in it.
    fn tile(&self, w: f32, theme: &Theme) -> (f32, f32) {
        let v = &tokens(theme).variations;
        let w = v.panel.width.min(w);
        let cols = v.columns.max(1) as f32;
        let inner = w - v.pad * 2.0;
        (w, ((inner - v.tile_gap * (cols - 1.0)) / cols).max(0.0))
    }

    fn rects(&self, rect: Rect, theme: &Theme) -> Vec<Rect> {
        let v = &tokens(theme).variations;
        let (_, edge) = self.tile(rect.w, theme);
        let head_h = v.close.max(v.title.line_height);
        let close = Rect::new(
            rect.right() - v.pad - v.close,
            rect.y + v.pad_top,
            v.close,
            v.close,
        );
        let top = rect.y + v.pad_top + head_h + v.head_gap;
        let mut out = vec![close];
        for i in 0..self.count {
            let (col, row) = ((i % v.columns) as f32, (i / v.columns) as f32);
            out.push(Rect::new(
                rect.x + v.pad + col * (edge + v.tile_gap),
                top + row * (edge + v.tile_gap),
                edge,
                edge,
            ));
        }
        out
    }
}

impl ModalBody for Variations {
    fn title(&self) -> &str {
        "Variations"
    }

    fn size(&self, w: f32, theme: &Theme) -> (f32, f32) {
        let v = &tokens(theme).variations;
        let (w, edge) = self.tile(w, theme);
        let rows = self.count.div_ceil(v.columns.max(1)) as f32;
        let head_h = v.close.max(v.title.line_height);
        let grid = rows * edge + (rows - 1.0).max(0.0) * v.tile_gap;
        (w, v.pad_top + head_h + v.head_gap + grid + v.pad)
    }

    fn handle_event(&mut self, event: &WidgetEvent, rect: Rect, theme: &Theme) -> EventResult {
        let rects = self.rects(rect, theme);
        let (clicked, mut r) = self.controls.handle_event(event, &rects, theme);
        if let Some(i) = clicked.filter(|i| *i != CLOSE) {
            r = r.merge(EventResult::changed());
            self.choose(i - 1);
        }
        r
    }

    fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        let t = tokens(theme);
        let (v, ink) = (&t.variations, &t.ink);
        panel(c, rect, &v.panel, theme);
        let rects = self.rects(rect, theme);
        let head = Rect::new(
            rect.x + v.pad,
            rect.y + v.pad_top,
            rect.w - v.pad * 2.0,
            v.close,
        );
        let count = self.count_text();
        let count_w = text_w(&count, &v.count);
        let count_end = rects[CLOSE].x - v.count_gap;
        let title_w = (count_end - count_w - head.x - v.count_gap).max(0.0);
        text(
            c,
            self.title(),
            &v.title,
            head.x,
            middle(&v.title, head.y, head.h),
            title_w,
            ink.ink,
        );
        text_end(
            c,
            &count,
            &v.count,
            count_end,
            middle(&v.count, head.y, head.h),
            count_w,
            ink.muted,
        );
        for (i, r) in rects.iter().enumerate().skip(1) {
            let colors = t.pictures.variations[(i - 1) % t.pictures.variations.len()];
            picture(c, *r, v.tile_radius, colors);
            let lit = self.controls.item(i).is_some_and(Item::lit);
            let focused = self.controls.0[i].is_focused();
            if lit || focused {
                // The ring: the paper band inside, the line at the edge.
                let line = if focused { theme.colors.accent } else { v.ring };
                stroke(c, *r, v.tile_radius, ink.paper, v.ring_band);
                stroke(c, *r, v.tile_radius, line, v.ring_w);
            }
        }
        self.controls.render(c, &rects, theme);
    }

    fn project(&self, b: &mut SnapshotBuilder, id: &SemId, parent: &SemId) {
        b.node(id, Role::Group, self.title())
            .parent(parent)
            .description(self.count_text());
        if let Some(tiles) = &self.tiles {
            b.node(tiles, Role::Group, "Variations to choose")
                .parent(id)
                .state(
                    "selected",
                    self.chosen.map(|i| i.to_string()).unwrap_or_default(),
                );
        }
        self.controls.project(b, id);
    }

    fn run(
        &mut self,
        target: &SemId,
        action: &str,
        param: Option<&Value>,
    ) -> Option<Result<bool, String>> {
        let (i, done) = self.controls.run(target, action, param)?;
        Some(done.map(|changed| {
            if i == CLOSE {
                changed
            } else {
                self.choose(i - 1)
            }
        }))
    }

    fn focus(&mut self, id: Option<&SemId>) -> bool {
        self.controls.focus(id)
    }
}
