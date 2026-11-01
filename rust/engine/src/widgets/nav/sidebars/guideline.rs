//! The kit's GuidelineSidebar (`components/GuidelineSidebar`): the logo,
//! then a caption and the kit's links per group, the current page's link
//! raised. A press on a link makes it current, as following it would.
//!
//! With a [`SemId`] the links are `<id>[<n>]`, counted across the
//! groups, and [`GuidelineSidebar::project`] declares a `Role::Region`
//! named by the label, a `Role::Group` per caption, its links in it.

use accord::{Role, SemId, SnapshotBuilder};

use crate::drawing::compositor::{Compositor, LayerId};
use crate::styling::theme::Theme;
use crate::widgets::nav::NavLink;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

use super::{PRODUCT, caption, caption_h, head_h, logo_rect, shell};
use crate::widgets::header::Logo;

/// A caption and its links.
#[derive(Clone, Debug)]
pub struct GuideGroup {
    pub title: String,
    pub links: Vec<NavLink>,
}

#[derive(Clone, Debug)]
pub struct GuidelineSidebar {
    pub groups: Vec<GuideGroup>,
    pub logo: Logo,
    sem: Option<SemId>,
}

impl GuidelineSidebar {
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            logo: Logo::new(PRODUCT),
            sem: None,
        }
    }

    /// A group of links, `(label, icon)`.
    pub fn group(mut self, title: impl Into<String>, links: &[(&str, &'static str)]) -> Self {
        let links = links
            .iter()
            .map(|(label, icon)| NavLink::new(*label).icon(icon))
            .collect();
        self.groups.push(GuideGroup {
            title: title.into(),
            links,
        });
        self
    }

    /// The link at `n`, counted across the groups, is the current one.
    pub fn current(mut self, n: usize) -> Self {
        self.set_current(n);
        self
    }

    pub fn sem(mut self, id: SemId) -> Self {
        if let Ok(l) = id.child("logo") {
            self.logo = self.logo.clone().sem(l);
        }
        let links = self.groups.iter_mut().flat_map(|g| g.links.iter_mut());
        for (n, link) in links.enumerate() {
            *link = link.clone().sem(id.key(n.to_string()));
        }
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// `<id>.logo`, the link home.
    pub fn logo_id(&self) -> Option<&SemId> {
        self.logo.sem_id()
    }

    fn links(&self) -> impl Iterator<Item = &NavLink> {
        self.groups.iter().flat_map(|g| g.links.iter())
    }

    fn links_mut(&mut self) -> impl Iterator<Item = &mut NavLink> {
        self.groups.iter_mut().flat_map(|g| g.links.iter_mut())
    }

    pub fn len(&self) -> usize {
        self.links().count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn current_link(&self) -> Option<usize> {
        self.links().position(|l| l.active)
    }

    pub fn set_current(&mut self, n: usize) {
        for (i, link) in self.links_mut().enumerate() {
            link.active = i == n;
        }
    }

    /// The link `id` names.
    pub fn position(&self, id: &SemId) -> Option<usize> {
        self.links().position(|l| l.sem_id() == Some(id))
    }

    /// Keyboard focus on a link, or on none.
    pub fn set_focused(&mut self, n: Option<usize>) {
        for (i, link) in self.links_mut().enumerate() {
            link.set_focused(n == Some(i));
        }
    }

    /// As tall as its rows, never shorter than the kit's `min-h-219`.
    pub fn size(&self, theme: &Theme) -> (f32, f32) {
        let t = &theme.navigation.sidebars;
        let link = &theme.navigation.link;
        let mut h = head_h(theme);
        for (g, group) in self.groups.iter().enumerate() {
            let n = group.links.len() as f32;
            h += caption_h(theme) + link.height * n + link.link_gap * (n - 1.0).max(0.0);
            if g + 1 < self.groups.len() {
                h += t.group_gap;
            }
        }
        (t.width, (h + t.body_pad).max(t.min_h))
    }

    /// The caption's top and the links' rects, by group.
    pub fn layout(&self, bounds: Rect, theme: &Theme) -> (Vec<f32>, Vec<Rect>) {
        let t = &theme.navigation.sidebars;
        let link = &theme.navigation.link;
        let (x, w) = (bounds.x + t.body_pad, bounds.w - t.body_pad * 2.0);
        let mut y = bounds.y + head_h(theme);
        let (mut captions, mut rects) = (Vec::new(), Vec::new());
        for group in &self.groups {
            captions.push(y);
            y += caption_h(theme);
            for _ in &group.links {
                rects.push(Rect::new(x, y, w, link.height));
                y += link.height + link.link_gap;
            }
            y += t.group_gap - link.link_gap;
        }
        (captions, rects)
    }

    /// Hover and press on the links; a release on the pressed one makes
    /// it current (`clicked`).
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let (_, rects) = self.layout(bounds, theme);
        let pad = theme.navigation.sidebars.head_pad;
        let at = logo_rect(&self.logo, bounds, pad, theme);
        let mut out = self.logo.handle_event(event, at);
        let mut picked = None;
        for (i, (link, r)) in self.links_mut().zip(rects).enumerate() {
            let res = link.handle_event(event, r);
            if res.clicked {
                picked = Some(i);
            }
            out = out.merge(res);
        }
        if let Some(n) = picked {
            self.set_current(n);
        }
        out
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        shell(c, bounds, false, theme);
        let t = &theme.navigation.sidebars;
        let at = logo_rect(&self.logo, bounds, t.head_pad, theme);
        self.logo.render(c, at, theme);
        let t = &theme.navigation.sidebars;
        let (captions, rects) = self.layout(bounds, theme);
        let (x, w) = (bounds.x + t.body_pad, bounds.w - t.body_pad * 2.0);
        for (group, y) in self.groups.iter().zip(captions) {
            caption(c, &group.title, x, y, w, theme);
        }
        if let Some(id) = &self.sem {
            c.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        for (link, r) in self.links().zip(rects) {
            link.render_with(c, LayerId::DEFAULT, r, theme, &theme.navigation.link);
        }
    }

    /// A `Role::Region` named `label`; a group per caption,
    /// `<id>.group[<g>]`, its links in it.
    #[track_caller]
    pub fn project(
        &self,
        b: &mut SnapshotBuilder,
        id: &SemId,
        label: &str,
        parent: Option<&SemId>,
    ) {
        let mut node = b.node(id, Role::Region, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        if let Some(l) = self.logo.sem_id() {
            self.logo.project(b, l, Some(id));
        }
        for (g, group) in self.groups.iter().enumerate() {
            let Ok(family) = id.child("group") else {
                continue;
            };
            let at = family.key(g.to_string());
            b.node(&at, Role::Group, group.title.as_str()).parent(id);
            for link in &group.links {
                if let Some(lid) = link.sem_id() {
                    link.project(b, lid, Some(&at));
                }
            }
        }
    }
}

impl Default for GuidelineSidebar {
    fn default() -> Self {
        Self::new()
    }
}
