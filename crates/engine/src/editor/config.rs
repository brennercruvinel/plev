//! Editor configuration and color theme.

use crate::text::backend::TextStyle;

/// Visual and behavioral configuration of an [`EditorView`](super::EditorView).
#[derive(Clone, Debug, PartialEq)]
pub struct EditorConfig {
    /// Font size in logical pixels.
    pub font_size: f32,
    /// Height of one text line in logical pixels. Every line has this exact
    /// height — that uniformity is what makes line virtualization O(visible).
    pub line_height: f32,
    /// Font family for code; `None` uses the engine default (Inter).
    pub font_family: Option<String>,
    /// Whether to draw the line-number gutter.
    pub show_gutter: bool,
    /// Number of spaces inserted by Tab.
    pub tab_width: usize,
    /// Seconds between primary-cursor blink toggles.
    pub cursor_blink_interval: f32,
    /// Extra lines shaped above/below the viewport so small scrolls do not
    /// pop blank lines in.
    pub overscan_lines: usize,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            line_height: 21.0,
            font_family: Some("JetBrains Mono".to_string()),
            show_gutter: true,
            tab_width: 4,
            cursor_blink_interval: 0.53,
            overscan_lines: 8,
        }
    }
}

impl EditorConfig {
    /// The [`TextStyle`] used for both shaping (render) and measuring
    /// (hit-test/caret), so the two always agree.
    pub fn text_style(&self) -> TextStyle {
        let style = TextStyle::new(self.font_size).with_line_height(self.line_height);
        match self.font_family {
            Some(ref family) => style.with_family(family),
            None => style,
        }
    }

    /// The string Tab inserts.
    pub fn tab_text(&self) -> String {
        " ".repeat(self.tab_width.max(1))
    }
}

/// Colors used by [`EditorView::render`](super::EditorView::render).
/// All colors are premultiplied-friendly linear RGBA arrays, matching
/// [`SceneNode`](crate::compositor::SceneNode) color fields.
#[derive(Clone, Debug, PartialEq)]
pub struct EditorTheme {
    pub background: [f32; 4],
    pub text: [f32; 4],
    pub gutter_background: [f32; 4],
    pub gutter_text: [f32; 4],
    pub gutter_separator: [f32; 4],
    /// Translucent fill drawn behind selected text, one rect per line.
    pub selection: [f32; 4],
    pub cursor: [f32; 4],
    /// Thin rect drawn under IME preedit text.
    pub preedit_underline: [f32; 4],
}

impl Default for EditorTheme {
    fn default() -> Self {
        Self {
            background: [0.071, 0.075, 0.094, 1.0],
            text: [0.871, 0.882, 0.914, 1.0],
            gutter_background: [0.063, 0.067, 0.082, 1.0],
            gutter_text: [0.376, 0.396, 0.467, 1.0],
            gutter_separator: [0.157, 0.165, 0.200, 1.0],
            selection: [0.263, 0.443, 0.812, 0.30],
            cursor: [0.388, 0.612, 1.0, 1.0],
            preedit_underline: [0.871, 0.882, 0.914, 0.85],
        }
    }
}
