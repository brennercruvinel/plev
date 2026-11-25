//! The HomeSidebar's pixels: its surface, the logo, and the body clipped
//! under it (the rows, the library, the new folder's field).

use super::super::{caption, folder_row, logo_rect, shell, text};
use super::{HomeLayout, HomeRow, HomeSidebar};
use crate::drawing::compositor::Compositor;
use crate::styling::theme::Theme;
use crate::widgets::icons;
use crate::widgets::recipe::{focus_ring, rounded_rect, rounded_rect_stroke};
use crate::widgets::selection::FolderRow;
use crate::widgets::widget::Rect;

impl HomeSidebar {
    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        shell(c, bounds, false, theme);
        self.render_parts(c, bounds, theme);
    }

    /// Everything but its own surface (the fill and the rim): the parts
    /// on a surface the owner painted, a drawer's edge panel
    /// (`overlay::sheet::paint_panel`).
    pub fn render_parts(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let t = &theme.navigation.sidebars;
        // bound before its parts: a hit on one of them is the part's
        if let Some(id) = &self.sem {
            c.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let at = logo_rect(&self.logo, bounds, t.head_pad, theme);
        self.logo.render(c, at, theme);
        let l = self.layout(bounds, theme);
        let body = self.body_rect(bounds, theme);
        c.push_clip(body.x, body.y, body.w, body.h);
        self.render_body(c, &l, theme);
        c.pop_clip();
    }

    /// The rows, the library and the new folder's field, as laid out.
    fn render_body(&self, c: &mut Compositor, l: &HomeLayout, theme: &Theme) {
        let t = &theme.navigation.sidebars;
        let ft = &theme.selection.folders;
        let recipe = &theme.navigation.link;
        for (row, r) in self.rows.iter().zip(&l.rows) {
            match row {
                HomeRow::Menu(m, links) => {
                    let h = links.height(r.w, theme);
                    m.render(c, *r, h, theme, |c, at| links.render(c, at, theme));
                }
                HomeRow::Link(link) => link.render_with(c, c.target(), *r, theme, recipe),
            }
        }
        if self.library {
            caption(c, &self.caption, l.scenes.x, l.caption_y, l.scenes.w, theme);
            self.scenes
                .render_with(c, c.target(), l.scenes, theme, recipe);
        }
        for row in self.folder_rows() {
            let r = Self::folder_rect(row, l);
            if let Some(id) = self.folder_id(row) {
                c.bind(&id, [r.x, r.y, r.w, r.h]);
            }
            let (label, icon) = match row {
                FolderRow::New => ("New Folder", ("kit-folder-new", ft.mark.0)),
                FolderRow::Folder(i) => (
                    self.folders[i].as_str(),
                    ("kit-folder", ft.hues[i % ft.hues.len()].0),
                ),
            };
            let (hovered, focused) = (self.hovered == Some(row), self.focused == Some(row));
            folder_row(c, r, label, icon, hovered, focused, theme);
        }
        if let Some(f) = l.field {
            c.push(rounded_rect(
                f.x,
                f.y,
                f.w,
                f.h,
                t.field_radius,
                t.field_fill.0,
            ));
            c.push(rounded_rect_stroke(
                f.x,
                f.y,
                f.w,
                f.h,
                t.field_radius,
                t.field_edge.0,
                t.edge_width,
            ));
            let iy = f.y + (f.h - ft.icon) / 2.0;
            if let Some(n) = icons::icon_at(
                "kit-folder",
                ft.icon,
                t.field_mark.0,
                f.x + t.field_icon_x,
                iy,
            ) {
                c.push(n);
            }
            let at = Self::name_rect(f, theme);
            if self.name.is_empty() && !self.name.is_focused() {
                let style = theme.typography.run(ft.text);
                let y = f.y + (f.h - style.line_height) / 2.0;
                let shown = "Untitled Folder";
                c.push(text(
                    shown,
                    &style,
                    at.x,
                    y,
                    t.field_placeholder.0,
                    Some(at.w),
                ));
            } else {
                self.name.render(c, at, theme);
            }
            if self.focused == Some(FolderRow::New) {
                c.push(focus_ring(f, t.field_radius, theme));
            }
        }
    }
}
