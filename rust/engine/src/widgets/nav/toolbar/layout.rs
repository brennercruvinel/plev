//! The toolbar's geometry: one row as the kit lays it, or, in a room
//! narrower than that row ([`Toolbar::size_in`]), the parts wrapped into
//! rows inside the bar and the export side on a row of its own under a
//! rule across. The rule is the bar's own width: nothing is a breakpoint.

use crate::styling::theme::Theme;
use crate::widgets::widget::Rect;

use super::{Toolbar, ToolbarLayout};

impl Toolbar {
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
        if bounds.w + 0.5 < w {
            return self.wrapped(bounds, theme);
        }
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

    /// The bar's size in `max_w`: its one row while it fits, else as
    /// wide as the room and as tall as its wrapped rows.
    pub fn size_in(&self, max_w: f32, theme: &Theme) -> (f32, f32) {
        let (w, h) = self.preferred_size(theme);
        if w <= max_w + 0.5 {
            return (w, h);
        }
        let r = Self::recipe(theme);
        let l = self.wrapped(Rect::new(0.0, 0.0, max_w, 0.0), theme);
        (max_w, l.split.bottom() + r.pad * 2.0 + r.tool.size)
    }

    /// The parts flowed left to right, a new row when the next would
    /// pass the bar's inner end; the export on its own row under a rule.
    fn wrapped(&self, bounds: Rect, theme: &Theme) -> ToolbarLayout {
        let r = Self::recipe(theme);
        let (x0, x1) = (bounds.x + r.pad, bounds.right() - r.pad);
        let (mut x, mut y) = (x0, bounds.y + r.pad);
        let mut place = |w: f32| {
            if x > x0 && x + w > x1 {
                x = x0;
                y += r.tool.size + r.gap;
            }
            let at = Rect::new(x, y, w, r.tool.size);
            x += w + r.gap;
            at
        };
        let mut tools = Vec::with_capacity(self.tools.len());
        let mut divider = None;
        for i in 0..self.tools.len() {
            tools.push(place(r.tool.size));
            if self.divider_after == Some(i + 1) {
                divider = Some(place(r.border_width));
            }
        }
        let zoom = place(self.zoom_w(theme));
        let history: Vec<Rect> = self.history.iter().map(|_| place(r.tool.size)).collect();
        let last = history.last().copied().unwrap_or(zoom);
        let split = Rect::new(bounds.x, last.bottom() + r.pad, bounds.w, r.border_width);
        let export_h = self.export.preferred_size(theme).1;
        let row = split.bottom() + r.pad;
        let export = Rect::new(
            x0,
            row + (r.tool.size - export_h) / 2.0,
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
}
