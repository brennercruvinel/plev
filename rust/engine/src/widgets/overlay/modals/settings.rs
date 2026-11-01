//! Settings (`components/Settings`): a side menu of five sections beside
//! the one it picks. General: the username edited in place, the email,
//! three switches and the language; Profile: private, the avatar's
//! upload, the portfolio link, the display name; Security: the password,
//! multi-factor authentication, logging out of every device;
//! Notifications: how to show them while working and three switches;
//! Subscription: the renewal, the plan with its price, its cancel and
//! upgrade, and its features.
//!
//! Projected: a `Role::Group` named "Settings", the menu as a tab list
//! (`<id>.section`) and the shown section's controls (`<id>.general.*`,
//! `<id>.profile.*`, `<id>.security.*`, `<id>.notifications.*`,
//! `<id>.subscription.*`).

use accord::{Role, SemId, SnapshotBuilder, Value};

use crate::drawing::compositor::{Compositor, LayerId};
use crate::styling::theme::Theme;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

use super::nav::Nav;
use super::settings_parts::{TITLES, controls, walk};
use super::{Controls, ModalBody, hairline, panel, tokens, vline};

pub struct Settings {
    nav: Nav,
    nav_controls: Controls,
    sections: Vec<Controls>,
}

struct Layout {
    stacked: bool,
    items: Vec<Rect>,
    rule: Rect,
    section: Rect,
    controls: Vec<Rect>,
    height: f32,
}

impl Settings {
    /// The general section shown, under `id`.
    pub fn new(id: Option<&SemId>) -> Self {
        let icons = ["settings", "user-check", "fingerprint", "zap", "wallet"];
        let (nav, nav_controls) = Nav::new(icons.into_iter().zip(TITLES).collect(), id);
        let sections = (0..TITLES.len()).map(|s| controls(s, id)).collect();
        Self {
            nav,
            nav_controls,
            sections,
        }
    }

    pub fn section(&self) -> usize {
        self.nav.active
    }

    fn layout(&self, rect: Rect, theme: &Theme) -> Layout {
        let t = tokens(theme);
        let (st, ink) = (&t.settings, &t.ink);
        let n = &st.nav;
        let stacked = Nav::stacked(rect, st.panel.width);
        let column_w = if stacked { rect.w } else { n.width.min(rect.w) };
        let (items, items_end) = self.nav.rects(Rect::new(rect.x, rect.y, column_w, 0.0), n);
        let column_h = items_end + n.pad - rect.y;
        let active = self.nav.active;
        let mut controls = Vec::new();
        let (rule, section, height);
        if stacked {
            rule = Rect::new(rect.x, rect.y + column_h, rect.w, ink.rule_w);
            let probe = Rect::new(rect.x, rule.bottom(), rect.w, 0.0);
            let h = walk(
                active,
                &self.sections[active],
                probe,
                theme,
                &mut controls,
                None,
            );
            section = Rect::new(probe.x, probe.y, probe.w, h);
            height = column_h + ink.rule_w + h;
        } else {
            let sx = rect.x + column_w + ink.rule_w;
            let probe = Rect::new(sx, rect.y, rect.right() - sx, 0.0);
            let h = walk(
                active,
                &self.sections[active],
                probe,
                theme,
                &mut controls,
                None,
            );
            height = st.min_h.max(column_h).max(h);
            section = Rect::new(sx, rect.y, probe.w, h);
            rule = Rect::new(rect.x + column_w, rect.y, ink.rule_w, height);
        }
        Layout {
            stacked,
            items,
            rule,
            section,
            controls,
            height,
        }
    }
}

impl ModalBody for Settings {
    fn title(&self) -> &str {
        "Settings"
    }

    fn size(&self, w: f32, theme: &Theme) -> (f32, f32) {
        let w = tokens(theme).settings.panel.width.min(w);
        (w, self.layout(Rect::new(0.0, 0.0, w, 0.0), theme).height)
    }

    fn handle_event(&mut self, event: &WidgetEvent, rect: Rect, theme: &Theme) -> EventResult {
        let l = self.layout(rect, theme);
        let active = self.nav.active;
        if self.sections[active].any_open() {
            return self.sections[active]
                .handle_event(event, &l.controls, theme)
                .1;
        }
        let (clicked, r) = self.nav_controls.handle_event(event, &l.items, theme);
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
        let (st, ink) = (&t.settings, &t.ink);
        panel(c, rect, &st.panel, theme);
        let l = self.layout(rect, theme);
        if l.stacked {
            hairline(c, l.rule.x, l.rule.y, l.rule.w, ink.rule_w, ink.rule);
        } else {
            vline(c, l.rule.x, l.rule.y, l.rule.h, ink.rule_w, ink.rule);
        }
        self.nav
            .draw(c, &l.items, &self.nav_controls, &st.nav, ink, theme);
        self.nav_controls.render(c, &l.items, theme);
        let active = self.nav.active;
        let mut scratch = Vec::new();
        walk(
            active,
            &self.sections[active],
            l.section,
            theme,
            &mut scratch,
            Some(c),
        );
        self.sections[active].render(c, &l.controls, theme);
    }

    fn render_overlay(&self, c: &mut Compositor, layer: LayerId, rect: Rect, theme: &Theme) {
        let l = self.layout(rect, theme);
        self.sections[self.nav.active].render_overlay(c, layer, &l.controls, theme);
    }

    fn project(&self, b: &mut SnapshotBuilder, id: &SemId, parent: &SemId) {
        let active = self.nav.active;
        b.node(id, Role::Group, self.title())
            .parent(parent)
            .description(TITLES[active]);
        self.nav.project(b, id, "Settings sections");
        self.nav_controls.project(b, id);
        self.sections[active].project(b, id);
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
        let active = self.nav.active;
        self.sections[active]
            .run(target, action, param)
            .map(|(_, r)| r)
    }

    fn focus(&mut self, id: Option<&SemId>) -> bool {
        let active = self.nav.active;
        let a = self.nav_controls.focus(id);
        let b = self.sections[active].focus(id);
        a || b
    }

    fn has_open(&self) -> bool {
        self.sections[self.nav.active].any_open()
    }

    fn text_mut(&mut self) -> Option<&mut crate::widgets::form::TextField> {
        let active = self.nav.active;
        self.sections[active].text_mut()
    }

    fn ime_area(&self, rect: Rect, theme: &Theme) -> Option<Rect> {
        let l = self.layout(rect, theme);
        self.sections[self.nav.active].ime_area(&l.controls, theme)
    }

    fn enter(&mut self) -> bool {
        let active = self.nav.active;
        self.sections[active].enter()
    }

    fn escape_text(&mut self) -> bool {
        let active = self.nav.active;
        self.sections[active].escape()
    }

    fn end_compose(&mut self) -> bool {
        self.sections
            .iter_mut()
            .fold(false, |d, s| s.end_compose() | d)
    }

    fn close(&mut self) -> bool {
        let active = self.nav.active;
        self.sections[active].close()
    }

    fn tick(&mut self, dt: f32) -> bool {
        let mut moving = false;
        for s in &mut self.sections {
            moving |= s.tick(dt);
        }
        moving
    }
}
