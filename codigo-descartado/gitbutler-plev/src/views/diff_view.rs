use plev::compositor::{Compositor, SceneNode, TextNodeKey};
use plev::scroll::ScrollState;
use crate::theme::Theme;

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
    pub fn new() -> Self {
        Self {
            filename: "src/compositor.rs".into(),
            lines: mock_diff(),
            scroll: ScrollState::new(),
        }
    }

    /// Clear the diff view (no file selected).
    pub fn clear(&mut self) {
        self.filename = String::new();
        self.lines = Vec::new();
        self.scroll = ScrollState::new();
    }

    /// Update the diff view to show a mock diff for the given file.
    pub fn set_file(&mut self, path: &str, status: super::unassigned_view::FileStatus) {
        self.filename = path.to_string();
        self.lines = generate_diff_for_file(path, status);
        self.scroll = ScrollState::new();
    }

    /// Update the diff view to show a mock diff for a commit.
    pub fn set_commit(&mut self, message: &str, sha: &str) {
        self.filename = format!("{} ({})", message.get(..40).unwrap_or(message), sha.get(..7).unwrap_or(sha));
        self.lines = generate_commit_diff(message);
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
            x, y, w, h,
            color: theme.bg_1.to_array(),
        });

        // Header
        compositor.push(SceneNode::Rect {
            x, y, w, h: HEADER_H,
            color: theme.bg_2.to_array(),
        });
        compositor.push(SceneNode::Text {
            key: TextNodeKey::new(&self.filename, 12.0, 16.0, Some(w - PAD_X * 2.0)).with_weight(500),
            x: x + PAD_X,
            y: y + 14.0,
            color: theme.text_1.to_array(),
        });
        compositor.push(SceneNode::Rect {
            x, y: y + HEADER_H, w, h: 1.0,
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
                x, y: ly, w, h: LINE_H,
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
                DiffLineKind::Added   => "+",
                DiffLineKind::Removed => "-",
                DiffLineKind::Header  => "@",
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
                key: TextNodeKey::new(&line.content, FONT_SIZE, LINE_H, Some(max_code_w)).with_weight(400),
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
            ([c.0[0] * 0.08, c.0[1] * 0.08, c.0[2] * 0.08, 1.0], c.to_array())
        }
        DiffLineKind::Removed => {
            let c = theme.danger;
            ([c.0[0] * 0.08, c.0[1] * 0.03, c.0[2] * 0.03, 1.0], c.to_array())
        }
        DiffLineKind::Header => {
            ([theme.bg_3.0[0], theme.bg_3.0[1], theme.bg_3.0[2], 1.0], theme.pop.to_array())
        }
        DiffLineKind::Context => {
            (theme.bg_1.to_array(), theme.text_2.to_array())
        }
    }
}

fn generate_diff_for_file(path: &str, status: super::unassigned_view::FileStatus) -> Vec<DiffLine> {
    use super::unassigned_view::FileStatus;
    let ext = path.rsplit('.').next().unwrap_or("");
    match status {
        FileStatus::Added => {
            let mut lines = vec![
                DiffLine { kind: DiffLineKind::Header, line_no_old: None, line_no_new: None,
                    content: format!("@@ -0,0 +1,5 @@ new file: {}", path) },
            ];
            for i in 1..=5 {
                lines.push(DiffLine { kind: DiffLineKind::Added, line_no_old: None, line_no_new: Some(i),
                    content: format!("// new {} content line {}", ext, i) });
            }
            lines
        }
        FileStatus::Deleted => {
            let mut lines = vec![
                DiffLine { kind: DiffLineKind::Header, line_no_old: None, line_no_new: None,
                    content: format!("@@ -1,5 +0,0 @@ deleted file: {}", path) },
            ];
            for i in 1..=5 {
                lines.push(DiffLine { kind: DiffLineKind::Removed, line_no_old: Some(i), line_no_new: None,
                    content: format!("// removed {} line {}", ext, i) });
            }
            lines
        }
        _ => {
            // Modified / Renamed / Untracked — show a mix
            vec![
                DiffLine { kind: DiffLineKind::Header, line_no_old: None, line_no_new: None,
                    content: format!("@@ -10,7 +10,9 @@ changes in {}", path) },
                DiffLine { kind: DiffLineKind::Context, line_no_old: Some(10), line_no_new: Some(10),
                    content: format!("use crate::{};", ext) },
                DiffLine { kind: DiffLineKind::Context, line_no_old: Some(11), line_no_new: Some(11),
                    content: String::new() },
                DiffLine { kind: DiffLineKind::Removed, line_no_old: Some(12), line_no_new: None,
                    content: "    let old_value = compute();".into() },
                DiffLine { kind: DiffLineKind::Added, line_no_old: None, line_no_new: Some(12),
                    content: "    let new_value = compute_v2();".into() },
                DiffLine { kind: DiffLineKind::Added, line_no_old: None, line_no_new: Some(13),
                    content: "    log::debug!(\"updated\");".into() },
                DiffLine { kind: DiffLineKind::Context, line_no_old: Some(13), line_no_new: Some(14),
                    content: "}".into() },
                DiffLine { kind: DiffLineKind::Context, line_no_old: Some(14), line_no_new: Some(15),
                    content: String::new() },
                DiffLine { kind: DiffLineKind::Context, line_no_old: Some(15), line_no_new: Some(16),
                    content: format!("fn main() {{ /* {} */ }}", path) },
            ]
        }
    }
}

fn generate_commit_diff(message: &str) -> Vec<DiffLine> {
    vec![
        DiffLine { kind: DiffLineKind::Header, line_no_old: None, line_no_new: None,
            content: format!("@@ commit: {}", message.get(..60).unwrap_or(message)) },
        DiffLine { kind: DiffLineKind::Context, line_no_old: Some(1), line_no_new: Some(1),
            content: "// context".into() },
        DiffLine { kind: DiffLineKind::Removed, line_no_old: Some(2), line_no_new: None,
            content: "    old_implementation();".into() },
        DiffLine { kind: DiffLineKind::Added, line_no_old: None, line_no_new: Some(2),
            content: "    new_implementation();".into() },
        DiffLine { kind: DiffLineKind::Added, line_no_old: None, line_no_new: Some(3),
            content: "    additional_feature();".into() },
        DiffLine { kind: DiffLineKind::Context, line_no_old: Some(3), line_no_new: Some(4),
            content: "}".into() },
    ]
}

fn mock_diff() -> Vec<DiffLine> {
    vec![
        DiffLine { kind: DiffLineKind::Header, line_no_old: None, line_no_new: None, content: "@@ -62,6 +62,19 @@ pub struct TextNodeKey {".into() },
        DiffLine { kind: DiffLineKind::Context, line_no_old: Some(62), line_no_new: Some(62), content: "    pub text: String,".into() },
        DiffLine { kind: DiffLineKind::Context, line_no_old: Some(63), line_no_new: Some(63), content: "    pub font_size_bits: u32,".into() },
        DiffLine { kind: DiffLineKind::Context, line_no_old: Some(64), line_no_new: Some(64), content: "    pub line_height_bits: u32,".into() },
        DiffLine { kind: DiffLineKind::Context, line_no_old: Some(65), line_no_new: Some(65), content: "    pub max_width_bits: Option<u32>,".into() },
        DiffLine { kind: DiffLineKind::Added, line_no_old: None, line_no_new: Some(66), content: "    /// Font weight: 400 = normal, 700 = bold.".into() },
        DiffLine { kind: DiffLineKind::Added, line_no_old: None, line_no_new: Some(67), content: "    pub font_weight: u16,".into() },
        DiffLine { kind: DiffLineKind::Context, line_no_old: Some(66), line_no_new: Some(68), content: "}".into() },
        DiffLine { kind: DiffLineKind::Context, line_no_old: Some(67), line_no_new: Some(69), content: "".into() },
        DiffLine { kind: DiffLineKind::Context, line_no_old: Some(68), line_no_new: Some(70), content: "impl TextNodeKey {".into() },
        DiffLine { kind: DiffLineKind::Context, line_no_old: Some(69), line_no_new: Some(71), content: "    pub fn new(text: &str, font_size: f32, line_height: f32, max_width: Option<f32>) -> Self {".into() },
        DiffLine { kind: DiffLineKind::Context, line_no_old: Some(70), line_no_new: Some(72), content: "        Self {".into() },
        DiffLine { kind: DiffLineKind::Context, line_no_old: Some(71), line_no_new: Some(73), content: "            text: text.to_string(),".into() },
        DiffLine { kind: DiffLineKind::Context, line_no_old: Some(72), line_no_new: Some(74), content: "            font_size_bits: font_size.to_bits(),".into() },
        DiffLine { kind: DiffLineKind::Context, line_no_old: Some(73), line_no_new: Some(75), content: "            line_height_bits: line_height.to_bits(),".into() },
        DiffLine { kind: DiffLineKind::Context, line_no_old: Some(74), line_no_new: Some(76), content: "            max_width_bits: max_width.map(|w| w.to_bits()),".into() },
        DiffLine { kind: DiffLineKind::Added, line_no_old: None, line_no_new: Some(77), content: "            font_weight: 400,".into() },
        DiffLine { kind: DiffLineKind::Context, line_no_old: Some(75), line_no_new: Some(78), content: "        }".into() },
        DiffLine { kind: DiffLineKind::Context, line_no_old: Some(76), line_no_new: Some(79), content: "    }".into() },
        DiffLine { kind: DiffLineKind::Added, line_no_old: None, line_no_new: Some(80), content: "".into() },
        DiffLine { kind: DiffLineKind::Added, line_no_old: None, line_no_new: Some(81), content: "    pub fn with_weight(mut self, weight: u16) -> Self {".into() },
        DiffLine { kind: DiffLineKind::Added, line_no_old: None, line_no_new: Some(82), content: "        self.font_weight = weight;".into() },
        DiffLine { kind: DiffLineKind::Added, line_no_old: None, line_no_new: Some(83), content: "        self".into() },
        DiffLine { kind: DiffLineKind::Added, line_no_old: None, line_no_new: Some(84), content: "    }".into() },
        DiffLine { kind: DiffLineKind::Context, line_no_old: Some(77), line_no_new: Some(85), content: "}".into() },
        DiffLine { kind: DiffLineKind::Header, line_no_old: None, line_no_new: None, content: "@@ -243,6 +257,9 @@ buffer.set_text(font_system, &key.text, &attrs, ...)".into() },
        DiffLine { kind: DiffLineKind::Removed, line_no_old: Some(243), line_no_new: None, content: "                    &Attrs::new(),".into() },
        DiffLine { kind: DiffLineKind::Added, line_no_old: None, line_no_new: Some(257), content: "                    &Attrs::new().weight(Weight(key.font_weight)),".into() },
    ]
}
