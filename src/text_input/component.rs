//! TextInput: editable text field component with focus, blinking cursor, and scene generation.

use crate::compositor::{SceneNode, TextNodeKey};

use super::buffer::TextBuffer;
use super::cursor_map::{cursor_to_x, x_to_cursor};

const CURSOR_BLINK_INTERVAL: f32 = 0.53;

pub struct TextInput {
    pub buffer: TextBuffer,
    pub focused: bool,
    pub placeholder: String,
    pub font_size: f32,
    pub text_color: [f32; 4],
    pub bg_color: [f32; 4],
    pub placeholder_color: [f32; 4],
    pub cursor_color: [f32; 4],
    pub selection_color: [f32; 4],
    pub(crate) cursor_visible: bool,
    cursor_blink_timer: f32,
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            buffer: TextBuffer::new(),
            focused: false,
            placeholder: String::new(),
            font_size: 16.0,
            text_color: [0.93, 0.93, 0.96, 1.0],
            bg_color: [0.12, 0.12, 0.20, 1.0],
            placeholder_color: [0.45, 0.45, 0.55, 0.8],
            cursor_color: [0.30, 0.55, 1.0, 1.0],
            selection_color: [0.30, 0.55, 1.0, 0.3],
            cursor_visible: true,
            cursor_blink_timer: 0.0,
        }
    }

    pub fn with_placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = placeholder.to_string();
        self
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn with_text_color(mut self, color: [f32; 4]) -> Self {
        self.text_color = color;
        self
    }

    pub fn with_bg_color(mut self, color: [f32; 4]) -> Self {
        self.bg_color = color;
        self
    }

    pub fn focus(&mut self) {
        self.focused = true;
        self.cursor_visible = true;
        self.cursor_blink_timer = 0.0;
    }

    pub fn unfocus(&mut self) {
        self.focused = false;
    }

    pub fn tick(&mut self, dt: f32) {
        if !self.focused {
            return;
        }
        self.cursor_blink_timer += dt;
        if self.cursor_blink_timer >= CURSOR_BLINK_INTERVAL {
            self.cursor_blink_timer -= CURSOR_BLINK_INTERVAL;
            self.cursor_visible = !self.cursor_visible;
        }
    }

    pub fn reset_blink(&mut self) {
        self.cursor_visible = true;
        self.cursor_blink_timer = 0.0;
    }

    pub fn handle_char(&mut self, c: char) {
        if !self.focused {
            return;
        }
        self.buffer.insert_char(c);
        self.reset_blink();
    }

    pub fn handle_backspace(&mut self) {
        if !self.focused {
            return;
        }
        self.buffer.delete_back();
        self.reset_blink();
    }

    pub fn handle_delete(&mut self) {
        if !self.focused {
            return;
        }
        self.buffer.delete_forward();
        self.reset_blink();
    }

    pub fn handle_left(&mut self) {
        if !self.focused {
            return;
        }
        self.buffer.move_left();
        self.reset_blink();
    }

    pub fn handle_right(&mut self) {
        if !self.focused {
            return;
        }
        self.buffer.move_right();
        self.reset_blink();
    }

    pub fn handle_home(&mut self) {
        if !self.focused {
            return;
        }
        self.buffer.move_home();
        self.reset_blink();
    }

    pub fn handle_end(&mut self) {
        if !self.focused {
            return;
        }
        self.buffer.move_end();
        self.reset_blink();
    }

    pub fn handle_select_all(&mut self) {
        if !self.focused {
            return;
        }
        self.buffer.select_all();
        self.reset_blink();
    }

    pub fn handle_ime(&mut self, committed: &str, preedit: &str) {
        if !self.focused {
            return;
        }
        if !committed.is_empty() {
            self.buffer.insert_str(committed);
            self.reset_blink();
        }
        // preedit stored for future inline rendering
        let _ = preedit;
    }

    pub fn handle_click(&mut self, local_x: f32) {
        self.focus();
        let cursor_pos = x_to_cursor(self.buffer.text(), local_x, self.font_size);
        self.buffer.cursor = cursor_pos;
        self.buffer.selection = None;
        self.reset_blink();
    }

    /// Generate SceneNodes for this text input at position (x, y) with width w.
    pub fn build_scene(&self, x: f32, y: f32, w: f32) -> Vec<SceneNode> {
        let mut nodes = Vec::new();
        let h = self.font_size * 2.0;
        let pad = 8.0;

        // Background
        nodes.push(SceneNode::Rect {
            x,
            y,
            w,
            h,
            color: self.bg_color,
        });

        // Focus border
        if self.focused {
            // Top
            nodes.push(SceneNode::Rect {
                x,
                y,
                w,
                h: 1.0,
                color: self.cursor_color,
            });
            // Bottom
            nodes.push(SceneNode::Rect {
                x,
                y: y + h - 1.0,
                w,
                h: 1.0,
                color: self.cursor_color,
            });
            // Left
            nodes.push(SceneNode::Rect {
                x,
                y,
                w: 1.0,
                h,
                color: self.cursor_color,
            });
            // Right
            nodes.push(SceneNode::Rect {
                x: x + w - 1.0,
                y,
                w: 1.0,
                h,
                color: self.cursor_color,
            });
        }

        let text_y = y + (h - self.font_size) / 2.0;

        // Selection highlight
        if self.focused
            && let Some((start, end)) = self.buffer.selection()
        {
            let (lo, hi) = if start <= end {
                (start, end)
            } else {
                (end, start)
            };
            let sel_x = cursor_to_x(self.buffer.text(), lo, self.font_size);
            let sel_w = cursor_to_x(self.buffer.text(), hi, self.font_size) - sel_x;
            nodes.push(SceneNode::Rect {
                x: x + pad + sel_x,
                y: text_y - 2.0,
                w: sel_w,
                h: self.font_size + 4.0,
                color: self.selection_color,
            });
        }

        // Text or placeholder
        if self.buffer.is_empty() && !self.focused {
            if !self.placeholder.is_empty() {
                nodes.push(SceneNode::Text {
                    key: TextNodeKey::new(
                        &self.placeholder,
                        self.font_size,
                        self.font_size * 1.3,
                        Some(w - pad * 2.0),
                    ),
                    x: x + pad,
                    y: text_y,
                    color: self.placeholder_color,
                });
            }
        } else {
            let display_text = if self.buffer.is_empty() {
                &self.placeholder
            } else {
                self.buffer.text()
            };
            if !display_text.is_empty() {
                let color = if self.buffer.is_empty() {
                    self.placeholder_color
                } else {
                    self.text_color
                };
                nodes.push(SceneNode::Text {
                    key: TextNodeKey::new(
                        display_text,
                        self.font_size,
                        self.font_size * 1.3,
                        Some(w - pad * 2.0),
                    ),
                    x: x + pad,
                    y: text_y,
                    color,
                });
            }
        }

        // Cursor
        if self.focused && self.cursor_visible {
            let cursor_x = cursor_to_x(self.buffer.text(), self.buffer.cursor(), self.font_size);
            nodes.push(SceneNode::Rect {
                x: x + pad + cursor_x,
                y: text_y - 2.0,
                w: 2.0,
                h: self.font_size + 4.0,
                color: self.cursor_color,
            });
        }

        nodes
    }
}
