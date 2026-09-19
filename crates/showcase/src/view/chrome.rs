//! Chrome section: the pieces an app screen is framed with, the ones
//! the ide, urnaui and this gallery used to draw by hand. Avatars and
//! badges, a panel header with its count, nav links in every state, a
//! breadcrumb that collapses when narrow, stats, a code block, skeleton
//! placeholders, a virtualized table that drops columns, all on panels.
//! Every geometry here derives from `theme`; the section itself keeps
//! only the gaps between its groups.

use comps::content::{
    Avatar, AvatarSize, CodeBlock, Column, Panel, Separator, Skeleton, SkeletonShape, Stat,
    StatSize, Table,
};
use comps::nav::{Breadcrumb, NavLink, PanelHeader};
use comps::prelude::{Align, Badge, EventResult, Rect, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::{Intent, Theme};

use super::group_label;

pub struct ChromeSection {
    avatars: [Avatar; 3],
    badges: Vec<Badge>,
    header: PanelHeader,
    links: Vec<NavLink>,
    crumbs: Breadcrumb,
    stats: [Stat; 3],
    code: CodeBlock,
    skeletons: [Skeleton; 3],
    table: Table,
    /// The skeletons breathe only while this is on (an idle section
    /// settles, like every gallery section).
    breathing: bool,
    /// The last crumb clicked, echoed under the trail.
    last_crumb: Option<usize>,
}

/// Rects for everything the section draws, top to bottom.
struct Layout {
    avatars: [Rect; 3],
    badges: Vec<Rect>,
    header: Rect,
    links: Vec<Rect>,
    rail_link: Rect,
    crumbs: Rect,
    stats: [Rect; 3],
    code: Rect,
    skeletons: [Rect; 3],
    table: Rect,
    labels: Vec<(&'static str, f32)>,
    total_h: f32,
}

impl ChromeSection {
    pub fn new(theme: &Theme) -> Self {
        let mut table = Table::new(
            vec![
                Column::new("file", theme)
                    .weight(2.0)
                    .min_w(theme.size.field_min_w),
                Column::new("status", theme)
                    .min_w(theme.size.field_min_w / 2.0)
                    .priority(2),
                Column::new("size", theme)
                    .align(Align::End)
                    .min_w(theme.size.field_min_w / 2.0)
                    .priority(1),
                Column::new("hash", theme)
                    .min_w(theme.size.field_min_w)
                    .priority(0),
            ],
            theme,
        );
        table.set_row_count(2_000);
        Self {
            avatars: [
                Avatar::new("ann").size(AvatarSize::Sm),
                Avatar::new("brenner").online(true),
                Avatar::new("hoff").size(AvatarSize::Lg),
            ],
            badges: vec![
                Badge::count("3"),
                Badge::count("128"),
                Badge::count("1").intent(Intent::Constructive),
                Badge::tag("MODIFIED"),
                Badge::tag("staged").intent(Intent::Constructive),
                Badge::tag("conflict").intent(Intent::Destructive),
            ],
            header: PanelHeader::new("Changes")
                .blurb("the column head every panel starts with")
                .badge(Badge::tag("12")),
            links: vec![
                NavLink::new("Workspace")
                    .icon("layout-grid")
                    .hint("1")
                    .active(true),
                NavLink::new("Branches").icon("git-branch").hint("2"),
                NavLink::new("History")
                    .icon("history")
                    .badge(Badge::count("4")),
                NavLink::new("A label long enough to truncate in a narrow rail")
                    .icon("file")
                    .hint("9"),
            ],
            crumbs: Breadcrumb::new(["crates", "comps", "src", "content", "table.rs"]),
            stats: [
                Stat::new("38 412", "chunks").size(StatSize::Sm),
                Stat::new("1.3", "on disk")
                    .unit("GB")
                    .delta("+12%", Intent::Constructive),
                Stat::new("97", "recall@10")
                    .unit("%")
                    .size(StatSize::Lg)
                    .delta("-2", Intent::Destructive),
            ],
            code: CodeBlock::new(
                "cargo run -p showcase -- chrome hoff\ncargo test -p comps matrix",
            )
            .caption("shell"),
            skeletons: [
                Skeleton::new(SkeletonShape::Disc),
                Skeleton::new(SkeletonShape::Lines).lines(3),
                Skeleton::new(SkeletonShape::Block),
            ],
            table,
            breathing: false,
            last_crumb: None,
        }
    }

    fn label_h(theme: &Theme) -> f32 {
        theme.typography.caption_sm().line_height + theme.spacing.md
    }

    fn row_gap(theme: &Theme) -> f32 {
        theme.spacing.xl
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Layout {
        let gap = theme.spacing.md;
        let label_h = Self::label_h(theme);
        let row_gap = Self::row_gap(theme);
        let mut labels = Vec::new();
        let mut y = content.y;

        labels.push(("AVATARS", y));
        y += label_h;
        let mut x = content.x;
        let mut row_h: f32 = 0.0;
        let avatars = std::array::from_fn(|i| {
            let (w, h) = self.avatars[i].preferred_size(theme);
            let r = Rect::new(x, y, w, h);
            x += w + gap;
            row_h = row_h.max(h);
            r
        });
        // Badges share the row, after the avatars.
        let mut badges = Vec::with_capacity(self.badges.len());
        for badge in &self.badges {
            let (w, h) = badge.preferred_size(theme);
            if x + w > content.x + content.w {
                x = content.x;
                y += row_h + gap;
                row_h = 0.0;
            }
            badges.push(Rect::new(x, y + (row_h.max(h) - h) / 2.0, w, h));
            x += w + gap;
            row_h = row_h.max(h);
        }
        y += row_h + row_gap;

        labels.push(("PANEL HEADER", y));
        y += label_h;
        let header = Rect::new(
            content.x,
            y,
            content.w.min(theme.size.readable_max_w),
            self.header.height(theme),
        );
        y += header.h + row_gap;

        labels.push(("NAV LINKS", y));
        y += label_h;
        let link_w = theme.size.sidebar_w - theme.spacing.md * 2.0;
        let link_h = NavLink::height(theme);
        let links = (0..self.links.len())
            .map(|i| {
                Rect::new(
                    content.x,
                    y + i as f32 * (link_h + theme.spacing.xs),
                    link_w,
                    link_h,
                )
            })
            .collect::<Vec<_>>();
        // The rail form beside the column.
        let rail_w = theme.size.sidebar_rail_w - theme.spacing.md * 2.0;
        let rail_link = Rect::new(content.x + link_w + gap, y, rail_w, link_h);
        y += self.links.len() as f32 * (link_h + theme.spacing.xs) + row_gap;

        labels.push(("BREADCRUMB", y));
        y += label_h;
        let crumbs = Rect::new(
            content.x,
            y,
            content.w.min(theme.size.readable_max_w / 2.0),
            theme.typography.base_2r().line_height,
        );
        y += crumbs.h + row_gap;

        labels.push(("STATS", y));
        y += label_h;
        let mut x = content.x;
        let mut row_h: f32 = 0.0;
        let stats = std::array::from_fn(|i| {
            let (w, h) = self.stats[i].preferred_size(theme);
            let r = Rect::new(x, y, w, h);
            x += w + theme.spacing.xxl;
            row_h = row_h.max(h);
            r
        });
        y += row_h + row_gap;

        labels.push(("CODE BLOCK", y));
        y += label_h;
        let code_w = content.w.min(theme.size.readable_max_w);
        let code = Rect::new(content.x, y, code_w, self.code.height_for(code_w, theme));
        y += code.h + row_gap;

        labels.push(("SKELETON", y));
        y += label_h;
        let disc = theme.control.height(engine::theme::ControlSize::Md);
        let lines_h = self.skeletons[1].preferred_height(theme).unwrap_or(disc);
        let block_w = (content.w - disc - gap * 2.0) / 2.0;
        let skeletons = [
            Rect::new(content.x, y, disc, disc),
            Rect::new(content.x + disc + gap, y, block_w.max(0.0), lines_h),
            Rect::new(
                content.x + disc + gap + block_w + gap,
                y,
                block_w.max(0.0),
                lines_h,
            ),
        ];
        y += lines_h.max(disc) + row_gap;

        labels.push(("TABLE", y));
        y += label_h;
        let table_h = Table::header_h(theme) + Table::row_h(theme) * 8.0;
        let table = Rect::new(content.x, y, content.w, table_h);
        y += table_h;

        Layout {
            avatars,
            badges,
            header,
            links,
            rail_link,
            crumbs,
            stats,
            code,
            skeletons,
            table,
            labels,
            total_h: y - content.y,
        }
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme).total_h + Self::row_gap(theme)
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let l = self.layout(content, theme);
        let mut result = EventResult::IGNORED;
        let mut clicked = None;
        for (i, (link, rect)) in self.links.iter_mut().zip(&l.links).enumerate() {
            let r = link.handle_event(event, *rect);
            if r.clicked {
                clicked = Some(i);
            }
            result = result.merge(r);
        }
        if let Some(i) = clicked {
            // A click moves the active mark, like a real sidebar, and
            // toggles the skeletons' breathing for the demo.
            for (k, link) in self.links.iter_mut().enumerate() {
                link.active = k == i;
            }
            self.breathing = !self.breathing;
        }
        let (r, crumb) = self.crumbs.handle_event(event, l.crumbs, theme);
        if let Some(i) = crumb {
            self.last_crumb = Some(i);
        }
        result = result.merge(r);
        result.merge(self.table.handle_event(event, l.table, theme))
    }

    /// Skeletons breathe while toggled on (by clicking any nav link).
    pub fn tick(&mut self, dt: f32) -> bool {
        let mut animating = self.table.tick(dt);
        if self.breathing {
            for s in &mut self.skeletons {
                s.tick(dt);
            }
            animating = true;
        }
        animating
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let l = self.layout(content, theme);
        for (label, y) in &l.labels {
            group_label(c, label, content.x, *y, theme);
        }
        for (avatar, rect) in self.avatars.iter().zip(&l.avatars) {
            avatar.render(c, *rect, theme);
        }
        for (badge, rect) in self.badges.iter().zip(&l.badges) {
            badge.render(c, *rect, theme);
        }

        Panel::new().render(c, l.header, theme);
        self.header.render(c, l.header, theme);

        for (link, rect) in self.links.iter().zip(&l.links) {
            link.render(c, *rect, theme);
        }
        self.links[0].render(c, l.rail_link, theme);

        self.crumbs.render(c, l.crumbs, theme);
        if let Some(i) = self.last_crumb {
            let note = format!("clicked crumb {i}");
            let style = theme.typography.caption_r();
            c.push(engine::compositor::SceneNode::Text {
                key: engine::compositor::TextNodeKey::from_style(&note, &style, None),
                x: l.crumbs.right() + theme.spacing.lg,
                y: l.crumbs.y,
                color: theme.glass.text_faint.0,
            });
        }

        for (stat, rect) in self.stats.iter().zip(&l.stats) {
            stat.render(c, *rect, theme);
        }
        self.code.render(c, l.code, theme);
        for (sk, rect) in self.skeletons.iter().zip(&l.skeletons) {
            sk.render(c, *rect, theme);
        }
        Separator::Horizontal.render(
            c,
            Rect::new(
                content.x,
                l.table.y - Self::label_h(theme) - theme.spacing.md,
                content.w,
                theme.spacing.xs,
            ),
            theme,
        );

        Panel::new().render(c, l.table.outset(theme.spacing.sm), theme);
        let text = theme.colors.text_mid.0;
        let dim = theme.glass.text_faint.0;
        self.table
            .render_with(c, l.table, theme, |c, row, cell, col| {
                let (label, align, color) = match col {
                    0 => (
                        format!("crates/comps/src/file_{row}.rs"),
                        Align::Start,
                        text,
                    ),
                    1 => (
                        ["modified", "added", "renamed"][row % 3].to_string(),
                        Align::Start,
                        dim,
                    ),
                    2 => (format!("{} KB", (row * 37) % 900 + 3), Align::End, dim),
                    _ => (
                        format!("{:07x}", row * 2654435761u64 as usize % 0xfffffff),
                        Align::Start,
                        dim,
                    ),
                };
                Table::draw_cell(
                    c,
                    engine::compositor::LayerId::DEFAULT,
                    cell,
                    &label,
                    align,
                    color,
                    theme,
                );
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chrome_lays_out_at_narrow_and_wide_without_overflow() {
        let theme = Theme::hoff();
        let mut section = ChromeSection::new(&theme);
        for w in [340.0, 700.0, 1272.0] {
            let content = Rect::new(40.0, 80.0, w, 900.0);
            let l = section.layout(content, &theme);
            for r in l.badges.iter().chain(l.avatars.iter()) {
                assert!(
                    r.right() <= content.right() + 0.5,
                    "{w}: badge/avatar overflows"
                );
            }
            assert!(l.code.right() <= content.right() + 0.5);
            assert!(l.table.right() <= content.right() + 0.5);
            let mut c = Compositor::new();
            c.begin_frame();
            section.render(&mut c, content, &theme);
            assert!(section.content_height(content, &theme) > 0.0);
        }
    }

    #[test]
    fn table_drops_columns_when_the_gallery_is_narrow() {
        let theme = Theme::hoff();
        let section = ChromeSection::new(&theme);
        let wide = section.table.column_layout(1272.0, &theme);
        assert_eq!(wide.visible.len(), 4);
        let narrow = section.table.column_layout(340.0, &theme);
        assert!(narrow.visible.len() < 4);
        assert_eq!(narrow.visible[0], 0, "the file column never drops");
    }

    #[test]
    fn crumb_click_is_echoed_and_settles() {
        let theme = Theme::hoff();
        let mut section = ChromeSection::new(&theme);
        let content = Rect::new(40.0, 80.0, 1272.0, 900.0);
        let l = section.layout(content, &theme);
        let rects = section.crumbs.crumb_rects(l.crumbs, &theme);
        let (x, y) = rects[1].1.center();
        section.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
        let r = section.handle_event(&WidgetEvent::MouseUp { x, y }, content, &theme);
        assert!(r.clicked);
        assert_eq!(section.last_crumb, Some(1));
        // Idle: the section settles (skeletons only breathe when toggled).
        let mut animating = true;
        for _ in 0..600 {
            animating = section.tick(1.0 / 60.0);
            if !animating {
                break;
            }
        }
        assert!(!animating);
    }
}
