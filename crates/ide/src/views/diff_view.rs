//! Right "Diff" column — HOFF RightSidebar surface (rgba(40,40,40,.8)),
//! 68px head with the file name in base-2m at rgba($n2,.76). Diff rows keep
//! the green/red convention but harmonized with the HOFF accents: content
//! text stays monochrome (.56/.70); only row tints and +/- prefixes carry
//! #55F08B / #BD3027. Line numbers at the .25 placeholder alpha.

use crate::components::hoff;
use crate::theme::Theme;
use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::input::scroll::ScrollState;

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

const HEADER_H: f32 = 68.0;
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

        // Column surface — RightSidebar rgba(40,40,40,.8).
        compositor.push(SceneNode::Rect {
            x,
            y,
            w,
            h,
            color: theme.bg_sidebar.to_array(),
        });

        // Head — file name in base-2m (14/500) at .76.
        compositor.push(SceneNode::Text {
            key: TextNodeKey::new(&self.filename, 14.0, 14.0 * 1.4, Some(w - PAD_X * 2.0))
                .with_weight(500),
            x: x + PAD_X,
            y: y + (HEADER_H - 14.0 * 1.4) / 2.0,
            color: theme.text_active.to_array(),
        });

        let code_y = y + HEADER_H;
        let scroll_offset = self.scroll.offset();

        // Scrolled code clips to the area below the panel head.
        compositor.push(SceneNode::PushClip {
            x,
            y: code_y,
            w,
            h: h - HEADER_H,
        });
        for (i, line) in self.lines.iter().enumerate() {
            let ly = code_y + i as f32 * LINE_H - scroll_offset;
            if ly + LINE_H < code_y || ly > y + h {
                continue;
            }

            let (bg, prefix_col, text_col) = line_colors(line.kind, theme);
            if bg[3] > 0.0 {
                compositor.push(SceneNode::Rect {
                    x,
                    y: ly,
                    w,
                    h: LINE_H,
                    color: bg,
                });
            }

            // Old line number — placeholder alpha (.25).
            if let Some(no) = line.line_no_old {
                let no_str = no.to_string();
                compositor.push(SceneNode::Text {
                    key: TextNodeKey::new(&no_str, FONT_SIZE - 1.0, LINE_H, None).with_weight(400),
                    x: x + 4.0,
                    y: ly + (LINE_H - (FONT_SIZE - 1.0) * 1.2) / 2.0,
                    color: theme.text_placeholder.to_array(),
                });
            }
            // New line number
            if let Some(no) = line.line_no_new {
                let no_str = no.to_string();
                compositor.push(SceneNode::Text {
                    key: TextNodeKey::new(&no_str, FONT_SIZE - 1.0, LINE_H, None).with_weight(400),
                    x: x + LINE_NO_W + 4.0,
                    y: ly + (LINE_H - (FONT_SIZE - 1.0) * 1.2) / 2.0,
                    color: theme.text_placeholder.to_array(),
                });
            }

            // Diff prefix (+/-/ /@@) — carries the accent.
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
                color: prefix_col,
            });

            // Code content — monochrome.
            let max_code_w = w - CODE_X_OFFSET - PAD_X;
            compositor.push(SceneNode::Text {
                key: TextNodeKey::new(&line.content, FONT_SIZE, LINE_H, Some(max_code_w))
                    .with_weight(400),
                x: x + CODE_X_OFFSET + 10.0,
                y: ly + (LINE_H - FONT_SIZE * 1.2) / 2.0,
                color: text_col,
            });
        }
        compositor.push(SceneNode::PopClip);

        // Scrollbar
        if self.scroll.is_scrollable() {
            hoff::draw_scrollbar(
                compositor,
                theme,
                x + w - 4.0,
                code_y,
                h - HEADER_H,
                &self.scroll,
            );
        }
    }
}

/// (row bg, prefix color, content color) per diff line kind.
fn line_colors(kind: DiffLineKind, theme: &Theme) -> ([f32; 4], [f32; 4], [f32; 4]) {
    match kind {
        DiffLineKind::Added => {
            let g = theme.accent_green.to_array();
            ([g[0], g[1], g[2], 0.07], g, theme.text_secondary.to_array())
        }
        DiffLineKind::Removed => {
            let r = theme.accent_red.to_array();
            ([r[0], r[1], r[2], 0.12], r, theme.text_secondary.to_array())
        }
        DiffLineKind::Header => (
            theme.surface.to_array(),
            theme.text_muted.to_array(),
            theme.text_muted.to_array(),
        ),
        DiffLineKind::Context => (
            [0.0; 4],
            theme.text_default.to_array(),
            theme.text_default.to_array(),
        ),
    }
}
