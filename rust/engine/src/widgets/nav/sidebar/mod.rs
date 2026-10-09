//! HOFF sidebar: the raised surface rail with a brand block on top, a
//! scrollable band of [`NavLink`]s, and a footer band. Its mode follows
//! the viewport breakpoint (`theme.layout.sidebar_mode`): Full with
//! labels at `size.sidebar_w`, Rail with icons only at
//! `size.sidebar_rail_w`, or a Drawer that slides over the content on a
//! phone and closes on navigation. Everything scrolls in its band, so a
//! short window never paints links over the brand or the footer. Links
//! can be partitioned into [`NavGroup`]s: a caption row per group that
//! expands and collapses its links on click (a tree of one level).
//!
//! `mod.rs` is the state and the geometry (mode, rect, band, rows),
//! `input.rs` the event routing, `render.rs` the drawing.

mod input;
mod render;

use engine::input::scroll::ScrollState;
use engine::theme::{SidebarMode, Theme};

use crate::core::Rect;
use crate::nav::NavLink;

/// A run of `len` consecutive links under one caption. The groups of a
/// sidebar partition its links in order; the active index space is
/// unchanged (collapsed links keep their index, they just take no room).
#[derive(Clone, Debug)]
pub struct NavGroup {
    pub label: String,
    pub len: usize,
    pub expanded: bool,
}

impl NavGroup {
    pub fn new(label: impl Into<String>, len: usize) -> Self {
        Self {
            label: label.into(),
            len,
            expanded: true,
        }
    }

    pub fn collapsed(mut self) -> Self {
        self.expanded = false;
        self
    }
}

#[derive(Debug)]
pub struct Sidebar {
    pub links: Vec<NavLink>,
    /// Captions partitioning `links` in order (empty: one flat band).
    pub groups: Vec<NavGroup>,
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
    /// Top-left of the sidebar in window coordinates. Zero for an app
    /// that owns the window; a host embedding the app inside its own
    /// chrome moves it (`vw`/`vh` stay the embedded viewport's size).
    pub origin: (f32, f32),
    scroll: ScrollState,
}

impl Sidebar {
    pub fn new(links: Vec<NavLink>) -> Self {
        let mut s = Self {
            links,
            groups: Vec::new(),
            active: 0,
            footer_links: Vec::new(),
            brand: String::new(),
            tagline: String::new(),
            footer: Vec::new(),
            drawer_open: false,
            fixed_mode: None,
            origin: (0.0, 0.0),
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

    /// Partition the links into captioned groups. The lengths must add
    /// up to the link count; the extra links, if any, form a trailing
    /// uncaptioned run.
    pub fn groups(mut self, groups: Vec<NavGroup>) -> Self {
        debug_assert!(
            groups.iter().map(|g| g.len).sum::<usize>() <= self.links.len(),
            "sidebar groups cover more links than there are"
        );
        self.groups = groups;
        self.sync_active();
        self
    }

    /// The group a link index falls in, if the sidebar is grouped.
    fn group_of(&self, index: usize) -> Option<usize> {
        let mut start = 0;
        for (g, group) in self.groups.iter().enumerate() {
            if index < start + group.len {
                return Some(g);
            }
            start += group.len;
        }
        None
    }

    /// Toggle a group open or closed.
    pub fn toggle_group(&mut self, g: usize) {
        if let Some(group) = self.groups.get_mut(g) {
            group.expanded = !group.expanded;
        }
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
        if let Some(g) = self.group_of(self.active) {
            self.groups[g].expanded = true;
        }
        self.sync_active();
    }

    /// Height of a group caption row.
    pub fn group_h(theme: &Theme) -> f32 {
        theme.typography.caption_sm().line_height + theme.spacing.md
    }

    /// The band's rows in scrolled space: one rect per group caption and
    /// one per link, top to bottom. A collapsed group's links get a
    /// zero-height rect at the caption's edge: nothing draws or hits
    /// them, and their index stays put.
    fn rows(&self, band: Rect, theme: &Theme) -> (Vec<Rect>, Vec<Rect>) {
        let pad = theme.spacing.md;
        let h = NavLink::height(theme);
        let gap = theme.spacing.xs;
        let x = band.x + pad;
        let w = band.w - pad * 2.0;
        let mut y = band.y - self.scroll.offset();
        let mut links = Vec::with_capacity(self.links.len());
        let mut groups = Vec::with_capacity(self.groups.len());
        let mut i = 0;
        for group in &self.groups {
            let group_h = Self::group_h(theme);
            groups.push(Rect::new(x, y, w, group_h));
            y += group_h;
            for _ in 0..group.len.min(self.links.len() - i) {
                if group.expanded {
                    links.push(Rect::new(x, y, w, h));
                    y += h + gap;
                } else {
                    links.push(Rect::new(x, y, w, 0.0));
                }
                i += 1;
            }
            if group.expanded {
                y += gap;
            }
        }
        for _ in i..self.links.len() {
            links.push(Rect::new(x, y, w, h));
            y += h + gap;
        }
        (groups, links)
    }

    /// Total scrolled height of the band's rows.
    fn rows_h(&self, theme: &Theme) -> f32 {
        let band = Rect::new(0.0, self.scroll.offset(), 0.0, 0.0);
        let (groups, links) = self.rows(band, theme);
        groups
            .iter()
            .chain(links.iter())
            .map(|r| r.y + r.h)
            .fold(0.0, f32::max)
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
        let (x, y) = self.origin;
        match self.mode(theme, vw) {
            SidebarMode::Drawer => self
                .drawer_open
                .then(|| Rect::new(x, y, theme.size.sidebar_w.min(vw), vh)),
            SidebarMode::Rail => Some(Rect::new(x, y, theme.size.sidebar_rail_w, vh)),
            SidebarMode::Full => Some(Rect::new(x, y, theme.size.sidebar_w, vh)),
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

    /// Link rects in the band, scrolled (zero height under a collapsed
    /// group).
    pub fn link_rects(&self, rect: Rect, theme: &Theme, mode: SidebarMode) -> Vec<Rect> {
        self.rows(self.band(rect, theme, mode), theme).1
    }

    /// Group caption rects in the band, scrolled.
    pub fn group_rects(&self, rect: Rect, theme: &Theme, mode: SidebarMode) -> Vec<Rect> {
        self.rows(self.band(rect, theme, mode), theme).0
    }

    fn sync_scroll(&mut self, rect: Rect, theme: &Theme, mode: SidebarMode) {
        let band = self.band(rect, theme, mode);
        self.scroll.set_viewport(band.h);
        self.scroll.set_content(self.rows_h(theme));
    }

    /// Whether the sidebar is a drawer at this viewport (the app shows a
    /// menu button then).
    pub fn is_drawer(&self, theme: &Theme, vw: f32) -> bool {
        self.mode(theme, vw) == SidebarMode::Drawer
    }
}
