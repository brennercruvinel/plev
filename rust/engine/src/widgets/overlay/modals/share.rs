//! The shared post (`components/SharePost`) and the shared profile
//! (`components/ShareProfile`): what is shared, drawn with the picture
//! tokens in place of the kit's photos, over a row of social links and
//! the copy button. The links are the kit's Threads, X and Instagram,
//! with the Lucide glyphs closest to them (`at-sign`, `twitter`,
//! `instagram`).
//!
//! Projected: a `Role::Group` named by what is shared, described by its
//! line (the author and the date, the count of scenes), the links
//! (`<id>.social[<i>]`) and the copy button (`<id>.copy`).

use accord::{Role, SemId, SnapshotBuilder, Value};

use crate::drawing::compositor::Compositor;
use crate::styling::theme::Theme;
use crate::widgets::buttons::{
    Button, ButtonVariant, IconButton, IconButtonSize, IconButtonVariant,
};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

use super::tokens::SocialTokens;
use super::{
    Controls, Kind, ModalBody, child, glyph, middle, panel, picture, picture_corners, text,
    text_centered, text_w, tokens,
};

/// The kit's three links: the glyph and what it names.
const SOCIALS: [(&str, &str); 3] = [
    ("at-sign", "Share on Threads"),
    ("twitter", "Share on X"),
    ("instagram", "Share on Instagram"),
];

/// The links and the copy button, the controls both modals end in.
fn socials(id: Option<&SemId>) -> Controls {
    let mut controls = Controls::default();
    let family = child(id, "social");
    for (i, (icon, label)) in SOCIALS.iter().enumerate() {
        let link = IconButton::new(icon)
            .variant(IconButtonVariant::Outline)
            .size(IconButtonSize::Lg);
        let link_id = family.as_ref().map(|f| f.key(i.to_string()));
        controls.push(Kind::Icon(link, (*label).into()), link_id);
    }
    let copy = Button::new("Copy link").variant(ButtonVariant::Primary);
    controls.push(Kind::Button(copy), child(id, "copy"));
    controls
}

/// The links from `x` and the copy button after them, `copy_w` wide (or
/// its own width), in a row at `y`.
fn social_rects(
    controls: &Controls,
    s: &SocialTokens,
    x: f32,
    y: f32,
    copy_w: Option<f32>,
    theme: &Theme,
) -> Vec<Rect> {
    let mut out = Vec::with_capacity(SOCIALS.len() + 1);
    let mut at = x;
    for _ in SOCIALS {
        out.push(Rect::new(at, y, s.size, s.size));
        at += s.size + s.gap;
    }
    let (own, h) = match controls.0.last().map(|c| &c.kind) {
        Some(Kind::Button(b)) => b.preferred_size(theme),
        _ => (0.0, s.size),
    };
    let w = copy_w.unwrap_or(own).max(0.0);
    out.push(Rect::new(at, y + (s.size - h) / 2.0, w, h));
    out
}

/// The row's width with the copy button at its own width.
fn social_width(controls: &Controls, s: &SocialTokens, theme: &Theme) -> f32 {
    let r = social_rects(controls, s, 0.0, 0.0, None, theme);
    r.last().map_or(0.0, Rect::right)
}

fn run_on(
    controls: &mut Controls,
    target: &SemId,
    action: &str,
    param: Option<&Value>,
) -> Option<Result<bool, String>> {
    controls.run(target, action, param).map(|(_, r)| r)
}

/// The shared post: its render in a sunk box, its title, its author and
/// date, the links and the copy button that takes the rest of the row.
pub struct SharePost {
    pub title: String,
    pub author: String,
    pub date: String,
    controls: Controls,
}

impl SharePost {
    pub fn new(title: &str, author: &str, date: &str, id: Option<&SemId>) -> Self {
        Self {
            title: title.into(),
            author: author.into(),
            date: date.into(),
            controls: socials(id),
        }
    }

    /// The render's box at `rect`'s top.
    fn picture_box(rect: Rect, theme: &Theme) -> Rect {
        let p = &tokens(theme).share_post;
        let w = rect.w - p.pad * 2.0;
        Rect::new(rect.x + p.pad, rect.y + p.pad, w, w)
    }

    fn body_top(rect: Rect, theme: &Theme) -> f32 {
        Self::picture_box(rect, theme).bottom() + tokens(theme).share_post.body_pad
    }

    fn meta_h(theme: &Theme) -> f32 {
        let p = &tokens(theme).share_post;
        p.avatar
            .max(theme.typography.run(p.author).line_height)
            .max(p.clock)
    }

    fn rects(&self, rect: Rect, theme: &Theme) -> Vec<Rect> {
        let p = &tokens(theme).share_post;
        let x = rect.x + p.pad + p.body_pad;
        let y = Self::body_top(rect, theme)
            + theme.typography.run(p.title).line_height
            + p.title_gap
            + Self::meta_h(theme)
            + p.meta_bottom;
        let row_w = rect.w - (p.pad + p.body_pad) * 2.0;
        let links = SOCIALS.len() as f32 * (p.social.size + p.social.gap);
        social_rects(&self.controls, &p.social, x, y, Some(row_w - links), theme)
    }
}

impl ModalBody for SharePost {
    fn title(&self) -> &str {
        &self.title
    }

    fn size(&self, w: f32, theme: &Theme) -> (f32, f32) {
        let p = &tokens(theme).share_post;
        let w = p.panel.width.min(w);
        let rect = Rect::new(0.0, 0.0, w, 0.0);
        let body = Self::body_top(rect, theme)
            + theme.typography.run(p.title).line_height
            + p.title_gap
            + Self::meta_h(theme)
            + p.meta_bottom
            + p.social.size
            + p.body_pad
            + p.pad;
        (w, body)
    }

    fn handle_event(&mut self, event: &WidgetEvent, rect: Rect, theme: &Theme) -> EventResult {
        let rects = self.rects(rect, theme);
        self.controls.handle_event(event, &rects, theme).1
    }

    fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        let scale = &theme.typography;
        let t = tokens(theme);
        let (p, ink) = (&t.share_post, &t.ink);
        panel(c, rect, &p.panel, theme);
        let frame = Self::picture_box(rect, theme);
        super::boxed(
            c,
            frame,
            p.picture_radius,
            ink.well,
            Some((ink.rule, ink.rule_w)),
        );
        let art = frame.inset(p.picture_pad);
        picture(c, art, 0.0, t.pictures.post);
        let x = rect.x + p.pad + p.body_pad;
        let w = rect.w - (p.pad + p.body_pad) * 2.0;
        let mut y = Self::body_top(rect, theme);
        text(c, &self.title, &scale.run(p.title), x, y, w, ink.ink);
        y += scale.run(p.title).line_height + p.title_gap;
        let h = Self::meta_h(theme);
        let avatar = Rect::new(x, y + (h - p.avatar) / 2.0, p.avatar, p.avatar);
        picture(c, avatar, p.avatar / 2.0, t.pictures.avatars[1]);
        let name_x = avatar.right() + p.author_gap;
        let name_w = text_w(&self.author, &scale.run(p.author)).min(w);
        text(
            c,
            &self.author,
            &scale.run(p.author),
            name_x,
            middle(&scale.run(p.author), y, h),
            name_w,
            ink.ink,
        );
        let clock = Rect::new(
            name_x + name_w + p.meta_gap,
            y + (h - p.clock) / 2.0,
            p.clock,
            p.clock,
        );
        glyph(c, "clock", p.clock, ink.muted, clock);
        let date_x = clock.right() + p.date_gap;
        text(
            c,
            &self.date,
            &scale.run(p.date),
            date_x,
            middle(&scale.run(p.date), y, h),
            x + w - date_x,
            ink.muted,
        );
        self.controls.render(c, &self.rects(rect, theme), theme);
    }

    fn project(&self, b: &mut SnapshotBuilder, id: &SemId, parent: &SemId) {
        b.node(id, Role::Group, self.title.as_str())
            .parent(parent)
            .description(format!("{}, {}", self.author, self.date));
        self.controls.project(b, id);
    }

    fn run(
        &mut self,
        target: &SemId,
        action: &str,
        param: Option<&Value>,
    ) -> Option<Result<bool, String>> {
        run_on(&mut self.controls, target, action, param)
    }

    fn focus(&mut self, id: Option<&SemId>) -> bool {
        self.controls.focus(id)
    }
}

/// The shared profile: three covers, the avatar over them in a paper
/// ring, the name, the count of scenes, the links and the copy button
/// centered under them.
pub struct ShareProfile {
    pub name: String,
    pub line: String,
    controls: Controls,
}

impl ShareProfile {
    pub fn new(name: &str, line: &str, id: Option<&SemId>) -> Self {
        Self {
            name: name.into(),
            line: line.into(),
            controls: socials(id),
        }
    }

    /// The avatar's ring: centered under the covers, overlapping them by
    /// half.
    fn ring(rect: Rect, theme: &Theme) -> Rect {
        let p = &tokens(theme).share_profile;
        let d = p.avatar + p.avatar_ring * 2.0;
        let y = rect.y + p.pad + p.cover_h - d / 2.0;
        Rect::new(rect.x + (rect.w - d) / 2.0, y, d, d)
    }

    fn text_top(rect: Rect, theme: &Theme) -> f32 {
        Self::ring(rect, theme).bottom() + tokens(theme).share_profile.avatar_bottom
    }

    fn row_top(rect: Rect, theme: &Theme) -> f32 {
        let scale = &theme.typography;
        let p = &tokens(theme).share_profile;
        Self::text_top(rect, theme)
            + scale.run(p.name).line_height
            + p.count_gap
            + scale.run(p.count).line_height
            + p.text_bottom
            + p.actions_pad
    }

    fn rects(&self, rect: Rect, theme: &Theme) -> Vec<Rect> {
        let p = &tokens(theme).share_profile;
        let row = social_width(&self.controls, &p.social, theme);
        let x = rect.x + (rect.w - row) / 2.0;
        social_rects(
            &self.controls,
            &p.social,
            x,
            Self::row_top(rect, theme),
            None,
            theme,
        )
    }
}

impl ModalBody for ShareProfile {
    fn title(&self) -> &str {
        &self.name
    }

    fn size(&self, w: f32, theme: &Theme) -> (f32, f32) {
        let p = &tokens(theme).share_profile;
        let w = p.panel.width.min(w);
        let top = Self::row_top(Rect::new(0.0, 0.0, w, 0.0), theme);
        (w, top + p.social.size + p.actions_pad + p.pad)
    }

    fn handle_event(&mut self, event: &WidgetEvent, rect: Rect, theme: &Theme) -> EventResult {
        let rects = self.rects(rect, theme);
        self.controls.handle_event(event, &rects, theme).1
    }

    fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        let scale = &theme.typography;
        let t = tokens(theme);
        let (p, ink) = (&t.share_profile, &t.ink);
        panel(c, rect, &p.panel, theme);
        let n = t.pictures.covers.len();
        let inner = rect.w - p.pad * 2.0;
        let cover_w = (inner - p.cover_gap * (n as f32 - 1.0)) / n as f32;
        for (i, colors) in t.pictures.covers.iter().enumerate() {
            let x = rect.x + p.pad + i as f32 * (cover_w + p.cover_gap);
            let r = Rect::new(x, rect.y + p.pad, cover_w, p.cover_h);
            let start = if i == 0 { p.cover_outer } else { p.cover_inner };
            let end = if i + 1 == n {
                p.cover_outer
            } else {
                p.cover_inner
            };
            picture_corners(c, r, start, end, *colors);
        }
        let ring = Self::ring(rect, theme);
        picture(c, ring, ring.w / 2.0, [ink.paper, ink.paper]);
        let photo = ring.inset(p.avatar_ring);
        picture(c, photo, photo.w / 2.0, t.pictures.avatars[0]);
        let cx = rect.x + rect.w / 2.0;
        let w = inner;
        let y = Self::text_top(rect, theme);
        text_centered(c, &self.name, &scale.run(p.name), cx, y, w, ink.ink);
        let y = y + scale.run(p.name).line_height + p.count_gap;
        text_centered(c, &self.line, &scale.run(p.count), cx, y, w, ink.muted);
        self.controls.render(c, &self.rects(rect, theme), theme);
    }

    fn project(&self, b: &mut SnapshotBuilder, id: &SemId, parent: &SemId) {
        b.node(id, Role::Group, self.name.as_str())
            .parent(parent)
            .description(self.line.as_str());
        self.controls.project(b, id);
    }

    fn run(
        &mut self,
        target: &SemId,
        action: &str,
        param: Option<&Value>,
    ) -> Option<Result<bool, String>> {
        run_on(&mut self.controls, target, action, param)
    }

    fn focus(&mut self, id: Option<&SemId>) -> bool {
        self.controls.focus(id)
    }
}
