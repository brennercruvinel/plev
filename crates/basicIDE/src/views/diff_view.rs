use crate::theme::Theme;
use plev::compositor::{Compositor, SceneNode, TextNodeKey};
use plev::scroll::ScrollState;

/// A single line in a diff.
#[derive(Clone, Debug)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub line_no_old: Option<u32>,
    pub line_no_new: Option<u32>,
    pub content: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DiffLineKind {
    Context,
    Added,
    Removed,
    Header,
}

/// State for the right "Diff" panel.
pub struct DiffView {
    pub filename: String,
    pub lines: Vec<DiffLine>,
    pub scroll: ScrollState,
}

const HEADER_H: f32 = 44.0;
const LINE_H: f32 = 20.0;
const LINE_NO_W: f32 = 40.0;
const CODE_X_OFFSET: f32 = LINE_NO_W * 2.0 + 8.0;
const FONT_SIZE: f32 = 12.0;
const PAD_X: f32 = 12.0;

impl DiffView {
    /// Starts empty; the app injects real hunks via [`set_lines`](Self::set_lines).
    pub fn new() -> Self {
        Self {
            filename: String::new(),
            lines: Vec::new(),
            scroll: ScrollState::new(),
        }
    }

    /// Clear the diff view (no file selected).
    pub fn clear(&mut self) {
        self.filename = String::new();
        self.lines = Vec::new();
        self.scroll = ScrollState::new();
    }

    /// Point the view at a file whose diff is being fetched. The header
    /// updates immediately; lines arrive later via [`set_lines`](Self::set_lines).
    pub fn show_file(&mut self, path: &str) {
        self.filename = path.to_string();
        self.lines = Vec::new();
        self.scroll = ScrollState::new();
    }

    /// Point the view at a commit whose diff is being fetched.
    pub fn show_commit(&mut self, message: &str, sha: &str) {
        self.filename = format!(
            "{} ({})",
            message.get(..40).unwrap_or(message),
            sha.get(..7).unwrap_or(sha)
        );
        self.lines = Vec::new();
        self.scroll = ScrollState::new();
    }

    /// Replaces the diff content (adapter output of real hunks).
    pub fn set_lines(&mut self, lines: Vec<DiffLine>) {
        self.lines = lines;
        self.scroll = ScrollState::new();
    }

    pub fn render(
        &mut self,
        compositor: &mut Compositor,
        theme: &Theme,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        let content_h = self.lines.len() as f32 * LINE_H;
        self.scroll.set_viewport(h - HEADER_H);
        self.scroll.set_content(content_h);

        // Panel background
        compositor.push(SceneNode::Rect {
            x,
            y,
            w,
            h,
            color: theme.bg_1.to_array(),
        });

        // Header
        compositor.push(SceneNode::Rect {
            x,
            y,
            w,
            h: HEADER_H,
            color: theme.bg_2.to_array(),
        });
        compositor.push(SceneNode::Text {
            key: TextNodeKey::new(&self.filename, 12.0, 16.0, Some(w - PAD_X * 2.0))
                .with_weight(500),
            x: x + PAD_X,
            y: y + 14.0,
            color: theme.text_1.to_array(),
        });
        compositor.push(SceneNode::Rect {
            x,
            y: y + HEADER_H,
            w,
            h: 1.0,
            color: theme.border.to_array(),
        });

        let code_y = y + HEADER_H + 1.0;
        let scroll_offset = self.scroll.offset();

        for (i, line) in self.lines.iter().enumerate() {
            let ly = code_y + i as f32 * LINE_H - scroll_offset;
            if ly + LINE_H < code_y || ly > y + h {
                continue;
            }

            let (bg, text_col) = line_colors(line.kind, theme);
            compositor.push(SceneNode::Rect {
                x,
                y: ly,
                w,
                h: LINE_H,
                color: bg,
            });

            // Old line number
            if let Some(no) = line.line_no_old {
                let no_str = no.to_string();
                compositor.push(SceneNode::Text {
                    key: TextNodeKey::new(&no_str, FONT_SIZE - 1.0, LINE_H, None).with_weight(400),
                    x: x + 4.0,
                    y: ly + (LINE_H - (FONT_SIZE - 1.0) * 1.2) / 2.0,
                    color: theme.text_3.to_array(),
                });
            }
            // New line number
            if let Some(no) = line.line_no_new {
                let no_str = no.to_string();
                compositor.push(SceneNode::Text {
                    key: TextNodeKey::new(&no_str, FONT_SIZE - 1.0, LINE_H, None).with_weight(400),
                    x: x + LINE_NO_W + 4.0,
                    y: ly + (LINE_H - (FONT_SIZE - 1.0) * 1.2) / 2.0,
                    color: theme.text_3.to_array(),
                });
            }

            // Diff prefix (+/-/ /@@)
            let prefix = match line.kind {
                DiffLineKind::Added => "+",
                DiffLineKind::Removed => "-",
                DiffLineKind::Header => "@",
                DiffLineKind::Context => " ",
            };
            compositor.push(SceneNode::Text {
                key: TextNodeKey::new(prefix, FONT_SIZE, LINE_H, None).with_weight(600),
                x: x + LINE_NO_W * 2.0,
                y: ly + (LINE_H - FONT_SIZE * 1.2) / 2.0,
                color: text_col,
            });

            // Code content
            let max_code_w = w - CODE_X_OFFSET - PAD_X;
            compositor.push(SceneNode::Text {
                key: TextNodeKey::new(&line.content, FONT_SIZE, LINE_H, Some(max_code_w))
                    .with_weight(400),
                x: x + CODE_X_OFFSET + 10.0,
                y: ly + (LINE_H - FONT_SIZE * 1.2) / 2.0,
                color: text_col,
            });
        }

        // Scrollbar
        if self.scroll.is_scrollable() {
            let thumb_h = ((h - HEADER_H) * self.scroll.thumb_ratio()).max(20.0);
            let thumb_y = code_y + ((h - HEADER_H) - thumb_h) * self.scroll.thumb_position();
            compositor.push(SceneNode::Rect {
                x: x + w - 4.0,
                y: thumb_y,
                w: 4.0,
                h: thumb_h,
                color: [theme.text_3.0[0], theme.text_3.0[1], theme.text_3.0[2], 0.5],
            });
        }
    }
}

fn line_colors(kind: DiffLineKind, theme: &Theme) -> ([f32; 4], [f32; 4]) {
    match kind {
        DiffLineKind::Added => {
            let c = theme.safe;
            (
                [c.0[0] * 0.08, c.0[1] * 0.08, c.0[2] * 0.08, 1.0],
                c.to_array(),
            )
        }
        DiffLineKind::Removed => {
            let c = theme.danger;
            (
                [c.0[0] * 0.08, c.0[1] * 0.03, c.0[2] * 0.03, 1.0],
                c.to_array(),
            )
        }
        DiffLineKind::Header => (
            [theme.bg_3.0[0], theme.bg_3.0[1], theme.bg_3.0[2], 1.0],
            theme.pop.to_array(),
        ),
        DiffLineKind::Context => (theme.bg_1.to_array(), theme.text_2.to_array()),
    }
}
