use crate::theme::Theme;
use plev::compositor::{Compositor, SceneNode, TextNodeKey};
use plev::scroll::ScrollState;

/// File change status.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FileStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
}

impl FileStatus {
    fn label(self) -> &'static str {
        match self {
            FileStatus::Modified => "M",
            FileStatus::Added => "A",
            FileStatus::Deleted => "D",
            FileStatus::Renamed => "R",
            FileStatus::Untracked => "?",
        }
    }
    fn color(self, theme: &Theme) -> [f32; 4] {
        match self {
            FileStatus::Modified => theme.warn.to_array(),
            FileStatus::Added => theme.safe.to_array(),
            FileStatus::Deleted => theme.danger.to_array(),
            FileStatus::Renamed => theme.pop.to_array(),
            FileStatus::Untracked => theme.text_3.to_array(),
        }
    }
}

/// A file entry in the uncommitted changes list.
#[derive(Clone, Debug)]
pub struct FileEntry {
    pub path: String,
    pub status: FileStatus,
}

/// State for the left "Unassigned Changes" panel.
pub struct UnassignedView {
    pub files: Vec<FileEntry>,
    pub selected_idx: Option<usize>,
    pub scroll: ScrollState,
    /// Cached hit rects from last render (x, y, w, h) per file row.
    hit_rects: Vec<(f32, f32, f32, f32)>,
}

const ITEM_H: f32 = 36.0;
const HEADER_H: f32 = 44.0;
const STATUS_W: f32 = 18.0;
const PAD_X: f32 = 12.0;
const FONT_SIZE: f32 = 13.0;

impl UnassignedView {
    pub fn new() -> Self {
        Self {
            files: mock_files(),
            selected_idx: None,
            scroll: ScrollState::new(),
            hit_rects: Vec::new(),
        }
    }

    pub fn hit_rects(&self) -> &[(f32, f32, f32, f32)] {
        &self.hit_rects
    }

    pub fn hit_test(&self, cx: f32, cy: f32) -> Option<usize> {
        self.hit_rects
            .iter()
            .position(|(rx, ry, rw, rh)| cx >= *rx && cx <= rx + rw && cy >= *ry && cy <= ry + rh)
    }

    /// Select file by index. Returns true if selection changed.
    pub fn select(&mut self, idx: Option<usize>) -> bool {
        if self.selected_idx == idx {
            return false;
        }
        self.selected_idx = idx;
        true
    }

    /// Move selection up. Returns true if changed.
    pub fn select_prev(&mut self) -> bool {
        let new = match self.selected_idx {
            Some(0) | None => Some(0),
            Some(i) => Some(i - 1),
        };
        self.select(new)
    }

    /// Move selection down. Returns true if changed.
    pub fn select_next(&mut self) -> bool {
        let max = self.files.len().saturating_sub(1);
        let new = match self.selected_idx {
            None => Some(0),
            Some(i) => Some((i + 1).min(max)),
        };
        self.select(new)
    }

    /// Get the currently selected file entry, if any.
    pub fn selected_file(&self) -> Option<&FileEntry> {
        self.selected_idx.and_then(|i| self.files.get(i))
    }

    /// Build and render into a compositor layer.
    /// Returns a list of (x, y, w, h) hit rects for each file row (for click detection).
    pub fn render(
        &mut self,
        compositor: &mut Compositor,
        theme: &Theme,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        hover_idx: Option<usize>,
    ) -> Vec<(f32, f32, f32, f32)> {
        let content_h = self.files.len() as f32 * ITEM_H;
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
            key: TextNodeKey::new("Changes", 12.0, 16.0, None).with_weight(600),
            x: x + PAD_X,
            y: y + 14.0,
            color: theme.text_2.to_array(),
        });
        // File count badge (pill)
        let count_str = self.files.len().to_string();
        let badge_w = count_str.len() as f32 * 7.0 + 12.0;
        let badge_h = 18.0;
        let badge_x = x + w - PAD_X - badge_w;
        let badge_y = y + (HEADER_H - badge_h) / 2.0;
        compositor.push(SceneNode::RoundedRect {
            x: badge_x,
            y: badge_y,
            w: badge_w,
            h: badge_h,
            color: theme.bg_3.to_array(),
            corner_radius: badge_h / 2.0,
            border_width: 0.0,
            border_color: [0.0; 4],
        });
        compositor.push(SceneNode::Text {
            key: TextNodeKey::new(&count_str, 11.0, 14.0, None).with_weight(600),
            x: badge_x + 6.0,
            y: badge_y + 2.0,
            color: theme.text_2.to_array(),
        });

        // Divider below header
        compositor.push(SceneNode::Rect {
            x,
            y: y + HEADER_H,
            w,
            h: 1.0,
            color: theme.border.to_array(),
        });

        // File list
        let list_y = y + HEADER_H + 1.0;
        let scroll_offset = self.scroll.offset();
        let mut hit_rects = Vec::with_capacity(self.files.len());

        for (i, file) in self.files.iter().enumerate() {
            let item_y = list_y + i as f32 * ITEM_H - scroll_offset;
            // Skip items outside the visible area
            if item_y + ITEM_H < list_y || item_y > y + h {
                hit_rects.push((x, item_y, w, ITEM_H));
                continue;
            }

            let is_selected = self.selected_idx == Some(i);
            let is_hovered = hover_idx == Some(i);

            let row_bg = if is_selected {
                theme.bg_3
            } else if is_hovered {
                theme.hover_bg_1
            } else {
                theme.bg_1
            };

            compositor.push(SceneNode::Rect {
                x,
                y: item_y,
                w,
                h: ITEM_H,
                color: row_bg.to_array(),
            });

            // Status badge (single letter)
            compositor.push(SceneNode::Text {
                key: TextNodeKey::new(file.status.label(), FONT_SIZE, FONT_SIZE * 1.3, None)
                    .with_weight(700),
                x: x + PAD_X,
                y: item_y + (ITEM_H - FONT_SIZE * 1.3) / 2.0,
                color: file.status.color(theme),
            });

            // Filename (truncated)
            let display_name = truncate_path(&file.path, 32);
            compositor.push(SceneNode::Text {
                key: TextNodeKey::new(
                    &display_name,
                    FONT_SIZE,
                    FONT_SIZE * 1.3,
                    Some(w - STATUS_W - PAD_X * 3.0),
                )
                .with_weight(400),
                x: x + PAD_X + STATUS_W,
                y: item_y + (ITEM_H - FONT_SIZE * 1.3) / 2.0,
                color: if is_selected {
                    theme.text_1
                } else {
                    theme.text_2
                }
                .to_array(),
            });

            hit_rects.push((x, item_y, w, ITEM_H));
        }

        // Scrollbar (if needed)
        if self.scroll.is_scrollable() {
            draw_scrollbar(
                compositor,
                theme,
                x + w - 4.0,
                list_y,
                h - HEADER_H - 1.0,
                &self.scroll,
            );
        }

        self.hit_rects = hit_rects.clone();
        hit_rects
    }
}

fn draw_scrollbar(
    compositor: &mut Compositor,
    theme: &Theme,
    x: f32,
    y: f32,
    h: f32,
    scroll: &ScrollState,
) {
    let track_h = h;
    let thumb_h = (track_h * scroll.thumb_ratio()).max(24.0);
    let thumb_y = y + (track_h - thumb_h) * scroll.thumb_position();
    compositor.push(SceneNode::Rect {
        x,
        y,
        w: 4.0,
        h: track_h,
        color: [0.0, 0.0, 0.0, 0.0],
    });
    compositor.push(SceneNode::Rect {
        x,
        y: thumb_y,
        w: 4.0,
        h: thumb_h,
        color: [theme.text_3.0[0], theme.text_3.0[1], theme.text_3.0[2], 0.5],
    });
}

fn truncate_path(path: &str, max_chars: usize) -> String {
    if path.chars().count() <= max_chars {
        path.to_string()
    } else {
        let start = path.chars().count() - max_chars + 1;
        let s: String = path.chars().skip(start).collect();
        format!("\u{2026}{}", s)
    }
}

fn mock_files() -> Vec<FileEntry> {
    vec![
        FileEntry {
            path: "src/compositor.rs".into(),
            status: FileStatus::Modified,
        },
        FileEntry {
            path: "src/builder.rs".into(),
            status: FileStatus::Modified,
        },
        FileEntry {
            path: "src/scroll.rs".into(),
            status: FileStatus::Added,
        },
        FileEntry {
            path: "src/text.rs".into(),
            status: FileStatus::Modified,
        },
        FileEntry {
            path: "examples/todo_app.rs".into(),
            status: FileStatus::Modified,
        },
        FileEntry {
            path: "Cargo.toml".into(),
            status: FileStatus::Modified,
        },
        FileEntry {
            path: "CLAUDE.md".into(),
            status: FileStatus::Untracked,
        },
        FileEntry {
            path: "src/window.rs".into(),
            status: FileStatus::Modified,
        },
        FileEntry {
            path: "src/gpu.rs".into(),
            status: FileStatus::Modified,
        },
        FileEntry {
            path: "src/signal.rs".into(),
            status: FileStatus::Modified,
        },
        FileEntry {
            path: "shaders/quad.wgsl".into(),
            status: FileStatus::Modified,
        },
        FileEntry {
            path: "shaders/text.wgsl".into(),
            status: FileStatus::Modified,
        },
        FileEntry {
            path: "crates/basicIDE-plev/src/main.rs".into(),
            status: FileStatus::Added,
        },
        FileEntry {
            path: "crates/basicIDE-plev/src/theme.rs".into(),
            status: FileStatus::Added,
        },
        FileEntry {
            path: "assets/fonts/Inter-Regular.ttf".into(),
            status: FileStatus::Added,
        },
        FileEntry {
            path: "mission/readme.md".into(),
            status: FileStatus::Modified,
        },
        FileEntry {
            path: "mission/steps/ongoing/TASK-42.md".into(),
            status: FileStatus::Added,
        },
        FileEntry {
            path: "src/components/button.rs".into(),
            status: FileStatus::Deleted,
        },
        FileEntry {
            path: "src/lib.rs".into(),
            status: FileStatus::Modified,
        },
        FileEntry {
            path: "Cargo.lock".into(),
            status: FileStatus::Modified,
        },
    ]
}
