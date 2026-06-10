/// Color type for the builder API.
/// Stored as [f32; 4] RGBA, all values in 0.0..=1.0.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(pub [f32; 4]);

impl Default for Color {
    fn default() -> Self {
        Color::TRANSPARENT
    }
}

impl Color {
    pub const WHITE: Color = Color([1.0, 1.0, 1.0, 1.0]);
    pub const BLACK: Color = Color([0.0, 0.0, 0.0, 1.0]);
    pub const RED: Color = Color([1.0, 0.0, 0.0, 1.0]);
    pub const GREEN: Color = Color([0.0, 0.8, 0.3, 1.0]);
    pub const BLUE: Color = Color([0.2, 0.4, 1.0, 1.0]);
    pub const YELLOW: Color = Color([1.0, 0.9, 0.0, 1.0]);
    pub const ORANGE: Color = Color([1.0, 0.6, 0.0, 1.0]);
    pub const PURPLE: Color = Color([0.6, 0.2, 0.8, 1.0]);
    pub const PINK: Color = Color([1.0, 0.4, 0.7, 1.0]);
    pub const CYAN: Color = Color([0.0, 0.8, 0.9, 1.0]);
    pub const GRAY: Color = Color([0.5, 0.5, 0.5, 1.0]);
    pub const DARK_GRAY: Color = Color([0.2, 0.2, 0.2, 1.0]);
    pub const LIGHT_GRAY: Color = Color([0.8, 0.8, 0.8, 1.0]);
    pub const TRANSPARENT: Color = Color([0.0, 0.0, 0.0, 0.0]);

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color([r, g, b, a])
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Color([r, g, b, 1.0])
    }

    /// Create a color from a hex value like 0xFF8800 or 0xFF880080 (with alpha).
    pub const fn hex(hex: u32) -> Self {
        if hex > 0xFF_FF_FF {
            // 8-digit hex: RRGGBBAA
            Color([
                ((hex >> 24) & 0xFF) as f32 / 255.0,
                ((hex >> 16) & 0xFF) as f32 / 255.0,
                ((hex >> 8) & 0xFF) as f32 / 255.0,
                (hex & 0xFF) as f32 / 255.0,
            ])
        } else {
            // 6-digit hex: RRGGBB (alpha = 1.0)
            Color([
                ((hex >> 16) & 0xFF) as f32 / 255.0,
                ((hex >> 8) & 0xFF) as f32 / 255.0,
                (hex & 0xFF) as f32 / 255.0,
                1.0,
            ])
        }
    }

    pub const fn to_array(self) -> [f32; 4] {
        self.0
    }
}

pub trait IntoColor {
    fn into_color(self) -> Color;
}

impl IntoColor for Color {
    fn into_color(self) -> Color {
        self
    }
}

impl IntoColor for [f32; 4] {
    fn into_color(self) -> Color {
        Color(self)
    }
}

impl IntoColor for [f32; 3] {
    fn into_color(self) -> Color {
        Color([self[0], self[1], self[2], 1.0])
    }
}

impl IntoColor for &str {
    fn into_color(self) -> Color {
        match self {
            "white" => Color::WHITE,
            "black" => Color::BLACK,
            "red" => Color::RED,
            "green" => Color::GREEN,
            "blue" => Color::BLUE,
            "yellow" => Color::YELLOW,
            "orange" => Color::ORANGE,
            "purple" => Color::PURPLE,
            "pink" => Color::PINK,
            "cyan" => Color::CYAN,
            "gray" | "grey" => Color::GRAY,
            "dark_gray" | "dark_grey" => Color::DARK_GRAY,
            "light_gray" | "light_grey" => Color::LIGHT_GRAY,
            "transparent" => Color::TRANSPARENT,
            _ => Color::WHITE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_colors_correct() {
        assert_eq!(Color::RED.0, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(Color::WHITE.0, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(Color::TRANSPARENT.0, [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn rgb_rgba_constructors() {
        let c = Color::rgb(0.5, 0.6, 0.7);
        assert_eq!(c.0[3], 1.0);

        let c = Color::rgba(0.1, 0.2, 0.3, 0.4);
        assert_eq!(c.0, [0.1, 0.2, 0.3, 0.4]);
    }

    #[test]
    fn hex_6digit() {
        let c = Color::hex(0xFF0000);
        assert!((c.0[0] - 1.0).abs() < 0.01);
        assert!((c.0[1]).abs() < 0.01);
        assert!((c.0[2]).abs() < 0.01);
        assert_eq!(c.0[3], 1.0);
    }

    #[test]
    fn hex_8digit() {
        let c = Color::hex(0xFF000080);
        assert!((c.0[0] - 1.0).abs() < 0.01);
        assert!((c.0[3] - 128.0 / 255.0).abs() < 0.01);
    }

    #[test]
    fn into_color_str() {
        assert_eq!("red".into_color(), Color::RED);
        assert_eq!("blue".into_color(), Color::BLUE);
        assert_eq!("dark_gray".into_color(), Color::DARK_GRAY);
        assert_eq!("dark_grey".into_color(), Color::DARK_GRAY);
    }

    #[test]
    fn into_color_array() {
        let c: Color = [0.1, 0.2, 0.3, 0.4].into_color();
        assert_eq!(c.0, [0.1, 0.2, 0.3, 0.4]);

        let c: Color = [0.5, 0.6, 0.7].into_color();
        assert_eq!(c.0, [0.5, 0.6, 0.7, 1.0]);
    }

    #[test]
    fn to_array() {
        assert_eq!(Color::RED.to_array(), [1.0, 0.0, 0.0, 1.0]);
    }
}
