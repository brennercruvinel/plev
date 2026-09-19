//! Design tokens for the demo, read from the engine theme (the HOFF
//! default): the same page, surface, text and intent colors every plev
//! app draws with. A demo palette used to be a second color system;
//! now it is a view over `Theme`.

use std::sync::LazyLock;

use engine::text::TextStyle;
use engine::theme::Theme;

pub(crate) struct Palette {
    pub bg: [f32; 4],
    pub header_bg: [f32; 4],
    pub surface: [f32; 4],
    pub accent: [f32; 4],
    pub accent_dim: [f32; 4],
    pub green: [f32; 4],
    pub red: [f32; 4],
    pub yellow: [f32; 4],
    pub purple: [f32; 4],
    pub cyan: [f32; 4],
    pub orange: [f32; 4],
    pub text: [f32; 4],
    pub text_dim: [f32; 4],
    pub text_mid: [f32; 4],
    pub divider: [f32; 4],
    pub footer_bg: [f32; 4],
    pub dot: [f32; 4],
    pub btn_normal: [f32; 4],
    pub btn_hover: [f32; 4],
    pub btn_border: [f32; 4],
}

impl Palette {
    fn from_theme(t: &Theme) -> Self {
        Self {
            bg: t.colors.bg.0,
            header_bg: t.colors.surface.0,
            surface: t.colors.bg_panel.0,
            accent: t.colors.accent.0,
            accent_dim: t.colors.accent_dim.0,
            green: t.colors.success.0,
            red: t.colors.danger.0,
            yellow: t.colors.warning.0,
            purple: t.colors.info.0,
            cyan: t.colors.info.0,
            orange: t.colors.warning.0,
            text: t.colors.text.0,
            text_dim: t.colors.text_dim.0,
            text_mid: t.colors.text_mid.0,
            divider: t.colors.divider.0,
            footer_bg: t.colors.surface.0,
            dot: t.glass.surface_active.0,
            btn_normal: t.glass.button.0,
            btn_hover: t.glass.button_hover.0,
            btn_border: t.glass.edge.0,
        }
    }
}

static PALETTE: LazyLock<Palette> = LazyLock::new(|| Palette::from_theme(&Theme::default()));

/// The demo palette, resolved once from the default theme.
pub(crate) fn pal() -> &'static Palette {
    &PALETTE
}

// One TextStyle per run, shared by measurement and drawing
// (docs/adr/one-text-style-for-measurement-and-drawing.md). Weights are
// semantic: 700 page title, 600 card title, 400 body; numeric status
// readouts render in the same embedded Inclusive Sans (the only UI family).

pub(crate) fn title_style(size: f32, line_height: f32) -> TextStyle {
    TextStyle::new(size)
        .with_line_height(line_height)
        .with_weight(700)
}

pub(crate) fn card_title_style(size: f32, line_height: f32) -> TextStyle {
    TextStyle::new(size)
        .with_line_height(line_height)
        .with_weight(600)
}

pub(crate) fn body_style(size: f32, line_height: f32) -> TextStyle {
    TextStyle::new(size).with_line_height(line_height)
}

pub(crate) fn code_style(size: f32, line_height: f32) -> TextStyle {
    TextStyle::new(size)
        .with_line_height(line_height)
        .with_family("Inclusive Sans")
}
