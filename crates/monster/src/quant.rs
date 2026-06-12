//! quantization primitives of doc/monster-format-v0.md decision 5: values
//! are quantized in the file and f32 in memory. coordinates are i32
//! twentieths of a logical px (the twips lesson: integer determinism),
//! colors rgba8, angles and ratios u16 fixed point, custom bezier
//! control points u8 (x in [0,1], y in [-0.5,1.5] covers overshoot).
//!
//! every pair round-trips exactly from the quantized side:
//! quantize(dequantize(q)) == q. the float side carries at most half a
//! quantization step of error.

/// Twentieths of a logical px per px.
pub const TWIPS_PER_PX: i32 = 20;

/// Logical px to twips (nearest twentieth).
pub fn px_to_twips(px: f32) -> i32 {
    (px * TWIPS_PER_PX as f32).round() as i32
}

/// Twips back to logical px.
pub fn twips_to_px(twips: i32) -> f32 {
    twips as f32 / TWIPS_PER_PX as f32
}

/// One color channel in [0,1] to a byte; out-of-range input clamps.
pub fn channel_to_u8(c: f32) -> u8 {
    (c.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Byte back to a channel in [0,1].
pub fn u8_to_channel(b: u8) -> f32 {
    b as f32 / 255.0
}

/// rgba color (f32 in [0,1] per channel) to rgba8.
pub fn rgba_to_bytes(color: [f32; 4]) -> [u8; 4] {
    color.map(channel_to_u8)
}

/// rgba8 back to f32 channels.
pub fn bytes_to_rgba(bytes: [u8; 4]) -> [f32; 4] {
    bytes.map(u8_to_channel)
}

/// Ratio in [0,1] to u16 fixed point (0 -> 0x0000, 1 -> 0xFFFF);
/// out-of-range input clamps.
pub fn ratio_to_u16(r: f32) -> u16 {
    (r.clamp(0.0, 1.0) * u16::MAX as f32).round() as u16
}

/// u16 fixed point back to a ratio in [0,1].
pub fn u16_to_ratio(q: u16) -> f32 {
    q as f32 / u16::MAX as f32
}

/// Angle in degrees to u16 fixed point over one turn: the angle wraps
/// into [0,360) and maps to [0,65536), so 360 encodes as 0 and lerp in
/// quantized space stays meaningful.
pub fn angle_deg_to_u16(deg: f32) -> u16 {
    let turn = deg.rem_euclid(360.0) / 360.0;
    // round() may land exactly on 65536 (e.g. 359.999); wrap it home.
    ((turn * 65536.0).round() as u32 % 65536) as u16
}

/// u16 fixed point back to degrees in [0,360).
pub fn u16_to_angle_deg(q: u16) -> f32 {
    q as f32 / 65536.0 * 360.0
}

/// Bezier control point x in [0,1] to a byte; clamps.
pub fn bezier_x_to_u8(x: f32) -> u8 {
    (x.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Byte back to bezier x in [0,1].
pub fn u8_to_bezier_x(b: u8) -> f32 {
    b as f32 / 255.0
}

/// Bezier control point y in [-0.5,1.5] (overshoot range) to a byte;
/// clamps.
pub fn bezier_y_to_u8(y: f32) -> u8 {
    ((y.clamp(-0.5, 1.5) + 0.5) / 2.0 * 255.0).round() as u8
}

/// Byte back to bezier y in [-0.5,1.5].
pub fn u8_to_bezier_y(b: u8) -> f32 {
    b as f32 / 255.0 * 2.0 - 0.5
}
