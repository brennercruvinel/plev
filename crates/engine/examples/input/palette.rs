// Design tokens for the input demo.

use engine::compositor::Compositor;
use engine::text::TextStyle;

pub(crate) const BG: [f32; 4] = [0.06, 0.06, 0.10, 1.0];
pub(crate) const HEADER_BG: [f32; 4] = [0.08, 0.08, 0.14, 1.0];
pub(crate) const SURFACE: [f32; 4] = [0.10, 0.10, 0.16, 1.0];
pub(crate) const ACCENT: [f32; 4] = [0.30, 0.55, 1.0, 1.0];
pub(crate) const ACCENT_DIM: [f32; 4] = [0.20, 0.35, 0.70, 1.0];
pub(crate) const _GREEN: [f32; 4] = [0.20, 0.80, 0.45, 1.0];
pub(crate) const _RED: [f32; 4] = [1.0, 0.30, 0.25, 1.0];
pub(crate) const _YELLOW: [f32; 4] = [1.0, 0.85, 0.20, 1.0];
pub(crate) const _PURPLE: [f32; 4] = [0.60, 0.30, 0.90, 1.0];
pub(crate) const CYAN: [f32; 4] = [0.0, 0.85, 0.95, 1.0];
pub(crate) const _ORANGE: [f32; 4] = [1.0, 0.55, 0.10, 1.0];
pub(crate) const TEXT: [f32; 4] = [0.93, 0.93, 0.96, 1.0];
pub(crate) const TEXT_DIM: [f32; 4] = [0.55, 0.55, 0.65, 1.0];
pub(crate) const TEXT_MID: [f32; 4] = [0.75, 0.75, 0.85, 1.0];
pub(crate) const DIVIDER: [f32; 4] = [0.18, 0.18, 0.25, 1.0];
pub(crate) const FOOTER_BG: [f32; 4] = [0.07, 0.07, 0.12, 1.0];

pub(crate) const BTN_NORMAL: [f32; 4] = [0.18, 0.38, 0.85, 1.0];
pub(crate) const BTN_HOVER: [f32; 4] = [0.30, 0.55, 1.0, 1.0];
pub(crate) const BTN_BORDER: [f32; 4] = [0.40, 0.65, 1.0, 0.30];

pub(crate) fn card(compositor: &mut Compositor, x: f32, y: f32, w: f32, h: f32, accent: [f32; 4]) {
    compositor.draw_rect(x, y, w, h, SURFACE);
    compositor.draw_rect(x + 1.0, y, w - 2.0, 2.0, accent);
}

// One TextStyle per run, shared by measurement and drawing
// (kdb/adr/one-text-style-for-measurement-and-drawing.md). Weights are
// semantic: 700 page title, 600 card title, 500 label, 400 body; code and
// counters render in JetBrains Mono.

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

pub(crate) fn label_style(size: f32, line_height: f32) -> TextStyle {
    TextStyle::new(size)
        .with_line_height(line_height)
        .with_weight(500)
}

pub(crate) fn body_style(size: f32, line_height: f32) -> TextStyle {
    TextStyle::new(size).with_line_height(line_height)
}

pub(crate) fn code_style(size: f32, line_height: f32) -> TextStyle {
    TextStyle::new(size)
        .with_line_height(line_height)
        .with_family("JetBrains Mono")
}
