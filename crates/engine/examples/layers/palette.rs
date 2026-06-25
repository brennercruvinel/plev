//! Color palette and typography for the layers demo.

use engine::text::TextStyle;

pub const BG: [f32; 4] = [0.06, 0.06, 0.10, 1.0];
pub const HEADER_BG: [f32; 4] = [0.08, 0.08, 0.14, 1.0];
pub const SURFACE: [f32; 4] = [0.10, 0.10, 0.16, 1.0];
pub const ACCENT: [f32; 4] = [0.30, 0.55, 1.0, 1.0];
pub const ACCENT_DIM: [f32; 4] = [0.20, 0.35, 0.70, 1.0];
pub const GREEN: [f32; 4] = [0.20, 0.80, 0.45, 1.0];
pub const RED: [f32; 4] = [1.0, 0.30, 0.25, 1.0];
pub const YELLOW: [f32; 4] = [1.0, 0.85, 0.20, 1.0];
pub const PURPLE: [f32; 4] = [0.60, 0.30, 0.90, 1.0];
pub const CYAN: [f32; 4] = [0.0, 0.85, 0.95, 1.0];
pub const ORANGE: [f32; 4] = [1.0, 0.55, 0.10, 1.0];
pub const TEXT: [f32; 4] = [0.93, 0.93, 0.96, 1.0];
pub const TEXT_DIM: [f32; 4] = [0.55, 0.55, 0.65, 1.0];
pub const TEXT_MID: [f32; 4] = [0.75, 0.75, 0.85, 1.0];
pub const DIVIDER: [f32; 4] = [0.18, 0.18, 0.25, 1.0];
pub const FOOTER_BG: [f32; 4] = [0.07, 0.07, 0.12, 1.0];
pub const DOT_COLOR: [f32; 4] = [0.12, 0.12, 0.18, 1.0];

// One TextStyle per run, shared by measurement and drawing
// (kdb/adr/one-text-style-for-measurement-and-drawing.md). Weights are
// semantic: 700 page title, 600 card title, 400 body; aligned readouts
// render in JetBrains Mono.

pub fn title_style(size: f32, line_height: f32) -> TextStyle {
    TextStyle::new(size)
        .with_line_height(line_height)
        .with_weight(700)
}

pub fn card_title_style(size: f32, line_height: f32) -> TextStyle {
    TextStyle::new(size)
        .with_line_height(line_height)
        .with_weight(600)
}

pub fn body_style(size: f32, line_height: f32) -> TextStyle {
    TextStyle::new(size).with_line_height(line_height)
}

pub fn code_style(size: f32, line_height: f32) -> TextStyle {
    TextStyle::new(size)
        .with_line_height(line_height)
        .with_family("JetBrains Mono")
}
