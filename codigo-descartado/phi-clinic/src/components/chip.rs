use phi::builder::{div, text, Element};
use phi::color::Color;

use crate::theme::*;

/// Chip variant — the only colored element in the design system.
#[derive(Clone, Copy)]
pub enum ChipVariant {
    Essencial,
    Recomendado,
    Opcional,
    Presente,
    Config,
    Basico,
}

impl ChipVariant {
    fn color(self) -> Color {
        match self {
            Self::Essencial => CHIP_ESSENCIAL,
            Self::Recomendado => CHIP_RECOMENDADO,
            Self::Opcional => CHIP_OPCIONAL,
            Self::Presente => CHIP_PRESENTE,
            Self::Config => CHIP_CONFIG,
            Self::Basico => CHIP_BASICO,
        }
    }

    fn text_color(self) -> Color {
        match self {
            Self::Opcional | Self::Config => TEXT_ACCENT, // white text on blue/purple
            _ => self.color(),
        }
    }
}

/// Chip size.
#[derive(Clone, Copy)]
pub enum ChipSize {
    Xs,
    Sm,
    Md,
}

/// Build a chip badge.
pub fn chip(label: &str, variant: ChipVariant) -> ChipBuilder {
    ChipBuilder {
        label: label.to_string(),
        variant,
        size: ChipSize::Sm,
    }
}

pub struct ChipBuilder {
    label: String,
    variant: ChipVariant,
    size: ChipSize,
}

impl ChipBuilder {
    pub fn size(mut self, s: ChipSize) -> Self {
        self.size = s;
        self
    }

    pub fn build(self) -> Element {
        let c = self.variant.color();
        let bg = chip_bg(c);
        let tc = self.variant.text_color();

        let (font_size, px, py) = match self.size {
            ChipSize::Xs => (FONT_2XS, 4.0, 1.0),
            ChipSize::Sm => (FONT_XS, 6.0, 2.0),
            ChipSize::Md => (FONT_SM, 8.0, 2.0),
        };

        div()
            .bg(bg)
            .rounded(2.0)
            .px(px)
            .py(py)
            .child(
                text(&self.label)
                    .font_size(font_size)
                    .bold()
                    .uppercase()
                    .tracking(0.1)
                    .text_color(tc),
            )
    }
}
