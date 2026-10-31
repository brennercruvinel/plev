//! HOFF avatar: a circle one `Md` control wide (or any size the owner
//! picks), showing a photo or, without one, the chip-glass disc with the
//! initial letter in the active text tone. An optional online dot, the
//! kit's green mark ringed by the paper (`theme.message.avatar`), sits on
//! the bottom end rim.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::gpu::image::ImageHandle;
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::{ControlSize, Theme};

use crate::widgets::recipe::{glass_pill, rounded_rect};
use crate::widgets::widget::Rect;

/// Avatar sizes on the control ladder.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarSize {
    /// Inline in a row: `Xs`.
    Sm,
    /// The HOFF default: `Md` (44).
    #[default]
    Md,
    /// Profile header: two `Md` controls.
    Lg,
}

impl AvatarSize {
    pub fn px(self, theme: &Theme) -> f32 {
        match self {
            AvatarSize::Sm => theme.control.height(ControlSize::Xs),
            AvatarSize::Md => theme.control.height(ControlSize::Md),
            AvatarSize::Lg => theme.control.height(ControlSize::Md) * 2.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Avatar {
    /// The name the initial is taken from.
    pub name: String,
    pub image: Option<ImageHandle>,
    pub size: AvatarSize,
    pub online: bool,
}

impl Avatar {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            image: None,
            size: AvatarSize::default(),
            online: false,
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

    /// Draw the avatar centered in `bounds` (square of the smaller side).
    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let px = self.size.px(theme).min(bounds.w).min(bounds.h);
        let disc = Rect::new(
            bounds.x + (bounds.w - px) / 2.0,
            bounds.y + (bounds.h - px) / 2.0,
            px,
            px,
        );
        let glass = &theme.glass;
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
            None => {
                for node in glass_pill(
                    disc,
                    px / 2.0,
                    glass.edge.0,
                    theme.control.edge_width_strong,
                    glass.surface_active.0,
                    theme,
                ) {
                    c.push_to_layer(layer, node);
                }
                let initial = self.initial();
                if !initial.is_empty() {
                    let style = Self::initial_style(theme, px);
                    let (tw, _) = TextMeasurer::measure_styled(&initial, &style, None);
                    c.push_to_layer(
                        layer,
                        SceneNode::Text {
                            key: TextNodeKey::from_style(&initial, &style, None),
                            x: disc.x + (disc.w - tw) / 2.0,
                            y: disc.y + TextMeasurer::vertical_center(&style, disc.h),
                            color: glass.text_active.0,
                        },
                    );
                }
            }
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
}
