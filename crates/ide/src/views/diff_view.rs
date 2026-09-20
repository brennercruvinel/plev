//! Right "Diff" column: the raised surface with the file name in the
//! panel header, mono-ramp diff rows. Content text stays monochrome
//! (text-default, text-mid); only row tints and the +/- prefixes carry
//! the success / danger colors. Line numbers at the placeholder tone.

use comps::feedback::Scrollbar;
use comps::nav::PanelHeader;
use comps::prelude::Rect;
use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::input::scroll::ScrollState;
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::Theme;

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
    scrollbar: Scrollbar,
}

impl DiffView {
    /// Starts empty; the app injects real hunks via [`set_lines`](Self::set_lines).
    pub fn new() -> Self {
        Self {
            filename: String::new(),
            lines: Vec::new(),
            scroll: ScrollState::new(),
            scrollbar: Scrollbar::new(),
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

    /// Code style: the mono ramp step; every row is one of its line boxes.
    fn code_style(theme: &Theme) -> TextStyle {
        theme.typography.mono()
    }

    pub fn notify_scroll(&mut self) {
        self.scrollbar.notify_scroll();
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        self.scrollbar.tick(dt)
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
        let header = PanelHeader::new(self.filename.clone());
        let header_h = header.height(theme);
        let code = Self::code_style(theme);
        let line_h = code.line_height;
        let number_style = theme.typography.small_r();
        let pad = theme.spacing.md;
        let content_h = self.lines.len() as f32 * line_h;
        self.scroll.set_viewport(h - header_h);
        self.scroll.set_content(content_h);

        compositor.push(SceneNode::Rect {
            x,
            y,
            w,
            h,
            color: theme.colors.surface.0,
        });
        header.render(compositor, Rect::new(x, y, w, header_h), theme);

        // Line-number gutters: wide enough for the largest number drawn.
        let widest = self
            .lines
            .iter()
            .flat_map(|l| [l.line_no_old, l.line_no_new])
            .flatten()
            .max()
            .unwrap_or(0);
        let (num_w, _) = TextMeasurer::measure_styled(&widest.to_string(), &number_style, None);
        let gutter_w = num_w + theme.spacing.sm;
        let prefix_x = x + pad + gutter_w * 2.0;
        let code_x = prefix_x + theme.spacing.md;

        let code_y = y + header_h;
        let list = Rect::new(x, code_y, w, h - header_h);
        let scroll_offset = self.scroll.offset();
        compositor.push(SceneNode::PushClip {
            x: list.x,
            y: list.y,
            w: list.w,
            h: list.h,
        });
        for (i, line) in self.lines.iter().enumerate() {
            let ly = code_y + i as f32 * line_h - scroll_offset;
            if ly + line_h < code_y || ly > y + h {
                continue;
            }

            let (bg, prefix_col, text_col) = line_colors(line.kind, theme);
            if bg[3] > 0.0 {
                compositor.push(SceneNode::Rect {
                    x,
                    y: ly,
                    w,
                    h: line_h,
                    color: bg,
                });
            }
            let num_y = ly + TextMeasurer::vertical_center(&number_style, line_h);
            for (slot, no) in [line.line_no_old, line.line_no_new].into_iter().enumerate() {
                if let Some(no) = no {
                    let s = no.to_string();
                    let (sw, _) = TextMeasurer::measure_styled(&s, &number_style, None);
                    compositor.push(SceneNode::Text {
                        key: TextNodeKey::from_style(&s, &number_style, None),
                        x: x + pad + slot as f32 * gutter_w + num_w - sw,
                        y: num_y,
                        color: theme.glass.text_placeholder.0,
                    });
                }
            }

            let prefix = match line.kind {
                DiffLineKind::Added => "+",
                DiffLineKind::Removed => "-",
                DiffLineKind::Header => "@",
                DiffLineKind::Context => " ",
            };
            compositor.push(SceneNode::Text {
                key: TextNodeKey::from_style(prefix, &code, None),
                x: prefix_x,
                y: ly,
                color: prefix_col,
            });
            let max_code_w = (x + w - pad - code_x).max(0.0);
            let shown = TextMeasurer::truncate_to_width(&line.content, &code, max_code_w);
            compositor.push(SceneNode::Text {
                key: TextNodeKey::from_style(&shown, &code, None),
                x: code_x,
                y: ly,
                color: text_col,
            });
        }
        compositor.push(SceneNode::PopClip);

        self.scrollbar.render(compositor, list, &self.scroll, theme);
    }
}

/// (row bg, prefix color, content color) per diff line kind.
fn line_colors(kind: DiffLineKind, theme: &Theme) -> ([f32; 4], [f32; 4], [f32; 4]) {
    let g = &theme.glass;
    match kind {
        DiffLineKind::Added => {
            let c = theme.colors.success.0;
            (
                [c[0], c[1], c[2], g.surface_hover.0[3]],
                c,
                theme.colors.text_mid.0,
            )
        }
        DiffLineKind::Removed => {
            let c = theme.colors.danger.0;
            (
                [c[0], c[1], c[2], g.surface_active.0[3]],
                c,
                theme.colors.text_mid.0,
            )
        }
        DiffLineKind::Header => (g.surface.0, g.text_faint.0, g.text_faint.0),
        DiffLineKind::Context => ([0.0; 4], g.text_default.0, g.text_default.0),
    }
}
