#![allow(dead_code)]
use plev::color::Color;

// ---------------------------------------------------------------------------
// Surface palette — grayscale depth layers
// ---------------------------------------------------------------------------

pub const SURFACE_0: Color = Color::hex(0x030303); // deepest black, app background
pub const SURFACE_1: Color = Color::hex(0x080808); // panels, main bg
pub const SURFACE_2: Color = Color::hex(0x0e0e0e); // hover, subtle elevation
pub const SURFACE_3: Color = Color::hex(0x181818); // borders, dividers
pub const SURFACE_4: Color = Color::hex(0x222222); // emphasis borders
pub const SURFACE_5: Color = Color::hex(0x262626); // light accents

// ---------------------------------------------------------------------------
// Text hierarchy
// ---------------------------------------------------------------------------

pub const TEXT_PRIMARY: Color = Color::hex(0xe5e5e5); // main text
pub const TEXT_MUTED: Color = Color::hex(0x555555);   // secondary, labels
pub const TEXT_DIM: Color = Color::hex(0x333333);      // disabled, very subtle
pub const TEXT_ACCENT: Color = Color::hex(0xffffff);   // white, emphasis

// ---------------------------------------------------------------------------
// Border
// ---------------------------------------------------------------------------

pub const BORDER_DEFAULT: Color = Color::hex(0x181818);
pub const BORDER_HOVER: Color = Color::hex(0x222222);

// ---------------------------------------------------------------------------
// Chip variants — only color in entire B&W design
// ---------------------------------------------------------------------------

pub const CHIP_ESSENCIAL: Color = Color::hex(0xef4444);   // red
pub const CHIP_RECOMENDADO: Color = Color::hex(0xf59e0b); // amber
pub const CHIP_OPCIONAL: Color = Color::hex(0x3b82f6);    // blue
pub const CHIP_PRESENTE: Color = Color::hex(0x22c55e);    // green
pub const CHIP_CONFIG: Color = Color::hex(0x8b5cf6);      // purple
pub const CHIP_BASICO: Color = Color::hex(0x06b6d4);      // cyan

// ---------------------------------------------------------------------------
// HUD decorative
// ---------------------------------------------------------------------------

pub const HUD_CORNER_COLOR: Color = Color::hex(0x222222);
pub const HUD_CORNER_SIZE: f32 = 12.0;

// Alpha helpers
pub const WHITE_70: Color = Color::rgba(1.0, 1.0, 1.0, 0.7);
pub const WHITE_30: Color = Color::rgba(1.0, 1.0, 1.0, 0.3);
pub const WHITE_20: Color = Color::rgba(1.0, 1.0, 1.0, 0.2);
pub const WHITE_10: Color = Color::rgba(1.0, 1.0, 1.0, 0.1);

// ---------------------------------------------------------------------------
// Spacing scale (px)
// ---------------------------------------------------------------------------

pub const SPACE_XS: f32 = 4.0;
pub const SPACE_SM: f32 = 8.0;
pub const SPACE_MD: f32 = 16.0;
pub const SPACE_LG: f32 = 24.0;
pub const SPACE_XL: f32 = 32.0;

// ---------------------------------------------------------------------------
// Font sizes (px)
// ---------------------------------------------------------------------------

pub const FONT_2XS: f32 = 7.0;
pub const FONT_XS: f32 = 8.0;
pub const FONT_SM: f32 = 9.0;
pub const FONT_BASE: f32 = 11.0;
pub const FONT_LG: f32 = 13.0;
pub const FONT_XL: f32 = 16.0;
pub const FONT_2XL: f32 = 22.0;
pub const FONT_3XL: f32 = 28.0;

// ---------------------------------------------------------------------------
// Chip color helpers — bg at 20% opacity
// ---------------------------------------------------------------------------

pub fn chip_bg(color: Color) -> Color {
    Color::rgba(color.0[0], color.0[1], color.0[2], 0.2)
}
