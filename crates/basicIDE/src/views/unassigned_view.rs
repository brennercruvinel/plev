//! Left "Changes" column — HOFF row-sidebar (rgba(40,40,40,.8)) with a
//! 68px title head and 44px list rows in the Actions-item recipe:
//! radius 16, bg rgba($n2,.02) -> hover .05 -> selected .10 + edge-light;
//! filename in base-2sm at rgba($n2,.56) (.76 selected), status letter in
//! the HOFF accent set, staged marker = 8px #55F08B "new" dot.

use crate::components::badge::{self, BadgeKind};
use crate::components::hoff;
use crate::theme::{StatusColors, Theme};
use plev::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
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
        let s = StatusColors::of(theme);
        match self {
            FileStatus::Modified => s.modified.to_array(),
            FileStatus::Added => s.added.to_array(),
            FileStatus::Deleted => s.deleted.to_array(),
            FileStatus::Renamed => s.renamed.to_array(),
            FileStatus::Untracked => s.untracked.to_array(),
        }
    }
}

/// A file entry in the uncommitted changes list.
#[derive(Clone, Debug)]
pub struct FileEntry {
    pub path: String,
    pub status: FileStatus,
    /// `true` when this change is in the index (shown with a dot marker).
    pub staged: bool,
}

/// State for the left "Unassigned Changes" panel.
pub struct UnassignedView {
    pub files: Vec<FileEntry>,
    pub selected_idx: Option<usize>,
    pub scroll: ScrollState,
    /// Cached hit rects from last render (x, y, w, h) per file row.
    hit_rects: Vec<(f32, f32, f32, f32)>,
}

const HEADER_H: f32 = 68.0;
const ITEM_H: f32 = 44.0;
const ITEM_GAP: f32 = 4.0;
const PAD: f32 = 12.0;
const STATUS_W: f32 = 18.0;
const FONT_SIZE: f32 = 14.0;
const LINE_H: f32 = 14.0 * 1.4;

impl UnassignedView {
    /// Starts empty; the app injects real data via [`set_files`](Self::set_files).
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            selected_idx: None,
            scroll: ScrollState::new(),
            hit_rects: Vec::new(),
        }
    }

    /// Replaces the file list (e.g. from a fresh `git status`), keeping the
    /// selection on the same path when it still exists.
    pub fn set_files(&mut self, files: Vec<FileEntry>) {
        let selected_path = self
            .selected_idx
            .and_then(|i| self.files.get(i))
            .map(|f| f.path.clone());
        self.files = files;
        self.selected_idx =
            selected_path.and_then(|path| self.files.iter().position(|f| f.path == path));
    }

    /// Row hit rects from the last render (test-only accessor; interaction
    /// code goes through [`hit_test`](Self::hit_test)).
    #[cfg(test)]
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
        let content_h = self.files.len() as f32 * (ITEM_H + ITEM_GAP);
        self.scroll.set_viewport(h - HEADER_H);
        self.scroll.set_content(content_h);

        // Column surface — row-sidebar rgba(40,40,40,.8).
        compositor.push(SceneNode::Rect {
            x,
            y,
            w,
            h,
            color: theme.bg_sidebar.to_array(),
        });

        // Head — title (20/500) at .56 + count chip.
        compositor.push(SceneNode::Text {
            key: TextNodeKey::new("Changes", 20.0, 20.0 * 1.2, None).with_weight(500),
            x: x + PAD,
            y: y + (HEADER_H - 20.0 * 1.2) / 2.0,
            color: theme.text_default.to_array(),
        });
        // Count chip — the Tag badge recipe; `tag_width` is the same real
        // measurement `badge::draw` uses, so the right-aligned chip always
        // fits its number.
        let count_str = self.files.len().to_string();
        let chip_w = badge::tag_width(&count_str);
        let chip_h = 22.0;
        let chip_x = x + w - PAD - chip_w;
        let chip_y = y + (HEADER_H - chip_h) / 2.0;
        badge::draw(
            compositor,
            theme,
            chip_x,
            chip_y,
            &count_str,
            BadgeKind::Tag,
        );

        // File list — card rows inset by the 12px body padding. Rows are
        // clipped to the list viewport so scrolled rows never paint over
        // the panel head.
        let list_y = y + HEADER_H;
        let row_x = x + PAD;
        let row_w = w - PAD * 2.0;
        let scroll_offset = self.scroll.offset();
        let mut hit_rects = Vec::with_capacity(self.files.len());
        compositor.push(SceneNode::PushClip {
            x,
            y: list_y,
            w,
            h: h - HEADER_H,
        });

        for (i, file) in self.files.iter().enumerate() {
            let item_y = list_y + i as f32 * (ITEM_H + ITEM_GAP) - scroll_offset;
            // Skip items outside the visible area. Their hit rect must be
            // empty too: a row hidden behind the panel head is not
            // clickable (the vec stays index-aligned with `files`).
            if item_y + ITEM_H < list_y || item_y > y + h {
                hit_rects.push((row_x, item_y, 0.0, 0.0));
                continue;
            }

            let is_selected = self.selected_idx == Some(i);
            let is_hovered = hover_idx == Some(i);

            let row_bg = if is_selected {
                theme.surface_active
            } else if is_hovered {
                theme.surface_hover
            } else {
                theme.surface
            };

            compositor.push(SceneNode::RoundedRect {
                x: row_x,
                y: item_y,
                w: row_w,
                h: ITEM_H,
                color: row_bg.to_array(),
                corner_radius: theme.radius_item,
                border_width: 0.0,
                border_color: [0.0; 4],
            });
            // Edge-light rim: soft on hover, the stronger .10 when selected.
            if is_selected || is_hovered {
                hoff::edge_light(
                    compositor,
                    LayerId::DEFAULT,
                    row_x,
                    item_y,
                    row_w,
                    ITEM_H,
                    theme.radius_item,
                    1.0,
                    if is_selected {
                        theme.edge_strong
                    } else {
                        theme.edge
                    },
                );
            }

            // Status letter — caption-sm in the HOFF accent for the state.
            compositor.push(SceneNode::Text {
                key: TextNodeKey::new(file.status.label(), 12.0, 12.0 * 1.33, None)
                    .with_weight(600),
                x: row_x + PAD,
                y: item_y + (ITEM_H - 12.0 * 1.33) / 2.0,
                color: file.status.color(theme),
            });

            // Filename — base-2sm, .56 at rest, .76 selected.
            let display_name = truncate_path(&file.path, 32);
            compositor.push(SceneNode::Text {
                key: TextNodeKey::new(
                    &display_name,
                    FONT_SIZE,
                    LINE_H,
                    Some(row_w - STATUS_W - PAD * 3.0),
                )
                .with_weight(600),
                x: row_x + PAD + STATUS_W,
                y: item_y + (ITEM_H - LINE_H) / 2.0,
                color: if is_selected || is_hovered {
                    theme.text_active
                } else {
                    theme.text_default
                }
                .to_array(),
            });

            // Staged marker — the 8px green "new" dot.
            if file.staged {
                let dot = 8.0;
                compositor.push(SceneNode::RoundedRect {
                    x: row_x + row_w - PAD - dot,
                    y: item_y + (ITEM_H - dot) / 2.0,
                    w: dot,
                    h: dot,
                    color: theme.accent_green.to_array(),
                    corner_radius: dot / 2.0,
                    border_width: 0.0,
                    border_color: [0.0; 4],
                });
            }

            // Hit rect clamped to the visible part of the row: a row half
            // hidden under the panel head only responds on its visible half.
            let top = item_y.max(list_y);
            let bottom = (item_y + ITEM_H).min(y + h);
            hit_rects.push((row_x, top, row_w, (bottom - top).max(0.0)));
        }
        compositor.push(SceneNode::PopClip);

        // Scrollbar (if needed)
        if self.scroll.is_scrollable() {
            hoff::draw_scrollbar(
                compositor,
                theme,
                x + w - 4.0,
                list_y,
                h - HEADER_H,
                &self.scroll,
            );
        }

        self.hit_rects = hit_rects.clone();
        hit_rects
    }
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
