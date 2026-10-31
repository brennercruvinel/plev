//! Export (`components/Export`): a side menu of what to export (images,
//! video, 3D object) over the preview of what is exported, beside the
//! section the menu picks. Images: the sizes (each a scale, a color
//! profile, a format and a remove), the add, the compression; video: the
//! camera, the format, the frame rate and the resolution with the time
//! and size it estimates; 3D object: the format, the camera and the
//! material. Each ends in the export button. The preview's button
//! expands the still or plays the video.
//!
//! Projected: a `Role::Group` named "Export", the menu as a tab list
//! (`<id>.section`), the preview's button (`<id>.preview`), and the
//! shown section's controls (`<id>.images.*`, `<id>.video.*`,
//! `<id>.object.*`).

use accord::{SemId, SnapshotBuilder, Value};

use crate::drawing::compositor::{Compositor, LayerId};
use crate::styling::theme::Theme;
use crate::widgets::recipe::focus_ring;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

use super::export_parts::{SECTIONS, Section, controls, walk};
use super::nav::Nav;
use super::{
    Controls, Item, ItemRole, Kind, ModalBody, boxed, child, glyph, hairline, panel, picture,
    raise, stroke, tokens, vline,
};

pub struct Export {
    pub title: String,
    nav: Nav,
    nav_controls: Controls,
    preview: Controls,
    sections: Vec<Controls>,
}

/// Where the parts go.
struct Layout {
    stacked: bool,
    items: Vec<Rect>,
    preview: Rect,
    button: Rect,
    /// The rule between the menu and the section, and the section.
    rule: Rect,
    section: Rect,
    controls: Vec<Rect>,
    height: f32,
}

impl Export {
    /// Exporting `title`, the images section shown, under `id`.
    pub fn new(title: &str, id: Option<&SemId>) -> Self {
        let (nav, nav_controls) = Nav::new(
            vec![
                ("image", "Images"),
                ("video", "Video"),
                ("box", "3D Object"),
            ],
            id,
        );
        let mut preview = Controls::default();
        preview.push(
            Kind::Item(Item::new("Preview", ItemRole::Button)),
            child(id, "preview"),
        );
        let sections = SECTIONS.iter().map(|s| controls(*s, id, title)).collect();
        Self {
            title: title.into(),
            nav,
            nav_controls,
            preview,
            sections,
        }
    }

    pub fn section(&self) -> usize {
        self.nav.active
    }

    fn shown(&self) -> Section {
        SECTIONS[self.nav.active]
    }

    fn layout(&self, rect: Rect, theme: &Theme) -> Layout {
        let t = tokens(theme);
        let (e, ink) = (&t.export, &t.ink);
        let n = &e.nav;
        let p = &e.preview;
        let stacked = Nav::stacked(rect, e.panel.width);
        let column_w = if stacked { rect.w } else { n.width.min(rect.w) };
        let column = Rect::new(rect.x, rect.y, column_w, 0.0);
        let (items, items_end) = self.nav.rects(column, n);
        let pw = (n.width.min(rect.w) - n.pad * 2.0).max(0.0);
        let ph = pw * p.aspect;
        let column_h = items_end + p.top + ph + n.pad - rect.y;
        let mut controls = Vec::new();
        let (rule, section, height);
        let preview;
        if stacked {
            preview = Rect::new(rect.x + n.pad, items_end + p.top, pw, ph);
            rule = Rect::new(rect.x, rect.y + column_h, rect.w, ink.rule_w);
            let top = rule.bottom();
            let probe = Rect::new(rect.x, top, rect.w, 0.0);
            let h = walk(
                self.shown(),
                &self.sections[self.nav.active],
                probe,
                theme,
                &mut controls,
                None,
            );
            section = Rect::new(rect.x, top, rect.w, h);
            walk(
                self.shown(),
                &self.sections[self.nav.active],
                section,
                theme,
                &mut controls,
                None,
            );
            height = column_h + ink.rule_w + h;
        } else {
            let sx = rect.x + column_w + ink.rule_w;
            let probe = Rect::new(sx, rect.y, rect.right() - sx, 0.0);
            let natural = walk(
                self.shown(),
                &self.sections[self.nav.active],
                probe,
                theme,
                &mut controls,
                None,
            );
            height = e.min_h.max(column_h).max(natural);
            section = Rect::new(sx, rect.y, probe.w, height);
            walk(
                self.shown(),
                &self.sections[self.nav.active],
                section,
                theme,
                &mut controls,
                None,
            );
            rule = Rect::new(rect.x + column_w, rect.y, ink.rule_w, height);
            preview = Rect::new(rect.x + n.pad, rect.y + height - n.pad - ph, pw, ph);
        }
        let button = Rect::new(
            preview.right() - p.button_inset - p.button,
            preview.y + p.button_inset,
            p.button,
            p.button,
        );
        Layout {
            stacked,
            items,
            preview,
            button,
            rule,
            section,
            controls,
            height,
        }
    }
}

impl ModalBody for Export {
    fn title(&self) -> &str {
        "Export"
    }

    fn size(&self, w: f32, theme: &Theme) -> (f32, f32) {
        let w = tokens(theme).export.panel.width.min(w);
        (w, self.layout(Rect::new(0.0, 0.0, w, 0.0), theme).height)
    }

    fn handle_event(&mut self, event: &WidgetEvent, rect: Rect, theme: &Theme) -> EventResult {
        let l = self.layout(rect, theme);
        let active = self.nav.active;
        let section = &mut self.sections[active];
        if section.any_open() {
            return section.handle_event(event, &l.controls, theme).1;
        }
        let (clicked, r) = self.nav_controls.handle_event(event, &l.items, theme);
        let r = r.merge(self.preview.handle_event(event, &[l.button], theme).1);
        let r = r.merge(
            self.sections[active]
                .handle_event(event, &l.controls, theme)
                .1,
        );
        if let Some(i) = clicked
            && self.nav.activate(&mut self.nav_controls, i)
        {
            return r.merge(EventResult::changed());
        }
        r
    }

    fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        let t = tokens(theme);
        let (e, ink) = (&t.export, &t.ink);
        panel(c, rect, &e.panel, theme);
        let l = self.layout(rect, theme);
        if l.stacked {
            hairline(c, l.rule.x, l.rule.y, l.rule.w, ink.rule_w, ink.rule);
        } else {
            vline(c, l.rule.x, l.rule.y, l.rule.h, ink.rule_w, ink.rule);
        }
        self.nav
            .draw(c, &l.items, &self.nav_controls, &e.nav, ink, theme);
        self.nav_controls.render(c, &l.items, theme);
        // The preview, its frame, its button.
        let p = &e.preview;
        picture(c, l.preview, p.radius, t.pictures.preview);
        stroke(c, l.preview, p.radius, p.border, ink.rule_w);
        raise(c, l.button, p.button_radius, &p.button_shadow);
        boxed(c, l.button, p.button_radius, ink.paper, None);
        let video = self.shown() == Section::Video;
        let (icon, size) = if video {
            ("play", p.play)
        } else {
            ("maximize-2", p.icon)
        };
        let hovered = self.preview.item(0).is_some_and(|i| i.press.hovered);
        glyph(
            c,
            icon,
            size,
            if hovered { ink.ink } else { ink.icon },
            l.button,
        );
        if self.preview.0[0].is_focused() {
            c.push(focus_ring(l.button, p.button_radius, theme));
        }
        self.preview.render(c, &[l.button], theme);
        let mut scratch = Vec::new();
        walk(
            self.shown(),
            &self.sections[self.nav.active],
            l.section,
            theme,
            &mut scratch,
            Some(c),
        );
        self.sections[self.nav.active].render(c, &l.controls, theme);
    }

    fn render_overlay(&self, c: &mut Compositor, layer: LayerId, rect: Rect, theme: &Theme) {
        let l = self.layout(rect, theme);
        self.sections[self.nav.active].render_overlay(c, layer, &l.controls, theme);
    }

    fn project(&self, b: &mut SnapshotBuilder, id: &SemId, parent: &SemId) {
        b.node(id, accord::Role::Group, self.title())
            .parent(parent)
            .description(format!("Export {}", self.title));
        self.nav.project(b, id, "Export as");
        self.nav_controls.project(b, id);
        self.preview.project(b, id);
        self.sections[self.nav.active].project(b, id);
    }

    fn run(
        &mut self,
        target: &SemId,
        action: &str,
        param: Option<&Value>,
    ) -> Option<Result<bool, String>> {
        if let Some((i, done)) = self.nav_controls.run(target, action, param) {
            return Some(done.map(|_| self.nav.activate(&mut self.nav_controls, i)));
        }
        if let Some((_, done)) = self.preview.run(target, action, param) {
            return Some(done);
        }
        let active = self.nav.active;
        self.sections[active]
            .run(target, action, param)
            .map(|(_, r)| r)
    }

    fn focus(&mut self, id: Option<&SemId>) -> bool {
        let active = self.nav.active;
        let a = self.nav_controls.focus(id);
        let b = self.preview.focus(id);
        let c = self.sections[active].focus(id);
        a || b || c
    }

    fn has_open(&self) -> bool {
        self.sections[self.nav.active].any_open()
    }

    fn close(&mut self) -> bool {
        let active = self.nav.active;
        self.sections[active].close()
    }
}
