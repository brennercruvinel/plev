//! HOFF sidebar: the raised surface rail with a brand block on top, a
//! scrollable band of [`NavLink`]s, and a footer band. Its mode follows
//! the viewport breakpoint (`theme.layout.sidebar_mode`): Full with
//! labels at `size.sidebar_w`, Rail with icons only at
//! `size.sidebar_rail_w`, or a Drawer that slides over the content on a
//! phone and closes on navigation. Everything scrolls in its band, so a
//! short window never paints links over the brand or the footer.

use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::input::scroll::ScrollState;
use engine::text::TextMeasurer;
use engine::theme::{SidebarMode, Theme};

use crate::core::{EventResult, Rect, WidgetEvent};
use crate::nav::NavLink;

#[derive(Debug)]
pub struct Sidebar {
    pub links: Vec<NavLink>,
    /// Index into `links`.
    pub active: usize,
    /// Links pinned at the bottom (settings, account). They take part in
    /// the same active index space, after `links`.
    pub footer_links: Vec<NavLink>,
    pub brand: String,
    pub tagline: String,
    /// Footer hint lines in the placeholder tone.
    pub footer: Vec<String>,
    /// Drawer open state (only meaningful at the Compact breakpoint).
    pub drawer_open: bool,
    /// A mode the app pins regardless of the breakpoint (a tool that is
    /// always a rail). `None` follows `theme.layout`.
    pub fixed_mode: Option<SidebarMode>,
    scroll: ScrollState,
}

impl Sidebar {
    pub fn new(links: Vec<NavLink>) -> Self {
        let mut s = Self {
            links,
            active: 0,
            footer_links: Vec::new(),
            brand: String::new(),
            tagline: String::new(),
            footer: Vec::new(),
            drawer_open: false,
            fixed_mode: None,
            scroll: ScrollState::new(),
        };
        s.sync_active();
        s
    }

    pub fn footer_links(mut self, links: Vec<NavLink>) -> Self {
        self.footer_links = links;
        self.sync_active();
        self
    }

    pub fn fixed_mode(mut self, mode: SidebarMode) -> Self {
        self.fixed_mode = Some(mode);
        self
    }

    /// All links, band then footer, in active-index order.
    fn all_links_mut(&mut self) -> impl Iterator<Item = &mut NavLink> {
        self.links.iter_mut().chain(self.footer_links.iter_mut())
    }

    pub fn brand(mut self, brand: impl Into<String>, tagline: impl Into<String>) -> Self {
        self.brand = brand.into();
        self.tagline = tagline.into();
        self
    }

    pub fn footer(mut self, lines: Vec<String>) -> Self {
        self.footer = lines;
        self
    }

    pub fn set_active(&mut self, index: usize) {
        let n = self.links.len() + self.footer_links.len();
        self.active = index.min(n.saturating_sub(1));
        self.sync_active();
    }

    fn sync_active(&mut self) {
        let active = self.active;
        for (i, link) in self.all_links_mut().enumerate() {
            link.active = i == active;
        }
    }

    /// The sidebar mode for a viewport width: the pinned mode, or the
    /// breakpoint's.
    pub fn mode(&self, theme: &Theme, viewport_w: f32) -> SidebarMode {
        self.fixed_mode.unwrap_or_else(|| {
            theme
                .layout
                .sidebar_mode(theme.layout.breakpoint(viewport_w))
        })
    }

    /// Width the sidebar takes from the page at this viewport: nothing
    /// as a drawer (it floats over the content), the rail or the full
    /// width otherwise.
    pub fn page_width(&self, theme: &Theme, viewport_w: f32) -> f32 {
        match self.mode(theme, viewport_w) {
            SidebarMode::Drawer => 0.0,
            SidebarMode::Rail => theme.size.sidebar_rail_w,
            SidebarMode::Full => theme.size.sidebar_w,
        }
    }

    /// The sidebar's own rect for a viewport: the left column, or the
    /// full-width drawer sheet when open on a phone (`None` when a closed
    /// drawer draws nothing).
    pub fn rect(&self, theme: &Theme, vw: f32, vh: f32) -> Option<Rect> {
        match self.mode(theme, vw) {
            SidebarMode::Drawer => self
                .drawer_open
                .then(|| Rect::new(0.0, 0.0, theme.size.sidebar_w.min(vw), vh)),
            SidebarMode::Rail => Some(Rect::new(0.0, 0.0, theme.size.sidebar_rail_w, vh)),
            SidebarMode::Full => Some(Rect::new(0.0, 0.0, theme.size.sidebar_w, vh)),
        }
    }

    /// Brand block height: the title line plus the tagline plus the
    /// paddings (nothing when the sidebar has no brand).
    fn brand_h(&self, theme: &Theme, mode: SidebarMode) -> f32 {
        if self.brand.is_empty() || mode == SidebarMode::Rail {
            return theme.spacing.lg;
        }
        theme.spacing.xl
            + theme.typography.title().line_height
            + theme.typography.small_sm().line_height
            + theme.spacing.lg
    }

    fn footer_h(&self, theme: &Theme, mode: SidebarMode) -> f32 {
        let pad = theme.spacing.md;
        let links = self.footer_links.len() as f32;
        let links_h = if links > 0.0 {
            links * NavLink::height(theme) + (links - 1.0) * theme.spacing.xs + pad
        } else {
            0.0
        };
        if self.footer.is_empty() || mode == SidebarMode::Rail {
            return theme.spacing.lg + links_h;
        }
        theme.spacing.lg
            + self.footer.len() as f32 * theme.typography.caption_r().line_height
            + theme.spacing.lg
            + links_h
    }

    /// Rects of the pinned footer links, bottom-up from the sidebar foot.
    pub fn footer_link_rects(&self, rect: Rect, theme: &Theme) -> Vec<Rect> {
        let pad = theme.spacing.md;
        let h = NavLink::height(theme);
        let gap = theme.spacing.xs;
        let n = self.footer_links.len();
        (0..n)
            .map(|i| {
                let from_bottom = (n - i) as f32;
                Rect::new(
                    rect.x + pad,
                    rect.y + rect.h - pad - from_bottom * h - (from_bottom - 1.0) * gap,
                    rect.w - pad * 2.0,
                    h,
                )
            })
            .collect()
    }

    /// The band the links scroll in.
    pub fn band(&self, rect: Rect, theme: &Theme, mode: SidebarMode) -> Rect {
        let top = self.brand_h(theme, mode);
        let bottom = self.footer_h(theme, mode);
        Rect::new(
            rect.x,
            rect.y + top,
            rect.w,
            (rect.h - top - bottom).max(0.0),
        )
    }

    /// Link rects in the band, scrolled.
    pub fn link_rects(&self, rect: Rect, theme: &Theme, mode: SidebarMode) -> Vec<Rect> {
        let band = self.band(rect, theme, mode);
        let pad = theme.spacing.md;
        let h = NavLink::height(theme);
        let gap = theme.spacing.xs;
        let offset = self.scroll.offset();
        (0..self.links.len())
            .map(|i| {
                Rect::new(
                    band.x + pad,
                    band.y + i as f32 * (h + gap) - offset,
                    band.w - pad * 2.0,
                    h,
                )
            })
            .collect()
    }

    fn sync_scroll(&mut self, rect: Rect, theme: &Theme, mode: SidebarMode) {
        let band = self.band(rect, theme, mode);
        let h = NavLink::height(theme);
        let gap = theme.spacing.xs;
        let n = self.links.len() as f32;
        self.scroll.set_viewport(band.h);
        self.scroll
            .set_content((n * h + (n - 1.0).max(0.0) * gap).max(0.0));
    }

    /// Route an event. Returns the result and the index of the link that
    /// was activated, if any. A drawer closes on navigation and on a
    /// press outside it.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        theme: &Theme,
        vw: f32,
        vh: f32,
    ) -> (EventResult, Option<usize>) {
        let mode = self.mode(theme, vw);
        let Some(rect) = self.rect(theme, vw, vh) else {
            return (EventResult::IGNORED, None);
        };
        self.sync_scroll(rect, theme, mode);
        let band = self.band(rect, theme, mode);

        // Pinned footer links first: they never scroll.
        let footer_rects = self.footer_link_rects(rect, theme);
        let base = self.links.len();
        let mut footer_clicked = None;
        let mut footer_result = EventResult::IGNORED;
        for (i, (link, r)) in self.footer_links.iter_mut().zip(&footer_rects).enumerate() {
            let lr = link.handle_event(event, *r);
            if lr.clicked {
                footer_clicked = Some(base + i);
            }
            footer_result = footer_result.merge(lr);
        }
        if let Some(i) = footer_clicked {
            self.set_active(i);
            if mode == SidebarMode::Drawer {
                self.drawer_open = false;
            }
            return (EventResult::clicked(), Some(i));
        }

        if let WidgetEvent::Scroll { x, y, delta } = *event
            && band.contains(x, y)
        {
            let before = self.scroll.offset();
            self.scroll.scroll_by(delta);
            return if self.scroll.offset() != before {
                (EventResult::changed(), None)
            } else {
                (
                    EventResult {
                        handled: true,
                        ..EventResult::IGNORED
                    },
                    None,
                )
            };
        }

        let rects = self.link_rects(rect, theme, mode);
        let mut result = footer_result;
        let mut clicked = None;
        for (i, (link, r)) in self.links.iter_mut().zip(&rects).enumerate() {
            // Links outside the band are clipped away: they cannot be hit.
            let (px, py) = event.pos();
            if !band.contains(px, py) && !link.is_hovered() {
                continue;
            }
            let lr = link.handle_event(event, *r);
            if lr.clicked {
                clicked = Some(i);
            }
            result = result.merge(lr);
        }
        if let Some(i) = clicked {
            self.set_active(i);
            if mode == SidebarMode::Drawer {
                self.drawer_open = false;
            }
            return (EventResult::clicked(), Some(i));
        }
        if mode == SidebarMode::Drawer
            && let WidgetEvent::MouseDown { x, y } = *event
            && !rect.contains(x, y)
        {
            self.drawer_open = false;
            return (EventResult::changed(), None);
        }
        // The sidebar surface swallows what lands on it.
        if let (x, y) = event.pos()
            && rect.contains(x, y)
        {
            result.handled = true;
        }
        (result, None)
    }

    pub fn render(&mut self, c: &mut Compositor, layer: LayerId, theme: &Theme, vw: f32, vh: f32) {
        let mode = self.mode(theme, vw);
        let Some(rect) = self.rect(theme, vw, vh) else {
            return;
        };
        self.sync_scroll(rect, theme, mode);
        let glass = &theme.glass;

        // Drawer: scrim behind the sheet.
        if mode == SidebarMode::Drawer {
            c.push_to_layer(
                layer,
                SceneNode::Rect {
                    x: 0.0,
                    y: 0.0,
                    w: vw,
                    h: vh,
                    color: glass.scrim.0,
                },
            );
        }
        c.push_to_layer(
            layer,
            SceneNode::Rect {
                x: rect.x,
                y: rect.y,
                w: rect.w,
                h: rect.h,
                color: theme.colors.surface.0,
            },
        );

        // Brand block.
        if !self.brand.is_empty() && mode != SidebarMode::Rail {
            let pad = theme.spacing.xl;
            let title = theme.typography.title();
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(&self.brand, &title, Some(rect.w - pad * 2.0)),
                    x: rect.x + pad,
                    y: rect.y + pad,
                    color: theme.colors.text.0,
                },
            );
            if !self.tagline.is_empty() {
                let small = theme.typography.small_sm();
                c.push_to_layer(
                    layer,
                    SceneNode::Text {
                        key: TextNodeKey::from_style(
                            &self.tagline,
                            &small,
                            Some(rect.w - pad * 2.0),
                        ),
                        x: rect.x + pad,
                        y: rect.y + pad + title.line_height,
                        color: glass.text_placeholder.0,
                    },
                );
            }
        }

        // Links, clipped to their band.
        let band = self.band(rect, theme, mode);
        c.push_to_layer(
            layer,
            SceneNode::PushClip {
                x: band.x,
                y: band.y,
                w: band.w,
                h: band.h,
            },
        );
        for (link, r) in self.links.iter().zip(self.link_rects(rect, theme, mode)) {
            if r.y + r.h < band.y || r.y > band.y + band.h {
                continue;
            }
            link.render_to_layer(c, layer, r, theme);
        }
        c.push_to_layer(layer, SceneNode::PopClip);

        // Pinned footer links.
        for (link, r) in self
            .footer_links
            .iter()
            .zip(self.footer_link_rects(rect, theme))
        {
            link.render_to_layer(c, layer, r, theme);
        }

        // Footer hints.
        if !self.footer.is_empty() && mode != SidebarMode::Rail {
            let style = theme.typography.caption_r();
            let pad = theme.spacing.xl;
            let mut y = rect.y + rect.h - self.footer_h(theme, mode) + theme.spacing.lg;
            for line in &self.footer {
                let shown = TextMeasurer::truncate_to_width(line, &style, rect.w - pad * 2.0);
                c.push_to_layer(
                    layer,
                    SceneNode::Text {
                        key: TextNodeKey::from_style(&shown, &style, None),
                        x: rect.x + pad,
                        y,
                        color: glass.text_placeholder.0,
                    },
                );
                y += style.line_height;
            }
        }
    }

    /// Whether the sidebar is a drawer at this viewport (the app shows a
    /// menu button then).
    pub fn is_drawer(&self, theme: &Theme, vw: f32) -> bool {
        self.mode(theme, vw) == SidebarMode::Drawer
    }
}
