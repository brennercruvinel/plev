//! The hoff core kit's avatar: a disc on the kit's ladder of edges
//! (`theme.message.avatar.ladder`, 24 to 80 in the core, as a share of the
//! theme's control `Md` elsewhere), showing a photo or, without one, the
//! kit's upload disc (`UploadAvatar`, `#F8F7F7`) with the initial in the
//! ink. An optional online dot, the kit's green mark ringed by the paper,
//! sits on the bottom end rim.
//!
//! [`AvatarStack`] is the kit's stack of people
//! (`DesignAndAnimationSidebar/Head`): each disc in a paper ring, sliding
//! under the one before, the first on top; where the row is narrower than
//! everyone, the last disc it holds counts the rest.
//!
//! Behavior apart from the look: an avatar is an image a reader lands on,
//! named by the person, with its presence; it declares no action. Every
//! value is `theme.message.avatar`.

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::gpu::image::ImageHandle;
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::{ControlSize, Theme};

use crate::widgets::recipe::{focus_ring, rounded_rect, rounded_rect_stroke};
use crate::widgets::widget::Rect;

/// The kit's avatar edges, smallest first.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarSize {
    /// On a comment pin (`CommentPin`, 24 in the core).
    Xs,
    /// In a thread row and a stack (`Comment/Message`, 32).
    Sm,
    /// The header's (`HeaderSimple`, 40): the control `Md` step.
    #[default]
    Md,
    /// A settings row or a notification (`UploadAvatar`, 48).
    Lg,
    /// A profile header (`ShareProfile`, 80).
    Xl,
}

impl AvatarSize {
    pub const ALL: [AvatarSize; 5] = [
        AvatarSize::Xs,
        AvatarSize::Sm,
        AvatarSize::Md,
        AvatarSize::Lg,
        AvatarSize::Xl,
    ];

    /// The edge under `theme`: the kit's step as the same share of the
    /// theme's control `Md` it is of the kit's.
    pub fn px(self, theme: &Theme) -> f32 {
        let t = &theme.message.avatar;
        t.ladder[self as usize] / t.basis * theme.control.height(ControlSize::Md)
    }

    pub fn name(self) -> &'static str {
        match self {
            AvatarSize::Xs => "xs",
            AvatarSize::Sm => "sm",
            AvatarSize::Md => "md",
            AvatarSize::Lg => "lg",
            AvatarSize::Xl => "xl",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Avatar {
    /// The name the initial is taken from, and a reader hears.
    pub name: String,
    pub image: Option<ImageHandle>,
    pub size: AvatarSize,
    pub online: bool,
    focused: bool,
    sem: Option<SemId>,
}

impl Avatar {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            image: None,
            size: AvatarSize::default(),
            online: false,
            focused: false,
            sem: None,
        }
    }

    pub fn image(mut self, image: ImageHandle) -> Self {
        self.image = Some(image);
        self
    }

    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self
    }

    pub fn online(mut self, online: bool) -> Self {
        self.online = online;
        self
    }

    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// A reader's focus: the ring around the disc.
    pub fn set_focused(&mut self, on: bool) {
        self.focused = on;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// The initial drawn without a photo: the first letter, uppercased.
    pub fn initial(&self) -> String {
        self.name
            .chars()
            .next()
            .map(|c| c.to_uppercase().collect())
            .unwrap_or_default()
    }

    /// Initial style for a disc of `px`: the ramp step nearest a third of
    /// the disc, weight 600, so the glyph keeps its share at every size.
    fn initial_style(theme: &Theme, px: f32) -> TextStyle {
        let ty = &theme.typography;
        let target = px / 3.0;
        let size = [
            ty.small,
            ty.caption,
            ty.body_sm,
            ty.body,
            ty.title_sm,
            ty.title,
        ]
        .into_iter()
        .min_by(|a, b| (a - target).abs().total_cmp(&(b - target).abs()))
        .unwrap_or(ty.body);
        ty.exact(size, size, 600)
    }

    pub fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        let px = self.size.px(theme);
        (px, px)
    }

    /// The disc drawn in `bounds`: its edge, no larger than the bounds'
    /// smaller side, centered.
    pub fn disc(&self, bounds: Rect, theme: &Theme) -> Rect {
        let px = self.size.px(theme).min(bounds.w).min(bounds.h).max(0.0);
        Rect::new(
            bounds.x + (bounds.w - px) / 2.0,
            bounds.y + (bounds.h - px) / 2.0,
            px,
            px,
        )
    }

    /// Draw the avatar centered in `bounds` (square of the smaller side).
    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let disc = self.disc(bounds, theme);
        let px = disc.w;
        if let Some(id) = &self.sem {
            c.bind(id, [disc.x, disc.y, disc.w, disc.h]);
        }
        match self.image {
            Some(image) => c.push_to_layer(
                layer,
                SceneNode::Image {
                    x: disc.x,
                    y: disc.y,
                    w: disc.w,
                    h: disc.h,
                    image,
                    corner_radius: px / 2.0,
                },
            ),
            None => plain_disc(c, layer, disc, &self.initial(), theme),
        }
        if self.online {
            let r = Self::online_rect(disc, theme);
            let t = &theme.message.avatar;
            let ring = r.w / 2.0 + t.online_ring;
            let (cx, cy) = r.center();
            c.push_to_layer(
                layer,
                rounded_rect(cx - ring, cy - ring, ring * 2.0, ring * 2.0, ring, t.ring.0),
            );
            c.push_to_layer(
                layer,
                rounded_rect(r.x, r.y, r.w, r.h, r.w / 2.0, t.online_fill.0),
            );
        }
        if self.focused {
            c.push_to_layer(layer, focus_ring(disc, px / 2.0, theme));
        }
    }

    /// The online dot on `disc`: centered on the rim at the bottom end, at
    /// 45 degrees, as the kit's marks sit on an avatar
    /// (`Notification:31`, `-right-1 -bottom-1`).
    pub fn online_rect(disc: Rect, theme: &Theme) -> Rect {
        let d = theme.message.avatar.online;
        let (cx, cy) = disc.center();
        let reach = disc.w / 2.0 * std::f32::consts::FRAC_1_SQRT_2;
        Rect::new(cx + reach - d / 2.0, cy + reach - d / 2.0, d, d)
    }

    /// A reader's view: an image named by the person, online or not,
    /// focusable so a reader lands on it. It declares no action.
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let mut node = b.node(id, Role::Image, self.name.as_str());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("online", self.online);
        node.focusable(true);
        node
    }
}

/// The disc without a photo: the kit's fill, its hairline, and `text`
/// (the initial, or a stack's count) centered in the ink.
fn plain_disc(c: &mut Compositor, layer: LayerId, disc: Rect, text: &str, theme: &Theme) {
    let t = &theme.message.avatar;
    let r = disc.w / 2.0;
    c.push_to_layer(
        layer,
        rounded_rect(disc.x, disc.y, disc.w, disc.h, r, t.fill.0),
    );
    c.push_to_layer(
        layer,
        rounded_rect_stroke(disc.x, disc.y, disc.w, disc.h, r, t.edge.0, t.edge_width),
    );
    if text.is_empty() {
        return;
    }
    let style = Avatar::initial_style(theme, disc.w);
    let (tw, _) = TextMeasurer::measure_styled(text, &style, None);
    if tw > disc.w {
        return;
    }
    c.push_to_layer(
        layer,
        SceneNode::Text {
            key: TextNodeKey::from_style(text, &style, None),
            x: disc.x + (disc.w - tw) / 2.0,
            y: disc.y + TextMeasurer::vertical_center(&style, disc.h),
            color: t.initial.0,
        },
    );
}

/// The kit's stack of people: overlapping discs in paper rings, the first
/// on top, the rest counted on the last disc where the row runs out.
#[derive(Clone, Debug)]
pub struct AvatarStack {
    people: Vec<Avatar>,
    size: AvatarSize,
    sem: Option<SemId>,
}

/// Where a stack lands in its bounds: each shown person's ring, and the
/// count's ring with how many it holds.
#[derive(Clone, Debug, PartialEq)]
pub struct StackLayout {
    pub rect: Rect,
    pub people: Vec<Rect>,
    pub more: Option<(Rect, usize)>,
}

impl AvatarStack {
    /// The kit's stack: people at `Sm`.
    pub fn new(people: impl IntoIterator<Item = Avatar>) -> Self {
        Self {
            people: people.into_iter().collect(),
            size: AvatarSize::Sm,
            sem: None,
        }
        .size(AvatarSize::Sm)
    }

    /// The stack's edge, every person's with it.
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        for p in &mut self.people {
            p.size = size;
        }
        self
    }

    /// Binds the stack and gives each person an id under it,
    /// `<id>.avatar[<i>]`, and the count `<id>.more`.
    pub fn sem(mut self, id: SemId) -> Self {
        for (i, p) in self.people.iter_mut().enumerate() {
            if let Some(pid) = id.child("avatar").ok().map(|f| f.key(i.to_string())) {
                p.sem = Some(pid);
            }
        }
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn people(&self) -> &[Avatar] {
        &self.people
    }

    pub fn people_mut(&mut self) -> &mut [Avatar] {
        &mut self.people
    }

    /// Edge of a disc, and the step from one disc to the next.
    fn steps(&self, theme: &Theme) -> (f32, f32) {
        let px = self.size.px(theme);
        (px, px * (1.0 - theme.message.avatar.stack_overlap))
    }

    /// Everyone, side by side as the kit stacks them.
    pub fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        let (px, step) = self.steps(theme);
        let n = self.people.len().max(1) as f32;
        (px + (n - 1.0) * step, px)
    }

    /// The discs that fit `bounds.w` from its start, vertically centered:
    /// everyone, or as many less one as fit and a disc counting the rest.
    pub fn layout(&self, bounds: Rect, theme: &Theme) -> StackLayout {
        let (px, step) = self.steps(theme);
        let n = self.people.len();
        let fit = if bounds.w < px {
            0
        } else {
            1 + ((bounds.w - px) / step).floor() as usize
        };
        let (shown, more) = if fit >= n {
            (n, 0)
        } else {
            let shown = fit.saturating_sub(1);
            (shown, n - shown)
        };
        let y = bounds.y + (bounds.h - px) / 2.0;
        let at = |i: usize| Rect::new(bounds.x + i as f32 * step, y, px, px);
        let people: Vec<Rect> = (0..shown).map(at).collect();
        let more = (more > 0 && fit > 0).then(|| (at(shown), more));
        let slots = shown + usize::from(more.is_some());
        let w = if slots == 0 {
            0.0
        } else {
            px + (slots - 1) as f32 * step
        };
        StackLayout {
            rect: Rect::new(bounds.x, y, w, px),
            people,
            more,
        }
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let t = &theme.message.avatar;
        let l = self.layout(bounds, theme);
        if let Some(id) = &self.sem {
            c.bind(id, [l.rect.x, l.rect.y, l.rect.w, l.rect.h]);
        }
        let ringed = |c: &mut Compositor, slot: Rect| {
            c.push(rounded_rect(
                slot.x,
                slot.y,
                slot.w,
                slot.h,
                slot.w / 2.0,
                t.ring.0,
            ));
            slot.inset(t.stack_ring)
        };
        // the first on top: drawn last
        if let Some((slot, n)) = l.more {
            let disc = ringed(c, slot);
            plain_disc(c, LayerId::DEFAULT, disc, &format!("+{n}"), theme);
            // the people the count holds are placed on it
            for p in &self.people[l.people.len()..] {
                if let Some(id) = &p.sem {
                    c.bind(id, [disc.x, disc.y, disc.w, disc.h]);
                }
            }
        }
        for (person, slot) in self.people.iter().zip(&l.people).rev() {
            let disc = ringed(c, *slot);
            person.render(c, disc, theme);
        }
    }

    /// A reader's view: a list named `label` with everyone in it, those
    /// the count holds included (placed on the count).
    pub fn project(
        &self,
        b: &mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
        label: &str,
    ) {
        let mut node = b.node(id, Role::List, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        for p in &self.people {
            if let Some(pid) = &p.sem {
                p.project(b, pid, Some(id));
            }
        }
    }
}
