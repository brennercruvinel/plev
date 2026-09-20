//! Code block: monospaced-feel text (the mono ramp step, the single
//! embedded family) on a field-tone surface at the nav radius, wrapped
//! at the block width, with an optional caption line. The `code_style`
//! helper three gallery sections grew is this one widget.

use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::Theme;

use crate::core::Rect;
use crate::recipe::{rounded_rect, rounded_rect_stroke};

#[derive(Clone, Debug)]
pub struct CodeBlock {
    pub code: String,
    /// Small caption above the code (a file name, a language).
    pub caption: Option<String>,
}

impl CodeBlock {
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            caption: None,
        }
    }

    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    pub fn text_style(theme: &Theme) -> TextStyle {
        theme.typography.mono()
    }

    fn caption_style(theme: &Theme) -> TextStyle {
        theme.typography.caption_sm()
    }

    /// Inner padding: the `md` step.
    fn pad(theme: &Theme) -> f32 {
        theme.spacing.md
    }

    /// Height for a given width: the code wrapped at the inner width plus
    /// the padding and the caption band.
    pub fn height_for(&self, width: f32, theme: &Theme) -> f32 {
        let pad = Self::pad(theme);
        let style = Self::text_style(theme);
        let inner = (width - pad * 2.0).max(0.0);
        let (_, th) = TextMeasurer::measure_styled(&self.code, &style, Some(inner));
        let caption = self
            .caption
            .as_ref()
            .map(|_| Self::caption_style(theme).line_height + theme.spacing.xs)
            .unwrap_or(0.0);
        pad + caption + th.max(style.line_height) + pad
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let glass = &theme.glass;
        let pad = Self::pad(theme);
        let radius = theme.shape.nav;
        c.push_to_layer(
            layer,
            rounded_rect(
                bounds.x,
                bounds.y,
                bounds.w,
                bounds.h,
                radius,
                glass.field.0,
            ),
        );
        c.push_to_layer(
            layer,
            rounded_rect_stroke(
                bounds.x,
                bounds.y,
                bounds.w,
                bounds.h,
                radius,
                glass.edge_soft.0,
                theme.control.edge_width,
            ),
        );
        let inner = (bounds.w - pad * 2.0).max(0.0);
        let mut y = bounds.y + pad;
        if let Some(caption) = &self.caption {
            let style = Self::caption_style(theme);
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(caption, &style, Some(inner)),
                    x: bounds.x + pad,
                    y,
                    color: glass.text_faint.0,
                },
            );
            y += style.line_height + theme.spacing.xs;
        }
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&self.code, &Self::text_style(theme), Some(inner)),
                x: bounds.x + pad,
                y,
                color: theme.colors.text_mid.0,
            },
        );
    }
}
