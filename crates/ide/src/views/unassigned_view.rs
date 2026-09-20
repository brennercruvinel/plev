//! Left "Changes" column: the raised surface with a [`PanelHeader`] and
//! `Md`-tall list rows in the item recipe (item radius, surface wash at
//! rest, hover and selected washes with the edge rim); filename in
//! base-2sm (text-default, text-active when selected), status letter in
//! the theme's intent colors, staged marker as the success dot.

use crate::status::StatusColors;
use comps::action::Badge;
use comps::feedback::Scrollbar;
use comps::nav::PanelHeader;
use comps::prelude::{Rect, edge_light, rounded_rect};
use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::input::scroll::ScrollState;
use engine::text::TextMeasurer;
use engine::theme::{ControlSize, Theme};

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
            FileStatus::Modified => s.modified.0,
            FileStatus::Added => s.added.0,
            FileStatus::Deleted => s.deleted.0,
            FileStatus::Renamed => s.renamed.0,
            FileStatus::Untracked => s.untracked.0,
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
    scrollbar: Scrollbar,
    /// Cached hit rects from last render (x, y, w, h) per file row.
    hit_rects: Vec<(f32, f32, f32, f32)>,
}

impl UnassignedView {
    /// Starts empty; the app injects real data via [`set_files`](Self::set_files).
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            selected_idx: None,
            scroll: ScrollState::new(),
            scrollbar: Scrollbar::new(),
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

    /// Row pitch: one `Md` control plus the `xs` gap.
    fn row_h(theme: &Theme) -> f32 {
        theme.control.height(ControlSize::Md)
    }

    /// Notify the scrollbar of a wheel scroll (it fades in).
    pub fn notify_scroll(&mut self) {
        self.scrollbar.notify_scroll();
    }

    /// Advance the scrollbar fade. `true` while animating.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.scrollbar.tick(dt)
    }

    /// Build and render into a compositor layer.
    /// Returns a list of (x, y, w, h) hit rects for each file row (for click detection).
    // Panel geometry stays flat like every other render fn (card.rs
    // trade-off); a rect bag would be repacked at the call site.
    #[allow(clippy::too_many_arguments)]
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
        let header = PanelHeader::new("Changes").badge(Badge::tag(self.files.len().to_string()));
        let header_h = header.height(theme);
        let row_h = Self::row_h(theme);
        let gap = theme.spacing.xs;
        let pad = theme.spacing.md;
        let content_h = self.files.len() as f32 * (row_h + gap);
        self.scroll.set_viewport(h - header_h);
        self.scroll.set_content(content_h);

        // Column surface.
        compositor.push(SceneNode::Rect {
            x,
            y,
            w,
            h,
            color: theme.colors.surface.0,
        });
        header.render(compositor, Rect::new(x, y, w, header_h), theme);

        // File list: card rows inset by the body padding, clipped to the
        // list viewport so scrolled rows never paint over the panel head.
        let list_y = y + header_h;
        let list = Rect::new(x, list_y, w, h - header_h);
        let row_x = x + pad;
        let row_w = w - pad * 2.0;
        let scroll_offset = self.scroll.offset();
        let mut hit_rects = Vec::with_capacity(self.files.len());
        compositor.push(SceneNode::PushClip {
            x: list.x,
            y: list.y,
            w: list.w,
            h: list.h,
        });

        let status_style = theme.typography.caption_sm();
        let name_style = theme.typography.base_2sm();
        let status_w = theme.control.box_size;
        let dot = theme.spacing.sm;
        for (i, file) in self.files.iter().enumerate() {
            let item_y = list_y + i as f32 * (row_h + gap) - scroll_offset;
            // Skip items outside the visible area. Their hit rect must be
            // empty too: a row hidden behind the panel head is not
            // clickable (the vec stays index-aligned with `files`).
            if item_y + row_h < list_y || item_y > y + h {
                hit_rects.push((row_x, item_y, 0.0, 0.0));
                continue;
            }

            let is_selected = self.selected_idx == Some(i);
            let is_hovered = hover_idx == Some(i);
            let glass = &theme.glass;
            let row_bg = if is_selected {
                glass.surface_active
            } else if is_hovered {
                glass.surface_hover
            } else {
                glass.surface
            };
            let row = Rect::new(row_x, item_y, row_w, row_h);
            compositor.push(rounded_rect(
                row.x,
                row.y,
                row.w,
                row.h,
                theme.shape.item,
                row_bg.0,
            ));
            // Edge-light rim: soft on hover, strong when selected.
            if is_selected || is_hovered {
                edge_light(
                    compositor,
                    LayerId::DEFAULT,
                    row,
                    theme.shape.item,
                    theme.control.edge_width,
                    if is_selected {
                        glass.edge.0
                    } else {
                        glass.edge_soft.0
                    },
                );
            }

            // Status letter in the intent color for the state.
            compositor.push(SceneNode::Text {
                key: TextNodeKey::from_style(file.status.label(), &status_style, None),
                x: row_x + pad,
                y: item_y + TextMeasurer::vertical_center(&status_style, row_h),
                color: file.status.color(theme),
            });

            // Filename, elided to the column width with the same style it
            // is drawn with.
            let name_w = row_w - status_w - pad * 3.0;
            let display_name = TextMeasurer::elide_path(&file.path, &name_style, name_w);
            compositor.push(SceneNode::Text {
                key: TextNodeKey::from_style(&display_name, &name_style, Some(name_w)),
                x: row_x + pad + status_w,
                y: item_y + TextMeasurer::vertical_center(&name_style, row_h),
                color: if is_selected || is_hovered {
                    glass.text_active.0
                } else {
                    glass.text_default.0
                },
            });

            // Staged marker: the success dot.
            if file.staged {
                compositor.push(rounded_rect(
                    row_x + row_w - pad - dot,
                    item_y + (row_h - dot) / 2.0,
                    dot,
                    dot,
                    dot / 2.0,
                    theme.colors.success.0,
                ));
            }

            // Hit rect clamped to the visible part of the row: a row half
            // hidden under the panel head only responds on its visible half.
            let top = item_y.max(list_y);
            let bottom = (item_y + row_h).min(y + h);
            hit_rects.push((row_x, top, row_w, (bottom - top).max(0.0)));
        }
        compositor.push(SceneNode::PopClip);

        self.scrollbar.render(compositor, list, &self.scroll, theme);

        self.hit_rects = hit_rects.clone();
        hit_rects
    }
}
