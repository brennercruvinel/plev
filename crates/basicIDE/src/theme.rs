/// Design token system for basicIDE UI.
///
/// Colors approximated from @basicIDE/design-core visual inspection.
/// All values are RGBA in 0.0–1.0 range, premultiplied-alpha-safe.
use plev::color::Color;

// Catálogo de tokens: campos ainda não consumidos pelas views fazem parte
// da paleta e ficam disponíveis para os próximos componentes.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    // -- Backgrounds --
    pub bg_1: Color,     // main surface
    pub bg_2: Color,     // secondary surface (cards, panels)
    pub bg_3: Color,     // tertiary surface (headers, selected rows)
    pub bg_muted: Color, // very subtle background tint

    // -- Text --
    pub text_1: Color, // primary text
    pub text_2: Color, // secondary / label text
    pub text_3: Color, // hint / disabled text

    // -- Semantic --
    pub pop: Color,    // accent / action blue-purple
    pub danger: Color, // error / destructive red
    pub safe: Color,   // success green
    pub warn: Color,   // warning amber
    pub purple: Color, // purple accent

    // -- Borders --
    pub border: Color,       // default border
    pub border_focus: Color, // focused element border

    // -- Interactive overlays --
    pub hover_bg_1: Color, // hover on bg_1
    pub hover_bg_2: Color, // hover on bg_2
    pub hover_bg_3: Color, // hover on bg_3

    // -- Transition timing (seconds) --
    pub fast: f32,   // 0.05s
    pub medium: f32, // 0.15s
    pub slow: f32,   // 0.20s

    // -- Border radii --
    pub radius_s: f32,  // small: 4px
    pub radius_ml: f32, // medium-large: 8px
    pub radius_l: f32,  // large: 12px
}

pub const DARK: Theme = Theme {
    bg_1: Color::hex(0x1A1D23),
    bg_2: Color::hex(0x22262E),
    bg_3: Color::hex(0x2A2F39),
    bg_muted: Color::hex(0x161920),

    text_1: Color::hex(0xE2E7EE),
    text_2: Color::hex(0x8E97A8),
    text_3: Color::hex(0x545C6B),

    pop: Color::hex(0x6B6BFF),
    danger: Color::hex(0xFF5252),
    safe: Color::hex(0x3CBF7A),
    warn: Color::hex(0xF0B429),
    purple: Color::hex(0xA855F7),

    border: Color::hex(0x2D3240),
    border_focus: Color::hex(0x6B6BFF),

    hover_bg_1: Color::hex(0x1F232B),
    hover_bg_2: Color::hex(0x272C36),
    hover_bg_3: Color::hex(0x303642),

    fast: 0.05,
    medium: 0.15,
    slow: 0.20,

    radius_s: 4.0,
    radius_ml: 8.0,
    radius_l: 12.0,
};

pub const LIGHT: Theme = Theme {
    bg_1: Color::hex(0xFFFFFF),
    bg_2: Color::hex(0xF4F5F7),
    bg_3: Color::hex(0xECEDF1),
    bg_muted: Color::hex(0xFAFAFC),

    text_1: Color::hex(0x1A1D24),
    text_2: Color::hex(0x50545F),
    text_3: Color::hex(0x9098A8),

    pop: Color::hex(0x5A50DF),
    danger: Color::hex(0xE33030),
    safe: Color::hex(0x2DA66B),
    warn: Color::hex(0xD4940A),
    purple: Color::hex(0x7C3AED),

    border: Color::hex(0xD8DCE6),
    border_focus: Color::hex(0x5A50DF),

    hover_bg_1: Color::hex(0xF7F8FA),
    hover_bg_2: Color::hex(0xEDEFF3),
    hover_bg_3: Color::hex(0xE4E6EC),

    fast: 0.05,
    medium: 0.15,
    slow: 0.20,

    radius_s: 4.0,
    radius_ml: 8.0,
    radius_l: 12.0,
};

/// File status colors.
#[allow(dead_code)]
pub struct StatusColors {
    pub modified: Color,
    pub added: Color,
    pub deleted: Color,
    pub renamed: Color,
    pub untracked: Color,
}

#[allow(dead_code)]
impl StatusColors {
    pub fn dark() -> Self {
        Self {
            modified: Color::hex(0xF0B429),
            added: Color::hex(0x3CBF7A),
            deleted: Color::hex(0xFF5252),
            renamed: Color::hex(0x6B6BFF),
            untracked: Color::hex(0x8E97A8),
        }
    }

    pub fn light() -> Self {
        Self {
            modified: Color::hex(0xD4940A),
            added: Color::hex(0x2DA66B),
            deleted: Color::hex(0xE33030),
            renamed: Color::hex(0x5A50DF),
            untracked: Color::hex(0x9098A8),
        }
    }
}
