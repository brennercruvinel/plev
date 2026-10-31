//! The drawer of the hoff core kit: the navigation the kit's `Layout`
//! slides in from the left below 1023 px (`components/GuidelineSidebar`
//! as a drawer), over a scrim: the app's name with a close button, then
//! groups of links, the current one marked. A link picked, the close
//! button, a press on the scrim and Escape close it. Every value is
//! `theme.overlay.drawer` ([`DrawerTokens`]).
//!
//! It composes the sheet's edge panel ([`super::sheet`]): the same
//! geometry (the token's extent, capped by the container less a touch
//! target of scrim), the same surface, the same slide with its overlay
//! entry's spring (a cut with reduced motion). What it adds is the
//! navigation inside, and that it is held to the left by default.
//!
//! With a [`SemId`] ([`Drawer::sem`]) the render binds the panel, its
//! close button (`<id>[close]`) and each link (`<id>[<i>]`, `i` counting
//! the links of every group in order); [`Drawer::project`] declares it:
//! a modal `Role::Dialog` named by the app's name, its groups and links.

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use super::sheet::{Edge, edge_rect, paint_panel, paint_scrim, slide};
use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::Theme;
use crate::widgets::buttons::{IconButton, IconButtonSize, IconButtonVariant};
use crate::widgets::icons;
use crate::widgets::recipe::{rounded_rect, shadow_node};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// A link: its label and its icon.
#[derive(Clone, Debug, PartialEq)]
pub struct DrawerLink {
    pub label: String,
    pub icon: &'static str,
}

/// A titled group of links.
#[derive(Clone, Debug, PartialEq)]
pub struct DrawerGroup {
    pub title: String,
    pub links: Vec<DrawerLink>,
}

/// What a person did on the drawer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawerAction {
    None,
    /// The close button, or a press on the scrim.
    Close,
    /// The link at this index (every group's, in order).
    Pick(usize),
}

/// The kit's navigation drawer.
#[derive(Clone, Debug)]
pub struct Drawer {
    pub edge: Edge,
    pub title: String,
    pub groups: Vec<DrawerGroup>,
    /// The link of the page on screen.
    pub current: Option<usize>,
    hovered: Option<usize>,
    pressed: Option<usize>,
    focused: Option<usize>,
    close: IconButton,
    sem: Option<SemId>,
}

impl Drawer {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            edge: Edge::Left,
            title: title.into(),
            groups: Vec::new(),
            current: None,
            hovered: None,
            pressed: None,
            focused: None,
            close: IconButton::new("x")
                .variant(IconButtonVariant::Ghost)
                .size(IconButtonSize::Md),
            sem: None,
        }
    }

    pub fn edge(mut self, edge: Edge) -> Self {
        self.edge = edge;
        self
    }

    pub fn group(mut self, title: impl Into<String>, links: &[(&str, &'static str)]) -> Self {
        let links = links
            .iter()
            .map(|(label, icon)| DrawerLink {
                label: (*label).to_owned(),
                icon,
            })
            .collect();
        self.groups.push(DrawerGroup {
            title: title.into(),
            links,
        });
        self
    }

    pub fn current(mut self, at: Option<usize>) -> Self {
        self.current = at;
        self
    }

    /// The identity the render binds the panel to; the close button
    /// binds as `<id>[close]`, each link as `<id>[<i>]`.
    pub fn sem(mut self, id: SemId) -> Self {
        self.close = self.close.sem(id.key("close"));
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// Every link, in order.
    pub fn links(&self) -> impl Iterator<Item = &DrawerLink> {
        self.groups.iter().flat_map(|g| g.links.iter())
    }

    pub fn link_count(&self) -> usize {
        self.groups.iter().map(|g| g.links.len()).sum()
    }

    /// Keyboard focus, driven by the owner: the close button (`close`)
    /// or a link; `(false, None)` clears it.
    pub fn set_focused(&mut self, close: bool, link: Option<usize>) {
        self.close.set_focused(close);
        self.focused = if close { None } else { link };
    }

    /// The panel settled in `bounds`: the token's extent across its edge.
    pub fn panel_rect(&self, theme: &Theme, bounds: Rect) -> Rect {
        edge_rect(self.edge, theme.overlay.drawer.panel.extent, bounds, theme)
    }

    fn link_h(theme: &Theme) -> f32 {
        let t = &theme.overlay.drawer;
        t.item_pad * 2.0 + t.icon_box
    }

    fn close_rect(&self, theme: &Theme, p: Rect) -> Rect {
        let t = &theme.overlay.drawer;
        let (w, h) = self.close.preferred_size(theme);
        let y = p.y + t.head_pad + (t.head_h - h) / 2.0;
        Rect::new(p.right() - t.head_pad_end - w, y, w, h)
    }

    /// Each group's label top and each link's rect, in order, in `p`.
    fn layout(&self, theme: &Theme, p: Rect) -> (Vec<f32>, Vec<Rect>) {
        let t = &theme.overlay.drawer;
        let link_h = Self::link_h(theme);
        let x = p.x + t.body_pad;
        let w = (p.w - t.body_pad * 2.0).max(0.0);
        let mut y = p.y + t.head_pad * 2.0 + t.head_h;
        let (mut labels, mut links) = (Vec::new(), Vec::new());
        for (g, group) in self.groups.iter().enumerate() {
            if g > 0 {
                y += t.group_gap;
            }
            labels.push(y);
            y += t.group_pad * 2.0 + t.group_label.line_height + t.item_gap;
            for (i, _) in group.links.iter().enumerate() {
                if i > 0 {
                    y += t.item_gap;
                }
                links.push(Rect::new(x, y, w, link_h));
                y += link_h;
            }
        }
        (labels, links)
    }

    /// Its content's height: the head, the groups, the bottom pad.
    pub fn content_h(&self, theme: &Theme) -> f32 {
        let t = &theme.overlay.drawer;
        let (_, links) = self.layout(theme, Rect::new(0.0, 0.0, 0.0, 0.0));
        links
            .last()
            .map_or(t.head_pad * 2.0 + t.head_h, |r| r.bottom())
            + t.body_pad
    }

    fn link_at(&self, theme: &Theme, p: Rect, x: f32, y: f32) -> Option<usize> {
        let (_, links) = self.layout(theme, p);
        links
            .iter()
            .position(|r| r.contains(x, y) && y < p.bottom())
    }

    /// Routes an event to the drawer settled in `bounds`: hover and
    /// press on the links (a pick on release over the link pressed), the
    /// close button, a press on the scrim; everything else is swallowed.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        theme: &Theme,
        bounds: Rect,
    ) -> (DrawerAction, EventResult) {
        let panel = self.panel_rect(theme, bounds);
        let close = self
            .close
            .handle_event(event, self.close_rect(theme, panel));
        if close.clicked {
            return (DrawerAction::Close, close);
        }
        let mut r = EventResult {
            handled: true,
            ..close
        };
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let at = self.link_at(theme, panel, x, y);
                if at != self.hovered {
                    self.hovered = at;
                    r = r.merge(EventResult::changed());
                }
            }
            WidgetEvent::MouseDown { x, y } if !panel.contains(x, y) => {
                return (DrawerAction::Close, EventResult::clicked());
            }
            WidgetEvent::MouseDown { x, y } => {
                self.pressed = self.link_at(theme, panel, x, y);
            }
            WidgetEvent::MouseUp { x, y } => {
                let at = self.link_at(theme, panel, x, y);
                let pressed = self.pressed.take();
                if let Some(i) = at.filter(|i| pressed == Some(*i)) {
                    return (DrawerAction::Pick(i), EventResult::clicked());
                }
            }
            _ => {}
        }
        (DrawerAction::None, r)
    }

    /// The scrim over `bounds` and the drawer sliding in from its edge,
    /// `shown` of the way (its overlay entry's progress).
    pub fn render(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        theme: &Theme,
        bounds: Rect,
        shown: f32,
    ) {
        paint_scrim(c, layer, bounds, &theme.overlay.drawer.panel, shown);
        let panel = slide(self.panel_rect(theme, bounds), self.edge, shown);
        self.render_in(c, layer, theme, panel, bounds);
    }

    /// The drawer at `panel`, inside `clip`, without a scrim.
    pub fn render_in(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        theme: &Theme,
        panel: Rect,
        clip: Rect,
    ) {
        let t = &theme.overlay.drawer;
        if let Some(id) = &self.sem {
            c.bind_to_layer(layer, id, [panel.x, panel.y, panel.w, panel.h]);
        }
        paint_panel(c, layer, panel, clip, self.edge, &t.panel);
        let inside = Rect::new(
            panel.x.max(clip.x),
            panel.y.max(clip.y),
            (panel.right().min(clip.right()) - panel.x.max(clip.x)).max(0.0),
            (panel.bottom().min(clip.bottom()) - panel.y.max(clip.y)).max(0.0),
        );
        c.push_to_layer(
            layer,
            SceneNode::PushClip {
                x: inside.x,
                y: inside.y,
                w: inside.w,
                h: inside.h,
            },
        );
        let close = self.close_rect(theme, panel);
        let name_w = (close.x - panel.x - t.head_pad).max(1.0);
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&self.title, &t.brand, Some(name_w)),
                x: panel.x + t.head_pad,
                y: panel.y + t.head_pad + TextMeasurer::vertical_center(&t.brand, t.head_h),
                color: t.brand_color.0,
            },
        );
        self.close.render_to_layer(c, layer, close, theme);

        let (labels, rects) = self.layout(theme, panel);
        for (group, y) in self.groups.iter().zip(&labels) {
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(&group.title, &t.group_label, None),
                    x: panel.x + t.body_pad + t.group_pad,
                    y: y + t.group_pad,
                    color: t.group_color.0,
                },
            );
        }
        for (i, (link, r)) in self.links().zip(&rects).enumerate() {
            self.render_link(c, layer, theme, link, *r, i);
        }
        c.push_to_layer(layer, SceneNode::PopClip);
    }

    fn render_link(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        theme: &Theme,
        link: &DrawerLink,
        r: Rect,
        i: usize,
    ) {
        let t = &theme.overlay.drawer;
        if let Some(id) = &self.sem {
            c.bind_to_layer(layer, &id.key(i.to_string()), [r.x, r.y, r.w, r.h]);
        }
        let current = self.current == Some(i);
        let lit = current || self.hovered == Some(i) || self.focused == Some(i);
        if lit {
            c.push_to_layer(
                layer,
                rounded_rect(r.x, r.y, r.w, r.h, t.item_radius, t.item_fill.0),
            );
        }
        if self.focused == Some(i) {
            let ring = crate::widgets::recipe::focus_ring_inside(r, t.item_radius, theme);
            c.push_to_layer(layer, ring);
        }
        let b = Rect::new(r.x + t.item_pad, r.y + t.item_pad, t.icon_box, t.icon_box);
        if current {
            for spec in &t.icon_box_shadow {
                if let Some(node) = shadow_node(b, t.icon_box_radius, spec) {
                    c.push_to_layer(layer, node);
                }
            }
            c.push_to_layer(
                layer,
                rounded_rect(b.x, b.y, b.w, b.h, t.icon_box_radius, t.icon_box_fill.0),
            );
        }
        let icon_color = if lit { t.icon_active } else { t.icon_color };
        let inset = (t.icon_box - t.icon) / 2.0;
        if let Some(node) =
            icons::icon_at(link.icon, t.icon, icon_color.0, b.x + inset, b.y + inset)
        {
            c.push_to_layer(layer, node);
        }
        let x = b.right() + t.item_inline;
        let w = (r.right() - t.item_pad - x).max(1.0);
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&link.label, &t.item_label, Some(w)),
                x,
                y: r.y + TextMeasurer::vertical_center(&t.item_label, r.h),
                color: t.item_color.0,
            },
        );
    }

    /// Declares the drawer as `id`: a modal `Role::Dialog` named by its
    /// title; its close button `<id>[close]`; each group a `Role::Group`
    /// (`<id>.group[g]`) of its links, each a `Role::Link` (`<id>[<i>]`)
    /// with its `current` state and an `activate`. Pass the id given to
    /// [`Drawer::sem`].
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let mut close = b.node(id.key("close"), Role::Button, "Close");
        close.parent(id);
        close.action("activate");
        let mut i = 0;
        if let Ok(groups) = id.child("group") {
            for (g, group) in self.groups.iter().enumerate() {
                let gid = groups.key(g.to_string());
                b.node(&gid, Role::Group, group.title.as_str()).parent(id);
                for link in &group.links {
                    let mut node = b.node(id.key(i.to_string()), Role::Link, link.label.as_str());
                    node.parent(&gid);
                    node.state("current", self.current == Some(i));
                    node.action("activate");
                    i += 1;
                }
            }
        }
        let mut node = b.node(id, Role::Dialog, self.title.as_str());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("edge", self.edge.name());
        node.action("close");
        node
    }
}
