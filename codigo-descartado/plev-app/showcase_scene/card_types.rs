//! Shared types and constants for showcase card functions.

pub(crate) const PURPLE: [f32; 4] = [0.60, 0.30, 0.90, 1.0];
pub(crate) const PINK: [f32; 4] = [1.0, 0.40, 0.70, 1.0];
pub(crate) const ORANGE: [f32; 4] = [1.0, 0.55, 0.10, 1.0];
pub(crate) const HOVER: [f32; 4] = [0.35, 0.60, 1.0, 1.0];

/// Card position, size, and common surface colors shared by all cards.
#[derive(Clone, Copy)]
pub(crate) struct CardLayout {
    pub cx: f32,
    pub cy: f32,
    pub card_w: f32,
    pub card_h: f32,
    pub surface: [f32; 4],
    pub accent_dim: [f32; 4],
}

/// Full theme color palette passed to card functions that need many colors.
#[derive(Clone, Copy)]
pub(crate) struct CardColors {
    pub accent: [f32; 4],
    pub green: [f32; 4],
    pub red: [f32; 4],
    pub yellow: [f32; 4],
    pub cyan: [f32; 4],
    pub text: [f32; 4],
    pub text_dim: [f32; 4],
    pub text_mid: [f32; 4],
}
