//! HOFF avatar: a circle one `Md` control wide (or any size the owner
//! picks), showing a photo or, without one, the chip-glass disc with the
//! initial letter in the active text tone. An optional online dot in the
//! knob gradient sits on the top-left rim.

use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::gpu::image::ImageHandle;
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::{ControlSize, Theme};

use crate::core::Rect;
use crate::recipe::glass_pill;

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
        TextStyle::new(size).with_line_height(size).with_weight(600)
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
            let d = theme.spacing.md;
            let off = theme.control.edge_width * 2.0;
            c.push_to_layer(
                layer,
                SceneNode::GradientRect {
                    x: disc.x - off,
                    y: disc.y - off,
                    w: d,
                    h: d,
                    color: glass.knob_gradient[0].0,
                    color2: glass.knob_gradient[1].0,
                    angle_deg: 180.0,
                    corner_radius: d / 2.0,
                    border_width: 0.0,
                    border_color: [0.0; 4],
                },
            );
        }
    }
}
