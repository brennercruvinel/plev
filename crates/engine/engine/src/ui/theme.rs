/// Semantic color accent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Accent {
    Gray,
    Pop,
    Safe,
    Warn,
    Danger,
    Purple,
}

/// Minimal theme tokens for UI rendering.
#[derive(Clone, Debug)]
pub struct UiTheme {
    pub bg: [[f32; 4]; 4],
    pub text: [[f32; 4]; 3],
    pub border: [f32; 4],
    pub accent_bg: [[f32; 4]; 6], // Gray, Pop, Safe, Warn, Danger, Purple
    pub accent_fg: [[f32; 4]; 6],
    pub accent_soft_bg: [[f32; 4]; 6],
    pub accent_soft_fg: [[f32; 4]; 6],
    pub radius: [f32; 4], // s, m, ml, l
}

impl UiTheme {
    pub fn dark() -> Self {
        Self {
            bg: [
                [0.102, 0.114, 0.137, 1.0],
                [0.133, 0.149, 0.180, 1.0],
                [0.165, 0.184, 0.224, 1.0],
                [0.086, 0.098, 0.125, 1.0],
            ],
            text: [
                [0.886, 0.906, 0.933, 1.0],
                [0.557, 0.592, 0.659, 1.0],
                [0.329, 0.361, 0.420, 1.0],
            ],
            border: [0.176, 0.196, 0.251, 1.0],
            accent_bg: [
                [0.25, 0.27, 0.31, 1.0], // Gray
                [0.42, 0.42, 1.0, 1.0],  // Pop
                [0.24, 0.75, 0.48, 1.0], // Safe
                [0.94, 0.70, 0.16, 1.0], // Warn
                [1.0, 0.32, 0.32, 1.0],  // Danger
                [0.66, 0.33, 0.97, 1.0], // Purple
            ],
            accent_fg: [
                [0.886, 0.906, 0.933, 1.0], // Gray
                [1.0, 1.0, 1.0, 1.0],       // Pop
                [0.0, 0.0, 0.0, 1.0],       // Safe
                [0.0, 0.0, 0.0, 1.0],       // Warn
                [1.0, 1.0, 1.0, 1.0],       // Danger
                [1.0, 1.0, 1.0, 1.0],       // Purple
            ],
            accent_soft_bg: [
                [0.18, 0.20, 0.24, 1.0],
                [0.42, 0.42, 1.0, 0.15],
                [0.24, 0.75, 0.48, 0.15],
                [0.94, 0.70, 0.16, 0.15],
                [1.0, 0.32, 0.32, 0.15],
                [0.66, 0.33, 0.97, 0.15],
            ],
            accent_soft_fg: [
                [0.886, 0.906, 0.933, 1.0],
                [0.55, 0.55, 1.0, 1.0],
                [0.30, 0.85, 0.55, 1.0],
                [0.94, 0.75, 0.25, 1.0],
                [1.0, 0.45, 0.45, 1.0],
                [0.72, 0.45, 0.97, 1.0],
            ],
            radius: [4.0, 6.0, 8.0, 12.0],
        }
    }

    pub(crate) fn accent_idx(a: Accent) -> usize {
        match a {
            Accent::Gray => 0,
            Accent::Pop => 1,
            Accent::Safe => 2,
            Accent::Warn => 3,
            Accent::Danger => 4,
            Accent::Purple => 5,
        }
    }
}
