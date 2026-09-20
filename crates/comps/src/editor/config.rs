//! Editor configuration and color theme.

use engine::text::backend::TextStyle;
use engine::theme::Theme;

/// Visual and behavioral configuration of an [`EditorView`](super::EditorView).
#[derive(Clone, Debug, PartialEq)]
pub struct EditorConfig {
    /// Font size in logical pixels.
    pub font_size: f32,
    /// Height of one text line in logical pixels. Every line has this exact
    /// height — that uniformity is what makes line virtualization O(visible).
    pub line_height: f32,
    /// Font family for code; `None` uses the engine default (Inclusive Sans).
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
    /// Padding on each side of the line numbers in the gutter.
    pub gutter_pad: f32,
    /// Gap between the gutter and the first glyph of a line.
    pub text_pad_x: f32,
    /// Caret width.
    pub cursor_w: f32,
    /// IME preedit underline thickness.
    pub preedit_underline_h: f32,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self::from_theme(&Theme::default())
    }
}

impl EditorConfig {
    /// Configuration from the theme: the mono ramp style sets the font and
    /// the line height, the caret blinks at the platform half-period.
    pub fn from_theme(theme: &Theme) -> Self {
        let style = theme.typography.mono();
        Self {
            font_size: style.font_size,
            line_height: style.line_height,
            font_family: style.font_family.clone(),
            show_gutter: true,
            tab_width: 4,
            cursor_blink_interval: 0.53,
            overscan_lines: 8,
            gutter_pad: theme.spacing.sm + theme.spacing.xs / 2.0,
            text_pad_x: theme.spacing.sm,
            cursor_w: theme.control.focus_ring_width,
            preedit_underline_h: theme.control.edge_width,
        }
    }

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
/// [`SceneNode`](engine::compositor::SceneNode) color fields.
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
        Self::from_theme(&Theme::default())
    }
}

impl EditorTheme {
    /// Editor colors from the theme tokens: the page as the canvas, the
    /// raised surface as the gutter, the accent for caret and selection
    /// wash, the placeholder tone for line numbers.
    pub fn from_theme(theme: &Theme) -> Self {
        let accent = theme.colors.accent.0;
        Self {
            background: theme.colors.bg.0,
            text: theme.colors.text.0,
            gutter_background: theme.colors.surface.0,
            gutter_text: theme.glass.text_placeholder.0,
            gutter_separator: theme.colors.divider.0,
            selection: [
                accent[0],
                accent[1],
                accent[2],
                theme.glass.wash_hover_alpha,
            ],
            cursor: accent,
            preedit_underline: theme.glass.text_active.0,
        }
    }
}
