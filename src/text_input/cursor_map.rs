//! Cursor-pixel mapping (approximate, no font system dependency).

/// Approximate cursor byte offset to x position.
/// Uses char count up to cursor * average char width.
pub fn cursor_to_x(text: &str, cursor_byte: usize, font_size: f32) -> f32 {
    let char_width = font_size * 0.6;
    let chars_before = text[..cursor_byte.min(text.len())].chars().count();
    chars_before as f32 * char_width
}

/// Approximate x position to cursor byte offset.
pub fn x_to_cursor(text: &str, x: f32, font_size: f32) -> usize {
    let char_width = font_size * 0.6;
    if char_width <= 0.0 {
        return 0;
    }
    let target_chars = (x / char_width).round().max(0.0) as usize;
    let mut byte_pos = 0;
    for (char_count, c) in text.chars().enumerate() {
        if char_count >= target_chars {
            break;
        }
        byte_pos += c.len_utf8();
    }
    byte_pos
}
